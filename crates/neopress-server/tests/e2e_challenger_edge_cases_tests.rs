//! Milestone 4 Empirical Challenger Adversarial Stress Test Suite.
//!
//! Validates:
//! 1. Auth nonces enforcement: Missing, forged, and valid `wp_rest_admin` nonces.
//! 2. 404 Monitoring persistence: Concurrency, repeated hits (UPSERT), SQLi paths, Unicode paths.
//! 3. Theme specificity tie-breaking: Equal priority multi-category conflict, non-existent terms.
//! 4. High concurrency stress: Parallel REST mutations, layout saves, SSR reads, and 404s.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use neopress_ast::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};
use neopress_server::{create_app, Database};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

fn make_text_layout(id: &str, html: &str) -> NeopressLayout {
    NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: id.to_string(),
            kind: NodeKind::Text {
                content_html: html.to_string(),
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
        global_styles: None,
    }
}

async fn parse_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

// ---------------------------------------------------------------------------
// 1. Auth Nonces Enforcement & Bypass Resistance
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_challenger_auth_nonces_enforcement_and_bypass_resistance() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    let page_payload = json!({
        "title": "Confidential Strategy",
        "slug": "confidential-strategy",
        "content": "<p>Internal only</p>",
        "status": "publish"
    });

    // A. Request without X-WP-Nonce -> 401 Unauthorized
    let req_no_nonce = Request::builder()
        .method("POST")
        .uri("/wp-json/wp/v2/pages")
        .header("Content-Type", "application/json")
        .body(Body::from(page_payload.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req_no_nonce).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "Missing nonce must return 401");

    // B. Request with forged nonce -> 401 Unauthorized
    let req_forged = Request::builder()
        .method("POST")
        .uri("/wp-json/wp/v2/pages")
        .header("Content-Type", "application/json")
        .header("X-WP-Nonce", "forged_malicious_token_12345")
        .body(Body::from(page_payload.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req_forged).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "Forged nonce must return 401");

    // C. Request with valid nonce -> 201 Created
    let req_valid = Request::builder()
        .method("POST")
        .uri("/wp-json/wp/v2/pages")
        .header("Content-Type", "application/json")
        .header("X-WP-Nonce", "wp_rest_admin")
        .body(Body::from(page_payload.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req_valid).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED, "Valid nonce must allow creation");
    let page_id = parse_json(res).await["id"].as_i64().expect("page id");

    // D. Protected layout save without nonce -> 401 Unauthorized
    let layout_payload = json!({
        "layout": make_text_layout("l1", "<p>Secured Layout</p>")
    });
    let req_layout_no_auth = Request::builder()
        .method("POST")
        .uri(&format!("/wp-json/neopress/v1/layout/{}", page_id))
        .header("Content-Type", "application/json")
        .body(Body::from(layout_payload.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req_layout_no_auth).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "Layout save without nonce must return 401");

    // E. Protected layout save with valid nonce -> 200 OK
    let req_layout_auth = Request::builder()
        .method("POST")
        .uri(&format!("/wp-json/neopress/v1/layout/{}", page_id))
        .header("Content-Type", "application/json")
        .header("X-WP-Nonce", "wp_rest_admin")
        .body(Body::from(layout_payload.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req_layout_auth).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "Layout save with valid nonce must succeed");

    // F. Protected template creation without nonce -> 401 Unauthorized
    let tpl_payload = json!({
        "title": "Unauthorized Header",
        "template_type": "header",
        "priority": 10
    });
    let req_tpl_no_auth = Request::builder()
        .method("POST")
        .uri("/wp-json/neopress/v1/templates")
        .header("Content-Type", "application/json")
        .body(Body::from(tpl_payload.to_string()))
        .unwrap();
    let res = app.clone().oneshot(req_tpl_no_auth).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "Template creation without nonce must return 401");
}

// ---------------------------------------------------------------------------
// 2. 404 Logging Concurrency, Repeated Hits & Malicious Paths
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_challenger_404_logging_concurrency_and_extreme_paths() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    let target_path = "/quantum-missing-service/";

    // A. Five consecutive 404 hits on same path: verify UPSERT increments hits counter
    for i in 1..=5 {
        let req = Request::builder()
            .method("GET")
            .uri(target_path)
            .header("User-Agent", format!("ChallengerBot/{}", i))
            .header("Referer", "https://google.com/search")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    let logs = db.list_404_logs(10).expect("list 404 logs");
    let logged = logs.iter().find(|l| l.path == target_path).expect("logged 404");
    assert_eq!(logged.hits, 5, "Repeated 404 requests must increment hit count to 5");
    assert_eq!(logged.user_agent, "ChallengerBot/5", "Last seen user agent must be preserved");

    // B. Extreme path with SQL injection characters (percent-encoded for HTTP request wire)
    let sqli_uri = "/sqli-test%27%20UNION%20SELECT%201,2,3;%20--/";
    let req_sqli = Request::builder().method("GET").uri(sqli_uri).body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req_sqli).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // C. Extreme path with percent-encoded multi-byte Unicode and emojis (RFC 3986 wire format)
    let encoded_path = "/%E4%B8%8D%E5%AD%98%E5%9C%A8%E7%9A%84%E9%A1%B5%E9%9D%A2-%F0%9F%9A%80/";
    let req_utf8 = Request::builder().method("GET").uri(encoded_path).body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req_utf8).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Verify all logged safely without SQL corruption
    let all_logs = db.list_404_logs(50).expect("list 404 logs");
    assert!(
        all_logs.iter().any(|l| l.path.contains("UNION SELECT")),
        "SQLi path must be recorded as string literal"
    );
    assert!(
        all_logs.iter().any(|l| l.path.contains("不存在的页面") || l.path.contains("🚀")),
        "Axum must decode wire percent-encoding and SQLite must preserve multi-byte UTF-8 in 404 log"
    );
}

// ---------------------------------------------------------------------------
// 3. Theme Specificity Tie-Breaking & Non-Existent Terms
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_challenger_theme_specificity_and_tie_breaking_edge_cases() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    let auth_header = |builder: axum::http::request::Builder| {
        builder
            .header("Content-Type", "application/json")
            .header("X-WP-Nonce", "wp_rest_admin")
    };

    // 1. Create two categories: DevOps (1) and Kubernetes (2)
    let cat1_res = app.clone().oneshot(auth_header(Request::builder().method("POST").uri("/wp-json/wp/v2/categories"))
        .body(Body::from(json!({ "name": "DevOps", "slug": "devops" }).to_string())).unwrap()).await.unwrap();
    let cat1_id = parse_json(cat1_res).await["id"].as_i64().unwrap();

    let cat2_res = app.clone().oneshot(auth_header(Request::builder().method("POST").uri("/wp-json/wp/v2/categories"))
        .body(Body::from(json!({ "name": "Kubernetes", "slug": "kubernetes" }).to_string())).unwrap()).await.unwrap();
    let cat2_id = parse_json(cat2_res).await["id"].as_i64().unwrap();

    // 2. Create Post assigned to BOTH categories
    let post_res = app.clone().oneshot(auth_header(Request::builder().method("POST").uri("/wp-json/wp/v2/posts"))
        .body(Body::from(json!({
            "title": "Cloud Orchestration Deep Dive",
            "slug": "cloud-orchestration",
            "content": "<p>Architecture</p>",
            "status": "publish",
            "categories": [cat1_id, cat2_id]
        }).to_string())).unwrap()).await.unwrap();
    assert_eq!(post_res.status(), StatusCode::CREATED);

    // 3. Create Template A targeting DevOps (equal priority 10, specificity 30)
    let tpl_a_payload = json!({
        "title": "DevOps Special Header",
        "template_type": "header",
        "priority": 10,
        "is_active": true,
        "layout": make_text_layout("hdr_devops", "<div class=\"hdr-tag\">DevOps Banner</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "category", "target_id": cat1_id.to_string(), "priority": 10 }]
    });
    let res_a = app.clone().oneshot(auth_header(Request::builder().method("POST").uri("/wp-json/neopress/v1/templates"))
        .body(Body::from(tpl_a_payload.to_string())).unwrap()).await.unwrap();
    assert_eq!(res_a.status(), StatusCode::CREATED);

    // 4. Create Template B targeting Kubernetes (equal priority 10, specificity 30, higher ID)
    let tpl_b_payload = json!({
        "title": "Kubernetes Special Header",
        "template_type": "header",
        "priority": 10,
        "is_active": true,
        "layout": make_text_layout("hdr_k8s", "<div class=\"hdr-tag\">Kubernetes Banner</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "category", "target_id": cat2_id.to_string(), "priority": 10 }]
    });
    let res_b = app.clone().oneshot(auth_header(Request::builder().method("POST").uri("/wp-json/neopress/v1/templates"))
        .body(Body::from(tpl_b_payload.to_string())).unwrap()).await.unwrap();
    assert_eq!(res_b.status(), StatusCode::CREATED);

    // 5. Query single post route: When priority and specificity are tied, highest ID (Template B) must win!
    let req_ssr = Request::builder().method("GET").uri("/cloud-orchestration/").body(Body::empty()).unwrap();
    let res_ssr = app.clone().oneshot(req_ssr).await.unwrap();
    assert_eq!(res_ssr.status(), StatusCode::OK);
    let html_bytes = to_bytes(res_ssr.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&html_bytes);

    assert!(
        html.contains("Kubernetes Banner"),
        "Deterministic tie-breaking: Higher template ID must win when priority and specificity are equal"
    );
    assert!(
        !html.contains("DevOps Banner"),
        "Lower template ID must be overridden in tie-break"
    );

    // 6. Test Non-Existent Category Condition: Should safely evaluate to NoMatch
    let non_existent_payload = json!({
        "title": "Ghost Category Header",
        "template_type": "header",
        "priority": 100, // Very high priority
        "is_active": true,
        "layout": make_text_layout("hdr_ghost", "<div class=\"hdr-ghost\">Ghost Header</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "category", "target_id": "99999999", "priority": 100 }]
    });
    let res_ghost = app.clone().oneshot(auth_header(Request::builder().method("POST").uri("/wp-json/neopress/v1/templates"))
        .body(Body::from(non_existent_payload.to_string())).unwrap()).await.unwrap();
    assert_eq!(res_ghost.status(), StatusCode::CREATED);

    // Single post route should NOT match Ghost header despite priority 100
    let res_ssr2 = app.clone().oneshot(Request::builder().method("GET").uri("/cloud-orchestration/").body(Body::empty()).unwrap()).await.unwrap();
    let body_bytes2 = to_bytes(res_ssr2.into_body(), usize::MAX).await.unwrap();
    let html2 = String::from_utf8_lossy(&body_bytes2);
    assert!(!html2.contains("Ghost Header"), "Non-existent category condition must not match");
}

// ---------------------------------------------------------------------------
// 4. High Concurrency Stress Across REST, Layouts, SSR and 404s
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_challenger_concurrent_rest_and_ssr_stress() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = Arc::new(create_app(db.clone()));

    // Seed initial post
    let init_req = Request::builder()
        .method("POST")
        .uri("/wp-json/wp/v2/posts")
        .header("Content-Type", "application/json")
        .header("X-WP-Nonce", "wp_rest_admin")
        .body(Body::from(json!({ "title": "Base Stress Post", "slug": "stress-post", "content": "<p>Base</p>", "status": "publish" }).to_string()))
        .unwrap();
    let init_res = app.as_ref().clone().oneshot(init_req).await.unwrap();
    assert_eq!(init_res.status(), StatusCode::CREATED);

    let mut tasks = Vec::new();
    let num_tasks = 16;

    for i in 0..num_tasks {
        let app_clone = Arc::clone(&app);
        tasks.push(tokio::spawn(async move {
            let app_instance = app_clone.as_ref().clone();
            match i % 4 {
                0 => {
                    // Create post
                    let req = Request::builder()
                        .method("POST")
                        .uri("/wp-json/wp/v2/posts")
                        .header("Content-Type", "application/json")
                        .header("X-WP-Nonce", "wp_rest_admin")
                        .body(Body::from(json!({
                            "title": format!("Concurrent Stress Post {}", i),
                            "slug": format!("stress-post-{}", i),
                            "content": "<p>Stress Content</p>",
                            "status": "publish"
                        }).to_string()))
                        .unwrap();
                    let res = app_instance.oneshot(req).await.unwrap();
                    assert_eq!(res.status(), StatusCode::CREATED);
                }
                1 => {
                    // Save layout
                    let req = Request::builder()
                        .method("POST")
                        .uri("/wp-json/neopress/v1/layout/1")
                        .header("Content-Type", "application/json")
                        .header("X-WP-Nonce", "wp_rest_admin")
                        .body(Body::from(json!({
                            "layout": make_text_layout(&format!("c_node_{}", i), "<p>Updated Layout</p>")
                        }).to_string()))
                        .unwrap();
                    let res = app_instance.oneshot(req).await.unwrap();
                    assert_eq!(res.status(), StatusCode::OK);
                }
                2 => {
                    // Read SSR
                    let req = Request::builder().method("GET").uri("/stress-post/").body(Body::empty()).unwrap();
                    let res = app_instance.oneshot(req).await.unwrap();
                    assert_eq!(res.status(), StatusCode::OK);
                }
                _ => {
                    // 404 hit
                    let req = Request::builder()
                        .method("GET")
                        .uri(&format!("/missing-concurrent-path-{}/", i))
                        .body(Body::empty())
                        .unwrap();
                    let res = app_instance.oneshot(req).await.unwrap();
                    assert_eq!(res.status(), StatusCode::NOT_FOUND);
                }
            }
        }));
    }

    for task in tasks {
        task.await.expect("Concurrent task failed");
    }

    // Verify database remains responsive and uncorrupted
    let stats_req = Request::builder()
        .method("GET")
        .uri("/wp-json/neopress/v1/stats")
        .header("X-WP-Nonce", "wp_rest_admin")
        .body(Body::empty())
        .unwrap();
    let stats_res = app.as_ref().clone().oneshot(stats_req).await.unwrap();
    assert_eq!(stats_res.status(), StatusCode::OK);
}
