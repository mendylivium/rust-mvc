use crate::{
    AppState,
    app::{
        http::{error::Result, middleware::session::AuthUser},
        models::user::User,
        policies,
    },
};
use axum::{Json, extract::State};
use serde_json::{Value, json};

pub async fn index(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
) -> Result<Json<Value>> {
    policies::admin(&actor)?;
    let (total, active, admins, new_this_month): (i64, i64, i64, i64) = sqlx::query_as("SELECT COUNT(*), COALESCE(SUM(status = 'active'),0), COALESCE(SUM(role = 'admin'),0), COALESCE(SUM(created_at >= strftime('%Y-%m-01', 'now')),0) FROM users").fetch_one(&state.db).await?;
    let recent: Vec<User> = sqlx::query_as("SELECT * FROM users ORDER BY id DESC LIMIT 5")
        .fetch_all(&state.db)
        .await?;
    let growth: Vec<(String, i64)> = sqlx::query_as("WITH RECURSIVE months(month, n) AS (SELECT date('now', 'start of month', '-5 months'), 1 UNION ALL SELECT date(month, '+1 month'), n + 1 FROM months WHERE n < 6) SELECT strftime('%Y-%m', month), (SELECT COUNT(*) FROM users WHERE strftime('%Y-%m', created_at) = strftime('%Y-%m', month)) FROM months").fetch_all(&state.db).await?;
    Ok(Json(
        json!({"stats":{"total":total,"active":active,"admins":admins,"new_this_month":new_this_month},"recent":recent,"growth":growth}),
    ))
}
