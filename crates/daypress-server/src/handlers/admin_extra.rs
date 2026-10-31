//! Extra admin page handlers (`handlers/admin_extra.rs`).

use super::params::CommentQueryParams;
use crate::models;
use crate::templates;
use crate::Database;
use axum::extract::{Form, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};

/// 1. Dashboard Updates
pub async fn handle_admin_updates(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    let update_status = crate::updater::get_update_status(&db);
    Html(templates::render_updates_view_with_status(
        &settings,
        &stats,
        &update_status,
    ))
}

/// 2. Media Add New
pub async fn handle_admin_media_new(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_media_new_view(&settings))
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
    Html(templates::render_comments_view(
        &settings,
        &comments,
        &counts,
        params.comment_status.as_deref(),
    ))
}

/// 4. Theme Header & Footer Manager
pub async fn handle_admin_headers_footers(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let matrix = crate::theme_builder::build_header_footer_matrix(&db);
    let starters = crate::theme_builder::get_starter_catalog();
    Html(templates::render_headers_footers_view(
        &settings, &matrix, &starters,
    ))
}

pub fn template_to_builder_post(
    tpl: &crate::theme_builder::ThemeTemplateRecord,
) -> crate::models::DaypressPostResponse {
    use crate::models::{DaypressGuid, DaypressPostResponse, DaypressRenderedText};
    DaypressPostResponse {
        id: tpl.id,
        date: tpl.created_at.clone(),
        date_gmt: tpl.created_at.clone(),
        guid: DaypressGuid {
            rendered: String::new(),
        },
        modified: tpl.updated_at.clone(),
        modified_gmt: tpl.updated_at.clone(),
        slug: tpl.template_type.as_str().to_string(),
        status: tpl.status.clone(),
        post_type: format!("template:{}", tpl.template_type.as_str()),
        link: format!("/wp-json/daypress/v1/templates/preview/{}", tpl.id),
        title: DaypressRenderedText {
            rendered: tpl.title.clone(),
            protected: false,
        },
        content: DaypressRenderedText {
            rendered: tpl.rendered_html.clone().unwrap_or_default(),
            protected: false,
        },
        excerpt: DaypressRenderedText {
            rendered: String::new(),
            protected: false,
        },
        author: 1,
        featured_media: 0,
        comment_status: "closed".to_string(),
        ping_status: "closed".to_string(),
        sticky: false,
        template: "default".to_string(),
        format: "standard".to_string(),
        meta: serde_json::json!({}),
        categories: vec![],
        tags: vec![],
        parent: 0,
        menu_order: 0,
        author_name: "admin".to_string(),
        category_names: vec![],
        tag_names: vec![],
        featured_image_url: None,
        comments: vec![],
        seo_title: String::new(),
        seo_description: String::new(),
        seo_keyword: String::new(),
        seo_canonical: String::new(),
        seo_noindex: false,
        seo_schema_type: "WebPage".to_string(),
        seo_og_image: String::new(),
    }
}

/// Theme Builder header/footer the public page resolves for `post`, so DayStudio frames the
/// artboard with the same site chrome visitors see.
pub(crate) fn builder_theme_partials(
    db: &Database,
    settings: &models::DaypressSettingsResponse,
    post: &models::DaypressPostResponse,
) -> crate::theme_builder::models::ResolvedThemePartials {
    let terms = super::public_ssr::hydrate_post_terms(db, post);
    let req_ctx = crate::theme_builder::models::RequestContext::for_post(
        post.id,
        &post.post_type,
        settings.page_on_front > 0 && post.id == settings.page_on_front,
        terms,
    );
    crate::theme_builder::ssr_injector::SsrInjector::resolve_and_hydrate_partials_with_post(
        db,
        &req_ctx,
        settings,
        Some(post),
    )
}

/// Stand-in post DayStudio opens on when the site has no content yet.
pub(crate) fn placeholder_builder_post() -> models::DaypressPostResponse {
    models::DaypressPostResponse {
        id: 1,
        date: "2026-10-22T00:00:00".to_string(),
        date_gmt: "2026-10-22T00:00:00".to_string(),
        guid: models::DaypressGuid {
            rendered: "".to_string(),
        },
        modified: "2026-10-22T00:00:00".to_string(),
        modified_gmt: "2026-10-22T00:00:00".to_string(),
        slug: "sample-page".to_string(),
        status: "publish".to_string(),
        post_type: "page".to_string(),
        link: "/sample-page/".to_string(),
        title: models::DaypressRenderedText {
            rendered: "Sample Page".to_string(),
            protected: false,
        },
        content: models::DaypressRenderedText {
            rendered: "".to_string(),
            protected: false,
        },
        excerpt: models::DaypressRenderedText {
            rendered: "".to_string(),
            protected: false,
        },
        author: 1,
        featured_media: 0,
        comment_status: "open".to_string(),
        ping_status: "open".to_string(),
        sticky: false,
        template: "default".to_string(),
        format: "standard".to_string(),
        meta: serde_json::json!({}),
        categories: vec![1],
        tags: vec![],
        parent: 0,
        menu_order: 0,
        author_name: "admin".to_string(),
        category_names: vec![],
        tag_names: vec![],
        featured_image_url: None,
        comments: vec![],
        seo_title: String::new(),
        seo_description: String::new(),
        seo_keyword: String::new(),
        seo_canonical: String::new(),
        seo_noindex: false,
        seo_schema_type: "WebPage".to_string(),
        seo_og_image: String::new(),
    }
}

pub async fn handle_setup_page(State(db): State<Database>) -> impl IntoResponse {
    if !db.needs_onboarding() {
        return axum::response::Redirect::to("/d-admin/").into_response();
    }
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_onboarding_view(&settings, None, None)).into_response()
}

pub async fn handle_setup_submit(
    State(db): State<Database>,
    Form(form): Form<crate::db::OnboardingSetupRequest>,
) -> impl IntoResponse {
    if !db.needs_onboarding() {
        return axum::response::Redirect::to("/login").into_response();
    }

    match db.complete_onboarding(&form) {
        Ok(user) => {
            let mut headers = HeaderMap::new();
            let secret = db.get_auth_secret().unwrap_or_default();
            let ver = db
                .get_user_detail(user.user_id)
                .ok()
                .flatten()
                .map(|u| u.session_ver)
                .unwrap_or(1);
            let cookie_val =
                crate::auth::session_cookie_header_with_version(user.user_id, &secret, true, ver);
            if let Ok(val) = cookie_val.parse() {
                headers.insert(header::SET_COOKIE, val);
            }
            let target = if form.starter_mode.trim().eq_ignore_ascii_case("migrate") {
                "/d-admin/import?welcome=1"
            } else {
                "/d-admin/?welcome=1"
            };
            if let Ok(loc) = target.parse() {
                headers.insert(header::LOCATION, loc);
            }
            (StatusCode::SEE_OTHER, headers, Html(String::new())).into_response()
        }
        Err(err) => {
            let settings = db.get_settings().unwrap_or_default();
            let msg = match err {
                crate::AppError::BadRequest(m)
                | crate::AppError::Forbidden(m)
                | crate::AppError::NotFound(m)
                | crate::AppError::Unauthorized(m) => m,
                other => other.to_string(),
            };
            (
                StatusCode::BAD_REQUEST,
                Html(templates::render_onboarding_view(
                    &settings,
                    Some(&form),
                    Some(&msg),
                )),
            )
                .into_response()
        }
    }
}
