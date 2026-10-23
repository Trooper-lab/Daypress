//! Category, Tag, Author, and Universal CPT Archive SSR Handlers (`src/handlers/archives_ssr.rs`).

use super::cpt_archive_render::render_cpt_archive;
use super::public_ssr::hydrate_post_terms;
use super::QueryPostParams;
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

pub async fn handle_category_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let (cat_opt, posts) = db.get_posts_by_category_slug(&slug).unwrap_or_default();
    if let Some(cat) = cat_opt {
        let req_ctx = RequestContext::for_archive("category", cat.id, &cat.slug);
        let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Category: {}", cat.name),
                "Category Archive",
                &posts,
                &settings,
                user.as_ref(),
                Some(&partials),
            )),
        )
    } else {
        let req_ctx = RequestContext::for_404(&format!("/category/{}/", slug));
        let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref(), Some(&partials))),
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
        let req_ctx = RequestContext::for_archive("post_tag", tag.id, &tag.slug);
        let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Tag: {}", tag.name),
                "Tag Archive",
                &posts,
                &settings,
                user.as_ref(),
                Some(&partials),
            )),
        )
    } else {
        let req_ctx = RequestContext::for_404(&format!("/tag/{}/", slug));
        let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref(), Some(&partials))),
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
        let req_ctx = RequestContext {
            is_archive: true,
            author_id: Some(author.id),
            ..Default::default()
        };
        let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Author: {}", author.display_name),
                "Author Archive",
                &posts,
                &settings,
                user.as_ref(),
                Some(&partials),
            )),
        )
    } else {
        let req_ctx = RequestContext::for_404(&format!("/author/{}/", slug));
        let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref(), Some(&partials))),
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
    let final_html = crate::plugins::inject_plugin_hooks_into_html(db, settings, None, archive_html);
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
    if let Ok(Some(mut post)) = db.get_post_by_slug(clean_slug) {
        if post.template == "page_templates/leeg.php"
            || post.content.rendered.trim_start().starts_with("<!DOCTYPE html")
        {
            return (StatusCode::OK, Html(post.content.rendered)).into_response();
        }
        let layout_json = db.get_post_layout(post.id).unwrap_or(None);
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
