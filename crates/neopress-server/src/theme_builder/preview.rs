//! Standalone Preview Renderer for Theme Builder Templates.

use super::models::{BreadcrumbItem, TemplateType};
use super::repository::ThemeTemplateRepository;
use super::ssr_injector::{HydrationContext, SsrInjector};
use crate::adapters::database::Database;
use crate::errors::AppError;
use crate::models::{WpPostResponse, WpSettingsResponse};
use axum::{
    extract::{Path as AxPath, State},
    response::Html,
};
use neopress_ast::{compile_layout_css, render_layout_to_html, sanitize_ssr_html};
use std::collections::HashMap;

pub async fn preview_template_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Html<String>, AppError> {
    let repo = ThemeTemplateRepository::from_database(&db);
    let template = repo
        .get_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))?;

    let settings = db.get_settings().unwrap_or_else(|_| WpSettingsResponse {
        title: "Preview Site".to_string(),
        description: "Visual Theme Builder Preview".to_string(),
        url: "http://localhost:8090".to_string(),
        ..Default::default()
    });

    let mock_post = WpPostResponse {
        id: 999,
        date: "2026-10-23 12:00:00".to_string(),
        date_gmt: "2026-10-23 12:00:00".to_string(),
        guid: crate::models::WpGuid { rendered: "http://localhost:8090/?p=999".to_string() },
        modified: "2026-10-23 12:00:00".to_string(),
        modified_gmt: "2026-10-23 12:00:00".to_string(),
        slug: "sample-preview-post".to_string(),
        status: "publish".to_string(),
        post_type: "post".to_string(),
        link: "http://localhost:8090/sample-preview-post/".to_string(),
        title: crate::models::WpRenderedText {
            rendered: "Designing for 2026: The New Web Architect Standard".to_string(),
            protected: false,
        },
        content: crate::models::WpRenderedText {
            rendered: "<p>This is a live preview of how dynamic post content renders inside your custom Theme Builder template layout. Every element is generated server-side with zero client overhead.</p>".to_string(),
            protected: false,
        },
        excerpt: crate::models::WpRenderedText {
            rendered: "A live preview of dynamic content in Neopress Theme Builder.".to_string(),
            protected: false,
        },
        author: 1,
        featured_media: 0,
        comment_status: "open".to_string(),
        ping_status: "open".to_string(),
        sticky: false,
        template: "".to_string(),
        format: "standard".to_string(),
        meta: serde_json::json!({}),
        categories: vec![],
        tags: vec![],
        parent: 0,
        menu_order: 0,
        author_name: "Lead Architect".to_string(),
        category_names: vec!["Architecture".to_string()],
        tag_names: vec!["2026".to_string()],
        featured_image_url: Some("/static/placeholder.jpg".to_string()),
        comments: vec![],
        seo_title: String::new(),
        seo_description: String::new(),
        seo_keyword: String::new(),
        seo_canonical: String::new(),
        seo_noindex: false,
        seo_schema_type: "Article".to_string(),
        seo_og_image: String::new(),
    };

    let breadcrumbs = vec![
        BreadcrumbItem {
            title: "Architecture".to_string(),
            url: "/category/architecture".to_string(),
            is_current: false,
        },
        BreadcrumbItem {
            title: mock_post.title.rendered.clone(),
            url: mock_post.link.clone(),
            is_current: true,
        },
    ];

    let hyd_ctx = HydrationContext {
        post: Some(&mock_post),
        settings: &settings,
        breadcrumbs,
        nav_menus: HashMap::new(),
    };

    let mut layout = template.layout.clone();
    SsrInjector::hydrate_layout(&mut layout, &hyd_ctx);

    let raw_body = render_layout_to_html(&layout);
    let compiled_css = compile_layout_css(&layout);

    let container_class = match template.template_type {
        TemplateType::Header => "np-theme-header",
        TemplateType::Footer => "np-theme-footer",
        TemplateType::SinglePost | TemplateType::SinglePage => "np-theme-single",
        TemplateType::Archive => "np-theme-archive",
        TemplateType::Error404 => "np-theme-404",
        TemplateType::Popup => "np-theme-popup",
        TemplateType::SectionSymbol => "np-theme-symbol",
    };

    let doc = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Preview: {title}</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" />
    <style>
        *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; background: #0D0F12; color: #F2F4F8; min-height: 100vh; }}
        .preview-banner {{ background: #15181E; border-bottom: 1px solid #242933; padding: 8px 16px; font-size: 12px; color: #8A93A6; display: flex; justify-content: space-between; align-items: center; }}
        .preview-badge {{ background: #2563EB; color: #FFFFFF; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 11px; }}
        {compiled_css}
    </style>
</head>
<body>
    <div class="preview-banner">
        <div>Theme Builder Live Preview: <strong>{title}</strong> ({type_str})</div>
        <div class="preview-badge">ID: {id}</div>
    </div>
    <div class="{container_class}">
        {raw_body}
    </div>
</body>
</html>"#,
        title = template.title,
        type_str = template.template_type.as_str(),
        id = template.id,
        compiled_css = compiled_css,
        container_class = container_class,
        raw_body = raw_body,
    );

    let sanitized = sanitize_ssr_html(&doc);
    Ok(Html(sanitized))
}
