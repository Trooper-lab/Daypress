//! Domain models and parsed representations for WordPress WXR imports.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ParsedWxrItem {
    pub id: Option<i64>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: String,
    pub post_date: String,
    pub post_status: String,
    pub post_type: String,
    pub post_mime_type: String,
    pub parent: i64,
    pub menu_order: i64,
    pub guid: String,
    pub attachment_url: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub custom_terms: Vec<(String, String, String)>,
    pub nav_menu_slug: Option<String>,
    pub elementor_library_type: Option<String>,
    pub postmeta: HashMap<String, String>,
    pub comments: Vec<ParsedWxrComment>,
    pub is_sticky: bool,
}

#[derive(Debug, Default)]
pub struct ParsedWxrComment {
    pub id: Option<i64>,
    pub author: String,
    pub email: String,
    pub url: String,
    pub date: String,
    pub content: String,
    pub approved: String,
    pub comment_type: String,
    pub parent: i64,
}

#[derive(Debug, Default)]
pub struct ParsedWxrTerm {
    pub taxonomy: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    #[allow(dead_code)]
    pub parent: i64,
}

/// Infers MIME type from a filename or URL extension when `post_mime_type` is empty in WXR.
pub fn infer_mime_type_from_filename(path_or_url: &str) -> &'static str {
    let clean = path_or_url
        .split('?')
        .next()
        .unwrap_or(path_or_url)
        .trim()
        .to_lowercase();
    if clean.ends_with(".jpg") || clean.ends_with(".jpeg") {
        "image/jpeg"
    } else if clean.ends_with(".png") {
        "image/png"
    } else if clean.ends_with(".webp") {
        "image/webp"
    } else if clean.ends_with(".svg") {
        "image/svg+xml"
    } else if clean.ends_with(".gif") {
        "image/gif"
    } else if clean.ends_with(".pdf") {
        "application/pdf"
    } else {
        ""
    }
}

pub fn normalize_menu_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if let Some(idx) = trimmed
            .find("://")
            .and_then(|i| trimmed[i + 3..].find('/').map(|j| i + 3 + j))
        {
            let path = &trimmed[idx..];
            if path.is_empty() {
                "/".to_string()
            } else {
                path.to_string()
            }
        } else {
            "/".to_string()
        }
    } else {
        trimmed.to_string()
    }
}

/// Strongly typed item for Pass 2 AST transpilation and template ingestion.
#[derive(Debug, Clone)]
pub struct PendingAstItem {
    pub post_id: i64,
    pub old_id: Option<i64>,
    pub title: String,
    pub post_type: String,
    pub post_status: String,
    pub raw_content: String,
    pub elementor_library_type: Option<String>,
    pub postmeta: HashMap<String, String>,
}
