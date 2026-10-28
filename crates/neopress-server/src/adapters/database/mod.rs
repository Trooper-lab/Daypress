//! # SQLite Storage Adapter (`adapters/database`)
//!
//! Splits persistence into focused, AI-friendly repository modules while implementing
//! all Hexagonal Output Ports (`PostRepository`, `TaxonomyRepository`, `OptionRepository`,
//! `CommentRepository`, `MigrationRepository`).

pub mod admin_lists;
pub mod comments;
pub mod ddl;
pub mod maintenance;
pub mod media;
pub mod migration;
pub mod options;
pub mod password_resets;
pub mod plugins_seo;
pub mod post_ast;
pub mod posts;
pub mod schema;
pub mod settings_extra;
pub mod taxonomies;
pub mod taxonomies_discovery;
pub mod users;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Concrete Embedded Rusqlite Adapter implementing the Hexagonal Output Ports.
#[derive(Clone)]
pub struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) db_path: PathBuf,
    /// Failed-login counter shared by every clone of this database handle.
    pub(crate) login_throttle: crate::auth::LoginThrottle,
    /// "Lost your password?" request counter, kept apart so reset requests never lock out logins.
    pub(crate) password_reset_throttle: crate::auth::LoginThrottle,
}

impl Database {
    pub fn theme_templates(&self) -> crate::theme_builder::repository::ThemeTemplateRepository {
        crate::theme_builder::repository::ThemeTemplateRepository::from_database(self)
    }
}

/// Alias for explicit Hexagonal Adapter naming (`SqliteDatabase`).
pub type SqliteDatabase = Database;
