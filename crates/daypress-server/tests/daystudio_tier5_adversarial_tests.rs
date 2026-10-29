//! Milestone 4 Tier 5 Adversarial Hardening & White-Box Stress Test Suite.
//!
//! 1. Malformed & deeply nested AST payloads and buffer extremes.
//! 2. SQL injection & path traversal attempts across template conditions & REST API.
//! 3. High-concurrency template lifecycle operations and cascading condition cleanup.
//! 4. Multi-byte UTF-8 extremes (astral emojis, ZWJs, RTL Hebrew/Arabic, Cyrillic, CJK).
//! 5. Comprehensive editor chrome leak penetration testing across 35+ variants.

#![allow(clippy::field_reassign_with_default)]
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use daypress_ast::{
    render_layout_to_html, sanitize_ssr_html, DaypressLayout, DaypressNode, NodeKind,
    StyleProperties,
};
use daypress_server::{
    create_app,
    models::WpSettingsResponse,
    templates::public::render_base_layout,
    theme_builder::{
        conditions::{evaluate_condition, ConditionMatch},
        models::{
            ConditionType, CreateConditionDto, CreateTemplateRequest, RequestContext, RuleTarget,
            TemplateType, UpdateTemplateRequest,
        },
        repository::ThemeTemplateRepository,
    },
    Database,
};
use serde_json::json;
use std::sync::{Arc, Barrier};
use std::thread;
use tower::ServiceExt;

fn auth_req(method: &str, uri: &str, body_val: Option<serde_json::Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", "Basic YWRtaW46YWRtaW4=");
    match body_val {
        Some(val) => builder.body(Body::from(val.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

// ---------------------------------------------------------------------------
// 1. Malformed & Deeply Nested AST Payloads
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_tier5_malformed_and_deep_ast_payloads() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db.clone());

    // A. Deeply nested AST tree (depth 35)
    let mut current = json!({
        "id": "leaf_node", "type": "Heading",
        "data": { "text": "Deep Leaf", "level": 2 }, "styles": {}
    });
    for i in (0..35).rev() {
        current = json!({
            "id": format!("nest_{}", i), "type": "Container",
            "styles": { "padding_top": "10px" }, "children": [current]
        });
    }
    let deep_ast = json!({ "layout": { "version": 1, "root": current } });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/layout/1",
            Some(deep_ast),
        ))
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Deeply nested AST must save without stack overflow"
    );

    // B. Huge string buffer payload (128 KB text in data)
    let huge_text = "A".repeat(128 * 1024);
    let huge_ast = json!({
        "layout": { "version": 1, "root": {
            "id": "huge_sec", "type": "Section", "styles": {}, "children": [{
                "id": "huge_h", "type": "Heading", "data": { "text": huge_text, "level": 1 }, "styles": {}
            }]
        }}
    });
    let res_huge = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/layout/1",
            Some(huge_ast),
        ))
        .await
        .unwrap();
    assert_eq!(
        res_huge.status(),
        StatusCode::OK,
        "128KB string payload must save successfully"
    );

    // C. Malformed primitive layout payload (invalid JSON schema)
    let bad_payload = json!({ "layout": "not-an-object" });
    let res_bad = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/layout/1",
            Some(bad_payload),
        ))
        .await
        .unwrap();
    assert!(
        res_bad.status().is_success() || res_bad.status().is_client_error(),
        "Malformed layout must not crash server"
    );
}

// ---------------------------------------------------------------------------
// 2. SQL Injection & Path Traversal Firewall
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_tier5_sql_injection_and_path_traversal_firewall() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db.clone());
    let repo = ThemeTemplateRepository::from_database(&db);

    let malicious_target_ids = [
        "' OR '1'='1",
        "'; DROP TABLE daypress_templates; --",
        "1' UNION SELECT 1,2,3,4,5,6,7,8,9,10,11--",
        "../../../../../../etc/passwd",
        "..\\..\\..\\Windows\\System32\\cmd.exe",
    ];

    for (idx, injection) in malicious_target_ids.iter().enumerate() {
        let create_req = json!({
            "title": format!("SQLi Test Header {}", idx),
            "template_type": "header", "priority": 15,
            "conditions": [{
                "condition_type": "include", "rule_target": "post",
                "target_id": injection, "priority": 15
            }]
        });

        let res = app
            .clone()
            .oneshot(auth_req(
                "POST",
                "/wp-json/daypress/v1/templates",
                Some(create_req),
            ))
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::CREATED,
            "Parameterized binding must accept literal strings"
        );

        // Verify SQLite table was not dropped
        let count_res: rusqlite::Result<i64> = repo.conn().lock().unwrap().query_row(
            "SELECT COUNT(*) FROM daypress_templates",
            [],
            |r| r.get(0),
        );
        assert!(
            count_res.is_ok(),
            "daypress_templates must remain intact after injection attempt"
        );

        // Evaluate condition: invalid target_id cannot match normal numeric post ID
        let req_ctx = RequestContext::for_post(42, "post", false, vec![]);
        let conds = repo.list(&Default::default()).unwrap();
        let matched = evaluate_condition(&conds[0].conditions[0], &req_ctx);
        assert_eq!(
            matched,
            ConditionMatch::NoMatch,
            "SQLi target_id must evaluate to NoMatch"
        );
    }

    // Path traversal in starter template route
    let trav_res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/templates/starter",
            Some(json!({ "starter_id": "../../../../Cargo.toml" })),
        ))
        .await
        .unwrap();
    assert_eq!(
        trav_res.status(),
        StatusCode::NOT_FOUND,
        "Path traversal in starter_id must return 404"
    );

    // Template list query injection
    let list_res = app
        .clone()
        .oneshot(auth_req(
            "GET",
            "/wp-json/daypress/v1/templates?template_type=%27%20OR%201=1--",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        list_res.status(),
        StatusCode::OK,
        "Query injection must be safely bound and return 200"
    );
}

// ---------------------------------------------------------------------------
// 3. High-Concurrency Template Lifecycle & Cascading Cleanup
// ---------------------------------------------------------------------------
#[test]
fn test_tier5_high_concurrency_template_lifecycle_and_cascade() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let repo = Arc::new(ThemeTemplateRepository::from_database(&db));
    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for t_idx in 0..num_threads {
        let repo_clone = Arc::clone(&repo);
        let barrier_clone = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier_clone.wait();
            for i in 0..10 {
                let created = repo_clone
                    .create(&CreateTemplateRequest {
                        title: format!("Concurrent Tpl t{}_{}", t_idx, i),
                        template_type: TemplateType::Header,
                        status: Some("publish".to_string()),
                        layout: None,
                        layout_json: None,
                        settings: None,
                        priority: Some(10 + t_idx as i32),
                        is_active: Some(true),
                        conditions: Some(vec![
                            CreateConditionDto {
                                id: None,
                                condition_type: ConditionType::Include,
                                rule_target: RuleTarget::EntireSite,
                                target_id: String::new(),
                                priority: 10,
                            },
                            CreateConditionDto {
                                id: None,
                                condition_type: ConditionType::Exclude,
                                rule_target: RuleTarget::Post,
                                target_id: format!("{}", 100 + i),
                                priority: 20,
                            },
                        ]),
                    })
                    .expect("Concurrent create failed");

                let conds = repo_clone
                    .get_conditions(created.id)
                    .expect("Get conditions failed");
                assert_eq!(conds.len(), 2);

                let _ = repo_clone.update(
                    created.id,
                    &UpdateTemplateRequest {
                        priority: Some(30),
                        ..Default::default()
                    },
                );

                if i % 2 == 0 {
                    let deleted = repo_clone
                        .delete(created.id, true)
                        .expect("Force delete failed");
                    assert!(deleted);
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("Worker thread panicked");
    }

    // Direct SQLite foreign key audit: verify 0 orphaned conditions remain
    let conn = repo.conn().lock().unwrap();
    let orphan_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM daypress_template_conditions c 
         LEFT JOIN daypress_templates t ON c.template_id = t.id 
         WHERE t.id IS NULL",
            [],
            |r| r.get(0),
        )
        .expect("Orphan query failed");

    assert_eq!(
        orphan_count, 0,
        "All deleted template conditions must be cascadingly cleaned up"
    );
}

// ---------------------------------------------------------------------------
// 4. Multi-Byte UTF-8 Extremes: Astral Emojis, RTL, Cyrillic, CJK
// ---------------------------------------------------------------------------
#[test]
fn test_tier5_multibyte_utf8_extremes_astral_emojis_rtl_and_cjk() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let repo = ThemeTemplateRepository::from_database(&db);

    let extreme_title = "🚀✨ 👨‍👩‍👧‍👦 עברית: שלום עולם | العربية: مرحبا | Тест темы | 日本語・中文";
    let extreme_content = "Heading: 🦹‍♂️ Anti-Hero \u{200D} & Features • שבת שלום • اهلا وسهلا";

    let mut layout = DaypressLayout::default();
    layout.root = DaypressNode {
        id: "utf8_root".to_string(),
        kind: NodeKind::Heading {
            text: extreme_content.to_string(),
            level: 1,
        },
        styles: StyleProperties::default(),
        children: vec![],
    };

    let created = repo
        .create(&CreateTemplateRequest {
            title: extreme_title.to_string(),
            template_type: TemplateType::Header,
            status: Some("publish".to_string()),
            layout: Some(layout.clone()),
            layout_json: None,
            settings: None,
            priority: Some(25),
            is_active: Some(true),
            conditions: Some(vec![CreateConditionDto {
                id: None,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::EntireSite,
                target_id: String::new(),
                priority: 10,
            }]),
        })
        .expect("UTF-8 template creation failed");

    // Verify round-trip retrieval from SQLite
    let fetched = repo
        .get_by_id(created.id)
        .unwrap()
        .expect("Template not found");
    assert_eq!(
        fetched.title, extreme_title,
        "Extreme UTF-8 title must match verbatim"
    );

    // Verify public SSR shell does not panic and preserves multi-byte strings
    let rendered_node = render_layout_to_html(&layout);
    assert!(rendered_node.contains("👨‍👩‍👧‍👦") || rendered_node.contains("🦹‍♂️"));

    let settings = WpSettingsResponse {
        title: extreme_title.to_string(),
        description: "RTL & Astral Emoji CMS".to_string(),
        ..Default::default()
    };
    let public_doc = render_base_layout(extreme_title, &settings, &rendered_node, None, None, None);

    assert!(
        public_doc.contains("עברית: שלום עולם"),
        "Public SSR must contain Hebrew text"
    );
    assert!(
        public_doc.contains("العربية: مرحبا"),
        "Public SSR must contain Arabic text"
    );
    assert!(
        public_doc.contains("Тест темы"),
        "Public SSR must contain Cyrillic text"
    );
    assert!(
        public_doc.contains("日本語・中文"),
        "Public SSR must contain CJK text"
    );
}

// ---------------------------------------------------------------------------
// 5. Comprehensive Editor Chrome Leak Penetration Testing (35+ Variants)
// ---------------------------------------------------------------------------
#[test]
fn test_tier5_editor_chrome_leak_penetration_35_variants() {
    let contaminated_html = r#"
        <div class="site-wrapper preserved-shell-class elementor-grid-handle daystudio-grid-handle ui-resizable">
            <header class="main-header daystudio-handle daystudio-selection-outline active-element"
                    contenteditable="true" contenteditable="false" contenteditable="" contenteditable
                    contenteditable='true' CONTENTEDITABLE="TRUE" ContentEditable="plaintext-only" contenteditable=true
                    draggable="true" draggable="false" DRAGGABLE="TRUE" draggable
                    data-daystudio="section" data-daystudio-node-id="sec_1" data-daystudio-type="PriceCard"
                    data-daystudio-active="true" data-daystudio-selected="true" data-daystudio-locked="true"
                    data-daystudio-drop-target="true" data-daystudio-breakpoint="mobile"
                    data-elementor-type="header" data-elementor-id="123" data-elementor-post-type="page"
                    data-elementor-settings="{}" data-drag-handle="true" data-drag-handle data-model-cid="c42">
                <h1 class="dp-title daystudio-node daystudio-element ui-draggable">Clean Public Title</h1>
                <p class="dp-desc daystudio-drop-indicator daystudio-hovered daystudio-box-guide ui-sortable">
                    Valid Body Text
                </p>
                <div class="pricing-card et_pb_element_editing visual-builder-active elementor-element-edit-mode elementor-editor-active">
                    <span class="price-value">$99</span>
                </div>
            </header>
        </div>
    "#;

    let sanitized = sanitize_ssr_html(contaminated_html);

    // 1. Assert 100% elimination of all 35+ editor chrome variants
    let forbidden_substrings = [
        "contenteditable",
        "CONTENTEDITABLE",
        "ContentEditable",
        "draggable",
        "DRAGGABLE",
        "data-daystudio",
        "data-elementor",
        "data-drag-handle",
        "data-model-cid",
        "elementor-grid-handle",
        "daystudio-grid-handle",
        "daystudio-handle",
        "daystudio-selection-outline",
        "daystudio-node",
        "daystudio-element",
        "daystudio-drop-indicator",
        "daystudio-hovered",
        "daystudio-box-guide",
        "ui-resizable",
        "ui-draggable",
        "ui-sortable",
        "active-element",
        "et_pb_element_editing",
        "visual-builder-active",
        "elementor-element-edit-mode",
        "elementor-editor-active",
        "class=\"\"",
        "class=\"  \"",
    ];

    for forbidden in forbidden_substrings {
        assert!(
            !sanitized.contains(forbidden),
            "SSR leaked forbidden editor chrome: '{}'",
            forbidden
        );
    }

    // 2. Assert preservation of legitimate classes and structure
    assert!(sanitized.contains("site-wrapper preserved-shell-class"));
    assert!(sanitized.contains("main-header"));
    assert!(sanitized.contains("dp-title"));
    assert!(sanitized.contains("dp-desc"));
    assert!(sanitized.contains("pricing-card"));
    assert!(sanitized.contains("Clean Public Title"));
    assert!(sanitized.contains("Valid Body Text"));
    assert!(sanitized.contains("$99"));
}
