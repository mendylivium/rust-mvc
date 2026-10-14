use crate::app::{
    http::error::{AppError, Result},
    models::user::User,
};
use axum::http::StatusCode;

pub fn admin(user: &User) -> Result<()> {
    if user.role != "admin" {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "Administrator access is required.",
        ));
    }
    Ok(())
}
pub fn modify_user(actor: &User, target: i64, role: &str, status: &str) -> Result<()> {
    admin(actor)?;
    if actor.id == target && (role != "admin" || status != "active") {
        return Err(AppError::validation(
            "role",
            "You cannot remove your own administrator access.",
        ));
    }
    Ok(())
}
