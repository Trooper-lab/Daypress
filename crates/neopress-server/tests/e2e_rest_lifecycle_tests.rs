//! Milestone 4 E2E REST API Multi-Resource Lifecycle Test Suite.
//!
//! Validates:
//! 1. Hierarchical Pages CRUD lifecycle with parent/child relationships and cascade.
//! 2. Taxonomy Terms (Categories & Tags) creation, post association, and integrity.
//! 3. Comments moderation workflow with nested replies and status transitions.
//! 4. Site Settings live propagation directly to public SSR without reboot.
//! 5. Database diagnostics and maintenance (stats, integrity-check, vacuum).

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
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
async fn test_e2e_hierarchical_pages_lifecycle() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create Parent Page
    let parent_payload = json!({
        "title": "Solutions",
        "slug": "solutions",
        "content": "<p>Enterprise Solutions</p>",
        "status": "publish"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/pages", Some(parent_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let parent_json = parse_body(res).await;
    let parent_id = parent_json["id"].as_i64().expect("parent id");
    assert_eq!(parent_json["slug"], "solutions");

    // 2. Create Child Page 1
    let child1_payload = json!({
        "title": "Cloud Architecture",
        "slug": "cloud-architecture",
        "content": "<p>Cloud Details</p>",
        "parent": parent_id,
        "menu_order": 1,
        "status": "publish"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/pages", Some(child1_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let child1_json = parse_body(res).await;
    let child1_id = child1_json["id"].as_i64().expect("child1 id");
    assert_eq!(child1_json["parent"], parent_id);

    // 3. Create Child Page 2
    let child2_payload = json!({
        "title": "Security & Compliance",
        "slug": "security-compliance",
        "content": "<p>Security Details</p>",
        "parent": parent_id,
        "menu_order": 2,
        "status": "publish"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/pages", Some(child2_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let child2_id = parse_body(res).await["id"].as_i64().expect("child2 id");

    // 4. Query Child Pages using ?parent=<id>
    let res = app.clone().oneshot(auth_req("GET", &format!("/wp-json/wp/v2/pages?parent={}", parent_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let children_list = parse_body(res).await;
    let children_arr = children_list.as_array().expect("array of children");
    assert_eq!(children_arr.len(), 2);
    let found_child1 = children_arr.iter().any(|p| p["id"] == child1_id);
    let found_child2 = children_arr.iter().any(|p| p["id"] == child2_id);
    assert!(found_child1 && found_child2);

    // 5. Update Child Page 1
    let update_payload = json!({
        "title": "Global Cloud Architecture",
        "menu_order": 10
    });
    let res = app.clone().oneshot(auth_req("PUT", &format!("/wp-json/wp/v2/pages/{}", child1_id), Some(update_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated_child = parse_body(res).await;
    assert_eq!(updated_child["title"]["rendered"], "Global Cloud Architecture");
    assert_eq!(updated_child["menu_order"], 10);

    // 6. Delete Child Page 2 with ?force=true
    let res = app.clone().oneshot(auth_req("DELETE", &format!("/wp-json/wp/v2/pages/{}?force=true", child2_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let del_json = parse_body(res).await;
    assert_eq!(del_json["deleted"], true);

    // 7. Verify Child Page 2 deleted from parent listing
    let res = app.clone().oneshot(auth_req("GET", &format!("/wp-json/wp/v2/pages?parent={}", parent_id), None)).await.unwrap();
    let children_arr = parse_body(res).await.as_array().unwrap().clone();
    assert_eq!(children_arr.len(), 1);
    assert_eq!(children_arr[0]["id"], child1_id);

    // 8. Delete Parent Page
    let res = app.clone().oneshot(auth_req("DELETE", &format!("/wp-json/wp/v2/pages/{}?force=true", parent_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_e2e_taxonomy_terms_and_post_association_lifecycle() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create Category
    let cat_payload = json!({
        "name": "Cloud Native",
        "slug": "cloud-native",
        "description": "Kubernetes and Rust microservices"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/categories", Some(cat_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let cat_json = parse_body(res).await;
    let cat_id = cat_json["id"].as_i64().expect("category id");
    assert_eq!(cat_json["slug"], "cloud-native");

    // 2. Create Tag
    let tag_payload = json!({
        "name": "Axum Web",
        "slug": "axum-web",
        "description": "Ergonomic Rust web framework"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/tags", Some(tag_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let tag_json = parse_body(res).await;
    let tag_id = tag_json["id"].as_i64().expect("tag id");

    // 3. Create Post associated with Category and Tag
    let post_payload = json!({
        "title": "Deploying Axum to Production",
        "slug": "deploying-axum-to-production",
        "content": "<p>Native Rust Axum web services run with microsecond latency.</p>",
        "status": "publish",
        "categories": [cat_id],
        "tags": [tag_id]
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/posts", Some(post_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let post_json = parse_body(res).await;
    let post_id = post_json["id"].as_i64().expect("post id");
    assert!(post_json["categories"].as_array().unwrap().iter().any(|c| c == cat_id));
    assert!(post_json["tags"].as_array().unwrap().iter().any(|t| t == tag_id));

    // 4. Filter Posts by Category
    let res = app.clone().oneshot(auth_req("GET", &format!("/wp-json/wp/v2/posts?categories={}", cat_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let posts = parse_body(res).await;
    assert!(posts.as_array().unwrap().iter().any(|p| p["id"] == post_id));

    // 5. Filter Posts by Tag
    let res = app.clone().oneshot(auth_req("GET", &format!("/wp-json/wp/v2/posts?tags={}", tag_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let posts = parse_body(res).await;
    assert!(posts.as_array().unwrap().iter().any(|p| p["id"] == post_id));

    // 6. Delete Category
    let res = app.clone().oneshot(auth_req("DELETE", &format!("/wp-json/wp/v2/categories/{}?force=true", cat_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 7. Verify Post remains valid after term deletion
    let res = app.clone().oneshot(auth_req("GET", &format!("/wp-json/wp/v2/posts/{}", post_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let post_json = parse_body(res).await;
    assert_eq!(post_json["id"], post_id);
}

#[tokio::test]
async fn test_e2e_comments_moderation_tree_lifecycle() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create a Post to receive comments
    let post_payload = json!({
        "title": "Community Announcement",
        "slug": "community-announcement",
        "content": "<p>Please leave feedback below.</p>",
        "status": "publish"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/posts", Some(post_payload))).await.unwrap();
    let post_id = parse_body(res).await["id"].as_i64().expect("post id");

    // 2. Submit Comment as Visitor
    let comment_payload = json!({
        "post": post_id,
        "content": "Outstanding engineering and speed!",
        "author_name": "DevAuditor",
        "author_email": "auditor@rust.org",
        "status": "hold"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/comments", Some(comment_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let comment_json = parse_body(res).await;
    let comment_id = comment_json["id"].as_i64().expect("comment id");
    assert!(comment_json["status"] == "0" || comment_json["status"] == "hold");

    // 3. Admin Moderates Comment to 'approved'
    let approve_payload = json!({
        "status": "approved"
    });
    let res = app.clone().oneshot(auth_req("PUT", &format!("/wp-json/wp/v2/comments/{}", comment_id), Some(approve_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated = parse_body(res).await;
    assert!(updated["status"] == "1" || updated["status"] == "approved");

    // 4. Submit Nested Reply to Comment
    let reply_payload = json!({
        "post": post_id,
        "content": "Thank you for the verification!",
        "author_name": "CoreTeam",
        "author_email": "core@neopress.io",
        "parent": comment_id,
        "status": "approved"
    });
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/wp/v2/comments", Some(reply_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let reply_id = parse_body(res).await["id"].as_i64().expect("reply id");

    // 5. Query Comments Tree for Post
    let res = app.clone().oneshot(auth_req("GET", &format!("/wp-json/wp/v2/comments?post={}", post_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let comments = parse_body(res).await;
    let comments_arr = comments.as_array().expect("comments array");
    assert_eq!(comments_arr.len(), 2);
    let reply_node = comments_arr.iter().find(|c| c["id"] == reply_id).expect("reply comment");
    assert_eq!(reply_node["post_id"], post_id);
    let post = db.get_post_by_id(post_id).unwrap().expect("post exists");
    let reply = post.comments.iter().find(|c| c.id == reply_id).expect("reply comment");
    assert_eq!(reply.parent, comment_id);

    // 6. Delete Reply
    let res = app.clone().oneshot(auth_req("DELETE", &format!("/wp-json/wp/v2/comments/{}?force=true", reply_id), None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_e2e_site_settings_live_propagation_to_ssr() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Read Current Settings
    let res = app.clone().oneshot(auth_req("GET", "/wp-json/wp/v2/settings", None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Update Site Title & Description
    let new_title = "Neopress Ultra 2026 Sovereign";
    let new_desc = "Sub-millisecond Rust CMS Engine with Embedded Rusqlite";
    let settings_payload = json!({
        "title": new_title,
        "description": new_desc
    });
    let res = app.clone().oneshot(auth_req("PUT", "/wp-json/wp/v2/settings", Some(settings_payload))).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated_settings = parse_body(res).await;
    assert_eq!(updated_settings["title"], new_title);

    // 3. Immediately Query Public SSR Root (GET /)
    let ssr_req = Request::builder().method("GET").uri("/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(ssr_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&html_bytes);

    // 4. Verify Live SSR reflects new title without restart
    assert!(
        html.contains(new_title),
        "Expected SSR HTML to render updated site title: {}",
        new_title
    );
}

#[tokio::test]
async fn test_e2e_diagnostics_and_maintenance_endpoints() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Test /wp-json/neopress/v1/stats
    let res = app.clone().oneshot(auth_req("GET", "/wp-json/neopress/v1/stats", None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let stats = parse_body(res).await;
    assert!(stats["total_posts"].as_i64().is_some());
    assert!(stats["total_pages"].as_i64().is_some());
    assert!(stats["file_size_bytes"].as_i64().is_some());

    // 2. Test /wp-json/neopress/v1/integrity-check
    let res = app.clone().oneshot(auth_req("GET", "/wp-json/neopress/v1/integrity-check", None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let integrity = parse_body(res).await;
    assert_eq!(integrity["status"], "success");

    // 3. Test /wp-json/neopress/v1/vacuum
    let res = app.clone().oneshot(auth_req("POST", "/wp-json/neopress/v1/vacuum", None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let vacuum = parse_body(res).await;
    assert_eq!(vacuum["status"], "success");
    assert!(vacuum["new_size_bytes"].as_i64().is_some());
}
