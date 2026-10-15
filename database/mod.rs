pub mod seeders;
use sqlx::{Any, AnyPool, Transaction, any::AnyPoolOptions};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Driver {
    Sqlite,
    Mysql,
    Postgres,
}

pub fn driver(db: &AnyPool) -> Driver {
    match db.connect_options().database_url.scheme() {
        "mysql" => Driver::Mysql,
        "postgres" | "postgresql" => Driver::Postgres,
        _ => Driver::Sqlite,
    }
}

pub async fn connect(url: &str) -> anyhow::Result<AnyPool> {
    sqlx::any::install_default_drivers();
    let sqlite = url.starts_with("sqlite:");
    if sqlite {
        std::fs::create_dir_all("storage")?;
    }
    let url = if sqlite && !url.contains(":memory:") && !url.contains("mode=") {
        format!("{url}{}mode=rwc", if url.contains('?') { '&' } else { '?' })
    } else {
        url.to_owned()
    };
    Ok(AnyPoolOptions::new()
        .max_connections(if url.contains(":memory:") { 1 } else { 5 })
        .acquire_timeout(Duration::from_secs(10))
        .after_connect(move |connection, _| {
            Box::pin(async move {
                if sqlite {
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query("PRAGMA busy_timeout = 5000")
                        .execute(&mut *connection)
                        .await?;
                }
                Ok(())
            })
        })
        .connect(&url)
        .await?)
}

pub async fn migrate(db: &AnyPool) -> anyhow::Result<()> {
    match driver(db) {
        Driver::Sqlite => sqlx::migrate!("./database/migrations").run(db).await?,
        Driver::Mysql => {
            sqlx::migrate!("./database/migrations/mysql")
                .run(db)
                .await?
        }
        Driver::Postgres => {
            sqlx::migrate!("./database/migrations/pgsql")
                .run(db)
                .await?
        }
    }
    Ok(())
}

/// Serialize bounded auth storage and credential/session writes across processes.
/// Lock before reading auth state; SQLite obtains a write lock, server DBs a row lock.
pub async fn auth_lock(tx: &mut Transaction<'_, Any>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE auth_lock SET id = id WHERE id = 1")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Translate this application's static SQL, never user input. Quoted question marks
/// remain literals; PostgreSQL requires numbered bind parameters.
pub fn sql(db: &AnyPool, statement: &str) -> String {
    translate(driver(db), statement)
}
fn translate(driver: Driver, statement: &str) -> String {
    let clock = match driver {
        Driver::Sqlite => "unixepoch()",
        Driver::Mysql => "UNIX_TIMESTAMP()",
        Driver::Postgres => "CAST(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) AS BIGINT)",
    };
    let statement = statement.replace("unixepoch()", clock);
    if driver != Driver::Postgres {
        return statement;
    }
    let mut result = String::new();
    let mut quoted = false;
    let mut index = 0;
    for c in statement.chars() {
        if c == '\'' {
            quoted = !quoted;
        }
        if c == '?' && !quoted {
            index += 1;
            result.push_str(&format!("${index}"));
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn postgres_parameters_preserve_literals() {
        assert_eq!(
            translate(Driver::Postgres, "SELECT '?' WHERE a = ? AND b = ?"),
            "SELECT '?' WHERE a = $1 AND b = $2"
        );
    }
}
