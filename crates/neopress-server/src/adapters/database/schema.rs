#![allow(unused_imports)]
use super::Database;
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
                        "SELECT 1 FROM wp_options WHERE option_name = 'neopress_site_migrated' AND option_value = '1' LIMIT 1;",
                        [],
                        |_| Ok(true),
                    )
                    .unwrap_or(false);
                let post_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM wp_posts;", [], |r| r.get(0))
                    .unwrap_or(0);
                let user_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM wp_users;", [], |r| r.get(0))
                    .unwrap_or(0);
                // Only a brand-new (or completely empty) database gets the starter site.
                // Deleting every post on a live site must never trigger a reseed.
                !is_migrated && (is_new || (post_count == 0 && user_count == 0))
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

    /// Guarantees the install has an administrator who can log in, without a shared default.
    ///
    /// Accounts still carrying the old seed placeholder (or no hash) get a fresh password:
    /// `NEOPRESS_ADMIN_PASSWORD` when set, otherwise a random one printed once to stderr.
    /// An install with no users at all gets an `admin` account the same way.
    pub fn ensure_admin_credentials(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM wp_users;", [], |r| r.get(0))?;
        if user_count == 0 {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            conn.execute(
                r#"INSERT INTO wp_users (ID, user_login, user_pass, user_nicename, user_email, user_registered, display_name)
                   VALUES (1, 'admin', ?1, 'admin', '', ?2, 'admin');"#,
                params![crate::auth::LEGACY_SEED_PASSWORD_HASH, now],
            )?;
        }

        let pending: Vec<(i64, String)> = {
            let mut stmt = conn.prepare(
                "SELECT ID, user_login FROM wp_users WHERE user_pass = ?1 OR TRIM(COALESCE(user_pass, '')) = '' ORDER BY ID;",
            )?;
            let rows = stmt.query_map(params![crate::auth::LEGACY_SEED_PASSWORD_HASH], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
            rows.collect::<Result<Vec<_>>>()?
        };

        let env_password = std::env::var("NEOPRESS_ADMIN_PASSWORD")
            .ok()
            .filter(|p| !p.trim().is_empty());
        for (index, (user_id, login)) in pending.into_iter().enumerate() {
            let from_env = index == 0 && env_password.is_some();
            let password = match (&env_password, from_env) {
                (Some(p), true) => p.clone(),
                _ => crate::auth::generate_random_password(),
            };
            conn.execute(
                "UPDATE wp_users SET user_pass = ?1 WHERE ID = ?2;",
                params![crate::auth::hash_user_password(&password), user_id],
            )?;
            // stderr, so `--mcp-stdio` keeps a clean JSON-RPC stdout.
            eprintln!("============================================================");
            if from_env {
                eprintln!("🔐 Admin password for '{login}' set from NEOPRESS_ADMIN_PASSWORD.");
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
            "SELECT option_value FROM wp_options WHERE option_name = 'siteurl'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "http://localhost:8090".to_string())
    }

    /// Dynamically derives site base URL from wp_options (defaults to http://localhost:8090).
    pub fn get_base_url(&self) -> String {
        let conn = self.conn.lock().unwrap();
        Self::get_base_url_with_conn(&conn)
    }

    pub(crate) fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
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

        // Ensure layout_json and is_sticky columns exist on existing wp_posts tables
        let mut has_layout_col = false;
        let mut has_is_sticky_col = false;
        if let Ok(mut stmt) = conn.prepare("PRAGMA table_info(wp_posts);") {
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
                "ALTER TABLE wp_posts ADD COLUMN layout_json TEXT DEFAULT NULL;",
                [],
            );
        }
        if !has_is_sticky_col {
            let _ = conn.execute(
                "ALTER TABLE wp_posts ADD COLUMN is_sticky INTEGER NOT NULL DEFAULT 0;",
                [],
            );
        }
        Ok(())
    }

    pub fn seed_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 1. Check if schemas/wordpress-core.sqlite exists and attach to import (skipped for in-memory test databases)
        let ref_sqlite = Path::new("schemas/wordpress-core.sqlite");
        if self.db_path != Path::new(":memory:") && ref_sqlite.exists() {
            eprintln!("📥 Found reference database at schemas/wordpress-core.sqlite, importing core records...");
            let ref_path_str = ref_sqlite.to_string_lossy().replace('\\', "/");
            let _ = conn.execute(
                &format!("ATTACH DATABASE 'file:{}?mode=ro' AS ref_db;", ref_path_str),
                [],
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO wp_options SELECT * FROM ref_db.wp_options;",
                [],
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO wp_posts SELECT * FROM ref_db.wp_posts;",
                [],
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO wp_users SELECT * FROM ref_db.wp_users;",
                [],
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO wp_terms SELECT * FROM ref_db.wp_terms;",
                [],
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO wp_term_taxonomy SELECT * FROM ref_db.wp_term_taxonomy;",
                [],
            );
            let _ = conn.execute("INSERT OR IGNORE INTO wp_term_relationships SELECT * FROM ref_db.wp_term_relationships;", []);
            let _ = conn.execute(
                "INSERT OR IGNORE INTO wp_comments SELECT * FROM ref_db.wp_comments;",
                [],
            );
            let _ = conn.execute("DETACH DATABASE ref_db;", []);
        }

        // 2. Ensure essential default options are present
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
                "INSERT OR IGNORE INTO wp_options (option_name, option_value) VALUES (?1, ?2);",
                params![name, val],
            )?;
        }

        // 3. Ensure admin user exists. In-memory databases are test fixtures, so they use the
        // well-known admin/admin login; file databases never do (see `ensure_admin_credentials`).
        conn.execute(
            r#"INSERT OR IGNORE INTO wp_users (ID, user_login, user_pass, user_nicename, user_email, user_registered, display_name)
               VALUES (1, 'admin', ?2, 'admin', 'admin@example.com', ?1, 'admin');"#,
            params![now, crate::auth::hash_user_password("admin")],
        )?;

        // 4. Ensure reference posts exist
        conn.execute(
            r#"INSERT OR IGNORE INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
               VALUES (1, 1, ?1, ?1, '<!-- wp:paragraph -->\n<p>Welcome to WordPress. This is your first post. Edit or delete it, then start writing!</p>\n<!-- /wp:paragraph -->', 'Hello world!', '<p>Welcome to WordPress. This is your first post. Edit or delete it, then start writing!</p>', 'publish', 'hello-world', ?1, ?1, 'http://localhost:8090/?p=1', 'post');"#,
            params![now],
        )?;

        conn.execute(
            r#"INSERT OR IGNORE INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
               VALUES (2, 1, ?1, ?1, '<!-- wp:paragraph -->\n<p>This is an example page. It is different from a blog post because it will stay in one place and will show up in your site navigation.</p>\n<!-- /wp:paragraph -->', 'Sample Page', '<p>This is an example page.</p>', 'publish', 'sample-page', ?1, ?1, 'http://localhost:8090/?page_id=2', 'page');"#,
            params![now],
        )?;

        conn.execute(
            r#"INSERT OR IGNORE INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
               VALUES (4, 1, ?1, ?1, '<p>This is reference content for schema extraction and verification.</p>', 'Welcome to Neopress Reference', '<p>This is reference content for schema extraction and verification.</p>', 'publish', 'welcome-to-neopress-reference', ?1, ?1, 'http://localhost:8090/?p=4', 'post');"#,
            params![now],
        )?;

        // 5. Ensure Uncategorized category exists
        conn.execute(
            "INSERT OR IGNORE INTO wp_terms (term_id, name, slug) VALUES (1, 'Uncategorized', 'uncategorized');",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO wp_term_taxonomy (term_taxonomy_id, term_id, taxonomy, count) VALUES (1, 1, 'category', 2);",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (1, 1), (4, 1);",
            [],
        )?;

        // 6. Ensure standard default comment exists
        conn.execute(
            r#"INSERT OR IGNORE INTO wp_comments (comment_ID, comment_post_ID, comment_author, comment_author_email, comment_author_url, comment_author_IP, comment_date, comment_date_gmt, comment_content, comment_karma, comment_approved, comment_agent, comment_type, comment_parent, user_id)
               VALUES (1, 1, 'A WordPress Commenter', 'wapuu@wordpress.example', 'https://wordpress.org/', '127.0.0.1', ?1, ?1, 'Hi, this is a comment.\nTo get started with moderating, editing, and deleting comments, please visit the Comments screen in the dashboard.\nCommenter avatars come from Gravatar.', 0, '1', '', 'comment', 0, 0);"#,
            params![now],
        )?;

        // 7. Seed First-Party Toggleable Modules & Built-in Plugins Catalog
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
                r#"INSERT OR IGNORE INTO wp_neopress_plugins (slug, name, version, description, category, is_builtin, is_active, manifest_json, config_json, updated_at)
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

        // 8. Seed a default Contact Form if wp_neopress_forms is empty
        let form_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM wp_neopress_forms", [], |r| r.get(0))
            .unwrap_or(0);
        if form_count == 0 {
            let default_fields = crate::convergence::default_contact_fields();
            let default_settings = FormSettings::default();
            let schema_json =
                serde_json::to_string(&default_fields).unwrap_or_else(|_| "[]".to_string());
            let settings_json =
                serde_json::to_string(&default_settings).unwrap_or_else(|_| "{}".to_string());
            let _ = conn.execute(
                "INSERT INTO wp_neopress_forms (id, title, schema_json, settings_json, created_at, updated_at) VALUES (1, 'General Contact Form', ?1, ?2, ?3, ?3)",
                params![schema_json, settings_json, now],
            );
        }

        Ok(())
    }
}
