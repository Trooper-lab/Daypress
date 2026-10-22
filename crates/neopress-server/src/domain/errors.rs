//! Pure Domain Errors.
//!
//! This module contains zero web framework (`axum`) or database (`rusqlite`) dependencies,
//! ensuring core CMS business logic and ports remain 100% infrastructure-agnostic.

use thiserror::Error;

/// Infrastructure-agnostic domain error for Neopress use cases and repository ports.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation / bad request: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
