//! Shared HTTP query and form parameter structures (`src/handlers/params.rs`).

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct QueryPostParams {
    pub p: Option<i64>,
    pub page_id: Option<i64>,
    pub s: Option<String>,
    pub q: Option<String>,
    /// WordPress `?paged=N` listing page (same as `/page/N/`).
    pub paged: Option<usize>,
    #[serde(alias = "tax", alias = "taxonomy")]
    pub dp_tax: Option<String>,
    #[serde(alias = "term")]
    pub dp_term: Option<String>,
    #[serde(alias = "sort")]
    pub dp_sort: Option<String>,
    pub dp_q: Option<String>,
    pub dp_page: Option<usize>,
    pub preview_token: Option<String>,
    pub subscriber_token: Option<String>,
    pub lang: Option<String>,
}

/// `?paged=N` and Collection Grid filter parameters on archive and single listings.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PagedQuery {
    pub paged: Option<usize>,
    #[serde(alias = "tax", alias = "taxonomy")]
    pub dp_tax: Option<String>,
    #[serde(alias = "term")]
    pub dp_term: Option<String>,
    #[serde(alias = "sort")]
    pub dp_sort: Option<String>,
    #[serde(alias = "q")]
    pub dp_q: Option<String>,
    pub dp_page: Option<usize>,
    pub preview_token: Option<String>,
    pub subscriber_token: Option<String>,
    pub lang: Option<String>,
}

/// Parses the `N` of a `/page/N/` URL; anything but a positive number is `None` (a 404).
pub fn parse_page_segment(raw: &str) -> Option<usize> {
    raw.trim_matches('/')
        .parse::<usize>()
        .ok()
        .filter(|n| *n >= 1)
}

impl PagedQuery {
    /// The requested 1-based page (defaults to 1).
    pub fn page(&self) -> usize {
        self.paged.unwrap_or(1).max(1)
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ProfileQueryParams {
    #[serde(alias = "id")]
    pub user_id: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginQueryParams {
    pub action: Option<String>,
    pub loggedout: Option<String>,
    pub redirect_to: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginForm {
    pub log: Option<String>,
    pub pwd: Option<String>,
    pub redirect_to: Option<String>,
    pub rememberme: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LoginTwoFactorForm {
    #[serde(alias = "code", alias = "totp_code")]
    pub authcode: Option<String>,
    pub redirect_to: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AdminEditQueryParams {
    pub post_type: Option<String>,
    pub id: Option<i64>,
    pub post: Option<i64>,
    pub post_id: Option<i64>,
    pub template_id: Option<i64>,
    pub action: Option<String>,
    pub taxonomy: Option<String>,
    pub lang: Option<String>,
    pub trgrp: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CommentQueryParams {
    pub comment_status: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateCommentStatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ExportPersonalDataQuery {
    pub user: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ErasePersonalDataPayload {
    pub identifier: String,
}

/// Reduces a post type or taxonomy query value to a WordPress `sanitize_key` slug
/// (lowercase ASCII letters, digits, `-`, `_`), so it is safe to echo into admin HTML.
pub fn sanitize_key(raw: &str) -> String {
    raw.trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .map(|c| c.to_ascii_lowercase())
        .take(32)
        .collect()
}
