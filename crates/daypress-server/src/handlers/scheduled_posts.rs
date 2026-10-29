//! Publishes scheduled (`future`) posts whose date has passed before a request is handled.
//!
//! The check is one indexed `UPDATE` (see `adapters::database::scheduling`) that changes
//! nothing when no post is due, so running it per request keeps every read path (pages,
//! archives, search, feeds, sitemap, REST) consistent without a background task. Static
//! asset requests skip it.

use crate::db::Database;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

/// Path prefixes that only serve files and never read posts.
const STATIC_PREFIXES: [&str; 4] = ["/static/", "/uploads/", "/wp-content/", "/favicon"];

/// Whether a request path can read posts (and so needs due posts published first).
pub fn path_reads_posts(path: &str) -> bool {
    !STATIC_PREFIXES.iter().any(|p| path.starts_with(p))
}

/// Middleware: promote due scheduled posts, then run the request.
pub async fn publish_due_posts_layer(
    State(db): State<Database>,
    req: Request,
    next: Next,
) -> Response {
    if path_reads_posts(req.uri().path()) {
        if let Err(e) = db.publish_due_scheduled_posts() {
            tracing::warn!("could not publish due scheduled posts: {e}");
        }
    }
    next.run(req).await
}
