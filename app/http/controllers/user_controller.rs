use super::auth_controller::hash_password_for_request;
use crate::app::http::requests::extractors::{Json as Input, Path, Query};
use crate::{
    AppState,
    app::{
        http::{
            error::{AppError, Result},
            middleware::session::AuthUser,
            requests::UserRequest,
        },
        models::user::User,
        policies,
    },
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, Default)]
pub struct Filters {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub role: Option<String>,
}
pub async fn index(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    Query(filters): Query<Filters>,
) -> Result<Json<Value>> {
    policies::admin(&actor)?;
    let search = format!(
        "%{}%",
        filters
            .search
            .unwrap_or_default()
            .chars()
            .take(100)
            .collect::<String>()
    );
    let role = filters.role.unwrap_or_default();
    let total: i64 = sqlx::query_scalar(&crate::database::sql(&state.db, "SELECT COUNT(*) FROM users WHERE (LOWER(name) LIKE LOWER(?) OR LOWER(email) LIKE LOWER(?)) AND (? = '' OR role = ?)"),)
    .bind(&search)
    .bind(&search)
    .bind(&role)
    .bind(&role)
    .fetch_one(&state.db)
    .await?;
    let last_page = ((total + 9) / 10).max(1);
    let page = filters.page.unwrap_or(1).clamp(1, last_page);
    let users: Vec<User> = sqlx::query_as(&crate::database::sql(&state.db, "SELECT * FROM users WHERE (LOWER(name) LIKE LOWER(?) OR LOWER(email) LIKE LOWER(?)) AND (? = '' OR role = ?) ORDER BY id DESC LIMIT 10 OFFSET ?"))
        .bind(&search).bind(&search).bind(&role).bind(&role).bind((page - 1) * 10).fetch_all(&state.db).await?;
    Ok(Json(
        json!({"data":users,"meta":{"total":total,"page":page,"per_page":10,"last_page":last_page}}),
    ))
}
pub async fn store(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    Input(input): Input<UserRequest>,
) -> Result<(StatusCode, Json<Value>)> {
    policies::admin(&actor)?;
    let input = input.validated(true)?;
    let password = hash_password_for_request(&state, input.password).await?;
    let mut tx = state.db.begin().await?;
    crate::database::auth_lock(&mut tx).await?;
    sqlx::query(&crate::database::sql(
        &state.db,
        "INSERT INTO users (name, email, password, role, status) VALUES (?, ?, ?, ?, ?)",
    ))
    .bind(input.name)
    .bind(&input.email)
    .bind(password)
    .bind(input.role)
    .bind(input.status)
    .execute(&mut *tx)
    .await?;
    let user: User = sqlx::query_as(&crate::database::sql(
        &state.db,
        "SELECT * FROM users WHERE email = ?",
    ))
    .bind(&input.email)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"data":user}))))
}
pub async fn update(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    Path(id): Path<i64>,
    Input(input): Input<UserRequest>,
) -> Result<Json<Value>> {
    policies::admin(&actor)?;
    let input = input.validated(false)?;
    policies::modify_user(&actor, id, &input.role, &input.status)?;
    if User::find(&state.db, id).await?.is_none() {
        return Err(AppError::new(StatusCode::NOT_FOUND, "User not found."));
    }
    let changed_password = !input.password.is_empty();
    let password = if changed_password {
        Some(hash_password_for_request(&state, input.password).await?)
    } else {
        None
    };
    let mut tx = state.db.begin().await?;
    crate::database::auth_lock(&mut tx).await?;
    sqlx::query(&crate::database::sql(&state.db, "UPDATE users SET name = ?, email = ?, password = COALESCE(?, password), role = ?, status = ? WHERE id = ?"),)
    .bind(input.name)
    .bind(input.email)
    .bind(password)
    .bind(input.role)
    .bind(&input.status)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if changed_password || input.status == "inactive" {
        sqlx::query(&crate::database::sql(
            &state.db,
            "DELETE FROM sessions WHERE user_id = ?",
        ))
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(Json(json!({"data":User::find(&state.db, id).await?})))
}
pub async fn destroy(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    policies::admin(&actor)?;
    if actor.id == id {
        return Err(AppError::validation(
            "user",
            "You cannot delete your own account.",
        ));
    }
    let result = sqlx::query(&crate::database::sql(
        &state.db,
        "DELETE FROM users WHERE id = ?",
    ))
    .bind(id)
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::new(StatusCode::NOT_FOUND, "User not found."));
    }
    Ok(StatusCode::NO_CONTENT)
}
