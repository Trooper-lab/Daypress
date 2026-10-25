use serde::{Deserialize, Serialize};

use super::common::WpRenderedText;

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
    /// WordPress user ID of the comment author (WP REST `author`). Only honoured for
    /// authenticated requests; anonymous submissions are always stored with user_id 0.
    #[serde(default)]
    pub author: Option<i64>,
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

/// One row of the admin Comments moderation table (`/n-admin/edit-comments`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCommentRow {
    pub item: WpCommentAdminItem,
    /// `comment_parent` (0 when top-level).
    pub parent_id: i64,
    /// Author name of the parent comment (empty when top-level / parent missing).
    pub parent_author: String,
    /// `_wp_trash_meta_status` – the status the comment had before being trashed/spammed.
    pub prev_status: String,
    pub user_id: i64,
}
