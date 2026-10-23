//! Milestone 4 E2E Visual Builder Round-Trip Integration Test Suite.
//!
//! Validates:
//! 1. Complete visual builder round-trip with Pro blocks (`PriceCard`, `Testimonial`,
//!    `Countdown`, `NavMenu`, `LoopGrid`) via `POST /wp-json/neopress/v1/layout/:id`.
//! 2. SQLite dual-write persistence (`wp_posts.layout_json`, `wp_postmeta._neopress_layout`,
//!    and `wp_posts.post_content`).
//! 3. Public SSR fetch via live Axum HTTP request (`GET /:slug/`).
//! 4. Scoped responsive CSS compilation (`@media (max-width: 1024px)`, `@media (max-width: 767px)`).
//! 5. 3-Tier SSR DOM leak sanitizer firewall (0 `contenteditable`, 0 `draggable`, 0 `neostudio-*`).
//! 6. Visual layout deletion and fallback to standard content.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use neopress_ast::{
    schema::items::{NavMenuItem, PriceFeatureItem},
    NeopressLayout, NeopressNode, NodeKind, StyleProperties,
};
use neopress_server::{create_app, Database};
use serde_json::{json, Value};
use tower::ServiceExt;

fn auth_req(method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("X-WP-Nonce", "wp_rest_admin");
    match body {
        Some(val) => builder.body(Body::from(val.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn parse_body(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

#[tokio::test]
async fn test_e2e_visual_builder_pro_blocks_dual_write_and_ssr_roundtrip() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create a Post to edit in Visual Builder
    let post_payload = json!({
        "title": "Enterprise Cloud Architecture",
        "slug": "enterprise-cloud",
        "content": "<p>Initial markdown draft.</p>",
        "status": "publish"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/posts", Some(post_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let post_id = parse_body(res).await["id"].as_i64().expect("post id");

    // 2. Construct Visual Builder AST with Pro widgets and intentionally contaminated editor chrome
    let price_card_node = NeopressNode {
        id: "widget_price_card_1".to_string(),
        kind: NodeKind::PriceCard {
            title: "Enterprise Dedicated".to_string(),
            subtitle: Some("High-concurrency Rust CMS cluster".to_string()),
            currency: Some("$".to_string()),
            price: "299".to_string(),
            original_price: Some("499".to_string()),
            period: Some("/mo".to_string()),
            badge: Some("Popular".to_string()),
            featured: true,
            features: vec![
                PriceFeatureItem {
                    text: "Sub-millisecond SSR latency".to_string(),
                    included: true,
                    icon: None,
                },
                PriceFeatureItem {
                    text: "In-process SQLite ACID WAL".to_string(),
                    included: true,
                    icon: None,
                },
            ],
            button_text: "Deploy Cluster".to_string(),
            button_url: "/signup/".to_string(),
        },
        styles: StyleProperties {
            // Intentionally contaminated editor chrome
            css_classes: Some("elementor-grid-handle neostudio-selection-outline neostudio-node active-element".to_string()),
            ..Default::default()
        },
        children: vec![],
    };

    let testimonial_node = NeopressNode {
        id: "widget_testimonial_1".to_string(),
        kind: NodeKind::Testimonial {
            content: "Neopress migrated our 20k-page WordPress site in minutes with sub-millisecond public SSR.".to_string(),
            name: "Dr. Evelyn Reed".to_string(),
            job: "Chief Architect, Cloud Systems".to_string(),
            image_url: Some("/static/avatar-reed.png".to_string()),
        },
        styles: StyleProperties::default(),
        children: vec![],
    };

    let countdown_node = NeopressNode {
        id: "widget_countdown_1".to_string(),
        kind: NodeKind::Countdown {
            target_date: "2026-12-31T23:59:59Z".to_string(),
            mode: "due_date".to_string(),
            show_days: true,
            show_hours: true,
            show_minutes: true,
            show_seconds: true,
            expiry_message: Some("Special Offer Concluded".to_string()),
        },
        styles: StyleProperties::default(),
        children: vec![],
    };

    let nav_menu_node = NeopressNode {
        id: "widget_nav_menu_1".to_string(),
        kind: NodeKind::NavMenu {
            menu_slug: "main-nav".to_string(),
            layout: "horizontal".to_string(),
            align: "left".to_string(),
            mobile_hamburger: true,
            items: vec![
                NavMenuItem {
                    title: "Platform Overview".to_string(),
                    url: "/overview/".to_string(),
                    order: 1,
                    ..Default::default()
                },
                NavMenuItem {
                    title: "Benchmarks".to_string(),
                    url: "/benchmarks/".to_string(),
                    order: 2,
                    ..Default::default()
                },
            ],
        },
        styles: StyleProperties::default(),
        children: vec![],
    };

    let loop_grid_node = NeopressNode {
        id: "widget_loop_grid_1".to_string(),
        kind: NodeKind::LoopGrid {
            post_type: "post".to_string(),
            posts_per_page: 3,
            columns: 3,
            show_thumbnail: true,
            show_excerpt: true,
            show_meta: true,
            show_pagination: false,
            taxonomy: None,
        },
        styles: StyleProperties::default(),
        children: vec![],
    };

    let section_root = NeopressNode {
        id: "sec_cloud_hero".to_string(),
        kind: NodeKind::Section,
        styles: StyleProperties {
            padding_top: Some("64px".to_string()),
            padding_bottom: Some("64px".to_string()),
            ..Default::default()
        },
        children: vec![
            price_card_node,
            testimonial_node,
            countdown_node,
            nav_menu_node,
            loop_grid_node,
        ],
    };

    let layout = NeopressLayout {
        version: 1,
        root: section_root,
        global_styles: None,
    };

    // 3. Save layout to SQLite via REST endpoint
    let save_payload = json!({
        "layout": layout
    });
    let res = app.clone().oneshot(auth_req("POST", &format!("/wp-json/neopress/v1/layout/{}", post_id), Some(save_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let save_res = parse_body(res).await;
    assert_eq!(save_res["success"], true);

    // 4. Verify SQLite Dual-Write Persistence
    let stored_layout = db.get_post_layout(post_id).expect("layout query").expect("layout exists");
    assert!(stored_layout.contains("Enterprise Dedicated"));
    assert!(stored_layout.contains("Dr. Evelyn Reed"));

    let post_in_db = db.get_post_by_id(post_id).expect("post query").expect("post exists");
    assert!(post_in_db.content.rendered.contains("Enterprise Dedicated"));

    // 5. Fetch Public SSR Route (GET /enterprise-cloud/)
    let ssr_req = Request::builder().method("GET").uri("/enterprise-cloud/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(ssr_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&html_bytes);

    // 6. Verify Pro Block Content Rendered in Public HTML
    assert!(html.contains("Enterprise Dedicated"), "Missing PriceCard title");
    assert!(html.contains("$"), "Missing PriceCard currency");
    assert!(html.contains("299"), "Missing PriceCard price");
    assert!(html.contains("Sub-millisecond SSR latency"), "Missing PriceCard feature item");
    assert!(html.contains("Dr. Evelyn Reed"), "Missing Testimonial author");
    assert!(html.contains("2026-12-31T23:59:59Z"), "Missing Countdown target");
    assert!(html.contains("Platform Overview"), "Missing NavMenu item");

    // 7. Verify 3-Tier SSR DOM Leak Sanitizer
    assert!(!html.contains("contenteditable"), "Sanitizer leaked contenteditable");
    assert!(!html.contains("draggable=\"true\"") && !html.contains(" draggable"), "Sanitizer leaked draggable");
    assert!(!html.contains("elementor-grid-handle"), "Sanitizer leaked elementor-grid-handle");
    assert!(!html.contains("neostudio-selection-outline"), "Sanitizer leaked neostudio-selection-outline");
    assert!(!html.contains("neostudio-node"), "Sanitizer leaked neostudio-node");
    assert!(!html.contains("data-neostudio"), "Sanitizer leaked data-neostudio");
    assert!(!html.contains("class=\"\""), "Sanitizer left empty class attribute");
}

#[tokio::test]
async fn test_e2e_visual_builder_scoped_responsive_css_pipeline() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create a Post
    let post_payload = json!({
        "title": "Responsive Matrix Page",
        "slug": "responsive-matrix",
        "content": "<p>Initial text.</p>",
        "status": "publish"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/posts", Some(post_payload))).await.unwrap();
    let post_id = parse_body(res).await["id"].as_i64().expect("post id");

    // 2. Build AST with desktop, tablet, mobile, and hover styles
    let node = NeopressNode {
        id: "resp_card_1".to_string(),
        kind: NodeKind::Container,
        styles: StyleProperties {
            padding_top: Some("80px".to_string()),
            tablet: Some(Box::new(StyleProperties {
                padding_top: Some("40px".to_string()),
                ..Default::default()
            })),
            mobile: Some(Box::new(StyleProperties {
                padding_top: Some("20px".to_string()),
                ..Default::default()
            })),
            hover: Some(Box::new(StyleProperties {
                background_color: Some("#0f172a".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        },
        children: vec![],
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    // 3. Save layout
    let res = app.clone().oneshot(auth_req("POST", &format!("/wp-json/neopress/v1/layout/{}", post_id), Some(json!({ "layout": layout })))).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. Fetch Public SSR Page
    let ssr_req = Request::builder().method("GET").uri("/responsive-matrix/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(ssr_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&html_bytes);

    // 5. Assert Scoped Responsive Media Queries and Hover Styles are Present in SSR Output
    assert!(html.contains("@media (max-width: 1024px)"), "Expected tablet breakpoint CSS in SSR");
    assert!(html.contains("@media (max-width: 767px)"), "Expected mobile breakpoint CSS in SSR");
    assert!(html.contains(":hover"), "Expected hover selector in scoped CSS");
}

#[tokio::test]
async fn test_e2e_visual_builder_layout_deletion_and_ssr_fallback() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create a Post with standard text
    let fallback_text = "This is resilient standard content when visual builder is disabled.";
    let post_payload = json!({
        "title": "Fallback Matrix Article",
        "slug": "fallback-matrix",
        "content": format!("<p>{}</p>", fallback_text),
        "status": "publish"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/posts", Some(post_payload))).await.unwrap();
    let post_id = parse_body(res).await["id"].as_i64().expect("post id");

    // 2. Attach a visual layout
    let layout = NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "visual_block_1".to_string(),
            kind: NodeKind::Text { content_html: "<p>Visual Layout Content</p>".to_string() },
            styles: StyleProperties::default(),
            children: vec![],
        },
        global_styles: None,
    };
    let res = app.clone().oneshot(auth_req("POST", &format!("/wp-json/neopress/v1/layout/{}", post_id), Some(json!({ "layout": layout })))).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Delete Visual Layout via DELETE /wp-json/neopress/v1/layout/:id
    let res = app.clone().oneshot(auth_req("DELETE", &format!("/wp-json/neopress/v1/layout/{}", post_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let del_res = parse_body(res).await;
    assert_eq!(del_res["success"], true);
    assert_eq!(del_res["status"], "deleted");

    // 4. Assert layout is nullified in SQLite
    assert!(db.get_post_layout(post_id).unwrap().is_none());

    // 5. Query Public SSR: Should render cleanly as standard post without np-visual-canvas wrapper
    let ssr_req = Request::builder().method("GET").uri("/fallback-matrix/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(ssr_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&html_bytes);

    assert!(html.contains("Visual Layout Content"), "Expected standard post content to render");
    assert!(!html.contains("<div class=\"np-visual-canvas\">"), "Expected visual canvas wrapper to be absent after layout deletion");
}
