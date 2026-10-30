//! DayStudio canvas ↔ public SSR parity regression tests.
//!
//! The builder canvas used to show the site header as unstyled links and stack `Grid` nodes in
//! one column, while the public page rendered a styled header and a multi-column grid. These
//! tests pin the fix:
//! 1. the artboard is framed with the exact header/footer markup the public page renders
//!    (native, Elementor or Theme Builder), and
//! 2. the public theme stylesheet (header, `.dp-grid`, breakpoints) is loaded into the builder,
//!    scoped to `#daystudio-artboard`, and
//! 3. the canvas renderer emits the same grid tracks as `daypress-ast`'s SSR,
//!    while the public SSR of the same page stays free of editor-only attributes.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
use daypress_server::theme_builder::models::*;
use daypress_server::theme_builder::ThemeTemplateRepository;
use daypress_server::{create_app, templates, Database};
use serde_json::{json, Value};
use tower::ServiceExt;

const ADMIN_BASIC: &str = "Basic YWRtaW46YWRtaW4=";

fn admin_req(method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", ADMIN_BASIC);
    match body {
        Some(val) => builder.body(Body::from(val.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

fn anon_get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn body_text(res: axum::response::Response) -> String {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Text between `start` (inclusive) and the first `end` after it (inclusive).
fn between<'a>(html: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = html.find(start)?;
    let len = html.get(from..)?.find(end)? + end.len();
    html.get(from..from + len)
}

fn site_css(builder_html: &str) -> &str {
    between(builder_html, "<style id=\"daystudio-kit-css\">", "</style>")
        .expect("builder must carry the scoped site stylesheet")
}

fn grid_page_layout() -> Value {
    let card = |id: &str, title: &str| {
        json!({
            "id": id, "type": "Container",
            "styles": { "background_color": "#ffffff", "padding_top": "24px", "border_radius": "10px" },
            "children": [
                { "id": format!("{id}_h"), "type": "Heading", "data": { "text": title, "level": 3 }, "styles": {} }
            ]
        })
    };
    json!({
        "version": 1,
        "root": {
            "id": "root", "type": "Root", "styles": {},
            "children": [{
                "id": "sec_values", "type": "Section",
                "styles": { "background_color": "#f6f1eb", "padding_top": "70px" },
                "children": [{
                    "id": "grid_values", "type": "Grid", "data": { "columns": 3, "gap": 24 }, "styles": {},
                    "children": [
                        card("c_v1", "Small batches"),
                        card("c_v2", "Direct sourcing"),
                        card("c_v3", "Fast shipping")
                    ]
                }]
            }]
        }
    })
}

/// Creates a published page with a 3-column Grid layout and returns its id.
async fn create_grid_page(app: &axum::Router, slug: &str) -> i64 {
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/wp/v2/pages",
            Some(json!({ "title": "Our Story", "slug": slug, "status": "publish", "content": "<p>Draft</p>" })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let page: Value = serde_json::from_str(&body_text(res).await).unwrap_or(Value::Null);
    let page_id = page["id"].as_i64().expect("page id");

    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            &format!("/wp-json/daypress/v1/layout/{}", page_id),
            Some(json!({ "layout": grid_page_layout() })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    page_id
}

#[tokio::test]
async fn test_builder_canvas_matches_public_header_and_grid_columns() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let page_id = create_grid_page(&app, "parity-story").await;

    // --- Public SSR of the page (logged-out visitor) ---
    let res = app
        .clone()
        .oneshot(anon_get("/parity-story/"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let public_html = body_text(res).await;
    assert!(
        public_html.contains(
            "class=\"dp-grid\" style=\"display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 24px;"
        ),
        "public SSR must render the Grid node as 3 columns"
    );
    assert!(
        !public_html.contains("contenteditable"),
        "public SSR leaked contenteditable"
    );
    assert!(
        !public_html.contains("elementor-grid-handle"),
        "public SSR leaked elementor-grid-handle"
    );
    assert!(
        !public_html.contains("draggable=\"true\""),
        "public SSR leaked draggable"
    );
    let public_header = between(&public_html, "<header class=\"day-header\">", "</header>")
        .expect("public page renders the native site header");

    // --- DayStudio builder for the same page ---
    let res = app
        .clone()
        .oneshot(admin_req(
            "GET",
            &format!("/d-admin/builder?post={}", page_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let builder_html = body_text(res).await;

    // 1. Same header markup as the public page, inside the artboard's header slot
    let header_slot = between(
        &builder_html,
        "<div id=\"daystudio-ssr-header\">",
        "</header>",
    )
    .expect("builder renders a site header in the artboard");
    assert!(
        header_slot.ends_with(public_header),
        "builder header must be the public header markup.\nbuilder: {header_slot}\npublic: {public_header}"
    );
    assert!(
        !builder_html.contains("preview-site-header"),
        "unstyled preview header is gone"
    );
    assert!(
        builder_html.contains("<footer class=\"day-footer\">"),
        "public footer framed too"
    );

    // 2. Public theme CSS is loaded and scoped to the artboard
    let css = site_css(&builder_html);
    assert!(
        css.contains("@scope (#daystudio-artboard) {"),
        "site CSS must be scoped"
    );
    assert!(
        css.contains("header.day-header {"),
        "header styles must reach the canvas"
    );
    assert!(
        css.contains("nav.day-nav {"),
        "nav styles must reach the canvas"
    );
    assert!(
        css.contains(".dp-grid {"),
        "grid styles must reach the canvas"
    );
    assert!(
        css.contains(".dp-section {"),
        "section styles must reach the canvas"
    );
    assert!(
        !css.contains("html, body {"),
        "document root rules must be mapped onto :scope"
    );
    assert!(
        css.contains(":scope {"),
        "document root rules must be mapped onto :scope"
    );
    assert!(
        !css.contains("-:scope"),
        "class names ending in -body must not be corrupted"
    );
    // Desktop artboard keeps window breakpoints (3 columns on a desktop screen, like public SSR)
    assert!(
        css.contains(":where(:scope.artboard-desktop) .dp-grid {"),
        "the 1-column grid breakpoint must only follow the window on the desktop artboard"
    );
    // Tablet/mobile artboards answer the same breakpoints by their own width
    assert!(
        css.contains("@container daystudio-artboard (max-width: 900px) {"),
        "tablet/mobile previews must hit the site breakpoints"
    );
    assert!(
        builder_html.contains(
            "#daystudio-artboard.artboard-tablet,\n#daystudio-artboard.artboard-mobile {\n    container: daystudio-artboard / inline-size;"
        ),
        "tablet/mobile artboards must be the breakpoint query container"
    );

    // 3. Canvas renderer emits the SSR grid tracks for Grid nodes
    assert!(
        builder_html.contains("\"columns\":3"),
        "Grid data reaches the canvas"
    );
    assert!(builder_html.contains("el.className = \"dp-grid\";"));
    assert!(!builder_html.contains("el.className = \"elementor-grid\";"));
    assert!(builder_html.contains("layoutDeclarations(node, own).forEach"));
    assert!(builder_html
        .contains("[\"grid-template-columns\", `repeat(${count(d.columns, 3)}, minmax(0, 1fr))`]"));
    assert!(builder_html.contains("[\"gap\", `${count(d.gap, 0)}px`]"));
}

#[tokio::test]
async fn test_builder_canvas_uses_theme_builder_header_like_public_page() {
    let db = Database::new_in_memory().expect("in-memory db");
    let repo = ThemeTemplateRepository::from_database(&db);
    let mut header_layout = DaypressLayout::default();
    header_layout.root.children.push(DaypressNode {
        id: "tb_header_brand".to_string(),
        kind: NodeKind::Text {
            content_html: "<p>Harbor Header Brand</p>".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    });
    repo.create(&CreateTemplateRequest {
        title: "Site Header".to_string(),
        template_type: TemplateType::Header,
        status: Some("publish".to_string()),
        layout: Some(header_layout),
        layout_json: None,
        settings: None,
        priority: Some(10),
        is_active: Some(true),
        conditions: Some(vec![CreateConditionDto {
            id: None,
            condition_type: ConditionType::Include,
            rule_target: RuleTarget::EntireSite,
            target_id: String::new(),
            priority: 10,
        }]),
    })
    .expect("create theme header");

    let app = create_app(db.clone());
    let page_id = create_grid_page(&app, "parity-tb").await;

    let public_html = body_text(app.clone().oneshot(anon_get("/parity-tb/")).await.unwrap()).await;
    let public_header = between(
        &public_html,
        "<header class=\"dp-theme-header\">",
        "</header>",
    )
    .expect("public page renders the Theme Builder header");
    assert!(public_header.contains("Harbor Header Brand"));

    let builder_html = body_text(
        app.clone()
            .oneshot(admin_req(
                "GET",
                &format!("/d-admin/builder?post={}", page_id),
                None,
            ))
            .await
            .unwrap(),
    )
    .await;
    let header_slot = between(
        &builder_html,
        "<div id=\"daystudio-ssr-header\">",
        "</header>",
    )
    .expect("builder renders the Theme Builder header");
    assert!(
        header_slot.ends_with(public_header),
        "builder must frame the artboard with the Theme Builder header.\nbuilder: {header_slot}"
    );
    assert!(
        !header_slot.contains("day-header"),
        "fallback header must not be shown"
    );
}

#[test]
fn test_builder_site_css_follows_the_public_html_theme_class() {
    let db = Database::new_in_memory().expect("in-memory db");
    let posts = db.get_posts("post", None, None, 1).expect("seed posts");
    let post = posts.first().expect("seed post");
    let layout = grid_page_layout().to_string();

    // Native dark theme: default Elementor kit fonts load, Elementor-only overrides stay inert
    let settings = db.get_settings().unwrap_or_default();
    let html = templates::render_builder_view(&settings, post, Some(&layout));
    let css = site_css(&html);
    assert!(
        css.starts_with("<style id=\"daystudio-kit-css\">@import url('https://fonts.googleapis.com/css2?family=Roboto"),
        "kit font imports must be hoisted in front of @scope"
    );
    assert!(css.contains("html.daystudio-theme-active .dp-heading"));
    assert!(!css.contains(":scope .dp-heading"));

    // Imported Elementor site: html.elementor-theme-active rules apply to the artboard
    let mut elementor = settings.clone();
    elementor.daystudio_kit_css = ":root { --e-global-color-primary: #c8742c; }".to_string();
    let html = templates::render_builder_view(&elementor, post, Some(&layout));
    let css = site_css(&html);
    assert!(
        css.contains(":scope .dp-heading {"),
        "Elementor heading colors apply on canvas"
    );
    assert!(
        css.contains(":scope {\n\t\t\tbackground-color: #ffffff;"),
        "Elementor body background applies"
    );
    assert!(
        css.contains(":scope { --e-global-color-primary: #c8742c; }"),
        "kit CSS stays scoped"
    );
}

#[test]
fn test_builder_responsive_artboard_width_fonts_and_scope_container_parity() {
    let db = Database::new_in_memory().expect("in-memory db");
    let posts = db.get_posts("post", None, None, 1).expect("seed posts");
    let post = posts.first().expect("seed post");
    let layout = grid_page_layout().to_string();
    let settings = db.get_settings().unwrap_or_default();
    let html = templates::render_builder_view(&settings, post, Some(&layout));

    // 1. Desktop artboard wrapper is a fixed 1200px (flex-shrink: 0) so studio panels never squeeze it narrower
    assert!(
        html.contains("#daystudio-zoom-wrapper {")
            && html.contains("width: 1200px;\n    flex-shrink: 0;"),
        "zoom wrapper must maintain 1200px width without shrinking"
    );
    // 2. Tablet and mobile zoom-wrapper parents act as container-query hosts so @container daystudio-artboard rules targeting :scope (body/html/:root) match #daystudio-artboard itself
    assert!(
        html.contains("#daystudio-zoom-wrapper:has(> #daystudio-artboard.artboard-tablet)")
            && html.contains("#daystudio-zoom-wrapper:has(> #daystudio-artboard.artboard-mobile)"),
        "parent zoom wrapper must be a daystudio-artboard query container for :scope body/root rules"
    );
    // 3. Public base font stack and artboard baseline font stack match
    assert!(
        html.contains("font-family: system-ui, -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif;"),
        "builder artboard and public SSR must share the exact same default font stack"
    );
}
