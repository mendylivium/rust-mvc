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

async fn state() -> AppState {
    let db = if let Ok(base) = std::env::var("TEST_DATABASE_URL") {
        // Only use a disposable test server: each test creates its own database.
        let admin = database::connect(&base).await.unwrap();
        let name = format!("rustmvc_test_{}", uuid::Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE DATABASE {name}"))
            .execute(&admin)
            .await
            .unwrap();
        admin.close().await;
        let mut url = url::Url::parse(&base).unwrap();
        url.set_path(&name);
        database::connect(url.as_str()).await.unwrap()
    } else {
        database::connect("sqlite::memory:").await.unwrap()
    };
    database::migrate(&db).await.unwrap();
    database::migrate(&db).await.unwrap();
    let hash = hash_password("test-password-123".into()).await.unwrap();
    for (name, email, role) in [
        ("Admin", "admin@example.com", "admin"),
        ("Member", "member@example.com", "member"),
    ] {
        sqlx::query(&database::sql(
            &db,
            "INSERT INTO users (name, email, password, role) VALUES (?, ?, ?, ?)",
        ))
        .bind(name)
        .bind(email)
        .bind(&hash)
        .bind(role)
        .execute(&db)
        .await
        .unwrap();
    }
    AppState {
        security: Default::default(),
        db,
        config: Config {
            name: "Test".into(),
            host: "127.0.0.1".into(),
            port: 3000,
            database_url: "sqlite::memory:".into(),
            cookie_secure: false,
        },
    }
}

async fn app() -> Router {
    application::router(state().await)
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
        StatusCode::UNAUTHORIZED
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

#[tokio::test]
async fn password_reset_serializes_with_session_issuance() {
    use rust_mvc::app::{http::middleware::session, models::user::User};
    let state = state().await;
    let app = application::router(state.clone());
    let mut admin = Client::default();
    admin.login(&app, "admin@example.com").await;
    let anonymous = session::create(&state).await.unwrap();
    let verified = User::find(&state.db, 2).await.unwrap().unwrap();
    let (resume, wait) = tokio::sync::oneshot::channel();
    let login_state = state.clone();
    // Pause after successful verification of the old credential, before session issuance.
    let login = tokio::spawn(async move {
        wait.await.unwrap();
        session::authenticate(&login_state, &verified, &anonymous.id).await
    });
    assert_eq!(admin.call(&app, "PUT", "/api/users/2", json!({"name":"Member","email":"member@example.com","password":"replacement-password-123","role":"member","status":"active"})).await.0, StatusCode::OK);
    resume.send(()).unwrap();
    assert!(
        login.await.unwrap().is_err(),
        "stale credentials must not issue a session"
    );
    let count: i64 = sqlx::query_scalar(&database::sql(
        &state.db,
        "SELECT count(*) FROM sessions WHERE user_id = 2",
    ))
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(count, 0);
    // The opposite ordering must revoke a session issued before the reset.
    let anonymous = session::create(&state).await.unwrap();
    let verified = User::find(&state.db, 2).await.unwrap().unwrap();
    let (session, _) = session::authenticate(&state, &verified, &anonymous.id)
        .await
        .unwrap();
    assert_eq!(admin.call(&app, "PUT", "/api/users/2", json!({"name":"Member","email":"member@example.com","password":"another-password-123","role":"member","status":"active"})).await.0, StatusCode::OK);
    let count: i64 = sqlx::query_scalar(&database::sql(
        &state.db,
        "SELECT count(*) FROM sessions WHERE id = ?",
    ))
    .bind(session.id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn admission_bounds_password_work_and_rotating_emails() {
    let state = state().await;
    let app = application::router(state.clone());
    let permits: Vec<_> = (0..4).map(|_| state.security.password().unwrap()).collect();
    let mut client = Client::default();
    client
        .call(&app, "GET", "/api/auth/session", Value::Null)
        .await;
    let result = client
        .call(
            &app,
            "POST",
            "/api/auth/login",
            json!({"email":"unknown@example.com","password":"wrong"}),
        )
        .await;
    assert_eq!(result.0, StatusCode::TOO_MANY_REQUESTS);
    drop(permits);
    assert_eq!(
        client
            .call(
                &app,
                "POST",
                "/api/auth/login",
                json!({"email":"unknown@example.com","password":"wrong"})
            )
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for n in 2..30 {
        assert_eq!(
            client
                .call(
                    &app,
                    "POST",
                    "/api/auth/login",
                    json!({"email":format!("rotate-{n}@example.com"),"password":"x".repeat(129)})
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
                json!({"email":"fresh@example.com","password":"wrong"})
            )
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    let count: i64 = sqlx::query_scalar(&database::sql(
        &state.db,
        "SELECT count(*) FROM login_attempts",
    ))
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(count, 1, "rejected admission must not create counters");
}

#[tokio::test]
async fn anonymous_session_rate_and_storage_are_bounded() {
    let state = state().await;
    let app = application::router(state.clone());
    for batch in 0..100 {
        let values = (batch * 100..(batch + 1) * 100)
            .map(|n| format!("('{n}', 'csrf', NULL, unixepoch()+60)"))
            .collect::<Vec<_>>()
            .join(",");
        sqlx::query(&database::sql(
            &state.db,
            &format!("INSERT INTO sessions (id, csrf, user_id, expires_at) VALUES {values}"),
        ))
        .execute(&state.db)
        .await
        .unwrap();
    }
    let mut client = Client::default();
    assert_eq!(
        client
            .call(&app, "GET", "/api/auth/session", Value::Null)
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    sqlx::query(&database::sql(
        &state.db,
        "UPDATE sessions SET expires_at = 0",
    ))
    .execute(&state.db)
    .await
    .unwrap();
    assert_eq!(
        client
            .call(&app, "GET", "/api/auth/session", Value::Null)
            .await
            .0,
        StatusCode::OK
    );
    for _ in 2..60 {
        let mut anonymous = Client::default();
        assert_eq!(
            anonymous
                .call(&app, "GET", "/api/auth/session", Value::Null)
                .await
                .0,
            StatusCode::OK
        );
    }
    assert_eq!(
        Client::default()
            .call(&app, "GET", "/api/auth/session", Value::Null)
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        client
            .call(&app, "GET", "/api/auth/session", Value::Null)
            .await
            .0,
        StatusCode::OK,
        "existing sessions do not consume creation quota"
    );
}

#[tokio::test]
async fn malformed_requests_keep_the_json_contract() {
    let app = app().await;
    let mut client = Client::default();
    client.login(&app, "admin@example.com").await;
    for (method, path, body, expected) in [
        (
            "GET",
            "/api/users?page=abc",
            Value::Null,
            StatusCode::BAD_REQUEST,
        ),
        (
            "POST",
            "/api/users",
            json!({"name":"Incomplete"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "DELETE",
            "/api/users/not-an-id",
            Value::Null,
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let (status, value) = client.call(&app, method, path, body).await;
        assert_eq!(status, expected);
        assert!(value["message"].is_string());
        assert!(value["errors"].is_object());
    }
    for (body, expected) in [
        ("{".to_owned(), StatusCode::BAD_REQUEST),
        ("x".repeat(17000), StatusCode::PAYLOAD_TOO_LARGE),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/users")
                    .header("content-type", "application/json")
                    .header("cookie", &client.cookie)
                    .header("x-csrf-token", &client.csrf)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
        assert_eq!(response.headers()["content-type"], "application/json");
        let value: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert!(value["message"].is_string());
    }
}

#[tokio::test]
async fn attempt_storage_is_capped_and_expired_rows_are_reclaimed() {
    let state = state().await;
    let app = application::router(state.clone());
    let mut client = Client::default();
    client
        .call(&app, "GET", "/api/auth/session", Value::Null)
        .await;
    for batch in 0..100 {
        let values = (batch * 100..(batch + 1) * 100)
            .map(|n| format!("('fixture-{n}', 1, unixepoch())"))
            .collect::<Vec<_>>()
            .join(",");
        sqlx::query(&database::sql(
            &state.db,
            &format!("INSERT INTO login_attempts (email, attempts, started_at) VALUES {values}"),
        ))
        .execute(&state.db)
        .await
        .unwrap();
    }
    let attempt = json!({"email":"unknown@example.com","password":"wrong"});
    assert_eq!(
        client
            .call(&app, "POST", "/api/auth/login", attempt.clone())
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    sqlx::query(&database::sql(
        &state.db,
        "UPDATE login_attempts SET started_at = unixepoch() - 901",
    ))
    .execute(&state.db)
    .await
    .unwrap();
    for _ in 0..5 {
        assert_eq!(
            client
                .call(&app, "POST", "/api/auth/login", attempt.clone())
                .await
                .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        client
            .call(&app, "POST", "/api/auth/login", attempt)
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS,
        "unknown and registered accounts use the same throttle"
    );
    let count: i64 = sqlx::query_scalar(&database::sql(
        &state.db,
        "SELECT count(*) FROM login_attempts",
    ))
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn logout_works_when_anonymous_session_creation_is_throttled() {
    let state = state().await;
    let app = application::router(state.clone());
    let mut client = Client::default();
    client.login(&app, "admin@example.com").await;
    for _ in 1..60 {
        state.security.session().unwrap();
    }
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
async fn server_database_cli_seeding_is_idempotent() {
    // SQLite CLI seeding is covered in tests/console.rs (a file database).
    if std::env::var("TEST_DATABASE_URL").is_err() {
        return;
    }
    let state = state().await;
    let url = state.db.connect_options().database_url.to_string();
    let run = |email: &str| {
        std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
            .env("DATABASE_URL", &url)
            .env("SEED_ADMIN_EMAIL", email)
            .env("SEED_ADMIN_PASSWORD", "test-password-123")
            .args(["migrate", "--seed"])
            .output()
            .unwrap()
    };
    assert!(run(" New-Admin@Example.com ").status.success());
    assert!(run("new-admin@example.com").status.success());
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = 'new-admin@example.com'")
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(count, 1);
    let app = application::router(state);
    let mut client = Client::default();
    client.login(&app, "new-admin@example.com").await;
}
