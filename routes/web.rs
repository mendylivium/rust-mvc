use crate::AppState;
use axum::Router;
#[cfg(not(feature = "embedded-assets"))]
use tower_http::services::{ServeDir, ServeFile};

#[cfg(not(feature = "embedded-assets"))]
pub fn routes() -> Router<AppState> {
    Router::new()
        .route_service("/", ServeFile::new("dist/index.html"))
        .route_service("/login", ServeFile::new("dist/index.html"))
        .route_service("/users", ServeFile::new("dist/index.html"))
        .route_service("/settings", ServeFile::new("dist/index.html"))
        .route_service("/guide", ServeFile::new("dist/index.html"))
        .route_service("/examples/elements", ServeFile::new("dist/index.html"))
        .route_service("/examples/cards", ServeFile::new("dist/index.html"))
        .route_service("/examples/tables", ServeFile::new("dist/index.html"))
        .route_service("/examples/forms", ServeFile::new("dist/index.html"))
        .fallback_service(
            ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
        )
}

#[cfg(feature = "embedded-assets")]
#[derive(rust_embed::RustEmbed)]
#[folder = "dist/"]
struct Assets;

#[cfg(feature = "embedded-assets")]
pub fn routes() -> Router<AppState> {
    Router::new().fallback(embedded)
}

#[cfg(feature = "embedded-assets")]
async fn embedded(method: axum::http::Method, uri: axum::http::Uri) -> axum::response::Response {
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    if method != axum::http::Method::GET && method != axum::http::Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let path = uri.path();
    let page = matches!(
        path,
        "/" | "/login"
            | "/users"
            | "/settings"
            | "/guide"
            | "/examples/elements"
            | "/examples/cards"
            | "/examples/tables"
            | "/examples/forms"
    );
    let asset = if page {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    let (status, file) = match Assets::get(asset) {
        Some(file) => (StatusCode::OK, Some((asset, file))),
        None if !path.starts_with("/assets/")
            && !path.rsplit('/').next().unwrap_or("").contains('.') =>
        {
            (
                StatusCode::NOT_FOUND,
                Assets::get("index.html").map(|file| ("index.html", file)),
            )
        }
        None => (StatusCode::NOT_FOUND, None),
    };
    match file {
        Some((name, file)) => {
            let mime = mime_guess::from_path(name)
                .first_or_octet_stream()
                .to_string();
            let length = file.data.len().to_string();
            let body = if method == axum::http::Method::HEAD {
                Vec::new()
            } else {
                file.data.into_owned()
            };
            (
                status,
                [
                    (header::CONTENT_TYPE, mime),
                    (header::CONTENT_LENGTH, length),
                ],
                body,
            )
                .into_response()
        }
        None => status.into_response(),
    }
}
