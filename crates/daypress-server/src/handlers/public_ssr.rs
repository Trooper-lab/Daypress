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

pub fn render_secret_preview_post_response(
    db: &Database,
    expected_post_id: Option<i64>,
    token: &str,
    user: Option<&crate::auth::AuthUser>,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    let Some((verified_post_id, _expires_at)) = db.verify_preview_token(token, expected_post_id)
    else {
        let req_ctx = RequestContext::for_404("/preview");
        let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, &settings);
        return (
            StatusCode::FORBIDDEN,
            [(
                header::HeaderName::from_static("x-robots-tag"),
                "noindex, nofollow",
            )],
            Html(templates::render_not_found_page(
                &settings,
                user,
                Some(&partials),
            )),
        )
            .into_response();
    };

    let Some(mut post) = db.get_post_by_id(verified_post_id).ok().flatten() else {
        let req_ctx = RequestContext::for_404("/preview");
        let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, &settings);
        return (
            StatusCode::NOT_FOUND,
            [(
                header::HeaderName::from_static("x-robots-tag"),
                "noindex, nofollow",
            )],
            Html(templates::render_not_found_page(
                &settings,
                user,
                Some(&partials),
            )),
        )
            .into_response();
    };

    let raw_layout = db.get_post_layout(post.id).unwrap_or(None);
    let hooks = crate::plugins::render_active_plugin_hooks(db, &settings, Some(&post));
    let expanded = crate::plugins::expand_shortcodes_in_content_for_post(
        &post.content.rendered,
        db,
        &settings,
        Some(&post),
    );
    post.content.rendered = format!(
        "{}{}{}",
        hooks.content_before_html, expanded, hooks.content_after_html
    );
    let terms = hydrate_post_terms(db, &post);
    let req_ctx = RequestContext::for_post(post.id, &post.post_type, false, terms);
    let layout_json = SsrInjector::hydrate_post_layout_json(
        db,
        &req_ctx,
        &settings,
        Some(&post),
        raw_layout.as_deref(),
    )
    .or(raw_layout);
    let partials =
        SsrInjector::resolve_and_hydrate_partials_with_post(db, &req_ctx, &settings, Some(&post));
    let html = templates::render_single_post(
        &post,
        &settings,
        user,
        layout_json.as_deref(),
        Some(&partials),
    );
    let mut final_html =
        crate::plugins::inject_plugin_hooks_into_html(db, &settings, Some(&post), html);

    let banner = format!(
        r#"<div class="dp-secret-preview-banner" style="background:#16171a;color:#fbf8f2;border-bottom:2px solid #a8401b;padding:10px 20px;font-family:var(--font-mono,monospace);font-size:12px;display:flex;justify-content:space-between;align-items:center;z-index:99999;position:relative;"><span><strong>Daypress Secret Draft Preview</strong> &middot; Shared via secret preview link (Status: {})</span><span style="opacity:0.75;">noindex, nofollow</span></div>"#,
        crate::templates::utils::escape_html(&post.status)
    );
    if let Some(body_idx) = final_html.find("<body") {
        if let Some(close_idx) = final_html[body_idx..].find('>') {
            let insert_pos = body_idx + close_idx + 1;
            final_html.insert_str(insert_pos, &format!("\n{}", banner));
        }
    }

    (
        StatusCode::OK,
        [
            (
                header::HeaderName::from_static("x-robots-tag"),
                "noindex, nofollow",
            ),
            (header::CACHE_CONTROL, "no-store, private"),
        ],
        Html(final_html),
    )
        .into_response()
}

pub async fn handle_secret_preview(
    State(db): State<Database>,
    AxPath(token): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    render_secret_preview_post_response(&db, None, token.trim_matches('/'), user.as_ref())
}

pub async fn handle_home_or_query(
    State(db): State<Database>,
    Query(params): Query<QueryPostParams>,
    headers: HeaderMap,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    if let Some(token) = params
        .preview_token
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return render_secret_preview_post_response(
            &db,
            params.p.or(params.page_id),
            token,
            user.as_ref(),
        );
    }
    if let Some(q) = params
        .s
        .as_deref()
        .or(params.q.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let settings = db.get_settings().unwrap_or_default();
        let mut results = Vec::new();
        if let Ok(fts) = db.search_fts(q, None, 50) {
            for hit in fts.results {
                if let Ok(Some(post)) = db.get_post_by_id(hit.id) {
                    results.push(post);
                }
            }
        }
        if results.is_empty() {
            results = db
                .get_posts("any", Some("publish"), Some(q), 50)
                .unwrap_or_default();
        }
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

    let page_num = params.paged.unwrap_or(1).max(1);
    let cache_key = if let Some(id) = params.p {
        format!("/?p={}", id)
    } else if let Some(id) = params.page_id {
        format!("/?page_id={}", id)
    } else if page_num <= 1 {
        "/".to_string()
    } else {
        format!("/?paged={}", page_num)
    };

    let has_dp_filter = params.dp_tax.is_some()
        || params.dp_term.is_some()
        || params.dp_sort.is_some()
        || params.dp_q.is_some()
        || params.dp_page.is_some()
        || params.subscriber_token.is_some();

    let has_sub_cookie =
        extract_subscriber_token(&headers, params.subscriber_token.as_deref()).is_some();
    if user.is_none() && !has_dp_filter && !has_sub_cookie {
        if let Some(hit) = db.hot_cache.try_serve(&cache_key, &headers) {
            return hit;
        }
    }

    let settings = db.get_settings().unwrap_or_default();

    if let Some(id) = params.p {
        if let Some(mut post) = db
            .get_post_by_id(id)
            .ok()
            .flatten()
            .filter(|p| crate::auth::can_view_post(p, user.as_ref()))
        {
            let (is_restricted, is_locked) =
                apply_content_access_guards(&db, &mut post, user.as_ref(), &headers);
            if !is_locked
                && (post.template == "page_templates/leeg.php"
                    || post
                        .content
                        .rendered
                        .trim_start()
                        .starts_with("<!DOCTYPE html"))
            {
                return db.hot_cache.store_and_serve(
                    &cache_key,
                    &headers,
                    user.is_some() || has_dp_filter || is_restricted,
                    post.content.rendered,
                );
            }
            let raw_layout = db.get_post_layout(id).unwrap_or(None);
            let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
            let expanded = crate::plugins::expand_shortcodes_in_content_for_post(
                &post.content.rendered,
                &db,
                &settings,
                Some(&post),
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
            let layout_json = if is_locked {
                None
            } else {
                SsrInjector::hydrate_post_layout_json_with_query(
                    &db,
                    &req_ctx,
                    &settings,
                    Some(&post),
                    raw_layout.as_deref(),
                    params.dp_tax.clone(),
                    params.dp_term.clone(),
                    params.dp_sort.clone(),
                    params.dp_q.clone(),
                    params.dp_page,
                )
                .or(raw_layout)
            };
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
            let final_html =
                crate::plugins::inject_plugin_hooks_into_html(&db, &settings, Some(&post), html);
            let visitor_hash = db.compute_daily_visitor_hash(&headers, None);
            let (final_html, ab_assignments) =
                db.apply_ab_experiments_for_post(post.id, &visitor_hash, &final_html);
            return db.hot_cache.store_and_serve(
                &cache_key,
                &headers,
                user.is_some() || has_dp_filter || is_restricted || !ab_assignments.is_empty(),
                final_html,
            );
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
            .filter(|p| crate::auth::can_view_post(p, user.as_ref()))
        {
            let (is_restricted, is_locked) =
                apply_content_access_guards(&db, &mut page, user.as_ref(), &headers);
            if !is_locked
                && (page.template == "page_templates/leeg.php"
                    || page
                        .content
                        .rendered
                        .trim_start()
                        .starts_with("<!DOCTYPE html"))
            {
                return db.hot_cache.store_and_serve(
                    &cache_key,
                    &headers,
                    user.is_some() || has_dp_filter || is_restricted,
                    page.content.rendered,
                );
            }
            let raw_layout = db.get_post_layout(id).unwrap_or(None);
            let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&page));
            let expanded = if raw_layout.is_none() || is_locked {
                crate::plugins::expand_shortcodes_in_content(&page.content.rendered, &db, &settings)
            } else {
                page.content.rendered.clone()
            };
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
            let layout_json = if is_locked {
                None
            } else {
                SsrInjector::hydrate_post_layout_json_with_query(
                    &db,
                    &req_ctx,
                    &settings,
                    Some(&page),
                    raw_layout.as_deref(),
                    params.dp_tax.clone(),
                    params.dp_term.clone(),
                    params.dp_sort.clone(),
                    params.dp_q.clone(),
                    params.dp_page,
                )
                .or(raw_layout)
            };
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
            let final_html =
                crate::plugins::inject_plugin_hooks_into_html(&db, &settings, Some(&page), html);
            let (final_html, ab_assignments) = if settings
                .active_plugins
                .iter()
                .any(|p| p == "daypress-ab-testing")
            {
                let visitor_hash = db.compute_daily_visitor_hash(&headers, None);
                db.apply_ab_experiments_for_post(page.id, &visitor_hash, &final_html)
            } else {
                (final_html, Vec::new())
            };
            return db.hot_cache.store_and_serve(
                &cache_key,
                &headers,
                user.is_some() || has_dp_filter || is_restricted || !ab_assignments.is_empty(),
                final_html,
            );
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
            .filter(|p| crate::auth::can_view_post(p, user.as_ref()))
        {
            let (is_restricted, is_locked) =
                apply_content_access_guards(&db, &mut front_page, user.as_ref(), &headers);
            let raw_layout = db.get_post_layout(front_page.id).unwrap_or(None);
            let hooks =
                crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&front_page));
            let expanded = if raw_layout.is_none() || is_locked {
                crate::plugins::expand_shortcodes_in_content(
                    &front_page.content.rendered,
                    &db,
                    &settings,
                )
            } else {
                front_page.content.rendered.clone()
            };
            front_page.content.rendered = format!(
                "{}{}{}",
                hooks.content_before_html, expanded, hooks.content_after_html
            );
            let req_ctx = RequestContext::for_front_page_post(front_page.id, &front_page.post_type);
            let layout_json = if is_locked {
                None
            } else {
                SsrInjector::hydrate_post_layout_json_with_query(
                    &db,
                    &req_ctx,
                    &settings,
                    Some(&front_page),
                    raw_layout.as_deref(),
                    params.dp_tax.clone(),
                    params.dp_term.clone(),
                    params.dp_sort.clone(),
                    params.dp_q.clone(),
                    params.dp_page,
                )
                .or(raw_layout)
            };
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
            let final_html = crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&front_page),
                html,
            );
            let (final_html, ab_assignments) = if settings
                .active_plugins
                .iter()
                .any(|p| p == "daypress-ab-testing")
            {
                let visitor_hash = db.compute_daily_visitor_hash(&headers, None);
                db.apply_ab_experiments_for_post(front_page.id, &visitor_hash, &final_html)
            } else {
                (final_html, Vec::new())
            };
            return db.hot_cache.store_and_serve(
                &cache_key,
                &headers,
                user.is_some() || has_dp_filter || is_restricted || !ab_assignments.is_empty(),
                final_html,
            );
        }
    }

    blog_index_response(
        &db,
        &settings,
        user.as_ref(),
        page_num,
        &cache_key,
        &headers,
    )
}

/// `/page/N/`: further pages of the blog index when the front page shows latest posts.
pub async fn handle_home_paged(
    State(db): State<Database>,
    AxPath(raw_page): AxPath<String>,
    headers: HeaderMap,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    let cache_key = format!("/page/{}/", raw_page.trim_matches('/'));
    if user.is_none() {
        if let Some(hit) = db.hot_cache.try_serve(&cache_key, &headers) {
            return hit;
        }
    }
    let settings = db.get_settings().unwrap_or_default();
    match parse_page_segment(&raw_page) {
        Some(page) if !(settings.show_on_front == "page" && settings.page_on_front > 0) => {
            blog_index_response(&db, &settings, user.as_ref(), page, &cache_key, &headers)
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
    headers: HeaderMap,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Response {
    let cache_key = format!(
        "/{}/page/{}/",
        slug.trim_matches('/'),
        raw_page.trim_matches('/')
    );
    if user.is_none() {
        if let Some(hit) = db.hot_cache.try_serve(&cache_key, &headers) {
            return hit;
        }
    }
    let settings = db.get_settings().unwrap_or_default();
    let page = parse_page_segment(&raw_page).unwrap_or(0);
    let clean_slug = slug.trim_matches('/');
    if page >= 1 && db.is_registered_language_code(clean_slug) {
        let per_page = settings.posts_per_page.clamp(1, 100) as usize;
        let all_posts = db
            .get_posts("post", Some("publish"), None, 10_000)
            .unwrap_or_default();
        let lang_lower = clean_slug.to_ascii_lowercase();
        let filtered: Vec<_> = all_posts
            .into_iter()
            .filter(|p| p.lang_code.eq_ignore_ascii_case(&lang_lower))
            .collect();
        let (page_posts, total_pages) = templates::pagination::paginate(&filtered, page, per_page);
        if page <= total_pages {
            let base_path = format!("/{}/", clean_slug);
            let nav = templates::pagination::render_pagination(&base_path, page, total_pages);
            let req_ctx = RequestContext::for_front_page();
            let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
            let html = templates::render_home_page_paginated(
                page_posts,
                &settings,
                user.as_ref(),
                Some(&partials),
                &nav,
            );
            let final_html =
                crate::plugins::inject_plugin_hooks_into_html(&db, &settings, None, html);
            return db
                .hot_cache
                .store_and_serve(&cache_key, &headers, user.is_some(), final_html);
        }
    }
    let posts_page = db
        .get_post_by_slug(clean_slug)
        .ok()
        .flatten()
        .filter(|p| crate::auth::can_view_post(p, user.as_ref()))
        .filter(|p| settings.show_on_front == "page" && p.id == settings.page_for_posts)
        .filter(|_| page >= 1);
    match posts_page {
        Some(post) => posts_page_response(
            &db,
            &settings,
            user.as_ref(),
            &post,
            page,
            &cache_key,
            &headers,
        ),
        None => {
            if page >= 1 {
                let pq = PagedQuery {
                    paged: Some(page),
                    dp_page: Some(page),
                    ..Default::default()
                };
                if let Some(archive_resp) =
                    crate::handlers::archives_ssr::try_handle_cpt_archive_with_query(
                        &db,
                        slug.trim_matches('/'),
                        &settings,
                        user.as_ref(),
                        &pq,
                    )
                {
                    return archive_resp;
                }
            }
            not_found_response(
                &db,
                &settings,
                user.as_ref(),
                &format!("/{}/page/{}/", slug.trim_matches('/'), raw_page),
            )
        }
    }
}

/// One page of the blog index (`posts_per_page` posts), with older/newer links.
fn blog_index_response(
    db: &Database,
    settings: &models::DaypressSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
    page: usize,
    cache_key: &str,
    headers: &HeaderMap,
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
    let final_html = crate::plugins::inject_plugin_hooks_into_html(db, settings, None, html);
    db.hot_cache
        .store_and_serve(cache_key, headers, user.is_some(), final_html)
}

/// One page of the posts page archive, with older/newer links under `/{slug}/page/N/`.
fn posts_page_response(
    db: &Database,
    settings: &models::DaypressSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
    post: &models::DaypressPostResponse,
    page: usize,
    cache_key: &str,
    headers: &HeaderMap,
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
    let final_html = crate::plugins::inject_plugin_hooks_into_html(db, settings, Some(post), html);
    db.hot_cache
        .store_and_serve(cache_key, headers, user.is_some(), final_html)
}

fn not_found_response(
    db: &Database,
    settings: &models::DaypressSettingsResponse,
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
) -> Response {
    let clean_slug = slug.trim_matches('/');

    if let Some(token) = paged
        .preview_token
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let post_id = db.get_post_by_slug(clean_slug).ok().flatten().map(|p| p.id);
        return render_secret_preview_post_response(&db, post_id, token, user.as_ref());
    }

    // 1. Check 301/302 Redirect Manager first
    let req_path = format!("/{}/", clean_slug);
    if let Some((target_url, status_code)) = db.match_redirect_and_increment(&req_path) {
        let mut resp_headers = HeaderMap::new();
        if let Ok(loc) = target_url.parse() {
            resp_headers.insert(header::LOCATION, loc);
        }
        let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::MOVED_PERMANENTLY);
        return (status, resp_headers, Html(String::new())).into_response();
    }

    let page_num = paged.page();
    let has_dp_filter = paged.dp_tax.is_some()
        || paged.dp_term.is_some()
        || paged.dp_sort.is_some()
        || paged.dp_q.is_some()
        || paged.dp_page.is_some()
        || paged.subscriber_token.is_some();
    let cache_key = if page_num <= 1 {
        req_path.clone()
    } else {
        format!("{}?paged={}", req_path, page_num)
    };
    let has_sub_cookie =
        extract_subscriber_token(&headers, paged.subscriber_token.as_deref()).is_some();
    if user.is_none() && !has_dp_filter && !has_sub_cookie {
        if let Some(hit) = db.hot_cache.try_serve(&cache_key, &headers) {
            return hit;
        }
    }

    let settings = db.get_settings().unwrap_or_default();

    // 1b. Multilingual Locale Prefix Front Page (e.g., `/de/` or `/nl/`)
    if db.is_registered_language_code(clean_slug) {
        if let Ok(Some(mut loc_front)) = db.get_localized_front_page(clean_slug) {
            if crate::auth::can_view_post(&loc_front, user.as_ref()) {
                let (is_restricted, is_locked) =
                    apply_content_access_guards(&db, &mut loc_front, user.as_ref(), &headers);
                let raw_layout = db.get_post_layout(loc_front.id).unwrap_or(None);
                let hooks =
                    crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&loc_front));
                let expanded = if raw_layout.is_none() || is_locked {
                    crate::plugins::expand_shortcodes_in_content_for_post(
                        &loc_front.content.rendered,
                        &db,
                        &settings,
                        Some(&loc_front),
                    )
                } else {
                    loc_front.content.rendered.clone()
                };
                loc_front.content.rendered = format!(
                    "{}{}{}",
                    hooks.content_before_html, expanded, hooks.content_after_html
                );
                let req_ctx =
                    RequestContext::for_front_page_post(loc_front.id, &loc_front.post_type);
                let layout_json = if is_locked {
                    None
                } else {
                    SsrInjector::hydrate_post_layout_json_with_query(
                        &db,
                        &req_ctx,
                        &settings,
                        Some(&loc_front),
                        raw_layout.as_deref(),
                        paged.dp_tax.clone(),
                        paged.dp_term.clone(),
                        paged.dp_sort.clone(),
                        paged.dp_q.clone(),
                        paged.dp_page,
                    )
                    .or(raw_layout)
                };
                let partials = SsrInjector::resolve_and_hydrate_partials_with_post(
                    &db,
                    &req_ctx,
                    &settings,
                    Some(&loc_front),
                );
                let html = templates::render_single_post(
                    &loc_front,
                    &settings,
                    user.as_ref(),
                    layout_json.as_deref(),
                    Some(&partials),
                );
                let final_html = crate::plugins::inject_plugin_hooks_into_html(
                    &db,
                    &settings,
                    Some(&loc_front),
                    html,
                );
                return db.hot_cache.store_and_serve(
                    &cache_key,
                    &headers,
                    user.is_some() || has_dp_filter || is_restricted,
                    final_html,
                );
            }
        } else if settings.show_on_front != "page" {
            // Blog index filtered by language
            let per_page = settings.posts_per_page.clamp(1, 100) as usize;
            let all_posts = db
                .get_posts("post", Some("publish"), None, 10_000)
                .unwrap_or_default();
            let lang_lower = clean_slug.to_ascii_lowercase();
            let filtered: Vec<_> = all_posts
                .into_iter()
                .filter(|p| p.lang_code.eq_ignore_ascii_case(&lang_lower))
                .collect();
            let (page_posts, total_pages) =
                templates::pagination::paginate(&filtered, page_num, per_page);
            let nav = templates::pagination::render_pagination(&req_path, page_num, total_pages);
            let req_ctx = RequestContext::for_front_page();
            let partials = SsrInjector::resolve_and_hydrate_partials(&db, &req_ctx, &settings);
            let html = templates::render_home_page_paginated(
                page_posts,
                &settings,
                user.as_ref(),
                Some(&partials),
                &nav,
            );
            let final_html =
                crate::plugins::inject_plugin_hooks_into_html(&db, &settings, None, html);
            return db
                .hot_cache
                .store_and_serve(&cache_key, &headers, user.is_some(), final_html);
        }
    }

    if let Some(mut post) = db
        .get_post_by_slug(&slug)
        .ok()
        .flatten()
        .filter(|p| crate::auth::can_view_post(p, user.as_ref()))
    {
        // WordPress canonical redirect: when a static front page is configured,
        // direct requests to /{front-page-slug}/ (e.g., /home/) 301 redirect to /
        if settings.show_on_front == "page"
            && settings.page_on_front > 0
            && post.id == settings.page_on_front
            && !has_dp_filter
        {
            let mut resp_headers = HeaderMap::new();
            if let Ok(loc) = "/".parse() {
                resp_headers.insert(header::LOCATION, loc);
            }
            return (
                StatusCode::MOVED_PERMANENTLY,
                resp_headers,
                Html(String::new()),
            )
                .into_response();
        }
        // Enforce password & member-gate locks BEFORE leeg.php / raw <!DOCTYPE html> bypasses!
        let (is_restricted, is_locked) =
            apply_content_access_guards(&db, &mut post, user.as_ref(), &headers);
        if !is_locked
            && (post.template == "page_templates/leeg.php"
                || post
                    .content
                    .rendered
                    .trim_start()
                    .starts_with("<!DOCTYPE html"))
        {
            return db.hot_cache.store_and_serve(
                &cache_key,
                &headers,
                user.is_some() || has_dp_filter || is_restricted,
                post.content.rendered,
            );
        }
        let raw_layout = db.get_post_layout(post.id).unwrap_or(None);
        if settings.show_on_front == "page"
            && settings.page_for_posts > 0
            && post.id == settings.page_for_posts
            && raw_layout.is_none()
            && post.content.rendered.trim().is_empty()
        {
            return posts_page_response(
                &db,
                &settings,
                user.as_ref(),
                &post,
                page_num,
                &cache_key,
                &headers,
            );
        }

        let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
        let expanded = if raw_layout.is_none() || is_locked {
            crate::plugins::expand_shortcodes_in_content_for_post(
                &post.content.rendered,
                &db,
                &settings,
                Some(&post),
            )
        } else {
            post.content.rendered.clone()
        };
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
        let layout_json = if is_locked {
            None
        } else {
            SsrInjector::hydrate_post_layout_json_with_query(
                &db,
                &req_ctx,
                &settings,
                Some(&post),
                raw_layout.as_deref(),
                paged.dp_tax.clone(),
                paged.dp_term.clone(),
                paged.dp_sort.clone(),
                paged.dp_q.clone(),
                paged.dp_page,
            )
            .or(raw_layout)
        };
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
        let final_html =
            crate::plugins::inject_plugin_hooks_into_html(&db, &settings, Some(&post), html);
        let (final_html, ab_assignments) = if settings
            .active_plugins
            .iter()
            .any(|p| p == "daypress-ab-testing")
        {
            let visitor_hash = db.compute_daily_visitor_hash(&headers, None);
            db.apply_ab_experiments_for_post(post.id, &visitor_hash, &final_html)
        } else {
            (final_html, Vec::new())
        };
        return db.hot_cache.store_and_serve(
            &cache_key,
            &headers,
            user.is_some() || has_dp_filter || is_restricted || !ab_assignments.is_empty(),
            final_html,
        );
    }

    if let Some(archive_resp) = crate::handlers::archives_ssr::try_handle_cpt_archive_with_query(
        &db,
        clean_slug,
        &settings,
        user.as_ref(),
        &paged,
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
    post: &models::DaypressPostResponse,
) -> Vec<TaxonomyTermRef> {
    let mut terms = Vec::new();
    if let Ok(conn) = db.conn.lock() {
        if let Ok(mut stmt) = conn.prepare_cached(
            "SELECT tt.taxonomy, t.term_id, t.slug FROM term_relationships r \
             JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id \
             JOIN terms t ON tt.term_id = t.term_id \
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

/// Computes the daily-salted cookieless `visitor_hash` (`SHA-256(ip + user_agent + YYYY-MM-DD)`)
/// matching `record_analytics_event_handler` so A/B test exposures and conversions correlate 1:1.
pub fn compute_visitor_hash_from_headers(headers: &HeaderMap) -> String {
    compute_visitor_hash_with_peer(headers, None)
}

pub fn compute_visitor_hash_with_peer(headers: &HeaderMap, peer_ip: Option<&str>) -> String {
    let client_ip = crate::auth::resolve_client_ip(headers, peer_ip);
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(client_ip.as_bytes());
    hasher.update(ua.as_bytes());
    hasher.update(today.as_bytes());
    hex::encode(hasher.finalize())
}

/// Extracts the `daypress_subscriber` session cookie (query parameters are intentionally
/// ignored so subscriber credentials cannot leak into URLs, Referer headers, or server logs).
pub fn extract_subscriber_token(headers: &HeaderMap, _query_token: Option<&str>) -> Option<String> {
    let cookie_hdr = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_hdr.split(';') {
        let trimmed = part.trim();
        if let Some(val) = trimmed.strip_prefix("daypress_subscriber=") {
            let v = val.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Applies both post-password protection and `post.member_gate`, redacting protected body/ACF
/// fields when the viewer is unauthorized and stripping `post.password` for non-editors.
/// Returns `(is_restricted, is_locked)`.
pub fn apply_content_access_guards(
    db: &Database,
    post: &mut models::DaypressPostResponse,
    user: Option<&crate::auth::AuthUser>,
    headers: &HeaderMap,
) -> (bool, bool) {
    let can_bypass = crate::auth::can_bypass_content_locks(post, user);
    let has_password = !post.password.trim().is_empty();
    if !can_bypass {
        post.password = String::new();
    }
    if has_password && !can_bypass {
        post.content.rendered = r#"<section class="dp-password-gate-card" style="margin:32px 0;padding:28px;border-radius:12px;border:1px solid #d4d4d8;text-align:center;"><h3>Password Protected</h3><p>This content is password protected.</p></section>"#.to_string();
        post.content.protected = true;
        post.excerpt.rendered = String::new();
        post.excerpt.protected = true;
        post.seo_description = String::new();
        post.meta = serde_json::json!({});
        post.acf = serde_json::json!({});
        return (true, true);
    }
    apply_member_gate_to_post(db, post, user, headers, None)
}

/// Redacts restricted or password-protected post fields before returning them over REST APIs.
pub fn redact_post_for_api(
    db: &Database,
    post: &mut models::DaypressPostResponse,
    user: Option<&crate::auth::AuthUser>,
    headers: &HeaderMap,
) -> bool {
    let (_is_restricted, is_locked) = apply_content_access_guards(db, post, user, headers);
    is_locked
}

/// Applies `post.member_gate` (`"public"`, `"subscribers_only"`, `"members_only"`).
/// Returns `(is_gated, is_locked)`. When `is_locked` is `true`, `post.content.rendered` is replaced
/// with a teaser preview + the Member Gate Paywall / Subscriber Unlock card.
pub fn apply_member_gate_to_post(
    db: &Database,
    post: &mut models::DaypressPostResponse,
    user: Option<&crate::auth::AuthUser>,
    headers: &HeaderMap,
    _query_subscriber_token: Option<&str>,
) -> (bool, bool) {
    let gate = post.member_gate.trim().to_ascii_lowercase();
    if gate != "subscribers_only" && gate != "members_only" {
        return (false, false);
    }

    let is_unlocked = if gate == "members_only" {
        user.is_some()
    } else {
        user.is_some()
            || extract_subscriber_token(headers, None)
                .is_some_and(|tok| db.verify_subscriber_token(&tok))
    };

    if is_unlocked {
        return (true, false);
    }

    let raw = &post.content.rendered;
    let teaser = if let Some(more_idx) = raw.find("<!--more-->") {
        raw[..more_idx].to_string()
    } else if !post.excerpt.rendered.trim().is_empty() {
        format!("<p>{}</p>", post.excerpt.rendered.trim())
    } else {
        let preview_chars: String = raw.chars().take(280).collect();
        format!(
            "<div class=\"dp-gated-teaser\">{}&hellip;</div>",
            preview_chars
        )
    };
    let gate_card = crate::plugins::render_member_gate_paywall_html(&post.title.rendered, &gate);
    post.excerpt.rendered = if raw.contains("<!--more-->") {
        teaser.clone()
    } else {
        String::new()
    };
    post.excerpt.protected = true;
    post.seo_description = String::new();
    post.meta = serde_json::json!({});
    post.content.rendered = format!("{}\n{}", teaser, gate_card);
    post.content.protected = true;
    post.acf = serde_json::json!({});
    (true, true)
}

/// `GET /newsletter/confirm/{token}`: Confirms a double opt-in subscriber and sets the `daypress_subscriber` cookie.
pub async fn handle_newsletter_confirm(
    State(db): State<Database>,
    req_headers: HeaderMap,
    crate::auth::TrustedProxy(trusted_proxy): crate::auth::TrustedProxy,
    AxPath(token): AxPath<String>,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    let secure = crate::auth::should_set_secure_cookies(&req_headers, trusted_proxy, &db);
    match db.confirm_subscriber_by_token(&token) {
        Ok(Some(sub)) => {
            let cookie_tok = db.issue_subscriber_cookie_token(sub.id);
            let cookie_val = crate::auth::format_cookie_with_security(
                format!(
                    "daypress_subscriber={}; Path=/; Max-Age=31536000; SameSite=Lax; HttpOnly",
                    cookie_tok
                ),
                secure,
            );
            let return_url = crate::auth::sanitize_local_redirect(Some(&sub.source_url), "/");
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Subscription Confirmed &mdash; {site_title}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #fafaf9; color: #18181b; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; padding: 20px;">
    <main style="max-width: 480px; width: 100%; background: #ffffff; border: 1px solid #e4e4e7; border-radius: 16px; padding: 36px 28px; text-align: center; box-shadow: 0 10px 25px -5px rgba(0,0,0,0.06);">
        <div style="font-size: 40px; margin-bottom: 12px;">✅</div>
        <h1 style="margin: 0 0 10px; font-size: 22px; font-weight: 800;">Subscription Confirmed!</h1>
        <p style="margin: 0 0 24px; font-size: 15px; line-height: 1.6; color: #52525b;">Thank you for confirming <strong>{email}</strong>. Your browser is now unlocked for all subscriber-exclusive posts on <strong>{site_title}</strong>.</p>
        <a href="{return_url}" style="display: inline-block; background: #a8401b; color: #ffffff; font-weight: 600; text-decoration: none; padding: 12px 24px; border-radius: 8px;">Continue Reading &rarr;</a>
    </main>
</body>
</html>"#,
                site_title = crate::templates::utils::escape_html(&settings.title),
                email = crate::templates::utils::escape_html(&sub.email),
                return_url = crate::templates::utils::escape_html(&return_url),
            );
            let mut resp_headers = HeaderMap::new();
            if let Ok(hv) = cookie_val.parse() {
                resp_headers.insert(header::SET_COOKIE, hv);
            }
            (StatusCode::OK, resp_headers, Html(html)).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Html(
                "<!DOCTYPE html><html><body style=\"font-family:sans-serif;text-align:center;padding:60px;\"><h1>Invalid or expired confirmation link</h1><p><a href=\"/\">Return home</a></p></body></html>"
                    .to_string(),
            ),
        )
            .into_response(),
    }
}

/// `GET|POST /newsletter/unsubscribe/{token}`: 1-Click RFC-8058 unsubscribe handler.
pub async fn handle_newsletter_unsubscribe(
    State(db): State<Database>,
    req_headers: HeaderMap,
    crate::auth::TrustedProxy(trusted_proxy): crate::auth::TrustedProxy,
    AxPath(token): AxPath<String>,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    let secure = crate::auth::should_set_secure_cookies(&req_headers, trusted_proxy, &db);
    match db.unsubscribe_by_token(&token) {
        Ok(Some(sub)) => {
            let mut resp_headers = HeaderMap::new();
            let clear_cookie = crate::auth::format_cookie_with_security(
                "daypress_subscriber=; Path=/; Max-Age=0; SameSite=Lax; HttpOnly".to_string(),
                secure,
            );
            if let Ok(hv) = clear_cookie.parse() {
                resp_headers.insert(header::SET_COOKIE, hv);
            }
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Unsubscribed &mdash; {site_title}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #fafaf9; color: #18181b; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; padding: 20px;">
    <main style="max-width: 480px; width: 100%; background: #ffffff; border: 1px solid #e4e4e7; border-radius: 16px; padding: 36px 28px; text-align: center;">
        <h1 style="margin: 0 0 10px; font-size: 22px; font-weight: 800;">You Have Been Unsubscribed</h1>
        <p style="margin: 0 0 24px; font-size: 15px; line-height: 1.6; color: #52525b;"><strong>{email}</strong> has been removed from our active mailing list and will no longer receive email dispatches.</p>
        <a href="/" style="display: inline-block; background: #18181b; color: #ffffff; font-weight: 600; text-decoration: none; padding: 11px 22px; border-radius: 8px;">Return to {site_title}</a>
    </main>
</body>
</html>"#,
                site_title = crate::templates::utils::escape_html(&settings.title),
                email = crate::templates::utils::escape_html(&sub.email),
            );
            (StatusCode::OK, resp_headers, Html(html)).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Html(
                "<!DOCTYPE html><html><body style=\"font-family:sans-serif;text-align:center;padding:60px;\"><h1>Unsubscribe link not found</h1><p><a href=\"/\">Return home</a></p></body></html>"
                    .to_string(),
            ),
        )
            .into_response(),
    }
}

/// `GET /go/{slug}`: Branded Shortlinks & Affiliate Tracker (`Pretty Links` parity).
/// Resolves `/go/{slug}`, appends configured UTM parameters (`utm_source`, `utm_medium`,
/// `utm_campaign`), increments click + unique visitor counters, logs a cookieless analytics
/// conversion event, and returns an HTTP `301`/`302`/`307` redirect with optional
/// `X-Robots-Tag: nofollow, sponsored` header.
pub async fn handle_shortlink_redirect(
    State(db): State<Database>,
    crate::auth::ClientIp(peer_ip): crate::auth::ClientIp,
    AxPath(slug): AxPath<String>,
    headers: HeaderMap,
) -> Response {
    let visitor_hash = db.compute_daily_visitor_hash(&headers, peer_ip.as_deref());

    let referrer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    match db.resolve_shortlink_and_record_click(&slug, &visitor_hash, referrer) {
        Some((resolved_target, status_code)) => {
            let status = match status_code {
                301 => StatusCode::MOVED_PERMANENTLY,
                302 => StatusCode::FOUND,
                307 => StatusCode::TEMPORARY_REDIRECT,
                308 => StatusCode::PERMANENT_REDIRECT,
                _ => StatusCode::TEMPORARY_REDIRECT,
            };
            let mut resp_headers = HeaderMap::new();
            if let Ok(loc) = resolved_target.parse() {
                resp_headers.insert(header::LOCATION, loc);
            }
            if let Ok(robots_val) = "nofollow, sponsored".parse() {
                resp_headers.insert("X-Robots-Tag", robots_val);
            }
            if let Ok(cc) = "no-store, no-cache, must-revalidate".parse() {
                resp_headers.insert(header::CACHE_CONTROL, cc);
            }
            (status, resp_headers, "").into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Html(
                "<!DOCTYPE html><html><body style=\"font-family:sans-serif;text-align:center;padding:60px;\"><h1>Shortlink not found</h1><p><a href=\"/\">Return home</a></p></body></html>"
                    .to_string(),
            ),
        )
            .into_response(),
    }
}
