use crate::{
    AppState,
    app::{
        http::{error::Result, middleware::session::AuthUser},
        models::user::User,
        policies,
    },
    database::sql,
};
use axum::{Json, extract::State};
use serde_json::{Value, json};

pub async fn index(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
) -> Result<Json<Value>> {
    policies::admin(&actor)?;
    let now = time::OffsetDateTime::now_utc();
    let month_index = now.year() * 12 + now.month() as i32 - 1;
    let month = |index: i32| {
        format!(
            "{:04}-{:02}",
            index.div_euclid(12),
            index.rem_euclid(12) + 1
        )
    };
    let month_start = format!("{}-01", month(month_index));
    let (total, active, admins, new_this_month): (i64, i64, i64, i64) = sqlx::query_as(&sql(&state.db,
        "SELECT COUNT(*), COUNT(CASE WHEN status = 'active' THEN 1 END), COUNT(CASE WHEN role = 'admin' THEN 1 END), COUNT(CASE WHEN created_at >= ? THEN 1 END) FROM users"))
        .bind(month_start).fetch_one(&state.db).await?;
    let recent: Vec<User> = sqlx::query_as("SELECT * FROM users ORDER BY id DESC LIMIT 5")
        .fetch_all(&state.db)
        .await?;
    let mut growth = Vec::new();
    for index in (month_index - 5)..=month_index {
        let label = month(index);
        let count: i64 = sqlx::query_scalar(&sql(
            &state.db,
            "SELECT COUNT(*) FROM users WHERE created_at >= ? AND created_at < ?",
        ))
        .bind(format!("{label}-01"))
        .bind(format!("{}-01", month(index + 1)))
        .fetch_one(&state.db)
        .await?;
        growth.push((label, count));
    }
    Ok(Json(
        json!({"stats":{"total":total,"active":active,"admins":admins,"new_this_month":new_this_month},"recent":recent,"growth":growth}),
    ))
}
