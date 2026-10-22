#![allow(unused_imports)]
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use super::Database;
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
        };

        db.init_schema()?;

        if path != Path::new(":memory:") {
            let needs_official_seed = {
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
                !is_migrated && (is_new || post_count == 0)
            };

            if needs_official_seed {
                db.seed_neopress_official_site()?;
            }
        } else if is_new {
            db.seed_data()?;
        }

        Ok(db)
    }

    /// Seeds the official Neopress marketing website, product pages, and high-converting articles.
    pub fn seed_neopress_official_site(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        crate::seed::seed_neopress_official_site(&conn)
    }

    /// Creates an in-memory SQLite database instance (ideal for automated unit and integration tests).
    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
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
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS wp_posts (
                ID INTEGER PRIMARY KEY AUTOINCREMENT,
                post_author INTEGER NOT NULL DEFAULT 1,
                post_date TEXT NOT NULL,
                post_date_gmt TEXT NOT NULL,
                post_content TEXT NOT NULL DEFAULT '',
                post_title TEXT NOT NULL DEFAULT '',
                post_excerpt TEXT NOT NULL DEFAULT '',
                post_status TEXT NOT NULL DEFAULT 'publish',
                comment_status TEXT NOT NULL DEFAULT 'open',
                ping_status TEXT NOT NULL DEFAULT 'open',
                post_password TEXT NOT NULL DEFAULT '',
                post_name TEXT NOT NULL DEFAULT '',
                to_ping TEXT NOT NULL DEFAULT '',
                pinged TEXT NOT NULL DEFAULT '',
                post_modified TEXT NOT NULL,
                post_modified_gmt TEXT NOT NULL,
                post_content_filtered TEXT NOT NULL DEFAULT '',
                post_parent INTEGER NOT NULL DEFAULT 0,
                guid TEXT NOT NULL DEFAULT '',
                menu_order INTEGER NOT NULL DEFAULT 0,
                post_type TEXT NOT NULL DEFAULT 'post',
                post_mime_type TEXT NOT NULL DEFAULT '',
                comment_count INTEGER NOT NULL DEFAULT 0,
                layout_json TEXT DEFAULT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_post_name ON wp_posts(post_name);
            CREATE INDEX IF NOT EXISTS idx_type_status_date ON wp_posts(post_type, post_status, post_date, ID);

            CREATE TABLE IF NOT EXISTS wp_postmeta (
                meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
                post_id INTEGER NOT NULL DEFAULT 0,
                meta_key TEXT DEFAULT NULL,
                meta_value TEXT DEFAULT NULL
            );

            CREATE TABLE IF NOT EXISTS wp_options (
                option_id INTEGER PRIMARY KEY AUTOINCREMENT,
                option_name TEXT NOT NULL UNIQUE,
                option_value TEXT NOT NULL,
                autoload TEXT NOT NULL DEFAULT 'yes'
            );

            CREATE TABLE IF NOT EXISTS wp_users (
                ID INTEGER PRIMARY KEY AUTOINCREMENT,
                user_login TEXT NOT NULL UNIQUE,
                user_pass TEXT NOT NULL,
                user_nicename TEXT NOT NULL DEFAULT '',
                user_email TEXT NOT NULL DEFAULT '',
                user_url TEXT NOT NULL DEFAULT '',
                user_registered TEXT NOT NULL,
                user_activation_key TEXT NOT NULL DEFAULT '',
                user_status INTEGER NOT NULL DEFAULT 0,
                display_name TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS wp_terms (
                term_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                term_group INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS wp_term_taxonomy (
                term_taxonomy_id INTEGER PRIMARY KEY AUTOINCREMENT,
                term_id INTEGER NOT NULL,
                taxonomy TEXT NOT NULL DEFAULT 'category',
                description TEXT NOT NULL DEFAULT '',
                parent INTEGER NOT NULL DEFAULT 0,
                count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS wp_term_relationships (
                object_id INTEGER NOT NULL,
                term_taxonomy_id INTEGER NOT NULL,
                term_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (object_id, term_taxonomy_id)
            );

            CREATE TABLE IF NOT EXISTS wp_comments (
                comment_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                comment_post_ID INTEGER NOT NULL DEFAULT 0,
                comment_author TEXT NOT NULL,
                comment_author_email TEXT NOT NULL DEFAULT '',
                comment_author_url TEXT NOT NULL DEFAULT '',
                comment_author_IP TEXT NOT NULL DEFAULT '',
                comment_date TEXT NOT NULL,
                comment_date_gmt TEXT NOT NULL,
                comment_content TEXT NOT NULL,
                comment_karma INTEGER NOT NULL DEFAULT 0,
                comment_approved TEXT NOT NULL DEFAULT '1',
                comment_agent TEXT NOT NULL DEFAULT '',
                comment_type TEXT NOT NULL DEFAULT 'comment',
                comment_parent INTEGER NOT NULL DEFAULT 0,
                user_id INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS wp_application_passwords (
                uuid TEXT PRIMARY KEY,
                app_id TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                app_name TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_used TEXT DEFAULT NULL
            );

            CREATE TABLE IF NOT EXISTS wp_usermeta (
                umeta_id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL DEFAULT 0,
                meta_key TEXT DEFAULT NULL,
                meta_value TEXT DEFAULT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_user_id ON wp_usermeta(user_id);
                PRAGMA user_version = 1;
                "#,
            )?;
        }

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS wp_application_passwords (
                uuid TEXT PRIMARY KEY,
                app_id TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                app_name TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_used TEXT DEFAULT NULL
            );
            CREATE TABLE IF NOT EXISTS wp_usermeta (
                umeta_id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL DEFAULT 0,
                meta_key TEXT DEFAULT NULL,
                meta_value TEXT DEFAULT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_user_id ON wp_usermeta(user_id);

            -- Native Subsystems: Transactional Email Queue & Delivery Logs
            CREATE TABLE IF NOT EXISTS wp_neopress_email_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recipient TEXT NOT NULL,
                subject TEXT NOT NULL,
                transport TEXT NOT NULL DEFAULT 'smtp',
                status TEXT NOT NULL DEFAULT 'delivered',
                latency_ms INTEGER NOT NULL DEFAULT 0,
                error_message TEXT DEFAULT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS wp_neopress_email_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recipient TEXT NOT NULL,
                subject TEXT NOT NULL,
                body_html TEXT NOT NULL DEFAULT '',
                body_text TEXT NOT NULL DEFAULT '',
                headers_json TEXT NOT NULL DEFAULT '{}',
                attempts INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL
            );

            -- Native Subsystems: Declarative Forms & Submissions
            CREATE TABLE IF NOT EXISTS wp_neopress_forms (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                schema_json TEXT NOT NULL DEFAULT '{}',
                settings_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS wp_neopress_form_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                form_id INTEGER NOT NULL,
                data_json TEXT NOT NULL DEFAULT '{}',
                ip_hash TEXT NOT NULL DEFAULT '',
                user_agent TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );

            -- Native Subsystems: Cookieless Daily Salted Analytics
            CREATE TABLE IF NOT EXISTS wp_neopress_analytics_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                visitor_hash TEXT NOT NULL,
                page_url TEXT NOT NULL,
                referrer TEXT NOT NULL DEFAULT '',
                device_type TEXT NOT NULL DEFAULT 'desktop',
                country TEXT NOT NULL DEFAULT 'XX',
                utm_source TEXT DEFAULT NULL,
                utm_medium TEXT DEFAULT NULL,
                utm_campaign TEXT DEFAULT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS wp_neopress_analytics_daily (
                date TEXT NOT NULL,
                page_url TEXT NOT NULL,
                unique_visitors INTEGER NOT NULL DEFAULT 0,
                pageviews INTEGER NOT NULL DEFAULT 0,
                bounces INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (date, page_url)
            );

            -- Native Subsystems: SEO Redirects, 404 Monitor & Plugin Ecosystem Registry
            CREATE TABLE IF NOT EXISTS wp_neopress_redirects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_path TEXT NOT NULL UNIQUE,
                target_url TEXT NOT NULL,
                status_code INTEGER NOT NULL DEFAULT 301,
                hits INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_redirects_source ON wp_neopress_redirects(source_path);

            CREATE TABLE IF NOT EXISTS wp_neopress_404_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_path TEXT NOT NULL UNIQUE,
                referrer TEXT NOT NULL DEFAULT '',
                user_agent TEXT NOT NULL DEFAULT '',
                hits INTEGER NOT NULL DEFAULT 1,
                last_seen_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS wp_neopress_plugins (
                slug TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL DEFAULT '1.0.0',
                description TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT 'general',
                is_builtin INTEGER NOT NULL DEFAULT 1,
                is_active INTEGER NOT NULL DEFAULT 1,
                manifest_json TEXT NOT NULL DEFAULT '{}',
                config_json TEXT NOT NULL DEFAULT '{}',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        )?;

        // Ensure layout_json column exists on existing wp_posts tables
        let mut has_layout_col = false;
        if let Ok(mut stmt) = conn.prepare("PRAGMA table_info(wp_posts);") {
            if let Ok(mut rows) = stmt.query([]) {
                while let Ok(Some(row)) = rows.next() {
                    if let Ok(col_name) = row.get::<_, String>(1) {
                        if col_name == "layout_json" {
                            has_layout_col = true;
                            break;
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
        Ok(())
    }

    pub fn seed_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 1. Check if schemas/wordpress-core.sqlite exists and attach to import (skipped for in-memory test databases)
        let ref_sqlite = Path::new("schemas/wordpress-core.sqlite");
        if self.db_path != Path::new(":memory:") && ref_sqlite.exists() {
            println!("📥 Found reference database at schemas/wordpress-core.sqlite, importing core records...");
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

        // 3. Ensure admin user exists
        conn.execute(
            r#"INSERT OR IGNORE INTO wp_users (ID, user_login, user_pass, user_nicename, user_email, user_registered, display_name)
               VALUES (1, 'admin', '$2y$10$abcdefghijklmnopqrstuvwxyz1234567890', 'admin', 'admin@example.com', ?1, 'admin');"#,
            params![now],
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
            let default_active = !matches!(manifest.slug.as_str(), "whatsapp-chat" | "code-snippets");
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
