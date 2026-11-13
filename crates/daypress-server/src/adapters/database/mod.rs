//! # SQLite Storage Adapter (`adapters/database`)
//!
//! Splits persistence into focused, AI-friendly repository modules while implementing
//! all Hexagonal Output Ports (`PostRepository`, `TaxonomyRepository`, `OptionRepository`,
//! `CommentRepository`, `MigrationRepository`).

pub mod ab_vitals;
pub mod admin_lists;
pub mod analytics;
pub mod comments;
pub mod content_model;
pub mod ddl;
pub mod editorial;
pub mod feeds;
pub mod i18n_repo;
pub mod maintenance;
pub mod media;
pub mod migration;
#[cfg(test)]
mod migration_tests;
pub mod newsletters;
pub mod options;
pub mod password_resets;
pub mod plugins_seo;
pub mod post_ast;
pub mod posts;
pub mod posts_write;
pub mod revisions;
pub mod scheduling;
pub mod schema;
pub mod search_fts;
pub mod settings_extra;
pub mod taxonomies;
pub mod taxonomies_discovery;
pub mod users;
pub mod v10_readiness;
pub mod wasm_extensions;
pub mod webhooks_shortlinks;

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
    /// Pre-compressed in-memory SSR hot cache (`<50µs` P99 latency) shared across all worker threads.
    pub(crate) hot_cache: Arc<crate::hot_cache::SsrHotCache>,
    /// Per-form sliding-window IP rate limiter for FormStudio submissions.
    pub(crate) form_rate_limiter: Arc<crate::hot_cache::FormRateLimiter>,
}

impl Database {
    pub fn theme_templates(&self) -> crate::theme_builder::repository::ThemeTemplateRepository {
        crate::theme_builder::repository::ThemeTemplateRepository::from_database(self)
    }

    pub fn db_path_display(&self) -> String {
        self.db_path.display().to_string()
    }

    /// Invalidates all entries in the in-memory SSR hot cache and triggers an asynchronous
    /// Cloudflare Zone cache purge if `daypress_cf_zone_id` and `daypress_cf_api_token` are configured.
    pub fn invalidate_hot_cache(&self, reason: &str) -> usize {
        let cleared = self.hot_cache.invalidate_all(reason);
        // Check if Cloudflare Zone purge credentials are set without holding a write lock
        let cf_creds = if let Ok(conn) = self.conn.lock() {
            let zone_id: Option<String> = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'daypress_cf_zone_id'",
                    [],
                    |r| r.get(0),
                )
                .ok()
                .filter(|s: &String| !s.trim().is_empty());
            let api_token: Option<String> = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'daypress_cf_api_token'",
                    [],
                    |r| r.get(0),
                )
                .ok()
                .filter(|s: &String| !s.trim().is_empty());
            match (zone_id, api_token) {
                (Some(z), Some(t)) => Some((z, t)),
                _ => None,
            }
        } else {
            None
        };

        if let Some((zone_id, api_token)) = cf_creds {
            if tokio::runtime::Handle::try_current().is_ok() {
                tokio::spawn(async move {
                    let url = format!(
                        "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
                        zone_id.trim()
                    );
                    let client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(5))
                        .build()
                        .unwrap_or_default();
                    let _ = client
                        .post(&url)
                        .bearer_auth(api_token.trim())
                        .json(&serde_json::json!({ "purge_everything": true }))
                        .send()
                        .await;
                });
            }
        }
        cleared
    }

    pub fn hot_cache_stats(&self) -> crate::hot_cache::HotCacheStats {
        let cf_configured = self
            .get_option("daypress_cf_zone_id")
            .ok()
            .flatten()
            .filter(|s| !s.trim().is_empty())
            .is_some()
            && self
                .get_option("daypress_cf_api_token")
                .ok()
                .flatten()
                .filter(|s| !s.trim().is_empty())
                .is_some();
        self.hot_cache.stats(cf_configured)
    }
}

/// Alias for explicit Hexagonal Adapter naming (`SqliteDatabase`).
pub type SqliteDatabase = Database;
pub use schema::OnboardingSetupRequest;
