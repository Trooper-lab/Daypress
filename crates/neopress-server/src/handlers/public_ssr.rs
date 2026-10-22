//! Public SSR & SEO Sitemap/Robots Handlers.

#![allow(unused_imports)]
use crate::auth::OptionalAuthUser;
use crate::db::Database;
use crate::models;
use crate::templates;
use axum::{
    extract::{Form, Path as AxPath, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use super::*;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct LoginQueryParams {
    pub action: Option<String>,
    pub loggedout: Option<String>,
    pub redirect_to: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub log: Option<String>,
    pub pwd: Option<String>,
    pub redirect_to: Option<String>,
    pub rememberme: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminEditQueryParams {
    pub post_type: Option<String>,
    pub id: Option<i64>,
    pub post: Option<i64>,
    pub action: Option<String>,
    pub taxonomy: Option<String>,
}

pub async fn handle_home_or_query(
    State(db): State<Database>,
    Query(params): Query<QueryPostParams>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();

    if let Some(q) = params
        .s
        .as_deref()
        .or(params.q.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let results = db
            .get_posts("any", Some("publish"), Some(q), 50)
            .unwrap_or_default();
        let html = templates::render_search_page(q, &results, &settings, user.as_ref());
        return Html(crate::plugins::inject_plugin_hooks_into_html(
            &db, &settings, None, html,
        ))
        .into_response();
    }

    if let Some(id) = params.p {
        if let Ok(Some(mut post)) = db.get_post_by_id(id) {
            let layout_json = db.get_post_layout(id).unwrap_or(None);
            let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
            let expanded = crate::plugins::expand_shortcodes_in_content(
                &post.content.rendered,
                &db,
                &settings,
            );
            post.content.rendered = format!(
                "{}{}{}",
                hooks.content_before_html, expanded, hooks.content_after_html
            );
            let html = templates::render_single_post(
                &post,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
            );
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&post),
                html,
            ))
            .into_response();
        } else {
            return (
                StatusCode::NOT_FOUND,
                Html(templates::render_not_found_page(&settings, user.as_ref())),
            )
                .into_response();
        }
    }

    if let Some(id) = params.page_id {
        if let Ok(Some(mut page)) = db.get_post_by_id(id) {
            let layout_json = db.get_post_layout(id).unwrap_or(None);
            let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&page));
            let expanded = crate::plugins::expand_shortcodes_in_content(
                &page.content.rendered,
                &db,
                &settings,
            );
            page.content.rendered = format!(
                "{}{}{}",
                hooks.content_before_html, expanded, hooks.content_after_html
            );
            let html = templates::render_single_post(
                &page,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
            );
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&page),
                html,
            ))
            .into_response();
        } else {
            return (
                StatusCode::NOT_FOUND,
                Html(templates::render_not_found_page(&settings, user.as_ref())),
            )
                .into_response();
        }
    }

    if settings.show_on_front == "page" && settings.page_on_front > 0 {
        if let Ok(Some(mut front_page)) = db.get_post_by_id(settings.page_on_front) {
            let layout_json = db.get_post_layout(front_page.id).unwrap_or(None);
            let hooks =
                crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&front_page));
            let expanded = crate::plugins::expand_shortcodes_in_content(
                &front_page.content.rendered,
                &db,
                &settings,
            );
            front_page.content.rendered = format!(
                "{}{}{}",
                hooks.content_before_html, expanded, hooks.content_after_html
            );
            let html = templates::render_single_post(
                &front_page,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
            );
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&front_page),
                html,
            ))
            .into_response();
        }
    }

    let limit = settings.posts_per_page.clamp(1, 100);
    let posts = db
        .get_posts("post", Some("publish"), None, limit)
        .unwrap_or_default();
    let html = templates::render_home_page(&posts, &settings, user.as_ref());
    Html(crate::plugins::inject_plugin_hooks_into_html(
        &db, &settings, None, html,
    ))
    .into_response()
}

pub async fn handle_single(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    headers: HeaderMap,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    // 1. Check 301/302 Redirect Manager first
    let req_path = format!("/{}/", slug.trim_matches('/'));
    if let Some((target_url, status_code)) = db.match_redirect_and_increment(&req_path) {
        let mut resp_headers = HeaderMap::new();
        if let Ok(loc) = target_url.parse() {
            resp_headers.insert(header::LOCATION, loc);
        }
        let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::MOVED_PERMANENTLY);
        return (status, resp_headers, Html(String::new())).into_response();
    }

    let settings = db.get_settings().unwrap_or_default();

    if let Ok(Some(mut post)) = db.get_post_by_slug(&slug) {
        let layout_json = db.get_post_layout(post.id).unwrap_or(None);
        if settings.show_on_front == "page"
            && settings.page_for_posts > 0
            && post.id == settings.page_for_posts
            && layout_json.is_none()
            && post.content.rendered.trim().is_empty()
        {
            let posts = db
                .get_posts(
                    "post",
                    Some("publish"),
                    None,
                    settings.posts_per_page.clamp(1, 100),
                )
                .unwrap_or_default();
            let html = templates::render_archive_page(
                &post.title.rendered,
                "Blog Archive",
                &posts,
                &settings,
                user.as_ref(),
            );
            return (
                StatusCode::OK,
                Html(crate::plugins::inject_plugin_hooks_into_html(
                    &db,
                    &settings,
                    Some(&post),
                    html,
                )),
            )
                .into_response();
        }

        let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
        let expanded = crate::plugins::expand_shortcodes_in_content(
            &post.content.rendered,
            &db,
            &settings,
        );
        post.content.rendered = format!(
            "{}{}{}",
            hooks.content_before_html, expanded, hooks.content_after_html
        );

        let html = templates::render_single_post(
            &post,
            &settings,
            user.as_ref(),
            layout_json.as_deref(),
        );
        return (
            StatusCode::OK,
            Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&post),
                html,
            )),
        )
            .into_response();
    }

    // Record 404 hit in 404 Monitor
    let referrer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    db.record_404_hit(&req_path, referrer, ua);

    (
        StatusCode::NOT_FOUND,
        Html(crate::plugins::inject_plugin_hooks_into_html(
            &db,
            &settings,
            None,
            templates::render_not_found_page(&settings, user.as_ref()),
        )),
    )
        .into_response()
}

pub async fn handle_category_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let (cat_opt, posts) = db.get_posts_by_category_slug(&slug).unwrap_or_default();
    if let Some(cat) = cat_opt {
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Category: {}", cat.name),
                "Category Archive",
                &posts,
                &settings,
                user.as_ref(),
            )),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref())),
        )
    }
}

pub async fn handle_tag_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let (tag_opt, posts) = db.get_posts_by_tag_slug(&slug).unwrap_or_default();
    if let Some(tag) = tag_opt {
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Tag: {}", tag.name),
                "Tag Archive",
                &posts,
                &settings,
                user.as_ref(),
            )),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref())),
        )
    }
}

pub async fn handle_author_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let (author_opt, posts) = db.get_posts_by_author_slug(&slug).unwrap_or_default();
    if let Some(author) = author_opt {
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Author: {}", author.display_name),
                "Author Archive",
                &posts,
                &settings,
                user.as_ref(),
            )),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref())),
        )
    }
}

pub async fn handle_search_page(
    State(db): State<Database>,
    Query(params): Query<QueryPostParams>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let q = params
        .s
        .as_deref()
        .or(params.q.as_deref())
        .unwrap_or("")
        .trim();
    let posts = if q.is_empty() {
        Vec::new()
    } else {
        db.get_posts("any", Some("publish"), Some(q), 50)
            .unwrap_or_default()
    };
    Html(templates::render_search_page(
        q,
        &posts,
        &settings,
        user.as_ref(),
    ))
}

// =========================================================================
// NATIVE SEO & STREAMING SITEMAP / ROBOTS HANDLERS
// =========================================================================

pub async fn handle_sitemap_xml(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let base_url = settings.url.trim_end_matches('/');
    let posts = db
        .get_posts("any", Some("publish"), None, 50000)
        .unwrap_or_default();

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Home Page
    xml.push_str(&format!(
        "  <url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>\n",
        base_url
    ));

    for post in posts {
        if post.seo_noindex {
            continue;
        }
        if post.post_type == "post" || post.post_type == "page" {
            let lastmod = if !post.modified.is_empty() {
                &post.modified
            } else {
                &post.date
            };
            let priority = if post.post_type == "page" {
                "0.8"
            } else {
                "0.6"
            };
            xml.push_str(&format!(
                "  <url><loc>{}/{}/</loc><lastmod>{}</lastmod><priority>{}</priority></url>\n",
                base_url, post.slug, lastmod, priority
            ));
        }
    }

    xml.push_str("</urlset>");

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
}

pub async fn handle_robots_txt(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let seo_global = db.get_global_seo_settings().unwrap_or_default();
    let base_url = settings.url.trim_end_matches('/');
    let robots = if !seo_global.custom_robots_txt.trim().is_empty() {
        seo_global.custom_robots_txt
    } else {
        format!(
            "User-agent: *\nAllow: /\nDisallow: /n-admin/\nDisallow: /wp-json/\n\nSitemap: {}/sitemap.xml\n",
            base_url
        )
    };
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots,
    )
}
