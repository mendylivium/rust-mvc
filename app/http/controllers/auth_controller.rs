use crate::app::http::requests::extractors::Json as Input;
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
use tokio::sync::OwnedSemaphorePermit;

pub async fn hash_password(password: String) -> anyhow::Result<String> {
    hash_with_permit(password, None).await
}

async fn hash_with_permit(
    password: String,
    permit: Option<OwnedSemaphorePermit>,
) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|hash| hash.to_string())
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    })
    .await?
}
pub async fn hash_password_for_request(state: &AppState, password: String) -> Result<String> {
    let permit = state.security.password()?;
    hash_with_permit(password, Some(permit))
        .await
        .map_err(|error| {
            tracing::error!(%error, "Password hashing failed");
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password hashing failed.",
            )
        })
}
pub async fn current(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<Value>)> {
    let session = match session::load(&state.db, &jar).await? {
        Some(session) => session,
        None => session::create(&state).await?,
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
    Input(input): Input<LoginRequest>,
) -> Result<(CookieJar, Json<Value>)> {
    state.security.login()?;
    let old = session::load(&state.db, &jar)
        .await?
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "Please sign in again."))?;
    let email = input.email.trim().to_lowercase();
    if email.len() > 254 || input.password.len() > 128 {
        return Err(AppError::validation("email", "Invalid credentials."));
    }
    let user: Option<User> = sqlx::query_as(&crate::database::sql(
        &state.db,
        "SELECT * FROM users WHERE email = ?",
    ))
    .bind(&email)
    .fetch_optional(&state.db)
    .await?;
    let permit = state.security.password()?;
    // Keep identical throttling for existing and unknown accounts without unbounded storage.
    let mut tx = state.db.begin().await?;
    crate::database::auth_lock(&mut tx).await?;
    sqlx::query(&crate::database::sql(
        &state.db,
        "DELETE FROM login_attempts WHERE started_at <= unixepoch() - 900",
    ))
    .execute(&mut *tx)
    .await?;
    let previous: Option<i64> = sqlx::query_scalar(&crate::database::sql(
        &state.db,
        "SELECT attempts FROM login_attempts WHERE email = ?",
    ))
    .bind(&email)
    .fetch_optional(&mut *tx)
    .await?;
    let attempts = if let Some(previous) = previous {
        sqlx::query(&crate::database::sql(
            &state.db,
            "UPDATE login_attempts SET attempts = attempts + 1 WHERE email = ?",
        ))
        .bind(&email)
        .execute(&mut *tx)
        .await?;
        Some(previous + 1)
    } else {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM login_attempts")
            .fetch_one(&mut *tx)
            .await?;
        if count < 10000 {
            sqlx::query(&crate::database::sql(&state.db, "INSERT INTO login_attempts (email, attempts, started_at) VALUES (?, 1, unixepoch())"))
                .bind(&email).execute(&mut *tx).await?;
            Some(1)
        } else {
            None
        }
    };
    tx.commit().await?;
    if attempts.is_none_or(|attempts| attempts > 5) {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts. Try again in 15 minutes.",
        ));
    }
    let hash = user.as_ref().map(|u| u.password.clone());
    let valid = tokio::task::spawn_blocking(move || {
        let _permit = permit;
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
    let (session, user) = session::authenticate(&state, &user, &old.id).await?;
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
        sqlx::query(&crate::database::sql(
            &state.db,
            "DELETE FROM sessions WHERE id = ?",
        ))
        .bind(old.id)
        .execute(&state.db)
        .await?;
    }
    let expired = axum_extra::extract::cookie::Cookie::build(session::COOKIE)
        .path("/")
        .build();
    Ok((
        jar.remove(expired),
        Json(json!({"user":null,"csrf_token":""})),
    ))
}
#[derive(serde::Deserialize)]
pub struct ProfileRequest {
    name: String,
}
pub async fn profile(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Input(input): Input<ProfileRequest>,
) -> Result<Json<Value>> {
    let name = input.name.trim();
    if !(2..=80).contains(&name.chars().count()) {
        return Err(AppError::validation(
            "name",
            "Use between 2 and 80 characters.",
        ));
    }
    sqlx::query(&crate::database::sql(
        &state.db,
        "UPDATE users SET name = ? WHERE id = ?",
    ))
    .bind(name)
    .bind(user.id)
    .execute(&state.db)
    .await?;
    Ok(Json(json!({"user": User::find(&state.db, user.id).await?})))
}
