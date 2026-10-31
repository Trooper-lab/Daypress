#![allow(unused_imports)]
use super::Database;
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, Result, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Payload submitted by the First-Time Onboarding Wizard (`POST /setup`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OnboardingSetupRequest {
    #[serde(default)]
    pub site_title: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub confirm_password: String,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub brand_color: String,
    #[serde(default)]
    pub typography_scale: String,
    /// `"starter"` (default Daypress sample content), `"blank"` (empty site), or `"migrate"` (1-Click WP Import).
    #[serde(default)]
    pub starter_mode: String,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let is_new = !path.exists();
        let conn = Connection::open(path)?;

        // Enable WAL mode and foreign keys for high performance
        let _: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |r| r.get(0))?;
        conn.execute_batch("PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: path.to_path_buf(),
            login_throttle: Default::default(),
            password_reset_throttle: Default::default(),
        };

        db.init_schema()?;

        if path != Path::new(":memory:") {
            let needs_starter_content = {
                let conn = db.conn.lock().unwrap();
                let is_migrated: bool = conn
                    .query_row(
                        "SELECT 1 FROM options WHERE option_name = 'daypress_site_migrated' AND option_value = '1' LIMIT 1;",
                        [],
                        |_| Ok(true),
                    )
                    .unwrap_or(false);
                let has_legacy_neopress_pack: bool = conn
                    .query_row(
                        "SELECT 1 FROM posts WHERE post_name = 'about-us' AND post_title = 'About Neopress Foundation' LIMIT 1;",
                        [],
                        |_| Ok(true),
                    )
                    .unwrap_or(false);
                let post_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM posts;", [], |r| r.get(0))
                    .unwrap_or(0);
                let user_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM users;", [], |r| r.get(0))
                    .unwrap_or(0);
                // Only a brand-new (or completely empty) database, or an untouched legacy Neopress
                // marketing pack, gets the starter site. Deleting every post on a live site never triggers a reseed.
                !is_migrated
                    && (is_new || (post_count == 0 && user_count == 0) || has_legacy_neopress_pack)
            };

            if needs_starter_content {
                db.seed_starter_site()?;
            }
            db.ensure_admin_credentials()?;
        } else if is_new {
            db.seed_data()?;
        }

        Ok(db)
    }

    /// Whether a stored `user_pass` value is a usable Daypress password hash (not the seed placeholder).
    pub(crate) fn is_usable_password_hash(stored_pass: &str) -> bool {
        let trimmed = stored_pass.trim();
        if trimmed.is_empty() || trimmed == crate::auth::LEGACY_SEED_PASSWORD_HASH {
            return false;
        }
        trimmed.starts_with("$argon2")
            || trimmed.starts_with("$daypress$sha256$")
            || (trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()))
    }

    /// Returns `true` when this Daypress install has not completed first-time setup yet
    /// (no administrator has a usable password set and `daypress_setup_completed` is not `'1'`).
    pub fn needs_onboarding(&self) -> bool {
        let Ok(conn) = self.conn.lock() else {
            return false;
        };
        let setup_done: bool = conn
            .query_row(
                "SELECT 1 FROM options WHERE option_name IN ('daypress_setup_completed', 'daypress_site_migrated') AND option_value = '1' LIMIT 1;",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if setup_done {
            return false;
        }
        let mut stmt = match conn.prepare("SELECT COALESCE(user_pass, '') FROM users;") {
            Ok(s) => s,
            Err(_) => return false,
        };
        let hashes: Vec<String> = match stmt.query_map([], |r| r.get::<_, String>(0)) {
            Ok(rows) => rows.flatten().collect(),
            Err(_) => return false,
        };
        !hashes.iter().any(|h| Self::is_usable_password_hash(h))
    }

    /// Completes the First-Time Onboarding Wizard (`/setup`), creating/updating the primary
    /// administrator account and persisting site identity, theme, and starter content choices.
    pub fn complete_onboarding(&self, req: &OnboardingSetupRequest) -> Result<AuthUser, AppError> {
        if !self.needs_onboarding() {
            return Err(AppError::Forbidden(
                "This Daypress site has already been set up. Please log in.".to_string(),
            ));
        }

        let username = req.username.trim();
        if username.chars().count() < 2 || username.chars().count() > 60 {
            return Err(AppError::BadRequest(
                "Username must be between 2 and 60 characters.".to_string(),
            ));
        }
        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '@'))
        {
            return Err(AppError::BadRequest(
                "Username may only contain letters, numbers, underscores, hyphens, dots, or @."
                    .to_string(),
            ));
        }

        let email = req.email.trim();
        if email.is_empty()
            || !email.contains('@')
            || !email.contains('.')
            || email.contains([' ', ',', ';'])
        {
            return Err(AppError::BadRequest(
                "Please enter a valid administrator email address.".to_string(),
            ));
        }

        let confirm = if req.confirm_password.is_empty() {
            req.password.as_str()
        } else {
            req.confirm_password.as_str()
        };
        crate::password_reset::validate_new_password(&req.password, confirm)
            .map_err(AppError::BadRequest)?;

        let display_name = if req.display_name.trim().is_empty() {
            username.to_string()
        } else {
            req.display_name.trim().to_string()
        };
        let site_title = if req.site_title.trim().is_empty() {
            "Daypress".to_string()
        } else {
            req.site_title.trim().to_string()
        };
        let tagline = if req.tagline.trim().is_empty() {
            "A self-hosted CMS written in Rust.".to_string()
        } else {
            req.tagline.trim().to_string()
        };
        let theme = match req.theme.trim() {
            "Daypress Daylight"
            | "Daypress Night"
            | "Daypress Modern"
            | "Editorial Minimalist"
            | "Twenty Twenty-Five" => req.theme.trim().to_string(),
            _ => "Daypress Daylight".to_string(),
        };
        let brand_color = {
            let c = req.brand_color.trim();
            if c.starts_with('#')
                && (c.len() == 4 || c.len() == 7)
                && c[1..].chars().all(|ch| ch.is_ascii_hexdigit())
            {
                c.to_string()
            } else {
                "#a8401b".to_string()
            }
        };
        let typography_scale = match req.typography_scale.trim() {
            "serif" | "system" | "mono" => req.typography_scale.trim().to_string(),
            _ => "serif".to_string(),
        };

        // Hash with Argon2id before taking the SQLite write lock.
        let pass_hash = crate::auth::hash_user_password(&req.password);
        let nicename = generated_slugify::slugify(username);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("Database connection lock poisoned".to_string()))?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        // Re-verify inside the transaction so concurrent requests cannot race.
        let already_done: bool = tx
            .query_row(
                "SELECT 1 FROM options WHERE option_name = 'daypress_setup_completed' AND option_value = '1' LIMIT 1;",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if already_done {
            return Err(AppError::Forbidden(
                "This Daypress site has already been set up. Please log in.".to_string(),
            ));
        }

        tx.execute(
            r#"INSERT INTO users (ID, user_login, user_pass, user_nicename, user_email, user_url, user_registered, display_name)
               VALUES (1, ?1, ?2, ?3, ?4, '', ?5, ?6)
               ON CONFLICT(ID) DO UPDATE SET
                   user_login = excluded.user_login,
                   user_pass = excluded.user_pass,
                   user_nicename = excluded.user_nicename,
                   user_email = excluded.user_email,
                   display_name = excluded.display_name;"#,
            params![username, pass_hash, nicename, email, now, display_name],
        )?;

        Self::upsert_usermeta(&tx, 1, "role", "Administrator")?;
        Self::upsert_usermeta(&tx, 1, "nickname", &display_name)?;

        let options = [
            ("blogname", site_title.as_str()),
            ("blogdescription", tagline.as_str()),
            ("admin_email", email),
            ("daypress_active_theme", theme.as_str()),
            ("stylesheet", theme.as_str()),
            ("daypress_brand_color", brand_color.as_str()),
            ("daypress_typography_scale", typography_scale.as_str()),
            ("daypress_setup_completed", "1"),
        ];
        for (k, v) in options {
            tx.execute(
                r#"INSERT INTO options (option_name, option_value) VALUES (?1, ?2)
                   ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;"#,
                params![k, v],
            )?;
        }

        if matches!(req.starter_mode.trim(), "blank" | "migrate") {
            tx.execute_batch(
                r#"
                DELETE FROM posts;
                DELETE FROM postmeta;
                DELETE FROM comments;
                DELETE FROM term_relationships;
                DELETE FROM options WHERE option_name = 'daypress_nav_menu_items';
                UPDATE term_taxonomy SET count = 0 WHERE term_taxonomy_id = 1;
                "#,
            )?;
        }

        tx.commit()?;

        Ok(AuthUser {
            user_id: 1,
            username: username.to_string(),
            role: "Administrator".to_string(),
        })
    }

    /// Guarantees the install has an administrator account row.
    ///
    /// - When `DAYPRESS_ADMIN_PASSWORD` is set in the environment, sets that password immediately
    ///   and marks first-time setup complete.
    /// - When a migrated database has accounts without a Daypress password hash, generates one.
    /// - On a fresh un-migrated install without `DAYPRESS_ADMIN_PASSWORD`, leaves the account in
    ///   first-run onboarding mode (`/setup`) so the user can create their admin account in the browser.
    pub fn ensure_admin_credentials(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users;", [], |r| r.get(0))?;
        if user_count == 0 {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            conn.execute(
                r#"INSERT INTO users (ID, user_login, user_pass, user_nicename, user_email, user_registered, display_name)
                   VALUES (1, 'admin', ?1, 'admin', 'admin@example.com', ?2, 'admin');"#,
                params![crate::auth::LEGACY_SEED_PASSWORD_HASH, now],
            )?;
        }

        let is_migrated: bool = conn
            .query_row(
                "SELECT 1 FROM options WHERE option_name = 'daypress_site_migrated' AND option_value = '1' LIMIT 1;",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);

        let pending: Vec<(i64, String)> = {
            let mut stmt = conn.prepare(
                "SELECT ID, user_login FROM users WHERE user_pass = ?1 OR TRIM(COALESCE(user_pass, '')) = '' ORDER BY ID;",
            )?;
            let rows = stmt.query_map(params![crate::auth::LEGACY_SEED_PASSWORD_HASH], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
            rows.collect::<Result<Vec<_>>>()?
        };

        if pending.is_empty() {
            return Ok(());
        }

        let env_password =
            crate::legacy::env_var("DAYPRESS_ADMIN_PASSWORD").filter(|p| !p.trim().is_empty());

        if env_password.is_none() && !is_migrated {
            eprintln!("============================================================");
            eprintln!("✨ First-Time Setup Ready:");
            eprintln!("   Open /setup (or /d-admin/) in your browser to create your");
            eprintln!("   administrator account and choose your Daypress theme.");
            eprintln!("============================================================");
            return Ok(());
        }

        for (index, (user_id, login)) in pending.into_iter().enumerate() {
            let from_env = index == 0 && env_password.is_some();
            let password = match (&env_password, from_env) {
                (Some(p), true) => p.clone(),
                _ => crate::auth::generate_random_password(),
            };
            conn.execute(
                "UPDATE users SET user_pass = ?1 WHERE ID = ?2;",
                params![crate::auth::hash_user_password(&password), user_id],
            )?;
            if from_env {
                conn.execute(
                    r#"INSERT INTO options (option_name, option_value) VALUES ('daypress_setup_completed', '1')
                       ON CONFLICT(option_name) DO UPDATE SET option_value = '1';"#,
                    [],
                )?;
            }
            // stderr, so `--mcp-stdio` keeps a clean JSON-RPC stdout.
            eprintln!("============================================================");
            if from_env {
                eprintln!("🔐 Admin password for '{login}' set from DAYPRESS_ADMIN_PASSWORD.");
            } else {
                eprintln!("🔐 Generated a password for '{login}' (shown only once):");
                eprintln!("   {password}");
            }
            eprintln!("   Log in at /login, then change it under Users → Profile.");
            eprintln!("============================================================");
        }
        Ok(())
    }

    /// Replaces all content with the starter content for a new install (see `crate::seed`).
    pub fn seed_starter_site(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        crate::seed::seed_starter_site(&conn)
    }

    /// Creates an in-memory SQLite database instance (ideal for automated unit and integration tests).
    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
            login_throttle: Default::default(),
            password_reset_throttle: Default::default(),
        };

        db.init_schema()?;
        db.seed_data()?;
        Ok(db)
    }

    /// Dynamically derives site base URL with an active database connection.
    pub(crate) fn get_base_url_with_conn(conn: &Connection) -> String {
        conn.query_row(
            "SELECT option_value FROM options WHERE option_name = 'siteurl'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "http://localhost:8090".to_string())
    }

    /// Dynamically derives site base URL from options (defaults to http://localhost:8090).
    pub fn get_base_url(&self) -> String {
        let conn = self.conn.lock().unwrap();
        Self::get_base_url_with_conn(&conn)
    }

    pub(crate) fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        crate::legacy::upgrade_schema(&conn)?;
        let version: i32 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap_or(0);
        if version == 0 {
            conn.execute_batch(super::ddl::CORE_SCHEMA_V0_SQL)?;
        }

        conn.execute_batch(super::ddl::SUBSYSTEMS_SCHEMA_SQL)?;
        conn.execute_batch(super::ddl::PASSWORD_RESETS_SCHEMA_SQL)?;
        crate::theme_builder::schema::init_theme_builder_schema(&conn)?;
        let _ = Self::ensure_builtin_plugins_seeded_with_conn(&conn);

        // Ensure layout_json and is_sticky columns exist on existing posts tables
        let mut has_layout_col = false;
        let mut has_is_sticky_col = false;
        if let Ok(mut stmt) = conn.prepare("PRAGMA table_info(posts);") {
            if let Ok(mut rows) = stmt.query([]) {
                while let Ok(Some(row)) = rows.next() {
                    if let Ok(col_name) = row.get::<_, String>(1) {
                        if col_name == "layout_json" {
                            has_layout_col = true;
                        } else if col_name == "is_sticky" {
                            has_is_sticky_col = true;
                        }
                    }
                }
            }
        }
        if !has_layout_col {
            let _ = conn.execute(
                "ALTER TABLE posts ADD COLUMN layout_json TEXT DEFAULT NULL;",
                [],
            );
        }
        if !has_is_sticky_col {
            let _ = conn.execute(
                "ALTER TABLE posts ADD COLUMN is_sticky INTEGER NOT NULL DEFAULT 0;",
                [],
            );
        }
        Ok(())
    }

    pub fn seed_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 1. Ensure essential default options are present
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let default_options = [
            ("blogname", "WordPress Reference"),
            ("blogdescription", "Next-Gen All-in-One Rust CMS"),
            ("siteurl", "http://localhost:8090"),
            ("home", "http://localhost:8090"),
            ("admin_email", "admin@example.com"),
            ("posts_per_page", "10"),
            ("date_format", "F j, Y"),
            ("time_format", "g:i a"),
        ];

        for (name, val) in default_options {
            conn.execute(
                "INSERT OR IGNORE INTO options (option_name, option_value) VALUES (?1, ?2);",
                params![name, val],
            )?;
        }

        // 2. Ensure admin user exists. In-memory databases are test fixtures, so they use the
        // well-known admin/admin login; file databases never do (see `ensure_admin_credentials`).
        conn.execute(
            r#"INSERT OR IGNORE INTO users (ID, user_login, user_pass, user_nicename, user_email, user_registered, display_name)
               VALUES (1, 'admin', ?2, 'admin', 'admin@example.com', ?1, 'admin');"#,
            params![now, crate::auth::hash_user_password("admin")],
        )?;

        // 3. Ensure reference posts exist
        conn.execute(
            r#"INSERT OR IGNORE INTO posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
               VALUES (1, 1, ?1, ?1, '<!-- wp:paragraph -->\n<p>Welcome to Daypress. This is your first post. Edit or delete it, then start writing!</p>\n<!-- /wp:paragraph -->', 'Hello world!', '<p>Welcome to Daypress. This is your first post. Edit or delete it, then start writing!</p>', 'publish', 'hello-world', ?1, ?1, 'http://localhost:8090/?p=1', 'post');"#,
            params![now],
        )?;

        conn.execute(
            r#"INSERT OR IGNORE INTO posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
               VALUES (2, 1, ?1, ?1, '<!-- wp:paragraph -->\n<p>This is an example page. It is different from a blog post because it will stay in one place and will show up in your site navigation.</p>\n<!-- /wp:paragraph -->', 'Sample Page', '<p>This is an example page.</p>', 'publish', 'sample-page', ?1, ?1, 'http://localhost:8090/?page_id=2', 'page');"#,
            params![now],
        )?;

        conn.execute(
            r#"INSERT OR IGNORE INTO posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
               VALUES (4, 1, ?1, ?1, '<p>This is reference content for schema extraction and verification.</p>', 'Welcome to Daypress Reference', '<p>This is reference content for schema extraction and verification.</p>', 'publish', 'welcome-to-daypress-reference', ?1, ?1, 'http://localhost:8090/?p=4', 'post');"#,
            params![now],
        )?;

        // 4. Ensure Uncategorized category exists
        conn.execute(
            "INSERT OR IGNORE INTO terms (term_id, name, slug) VALUES (1, 'Uncategorized', 'uncategorized');",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO term_taxonomy (term_taxonomy_id, term_id, taxonomy, count) VALUES (1, 1, 'category', 2);",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO term_relationships (object_id, term_taxonomy_id) VALUES (1, 1), (4, 1);",
            [],
        )?;

        // 5. Ensure standard default comment exists
        conn.execute(
            r#"INSERT OR IGNORE INTO comments (comment_ID, comment_post_ID, comment_author, comment_author_email, comment_author_url, comment_author_IP, comment_date, comment_date_gmt, comment_content, comment_karma, comment_approved, comment_agent, comment_type, comment_parent, user_id)
               VALUES (1, 1, 'A Daypress Commenter', 'commenter@example.com', 'https://wordpress.org/', '127.0.0.1', ?1, ?1, 'Hi, this is a comment.\nTo get started with moderating, editing, and deleting comments, please visit the Comments screen in the dashboard.\nCommenter avatars come from Gravatar.', 0, '1', '', 'comment', 0, 0);"#,
            params![now],
        )?;

        // 6. Seed First-Party Toggleable Modules & Built-in Plugins Catalog
        for manifest in crate::plugins::builtin_plugins_catalog() {
            let mut default_cfg = std::collections::HashMap::new();
            for field in &manifest.settings_schema {
                default_cfg.insert(field.key.clone(), field.default_value.clone());
            }
            let default_active =
                !matches!(manifest.slug.as_str(), "whatsapp-chat" | "code-snippets");
            let manifest_json =
                serde_json::to_string(&manifest).unwrap_or_else(|_| "{}".to_string());
            let config_json =
                serde_json::to_string(&default_cfg).unwrap_or_else(|_| "{}".to_string());
            let _ = conn.execute(
                r#"INSERT OR IGNORE INTO daypress_plugins (slug, name, version, description, category, is_builtin, is_active, manifest_json, config_json, updated_at)
                   VALUES (?1, ?2, ?3, ?4, 'builtin', 1, ?5, ?6, ?7, ?8)"#,
                params![
                    manifest.slug,
                    manifest.name,
                    manifest.version,
                    manifest.description,
                    if default_active { 1 } else { 0 },
                    manifest_json,
                    config_json,
                    now
                ],
            );
        }

        // 7. Seed a default Contact Form if daypress_forms is empty
        let form_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM daypress_forms", [], |r| r.get(0))
            .unwrap_or(0);
        if form_count == 0 {
            let default_fields = crate::convergence::default_contact_fields();
            let default_settings = FormSettings::default();
            let schema_json =
                serde_json::to_string(&default_fields).unwrap_or_else(|_| "[]".to_string());
            let settings_json =
                serde_json::to_string(&default_settings).unwrap_or_else(|_| "{}".to_string());
            let _ = conn.execute(
                "INSERT INTO daypress_forms (id, title, schema_json, settings_json, created_at, updated_at) VALUES (1, 'General Contact Form', ?1, ?2, ?3, ?3)",
                params![schema_json, settings_json, now],
            );
        }

        Ok(())
    }
}
