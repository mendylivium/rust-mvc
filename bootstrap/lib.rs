#[path = "../app/mod.rs"]
pub mod app;
pub mod application;
#[path = "../config/mod.rs"]
pub mod config;
#[path = "../database/mod.rs"]
pub mod database;
#[path = "../desktop/backend.rs"]
pub mod desktop;
#[path = "../routes/mod.rs"]
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::AnyPool,
    pub config: config::Config,
    pub security: app::http::middleware::security::Security,
}
