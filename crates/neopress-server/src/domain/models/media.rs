use serde::{Deserialize, Serialize};

use super::common::WpRenderedText;

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
}
