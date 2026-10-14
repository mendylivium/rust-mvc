use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

pub struct AppError(pub StatusCode, pub String, pub Value);
impl AppError {
    pub fn new(status: StatusCode, message: &str) -> Self {
        Self(status, message.into(), json!({}))
    }
    pub fn validation(field: &str, message: &str) -> Self {
        Self(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Please check the form.".into(),
            json!({field: [message]}),
        )
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"message": self.1, "errors": self.2}))).into_response()
    }
}
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db) = error
            && db.is_unique_violation()
        {
            return Self::validation("email", "This email is already taken.");
        }
        tracing::error!(%error, "Database request failed");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong. Please try again.",
        )
    }
}
pub type Result<T> = std::result::Result<T, AppError>;
