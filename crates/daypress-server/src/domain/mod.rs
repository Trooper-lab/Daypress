//! # Domain Layer (Pure State & Business Entities)
//!
//! Contains canonical WordPress-faithful domain models (`DaypressPostResponse`, `DaypressCategoryResponse`,
//! `DaypressSettingsResponse`, `ImportBundleRequest`, etc.) and pure business errors (`DomainError`).
//! Zero dependencies on web frameworks (`axum`) or storage drivers (`rusqlite`).

pub mod errors;
pub mod models;

pub use errors::{DomainError, DomainResult};
pub use models::*;
