use super::*;
use crate::adapters::mcp::{handle_mcp_rpc, JsonRpcRequest};
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_v04_hot_cache_hit_miss_etag_and_invalidation() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. First anonymous GET / -> MISS
    let res1 = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    assert_eq!(
        res1.headers()
            .get("x-daypress-cache")
            .and_then(|v| v.to_str().ok()),
        Some("MISS")
    );
    let etag = res1
        .headers()
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .expect("ETag header should be present")
        .to_string();

    // 2. Second anonymous GET / with Accept-Encoding: gzip -> HIT + gzip
    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .header(header::ACCEPT_ENCODING, "gzip, deflate")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::OK);
    assert_eq!(
        res2.headers()
            .get("x-daypress-cache")
            .and_then(|v| v.to_str().ok()),
        Some("HIT")
    );
    assert_eq!(
        res2.headers()
            .get(header::CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok()),
        Some("gzip")
    );

    // 3. Conditional GET / with If-None-Match -> 304 Not Modified
    let res3 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .header(header::IF_NONE_MATCH, &etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res3.status(), StatusCode::NOT_MODIFIED);

    // 4. Updating an option invalidates hot cache -> next GET / is MISS
    db.set_option("blogdescription", "Updated Tagline").unwrap();
    let res4 = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        res4.headers()
            .get("x-daypress-cache")
            .and_then(|v| v.to_str().ok()),
        Some("MISS")
    );
}

#[tokio::test]
async fn test_v04_formstudio_vault_antispam_rate_limit_and_analytics() {
    let db = Database::new_in_memory().expect("in-memory db");
    let form_id = db
        .save_form_record(&CreateFormRequest {
            id: None,
            title: "Quote Calculator Form".to_string(),
            fields: vec![
                FormFieldSchema {
                    name: "email".to_string(),
                    label: "Work Email".to_string(),
                    field_type: "email".to_string(),
                    required: true,
                    width: "half".to_string(),
                    step: 1,
                    ..Default::default()
                },
                FormFieldSchema {
                    name: "seats".to_string(),
                    label: "Seats".to_string(),
                    field_type: "number".to_string(),
                    step: 1,
                    ..Default::default()
                },
                FormFieldSchema {
                    name: "estimate".to_string(),
                    label: "Estimated Price".to_string(),
                    field_type: "calculated".to_string(),
                    calculation_formula: Some("{seats} * 29".to_string()),
                    step: 2,
                    ..Default::default()
                },
            ],
            settings: FormSettings {
                step_titles: vec!["Contact".to_string(), "Estimate".to_string()],
                min_submit_seconds: 3,
                rate_limit_per_minute: 3,
                ..Default::default()
            },
        })
        .expect("save form");

    // Verify shortcode SSR renders multi-step wizard and calculated formula
    let settings = db.get_settings().unwrap_or_default();
    let rendered_html = crate::plugins::expand_shortcodes_in_content(
        &format!("<p>[daypress_form id=\"{form_id}\"]</p>"),
        &db,
        &settings,
    );
    assert!(rendered_html.contains("Step 1 of 2: Contact"));
    assert!(rendered_html.contains("data-calc-formula=\"{seats} * 29\""));

    let app = create_app(db.clone());

    // 1. Valid submission -> saved as 'unread' + conversion recorded in analytics
    let valid_body = json!({
        "data": {
            "email": "founder@acme.dev",
            "seats": "10",
            "estimate": "290",
            "_dp_ts": 1000
        }
    });
    let res_ok = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-forwarded-for", "203.0.113.10")
                .body(Body::from(valid_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_ok.status(), StatusCode::OK);

    // 2. Honeypot hit -> quarantined as 'spam' in SQLite Vault
    let spam_body = json!({
        "data": {
            "email": "bot@spam.example",
            "_np_hp": "http://spam-link.example"
        }
    });
    let res_spam = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-forwarded-for", "203.0.113.10")
                .body(Body::from(spam_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_spam.status(), StatusCode::BAD_REQUEST);

    // 3. Timestamp trap hit (submitted 0ms after render when min_submit_seconds = 3) -> quarantined as 'spam'
    let now_ms = chrono::Utc::now().timestamp_millis();
    let fast_body = json!({
        "data": {
            "email": "fastbot@spam.example",
            "_dp_ts": now_ms
        }
    });
    let res_fast = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-forwarded-for", "203.0.113.10")
                .body(Body::from(fast_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_fast.status(), StatusCode::BAD_REQUEST);

    // 4. 4th submission from same IP within 60s exceeds rate_limit_per_minute=3 -> 429 Too Many Requests
    let res_rate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-forwarded-for", "203.0.113.10")
                .body(Body::from(valid_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_rate.status(), StatusCode::TOO_MANY_REQUESTS);

    // Verify Vault entries: 1 unread, 2 spam
    let unread = db
        .list_form_entries_filtered(Some(form_id), Some("unread"), None, 50)
        .unwrap();
    let spam = db
        .list_form_entries_filtered(Some(form_id), Some("spam"), None, 50)
        .unwrap();
    assert_eq!(unread.len(), 1);
    assert_eq!(spam.len(), 2);

    // Verify public /uploads/form-submissions/* is blocked with 403 Forbidden
    let res_upload_block = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/uploads/form-submissions/1/secret.pdf")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_upload_block.status(), StatusCode::FORBIDDEN);

    // Verify Cookieless Analytics dashboard recorded the form conversion + pageview with UTM
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/analytics/event")
                .header(header::CONTENT_TYPE, "application/json")
                .header("cf-ipcountry", "NL")
                .body(Body::from(
                    json!({
                        "page_url": "/pricing",
                        "referrer": "https://news.ycombinator.com/",
                        "device_type": "desktop",
                        "utm_source": "hn",
                        "utm_medium": "social",
                        "utm_campaign": "v04_launch"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let analytics = db.get_analytics_dashboard("30d").unwrap();
    assert!(analytics.total_pageviews >= 1);
    assert!(analytics.total_conversions >= 1);
    assert!(analytics.utm_sources.iter().any(|u| u.label == "hn"));
    assert!(analytics.countries.iter().any(|c| c.label == "NL"));
}

#[tokio::test]
async fn test_v04_privacy_cookie_banner_and_mcp_tools() {
    let db = Database::new_in_memory().expect("in-memory db");
    let mut priv_settings = db.get_privacy_settings().unwrap();
    priv_settings.cookie_banner_enabled = true;
    priv_settings.cookie_banner_position = "floating-right".to_string();
    priv_settings.cookie_banner_title = "GDPR Consent Test".to_string();
    db.update_privacy_settings(&priv_settings).unwrap();

    let app = create_app(db.clone());
    let res = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body_bytes);
    assert!(html.contains("id=\"dp-cookie-banner\""));
    assert!(html.contains("dp-cc-floating-right"));
    assert!(html.contains("GDPR Consent Test"));
    assert!(html.contains("Cookie Preferences"));

    // Verify MCP tools for FormStudio & Analytics
    let service = CmsService::from_adapter(db);
    let create_resp = handle_mcp_rpc(
        &service,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(1)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_create_form",
                "arguments": {
                    "title": "MCP Created Lead Form",
                    "fields": [
                        { "name": "email", "label": "Email", "field_type": "email", "required": true }
                    ]
                }
            }),
        },
    );
    let res_val = create_resp.result.unwrap();
    assert_eq!(res_val["isError"], false);
    let created_payload: Value =
        serde_json::from_str(res_val["content"][0]["text"].as_str().unwrap()).unwrap();
    let form_id = created_payload["id"].as_i64().unwrap();
    assert!(form_id > 0);

    let analytics_resp = handle_mcp_rpc(
        &service,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_get_analytics",
                "arguments": { "range": "30d" }
            }),
        },
    );
    assert_eq!(analytics_resp.result.unwrap()["isError"], false);
}

#[tokio::test]
async fn test_v05_dynamic_templates_symbols_popups_and_media_dam() {
    use crate::theme_builder::models::{
        ConditionType, CreateConditionDto, CreateTemplateRequest, RuleTarget, TemplateSettings,
        TemplateType,
    };
    use crate::theme_builder::repository::ThemeTemplateRepository;
    use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties, SymbolOverride};
    use std::collections::HashMap;

    let db = Database::new_in_memory().expect("in-memory db");
    let repo = ThemeTemplateRepository::from_database(&db);

    // 1. Create a SectionSymbol master template
    let symbol_layout = DaypressLayout {
        version: 1,
        global_styles: None,
        root: DaypressNode {
            id: "root".into(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![DaypressNode {
                id: "sym_cta_heading".into(),
                kind: NodeKind::Heading {
                    text: "Master Symbol Headline".into(),
                    level: 2,
                },
                styles: StyleProperties::default(),
                children: vec![],
            }],
        },
    };
    let sym_record = repo
        .create(&CreateTemplateRequest {
            title: "Global Promo Symbol".into(),
            template_type: TemplateType::SectionSymbol,
            layout: Some(symbol_layout),
            layout_json: None,
            status: Some("publish".into()),
            priority: Some(10),
            is_active: Some(true),
            settings: None,
            conditions: None,
        })
        .expect("create symbol");

    // 2. Create a SinglePost template that embeds {{post.title}} AND an overridden SymbolInstance
    let mut overrides = HashMap::new();
    overrides.insert(
        "sym_cta_heading".to_string(),
        SymbolOverride {
            text: Some("Instance Overridden CTA".to_string()),
            ..Default::default()
        },
    );
    let single_layout = DaypressLayout {
        version: 1,
        global_styles: None,
        root: DaypressNode {
            id: "root".into(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![
                DaypressNode {
                    id: "dyn_title".into(),
                    kind: NodeKind::Heading {
                        text: "Article: {{post.title}} by {{author.name}}".into(),
                        level: 1,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "sym_inst".into(),
                    kind: NodeKind::SymbolInstance {
                        symbol_id: sym_record.id,
                        overrides,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
            ],
        },
    };
    repo.create(&CreateTemplateRequest {
        title: "Custom Single Post Layout".into(),
        template_type: TemplateType::SinglePost,
        layout: Some(single_layout),
        layout_json: None,
        status: Some("publish".into()),
        priority: Some(20),
        is_active: Some(true),
        settings: None,
        conditions: Some(vec![CreateConditionDto {
            id: None,
            condition_type: ConditionType::Include,
            rule_target: RuleTarget::AllPosts,
            target_id: String::new(),
            priority: 20,
        }]),
    })
    .expect("create single_post template");

    // 3. Create a Top Announcement Bar Popup template
    let popup_settings = TemplateSettings {
        popup_variant: Some("announcement_bar_top".to_string()),
        dismiss_days: Some(7),
        ..Default::default()
    };
    let popup_layout = DaypressLayout {
        version: 1,
        global_styles: None,
        root: DaypressNode {
            id: "root".into(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![DaypressNode {
                id: "ann_txt".into(),
                kind: NodeKind::Text {
                    content_html: "Daypress 0.5 Shipped Announcement!".into(),
                },
                styles: StyleProperties::default(),
                children: vec![],
            }],
        },
    };
    repo.create(&CreateTemplateRequest {
        title: "Sitewide Announcement".into(),
        template_type: TemplateType::Popup,
        layout: Some(popup_layout),
        layout_json: None,
        status: Some("publish".into()),
        priority: Some(10),
        is_active: Some(true),
        settings: Some(popup_settings),
        conditions: None,
    })
    .expect("create popup");

    // 4. Request a single post (/?p=1 is the seeded "Hello world!" post)
    let app = create_app(db.clone());
    let res = app
        .oneshot(Request::builder().uri("/?p=1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body_bytes);
    assert!(
        html.contains("Article: Hello world!"),
        "Expected dynamic {{post.title}} binding in SSR output"
    );
    assert!(
        html.contains("Instance Overridden CTA"),
        "Expected SymbolInstance child override in SSR output"
    );
    assert!(
        html.contains("Daypress 0.5 Shipped Announcement!"),
        "Expected Announcement Bar popup in SSR output"
    );

    // 5. Verify Media Library DAM Virtual Folders & Unused Asset Scanner
    let folder = db
        .create_media_folder("Brand Icons", 0)
        .expect("create media folder");
    assert_eq!(folder.name, "Brand Icons");
    let folders = db.list_media_folders().expect("list folders");
    assert_eq!(folders.len(), 1);

    let att = db
        .create_attachment(
            "Unused Icon",
            "unused-test-icon.svg",
            "image/svg+xml",
            128,
            "http://localhost:8090/uploads/unused-test-icon.svg",
        )
        .expect("create attachment");
    let unused = db.scan_unused_media_attachments().expect("scan unused");
    assert!(
        unused.iter().any(|m| m.id == att.id),
        "Unreferenced attachment must be detected by Unused Asset Scanner"
    );
}
