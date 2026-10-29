//! Extra admin page handlers (`handlers/admin_extra.rs`).

use super::params::CommentQueryParams;
use crate::models;
use crate::templates;
use crate::Database;
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

/// 4. Theme Header & Footer Manager
pub async fn handle_admin_headers_footers(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let matrix = crate::theme_builder::build_header_footer_matrix(&db);
    let starters = crate::theme_builder::get_starter_catalog();
    Html(templates::render_wp_headers_footers_view(
        &settings, &matrix, &starters,
    ))
}

pub fn template_to_builder_post(
    tpl: &crate::theme_builder::ThemeTemplateRecord,
) -> crate::models::WpPostResponse {
    use crate::models::{WpGuid, WpPostResponse, WpRenderedText};
    WpPostResponse {
        id: tpl.id,
        date: tpl.created_at.clone(),
        date_gmt: tpl.created_at.clone(),
        guid: WpGuid {
            rendered: String::new(),
        },
        modified: tpl.updated_at.clone(),
        modified_gmt: tpl.updated_at.clone(),
        slug: tpl.template_type.as_str().to_string(),
        status: tpl.status.clone(),
        post_type: format!("template:{}", tpl.template_type.as_str()),
        link: format!("/wp-json/neopress/v1/templates/preview/{}", tpl.id),
        title: WpRenderedText {
            rendered: tpl.title.clone(),
            protected: false,
        },
        content: WpRenderedText {
            rendered: tpl.rendered_html.clone().unwrap_or_default(),
            protected: false,
        },
        excerpt: WpRenderedText {
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

/// Theme Builder header/footer the public page resolves for `post`, so NeoStudio frames the
/// artboard with the same site chrome visitors see.
pub(crate) fn builder_theme_partials(
    db: &Database,
    settings: &models::WpSettingsResponse,
    post: &models::WpPostResponse,
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

/// Stand-in post NeoStudio opens on when the site has no content yet.
pub(crate) fn placeholder_builder_post() -> models::WpPostResponse {
    models::WpPostResponse {
        id: 1,
        date: "2026-10-22T00:00:00".to_string(),
        date_gmt: "2026-10-22T00:00:00".to_string(),
        guid: models::WpGuid {
            rendered: "".to_string(),
        },
        modified: "2026-10-22T00:00:00".to_string(),
        modified_gmt: "2026-10-22T00:00:00".to_string(),
        slug: "sample-page".to_string(),
        status: "publish".to_string(),
        post_type: "page".to_string(),
        link: "/sample-page/".to_string(),
        title: models::WpRenderedText {
            rendered: "Sample Page".to_string(),
            protected: false,
        },
        content: models::WpRenderedText {
            rendered: "".to_string(),
            protected: false,
        },
        excerpt: models::WpRenderedText {
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
