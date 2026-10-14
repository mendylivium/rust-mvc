use std::env;

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            name: env::var("APP_NAME").unwrap_or_else(|_| "RustMVC".into()),
            host: env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("APP_PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://storage/database.sqlite".into()),
            cookie_secure: env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "false".into())
                .parse()?,
        })
    }
}
