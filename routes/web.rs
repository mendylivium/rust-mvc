use crate::AppState;
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

pub fn routes() -> Router<AppState> {
    Router::new().fallback_service(
        ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
    )
}
