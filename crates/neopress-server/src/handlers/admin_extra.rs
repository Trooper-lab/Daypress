//! Extra admin page handlers (`handlers/admin_extra.rs`).

use super::params::CommentQueryParams;
use crate::Database;
use crate::templates;
use axum::extract::{Query, State};
use axum::response::Html;

/// 1. Dashboard Updates
pub async fn handle_admin_updates(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_wp_updates_view(&settings, &stats))
}

/// 2. Media Add New
pub async fn handle_admin_media_new(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_media_new_view(&settings))
}

/// 3. Comments Moderation Queue
pub async fn handle_admin_comments(
    State(db): State<Database>,
    Query(params): Query<CommentQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let comments = db
        .get_comments(params.comment_status.as_deref(), 100)
        .unwrap_or_default();
    let counts = db.get_comment_counts().unwrap_or_default();
    Html(templates::render_wp_comments_view(
        &settings,
        &comments,
        &counts,
        params.comment_status.as_deref(),
    ))
}
