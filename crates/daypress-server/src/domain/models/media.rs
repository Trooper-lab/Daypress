use serde::{Deserialize, Serialize};

use super::common::DaypressRenderedText;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaSizeDetail {
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub source_url: String,
}

pub fn default_focal_coord() -> f32 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MediaFolderDto {
    pub id: i64,
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub parent_id: i64,
    #[serde(default)]
    pub count: i64,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDetails {
    pub width: u32,
    pub height: u32,
    pub file: String,
    #[serde(default)]
    pub filesize: u64,
    #[serde(default)]
    pub sizes: std::collections::HashMap<String, MediaSizeDetail>,
    #[serde(default = "default_focal_coord")]
    pub focal_x: f32,
    #[serde(default = "default_focal_coord")]
    pub focal_y: f32,
}

impl Default for MediaDetails {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            file: String::new(),
            filesize: 0,
            sizes: std::collections::HashMap::new(),
            focal_x: 0.5,
            focal_y: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaypressMediaResponse {
    pub id: i64,
    pub date: String,
    pub slug: String,
    pub title: DaypressRenderedText,
    pub source_url: String,
    pub mime_type: String,
    pub file_size: u64,
    #[serde(default = "default_media_type")]
    pub media_type: String,
    #[serde(default)]
    pub alt_text: String,
    #[serde(default)]
    pub media_details: MediaDetails,
    /// Attachment caption (`post_excerpt`), WP REST `caption`.
    #[serde(default = "empty_rendered_text")]
    pub caption: DaypressRenderedText,
    /// Attachment description (`post_content`), WP REST `description`.
    #[serde(default = "empty_rendered_text")]
    pub description: DaypressRenderedText,
    /// Parent post ID (`post_parent`), WP REST `post` (null when unattached).
    #[serde(default)]
    pub post: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_focal_coord")]
    pub focal_x: f32,
    #[serde(default = "default_focal_coord")]
    pub focal_y: f32,
}

pub fn empty_rendered_text() -> DaypressRenderedText {
    DaypressRenderedText {
        rendered: String::new(),
        protected: false,
    }
}

pub fn default_media_type() -> String {
    "image".to_string()
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
    #[serde(default)]
    pub folder_id: Option<i64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub focal_x: Option<f32>,
    #[serde(default)]
    pub focal_y: Option<f32>,
}
