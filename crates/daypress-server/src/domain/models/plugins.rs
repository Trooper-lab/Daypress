use serde::{Deserialize, Serialize};

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
    #[serde(default = "super::import::default_redirect_code")]
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

pub fn default_seo_separator() -> String {
    "–".to_string()
}

pub fn default_true_bool() -> bool {
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

pub fn default_field_type() -> String {
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

pub fn default_submit_label() -> String {
    "Send Message".to_string()
}

pub fn default_success_msg() -> String {
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
pub struct DaypressForm {
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

pub fn default_plugin_version() -> String {
    "1.0.0".to_string()
}

pub fn default_plugin_author() -> String {
    "Community Developer".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPluginRecord {
    pub manifest: PluginManifest,
    pub active: bool,
    pub config: std::collections::HashMap<String, String>,
    pub installed_at: String,
}
