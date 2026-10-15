use crate::{
    AppState,
    app::{
        http::error::{AppError, Result},
        models::user::User,
    },
};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use sqlx::FromRow;

pub const COOKIE: &str = "rustmvc_session";
#[derive(FromRow)]
pub struct Session {
    pub id: String,
    pub csrf: String,
    pub user_id: Option<i64>,
}
pub fn token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
pub fn cookie(id: String, secure: bool) -> Cookie<'static> {
    Cookie::build((COOKIE, id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(time::Duration::hours(12))
        .build()
}
pub async fn load(db: &sqlx::AnyPool, jar: &CookieJar) -> Result<Option<Session>> {
    let Some(cookie) = jar.get(COOKIE) else {
        return Ok(None);
    };
    Ok(sqlx::query_as(&crate::database::sql(
        db,
        "SELECT id, csrf, user_id FROM sessions WHERE id = ? AND expires_at > unixepoch()",
    ))
    .bind(cookie.value())
    .fetch_optional(db)
    .await?)
}
pub async fn create(state: &AppState) -> Result<Session> {
    state.security.session()?;
    let session = Session {
        id: token(),
        csrf: token(),
        user_id: None,
    };
    let mut tx = state.db.begin().await?;
    crate::database::auth_lock(&mut tx).await?;
    sqlx::query(&crate::database::sql(
        &state.db,
        "DELETE FROM sessions WHERE expires_at <= unixepoch()",
    ))
    .execute(&mut *tx)
    .await?;
    let inserted = sqlx::query(&crate::database::sql(&state.db, "INSERT INTO sessions (id, csrf, user_id, expires_at) SELECT ?, ?, NULL, unixepoch() + 43200 WHERE (SELECT COUNT(*) FROM sessions) < 10000"))
        .bind(&session.id).bind(&session.csrf).execute(&mut *tx).await?;
    if inserted.rows_affected() == 0 {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Session capacity reached. Try again later.",
        ));
    }
    tx.commit().await?;
    Ok(session)
}

/// Issue a session only if the verified credentials still match under the write lock.
/// A reset either precedes this check (login fails) or follows it (deletes this session).
pub async fn authenticate(
    state: &AppState,
    verified: &User,
    old_id: &str,
) -> Result<(Session, User)> {
    let session = Session {
        id: token(),
        csrf: token(),
        user_id: Some(verified.id),
    };
    let mut tx = state.db.begin().await?;
    crate::database::auth_lock(&mut tx).await?;
    let inserted = sqlx::query(&crate::database::sql(&state.db, "INSERT INTO sessions (id, csrf, user_id, expires_at) SELECT ?, ?, id, unixepoch() + 43200 FROM users WHERE id = ? AND password = ? AND status = 'active' AND EXISTS (SELECT 1 FROM sessions WHERE id = ? AND expires_at > unixepoch())"))
        .bind(&session.id).bind(&session.csrf).bind(verified.id).bind(&verified.password).bind(old_id)
        .execute(&mut *tx).await?;
    if inserted.rows_affected() == 0 {
        return Err(AppError::validation(
            "email",
            "Credentials or session changed. Please sign in again.",
        ));
    }
    sqlx::query(&crate::database::sql(
        &state.db,
        "DELETE FROM sessions WHERE id = ?",
    ))
    .bind(old_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(&crate::database::sql(
        &state.db,
        "DELETE FROM login_attempts WHERE email = ?",
    ))
    .bind(&verified.email)
    .execute(&mut *tx)
    .await?;
    let user = sqlx::query_as(&crate::database::sql(
        &state.db,
        "SELECT * FROM users WHERE id = ?",
    ))
    .bind(verified.id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((session, user))
}
pub async fn csrf(State(state): State<AppState>, request: Request, next: Next) -> Result<Response> {
    if !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    ) {
        let jar = CookieJar::from_headers(request.headers());
        let session = load(&state.db, &jar).await?.ok_or_else(|| {
            AppError::new(
                if jar.get(COOKIE).is_some() {
                    StatusCode::UNAUTHORIZED
                } else {
                    StatusCode::FORBIDDEN
                },
                "Your session expired. Refresh and try again.",
            )
        })?;
        let supplied = request
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        // All tokens are fixed-size random values. Compare without an early byte exit.
        let matches = supplied.len() == session.csrf.len()
            && supplied
                .bytes()
                .zip(session.csrf.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0;
        if !matches {
            return Err(AppError(
                StatusCode::FORBIDDEN,
                "Your session changed. Review and submit again.".into(),
                serde_json::json!({"csrf": ["Refresh the session before retrying."]}),
            ));
        }
    }
    Ok(next.run(request).await)
}
pub struct AuthUser(pub User);
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let jar = CookieJar::from_headers(&parts.headers);
        let session = load(&state.db, &jar).await?;
        if let Some(id) = session.and_then(|s| s.user_id)
            && let Some(user) = User::find(&state.db, id).await?
            && user.status == "active"
        {
            return Ok(Self(user));
        }
        Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "Please sign in to continue.",
        ))
    }
}
