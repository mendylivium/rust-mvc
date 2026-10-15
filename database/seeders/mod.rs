use crate::app::http::{controllers::auth_controller::hash_password, requests::normalize_email};

pub async fn run(db: &sqlx::AnyPool) -> anyhow::Result<()> {
    let email = std::env::var("SEED_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".into());
    let email = normalize_email(&email).map_err(|_| {
        anyhow::anyhow!("SEED_ADMIN_EMAIL must be a valid email address (maximum 254 bytes).")
    })?;
    let password = std::env::var("SEED_ADMIN_PASSWORD").unwrap_or_default();
    anyhow::ensure!(
        (12..=128).contains(&password.len()),
        "Set SEED_ADMIN_PASSWORD to a password of 12–128 bytes in .env before seeding."
    );
    let password = hash_password(password).await?;
    let mut tx = db.begin().await?;
    crate::database::auth_lock(&mut tx).await?;
    let exists: i64 = sqlx::query_scalar(&crate::database::sql(
        db,
        "SELECT COUNT(*) FROM users WHERE email = ?",
    ))
    .bind(&email)
    .fetch_one(&mut *tx)
    .await?;
    if exists == 0 {
        sqlx::query(&crate::database::sql(db, "INSERT INTO users (name, email, password, role) VALUES ('Workspace Admin', ?, ?, 'admin')"))
            .bind(&email).bind(password).execute(&mut *tx).await?;
        println!("Created administrator: {email}");
    } else {
        println!("Account already exists; no changes made.");
    }
    tx.commit().await?;
    Ok(())
}
