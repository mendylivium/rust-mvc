use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rust_mvc::{
    AppState, app::http::controllers::auth_controller::hash_password, application, config::Config,
    database,
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn app() -> Router {
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    database::migrate(&db).await.unwrap();
    let hash = hash_password("test-password-123".into()).await.unwrap();
    for (name, email, role) in [
        ("Admin", "admin@example.com", "admin"),
        ("Member", "member@example.com", "member"),
    ] {
        sqlx::query("INSERT INTO users (name, email, password, role) VALUES (?, ?, ?, ?)")
            .bind(name)
            .bind(email)
            .bind(&hash)
            .bind(role)
            .execute(&db)
            .await
            .unwrap();
    }
    application::router(AppState {
        db,
        config: Config {
            name: "Test".into(),
            host: "127.0.0.1".into(),
            port: 3000,
            database_url: "sqlite::memory:".into(),
            cookie_secure: false,
        },
    })
}

#[derive(Default)]
struct Client {
    cookie: String,
    csrf: String,
}
impl Client {
    async fn call(
        &mut self,
        app: &Router,
        method: &str,
        path: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json");
        if !self.cookie.is_empty() {
            builder = builder.header("cookie", &self.cookie);
        }
        if !self.csrf.is_empty() {
            builder = builder.header("x-csrf-token", &self.csrf);
        }
        let response = app
            .clone()
            .oneshot(
                builder
                    .body(if body.is_null() {
                        Body::empty()
                    } else {
                        Body::from(body.to_string())
                    })
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        if let Some(cookie) = response.headers().get("set-cookie") {
            self.cookie = cookie
                .to_str()
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .to_owned();
        }
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        if let Some(csrf) = value["csrf_token"].as_str() {
            self.csrf = csrf.into();
        }
        (status, value)
    }
    async fn login(&mut self, app: &Router, email: &str) {
        assert_eq!(
            self.call(app, "GET", "/api/auth/session", Value::Null)
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            self.call(
                app,
                "POST",
                "/api/auth/login",
                json!({"email":email,"password":"test-password-123"})
            )
            .await
            .0,
            StatusCode::OK
        );
    }
}

#[tokio::test]
async fn authentication_csrf_rotation_and_logout() {
    let app = app().await;
    let mut client = Client::default();
    assert_eq!(
        client.call(&app, "GET", "/api/users", Value::Null).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .call(
                &app,
                "POST",
                "/api/auth/login",
                json!({"email":"admin@example.com","password":"test-password-123"})
            )
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    client
        .call(&app, "GET", "/api/auth/session", Value::Null)
        .await;
    let old_cookie = client.cookie.clone();
    let old_csrf = client.csrf.clone();
    client.login(&app, "admin@example.com").await;
    assert_ne!(client.cookie, old_cookie);
    assert_ne!(client.csrf, old_csrf);
    let (status, user) = client
        .call(&app, "GET", "/api/auth/session", Value::Null)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(user["user"].get("password").is_none());
    let mut stale = Client {
        cookie: old_cookie,
        csrf: old_csrf,
    };
    assert_eq!(
        stale.call(&app, "GET", "/api/users", Value::Null).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        client
            .call(&app, "POST", "/api/auth/logout", json!({}))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        client.call(&app, "GET", "/api/users", Value::Null).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn members_cannot_manage_users_but_can_edit_profile() {
    let app = app().await;
    let mut client = Client::default();
    client.login(&app, "member@example.com").await;
    for path in ["/api/users", "/api/dashboard"] {
        assert_eq!(
            client.call(&app, "GET", path, Value::Null).await.0,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        client
            .call(&app, "DELETE", "/api/users/1", Value::Null)
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(client.call(&app, "POST", "/api/users", json!({"name":"No Access", "email":"no@example.com","password":"test-password-123","role":"admin","status":"active"})).await.0, StatusCode::FORBIDDEN);
    let (_, profile) = client
        .call(
            &app,
            "PATCH",
            "/api/profile",
            json!({"name":"New Name","role":"admin"}),
        )
        .await;
    assert_eq!(profile["user"]["name"], "New Name");
    assert_eq!(profile["user"]["role"], "member");
}

#[tokio::test]
async fn users_crud_validation_search_and_self_protection() {
    let app = app().await;
    let mut client = Client::default();
    client.login(&app, "admin@example.com").await;
    let mut input = json!({"name":"New Person", "email":"person@example.com", "password":"test-password-123", "role":"member", "status":"active"});
    let (status, created) = client.call(&app, "POST", "/api/users", input.clone()).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["data"]["id"].as_i64().unwrap();
    assert!(created["data"].get("password").is_none());
    assert_eq!(
        client
            .call(&app, "POST", "/api/users", input.clone())
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let (_, search) = client
        .call(
            &app,
            "GET",
            "/api/users?search=person&page=999",
            Value::Null,
        )
        .await;
    assert_eq!(search["meta"]["total"], 1);
    assert_eq!(search["meta"]["page"], 1);
    input["name"] = json!("Updated Person");
    input["password"] = json!("");
    assert_eq!(
        client
            .call(&app, "PUT", &format!("/api/users/{id}"), input.clone())
            .await
            .0,
        StatusCode::OK
    );
    input["role"] = json!("owner");
    assert_eq!(
        client
            .call(&app, "PUT", &format!("/api/users/{id}"), input.clone())
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    input["role"] = json!("member");
    assert_eq!(
        client.call(&app, "PUT", "/api/users/1", input).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        client
            .call(&app, "DELETE", "/api/users/1", Value::Null)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        client
            .call(&app, "DELETE", &format!("/api/users/{id}"), Value::Null)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        client
            .call(&app, "DELETE", &format!("/api/users/{id}"), Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let (_, dashboard) = client
        .call(&app, "GET", "/api/dashboard", Value::Null)
        .await;
    assert_eq!(dashboard["stats"]["total"], 2);
    assert_eq!(dashboard["growth"].as_array().unwrap().len(), 6);
}

#[tokio::test]
async fn disabling_account_revokes_existing_sessions() {
    let app = app().await;
    let mut admin = Client::default();
    let mut member = Client::default();
    admin.login(&app, "admin@example.com").await;
    member.login(&app, "member@example.com").await;
    assert_eq!(admin.call(&app, "PUT", "/api/users/2", json!({"name":"Member","email":"member@example.com","password":"","role":"member","status":"inactive"})).await.0, StatusCode::OK);
    assert_eq!(
        member
            .call(
                &app,
                "PATCH",
                "/api/profile",
                json!({"name":"Still logged in?"})
            )
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        member
            .call(&app, "GET", "/api/auth/session", Value::Null)
            .await
            .1["user"],
        Value::Null
    );
}

#[tokio::test]
async fn login_throttling_and_unknown_api_routes() {
    let app = app().await;
    let mut client = Client::default();
    client
        .call(&app, "GET", "/api/auth/session", Value::Null)
        .await;
    for _ in 0..5 {
        assert_eq!(
            client
                .call(
                    &app,
                    "POST",
                    "/api/auth/login",
                    json!({"email":"admin@example.com","password":"wrong"})
                )
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        client
            .call(
                &app,
                "POST",
                "/api/auth/login",
                json!({"email":"admin@example.com","password":"test-password-123"})
            )
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        client
            .call(&app, "GET", "/api/missing", Value::Null)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}
