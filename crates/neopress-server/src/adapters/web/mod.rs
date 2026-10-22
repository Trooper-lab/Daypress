//! # Web / HTTP Adapter (`adapters/web`)
//!
//! Maps `DomainError` and `AppError` into Axum HTTP responses (`IntoResponse`)
//! and exposes the WordPress REST API and Admin/Public SSR handlers.

use crate::domain::errors::DomainError;
use crate::errors::{AppError, WpErrorData, WpErrorResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

pub use crate::api;
pub use crate::handlers;

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        AppError::from(self).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "rest_not_found", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "rest_invalid_param", msg),
            AppError::Unauthorized(msg) => {
                let code = if msg.contains("logged in") {
                    "rest_not_logged_in"
                } else {
                    "rest_cannot_create"
                };
                (StatusCode::UNAUTHORIZED, code, msg)
            }
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "rest_forbidden", msg),
            AppError::Database(err) => {
                tracing::error!("Database error encountered: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "rest_database_error",
                    err.to_string(),
                )
            }
            AppError::Io(err) => {
                tracing::error!("I/O operation failed: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "rest_io_error",
                    err.to_string(),
                )
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "rest_internal_error",
                    msg,
                )
            }
        };

        let body = Json(WpErrorResponse {
            code: code.to_string(),
            message,
            data: WpErrorData {
                status: status.as_u16(),
            },
        });

        (status, body).into_response()
    }
}
