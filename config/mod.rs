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
            database_url: database_url(|key| env::var(key).ok())?,
            cookie_secure: env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "false".into())
                .parse()?,
        })
    }
}

// DATABASE_URL is an optional explicit override for hosting providers and legacy setups.
pub fn database_url(get: impl Fn(&str) -> Option<String>) -> anyhow::Result<String> {
    if let Some(url) = get("DATABASE_URL").filter(|value| !value.is_empty()) {
        anyhow::ensure!(
            url.starts_with("sqlite:")
                || url.starts_with("mysql://")
                || url.starts_with("postgres://")
                || url.starts_with("postgresql://"),
            "Unsupported DATABASE_URL scheme"
        );
        return Ok(url);
    }
    let connection = get("DB_CONNECTION").unwrap_or_else(|| "sqlite".into());
    if connection == "sqlite" {
        let path = get("DB_DATABASE")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "storage/database.sqlite".into());
        return Ok(if path == ":memory:" {
            "sqlite::memory:".into()
        } else {
            format!("sqlite://{path}")
        });
    }
    let (scheme, port) = match connection.as_str() {
        "mysql" => ("mysql", 3306),
        "pgsql" | "postgres" | "postgresql" => ("postgres", 5432),
        _ => anyhow::bail!("DB_CONNECTION must be sqlite, mysql, or pgsql"),
    };
    let mut url = url::Url::parse(&format!("{scheme}://localhost"))?;
    url.set_host(Some(&get("DB_HOST").unwrap_or_else(|| "127.0.0.1".into())))?;
    let port = get("DB_PORT")
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u16>())
        .transpose()?
        .unwrap_or(port);
    url.set_port(Some(port))
        .map_err(|_| anyhow::anyhow!("Invalid DB_PORT"))?;
    url.set_username(
        &get("DB_USERNAME")
            .unwrap_or_else(|| "rustmvc".into())
            .replace('%', "%25"),
    )
    .map_err(|_| anyhow::anyhow!("Invalid DB_USERNAME"))?;
    url.set_password(Some(
        &get("DB_PASSWORD").unwrap_or_default().replace('%', "%25"),
    ))
    .map_err(|_| anyhow::anyhow!("Invalid DB_PASSWORD"))?;
    let database = get("DB_DATABASE")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "rustmvc".into());
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("Invalid database URL"))?
        .push(&database);
    if let Some(mode) = get("DB_SSLMODE").filter(|s| !s.is_empty()) {
        url.query_pairs_mut().append_pair(
            if scheme == "postgres" {
                "sslmode"
            } else {
                "ssl-mode"
            },
            &mode,
        );
    }
    Ok(url.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn database_settings() {
        let get = |key: &str| match key {
            "DB_CONNECTION" => Some("pgsql".into()),
            "DB_PASSWORD" => Some("p@ss:/?#%".into()),
            _ => None,
        };
        let url = database_url(get).unwrap();
        assert!(url.starts_with("postgres://rustmvc:p%40ss%3A%2F%3F%23%25@127.0.0.1:5432/rustmvc"));
        assert_eq!(
            database_url(|_| None).unwrap(),
            "sqlite://storage/database.sqlite"
        );
        assert!(database_url(|key| (key == "DB_CONNECTION").then(|| "unknown".into())).is_err());
    }
    #[test]
    fn mysql_defaults_and_url_precedence() {
        let mysql = database_url(|key| match key {
            "DB_CONNECTION" => Some("mysql".into()),
            "DB_PORT" => Some(String::new()),
            "DB_SSLMODE" => Some("verify_identity".into()),
            _ => None,
        })
        .unwrap();
        let url = url::Url::parse(&mysql).unwrap();
        assert_eq!(url.port(), Some(3306));
        assert_eq!(url.path(), "/rustmvc");
        assert_eq!(url.query(), Some("ssl-mode=verify_identity"));
        assert_eq!(
            database_url(|key| match key {
                "DATABASE_URL" => Some("sqlite::memory:".into()),
                "DB_CONNECTION" => Some("mysql".into()),
                _ => None,
            })
            .unwrap(),
            "sqlite::memory:"
        );
        assert!(
            database_url(|key| match key {
                "DB_CONNECTION" => Some("pgsql".into()),
                "DB_PORT" => Some("not-a-port".into()),
                _ => None,
            })
            .is_err()
        );
    }
}
