use serde::{Deserialize, Serialize};

use super::comments::DaypressCommentResponse;
use super::common::{DaypressGuid, DaypressRenderedText};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaypressPostResponse {
    pub id: i64,
    pub date: String,
    pub date_gmt: String,
    pub guid: DaypressGuid,
    pub modified: String,
    pub modified_gmt: String,
    pub slug: String,
    pub status: String,
    #[serde(rename = "type")]
    pub post_type: String,
    pub link: String,
    pub title: DaypressRenderedText,
    pub content: DaypressRenderedText,
    pub excerpt: DaypressRenderedText,
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
    pub comments: Vec<DaypressCommentResponse>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatePostRequest {
    // Both optional, as in the WordPress REST API (e.g. DayStudio's quick "New Page").
    #[serde(default)]
    pub title: String,
    #[serde(default)]
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
    #[serde(default)]
    pub taxonomies: Option<std::collections::HashMap<String, Vec<i64>>>,
    /// Publish date (ISO 8601, site time = UTC). A `publish` post dated ahead is stored as
    /// `future` (scheduled), as in WordPress.
    #[serde(default)]
    pub date: Option<String>,
    /// Publish date in UTC; wins over `date` when both are given.
    #[serde(default)]
    pub date_gmt: Option<String>,
}

pub fn default_post_status() -> String {
    "publish".to_string()
}

pub fn default_post_type() -> String {
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
    #[serde(default)]
    pub taxonomies: Option<std::collections::HashMap<String, Vec<i64>>>,
    /// New publish date (ISO 8601, site time = UTC); see [`CreatePostRequest::date`].
    #[serde(default)]
    pub date: Option<String>,
    /// New publish date in UTC; wins over `date` when both are given.
    #[serde(default)]
    pub date_gmt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaypressLayoutPayload {
    pub layout: serde_json::Value,
    #[serde(default)]
    pub rendered_html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaypressRevisionResponse {
    pub id: i64,
    pub author: i64,
    #[serde(default)]
    pub author_name: String,
    pub date: String,
    pub date_gmt: String,
    pub modified: String,
    pub modified_gmt: String,
    pub parent: i64,
    pub slug: String,
    pub guid: DaypressGuid,
    pub title: DaypressRenderedText,
    pub content: DaypressRenderedText,
    pub excerpt: DaypressRenderedText,
    #[serde(default)]
    pub has_layout: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanRevisionsRequest {
    /// Keep the N most recent revisions per post (0 deletes all revisions).
    #[serde(default)]
    pub keep_per_post: Option<usize>,
    /// Optional age filter: only delete revisions older than N days.
    #[serde(default)]
    pub older_than_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanRevisionsResponse {
    pub deleted_count: usize,
    pub remaining_count: i64,
}
