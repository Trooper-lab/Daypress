//! Public SSR & SEO Sitemap/Robots Handlers.

#![allow(unused_imports)]
use super::*;
use crate::auth::OptionalAuthUser;
use crate::db::Database;
use crate::models;
use crate::templates;
use crate::theme_builder::models::{RequestContext, TaxonomyTermRef};
use crate::theme_builder::ssr_injector::SsrInjector;
use axum::{
    extract::{Form, Path as AxPath, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

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
        let req_ctx = RequestContext {
            is_search: true,
            ..Default::default()
        };
        let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
        let html =
            templates::render_search_page(q, &results, &settings, user.as_ref(), Some(&partials));
        return Html(crate::plugins::inject_plugin_hooks_into_html(
            &db, &settings, None, html,
        ))
        .into_response();
    }

    if let Some(id) = params.p {
        if let Some(mut post) = db
            .get_post_by_id(id)
            .ok()
            .flatten()
            .filter(|p| crate::auth::can_view_status(&p.status, user.as_ref()))
        {
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
            let terms = hydrate_post_terms(&db, &post);
            let req_ctx = RequestContext::for_post(
                post.id,
                &post.post_type,
                settings.page_on_front > 0 && post.id == settings.page_on_front,
                terms,
            );
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
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&post),
                html,
            ))
            .into_response();
        } else {
            let req_ctx = RequestContext::for_404(&format!("/?p={}", id));
            let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
            return (
                StatusCode::NOT_FOUND,
                Html(templates::render_not_found_page(
                    &settings,
                    user.as_ref(),
                    Some(&partials),
                )),
            )
                .into_response();
        }
    }

    if let Some(id) = params.page_id {
        if let Some(mut page) = db
            .get_post_by_id(id)
            .ok()
            .flatten()
            .filter(|p| crate::auth::can_view_status(&p.status, user.as_ref()))
        {
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
            let req_ctx = RequestContext::for_post(
                page.id,
                &page.post_type,
                settings.page_on_front > 0 && page.id == settings.page_on_front,
                vec![],
            );
            let partials = SsrInjector::resolve_and_hydrate_partials_with_post(
                &db,
                &req_ctx,
                &settings,
                Some(&page),
            );
            let html = templates::render_single_post(
                &page,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
                Some(&partials),
            );
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&page),
                html,
            ))
            .into_response();
        } else {
            let req_ctx = RequestContext::for_404(&format!("/?page_id={}", id));
            let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
            return (
                StatusCode::NOT_FOUND,
                Html(templates::render_not_found_page(
                    &settings,
                    user.as_ref(),
                    Some(&partials),
                )),
            )
                .into_response();
        }
    }

    if settings.show_on_front == "page" && settings.page_on_front > 0 {
        if let Some(mut front_page) = db
            .get_post_by_id(settings.page_on_front)
            .ok()
            .flatten()
            .filter(|p| crate::auth::can_view_status(&p.status, user.as_ref()))
        {
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
            let req_ctx = RequestContext::for_front_page_post(front_page.id, &front_page.post_type);
            let partials = SsrInjector::resolve_and_hydrate_partials_with_post(
                &db,
                &req_ctx,
                &settings,
                Some(&front_page),
            );
            let html = templates::render_single_post(
                &front_page,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
                Some(&partials),
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

    blog_index_response(
        &db,
        &settings,
        user.as_ref(),
        params.paged.unwrap_or(1).max(1),
    )
}

/// `/page/N/`: further pages of the blog index when the front page shows latest posts.
pub async fn handle_home_paged(
    State(db): State<Database>,
    AxPath(raw_page): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    match parse_page_segment(&raw_page) {
        Some(page) if !(settings.show_on_front == "page" && settings.page_on_front > 0) => {
            blog_index_response(&db, &settings, user.as_ref(), page)
        }
        _ => not_found_response(
            &db,
            &settings,
            user.as_ref(),
            &format!("/page/{}/", raw_page),
        ),
    }
}

/// `/{posts-page}/page/N/`: further pages of the page chosen under Settings → Reading → Posts page.
pub async fn handle_single_paged(
    State(db): State<Database>,
    AxPath((slug, raw_page)): AxPath<(String, String)>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    let page = parse_page_segment(&raw_page).unwrap_or(0);
    let posts_page = db
        .get_post_by_slug(slug.trim_matches('/'))
        .ok()
        .flatten()
        .filter(|p| crate::auth::can_view_status(&p.status, user.as_ref()))
        .filter(|p| settings.show_on_front == "page" && p.id == settings.page_for_posts)
        .filter(|_| page >= 1);
    match posts_page {
        Some(post) => posts_page_response(&db, &settings, user.as_ref(), &post, page),
        None => not_found_response(
            &db,
            &settings,
            user.as_ref(),
            &format!("/{}/page/{}/", slug.trim_matches('/'), raw_page),
        ),
    }
}

/// One page of the blog index (`posts_per_page` posts), with older/newer links.
fn blog_index_response(
    db: &Database,
    settings: &models::WpSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
    page: usize,
) -> Response {
    let per_page = settings.posts_per_page.clamp(1, 100) as usize;
    let posts = db
        .get_posts("post", Some("publish"), None, 10_000)
        .unwrap_or_default();
    let (page_posts, total_pages) = templates::pagination::paginate(&posts, page, per_page);
    if page > total_pages {
        return not_found_response(db, settings, user, &format!("/page/{}/", page));
    }
    let nav = templates::pagination::render_pagination("/", page, total_pages);
    let req_ctx = RequestContext::for_front_page();
    let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, settings);
    let html =
        templates::render_home_page_paginated(page_posts, settings, user, Some(&partials), &nav);
    Html(crate::plugins::inject_plugin_hooks_into_html(
        db, settings, None, html,
    ))
    .into_response()
}

/// One page of the posts page archive, with older/newer links under `/{slug}/page/N/`.
fn posts_page_response(
    db: &Database,
    settings: &models::WpSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
    post: &models::WpPostResponse,
    page: usize,
) -> Response {
    let per_page = settings.posts_per_page.clamp(1, 100) as usize;
    let posts = db
        .get_posts("post", Some("publish"), None, 10_000)
        .unwrap_or_default();
    let base_path = format!("/{}/", post.slug.trim_matches('/'));
    let (page_posts, total_pages) = templates::pagination::paginate(&posts, page, per_page);
    if page > total_pages {
        return not_found_response(db, settings, user, &format!("{}page/{}/", base_path, page));
    }
    let nav = templates::pagination::render_pagination(&base_path, page, total_pages);
    let req_ctx = RequestContext {
        is_archive: true,
        ..Default::default()
    };
    let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, settings);
    let html = templates::render_archive_page_paginated(
        &post.title.rendered,
        "Blog Archive",
        page_posts,
        settings,
        user,
        Some(&partials),
        &nav,
    );
    (
        StatusCode::OK,
        Html(crate::plugins::inject_plugin_hooks_into_html(
            db,
            settings,
            Some(post),
            html,
        )),
    )
        .into_response()
}

fn not_found_response(
    db: &Database,
    settings: &models::WpSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
    path: &str,
) -> Response {
    let req_ctx = RequestContext::for_404(path);
    let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, settings);
    (
        StatusCode::NOT_FOUND,
        Html(templates::render_not_found_page(
            settings,
            user,
            Some(&partials),
        )),
    )
        .into_response()
}

pub async fn handle_single(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    Query(paged): Query<PagedQuery>,
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

    let clean_slug = slug.trim_matches('/');

    if let Some(mut post) = db
        .get_post_by_slug(&slug)
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
        if settings.show_on_front == "page"
            && settings.page_for_posts > 0
            && post.id == settings.page_for_posts
            && layout_json.is_none()
            && post.content.rendered.trim().is_empty()
        {
            return posts_page_response(&db, &settings, user.as_ref(), &post, paged.page());
        }

        let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
        let expanded =
            crate::plugins::expand_shortcodes_in_content(&post.content.rendered, &db, &settings);
        post.content.rendered = format!(
            "{}{}{}",
            hooks.content_before_html, expanded, hooks.content_after_html
        );

        let terms = hydrate_post_terms(&db, &post);
        let req_ctx = RequestContext::for_post(
            post.id,
            &post.post_type,
            settings.page_on_front > 0 && post.id == settings.page_on_front,
            terms,
        );
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

    if let Some(archive_resp) = crate::handlers::archives_ssr::try_handle_cpt_archive(
        &db,
        clean_slug,
        &settings,
        user.as_ref(),
    ) {
        return archive_resp;
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

pub(crate) fn hydrate_post_terms(
    db: &Database,
    post: &models::WpPostResponse,
) -> Vec<TaxonomyTermRef> {
    let mut terms = Vec::new();
    if let Ok(conn) = db.conn.lock() {
        if let Ok(mut stmt) = conn.prepare_cached(
            "SELECT tt.taxonomy, t.term_id, t.slug FROM wp_term_relationships r \
             JOIN wp_term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id \
             JOIN wp_terms t ON tt.term_id = t.term_id \
             WHERE r.object_id = ?1",
        ) {
            if let Ok(rows) = stmt.query_map(rusqlite::params![post.id], |r| {
                let tax: String = r.get(0)?;
                let tid: i64 = r.get(1)?;
                let slug: String = r.get(2)?;
                Ok((tax, tid, slug))
            }) {
                for row in rows.flatten() {
                    let tax_clean = if row.0 == "post_tag" {
                        "tag".to_string()
                    } else {
                        row.0.clone()
                    };
                    terms.push(TaxonomyTermRef {
                        taxonomy: tax_clean,
                        term_id: row.1,
                        slug: row.2.clone(),
                    });
                    if row.0 == "post_tag" {
                        terms.push(TaxonomyTermRef {
                            taxonomy: "post_tag".to_string(),
                            term_id: row.1,
                            slug: row.2,
                        });
                    }
                }
            }
        }
    }
    if terms.is_empty() {
        for &cid in &post.categories {
            terms.push(TaxonomyTermRef {
                taxonomy: "category".to_string(),
                term_id: cid,
                slug: String::new(),
            });
        }
        for &tid in &post.tags {
            terms.push(TaxonomyTermRef {
                taxonomy: "tag".to_string(),
                term_id: tid,
                slug: String::new(),
            });
        }
    }
    terms
}
