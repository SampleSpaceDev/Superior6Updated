// errors.rs

use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Hashing error")]
    HashingError,

    #[error("Token creation failed")]
    TokenCreation,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing token")]
    MissingToken,

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email already exists")]
    EmailExists,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("Gameweek deadline passed")]
    DeadlinePassed,

    #[error("Predictions already submitted")]
    PredictionsAlreadySubmitted,

    #[error("Invalid prediction data")]
    InvalidPrediction,

    #[error("Template error: {0}")]
    TemplateError(#[from] askama::Error),

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            AppError::HashingError => (StatusCode::INTERNAL_SERVER_ERROR, "Hashing error"),
            AppError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed"),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AppError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing token"),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AppError::EmailExists => (StatusCode::CONFLICT, "Email already exists"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::DeadlinePassed => (StatusCode::BAD_REQUEST, "Prediction deadline has passed"),
            AppError::PredictionsAlreadySubmitted => (StatusCode::BAD_REQUEST, "Predictions already submitted for this gameweek"),
            AppError::InvalidPrediction => (StatusCode::BAD_REQUEST, "Invalid prediction data"),
            AppError::TemplateError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Template error"),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string(),
        }));

        (status, body).into_response()
    }
}