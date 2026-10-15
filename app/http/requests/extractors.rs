use crate::app::http::error::{AppError, Result};
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

pub struct Json<T>(pub T);
impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for Json<T> {
    type Rejection = AppError;
    async fn from_request(req: Request, state: &S) -> Result<Self> {
        axum::Json::<T>::from_request(req, state)
            .await
            .map(|v| Self(v.0))
            .map_err(|e| AppError::new(e.status(), &e.body_text()))
    }
}
pub struct Query<T>(pub T);
impl<S: Send + Sync, T: DeserializeOwned> FromRequestParts<S> for Query<T> {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        axum::extract::Query::<T>::from_request_parts(parts, state)
            .await
            .map(|v| Self(v.0))
            .map_err(|e| AppError::new(e.status(), &e.body_text()))
    }
}
pub struct Path<T>(pub T);
impl<S: Send + Sync, T: DeserializeOwned + Send> FromRequestParts<S> for Path<T> {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        axum::extract::Path::<T>::from_request_parts(parts, state)
            .await
            .map(|v| Self(v.0))
            .map_err(|e| AppError::new(e.status(), &e.body_text()))
    }
}
