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
pub async fn load(db: &sqlx::SqlitePool, jar: &CookieJar) -> Result<Option<Session>> {
    let Some(cookie) = jar.get(COOKIE) else {
        return Ok(None);
    };
    Ok(sqlx::query_as(
        "SELECT id, csrf, user_id FROM sessions WHERE id = ? AND expires_at > unixepoch()",
    )
    .bind(cookie.value())
    .fetch_optional(db)
    .await?)
}
pub async fn create(state: &AppState, user_id: Option<i64>) -> Result<Session> {
    let session = Session {
        id: token(),
        csrf: token(),
        user_id,
    };
    sqlx::query("DELETE FROM sessions WHERE expires_at <= unixepoch()")
        .execute(&state.db)
        .await?;
    sqlx::query("INSERT INTO sessions (id, csrf, user_id, expires_at) VALUES (?, ?, ?, unixepoch() + 43200)")
        .bind(&session.id).bind(&session.csrf).bind(user_id).execute(&state.db).await?;
    Ok(session)
}
pub async fn csrf(State(state): State<AppState>, request: Request, next: Next) -> Result<Response> {
    if !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    ) {
        let jar = CookieJar::from_headers(request.headers());
        let session = load(&state.db, &jar).await?.ok_or_else(|| {
            AppError::new(
                StatusCode::FORBIDDEN,
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
            return Err(AppError::new(
                StatusCode::FORBIDDEN,
                "Invalid CSRF token. Refresh and try again.",
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
