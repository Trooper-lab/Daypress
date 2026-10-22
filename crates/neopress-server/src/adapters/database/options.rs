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
    pub fn get_option(&self, name: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT option_value FROM wp_options WHERE option_name = ?1")?;
        let mut rows = stmt.query_map(params![name], |r| r.get(0))?;
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    pub fn set_option(&self, name: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO wp_options (option_name, option_value) VALUES (?1, ?2)
               ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;"#,
            params![name, value],
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<WpSettingsResponse> {
        let pending_comments_count: i64 = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT COUNT(*) FROM wp_comments WHERE comment_approved = '0' OR comment_approved = 'hold' OR comment_approved = 'pending'",
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
            timezone: self
                .get_option("timezone_string")?
                .unwrap_or_default(),
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
                .or_else(|| self.get_option("neopress_imported_theme").ok().flatten())
                .unwrap_or_else(|| "twentytwentyfour".to_string()),
            custom_css: self
                .get_option("custom_css")?
                .or_else(|| self.get_option("neopress_custom_css").ok().flatten())
                .unwrap_or_default(),
            brand_color: self
                .get_option("neopress_brand_color")?
                .unwrap_or_else(|| "#2271b1".to_string()),
            typography_scale: self
                .get_option("neopress_typography_scale")?
                .unwrap_or_else(|| "Inter (Modern Default)".to_string()),
            nav_menu_items: self
                .get_option("neopress_nav_menu_items")?
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            widgets_json: self
                .get_option("neopress_widgets")?
                .unwrap_or_default(),
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
                .get_option("neopress_elementor_kit_css")?
                .unwrap_or_default(),
            elementor_header_html: self
                .get_option("neopress_elementor_header_html")?
                .unwrap_or_default(),
            elementor_footer_html: self
                .get_option("neopress_elementor_footer_html")?
                .unwrap_or_default(),
            elementor_popups_html: self
                .get_option("neopress_elementor_popups_html")?
                .unwrap_or_default(),
            site_logo_url: self
                .get_option("neopress_site_logo")?
                .unwrap_or_default(),
            site_favicon_url: self
                .get_option("neopress_site_favicon")?
                .unwrap_or_default(),
        })
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
            "SELECT count(*) FROM wp_posts WHERE post_type = 'post'",
            [],
            |r| r.get(0),
        )?;
        let total_pages: i64 = conn.query_row(
            "SELECT count(*) FROM wp_posts WHERE post_type = 'page'",
            [],
            |r| r.get(0),
        )?;
        let total_comments: i64 =
            conn.query_row("SELECT count(*) FROM wp_comments", [], |r| r.get(0))?;
        let total_options: i64 =
            conn.query_row("SELECT count(*) FROM wp_options", [], |r| r.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size;", [], |r| r.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count;", [], |r| r.get(0))?;
        let journal_mode: String = conn.query_row("PRAGMA journal_mode;", [], |r| r.get(0))?;

        let file_size_bytes = fs::metadata(&self.db_path).map(|m| m.len()).unwrap_or(0);

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

    // --- Submenu Settings API ---

    pub fn get_writing_settings(&self) -> Result<WritingSettings> {
        Ok(WritingSettings {
            default_category: self
                .get_option("default_category")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            default_post_format: self
                .get_option("default_post_format")?
                .unwrap_or_else(|| "standard".to_string()),
            use_smilies: self
                .get_option("use_smilies")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            default_pingback_flag: self
                .get_option("default_pingback_flag")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            mailserver_url: self
                .get_option("mailserver_url")?
                .unwrap_or_else(|| "mail.example.com".to_string()),
            mailserver_login: self
                .get_option("mailserver_login")?
                .unwrap_or_else(|| "login@example.com".to_string()),
            mailserver_pass: self.get_option("mailserver_pass")?.unwrap_or_default(),
            mailserver_port: self
                .get_option("mailserver_port")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(110),
        })
    }

    pub fn update_writing_settings(&self, s: &WritingSettings) -> Result<()> {
        self.set_option("default_category", &s.default_category.to_string())?;
        self.set_option("default_post_format", &s.default_post_format)?;
        self.set_option("use_smilies", if s.use_smilies { "1" } else { "0" })?;
        self.set_option(
            "default_pingback_flag",
            if s.default_pingback_flag { "1" } else { "0" },
        )?;
        self.set_option("mailserver_url", &s.mailserver_url)?;
        self.set_option("mailserver_login", &s.mailserver_login)?;
        self.set_option("mailserver_pass", &s.mailserver_pass)?;
        self.set_option("mailserver_port", &s.mailserver_port.to_string())?;
        Ok(())
    }

    pub fn get_reading_settings(&self) -> Result<ReadingSettings> {
        Ok(ReadingSettings {
            show_on_front: self
                .get_option("show_on_front")?
                .unwrap_or_else(|| "posts".to_string()),
            page_on_front: self
                .get_option("page_on_front")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            page_for_posts: self
                .get_option("page_for_posts")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            posts_per_page: self
                .get_option("posts_per_page")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            posts_per_rss: self
                .get_option("posts_per_rss")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            blog_public: self
                .get_option("blog_public")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
        })
    }

    pub fn update_reading_settings(&self, s: &ReadingSettings) -> Result<()> {
        self.set_option("show_on_front", &s.show_on_front)?;
        self.set_option("page_on_front", &s.page_on_front.to_string())?;
        self.set_option("page_for_posts", &s.page_for_posts.to_string())?;
        self.set_option("posts_per_page", &s.posts_per_page.to_string())?;
        self.set_option("posts_per_rss", &s.posts_per_rss.to_string())?;
        self.set_option("blog_public", if s.blog_public { "1" } else { "0" })?;
        Ok(())
    }

    pub fn get_discussion_settings(&self) -> Result<DiscussionSettings> {
        Ok(DiscussionSettings {
            default_ping_status: self
                .get_option("default_ping_status")?
                .unwrap_or_else(|| "open".to_string()),
            default_comment_status: self
                .get_option("default_comment_status")?
                .unwrap_or_else(|| "open".to_string()),
            require_name_email: self
                .get_option("require_name_email")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            comment_registration: self
                .get_option("comment_registration")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
            close_comments_for_old_posts: self
                .get_option("close_comments_for_old_posts")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
            close_comments_days_old: self
                .get_option("close_comments_days_old")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(14),
            thread_comments: self
                .get_option("thread_comments")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            thread_comments_depth: self
                .get_option("thread_comments_depth")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            comment_moderation: self
                .get_option("comment_moderation")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
            comment_previously_approved: self
                .get_option("comment_previously_approved")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            show_avatars: self
                .get_option("show_avatars")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
        })
    }

    pub fn update_discussion_settings(&self, s: &DiscussionSettings) -> Result<()> {
        self.set_option("default_ping_status", &s.default_ping_status)?;
        self.set_option("default_comment_status", &s.default_comment_status)?;
        self.set_option(
            "require_name_email",
            if s.require_name_email { "1" } else { "0" },
        )?;
        self.set_option(
            "comment_registration",
            if s.comment_registration { "1" } else { "0" },
        )?;
        self.set_option(
            "close_comments_for_old_posts",
            if s.close_comments_for_old_posts {
                "1"
            } else {
                "0"
            },
        )?;
        self.set_option(
            "close_comments_days_old",
            &s.close_comments_days_old.to_string(),
        )?;
        self.set_option("thread_comments", if s.thread_comments { "1" } else { "0" })?;
        self.set_option(
            "thread_comments_depth",
            &s.thread_comments_depth.to_string(),
        )?;
        self.set_option(
            "comment_moderation",
            if s.comment_moderation { "1" } else { "0" },
        )?;
        self.set_option(
            "comment_previously_approved",
            if s.comment_previously_approved {
                "1"
            } else {
                "0"
            },
        )?;
        self.set_option("show_avatars", if s.show_avatars { "1" } else { "0" })?;
        Ok(())
    }

    pub fn get_media_settings(&self) -> Result<MediaSettings> {
        Ok(MediaSettings {
            thumbnail_size_w: self
                .get_option("thumbnail_size_w")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(150),
            thumbnail_size_h: self
                .get_option("thumbnail_size_h")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(150),
            thumbnail_crop: self
                .get_option("thumbnail_crop")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            medium_size_w: self
                .get_option("medium_size_w")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            medium_size_h: self
                .get_option("medium_size_h")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            large_size_w: self
                .get_option("large_size_w")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024),
            large_size_h: self
                .get_option("large_size_h")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024),
            uploads_use_yearmonth_folders: self
                .get_option("uploads_use_yearmonth_folders")?
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
        })
    }

    pub fn update_media_settings(&self, s: &MediaSettings) -> Result<()> {
        self.set_option("thumbnail_size_w", &s.thumbnail_size_w.to_string())?;
        self.set_option("thumbnail_size_h", &s.thumbnail_size_h.to_string())?;
        self.set_option("thumbnail_crop", if s.thumbnail_crop { "1" } else { "0" })?;
        self.set_option("medium_size_w", &s.medium_size_w.to_string())?;
        self.set_option("medium_size_h", &s.medium_size_h.to_string())?;
        self.set_option("large_size_w", &s.large_size_w.to_string())?;
        self.set_option("large_size_h", &s.large_size_h.to_string())?;
        self.set_option(
            "uploads_use_yearmonth_folders",
            if s.uploads_use_yearmonth_folders {
                "1"
            } else {
                "0"
            },
        )?;
        Ok(())
    }

    pub fn get_permalink_settings(&self) -> Result<PermalinkSettings> {
        Ok(PermalinkSettings {
            permalink_structure: self
                .get_option("permalink_structure")?
                .unwrap_or_else(|| "/%postname%/".to_string()),
            category_base: self.get_option("category_base")?.unwrap_or_default(),
            tag_base: self.get_option("tag_base")?.unwrap_or_default(),
        })
    }

    pub fn update_permalink_settings(&self, s: &PermalinkSettings) -> Result<()> {
        self.set_option("permalink_structure", &s.permalink_structure)?;
        self.set_option("category_base", &s.category_base)?;
        self.set_option("tag_base", &s.tag_base)?;
        Ok(())
    }

    pub fn get_privacy_settings(&self) -> Result<PrivacySettings> {
        Ok(PrivacySettings {
            wp_page_for_privacy_policy: self
                .get_option("wp_page_for_privacy_policy")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
        })
    }

    pub fn update_privacy_settings(&self, s: &PrivacySettings) -> Result<()> {
        self.set_option(
            "wp_page_for_privacy_policy",
            &s.wp_page_for_privacy_policy.to_string(),
        )?;
        Ok(())
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
