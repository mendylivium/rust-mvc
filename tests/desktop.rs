use rust_mvc::{
    AppState,
    config::Config,
    database,
    desktop::{ApiRequest, DesktopBackend},
};
use serde_json::{Value, json};

#[tokio::test]
async fn ipc_workspace_setup_authentication_and_csrf_without_http_listener() {
    let db = database::connect("sqlite::memory:").await.unwrap();
    database::migrate(&db).await.unwrap();
    let state = AppState {
        db,
        config: Config {
            name: "Desktop test".into(),
            host: "127.0.0.1".into(),
            port: 0,
            database_url: "sqlite::memory:".into(),
            cookie_secure: false,
        },
        security: Default::default(),
    };
    let backend = DesktopBackend::new(state);
    async fn call(
        backend: &DesktopBackend,
        method: &str,
        path: &str,
        body: Option<Value>,
        csrf: Option<&str>,
    ) -> rust_mvc::desktop::ApiResponse {
        backend
            .request(ApiRequest {
                method: method.into(),
                path: path.into(),
                body,
                csrf: csrf.map(str::to_owned),
            })
            .await
            .unwrap()
    }
    assert!(
        backend
            .request(ApiRequest {
                method: "GET".into(),
                path: "http://example.com/api/users".into(),
                body: None,
                csrf: None
            })
            .await
            .is_err()
    );
    let status = call(&backend, "GET", "/api/desktop/setup", None, None).await;
    assert_eq!(status.data["needs_setup"], true);
    let invalid = call(
        &backend,
        "POST",
        "/api/desktop/setup",
        Some(json!({"name":"Admin", "email":"bad", "password":"short"})),
        None,
    )
    .await;
    assert_eq!(invalid.status, 422);
    let credentials = json!({"name":"Desktop Admin", "email":"desktop@example.com", "password":"desktop-password-123"});
    assert_eq!(
        call(
            &backend,
            "POST",
            "/api/desktop/setup",
            Some(credentials.clone()),
            None
        )
        .await
        .status,
        201
    );
    assert_eq!(
        call(
            &backend,
            "POST",
            "/api/desktop/setup",
            Some(credentials.clone()),
            None
        )
        .await
        .status,
        409
    );
    let session = call(&backend, "GET", "/api/auth/session", None, None).await;
    let csrf = session.data["csrf_token"].as_str().unwrap();
    assert_eq!(
        call(
            &backend,
            "POST",
            "/api/auth/login",
            Some(credentials.clone()),
            None
        )
        .await
        .status,
        403
    );
    let login = call(
        &backend,
        "POST",
        "/api/auth/login",
        Some(credentials),
        Some(csrf),
    )
    .await;
    assert_eq!(login.status, 200);
    let csrf = login.data["csrf_token"].as_str().unwrap();
    assert_eq!(
        call(&backend, "GET", "/api/dashboard", None, None)
            .await
            .status,
        200
    );
    assert_eq!(
        call(
            &backend,
            "PATCH",
            "/api/profile",
            Some(json!({"name":"Updated Desktop"})),
            Some(csrf)
        )
        .await
        .status,
        200
    );
    assert_eq!(
        call(&backend, "GET", "/api/auth/session", None, None)
            .await
            .data["user"]["name"],
        "Updated Desktop"
    );
    assert_eq!(
        call(&backend, "POST", "/api/auth/logout", None, Some(csrf))
            .await
            .status,
        200
    );
    assert_eq!(
        call(&backend, "GET", "/api/users", None, None).await.status,
        401
    );
}

#[tokio::test]
async fn desktop_setup_is_not_exposed_by_server_router() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    let db = database::connect("sqlite::memory:").await.unwrap();
    database::migrate(&db).await.unwrap();
    let router = rust_mvc::application::router(AppState {
        db,
        config: Config {
            name: "Server".into(),
            host: "127.0.0.1".into(),
            port: 0,
            database_url: "sqlite::memory:".into(),
            cookie_secure: false,
        },
        security: Default::default(),
    });
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/desktop/setup")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 404);
}
