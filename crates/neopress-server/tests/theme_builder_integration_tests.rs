//! In-Memory Axum Integration Tests for Theme Builder Subsystem & SSR Pipeline.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use neopress_ast::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};
use neopress_server::theme_builder::models::*;
use neopress_server::theme_builder::ssr_injector::{HydrationContext, SsrInjector};
use neopress_server::theme_builder::ThemeTemplateRepository;
use neopress_server::{create_app, Database};
use std::collections::HashMap;
use tower::ServiceExt;

#[path = "theme_builder/adversarial.rs"]
mod adversarial;

fn auth_header() -> String {
    format!("Basic {}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "admin:admin"))
}

fn auth_post(uri: &str, val: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("Authorization", auth_header())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&val).unwrap()))
        .unwrap()
}

fn auth_put(uri: &str, val: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header("Authorization", auth_header())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&val).unwrap()))
        .unwrap()
}

fn cond(t: ConditionType, r: RuleTarget, id: &str, p: i32) -> TemplateCondition {
    TemplateCondition {
        id: None,
        template_id: 1,
        condition_type: t,
        rule_target: r,
        target_id: id.to_string(),
        priority: p,
        created_at: None,
    }
}

#[tokio::test]
async fn test_display_conditions_specificity_hierarchy_and_exclusion() {
    let entire_site = cond(ConditionType::Include, RuleTarget::EntireSite, "", 10);
    let post_type = cond(ConditionType::Include, RuleTarget::AllPosts, "", 20);
    let specific_post = cond(ConditionType::Include, RuleTarget::Post, "42", 40);
    let exclude_post = cond(ConditionType::Exclude, RuleTarget::Post, "42", 10);

    let ctx = RequestContext::for_post(42, "post", false, vec![]);

    let m1 = neopress_server::theme_builder::evaluate_condition(&entire_site, &ctx);
    let m2 = neopress_server::theme_builder::evaluate_condition(&post_type, &ctx);
    let m3 = neopress_server::theme_builder::evaluate_condition(&specific_post, &ctx);
    let m4 = neopress_server::theme_builder::evaluate_condition(&exclude_post, &ctx);

    assert_eq!(m1.specificity(), Some(10));
    assert_eq!(m2.specificity(), Some(20));
    assert_eq!(m3.specificity(), Some(40));
    assert_eq!(m4.specificity(), Some(40));
    assert!(m3.specificity().unwrap() > m2.specificity().unwrap());
    assert!(m2.specificity().unwrap() > m1.specificity().unwrap());

    // Verify strict exclusion override in template evaluation
    let tpl = ThemeTemplateRecord {
        id: 100,
        title: "Test".to_string(),
        template_type: TemplateType::Header,
        status: "publish".to_string(),
        layout: NeopressLayout::default(),
        rendered_html: None,
        settings: TemplateSettings::default(),
        priority: 10,
        is_active: true,
        conditions: vec![specific_post, exclude_post],
        created_at: String::new(),
        updated_at: String::new(),
    };
    assert!(neopress_server::theme_builder::evaluate_template(&tpl, &ctx).is_none());
}

#[tokio::test]
async fn test_theme_template_crud_persistence_and_cascading_conditions() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db.clone());

    // 1. Create Header Template via REST API
    let create_payload = serde_json::json!({
        "title": "Main Site Header",
        "template_type": "header",
        "status": "publish",
        "priority": 15,
        "is_active": true,
        "conditions": [{"condition_type": "include", "rule_target": "entire_site", "priority": 10}]
    });

    let resp = app.clone().oneshot(auth_post("/wp-json/neopress/v1/templates", create_payload)).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let created: ThemeTemplateRecord = serde_json::from_slice(&body).unwrap();
    assert_eq!(created.title, "Main Site Header");
    assert_eq!(created.conditions.len(), 1);

    // 2. Fetch via GET /wp-json/neopress/v1/templates/{id}
    let get_resp = app.clone().oneshot(
        Request::builder().method("GET").uri(format!("/wp-json/neopress/v1/templates/{}", created.id)).body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);

    // 3. Update via PUT /wp-json/neopress/v1/templates/{id}
    let update_resp = app.clone().oneshot(
        auth_put(&format!("/wp-json/neopress/v1/templates/{}", created.id), serde_json::json!({"title": "Updated Site Header", "priority": 30}))
    ).await.unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);
    let upd_body = to_bytes(update_resp.into_body(), usize::MAX).await.unwrap();
    let updated: ThemeTemplateRecord = serde_json::from_slice(&upd_body).unwrap();
    assert_eq!(updated.title, "Updated Site Header");
    assert_eq!(updated.priority, 30);

    // 4. Duplicate Template via Repository
    let repo = ThemeTemplateRepository::from_database(&db);
    let dup = repo.duplicate(created.id).unwrap().expect("Duplication failed");
    assert_eq!(dup.title, "Updated Site Header (Copy)");
    assert_eq!(dup.conditions.len(), 1);

    // 5. Delete Template with force=true and verify cascade
    let del_resp = app.clone().oneshot(
        Request::builder().method("DELETE").uri(format!("/wp-json/neopress/v1/templates/{}?force=true", created.id))
            .header("Authorization", auth_header()).body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(del_resp.status(), StatusCode::OK);
    assert!(repo.get_by_id(created.id).unwrap().is_none());
    assert!(repo.get_conditions(created.id).unwrap().is_empty());
}

#[tokio::test]
async fn test_dynamic_placeholder_ast_hydration_and_tokens() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let mut layout = NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "root-1".to_string(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![
                NeopressNode {
                    id: "dynamic-post-title".to_string(),
                    kind: NodeKind::Heading { level: 1, text: "Default".to_string() },
                    styles: StyleProperties::default(),
                    children: Vec::new(),
                },
                NeopressNode {
                    id: "dynamic-author-box".to_string(),
                    kind: NodeKind::Text { content_html: "Written by {{author_name}} on {{post_date}}".to_string() },
                    styles: StyleProperties::default(),
                    children: Vec::new(),
                },
                NeopressNode {
                    id: "dynamic-token-text".to_string(),
                    kind: NodeKind::Text { content_html: "Site: {{site_title}} | Post: {{post_title}}".to_string() },
                    styles: StyleProperties::default(),
                    children: Vec::new(),
                },
            ],
        },
        global_styles: None,
    };

    let mut post = db.get_post_by_id(1).unwrap().expect("Post id 1 missing in seeded DB");
    post.title.rendered = "Awesome Rust Article".to_string();
    post.author_name = "Ferris Developer".to_string();

    let mut settings = db.get_settings().unwrap_or_default();
    settings.title = "Neopress Native".to_string();

    let hyd_ctx = HydrationContext {
        post: Some(&post),
        settings: &settings,
        breadcrumbs: vec![
            BreadcrumbItem { title: "Home".to_string(), url: "/".to_string(), is_current: false },
            BreadcrumbItem { title: "Awesome Rust Article".to_string(), url: "/dynamic-test-post/".to_string(), is_current: true },
        ],
        nav_menus: HashMap::new(),
    };

    SsrInjector::hydrate_layout(&mut layout, &hyd_ctx);
    let rendered_html = neopress_ast::render_layout_to_html(&layout);

    assert!(rendered_html.contains("Awesome Rust Article"));
    assert!(rendered_html.contains("Ferris Developer"));
    assert!(rendered_html.contains("Neopress Native"));
}

#[tokio::test]
async fn test_ssr_dynamic_header_footer_injection_and_leak_sanitizer() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let repo = ThemeTemplateRepository::from_database(&db);

    let mut header_layout = NeopressLayout::default();
    header_layout.root.children.push(NeopressNode {
        id: "custom-header-bar".to_string(),
        kind: NodeKind::Text {
            content_html: r#"<div contenteditable="true" draggable="true" class="elementor-grid-handle">Custom Theme Header Brand</div>"#.to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    });

    let _hdr = repo.create(&CreateTemplateRequest {
        title: "Global Theme Header".to_string(),
        template_type: TemplateType::Header,
        status: Some("publish".to_string()),
        layout: Some(header_layout),
        layout_json: None,
        settings: Some(TemplateSettings { sticky: true, ..Default::default() }),
        priority: Some(10),
        is_active: Some(true),
        conditions: Some(vec![CreateConditionDto {
            id: None, condition_type: ConditionType::Include, rule_target: RuleTarget::EntireSite, target_id: String::new(), priority: 10,
        }]),
    }).unwrap();

    let mut footer_layout = NeopressLayout::default();
    footer_layout.root.children.push(NeopressNode {
        id: "custom-footer-bar".to_string(),
        kind: NodeKind::Text { content_html: "<span>Custom Theme Footer Copyright 2026</span>".to_string() },
        styles: StyleProperties::default(),
        children: Vec::new(),
    });

    let _ftr = repo.create(&CreateTemplateRequest {
        title: "Global Theme Footer".to_string(),
        template_type: TemplateType::Footer,
        status: Some("publish".to_string()),
        layout: Some(footer_layout),
        layout_json: None,
        settings: None,
        priority: Some(10),
        is_active: Some(true),
        conditions: Some(vec![CreateConditionDto {
            id: None, condition_type: ConditionType::Include, rule_target: RuleTarget::EntireSite, target_id: String::new(), priority: 10,
        }]),
    }).unwrap();

    let app = create_app(db.clone());
    let resp = app.clone().oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert!(html.contains("np-theme-header"));
    assert!(html.contains("np-theme-footer"));
    assert!(html.contains("Custom Theme Header Brand"));
    assert!(html.contains("Custom Theme Footer Copyright 2026"));
    assert!(!html.contains("contenteditable"));
    assert!(!html.contains("draggable="));
    assert!(!html.contains("elementor-grid-handle"));
}

#[tokio::test]
async fn test_starter_templates_catalog_and_install() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db.clone());

    let cat_resp = app.clone().oneshot(
        Request::builder().method("GET").uri("/wp-json/neopress/v1/templates/starter").body(Body::empty()).unwrap(),
    ).await.unwrap();

    assert_eq!(cat_resp.status(), StatusCode::OK);
    let cat_body = to_bytes(cat_resp.into_body(), usize::MAX).await.unwrap();
    let catalog: Vec<StarterTemplateInfo> = serde_json::from_slice(&cat_body).unwrap();
    assert_eq!(catalog.len(), 4);
    assert!(catalog.iter().any(|c| c.id == "dark-grid-footer"));

    let inst_resp = app.clone().oneshot(auth_post("/wp-json/neopress/v1/templates/starter", serde_json::json!({"starter_id": "dark-grid-footer"}))).await.unwrap();
    assert_eq!(inst_resp.status(), StatusCode::CREATED);
    let inst_body = to_bytes(inst_resp.into_body(), usize::MAX).await.unwrap();
    let installed: ThemeTemplateRecord = serde_json::from_slice(&inst_body).unwrap();
    assert_eq!(installed.title, "Modern Dark Grid Footer");
    assert!(installed.is_active);

    let repo = ThemeTemplateRepository::from_database(&db);
    let fetched = repo.get_by_id(installed.id).unwrap().expect("Starter template missing in DB");
    assert_eq!(fetched.title, "Modern Dark Grid Footer");
}

#[tokio::test]
async fn test_rest_api_active_resolution() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let repo = ThemeTemplateRepository::from_database(&db);

    let _hdr = repo.create(&CreateTemplateRequest {
        title: "Posts Specific Header".to_string(),
        template_type: TemplateType::Header,
        status: Some("publish".to_string()),
        layout: Some(NeopressLayout::default()),
        layout_json: None,
        settings: None,
        priority: Some(20),
        is_active: Some(true),
        conditions: Some(vec![CreateConditionDto {
            id: None, condition_type: ConditionType::Include, rule_target: RuleTarget::AllPosts, target_id: String::new(), priority: 20,
        }]),
    }).unwrap();

    let app = create_app(db.clone());
    let resp = app.clone().oneshot(
        Request::builder().method("GET").uri("/wp-json/neopress/v1/templates/active?post_type=post&post_id=5").body(Body::empty()).unwrap(),
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let resolved: ResolvedTemplatesResponse = serde_json::from_slice(&body).unwrap();
    assert!(resolved.header.is_some());
    assert_eq!(resolved.header.unwrap().title, "Posts Specific Header");
}

#[tokio::test]
async fn test_standalone_preview_endpoint() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let repo = ThemeTemplateRepository::from_database(&db);

    let tpl = repo.create(&CreateTemplateRequest {
        title: "Preview Test Template".to_string(),
        template_type: TemplateType::Header,
        status: Some("draft".to_string()),
        layout: Some(NeopressLayout::default()),
        layout_json: None,
        settings: None,
        priority: Some(10),
        is_active: Some(false),
        conditions: None,
    }).unwrap();

    let app = create_app(db.clone());
    let resp = app.clone().oneshot(
        Request::builder().method("GET").uri(format!("/wp-json/neopress/v1/templates/preview/{}", tpl.id)).body(Body::empty()).unwrap(),
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Theme Builder Live Preview"));
    assert!(html.contains("Preview Test Template"));
}
