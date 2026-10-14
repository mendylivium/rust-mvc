pub mod seeders;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{str::FromStr, time::Duration};

pub async fn connect(url: &str) -> anyhow::Result<sqlx::SqlitePool> {
    std::fs::create_dir_all("storage")?;
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    Ok(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?)
}

pub async fn migrate(db: &sqlx::SqlitePool) -> anyhow::Result<()> {
    sqlx::migrate!("./database/migrations").run(db).await?;
    Ok(())
}
