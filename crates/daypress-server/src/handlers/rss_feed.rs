//! RSS 2.0 feed handlers: `/feed/`, `/category/{slug}/feed/` and `/tag/{slug}/feed/`.
//!
//! Feeds list published blog posts only (`post_type = 'post'`, `post_status = 'publish'`),
//! newest first, as many as Settings → Reading → "Syndication feeds show the most recent".

use crate::adapters::database::feeds::FeedTerm;
use crate::db::Database;
use crate::errors::AppError;
use crate::templates::public::rss::{render_rss_feed, FeedChannel};
use axum::{
    extract::{Path as AxPath, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

/// `Content-Type` of every feed response.
pub const RSS_CONTENT_TYPE: &str = "application/rss+xml; charset=utf-8";

pub async fn handle_rss_feed(State(db): State<Database>) -> Result<Response, AppError> {
    feed_response(&db, None)
}

pub async fn handle_category_rss_feed(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, AppError> {
    term_feed_response(&db, "category", "category", &slug)
}

pub async fn handle_tag_rss_feed(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, AppError> {
    term_feed_response(&db, "post_tag", "tag", &slug)
}

fn term_feed_response(
    db: &Database,
    taxonomy: &str,
    route_base: &str,
    slug: &str,
) -> Result<Response, AppError> {
    match db.find_feed_term(taxonomy, slug.trim_matches('/'))? {
        Some(term) => feed_response(db, Some((&term, route_base))),
        None => Ok((
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Feed not found",
        )
            .into_response()),
    }
}

/// Settings → Reading → "Syndication feeds show the most recent N items" (default 10).
fn feed_item_limit(db: &Database) -> i64 {
    db.get_option("posts_per_rss")
        .ok()
        .flatten()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .unwrap_or(10)
        .clamp(1, 100)
}

fn feed_response(db: &Database, term: Option<(&FeedTerm, &str)>) -> Result<Response, AppError> {
    let settings = db.get_settings().unwrap_or_default();
    let posts = db.get_feed_posts(term.map(|(t, _)| t), feed_item_limit(db))?;

    let site_url = settings.url.trim().trim_end_matches('/');
    let site_name = match settings.title.trim() {
        "" => "Daypress",
        name => name,
    };
    let (title, self_path) = match term {
        None => (site_name.to_string(), "/feed/".to_string()),
        Some((t, base)) => (
            format!("{} - {}", site_name, t.name),
            format!("/{}/{}/feed/", base, t.slug),
        ),
    };
    let self_url = format!("{}{}", site_url, self_path);
    let xml = render_rss_feed(
        &FeedChannel {
            title: &title,
            site_url,
            self_url: &self_url,
            description: &settings.description,
            language: &settings.language,
        },
        &posts,
    );
    Ok(([(header::CONTENT_TYPE, RSS_CONTENT_TYPE)], xml).into_response())
}
