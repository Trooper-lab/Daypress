//! Category, Tag, Author, and Universal CPT Archive SSR Handlers (`src/handlers/archives_ssr.rs`).

use super::cpt_archive_render::render_cpt_archive;
use super::public_ssr::hydrate_post_terms;
use super::{PagedQuery, QueryPostParams};
use crate::auth::OptionalAuthUser;
use crate::db::Database;
use crate::models;
use crate::templates;
use crate::theme_builder::models::RequestContext;
use crate::theme_builder::ssr_injector::SsrInjector;
use axum::{
    extract::{Path as AxPath, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};

/// Which term archive a request is for.
#[derive(Clone, Copy)]
enum TermArchive {
    Category,
    Tag,
    Author,
}

/// Renders one page of a category, tag, or author archive (`/category/x/page/2/` or
/// `?paged=2`). Out-of-range pages and unknown terms are 404s.
fn term_archive_response(
    db: &Database,
    kind: TermArchive,
    slug: &str,
    page: usize,
    user: Option<&crate::auth::AuthUser>,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    let (base, found) = match kind {
        TermArchive::Category => {
            let (cat, posts) = db.get_posts_by_category_slug(slug).unwrap_or_default();
            (
                "category",
                cat.map(|c| {
                    let ctx = RequestContext::for_archive("category", c.id, &c.slug);
                    (
                        format!("Category: {}", c.name),
                        "Category Archive",
                        ctx,
                        posts,
                    )
                }),
            )
        }
        TermArchive::Tag => {
            let (tag, posts) = db.get_posts_by_tag_slug(slug).unwrap_or_default();
            (
                "tag",
                tag.map(|t| {
                    let ctx = RequestContext::for_archive("post_tag", t.id, &t.slug);
                    (format!("Tag: {}", t.name), "Tag Archive", ctx, posts)
                }),
            )
        }
        TermArchive::Author => {
            let (author, posts) = db.get_posts_by_author_slug(slug).unwrap_or_default();
            (
                "author",
                author.map(|a| {
                    let ctx = RequestContext {
                        is_archive: true,
                        author_id: Some(a.id),
                        ..Default::default()
                    };
                    (
                        format!("Author: {}", a.display_name),
                        "Author Archive",
                        ctx,
                        posts,
                    )
                }),
            )
        }
    };
    let base_path = format!("/{}/{}/", base, slug.trim_matches('/'));

    let per_page = settings.posts_per_page.clamp(1, 100) as usize;
    if let Some((title, subtitle, req_ctx, posts)) = found {
        let (page_posts, total_pages) = templates::pagination::paginate(&posts, page, per_page);
        if (1..=total_pages).contains(&page) {
            let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, &settings);
            let nav = templates::pagination::render_pagination(&base_path, page, total_pages);
            return (
                StatusCode::OK,
                Html(templates::render_archive_page_paginated(
                    &title,
                    subtitle,
                    page_posts,
                    &settings,
                    user,
                    Some(&partials),
                    &nav,
                )),
            )
                .into_response();
        }
    }
    let req_ctx = RequestContext::for_404(&base_path);
    let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, &settings);
    (
        StatusCode::NOT_FOUND,
        Html(templates::render_not_found_page(
            &settings,
            user,
            Some(&partials),
        )),
    )
        .into_response()
}

pub async fn handle_category_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    Query(q): Query<PagedQuery>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    term_archive_response(&db, TermArchive::Category, &slug, q.page(), user.as_ref())
}

pub async fn handle_category_archive_paged(
    State(db): State<Database>,
    AxPath((slug, raw_page)): AxPath<(String, String)>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    // Non-numeric pages fall through to page 0, which renders the 404.
    let page = super::parse_page_segment(&raw_page).unwrap_or(0);
    term_archive_response(&db, TermArchive::Category, &slug, page, user.as_ref())
}

pub async fn handle_tag_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    Query(q): Query<PagedQuery>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    term_archive_response(&db, TermArchive::Tag, &slug, q.page(), user.as_ref())
}

pub async fn handle_tag_archive_paged(
    State(db): State<Database>,
    AxPath((slug, raw_page)): AxPath<(String, String)>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    // Non-numeric pages fall through to page 0, which renders the 404.
    let page = super::parse_page_segment(&raw_page).unwrap_or(0);
    term_archive_response(&db, TermArchive::Tag, &slug, page, user.as_ref())
}

pub async fn handle_author_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    Query(q): Query<PagedQuery>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    term_archive_response(&db, TermArchive::Author, &slug, q.page(), user.as_ref())
}

pub async fn handle_author_archive_paged(
    State(db): State<Database>,
    AxPath((slug, raw_page)): AxPath<(String, String)>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    // Non-numeric pages fall through to page 0, which renders the 404.
    let page = super::parse_page_segment(&raw_page).unwrap_or(0);
    term_archive_response(&db, TermArchive::Author, &slug, page, user.as_ref())
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
    let req_ctx = RequestContext {
        is_search: true,
        ..Default::default()
    };
    let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
    Html(templates::render_search_page(
        q,
        &posts,
        &settings,
        user.as_ref(),
        Some(&partials),
    ))
}

pub fn try_handle_cpt_archive(
    db: &Database,
    cpt_slug: &str,
    settings: &models::WpSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
) -> Option<Response> {
    let archive_html = render_cpt_archive(db, cpt_slug, settings, user)?;
    let final_html =
        crate::plugins::inject_plugin_hooks_into_html(db, settings, None, archive_html);
    Some((StatusCode::OK, Html(final_html)).into_response())
}

pub async fn handle_cpt_single(
    State(db): State<Database>,
    AxPath((post_type, slug)): AxPath<(String, String)>,
    headers: HeaderMap,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let pt = post_type.trim_matches('/');
    let clean_slug = slug.trim_matches('/');
    if matches!(
        pt,
        "wp-content"
            | "wp-includes"
            | "wp-json"
            | "n-admin"
            | "static"
            | "uploads"
            | "category"
            | "tag"
            | "author"
    ) {
        let settings = db.get_settings().unwrap_or_default();
        return (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(
                &settings,
                user.as_ref(),
                None,
            )),
        )
            .into_response();
    }

    let req_path = format!("/{}/{}/", pt, clean_slug);
    if let Some((target_url, status_code)) = db.match_redirect_and_increment(&req_path) {
        let mut resp_headers = HeaderMap::new();
        if let Ok(loc) = target_url.parse() {
            resp_headers.insert(header::LOCATION, loc);
        }
        let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::MOVED_PERMANENTLY);
        return (status, resp_headers, Html(String::new())).into_response();
    }

    let settings = db.get_settings().unwrap_or_default();
    if let Some(mut post) = db
        .get_post_by_slug(clean_slug)
        .ok()
        .flatten()
        .filter(|p| crate::auth::can_view_status(&p.status, user.as_ref()))
    {
        if post.template == "page_templates/leeg.php"
            || post
                .content
                .rendered
                .trim_start()
                .starts_with("<!DOCTYPE html")
        {
            return (StatusCode::OK, Html(post.content.rendered)).into_response();
        }
        let layout_json = db.get_post_layout(post.id).unwrap_or(None);
        let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
        let expanded =
            crate::plugins::expand_shortcodes_in_content(&post.content.rendered, &db, &settings);
        post.content.rendered = format!(
            "{}{}{}",
            hooks.content_before_html, expanded, hooks.content_after_html
        );
        let terms = hydrate_post_terms(&db, &post);
        let req_ctx = RequestContext::for_post(post.id, &post.post_type, false, terms);
        let partials = SsrInjector::resolve_and_hydrate_partials_with_post(
            &db,
            &req_ctx,
            &settings,
            Some(&post),
        );
        let html = templates::render_single_post(
            &post,
            &settings,
            user.as_ref(),
            layout_json.as_deref(),
            Some(&partials),
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

    if let Some(resp) = super::taxonomy_archive_render::try_handle_taxonomy_archive(
        &db,
        pt,
        clean_slug,
        &settings,
        user.as_ref(),
    ) {
        return resp;
    }

    let referrer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    db.record_404_hit(&req_path, referrer, ua);

    let req_ctx = RequestContext::for_404(&req_path);
    let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);

    (
        StatusCode::NOT_FOUND,
        Html(crate::plugins::inject_plugin_hooks_into_html(
            &db,
            &settings,
            None,
            templates::render_not_found_page(&settings, user.as_ref(), Some(&partials)),
        )),
    )
        .into_response()
}
