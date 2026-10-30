use crate::domain::errors::DomainError;
use serde::Serialize;
use thiserror::Error;

/// Core error type bridging infrastructure adapters (`rusqlite`, `std::io`) and `DomainError`.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => AppError::NotFound(msg),
            DomainError::Validation(msg) => AppError::BadRequest(msg),
            DomainError::Unauthorized(msg) => AppError::Unauthorized(msg),
            DomainError::Forbidden(msg) => AppError::Forbidden(msg),
            DomainError::Storage(msg) => AppError::Internal(msg),
            DomainError::Io(msg) => AppError::Internal(msg),
            DomainError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

impl From<AppError> for DomainError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound(msg) => DomainError::NotFound(msg),
            AppError::BadRequest(msg) => DomainError::Validation(msg),
            AppError::Unauthorized(msg) => DomainError::Unauthorized(msg),
            AppError::Forbidden(msg) => DomainError::Forbidden(msg),
            AppError::TooManyRequests(msg) => DomainError::Forbidden(msg),
            AppError::Database(e) => DomainError::Storage(e.to_string()),
            AppError::Io(e) => DomainError::Io(e.to_string()),
            AppError::Internal(msg) => DomainError::Internal(msg),
        }
    }
}

/// 1:1 WordPress REST API Error format:
/// `{"code": "rest_...", "message": "...", "data": {"status": 404}}`
#[derive(Debug, Serialize)]
pub struct DaypressErrorResponse {
    pub code: String,
    pub message: String,
    pub data: DaypressErrorData,
}

#[derive(Debug, Serialize)]
pub struct DaypressErrorData {
    pub status: u16,
}
