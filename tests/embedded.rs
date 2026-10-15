#![cfg(feature = "embedded-assets")]
use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use rust_mvc::{AppState, config::Config, database};
use tower::ServiceExt;

#[tokio::test]
async fn embedded_pages_assets_and_status_codes() {
    let db = database::connect("sqlite::memory:").await.unwrap();
    let app = rust_mvc::application::router(AppState {
        db,
        config: Config {
            name: "Embedded".into(),
            host: "127.0.0.1".into(),
            port: 0,
            database_url: "sqlite::memory:".into(),
            cookie_secure: false,
        },
        security: Default::default(),
    });
    for path in [
        "/",
        "/login",
        "/users",
        "/settings",
        "/guide",
        "/examples/tables",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], "text/html");
        let html = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("<div id=\"app\"></div>"));
        if path == "/" {
            let asset = html
                .split("src=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            let response = app
                .clone()
                .oneshot(Request::builder().uri(asset).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), 200);
            assert!(
                response.headers()["content-type"]
                    .to_str()
                    .unwrap()
                    .contains("javascript")
            );
        }
    }
    for path in [
        "/missing-page",
        "/assets/missing.js",
        "/api/missing",
        "/.env",
    ] {
        assert_eq!(
            app.clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap()
                .status(),
            404
        );
    }
}
