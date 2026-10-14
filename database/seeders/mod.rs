use crate::app::http::controllers::auth_controller::hash_password;

pub async fn run(db: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let email = std::env::var("SEED_ADMIN_EMAIL")
        .unwrap_or_else(|_| "admin@example.com".into())
        .trim()
        .to_lowercase();
    let password = std::env::var("SEED_ADMIN_PASSWORD").unwrap_or_default();
    anyhow::ensure!(
        (12..=128).contains(&password.len()),
        "Set SEED_ADMIN_PASSWORD to a password of 12–128 bytes in .env before seeding."
    );
    let password = hash_password(password).await?;
    let result = sqlx::query("INSERT INTO users (name, email, password, role) VALUES ('Workspace Admin', ?, ?, 'admin') ON CONFLICT(email) DO NOTHING")
        .bind(&email).bind(password).execute(db).await?;
    if result.rows_affected() == 0 {
        println!("Account already exists; no changes made.");
    } else {
        println!("Created administrator: {email}");
    }
    Ok(())
}
