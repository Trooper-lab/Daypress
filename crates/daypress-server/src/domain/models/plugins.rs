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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldConditionRule {
    #[serde(default)]
    pub field: String,
    #[serde(default = "default_cond_operator")]
    pub operator: String,
    #[serde(default)]
    pub value: String,
}

pub fn default_cond_operator() -> String {
    "equals".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionalEmailRoute {
    #[serde(default)]
    pub field: String,
    #[serde(default = "default_cond_operator")]
    pub operator: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub recipient_email: String,
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
    #[serde(default = "default_field_width")]
    pub width: String,
    #[serde(default = "default_field_step")]
    pub step: u8,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub help_text: String,
    #[serde(default)]
    pub condition: Option<FieldConditionRule>,
    #[serde(default)]
    pub calculation_formula: Option<String>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step_increment: Option<f64>,
    #[serde(default, alias = "allowed_extensions")]
    pub accept_extensions: String,
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u32,
}

pub fn default_max_file_size_mb() -> u32 {
    10
}

pub fn default_field_type() -> String {
    "text".to_string()
}

pub fn default_field_width() -> String {
    "full".to_string()
}

pub fn default_field_step() -> u8 {
    1
}

impl Default for FormFieldSchema {
    fn default() -> Self {
        Self {
            name: String::new(),
            label: String::new(),
            field_type: default_field_type(),
            required: false,
            placeholder: String::new(),
            options: Vec::new(),
            width: default_field_width(),
            step: default_field_step(),
            default_value: String::new(),
            help_text: String::new(),
            condition: None,
            calculation_formula: None,
            min: None,
            max: None,
            step_increment: None,
            accept_extensions: String::new(),
            max_file_size_mb: default_max_file_size_mb(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSettings {
    #[serde(default = "default_submit_label")]
    pub submit_label: String,
    #[serde(default = "default_success_msg")]
    pub success_message: String,
    #[serde(default)]
    pub redirect_url: String,
    #[serde(default)]
    pub recipient_email: String,
    #[serde(default)]
    pub email_subject_template: String,
    #[serde(default)]
    pub conditional_routes: Vec<ConditionalEmailRoute>,
    #[serde(default, alias = "enable_autoresponder")]
    pub auto_responder_enabled: bool,
    #[serde(
        default = "default_autoresponder_field",
        alias = "autoresponder_email_field"
    )]
    pub auto_responder_email_field: String,
    #[serde(default, alias = "autoresponder_subject")]
    pub auto_responder_subject: String,
    #[serde(default, alias = "autoresponder_body")]
    pub auto_responder_body: String,
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    pub step_titles: Vec<String>,
    #[serde(default = "default_true_bool")]
    pub show_progress_bar: bool,
    #[serde(default = "default_true_bool")]
    pub enable_honeypot: bool,
    #[serde(default)]
    pub min_submit_seconds: u32,
    #[serde(default = "default_rate_limit_per_min")]
    pub rate_limit_per_minute: u32,
    #[serde(default = "default_captcha_provider")]
    pub captcha_provider: String,
    #[serde(default)]
    pub captcha_site_key: String,
    #[serde(default)]
    pub captcha_secret_key: String,
}

pub fn default_submit_label() -> String {
    "Send Message".to_string()
}

pub fn default_success_msg() -> String {
    "Thank you! Your submission has been received.".to_string()
}

pub fn default_autoresponder_field() -> String {
    "email".to_string()
}

pub fn default_rate_limit_per_min() -> u32 {
    15
}

pub fn default_captcha_provider() -> String {
    "none".to_string()
}

impl Default for FormSettings {
    fn default() -> Self {
        Self {
            submit_label: default_submit_label(),
            success_message: default_success_msg(),
            redirect_url: String::new(),
            recipient_email: String::new(),
            email_subject_template: String::new(),
            conditional_routes: Vec::new(),
            auto_responder_enabled: false,
            auto_responder_email_field: default_autoresponder_field(),
            auto_responder_subject: String::new(),
            auto_responder_body: String::new(),
            webhook_url: String::new(),
            step_titles: Vec::new(),
            show_progress_bar: true,
            enable_honeypot: true,
            min_submit_seconds: 0,
            rate_limit_per_minute: default_rate_limit_per_min(),
            captcha_provider: default_captcha_provider(),
            captcha_site_key: String::new(),
            captcha_secret_key: String::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormAttachmentMeta {
    #[serde(default)]
    pub field_name: String,
    #[serde(default, alias = "stored_name")]
    pub filename: String,
    #[serde(default)]
    pub original_name: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub stored_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormEntryItem {
    pub id: i64,
    pub form_id: i64,
    pub form_title: String,
    pub data_json: String,
    #[serde(default)]
    pub attachments: Vec<FormAttachmentMeta>,
    pub ip_hash: String,
    pub user_agent: String,
    #[serde(default = "default_entry_status")]
    pub status: String,
    #[serde(default)]
    pub is_starred: bool,
    #[serde(default = "default_email_status")]
    pub email_status: String,
    #[serde(default)]
    pub email_error: Option<String>,
    #[serde(default)]
    pub webhook_status: Option<String>,
    #[serde(default)]
    pub source_url: String,
    pub created_at: String,
}

pub fn default_entry_status() -> String {
    "unread".to_string()
}

pub fn default_email_status() -> String {
    "disabled".to_string()
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
    #[serde(default = "default_analytics_event_type")]
    pub event_type: String,
    #[serde(default)]
    pub goal_name: Option<String>,
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

pub fn default_analytics_event_type() -> String {
    "pageview".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsSummary {
    pub total_visitors: i64,
    pub total_pageviews: i64,
    pub top_pages: Vec<TopPageMetric>,
    #[serde(default)]
    pub total_conversions: i64,
    #[serde(default)]
    pub bounce_rate_pct: f64,
    #[serde(default)]
    pub range: String,
    #[serde(default, alias = "timeseries")]
    pub buckets: Vec<AnalyticsBucketPoint>,
    #[serde(default)]
    pub top_referrers: Vec<BreakdownMetric>,
    #[serde(default)]
    pub utm_sources: Vec<BreakdownMetric>,
    #[serde(default)]
    pub utm_mediums: Vec<BreakdownMetric>,
    #[serde(default)]
    pub utm_campaigns: Vec<BreakdownMetric>,
    #[serde(default)]
    pub countries: Vec<BreakdownMetric>,
    #[serde(default)]
    pub devices: Vec<BreakdownMetric>,
    #[serde(default)]
    pub conversions: Vec<ConversionMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopPageMetric {
    pub page_url: String,
    pub views: i64,
    #[serde(default)]
    pub visitors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsBucketPoint {
    #[serde(default, alias = "label")]
    pub bucket: String,
    pub visitors: i64,
    pub pageviews: i64,
    pub conversions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreakdownMetric {
    pub label: String,
    pub visitors: i64,
    #[serde(default, alias = "pageviews")]
    pub views: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversionMetric {
    pub goal_name: String,
    #[serde(default, alias = "conversions")]
    pub count: i64,
    pub unique_visitors: i64,
    pub conversion_rate_pct: f64,
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

// --- v0.9 Pillar 1: Built-in Newsletters, Member-Gated Posts & Automated Digests ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscriberRecord {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub status: String, // "pending" | "subscribed" | "unsubscribed"
    pub lists: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsubscribe_token: Option<String>,
    pub source_url: String,
    pub confirmed_at: Option<String>,
    pub unsubscribed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewsletterSubscribeRequest {
    pub email: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_newsletter_lists")]
    pub lists: Vec<String>,
    #[serde(default)]
    pub source_url: String,
    /// When an administrator imports or adds a subscriber directly, skip double opt-in.
    #[serde(default)]
    pub skip_double_opt_in: bool,
}

pub fn default_newsletter_lists() -> Vec<String> {
    vec!["general".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewsletterUnlockRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewsletterCampaignRecord {
    pub id: i64,
    pub subject: String,
    pub body_html: String,
    pub target_list: String,
    pub campaign_type: String, // "broadcast" | "digest"
    pub post_id: Option<i64>,
    pub status: String, // "draft" | "sending" | "sent" | "failed"
    pub recipient_count: i64,
    pub sent_count: i64,
    pub failed_count: i64,
    pub sent_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateNewsletterCampaignRequest {
    pub subject: String,
    pub body_html: String,
    #[serde(default = "default_newsletter_list_name")]
    pub target_list: String,
    #[serde(default = "default_true_bool")]
    pub send_now: bool,
}

pub fn default_newsletter_list_name() -> String {
    "general".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewsletterStatsSummary {
    pub total_subscribers: i64,
    pub confirmed_subscribers: i64,
    pub pending_subscribers: i64,
    pub unsubscribed_count: i64,
    pub total_campaigns_sent: i64,
    pub total_emails_delivered: i64,
}

// --- v0.9 Pillar 2: Branded Shortlinks (`/go/*`) & HMAC-Signed Outbound Webhooks ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShortlinkRecord {
    pub id: i64,
    pub slug: String,
    pub short_url: String,
    pub target_url: String,
    pub title: String,
    pub status_code: i64,
    pub utm_source: String,
    pub utm_medium: String,
    pub utm_campaign: String,
    pub clicks: i64,
    pub last_clicked_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateShortlinkRequest {
    pub slug: String,
    pub target_url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_shortlink_status_code")]
    pub status_code: i64,
    #[serde(default)]
    pub utm_source: String,
    #[serde(default)]
    pub utm_medium: String,
    #[serde(default)]
    pub utm_campaign: String,
    #[serde(default = "default_true_bool")]
    pub is_active: bool,
}

pub fn default_shortlink_status_code() -> i64 {
    302
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookEndpointRecord {
    pub id: i64,
    pub name: String,
    pub target_url: String,
    pub preset: String, // "generic" | "slack" | "discord" | "hubspot" | "zapier"
    pub events: Vec<String>,
    pub secret: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateWebhookEndpointRequest {
    pub id: Option<i64>,
    pub name: String,
    pub target_url: String,
    #[serde(default = "default_webhook_preset")]
    pub preset: String,
    #[serde(default = "default_webhook_events")]
    pub events: Vec<String>,
    #[serde(default)]
    pub secret: String,
    #[serde(default = "default_true_bool")]
    pub is_active: bool,
}

pub fn default_webhook_preset() -> String {
    "generic".to_string()
}

pub fn default_webhook_events() -> Vec<String> {
    vec![
        "post.published".to_string(),
        "form.submitted".to_string(),
        "subscriber.confirmed".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookDeliveryRecord {
    pub id: i64,
    pub webhook_id: i64,
    pub webhook_name: String,
    pub event_name: String,
    pub target_url: String,
    pub request_headers_json: String,
    pub payload_json: String,
    pub signature_header: String,
    pub response_status: i64,
    pub response_body: String,
    pub latency_ms: i64,
    pub attempt_count: i64,
    pub status: String,
    pub created_at: String,
}

// =========================================================================
// v0.9 Pillar 3: Server-Side A/B Split Testing & Real-User Core Web Vitals
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbTestRecord {
    pub id: i64,
    pub name: String,
    pub post_id: i64,
    pub post_title: String,
    pub target_selector: String,
    pub control_text: String,
    pub variant_text: String,
    pub goal_event: String,
    pub traffic_split: i64,
    pub status: String,
    pub control_views: i64,
    pub control_conversions: i64,
    pub variant_views: i64,
    pub variant_conversions: i64,
    pub control_cr_pct: f64,
    pub variant_cr_pct: f64,
    pub uplift_pct: f64,
    pub z_score: f64,
    pub confidence_pct: f64,
    pub recommended_winner: Option<String>,
    pub winner: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateAbTestRequest {
    pub id: Option<i64>,
    pub name: String,
    pub post_id: i64,
    #[serde(default)]
    pub target_selector: String,
    pub control_text: String,
    pub variant_text: String,
    #[serde(default = "default_ab_goal_event")]
    pub goal_event: String,
    #[serde(default = "default_ab_traffic_split")]
    pub traffic_split: i64,
}

pub fn default_ab_goal_event() -> String {
    "conversion".to_string()
}

pub fn default_ab_traffic_split() -> i64 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebVitalsBeaconRequest {
    #[serde(default = "default_vitals_page_url")]
    pub page_url: String,
    #[serde(default = "default_vitals_device")]
    pub device_type: String,
    pub lcp_ms: Option<f64>,
    pub inp_ms: Option<f64>,
    pub cls_score: Option<f64>,
    pub ttfb_ms: Option<f64>,
}

pub fn default_vitals_page_url() -> String {
    "/".to_string()
}

pub fn default_vitals_device() -> String {
    "desktop".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreWebVitalsSummary {
    pub sample_count: i64,
    pub lcp_p75_ms: f64,
    pub inp_p75_ms: f64,
    pub cls_p75: f64,
    pub ttfb_p75_ms: f64,
    pub lcp_status: String,
    pub inp_status: String,
    pub cls_status: String,
    pub overall_status: String,
}

// =========================================================================
// v0.9 Pillar 4: Sandboxed WebAssembly (`.wasm`) Extension Crates
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmExtensionRecord {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub shortcode: String,
    pub capabilities: daypress_wasm::WasmCapabilities,
    pub fuel_limit: u64,
    pub max_memory_pages: u32,
    pub wasm_sha256: String,
    pub wasm_size_bytes: usize,
    pub exports: Vec<String>,
    pub is_active: bool,
    pub executions: i64,
    pub last_error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallWasmExtensionRequest {
    pub slug: String,
    pub name: String,
    #[serde(default = "default_wasm_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub shortcode: String,
    #[serde(default)]
    pub capabilities: daypress_wasm::WasmCapabilities,
    pub fuel_limit: Option<u64>,
    pub max_memory_pages: Option<u32>,
    /// Either provide raw base64-encoded `.wasm` binary bytes (`wasm_base64`) OR a declarative `template_source`
    /// that Daypress compiles directly into a standards-compliant `.wasm` binary!
    pub wasm_base64: Option<String>,
    pub template_source: Option<String>,
    #[serde(default = "default_true_bool")]
    pub is_active: bool,
}

pub fn default_wasm_version() -> String {
    "1.0.0".to_string()
}
