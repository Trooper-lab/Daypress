//! Security Audit Remediation Regression Tests
//!
//! Covers confirmed vulnerabilities fixed across Steps 1-7 on branch
//! `fix/security-audit-remediation`.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use base64::Engine;
use daypress_server::{
    create_app, handlers, CreatePostRequest, CreateUserRequest, Database,
};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

fn basic_auth(user: &str, pass: &str) -> String {
    let token = base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
    format!("Basic {token}")
}

fn temp_db_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "daypress_sec_audit_{}_{}_{}.sqlite",
        label,
        std::process::id(),
        nanos
    ))
}

async fn body_to_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).expect("valid JSON response")
}

#[tokio::test]
async fn step1_auth01_missing_role_usermeta_never_grants_admin() {
    let db_path = temp_db_path("auth01");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::new_in_memory().expect("in-memory DB");

    // Verify seeded admin still retains explicit Administrator role
    let admin_user = db
        .verify_user_credentials("admin", "admin")
        .expect("query")
        .expect("admin authenticates");
    assert!(admin_user.is_admin());

    // Now test a file-backed DB where a user row lacks a `role` entry in `usermeta`
    let file_db = Database::new(&db_path).expect("file DB");
    file_db
        .seed_data()
        .expect("seed data on temp file db");

    let created = file_db
        .create_user(&CreateUserRequest {
            username: "roleless_user".to_string(),
            email: "roleless@example.com".to_string(),
            password: "StrongPassword123!".to_string(),
            role: Some("subscriber".to_string()),
            ..Default::default()
        })
        .expect("create user");

    // Delete user's `role` meta directly to simulate missing/corrupted usermeta row
    {
        let raw_conn = rusqlite::Connection::open(&db_path).expect("open raw sqlite");
        raw_conn
            .execute(
                "DELETE FROM usermeta WHERE user_id = ?1 AND meta_key = 'role'",
                rusqlite::params![created.id],
            )
            .expect("delete role meta");
    }

    let auth_user = file_db
        .verify_user_credentials("roleless_user", "StrongPassword123!")
        .expect("query")
        .expect("roleless user authenticates");
    assert_eq!(auth_user.role.to_ascii_lowercase(), "subscriber");
    assert!(!auth_user.is_admin());
    assert!(!auth_user.can_edit_content());

    let detail = file_db
        .get_user_detail(created.id)
        .expect("query detail")
        .expect("user exists");
    assert_eq!(detail.role.to_ascii_lowercase(), "subscriber");

    let all_users = file_db.get_all_users().expect("list users");
    let listed = all_users
        .iter()
        .find(|u| u.id == created.id)
        .expect("user in list");
    assert_eq!(listed.role.to_ascii_lowercase(), "subscriber");

    // Confirm HTTP request as roleless user cannot invoke admin-only endpoints
    let app = create_app(file_db.clone());
    let admin_res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/wp-json/wp/v2/settings")
                .header(
                    "Authorization",
                    basic_auth("roleless_user", "StrongPassword123!"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_res.status(), StatusCode::FORBIDDEN);

    drop(file_db);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn step1_infra01_setup_network_permission_locks_down_post_onboarding() {
    // 1. Fresh DB before onboarding allows setup probe
    let fresh_db_path = temp_db_path("infra01_fresh");
    let _ = std::fs::remove_file(&fresh_db_path);
    {
        let fresh_db = Database::new(&fresh_db_path).expect("fresh DB");
        assert!(fresh_db.needs_onboarding());
        let fresh_app = create_app(fresh_db.clone());
        let ok_res = fresh_app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/setup/network-permission")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"mode":"localhost"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok_res.status(), StatusCode::OK);
    }
    let _ = std::fs::remove_file(&fresh_db_path);

    // 2. Post-onboarding DB rejects unauthenticated and non-admin requests
    let db = Database::new_in_memory().expect("in-memory DB");
    db.set_option("daypress_setup_completed", "1")
        .expect("mark setup done");
    assert!(!db.needs_onboarding());

    let _sub = db
        .create_user(&CreateUserRequest {
            username: "sub_user".to_string(),
            email: "sub@example.com".to_string(),
            password: "SubPassword123!".to_string(),
            role: Some("subscriber".to_string()),
            ..Default::default()
        })
        .expect("create subscriber");

    let app = create_app(db.clone());

    let unauth = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/setup/network-permission")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"mode":"lan"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);

    let sub_req = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/setup/network-permission")
                .header("Authorization", basic_auth("sub_user", "SubPassword123!"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"mode":"lan"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub_req.status(), StatusCode::FORBIDDEN);
    assert_ne!(
        db.get_option("daypress_network_bind_mode")
            .ok()
            .flatten()
            .as_deref(),
        Some("lan")
    );

    let admin_req = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/setup/network-permission")
                .header("Authorization", basic_auth("admin", "admin"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"mode":"localhost"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_req.status(), StatusCode::OK);
}

#[tokio::test]
async fn step1_authz02_user_preferences_and_seo_audit_require_auth_and_ownership() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let sub = db
        .create_user(&CreateUserRequest {
            username: "sub_pref".to_string(),
            email: "sub_pref@example.com".to_string(),
            password: "SubPassword123!".to_string(),
            role: Some("subscriber".to_string()),
            ..Default::default()
        })
        .expect("create subscriber");
    let author = db
        .create_user(&CreateUserRequest {
            username: "author1".to_string(),
            email: "author1@example.com".to_string(),
            password: "AuthorPassword123!".to_string(),
            role: Some("author".to_string()),
            ..Default::default()
        })
        .expect("create author");

    let admin_draft = db
        .create_post(&CreatePostRequest {
            title: "Secret Admin Draft".to_string(),
            content: "<p>Confidential draft plan</p>".to_string(),
            status: "draft".to_string(),
            post_type: "post".to_string(),
            author: Some(1),
            ..Default::default()
        })
        .expect("create admin draft");

    let app = create_app(db.clone());

    // Unauthenticated user-preferences GET and POST must fail closed (401)
    let unauth_get = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/wp-json/daypress/v1/user-preferences")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth_get.status(), StatusCode::UNAUTHORIZED);

    let unauth_post = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/user-preferences")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"preferences":{"sidebar_collapsed":true}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth_post.status(), StatusCode::UNAUTHORIZED);

    // Authenticated user accesses their own preferences (never fallback to user 1)
    let sub_post = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/user-preferences")
                .header("Authorization", basic_auth("sub_pref", "SubPassword123!"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"preferences":{"media_view_mode":"list"}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub_post.status(), StatusCode::OK);
    let sub_json = body_to_json(sub_post).await;
    assert_eq!(sub_json["user_id"], json!(sub.id));

    // Unauthenticated and subscriber callers cannot call /seo/audit
    let unauth_audit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/seo/audit")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "post_id": admin_draft.id }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth_audit.status(), StatusCode::UNAUTHORIZED);

    let sub_audit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/seo/audit")
                .header("Authorization", basic_auth("sub_pref", "SubPassword123!"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"html":"<h1>Hi</h1>"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub_audit.status(), StatusCode::FORBIDDEN);

    // Author cannot audit another author's draft post by post_id (BOLA)
    let author_bola_audit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/seo/audit")
                .header("Authorization", basic_auth("author1", "AuthorPassword123!"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "post_id": admin_draft.id }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(author_bola_audit.status(), StatusCode::FORBIDDEN);

    // Author CAN audit their own draft post
    let own_draft = db
        .create_post(&CreatePostRequest {
            title: "Author Draft".to_string(),
            content: "<h1>Heading</h1><p>Body</p>".to_string(),
            status: "draft".to_string(),
            post_type: "post".to_string(),
            author: Some(author.id),
            ..Default::default()
        })
        .expect("create author draft");
    let author_own_audit = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/seo/audit")
                .header("Authorization", basic_auth("author1", "AuthorPassword123!"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "post_id": own_draft.id }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(author_own_audit.status(), StatusCode::OK);
    let _ = handlers::handle_setup_network_permission;
}
