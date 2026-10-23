//! Submenu Settings persistence (Writing, Reading, Discussion, Media, Permalinks, Privacy).

use super::Database;
use crate::models::*;
use rusqlite::Result;

impl Database {
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
