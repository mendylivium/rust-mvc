#[path = "../app/mod.rs"]
pub mod app;
pub mod application;
#[path = "../config/mod.rs"]
pub mod config;
#[path = "../database/mod.rs"]
pub mod database;
#[path = "../routes/mod.rs"]
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: config::Config,
}
