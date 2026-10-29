use serde::{Deserialize, Serialize};

use super::common::NavMenuItemDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpSettingsResponse {
    pub title: String,
    pub description: String,
    pub url: String,
    pub email: String,
    pub timezone: String,
    pub date_format: String,
    pub time_format: String,
    pub start_of_week: i64,
    pub language: String,
    pub use_smilies: bool,
    pub default_category: i64,
    pub default_post_format: String,
    pub posts_per_page: i64,
    #[serde(default = "default_show_on_front")]
    pub show_on_front: String,
    #[serde(default)]
    pub page_on_front: i64,
    #[serde(default)]
    pub page_for_posts: i64,
    #[serde(default = "default_active_theme")]
    pub active_theme: String,
    #[serde(default)]
    pub custom_css: String,
    #[serde(default = "default_brand_color")]
    pub brand_color: String,
    #[serde(default = "default_typography_scale")]
    pub typography_scale: String,
    #[serde(default)]
    pub nav_menu_items: Vec<NavMenuItemDto>,
    #[serde(default)]
    pub widgets_json: String,
    #[serde(default)]
    pub active_plugins: Vec<String>,
    #[serde(default)]
    pub pending_comments_count: i64,
    /// Settings → Reading "Search engine visibility" (`blog_public`); false adds noindex sitewide.
    #[serde(default = "default_blog_public")]
    pub blog_public: bool,
    /// Page chosen under Settings → Privacy (`wp_page_for_privacy_policy`); 0 when unset.
    #[serde(default)]
    pub privacy_policy_page_id: i64,
    #[serde(default)]
    pub elementor_kit_css: String,
    #[serde(default)]
    pub elementor_header_html: String,
    #[serde(default)]
    pub elementor_footer_html: String,
    #[serde(default)]
    pub elementor_popups_html: String,
    #[serde(default)]
    pub site_logo_url: String,
    #[serde(default)]
    pub site_favicon_url: String,
    #[serde(default)]
    pub custom_post_types: Vec<String>,
    #[serde(default)]
    pub theme_assets_html: String,
}

pub fn default_show_on_front() -> String {
    "posts".to_string()
}

pub fn default_active_theme() -> String {
    "twentytwentyfour".to_string()
}

pub fn default_brand_color() -> String {
    "#2271b1".to_string()
}

pub fn default_blog_public() -> bool {
    true
}

pub fn default_typography_scale() -> String {
    "Inter (Modern Default)".to_string()
}

impl Default for WpSettingsResponse {
    fn default() -> Self {
        Self {
            title: "Daypress".to_string(),
            description: "Next-Gen All-in-One Rust CMS".to_string(),
            url: "http://localhost:8090".to_string(),
            email: "admin@example.com".to_string(),
            timezone: "UTC".to_string(),
            date_format: "F j, Y".to_string(),
            time_format: "g:i a".to_string(),
            start_of_week: 1,
            language: "en_US".to_string(),
            use_smilies: true,
            default_category: 1,
            default_post_format: "standard".to_string(),
            posts_per_page: 10,
            show_on_front: default_show_on_front(),
            page_on_front: 0,
            page_for_posts: 0,
            active_theme: default_active_theme(),
            custom_css: String::new(),
            brand_color: default_brand_color(),
            typography_scale: default_typography_scale(),
            nav_menu_items: Vec::new(),
            widgets_json: String::new(),
            active_plugins: vec![
                "daypress-seo".to_string(),
                "daypress-forms".to_string(),
                "daypress-backups".to_string(),
                "daypress-wal-optimizer".to_string(),
                "webp-image-engine".to_string(),
                "wxr-streaming-xml-importer".to_string(),
            ],
            pending_comments_count: 0,
            blog_public: true,
            privacy_policy_page_id: 0,
            elementor_kit_css: String::new(),
            elementor_header_html: String::new(),
            elementor_footer_html: String::new(),
            elementor_popups_html: String::new(),
            site_logo_url: String::new(),
            site_favicon_url: String::new(),
            custom_post_types: Vec::new(),
            theme_assets_html: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSettingsRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub timezone_string: Option<String>,
    #[serde(default)]
    pub date_format: Option<String>,
    #[serde(default)]
    pub time_format: Option<String>,
    #[serde(default)]
    pub start_of_week: Option<i64>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub use_smilies: Option<bool>,
    #[serde(default)]
    pub default_category: Option<i64>,
    #[serde(default)]
    pub default_post_format: Option<String>,
    #[serde(default)]
    pub posts_per_page: Option<i64>,
    #[serde(default)]
    pub default_comment_status: Option<String>,
    #[serde(default)]
    pub show_on_front: Option<String>,
    #[serde(default)]
    pub page_on_front: Option<i64>,
    #[serde(default)]
    pub page_for_posts: Option<i64>,
    #[serde(default)]
    pub permalink_structure: Option<String>,
    #[serde(default)]
    pub active_theme: Option<String>,
    #[serde(default)]
    pub brand_color: Option<String>,
    #[serde(default)]
    pub typography_scale: Option<String>,
    #[serde(default)]
    pub custom_css: Option<String>,
    #[serde(default)]
    pub active_plugins: Option<Vec<String>>,
    #[serde(default)]
    pub nav_menu_items: Option<Vec<NavMenuItemDto>>,
    #[serde(default)]
    pub widgets: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingSettings {
    pub default_category: i64,
    pub default_post_format: String,
    pub use_smilies: bool,
    pub default_pingback_flag: bool,
    pub mailserver_url: String,
    pub mailserver_login: String,
    pub mailserver_pass: String,
    pub mailserver_port: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingSettings {
    pub show_on_front: String,
    pub page_on_front: i64,
    pub page_for_posts: i64,
    pub posts_per_page: i64,
    pub posts_per_rss: i64,
    pub blog_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionSettings {
    pub default_ping_status: String,
    pub default_comment_status: String,
    pub require_name_email: bool,
    pub comment_registration: bool,
    pub close_comments_for_old_posts: bool,
    pub close_comments_days_old: i64,
    pub thread_comments: bool,
    pub thread_comments_depth: i64,
    pub comment_moderation: bool,
    pub comment_previously_approved: bool,
    pub show_avatars: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSettings {
    pub thumbnail_size_w: i64,
    pub thumbnail_size_h: i64,
    pub thumbnail_crop: bool,
    pub medium_size_w: i64,
    pub medium_size_h: i64,
    pub large_size_w: i64,
    pub large_size_h: i64,
    pub uploads_use_yearmonth_folders: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermalinkSettings {
    pub permalink_structure: String,
    pub category_base: String,
    pub tag_base: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub wp_page_for_privacy_policy: i64,
}
