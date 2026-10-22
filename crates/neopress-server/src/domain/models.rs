use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpGuid {
    pub rendered: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpRenderedText {
    pub rendered: String,
    #[serde(default)]
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NavMenuItemDto {
    #[serde(default)]
    pub id: i64,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub parent: i64,
    #[serde(default)]
    pub menu_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpPostResponse {
    pub id: i64,
    pub date: String,
    pub date_gmt: String,
    pub guid: WpGuid,
    pub modified: String,
    pub modified_gmt: String,
    pub slug: String,
    pub status: String,
    #[serde(rename = "type")]
    pub post_type: String,
    pub link: String,
    pub title: WpRenderedText,
    pub content: WpRenderedText,
    pub excerpt: WpRenderedText,
    pub author: i64,
    pub featured_media: i64,
    pub comment_status: String,
    pub ping_status: String,
    pub sticky: bool,
    pub template: String,
    pub format: String,
    pub meta: serde_json::Value,
    pub categories: Vec<i64>,
    pub tags: Vec<i64>,
    #[serde(default)]
    pub parent: i64,
    #[serde(default)]
    pub menu_order: i64,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub category_names: Vec<String>,
    #[serde(default)]
    pub tag_names: Vec<String>,
    #[serde(default)]
    pub featured_image_url: Option<String>,
    #[serde(default)]
    pub comments: Vec<WpCommentResponse>,
    #[serde(default)]
    pub seo_title: String,
    #[serde(default)]
    pub seo_description: String,
    #[serde(default)]
    pub seo_keyword: String,
    #[serde(default)]
    pub seo_canonical: String,
    #[serde(default)]
    pub seo_noindex: bool,
    #[serde(default)]
    pub seo_schema_type: String,
    #[serde(default)]
    pub seo_og_image: String,
}

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
}

fn default_show_on_front() -> String {
    "posts".to_string()
}

fn default_active_theme() -> String {
    "twentytwentyfour".to_string()
}

fn default_brand_color() -> String {
    "#2271b1".to_string()
}

fn default_typography_scale() -> String {
    "Inter (Modern Default)".to_string()
}

impl Default for WpSettingsResponse {
    fn default() -> Self {
        Self {
            title: "Neopress".to_string(),
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
                "neopress-seo".to_string(),
                "neopress-forms".to_string(),
                "neopress-backups".to_string(),
                "neopress-wal-optimizer".to_string(),
                "webp-image-engine".to_string(),
                "wxr-streaming-xml-importer".to_string(),
            ],
            pending_comments_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpUserResponse {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub description: String,
    pub link: String,
    pub slug: String,
    pub avatar_urls: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpCategoryResponse {
    pub id: i64,
    pub count: i64,
    pub description: String,
    pub link: String,
    pub name: String,
    pub slug: String,
    pub taxonomy: String,
    pub parent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct WpCommentResponse {
    pub id: i64,
    pub post: i64,
    pub parent: i64,
    pub author: i64,
    pub author_name: String,
    pub author_url: String,
    pub date: String,
    pub date_gmt: String,
    pub content: WpRenderedText,
    pub link: String,
    pub status: String,
    #[serde(rename = "type")]
    pub comment_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    #[serde(default = "default_post_status")]
    pub status: String,
    pub slug: Option<String>,
    #[serde(default = "default_post_type")]
    pub post_type: String,
    pub excerpt: Option<String>,
    pub parent: Option<i64>,
    pub menu_order: Option<i64>,
    pub featured_media: Option<i64>,
    #[serde(default)]
    pub categories: Vec<i64>,
    #[serde(default)]
    pub tags: Vec<i64>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub author: Option<i64>,
    #[serde(default)]
    pub seo_title: Option<String>,
    #[serde(default)]
    pub seo_description: Option<String>,
    #[serde(default)]
    pub seo_keyword: Option<String>,
    #[serde(default)]
    pub seo_canonical: Option<String>,
    #[serde(default)]
    pub seo_noindex: Option<bool>,
    #[serde(default)]
    pub seo_schema_type: Option<String>,
    #[serde(default)]
    pub seo_og_image: Option<String>,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
}

fn default_post_status() -> String {
    "publish".to_string()
}

fn default_post_type() -> String {
    "post".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub parent: Option<i64>,
    pub menu_order: Option<i64>,
    pub featured_media: Option<i64>,
    pub categories: Option<Vec<i64>>,
    pub tags: Option<Vec<i64>>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub author: Option<i64>,
    #[serde(default)]
    pub seo_title: Option<String>,
    #[serde(default)]
    pub seo_description: Option<String>,
    #[serde(default)]
    pub seo_keyword: Option<String>,
    #[serde(default)]
    pub seo_canonical: Option<String>,
    #[serde(default)]
    pub seo_noindex: Option<bool>,
    #[serde(default)]
    pub seo_schema_type: Option<String>,
    #[serde(default)]
    pub seo_og_image: Option<String>,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
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
#[allow(dead_code)]
pub struct CreateCommentRequest {
    pub post: i64,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    #[serde(default)]
    pub author_url: Option<String>,
    #[serde(default)]
    pub parent: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCommentRequest {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub content: Option<serde_json::Value>,
    #[serde(default)]
    pub author_name: Option<String>,
    #[serde(default)]
    pub author_email: Option<String>,
    #[serde(default)]
    pub author_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateTermRequest {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTermRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateMediaRequest {
    #[serde(default)]
    pub title: Option<serde_json::Value>,
    #[serde(default)]
    pub alt_text: Option<String>,
    #[serde(default)]
    pub caption: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStats {
    pub db_path: String,
    pub file_size_bytes: u64,
    pub total_posts: i64,
    pub total_pages: i64,
    pub total_comments: i64,
    pub total_options: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub journal_mode: String,
}

impl Default for DbStats {
    fn default() -> Self {
        Self {
            db_path: "data/neopress.sqlite".to_string(),
            file_size_bytes: 0,
            total_posts: 0,
            total_pages: 0,
            total_comments: 0,
            total_options: 0,
            page_size: 4096,
            page_count: 0,
            journal_mode: "wal".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaSizeDetail {
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaDetails {
    pub width: u32,
    pub height: u32,
    pub file: String,
    #[serde(default)]
    pub filesize: u64,
    #[serde(default)]
    pub sizes: std::collections::HashMap<String, MediaSizeDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpMediaResponse {
    pub id: i64,
    pub date: String,
    pub slug: String,
    pub title: WpRenderedText,
    pub source_url: String,
    pub mime_type: String,
    pub file_size: u64,
    #[serde(default = "default_media_type")]
    pub media_type: String,
    #[serde(default)]
    pub alt_text: String,
    #[serde(default)]
    pub media_details: MediaDetails,
}

fn default_media_type() -> String {
    "image".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpUserDetail {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub url: String,
    pub registered: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub url: String,
    pub registered: String,
    pub role: String,
    pub post_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

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
    #[serde(default = "default_post_type")]
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

fn default_publish_status() -> String {
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
pub struct ImportBundleRequest {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportRedirect {
    pub source_path: String,
    pub target_url: String,
    #[serde(default = "default_redirect_code")]
    pub status_code: i64,
}

fn default_redirect_code() -> i64 {
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
pub struct AppPasswordResponse {
    pub uuid: String,
    pub app_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub created: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAppPasswordRequest {
    pub name: String,
    pub app_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtTokenRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtTokenResponse {
    pub token: String,
    pub user_email: String,
    pub user_nicename: String,
    pub user_display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtValidateResponse {
    pub code: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
    pub action: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpCommentAdminItem {
    pub id: i64,
    pub post_id: i64,
    pub post_title: String,
    pub author: String,
    pub author_email: String,
    pub author_url: String,
    pub author_ip: String,
    pub content: String,
    pub date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommentCounts {
    pub all: i64,
    pub pending: i64,
    pub approved: i64,
    pub spam: i64,
    pub trash: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeopressLayoutPayload {
    pub layout: serde_json::Value,
    #[serde(default)]
    pub rendered_html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeopressForm {
    pub id: i64,
    pub title: String,
    pub schema_json: String,
    pub settings_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSubmitRequest {
    pub data: serde_json::Value,
    #[serde(default)]
    pub antispam_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEventRequest {
    pub page_url: String,
    #[serde(default)]
    pub referrer: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
    #[serde(default)]
    pub utm_source: Option<String>,
    #[serde(default)]
    pub utm_medium: Option<String>,
    #[serde(default)]
    pub utm_campaign: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_visitors: i64,
    pub total_pageviews: i64,
    pub top_pages: Vec<TopPageMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopPageMetric {
    pub page_url: String,
    pub views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLogEntry {
    pub id: i64,
    pub recipient: String,
    pub subject: String,
    pub transport: String,
    pub status: String,
    pub latency_ms: i64,
    pub error_message: Option<String>,
    pub created_at: String,
}

// =========================================================================
// PLUGIN ECOSYSTEM, SEO, REDIRECTS, FORMS, BACKUPS & COMMUNITY SDK MODELS
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectRule {
    pub id: i64,
    pub source_path: String,
    pub target_url: String,
    pub status_code: i64,
    pub hits: i64,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRedirectRequest {
    pub source_path: String,
    pub target_url: String,
    #[serde(default = "default_redirect_code")]
    pub status_code: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotFoundLogEntry {
    pub id: i64,
    pub path: String,
    pub referrer: String,
    pub user_agent: String,
    pub hits: i64,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSeoSettings {
    #[serde(default = "default_seo_separator")]
    pub title_separator: String,
    #[serde(default)]
    pub org_name: String,
    #[serde(default)]
    pub org_logo_url: String,
    #[serde(default)]
    pub default_og_image: String,
    #[serde(default)]
    pub twitter_site: String,
    #[serde(default = "default_true_bool")]
    pub enable_breadcrumbs: bool,
    #[serde(default = "default_true_bool")]
    pub auto_redirect_slug_changes: bool,
    #[serde(default)]
    pub custom_robots_txt: String,
}

fn default_seo_separator() -> String {
    "–".to_string()
}

fn default_true_bool() -> bool {
    true
}

impl Default for GlobalSeoSettings {
    fn default() -> Self {
        Self {
            title_separator: "–".to_string(),
            org_name: String::new(),
            org_logo_url: String::new(),
            default_og_image: String::new(),
            twitter_site: String::new(),
            enable_breadcrumbs: true,
            auto_redirect_slug_changes: true,
            custom_robots_txt: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormFieldSchema {
    pub name: String,
    pub label: String,
    #[serde(default = "default_field_type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub options: Vec<String>,
}

fn default_field_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSettings {
    #[serde(default = "default_submit_label")]
    pub submit_label: String,
    #[serde(default = "default_success_msg")]
    pub success_message: String,
    #[serde(default)]
    pub recipient_email: String,
    #[serde(default)]
    pub webhook_url: String,
}

fn default_submit_label() -> String {
    "Send Message".to_string()
}

fn default_success_msg() -> String {
    "Thank you! Your submission has been received.".to_string()
}

impl Default for FormSettings {
    fn default() -> Self {
        Self {
            submit_label: default_submit_label(),
            success_message: default_success_msg(),
            recipient_email: String::new(),
            webhook_url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFormRequest {
    pub id: Option<i64>,
    pub title: String,
    #[serde(default)]
    pub fields: Vec<FormFieldSchema>,
    #[serde(default)]
    pub settings: FormSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormEntryItem {
    pub id: i64,
    pub form_id: i64,
    pub form_title: String,
    pub data_json: String,
    pub ip_hash: String,
    pub user_agent: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSnapshotItem {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginSettingField {
    pub key: String,
    pub label: String,
    #[serde(default = "default_field_type")]
    pub field_type: String,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginHooksManifest {
    #[serde(default)]
    pub head_html: String,
    #[serde(default)]
    pub body_top_html: String,
    #[serde(default)]
    pub content_before_html: String,
    #[serde(default)]
    pub content_after_html: String,
    #[serde(default)]
    pub footer_html: String,
    #[serde(default)]
    pub shortcode: String,
    #[serde(default)]
    pub shortcode_html: String,
    #[serde(default)]
    pub on_post_publish_webhook: String,
    #[serde(default)]
    pub on_form_submit_webhook: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub slug: String,
    pub name: String,
    #[serde(default = "default_plugin_version")]
    pub version: String,
    #[serde(default = "default_plugin_author")]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub settings_url: Option<String>,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub settings_schema: Vec<PluginSettingField>,
    #[serde(default)]
    pub hooks: PluginHooksManifest,
}

fn default_plugin_version() -> String {
    "1.0.0".to_string()
}

fn default_plugin_author() -> String {
    "Community Developer".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPluginRecord {
    pub manifest: PluginManifest,
    pub active: bool,
    pub config: std::collections::HashMap<String, String>,
    pub installed_at: String,
}
