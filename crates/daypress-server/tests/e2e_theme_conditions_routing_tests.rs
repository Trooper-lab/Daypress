//! Milestone 4 E2E Theme Template Display Conditions & Routing Test Suite.
//!
//! Validates:
//! 1. Priority & specificity resolution between FrontPage vs EntireSite headers on live HTTP paths.
//! 2. Category-specific template condition matching on both single posts (via hydrated terms) and category archives.
//! 3. 404 template rendering and automated SQLite 404 monitoring hit logging.
//! 4. 1-Click starter templates installation, condition generation, and REST retrieval.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
use daypress_server::{create_app, Database};
use serde_json::{json, Value};
use tower::ServiceExt;

fn auth_req(method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", "Basic YWRtaW46YWRtaW4=");
    match body {
        Some(val) => builder.body(Body::from(val.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn parse_body(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

fn make_text_layout(id: &str, html: &str) -> DaypressLayout {
    DaypressLayout {
        version: 1,
        root: DaypressNode {
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

#[tokio::test]
async fn test_e2e_theme_conditions_front_page_vs_entire_site() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create a sample post for non-frontpage routing
    let post_payload = json!({
        "title": "Kernel Memory Management",
        "slug": "kernel-memory-management",
        "content": "<p>Deep dive into memory allocators.</p>",
        "status": "publish"
    });
    let res = app
        .clone()
        .oneshot(auth_req("POST", "/wp-json/wp/v2/posts", Some(post_payload)))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 2. Create Global Header (EntireSite condition, specificity 10)
    let global_header_payload = json!({
        "title": "Global Header",
        "template_type": "header",
        "status": "publish",
        "priority": 10,
        "is_active": true,
        "layout": make_text_layout("hdr_global", "<div class=\"hdr-text\">Global Universal Header</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "entire_site", "priority": 10 }]
    });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/templates",
            Some(global_header_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Create FrontPage Header (FrontPage condition, specificity 40)
    let front_header_payload = json!({
        "title": "FrontPage Header",
        "template_type": "header",
        "status": "publish",
        "priority": 10,
        "is_active": true,
        "layout": make_text_layout("hdr_front", "<div class=\"hdr-text\">FrontPage Showcase Header</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "front_page", "priority": 10 }]
    });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/templates",
            Some(front_header_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 4. Create Global Footer (EntireSite condition)
    let footer_payload = json!({
        "title": "Universal Footer",
        "template_type": "footer",
        "status": "publish",
        "priority": 10,
        "is_active": true,
        "layout": make_text_layout("ftr_global", "<div class=\"ftr-text\">Universal Corporate Footer</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "entire_site", "priority": 10 }]
    });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/templates",
            Some(footer_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 5. Test Live HTTP GET /: FrontPage specificity (40) must beat EntireSite (10)
    let home_req = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(home_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let home_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let home_html = String::from_utf8_lossy(&home_bytes);

    assert!(
        home_html.contains("FrontPage Showcase Header"),
        "Expected FrontPage header on /"
    );
    assert!(
        !home_html.contains("Global Universal Header"),
        "Global header should be overridden by FrontPage condition on /"
    );
    assert!(
        home_html.contains("Universal Corporate Footer"),
        "Universal footer should render on /"
    );

    // 6. Test Live HTTP GET /kernel-memory-management/: FrontPage condition does not match; falls back to Global Header
    let post_req = Request::builder()
        .method("GET")
        .uri("/kernel-memory-management/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(post_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let post_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let post_html = String::from_utf8_lossy(&post_bytes);

    assert!(
        post_html.contains("Global Universal Header"),
        "Expected Global header on single post route"
    );
    assert!(
        !post_html.contains("FrontPage Showcase Header"),
        "FrontPage header must not leak to single post"
    );
    assert!(
        post_html.contains("Universal Corporate Footer"),
        "Universal footer should render on single post"
    );
}

#[tokio::test]
async fn test_e2e_theme_conditions_single_post_category_hydration_and_archive() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create Category "Distributed Systems"
    let cat_payload = json!({
        "name": "Distributed Systems",
        "slug": "distributed-systems"
    });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/wp/v2/categories",
            Some(cat_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let cat_id = parse_body(res).await["id"].as_i64().expect("cat id");

    // 2. Create Post assigned to Category
    let post_payload = json!({
        "title": "Raft Consensus Protocol",
        "slug": "raft-consensus-protocol",
        "content": "<p>Leader election and log replication principles.</p>",
        "status": "publish",
        "categories": [cat_id]
    });
    let res = app
        .clone()
        .oneshot(auth_req("POST", "/wp-json/wp/v2/posts", Some(post_payload)))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. Create Category-Specific Header Template
    let cat_header_payload = json!({
        "title": "Distributed Systems Specialized Header",
        "template_type": "header",
        "status": "publish",
        "priority": 10,
        "is_active": true,
        "layout": make_text_layout("hdr_dist", "<div class=\"hdr-special\">Distributed Systems Engineering Hub</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "category", "target_id": cat_id.to_string(), "priority": 10 }]
    });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/templates",
            Some(cat_header_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 4. Test Single Post Route: Post's category hydration in RequestContext::for_post must match Category condition
    let post_req = Request::builder()
        .method("GET")
        .uri("/raft-consensus-protocol/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(post_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let post_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let post_html = String::from_utf8_lossy(&post_bytes);

    assert!(
        post_html.contains("Distributed Systems Engineering Hub"),
        "Category-specific header must render on single post whose category matches condition"
    );

    // 5. Test Category Archive Route (/category/distributed-systems/)
    let archive_req = Request::builder()
        .method("GET")
        .uri("/category/distributed-systems/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(archive_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let archive_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let archive_html = String::from_utf8_lossy(&archive_bytes);

    assert!(
        archive_html.contains("Category: Distributed Systems"),
        "Expected archive title"
    );
    assert!(
        archive_html.contains("Distributed Systems Engineering Hub"),
        "Expected Category header on category archive route"
    );
    assert!(
        archive_html.contains("Raft Consensus Protocol"),
        "Expected post listed on category archive"
    );
}

#[tokio::test]
async fn test_e2e_theme_conditions_404_routing_and_hit_monitoring() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create 404-specific Header Template
    let not_found_payload = json!({
        "title": "Cyberpunk 404 Universe Header",
        "template_type": "header",
        "status": "publish",
        "priority": 20,
        "is_active": true,
        "layout": make_text_layout("tpl_404", "<div class=\"error-banner\">Cyberpunk 404: Lost in Deep Sub-Millisecond Space</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "not_found", "priority": 20 }]
    });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/templates",
            Some(not_found_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 2. Live HTTP GET to non-existent route
    let missing_path = "/non-existent-quantum-core/";
    let req = Request::builder()
        .method("GET")
        .uri(missing_path)
        .header("User-Agent", "Daypress-E2E-Auditor/4.0")
        .header("Referer", "https://search.daypress.io")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body_bytes);

    // 3. Verify 404 Template rendered in output
    assert!(
        html.contains("Cyberpunk 404: Lost in Deep Sub-Millisecond Space"),
        "Custom 404 theme template must be injected into NOT_FOUND response"
    );

    // 4. Verify 404 Event was recorded in SQLite daypress_404_log
    let logs = db.list_404_logs(10).expect("list 404 logs");
    let logged_hit = logs.iter().find(|l| l.path == missing_path);
    assert!(
        logged_hit.is_some(),
        "404 request path must be recorded in SQLite"
    );
    let hit = logged_hit.unwrap();
    assert_eq!(hit.hits, 1);
    assert_eq!(hit.user_agent, "Daypress-E2E-Auditor/4.0");
    assert_eq!(hit.referrer, "https://search.daypress.io");
}

#[tokio::test]
async fn test_e2e_theme_starter_templates_installation_and_conditions() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. List Starter Templates Catalog
    let res = app
        .clone()
        .oneshot(auth_req(
            "GET",
            "/wp-json/daypress/v1/templates/starter",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let starters = parse_body(res).await;
    let starters_arr = starters.as_array().expect("starter catalog array");
    assert!(
        !starters_arr.is_empty(),
        "Starters catalog should contain preset templates"
    );

    // 2. Install "agency-header" Starter Template via REST POST
    let install_payload = json!({
        "starter_id": "agency-header"
    });
    let res = app
        .clone()
        .oneshot(auth_req(
            "POST",
            "/wp-json/daypress/v1/templates/starter",
            Some(install_payload),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let installed = parse_body(res).await;
    let template_id = installed["id"].as_i64().expect("template id");
    assert_eq!(installed["template_type"], "header");

    // 3. Fetch Installed Template details via REST
    let res = app
        .clone()
        .oneshot(auth_req(
            "GET",
            &format!("/wp-json/daypress/v1/templates/{}", template_id),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let details = parse_body(res).await;
    assert_eq!(details["id"], template_id);
    let conditions = details["conditions"].as_array().expect("conditions array");
    assert!(
        !conditions.is_empty(),
        "Starter template should have default display conditions configured"
    );
    assert_eq!(conditions[0]["rule_target"], "entire_site");
}
