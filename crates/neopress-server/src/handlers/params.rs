//! Shared HTTP query and form parameter structures (`src/handlers/params.rs`).

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct QueryPostParams {
    pub p: Option<i64>,
    pub page_id: Option<i64>,
    pub s: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProfileQueryParams {
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
pub struct AdminEditQueryParams {
    pub post_type: Option<String>,
    pub id: Option<i64>,
    pub post: Option<i64>,
    pub post_id: Option<i64>,
    pub action: Option<String>,
    pub taxonomy: Option<String>,
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

