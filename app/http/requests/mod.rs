use super::error::{AppError, Result};
use serde::Deserialize;
use validator::Validate;
pub mod extractors;

pub fn normalize_email(email: &str) -> Result<String> {
    let email = email.trim().to_lowercase();
    if email.len() > 254 || !validator::ValidateEmail::validate_email(&email) {
        return Err(AppError::validation(
            "email",
            "Please enter a valid email address.",
        ));
    }
    Ok(email)
}

#[derive(Deserialize, Validate)]
pub struct UserRequest {
    #[validate(length(min = 2, max = 80))]
    pub name: String,
    #[validate(email, length(max = 254))]
    pub email: String,
    #[serde(default)]
    pub password: String,
    pub role: String,
    pub status: String,
}
impl UserRequest {
    pub fn validated(mut self, creating: bool) -> Result<Self> {
        self.name = self.name.trim().to_owned();
        self.email = normalize_email(&self.email)?;
        if let Err(errors) = self.validate() {
            let errors = errors
                .field_errors()
                .into_keys()
                .map(|key| {
                    let message = format!("Please enter a valid {key}.");
                    (key.into_owned(), vec![message])
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            return Err(AppError(
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                "Please check the form.".into(),
                serde_json::json!(errors),
            ));
        }
        if !["admin", "member"].contains(&self.role.as_str()) {
            return Err(AppError::validation("role", "Choose admin or member."));
        }
        if !["active", "inactive"].contains(&self.status.as_str()) {
            return Err(AppError::validation("status", "Choose active or inactive."));
        }
        if (creating || !self.password.is_empty()) && !(12..=128).contains(&self.password.len()) {
            return Err(AppError::validation(
                "password",
                "Use a password between 12 and 128 bytes.",
            ));
        }
        Ok(self)
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
