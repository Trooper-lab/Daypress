//! RSS 2.0 and Atom 1.0 feed handlers:
//! - `/feed/`, `/feed/rss/`, `/feed/rss2/`, `/feed/atom/`
//! - `/comments/feed/`, `/comments/feed/atom/`
//! - `/category/{slug}/feed/`, `/category/{slug}/feed/atom/`
//! - `/tag/{slug}/feed/`, `/tag/{slug}/feed/atom/`
//! - `/{slug}/feed/`, `/{slug}/feed/atom/` (single post comments feed)

use crate::adapters::database::feeds::FeedTerm;
use crate::db::Database;
use crate::errors::AppError;
use crate::templates::public::rss::{
    render_atom_feed, render_comments_atom_feed, render_comments_rss_feed, render_rss_feed,
    FeedChannel,
};
use axum::{
    extract::{Path as AxPath, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

/// `Content-Type` of every RSS 2.0 feed response.
pub const RSS_CONTENT_TYPE: &str = "application/rss+xml; charset=utf-8";

/// `Content-Type` of every Atom 1.0 feed response.
pub const ATOM_CONTENT_TYPE: &str = "application/atom+xml; charset=utf-8";

pub async fn handle_rss_feed(State(db): State<Database>) -> Result<Response, AppError> {
    feed_response(&db, None, false)
}

pub async fn handle_atom_feed(State(db): State<Database>) -> Result<Response, AppError> {
    feed_response(&db, None, true)
}

pub async fn handle_comments_rss_feed(State(db): State<Database>) -> Result<Response, AppError> {
    comments_feed_response(&db, None, false)
}

pub async fn handle_comments_atom_feed(State(db): State<Database>) -> Result<Response, AppError> {
    comments_feed_response(&db, None, true)
}

pub async fn handle_category_rss_feed(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, AppError> {
    term_feed_response(&db, "category", "category", &slug, false)
}

pub async fn handle_category_atom_feed(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, AppError> {
    term_feed_response(&db, "category", "category", &slug, true)
}

pub async fn handle_tag_rss_feed(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, AppError> {
    term_feed_response(&db, "post_tag", "tag", &slug, false)
}

pub async fn handle_tag_atom_feed(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, AppError> {
    term_feed_response(&db, "post_tag", "tag", &slug, true)
}

pub fn single_post_comments_feed_response(
    db: &Database,
    post_slug: &str,
    is_atom: bool,
) -> Result<Response, AppError> {
    let clean = post_slug.trim_matches('/');
    match db
        .get_post_by_slug(clean)?
        .filter(|p| p.status == "publish")
    {
        Some(post) => comments_feed_response(db, Some(&post), is_atom),
        None => Ok((
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Feed not found",
        )
            .into_response()),
    }
}

fn term_feed_response(
    db: &Database,
    taxonomy: &str,
    route_base: &str,
    slug: &str,
    is_atom: bool,
) -> Result<Response, AppError> {
    match db.find_feed_term(taxonomy, slug.trim_matches('/'))? {
        Some(term) => feed_response(db, Some((&term, route_base)), is_atom),
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

fn feed_response(
    db: &Database,
    term: Option<(&FeedTerm, &str)>,
    is_atom: bool,
) -> Result<Response, AppError> {
    let settings = db.get_settings().unwrap_or_default();
    let posts = db.get_feed_posts(term.map(|(t, _)| t), feed_item_limit(db))?;

    let site_url = settings.url.trim().trim_end_matches('/');
    let site_name = match settings.title.trim() {
        "" => "Daypress",
        name => name,
    };
    let suffix = if is_atom { "feed/atom/" } else { "feed/" };
    let (title, self_path) = match term {
        None => (site_name.to_string(), format!("/{}", suffix)),
        Some((t, base)) => (
            format!("{} - {}", site_name, t.name),
            format!("/{}/{}/{}", base, t.slug, suffix),
        ),
    };
    let self_url = format!("{}{}", site_url, self_path);
    let channel = FeedChannel {
        title: &title,
        site_url,
        self_url: &self_url,
        description: &settings.description,
        language: &settings.language,
    };
    if is_atom {
        let xml = render_atom_feed(&channel, &posts);
        Ok(([(header::CONTENT_TYPE, ATOM_CONTENT_TYPE)], xml).into_response())
    } else {
        let xml = render_rss_feed(&channel, &posts);
        Ok(([(header::CONTENT_TYPE, RSS_CONTENT_TYPE)], xml).into_response())
    }
}

fn comments_feed_response(
    db: &Database,
    post: Option<&crate::models::DaypressPostResponse>,
    is_atom: bool,
) -> Result<Response, AppError> {
    let settings = db.get_settings().unwrap_or_default();
    let comments = db.get_feed_comments(post.map(|p| p.id), feed_item_limit(db))?;

    let site_url = settings.url.trim().trim_end_matches('/');
    let site_name = match settings.title.trim() {
        "" => "Daypress",
        name => name,
    };
    let suffix = if is_atom { "feed/atom/" } else { "feed/" };
    let (title, link_url, self_path) = match post {
        None => (
            format!("Comments for {}", site_name),
            site_url.to_string(),
            format!("/comments/{}", suffix),
        ),
        Some(p) => (
            format!("Comments on: {}", p.title.rendered),
            format!("{}/{}/", site_url, p.slug.trim_matches('/')),
            format!("/{}/{}", p.slug.trim_matches('/'), suffix),
        ),
    };
    let self_url = format!("{}{}", site_url, self_path);
    let channel = FeedChannel {
        title: &title,
        site_url: &link_url,
        self_url: &self_url,
        description: &settings.description,
        language: &settings.language,
    };
    if is_atom {
        let xml = render_comments_atom_feed(&channel, &comments, post.is_some());
        Ok(([(header::CONTENT_TYPE, ATOM_CONTENT_TYPE)], xml).into_response())
    } else {
        let xml = render_comments_rss_feed(&channel, &comments, post.is_some());
        Ok(([(header::CONTENT_TYPE, RSS_CONTENT_TYPE)], xml).into_response())
    }
}
