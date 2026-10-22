//! # SQLite Storage Adapter (`adapters/database`)
//!
//! Splits persistence into focused, AI-friendly repository modules while implementing
//! all Hexagonal Output Ports (`PostRepository`, `TaxonomyRepository`, `OptionRepository`,
//! `CommentRepository`, `MigrationRepository`).

pub mod comments;
pub mod media;
pub mod migration;
pub mod options;
pub mod plugins_seo;
pub mod posts;
pub mod schema;
pub mod taxonomies;
pub mod users;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Concrete Embedded Rusqlite Adapter implementing the Hexagonal Output Ports.
#[derive(Clone)]
pub struct Database {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) db_path: PathBuf,
}

/// Alias for explicit Hexagonal Adapter naming (`SqliteDatabase`).
pub type SqliteDatabase = Database;
