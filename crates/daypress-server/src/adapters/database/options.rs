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
    pub fn get_option(&self, name: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT option_value FROM options WHERE option_name = ?1")?;
        let mut rows = stmt.query_map(params![name], |r| r.get(0))?;
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    pub fn set_option(&self, name: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO options (option_name, option_value) VALUES (?1, ?2)
               ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;"#,
            params![name, value],
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<WpSettingsResponse> {
        let pending_comments_count: i64 = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT COUNT(*) FROM comments WHERE comment_approved = '0' OR comment_approved = 'hold' OR comment_approved = 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0)
        };

        Ok(WpSettingsResponse {
            title: self
                .get_option("blogname")?
                .unwrap_or_else(|| "WordPress Reference".to_string()),
            description: self
                .get_option("blogdescription")?
                .unwrap_or_else(|| "".to_string()),
            url: self
                .get_option("siteurl")?
                .unwrap_or_else(|| "http://localhost:8090".to_string()),
            email: self
                .get_option("admin_email")?
                .unwrap_or_else(|| "admin@example.com".to_string()),
            timezone: self.get_option("timezone_string")?.unwrap_or_default(),
            date_format: self
                .get_option("date_format")?
                .unwrap_or_else(|| "F j, Y".to_string()),
            time_format: self
                .get_option("time_format")?
                .unwrap_or_else(|| "g:i a".to_string()),
            start_of_week: self
                .get_option("start_of_week")?
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
            language: self
                .get_option("WPLANG")?
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "en_US".to_string()),
            use_smilies: self
                .get_option("use_smilies")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            default_category: self
                .get_option("default_category")?
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
            default_post_format: self
                .get_option("default_post_format")?
                .unwrap_or_else(|| "standard".to_string()),
            posts_per_page: self
                .get_option("posts_per_page")?
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            show_on_front: self
                .get_option("show_on_front")?
                .unwrap_or_else(|| "posts".to_string()),
            page_on_front: self
                .get_option("page_on_front")?
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            page_for_posts: self
                .get_option("page_for_posts")?
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            active_theme: self
                .get_option("stylesheet")?
                .or_else(|| self.get_option("daypress_imported_theme").ok().flatten())
                .unwrap_or_else(|| "twentytwentyfour".to_string()),
            custom_css: self
                .get_option("custom_css")?
                .or_else(|| self.get_option("daypress_custom_css").ok().flatten())
                .unwrap_or_default(),
            brand_color: self
                .get_option("daypress_brand_color")?
                .unwrap_or_else(|| "#2271b1".to_string()),
            typography_scale: self
                .get_option("daypress_typography_scale")?
                .unwrap_or_else(|| "Inter (Modern Default)".to_string()),
            nav_menu_items: self
                .get_option("daypress_nav_menu_items")?
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            widgets_json: self.get_option("daypress_widgets")?.unwrap_or_default(),
            active_plugins: self
                .get_option("active_plugins")?
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| {
                    vec![
                        "elementor".to_string(),
                        "woocommerce".to_string(),
                        "yoast-seo".to_string(),
                    ]
                }),
            pending_comments_count,
            elementor_kit_css: self
                .get_option("daypress_elementor_kit_css")?
                .filter(|s| !s.trim().is_empty())
                .or_else(|| self.get_option("elementor_active_kit_css").ok().flatten())
                .unwrap_or_default(),
            elementor_header_html: self
                .get_option("daypress_elementor_header_html")?
                .unwrap_or_default(),
            elementor_footer_html: self
                .get_option("daypress_elementor_footer_html")?
                .unwrap_or_default(),
            elementor_popups_html: self
                .get_option("daypress_elementor_popups_html")?
                .unwrap_or_default(),
            blog_public: self
                .get_option("blog_public")?
                .map(|v| !matches!(v.trim(), "0" | "false"))
                .unwrap_or(true),
            privacy_policy_page_id: self
                .get_option("wp_page_for_privacy_policy")?
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0),
            site_logo_url: self.get_option("daypress_site_logo")?.unwrap_or_default(),
            site_favicon_url: self
                .get_option("daypress_site_favicon")?
                .unwrap_or_default(),
            custom_post_types: self.get_registered_post_types().unwrap_or_default(),
            theme_assets_html: self
                .get_option("daypress_theme_assets")?
                .map(|raw| {
                    if let Ok(manifest) =
                        serde_json::from_str::<crate::theme::ThemeAssetsManifest>(&raw)
                    {
                        manifest.compile_to_html()
                    } else {
                        raw
                    }
                })
                .unwrap_or_default(),
        })
    }

    pub fn get_registered_post_types(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT post_type FROM posts \
             WHERE post_type NOT IN ('post', 'page', 'attachment', 'revision', 'nav_menu_item', 'wp_global_styles', 'custom_css', 'customize_changeset', 'oembed_cache') \
               AND post_type IS NOT NULL \
               AND post_type != '' \
             ORDER BY post_type ASC"
        )?;
        let rows = stmt.query_map([], |r| r.get(0))?;
        let mut types = Vec::new();
        for t in rows.flatten() {
            types.push(t);
        }
        Ok(types)
    }

    pub fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("VACUUM;")?;
        Ok(())
    }

    pub fn integrity_check(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let res: String = conn.query_row("PRAGMA integrity_check;", [], |r| r.get(0))?;
        Ok(res)
    }

    pub fn get_stats(&self) -> Result<DbStats> {
        let conn = self.conn.lock().unwrap();
        let total_posts: i64 = conn.query_row(
            "SELECT count(*) FROM posts WHERE post_type = 'post'",
            [],
            |r| r.get(0),
        )?;
        let total_pages: i64 = conn.query_row(
            "SELECT count(*) FROM posts WHERE post_type = 'page'",
            [],
            |r| r.get(0),
        )?;
        let total_comments: i64 =
            conn.query_row("SELECT count(*) FROM comments", [], |r| r.get(0))?;
        let total_options: i64 =
            conn.query_row("SELECT count(*) FROM options", [], |r| r.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size;", [], |r| r.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count;", [], |r| r.get(0))?;
        let journal_mode: String = conn.query_row("PRAGMA journal_mode;", [], |r| r.get(0))?;

        // WAL mode keeps recent writes in `<db>-wal` until a checkpoint, so count both files.
        let wal_path = format!("{}-wal", self.db_path.to_string_lossy());
        let file_size_bytes = fs::metadata(&self.db_path).map(|m| m.len()).unwrap_or(0)
            + fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);

        Ok(DbStats {
            db_path: self.db_path.to_string_lossy().to_string(),
            file_size_bytes,
            total_posts,
            total_pages,
            total_comments,
            total_options,
            page_size,
            page_count,
            journal_mode,
        })
    }
}

impl crate::ports::output::OptionRepository for Database {
    fn get_option(&self, name: &str) -> DomainResult<Option<String>> {
        Database::get_option(self, name).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn set_option(&self, name: &str, value: &str) -> DomainResult<()> {
        Database::set_option(self, name, value).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_settings(&self) -> DomainResult<WpSettingsResponse> {
        Database::get_settings(self).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_stats(&self) -> DomainResult<DbStats> {
        Database::get_stats(self).map_err(|e| DomainError::Storage(e.to_string()))
    }
}
