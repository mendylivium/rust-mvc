use serde::Serialize;
use sqlx::{AnyPool, FromRow};

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
}
impl User {
    pub async fn find(db: &AnyPool, id: i64) -> std::result::Result<Option<Self>, sqlx::Error> {
        sqlx::query_as(&crate::database::sql(
            db,
            "SELECT * FROM users WHERE id = ?",
        ))
        .bind(id)
        .fetch_optional(db)
        .await
    }
}
