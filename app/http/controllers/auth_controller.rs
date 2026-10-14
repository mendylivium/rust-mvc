use crate::{
    AppState,
    app::{
        http::{
            error::{AppError, Result},
            middleware::session::{self, AuthUser},
            requests::LoginRequest,
        },
        models::user::User,
    },
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use serde_json::{Value, json};

pub async fn hash_password(password: String) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|hash| hash.to_string())
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    })
    .await?
}
pub async fn current(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<Value>)> {
    let session = match session::load(&state.db, &jar).await? {
        Some(session) => session,
        None => session::create(&state, None).await?,
    };
    let user = match session.user_id {
        Some(id) => User::find(&state.db, id)
            .await?
            .filter(|u| u.status == "active"),
        None => None,
    };
    Ok((
        jar.add(session::cookie(session.id, state.config.cookie_secure)),
        Json(json!({"user":user,"csrf_token":session.csrf,"app_name":state.config.name})),
    ))
}
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<LoginRequest>,
) -> Result<(CookieJar, Json<Value>)> {
    let email = input.email.trim().to_lowercase();
    if email.len() > 254 || input.password.len() > 128 {
        return Err(AppError::validation("email", "Invalid credentials."));
    }
    // Atomic per-account throttling, persisted across process restarts.
    let attempts: i64 = sqlx::query_scalar("INSERT INTO login_attempts (email, attempts, started_at) VALUES (?, 1, unixepoch()) ON CONFLICT(email) DO UPDATE SET attempts = CASE WHEN started_at < unixepoch() - 900 THEN 1 ELSE attempts + 1 END, started_at = CASE WHEN started_at < unixepoch() - 900 THEN unixepoch() ELSE started_at END RETURNING attempts")
        .bind(&email).fetch_one(&state.db).await?;
    if attempts > 5 {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts. Try again in 15 minutes.",
        ));
    }
    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
        .bind(&email)
        .fetch_optional(&state.db)
        .await?;
    let hash = user.as_ref().map(|u| u.password.clone());
    let valid = tokio::task::spawn_blocking(move || {
        if let Some(hash) = hash {
            PasswordHash::new(&hash).is_ok_and(|hash| {
                Argon2::default()
                    .verify_password(input.password.as_bytes(), &hash)
                    .is_ok()
            })
        } else {
            // Spend the same password-hash work for unknown accounts.
            let _ = Argon2::default()
                .hash_password(input.password.as_bytes(), &SaltString::generate(&mut OsRng));
            false
        }
    })
    .await
    .unwrap_or(false);
    let user = user
        .filter(|u| valid && u.status == "active")
        .ok_or_else(|| {
            AppError::validation("email", "These credentials do not match an active account.")
        })?;
    sqlx::query("DELETE FROM login_attempts WHERE email = ?")
        .bind(&email)
        .execute(&state.db)
        .await?;
    if let Some(old) = session::load(&state.db, &jar).await? {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(old.id)
            .execute(&state.db)
            .await?;
    }
    let session = session::create(&state, Some(user.id)).await?;
    Ok((
        jar.add(session::cookie(session.id, state.config.cookie_secure)),
        Json(json!({"user":user,"csrf_token":session.csrf})),
    ))
}
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<Value>)> {
    if let Some(old) = session::load(&state.db, &jar).await? {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(old.id)
            .execute(&state.db)
            .await?;
    }
    let session = session::create(&state, None).await?;
    Ok((
        jar.add(session::cookie(session.id, state.config.cookie_secure)),
        Json(json!({"user":null,"csrf_token":session.csrf})),
    ))
}
#[derive(serde::Deserialize)]
pub struct ProfileRequest {
    name: String,
}
pub async fn profile(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(input): Json<ProfileRequest>,
) -> Result<Json<Value>> {
    let name = input.name.trim();
    if !(2..=80).contains(&name.chars().count()) {
        return Err(AppError::validation(
            "name",
            "Use between 2 and 80 characters.",
        ));
    }
    sqlx::query("UPDATE users SET name = ? WHERE id = ?")
        .bind(name)
        .bind(user.id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"user": User::find(&state.db, user.id).await?})))
}
