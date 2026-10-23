use serde::{Deserialize, Serialize};

use super::common::NavMenuItemDto;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportSiteSettings {
    pub title: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub posts_per_page: Option<i64>,
    #[serde(default)]
    pub show_on_front: Option<String>,
    #[serde(default)]
    pub page_on_front: Option<i64>,
    #[serde(default)]
    pub page_for_posts: Option<i64>,
    #[serde(default)]
    pub permalink_structure: Option<String>,
    #[serde(default)]
    pub timezone_string: Option<String>,
    #[serde(default)]
    pub default_comment_status: Option<String>,
    #[serde(default)]
    pub site_icon: Option<i64>,
    #[serde(default, alias = "url", alias = "home_url", alias = "base_url")]
    pub site_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportTerm {
    pub id: Option<i64>,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parent: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportPost {
    pub id: Option<i64>,
    pub title: String,
    #[serde(default)]
    pub slug: String,
    pub content: String,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default = "default_publish_status")]
    pub status: String,
    #[serde(default = "super::posts::default_post_type")]
    pub post_type: String,
    pub date: Option<String>,
    #[serde(default)]
    pub modified: Option<String>,
    #[serde(default)]
    pub parent: Option<i64>,
    #[serde(default)]
    pub menu_order: Option<i64>,
    #[serde(default)]
    pub author: Option<i64>,
    #[serde(default)]
    pub comment_status: Option<String>,
    #[serde(default)]
    pub ping_status: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub categories: Vec<i64>,
    #[serde(default)]
    pub tags: Vec<i64>,
    pub featured_media: Option<i64>,
    #[serde(default)]
    pub postmeta: std::collections::HashMap<String, String>,
}

pub fn default_publish_status() -> String {
    "publish".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMedia {
    pub id: Option<i64>,
    pub title: String,
    pub file_name: String,
    pub mime_type: String,
    pub source_url: String,
    #[serde(default)]
    pub original_url: Option<String>,
    #[serde(default)]
    pub alt_text: Option<String>,
    pub file_size: Option<u64>,
    pub base64_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportThemeMods {
    pub active_theme: Option<String>,
    pub custom_css: Option<String>,
    #[serde(default)]
    pub custom_logo: Option<i64>,
    #[serde(default)]
    pub nav_menu_locations: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportMenu {
    pub id: Option<i64>,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default)]
    pub items: Vec<NavMenuItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportComment {
    pub id: Option<i64>,
    pub post_id: i64,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub author_email: String,
    #[serde(default)]
    pub author_url: String,
    #[serde(default)]
    pub author_ip: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub approved: String,
    #[serde(default)]
    pub parent: i64,
    #[serde(default)]
    pub user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportUser {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub nicename: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub registered: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportRedirect {
    pub source_path: String,
    pub target_url: String,
    #[serde(default = "default_redirect_code")]
    pub status_code: i64,
}

pub fn default_redirect_code() -> i64 {
    301
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportForm {
    pub id: Option<i64>,
    pub title: String,
    #[serde(default)]
    pub source_plugin: String,
    #[serde(default)]
    pub raw_form: String,
    #[serde(default)]
    pub recipient_email: String,
    #[serde(default)]
    pub schema_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportBundleRequest {
    #[serde(default, alias = "url")]
    pub site_url: Option<String>,
    pub site_settings: Option<ImportSiteSettings>,
    #[serde(default)]
    pub categories: Vec<ImportTerm>,
    #[serde(default)]
    pub tags: Vec<ImportTerm>,
    #[serde(default)]
    pub posts: Vec<ImportPost>,
    #[serde(default)]
    pub media: Vec<ImportMedia>,
    pub theme_mods: Option<ImportThemeMods>,
    #[serde(default)]
    pub menus: Vec<ImportMenu>,
    #[serde(default)]
    pub comments: Vec<ImportComment>,
    #[serde(default)]
    pub users: Vec<ImportUser>,
    #[serde(default)]
    pub widgets: Option<serde_json::Value>,
    #[serde(default)]
    pub elementor_kit: Option<serde_json::Value>,
    #[serde(default)]
    pub plugin_options: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub redirects: Vec<ImportRedirect>,
    #[serde(default)]
    pub forms: Vec<ImportForm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportBundleResponse {
    pub success: bool,
    pub migrated_posts: usize,
    pub migrated_pages: usize,
    pub migrated_categories: usize,
    pub migrated_tags: usize,
    pub migrated_media: usize,
    #[serde(default)]
    pub migrated_menus: usize,
    #[serde(default)]
    pub migrated_comments: usize,
    #[serde(default)]
    pub migrated_users: usize,
    #[serde(default)]
    pub migrated_forms: usize,
    #[serde(default)]
    pub migrated_redirects: usize,
    #[serde(default)]
    pub converged_plugins: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WxrImportResponse {
    pub success: bool,
    pub imported_posts: usize,
    pub imported_pages: usize,
    pub imported_media: usize,
    pub imported_categories: usize,
    pub imported_tags: usize,
    pub imported_comments: usize,
    pub message: String,
}
