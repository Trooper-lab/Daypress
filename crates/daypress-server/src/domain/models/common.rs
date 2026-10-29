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
            db_path: "data/daypress.sqlite".to_string(),
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
