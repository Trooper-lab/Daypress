//! Security Audit Remediation Regression Tests
//!
//! Covers confirmed vulnerabilities fixed across Steps 1-7 on branch
//! `fix/security-audit-remediation`.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use base64::Engine;
use daypress_server::{create_app, handlers, CreatePostRequest, CreateUserRequest, Database};
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
    file_db.seed_data().expect("seed data on temp file db");

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
                .body(Body::from(json!({ "post_id": admin_draft.id }).to_string()))
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
                .body(Body::from(json!({ "post_id": admin_draft.id }).to_string()))
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
                .body(Body::from(json!({ "post_id": own_draft.id }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(author_own_audit.status(), StatusCode::OK);
    let _ = handlers::handle_setup_network_permission;
}

#[tokio::test]
async fn step2_rbac_bfla_bola_and_user_escalation_blocked() {
    use daypress_server::theme_builder::{
        models::{CreateTemplateRequest, TemplateType},
        ThemeTemplateRepository,
    };

    let db = Database::new_in_memory().expect("in-memory DB");
    db.set_option("daypress_brand_color", "#112233")
        .expect("set initial brand color");

    let contributor = db
        .create_user(&CreateUserRequest {
            username: "contrib_s2".to_string(),
            email: "contrib_s2@example.com".to_string(),
            password: "ContribPassword123!".to_string(),
            role: Some("contributor".to_string()),
            ..Default::default()
        })
        .expect("create contributor");
    let author = db
        .create_user(&CreateUserRequest {
            username: "author_s2".to_string(),
            email: "author_s2@example.com".to_string(),
            password: "AuthorPassword123!".to_string(),
            role: Some("author".to_string()),
            ..Default::default()
        })
        .expect("create author");
    let _editor = db
        .create_user(&CreateUserRequest {
            username: "editor_s2".to_string(),
            email: "editor_s2@example.com".to_string(),
            password: "EditorPassword123!".to_string(),
            role: Some("editor".to_string()),
            ..Default::default()
        })
        .expect("create editor");

    let admin_post = db
        .create_post(&CreatePostRequest {
            title: "Admin Post".to_string(),
            content: "<p>Admin content</p>".to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            author: Some(1),
            ..Default::default()
        })
        .expect("create admin post");
    let author_post = db
        .create_post(&CreatePostRequest {
            title: "Author Own Draft".to_string(),
            content: "<p>Draft</p>".to_string(),
            status: "draft".to_string(),
            post_type: "post".to_string(),
            author: Some(author.id),
            ..Default::default()
        })
        .expect("create author post");
    let admin_draft = db
        .create_post(&CreatePostRequest {
            title: "Secret Admin Draft".to_string(),
            content: "<p>Private admin draft</p>".to_string(),
            status: "draft".to_string(),
            post_type: "post".to_string(),
            author: Some(1),
            ..Default::default()
        })
        .expect("create admin draft");

    let repo = ThemeTemplateRepository::from_database(&db);
    let tpl = repo
        .create(&CreateTemplateRequest {
            title: "Header Secret Template".to_string(),
            template_type: TemplateType::Header,
            status: Some("draft".to_string()),
            layout: None,
            layout_json: None,
            settings: None,
            priority: Some(10),
            is_active: Some(false),
            conditions: None,
        })
        .expect("create template");

    let contrib_auth = basic_auth("contrib_s2", "ContribPassword123!");
    let author_auth = basic_auth("author_s2", "AuthorPassword123!");
    let editor_auth = basic_auth("editor_s2", "EditorPassword123!");
    let admin_auth = basic_auth("admin", "admin");

    let app = create_app(db.clone());

    // 1. BFLA checks: Contributor and Author rejected (403) from Editor+/Admin endpoints
    for (method, uri, body) in [
        ("POST", "/wp-json/daypress/v1/languages", r#"{"code":"fr"}"#),
        ("POST", "/wp-json/daypress/v1/posts/empty-trash", r#"{}"#),
        (
            "POST",
            "/wp-json/daypress/v1/comments/bulk",
            r#"{"action":"delete","comment_ids":[1]}"#,
        ),
        ("GET", "/wp-json/daypress/v1/cache/stats", ""),
        (
            "POST",
            "/wp-json/daypress/v1/cache/purge",
            r#"{"cf_api_token":"evil"}"#,
        ),
        ("GET", "/wp-json/daypress/v1/field-groups", ""),
        ("POST", "/wp-json/daypress/v1/search/reindex", r#"{}"#),
        ("GET", "/wp-json/daypress/v1/newsletters/subscribers", ""),
        ("GET", "/wp-json/daypress/v1/shortlinks", ""),
        ("GET", "/wp-json/daypress/v1/webhooks", ""),
        ("GET", "/wp-json/daypress/v1/ab-tests", ""),
        ("GET", "/wp-json/daypress/v1/analytics/vitals", ""),
        ("GET", "/wp-json/daypress/v1/wasm-extensions", ""),
        ("GET", "/wp-json/daypress/v1/security/audit", ""),
        ("GET", "/wp-json/daypress/v1/benchmarks/latest", ""),
        ("GET", "/wp-json/daypress/v1/templates", ""),
        ("GET", "/wp-json/daypress/v1/templates/matrix", ""),
        ("GET", "/wp-json/wp/v2/users", ""),
    ] {
        for caller in [&contrib_auth, &author_auth] {
            let mut req = Request::builder()
                .method(method)
                .uri(uri)
                .header("Authorization", caller);
            if !body.is_empty() {
                req = req.header("Content-Type", "application/json");
            }
            let res = app
                .clone()
                .oneshot(req.body(Body::from(body.to_string())).unwrap())
                .await
                .unwrap();
            assert_eq!(
                res.status(),
                StatusCode::FORBIDDEN,
                "Expected 403 for {} {} with caller auth",
                method,
                uri
            );
        }
    }

    // Also unauthenticated GET /templates & /field-groups & /users fail closed (401)
    for uri in [
        "/wp-json/daypress/v1/templates",
        "/wp-json/daypress/v1/templates/matrix",
        "/wp-json/daypress/v1/field-groups",
        "/wp-json/wp/v2/users",
    ] {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "Expected 401 on {uri}"
        );
    }

    // Also get_template, get_conditions, preview_template, assign-page require AdminUser (Editor rejected with 403)
    for (method, uri, body) in [
        (
            "GET",
            format!("/wp-json/daypress/v1/templates/{}", tpl.id),
            String::new(),
        ),
        (
            "GET",
            format!("/wp-json/daypress/v1/templates/{}/conditions", tpl.id),
            String::new(),
        ),
        (
            "GET",
            format!("/wp-json/daypress/v1/templates/preview/{}", tpl.id),
            String::new(),
        ),
        (
            "POST",
            "/wp-json/daypress/v1/templates/assign-page".to_string(),
            json!({"post_id": 2, "header_id": tpl.id}).to_string(),
        ),
    ] {
        let mut req = Request::builder()
            .method(method)
            .uri(&uri)
            .header("Authorization", &editor_auth);
        if !body.is_empty() {
            req = req.header("Content-Type", "application/json");
        }
        let res = app
            .clone()
            .oneshot(req.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "Expected Editor to be blocked on Admin template endpoint {} {}",
            method,
            uri
        );
    }

    // Editor CAN use EditorOrAbove endpoints (empty-trash & bulk comments)
    let editor_trash_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/posts/empty-trash")
                .header("Authorization", &editor_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(editor_trash_res.status(), StatusCode::OK);

    // 2. BOLA checks: Author/Contributor blocked from reading Admin user detail, other user's drafts in builder, preview-links, revisions, bulk-edit
    let bola_user_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/wp-json/wp/v2/users/1")
                .header("Authorization", &author_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bola_user_res.status(), StatusCode::FORBIDDEN);

    // Author CAN read own user profile by ID
    let own_user_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/wp-json/wp/v2/users/{}", author.id))
                .header("Authorization", &author_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(own_user_res.status(), StatusCode::OK);

    let builder_bola_post = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/d-admin/builder?post_id={}", admin_draft.id))
                .header("Authorization", &author_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(builder_bola_post.status(), StatusCode::FORBIDDEN);

    let builder_bola_tpl = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/d-admin/builder?template_id={}", tpl.id))
                .header("Authorization", &editor_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(builder_bola_tpl.status(), StatusCode::FORBIDDEN);

    for (method, uri, body) in [
        (
            "POST",
            format!(
                "/wp-json/daypress/v1/posts/{}/preview-links",
                admin_draft.id
            ),
            r#"{"label":"hijack"}"#.to_string(),
        ),
        (
            "GET",
            format!(
                "/wp-json/daypress/v1/posts/{}/preview-links",
                admin_draft.id
            ),
            String::new(),
        ),
        (
            "GET",
            format!(
                "/wp-json/daypress/v1/posts/{}/revisions/diff",
                admin_draft.id
            ),
            String::new(),
        ),
        (
            "POST",
            "/wp-json/daypress/v1/posts/bulk-edit".to_string(),
            json!({"post_ids": [admin_post.id], "status": "draft"}).to_string(),
        ),
        (
            "POST",
            "/wp-json/daypress/v1/posts/bulk-edit".to_string(),
            json!({"post_ids": [author_post.id], "author": contributor.id}).to_string(),
        ),
    ] {
        let mut req = Request::builder()
            .method(method)
            .uri(&uri)
            .header("Authorization", &author_auth);
        if !body.is_empty() {
            req = req.header("Content-Type", "application/json");
        }
        let res = app
            .clone()
            .oneshot(req.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "Expected BOLA 403 on {} {}",
            method,
            uri
        );
    }

    // 3. Author saving layout on OWN post cannot escalate to overwrite site-wide global_styles!
    let save_layout_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/daypress/v1/layout/{}", author_post.id))
                .header("Authorization", &author_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "layout": {
                            "version": 1,
                            "root": { "id": "root", "type": "Root", "children": [] },
                            "global_styles": {
                                "system_colors": [
                                    { "id": "primary", "title": "Primary", "color": "#ff00ff" }
                                ]
                            }
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_layout_res.status(), StatusCode::OK);
    assert_eq!(
        db.get_option("daypress_brand_color").unwrap().as_deref(),
        Some("#112233"),
        "Author layout save must never overwrite global brand color"
    );

    // 4. Draft post_id passed to /templates/active must not leak draft post_type
    let active_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/wp-json/daypress/v1/templates/active?post_id={}",
                    admin_draft.id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(active_res.status(), StatusCode::OK);

    // Verify Admin still succeeds on /wp-json/daypress/v1/templates
    let admin_tpls = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/wp-json/daypress/v1/templates")
                .header("Authorization", admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_tpls.status(), StatusCode::OK);
}

#[tokio::test]
async fn step2_webhook_secrets_redacted_in_rest_and_admin_ui() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let raw_secret = "whsec_top_secret_token_never_leak_12345";
    let wh = db
        .save_webhook_endpoint(&daypress_server::models::CreateWebhookEndpointRequest {
            id: None,
            name: "Audit Redaction Hook".to_string(),
            target_url: "mock://hooks.example.test/events".to_string(),
            secret: raw_secret.to_string(),
            events: vec!["post.published".to_string()],
            preset: "custom".to_string(),
            is_active: true,
        })
        .expect("create webhook");
    assert_eq!(wh.secret, raw_secret);

    let app = create_app(db.clone());
    let admin_auth = basic_auth("admin", "admin");

    // REST list redacts secret
    let rest_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/wp-json/daypress/v1/webhooks")
                .header("Authorization", &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rest_res.status(), StatusCode::OK);
    let rest_bytes = to_bytes(rest_res.into_body(), usize::MAX).await.unwrap();
    let rest_str = String::from_utf8_lossy(&rest_bytes);
    assert!(!rest_str.contains("whsec_top_secret"));
    assert!(rest_str.contains("••••••••"));

    // Admin UI redacts secret preview (even prefix chars)
    let admin_res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/d-admin/webhooks")
                .header("Authorization", &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_res.status(), StatusCode::OK);
    let html_bytes = to_bytes(admin_res.into_body(), usize::MAX).await.unwrap();
    let html_str = String::from_utf8_lossy(&html_bytes);
    assert!(!html_str.contains("whsec_top"));
    assert!(html_str.contains("••••••••"));

    // Internal dispatch still signs using the real secret
    let post = db
        .create_post(&CreatePostRequest {
            title: "Webhook Internal Signing Verification".to_string(),
            content: "<p>Event</p>".to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            ..Default::default()
        })
        .expect("publish post");
    let _ = post;
    let deliveries = db
        .list_webhook_deliveries(Some(wh.id), 5)
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    let d = &deliveries[0];
    let parsed: Value = serde_json::from_str(&d.payload_json).unwrap();
    let ts = parsed["timestamp"].as_i64().unwrap();
    let expected =
        daypress_server::adapters::database::webhooks_shortlinks::compute_webhook_hmac_sha256(
            raw_secret,
            ts,
            &d.payload_json,
        );
    assert_eq!(d.signature_header, expected);
}

#[tokio::test]
async fn step3_newsletter_magic_link_no_sync_unlock_no_unsub_cookie_no_enumeration() {
    use daypress_server::models::NewsletterSubscribeRequest;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    db.create_post(&CreatePostRequest {
        title: "Subscriber Paywall Article".to_string(),
        slug: Some("gated-magic-article".to_string()),
        content: "<p>Public teaser paragraph.</p><!--more--><p>SECRET_MAGIC_PAYWALL_DATA</p>"
            .to_string(),
        status: "publish".to_string(),
        post_type: "post".to_string(),
        member_gate: Some("subscribers_only".to_string()),
        ..Default::default()
    })
    .expect("create gated post");

    // Pre-existing active subscriber created by admin / confirmed previously
    let alice = db
        .subscribe_newsletter_contact(
            &NewsletterSubscribeRequest {
                email: "alice-subscriber@example.com".to_string(),
                name: "Alice".to_string(),
                lists: vec!["general".to_string()],
                source_url: "/gated-magic-article/".to_string(),
                skip_double_opt_in: true,
            },
            "127.0.0.1",
        )
        .expect("subscribe alice");
    let old_confirm = alice.confirm_token.clone().expect("confirm_token");
    let unsub_tok = alice.unsubscribe_token.clone().expect("unsub_token");

    // 1. Forged cookie with raw unsubscribe_token or confirm_token or ?subscriber_token MUST NOT unlock!
    for hdr_or_qs in [
        ("cookie", format!("daypress_subscriber={}", unsub_tok)),
        ("cookie", format!("daypress_subscriber={}", old_confirm)),
    ] {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/gated-magic-article/")
                    .header("Cookie", hdr_or_qs.1)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let html = String::from_utf8(
            to_bytes(res.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(
            !html.contains("SECRET_MAGIC_PAYWALL_DATA"),
            "Raw unsubscribe/confirm token in cookie must never unlock member gate"
        );
    }

    // Also `?subscriber_token=` in query string must not unlock even with a real signed token
    let signed_tok = db.issue_subscriber_cookie_token(alice.id);
    let qs_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/gated-magic-article/?subscriber_token={}",
                    signed_tok
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let qs_html = String::from_utf8(
        to_bytes(qs_res.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        !qs_html.contains("SECRET_MAGIC_PAYWALL_DATA"),
        "Query string subscriber_token must be ignored to prevent token leakage in URLs"
    );

    // 2. Attacker posting Alice's email to /newsletters/unlock gets NO cookie, unlocked=false, and uniform response
    let unlock_existing = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/newsletters/unlock")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "alice-subscriber@example.com",
                        "source_url": "//evil.example.com/phish"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unlock_existing.status(), StatusCode::OK);
    assert!(
        unlock_existing
            .headers()
            .get(axum::http::header::SET_COOKIE)
            .is_none(),
        "/newsletters/unlock must NOT issue Set-Cookie before email link click"
    );
    let existing_json = body_to_json(unlock_existing).await;
    assert_eq!(existing_json["unlocked"], false);
    assert_eq!(existing_json["status"], "verification_sent");

    let unlock_unknown = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/newsletters/unlock")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "nonexistent-visitor@example.com",
                        "source_url": "/gated-magic-article/"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unlock_unknown.status(), StatusCode::OK);
    assert!(unlock_unknown
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .is_none());
    let unknown_json = body_to_json(unlock_unknown).await;
    assert_eq!(
        existing_json, unknown_json,
        "/newsletters/unlock must return uniform response without email enumeration"
    );

    // 3. Public /newsletters/subscribe with existing email OR skip_double_opt_in=true must NOT unlock or set cookie
    let sub_existing = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/newsletters/subscribe")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "alice-subscriber@example.com",
                        "skip_double_opt_in": true
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(sub_existing
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .is_none());
    let sub_existing_json = body_to_json(sub_existing).await;
    assert_eq!(sub_existing_json["unlocked"], false);
    assert_eq!(sub_existing_json["status"], "pending");

    // 4. Fetch the rotated magic-link confirm_token generated for Alice & click /newsletter/confirm/{token}
    let refreshed = db
        .list_subscribers(
            Some("subscribed"),
            None,
            Some("alice-subscriber@example.com"),
            true,
        )
        .unwrap();
    let magic_tok = refreshed[0].confirm_token.clone().expect("magic token");
    assert_ne!(
        magic_tok, old_confirm,
        "Magic link request must rotate token"
    );
    // Unsafe source_url (//evil.example.com) must not replace safe source_url
    assert_eq!(refreshed[0].source_url, "/gated-magic-article/");

    let confirm_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/newsletter/confirm/{}", magic_tok))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(confirm_res.status(), StatusCode::OK);
    let set_cookie = confirm_res
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .expect("signed subscriber cookie")
        .to_string();
    assert!(
        set_cookie.contains("HttpOnly"),
        "Subscriber cookie must be HttpOnly"
    );
    assert!(
        !set_cookie.contains(&unsub_tok),
        "Cookie must never contain unsubscribe_token"
    );
    assert!(
        !set_cookie.contains(&magic_tok),
        "Cookie must never contain confirm_token"
    );

    // One-time magic link cannot be replayed
    let replay_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/newsletter/confirm/{}", magic_tok))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(replay_res.status(), StatusCode::NOT_FOUND);

    // Real signed HttpOnly cookie unlocks the article!
    let cookie_pair = set_cookie.split(';').next().unwrap().to_string();
    let unlocked_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/gated-magic-article/")
                .header("Cookie", &cookie_pair)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let unlocked_html = String::from_utf8(
        to_bytes(unlocked_res.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(unlocked_html.contains("SECRET_MAGIC_PAYWALL_DATA"));
}

#[tokio::test]
async fn step3_member_gate_password_and_draft_isolation_across_ssr_rest_rss_og_search() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let admin_auth = basic_auth("admin", "admin");

    let contrib = db
        .create_user(&CreateUserRequest {
            username: "contrib_s3".to_string(),
            email: "contrib_s3@example.com".to_string(),
            password: "Password123!".to_string(),
            role: Some("contributor".to_string()),
            ..Default::default()
        })
        .unwrap();
    let contrib_auth = basic_auth(&contrib.username, "Password123!");

    // 1. Gated leeg.php / raw <!DOCTYPE html page
    let raw_gated = db
        .create_post(&CreatePostRequest {
            title: "Gated Landing Doc".to_string(),
            slug: Some("gated-raw-doc".to_string()),
            content: "<!DOCTYPE html><html><body><p>PUBLIC_GATE_HEAD</p><!--more--><p>SECRET_RAW_HTML_PAYWALL</p></body></html>"
                .to_string(),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            template: Some("page_templates/leeg.php".to_string()),
            member_gate: Some("members_only".to_string()),
            acf: Some(json!({ "secret_field": "SECRET_ACF_VALUE" })),
            ..Default::default()
        })
        .unwrap();

    // 2. Password-protected post + gated post with layout AST
    let pw_post = db
        .create_post(&CreatePostRequest {
            title: "Passworded Post Title".to_string(),
            slug: Some("pw-locked-post".to_string()),
            content: "<p>SECRET_PASSWORDED_BODY_TOKEN</p>".to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            password: Some("SuperSecretPostPass99".to_string()),
            acf: Some(json!({ "private_acf": "SECRET_PW_ACF" })),
            ..Default::default()
        })
        .unwrap();
    db.save_post_layout(
        pw_post.id,
        r#"{"version":1,"root":{"id":"sec_pw","type":"Section","children":[{"id":"txt_pw","type":"Text","data":{"content":"SECRET_AST_NODE_TEXT"}}]}}"#,
        None,
    )
    .unwrap();

    // 3. Another user's private/draft post (owned by admin=1)
    let admin_draft = db
        .create_post(&CreatePostRequest {
            title: "Secret Admin Draft".to_string(),
            slug: Some("secret-admin-draft".to_string()),
            content: "<p>SECRET_ADMIN_DRAFT_BODY</p>".to_string(),
            status: "draft".to_string(),
            post_type: "post".to_string(),
            author: Some(1),
            ..Default::default()
        })
        .unwrap();

    // Also test front page member gate bypass when static front page is gated
    db.set_option("show_on_front", "page").unwrap();
    db.set_option("page_on_front", &raw_gated.id.to_string())
        .unwrap();

    // Anonymous requests across SSR (?page_id=, /, /{slug}/, /{pt}/{slug}/), REST, layout, collection, feed, og-image, search
    for uri in [
        format!("/?page_id={}", raw_gated.id),
        "/".to_string(),
        "/gated-raw-doc/".to_string(),
        format!("/?p={}", pw_post.id),
        "/pw-locked-post/".to_string(),
        format!("/post/{}/", pw_post.slug),
        format!("/wp-json/wp/v2/pages/{}", raw_gated.id),
        format!("/wp-json/wp/v2/posts/{}", pw_post.id),
        "/wp-json/wp/v2/posts".to_string(),
        "/wp-json/daypress/v1/collection-query?post_type=post".to_string(),
        "/feed/".to_string(),
        format!("/og-image/{}?format=svg", pw_post.slug),
        format!(
            "/wp-json/daypress/v1/og-image?post_id={}&format=svg",
            raw_gated.id
        ),
    ] {
        let res = app
            .clone()
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = String::from_utf8(
            to_bytes(res.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(
            !body.contains("SECRET_RAW_HTML_PAYWALL"),
            "Gated raw leak at {uri}"
        );
        assert!(
            !body.contains("SECRET_ACF_VALUE"),
            "Gated ACF leak at {uri}"
        );
        assert!(
            !body.contains("SECRET_PASSWORDED_BODY_TOKEN"),
            "Passworded body leak at {uri}"
        );
        assert!(
            !body.contains("SECRET_PW_ACF"),
            "Passworded ACF leak at {uri}"
        );
        assert!(
            !body.contains("SuperSecretPostPass99"),
            "Plaintext post password leaked at {uri}"
        );
    }

    // FTS5 Search must exclude password-protected posts and redact gated bodies even when queried by title or body token
    for q in [
        "SECRET_PASSWORDED_BODY_TOKEN",
        "Passworded",
        "Gated Landing Doc",
    ] {
        let s_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/wp-json/daypress/v1/search?q={}",
                        q.replace(' ', "%20")
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(s_res.status(), StatusCode::OK);
        let s_json = body_to_json(s_res).await;
        let results_str = serde_json::to_string(&s_json["results"]).unwrap();
        assert!(!results_str.contains("SECRET_PASSWORDED_BODY_TOKEN"));
        assert!(!results_str.contains("SECRET_RAW_HTML_PAYWALL"));
    }

    // Layout API rejects locked passworded/gated post for anon (`403`)
    let layout_anon = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/wp-json/daypress/v1/layout/{}", pw_post.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(layout_anon.status(), StatusCode::FORBIDDEN);

    // Contributor cannot read admin's draft via REST, list?status=draft, layout API, SSR, or OG card!
    for uri in [
        format!("/wp-json/wp/v2/posts/{}", admin_draft.id),
        format!("/wp-json/daypress/v1/layout/{}", admin_draft.id),
    ] {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&uri)
                    .header("Authorization", &contrib_auth)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "Expected 404 on {uri}");
    }
    let list_drafts = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/posts?status=draft")
                .header("Authorization", &contrib_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let drafts_body = String::from_utf8(
        to_bytes(list_drafts.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(!drafts_body.contains("SECRET_ADMIN_DRAFT_BODY"));

    // Public OG image never leaks unpublished draft title
    let og_draft = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/og-image/{}?format=svg", admin_draft.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let og_svg = String::from_utf8(
        to_bytes(og_draft.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(!og_svg.contains("Secret Admin Draft"));

    // Admin/authorized editor can still read layout and passworded post
    let admin_layout = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/wp-json/daypress/v1/layout/{}", pw_post.id))
                .header("Authorization", &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_layout.status(), StatusCode::OK);
}

#[tokio::test]
async fn step4_jwt_alg_validation_and_session_invalidation_on_password_reset() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let secret = db.get_auth_secret().unwrap();

    // 1. Forged alg=none JWT (or HS512 header over HMAC-SHA256) must be rejected
    let header_none =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let claims_json = serde_json::json!({
        "iss": "http://localhost:8090",
        "iat": chrono::Utc::now().timestamp(),
        "exp": chrono::Utc::now().timestamp() + 3600,
        "session_ver": 0,
        "data": { "user": { "id": 1, "username": "admin" } }
    });
    let claims_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims_json.to_string());
    let alg_none_tok = format!("{}.{}.", header_none, claims_b64);
    assert!(
        daypress_server::auth::verify_jwt_token(&alg_none_tok, &secret).is_err(),
        "alg=none JWT must never validate"
    );

    let header_hs512 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"HS512","typ":"JWT"}"#);
    let signing_input = format!("{}.{}", header_hs512, claims_b64);
    let sig_hs256 = daypress_server::auth::hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let forged_hs512 = format!(
        "{}.{}",
        signing_input,
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig_hs256)
    );
    let res_forged_alg = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/users/me")
                .header("Authorization", format!("Bearer {forged_hs512}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_forged_alg.status(), StatusCode::UNAUTHORIZED);

    // 2. Capture valid JWT and session cookie before password reset
    let jwt_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/jwt-auth/v1/token")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "username": "admin", "password": "admin" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(jwt_res.status(), StatusCode::OK);
    let old_jwt = body_to_json(jwt_res).await["token"]
        .as_str()
        .unwrap()
        .to_string();
    let old_cookie = daypress_server::auth::session_cookie_for(1, &db);

    // Verify both work pre-reset
    for (hdr, val) in [
        ("Authorization", format!("Bearer {old_jwt}")),
        ("Cookie", old_cookie.clone()),
    ] {
        let ok_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/wp-json/wp/v2/users/me")
                    .header(hdr, val)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok_res.status(), StatusCode::OK);
    }

    // 3. Consume single-use password reset token
    let reset_tok = db.create_password_reset_token(1).unwrap();
    let uid = db
        .consume_password_reset_token(&reset_tok, "NewStrongPass#2026")
        .unwrap()
        .expect("token consumed");
    assert_eq!(uid, 1);

    // Both old JWT and old session cookie are immediately rejected (`401`)!
    for (hdr, val) in [
        ("Authorization", format!("Bearer {old_jwt}")),
        ("Cookie", old_cookie),
    ] {
        let rev_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/wp-json/wp/v2/users/me")
                    .header(hdr, val)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            rev_res.status(),
            StatusCode::UNAUTHORIZED,
            "Pre-reset credential via {hdr} must be rejected after password reset"
        );
    }

    // 4. Re-authenticating with the new password yields a fresh JWT (bumped session_ver) that works
    let new_jwt_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/jwt-auth/v1/token")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "username": "admin", "password": "NewStrongPass#2026" })
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(new_jwt_res.status(), StatusCode::OK);
    let new_jwt = body_to_json(new_jwt_res).await["token"]
        .as_str()
        .unwrap()
        .to_string();
    let ok_new = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/users/me")
                .header("Authorization", format!("Bearer {new_jwt}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok_new.status(), StatusCode::OK);

    // 5. CLI/maintenance reset_user_password also bumps session_ver & revokes the newest JWT
    db.reset_user_password("admin", Some("CliStrongPass#2026"))
        .unwrap();
    let rev_cli = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/users/me")
                .header("Authorization", format!("Bearer {new_jwt}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rev_cli.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn step4_two_factor_throttled_across_jwt_and_basic_otp_endpoints() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // Enable 2FA on admin
    let setup = db.begin_user_2fa_setup(1).unwrap();
    let valid_code =
        daypress_server::totp::current_totp_code(&setup.secret, chrono::Utc::now().timestamp())
            .unwrap();
    let _enable = db.enable_user_2fa(1, &valid_code).unwrap();

    // Burn MAX_FAILED_LOGINS (5) wrong OTP attempts against JWT token endpoint
    for _ in 0..5 {
        let bad = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/jwt-auth/v1/token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "username": "admin",
                            "password": "admin",
                            "totp_code": "000000"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);
    }

    // 6th attempt is throttled across BOTH JWT and Basic X-Daypress-OTP even if valid recovery code is presented!
    let recovery = _enable.recovery_codes[0].clone();
    let throttled_jwt = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/jwt-auth/v1/token")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "username": "admin",
                        "password": "admin",
                        "totp_code": recovery
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(throttled_jwt.status(), StatusCode::TOO_MANY_REQUESTS);

    let admin_auth = basic_auth("admin", "admin");
    let throttled_basic = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/users/me")
                .header("Authorization", &admin_auth)
                .header("X-Daypress-OTP", &recovery)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(throttled_basic.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn step4_csrf_origin_null_untrusted_proxy_secure_cookies_and_hot_cache() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let admin_cookie = daypress_server::auth::session_cookie_for(1, &db);

    // 1. Origin: null must NEVER bypass CSRF protection
    let null_origin = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Cookie", &admin_cookie)
                .header("Host", "localhost:8090")
                .header("Origin", "null")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "title": "Null Origin CSRF", "content": "x" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(null_origin.status(), StatusCode::FORBIDDEN);

    // 2. Spoofed X-Forwarded-Host from untrusted peer (non-loopback Host or remote ConnectInfo) must fail
    let spoofed_host_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Cookie", &admin_cookie)
                .header("Host", "cms.example.com")
                .header("X-Forwarded-Host", "attacker.example")
                .header("Origin", "https://attacker.example")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "title": "Spoofed XFH", "content": "x" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(spoofed_host_res.status(), StatusCode::FORBIDDEN);

    // Also test with explicit remote ConnectInfo even if attacker sends Host: 127.0.0.1:8090
    let mut remote_req = Request::builder()
        .method("POST")
        .uri("/wp-json/wp/v2/posts")
        .header("Cookie", &admin_cookie)
        .header("Host", "127.0.0.1:8090")
        .header("X-Forwarded-Host", "attacker.example")
        .header("Origin", "https://attacker.example")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::json!({ "title": "Remote Peer XFH", "content": "x" }).to_string(),
        ))
        .unwrap();
    remote_req
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
            [198, 51, 100, 44],
            55555,
        ))));
    let remote_res = app.clone().oneshot(remote_req).await.unwrap();
    assert_eq!(remote_res.status(), StatusCode::FORBIDDEN);

    // 3. HotCache must bypass for daypress_admin= and daypress_subscriber= cookies
    let mut admin_headers = axum::http::HeaderMap::new();
    admin_headers.insert(axum::http::header::COOKIE, admin_cookie.parse().unwrap());
    assert!(
        daypress_server::hot_cache::SsrHotCache::has_session_cookie(&admin_headers),
        "HotCache must detect daypress_admin cookie"
    );
    let mut sub_headers = axum::http::HeaderMap::new();
    sub_headers.insert(
        axum::http::header::COOKIE,
        "daypress_subscriber=sub.1.9999999999.deadbeef"
            .parse()
            .unwrap(),
    );
    assert!(
        daypress_server::hot_cache::SsrHotCache::has_session_cookie(&sub_headers),
        "HotCache must detect daypress_subscriber cookie"
    );

    // 4. Cookies carry `; Secure` when site URL is HTTPS or trusted proxy sets X-Forwarded-Proto: https
    db.set_option("siteurl", "https://secure.example.com")
        .unwrap();
    db.set_option("home", "https://secure.example.com").unwrap();
    let https_login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from("log=admin&pwd=admin&redirect_to=/d-admin/"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(https_login.status(), StatusCode::SEE_OTHER);
    let set_cookie = https_login
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        set_cookie.contains("; Secure"),
        "HTTPS deployment must emit Secure cookies: {set_cookie}"
    );
    assert!(set_cookie.contains("; HttpOnly"));

    // Subscriber confirmation cookie also receives `; Secure`
    let sub = db
        .subscribe_newsletter_contact(
            &daypress_server::models::NewsletterSubscribeRequest {
                email: "sec_sub@example.com".to_string(),
                name: "Sec Sub".to_string(),
                lists: vec!["all".to_string()],
                source_url: "/".to_string(),
                skip_double_opt_in: false,
            },
            "127.0.0.1",
        )
        .unwrap();
    let confirm_res = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/newsletter/confirm/{}",
                    sub.confirm_token.unwrap()
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let sub_cookie = confirm_res
        .headers()
        .get(axum::http::header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        sub_cookie.contains("; Secure"),
        "HTTPS subscriber cookie must include Secure: {sub_cookie}"
    );
}

#[tokio::test]
async fn step5_private_form_uploads_isolated_from_public_media_and_static_export() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    let form_id = db
        .save_form_record(&daypress_server::models::CreateFormRequest {
            id: None,
            title: "KYC Verification Form".to_string(),
            fields: Vec::new(),
            settings: daypress_server::models::FormSettings {
                recipient_email: "security@example.com".to_string(),
                webhook_url: "https://hooks.example.com/secret-token-xyz".to_string(),
                ..Default::default()
            },
        })
        .unwrap();

    // 1. Submit multipart form upload with private PDF attachment
    let boundary = "----DAYPRESS_BOUNDARY_STEP5";
    let multipart_body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nAlice\r\n\
         --{b}\r\nContent-Disposition: form-data; name=\"passport\"; filename=\"passport-scan.pdf\"\r\n\
         Content-Type: application/pdf\r\n\r\n%PDF-1.4\nCONFIDENTIAL_PASSPORT_BYTES\r\n\
         --{b}--\r\n",
        b = boundary
    );
    let submit_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/daypress/v1/forms/{}/submit", form_id))
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(multipart_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit_res.status(), StatusCode::OK);
    let submit_json = body_to_json(submit_res).await;

    // Public response must NEVER expose secret webhook_url or raw email_error!
    assert!(submit_json.get("webhook_url").is_none());
    assert!(submit_json.get("email_error").is_none());

    let entry_id = submit_json["entry_id"].as_i64().unwrap();
    let entry = db.get_form_entry_by_id(entry_id).unwrap().unwrap();
    assert_eq!(entry.attachments.len(), 1);
    let att = &entry.attachments[0];
    assert!(
        att.stored_path.starts_with("data/form-submissions/"),
        "Attachments must reside outside public uploads/: {}",
        att.stored_path
    );
    assert!(std::path::Path::new(&att.stored_path).is_file());

    // 2. Public media server blocks form-submissions case-insensitively (even uppercase on Windows/macOS)
    for blocked_uri in [
        format!("/uploads/form-submissions/{}/{}", form_id, att.filename),
        format!("/uploads/Form-Submissions/{}/{}", form_id, att.filename),
        "/uploads/FORM-SUBMISSIONS/1/secret.pdf".to_string(),
    ] {
        let pub_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&blocked_uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            pub_res.status(),
            StatusCode::FORBIDDEN,
            "Public media must block {blocked_uri}"
        );
    }

    // 3. Static Edge Publisher never includes form-submissions even if a legacy file sits in uploads/
    let legacy_dir = std::path::Path::new("uploads")
        .join("form-submissions")
        .join("99999");
    let _ = std::fs::create_dir_all(&legacy_dir);
    let legacy_file = legacy_dir.join("legacy-kyc.txt");
    std::fs::write(&legacy_file, b"LEGACY_SECRET_KYC").unwrap();
    let built = daypress_server::publisher::build_static_site(
        &db,
        &daypress_server::publisher::PublishSettings::default(),
    )
    .await
    .unwrap();
    let _ = std::fs::remove_dir_all(std::path::Path::new("uploads").join("form-submissions"));
    assert!(
        !built
            .files
            .keys()
            .any(|k| k.to_ascii_lowercase().contains("form-submissions")),
        "Static Edge export must exclude form-submissions"
    );

    // 4. Unauthenticated user cannot download private attachment; authenticated admin can with nosniff header
    let unauth_dl = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/d-admin/forms/attachment/{}/{}",
                    entry_id, att.filename
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        unauth_dl.status() == StatusCode::UNAUTHORIZED
            || unauth_dl.status() == StatusCode::FOUND
            || unauth_dl.status() == StatusCode::SEE_OTHER
            || unauth_dl.status() == StatusCode::FORBIDDEN,
        "Unauthenticated form attachment download must be blocked, got {}",
        unauth_dl.status()
    );

    let admin_cookie = daypress_server::auth::session_cookie_for(1, &db);
    let dl_res = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/d-admin/forms/attachment/{}/{}",
                    entry_id, att.filename
                ))
                .header("Cookie", &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dl_res.status(), StatusCode::OK);
    assert_eq!(
        dl_res
            .headers()
            .get(axum::http::header::X_CONTENT_TYPE_OPTIONS)
            .unwrap(),
        "nosniff"
    );

    // Cleanup disk files created by this test
    let _ = db.delete_form(form_id);
}

#[tokio::test]
async fn step5_client_ip_spoofing_blocked_and_ssrf_dns_redirect_no_default_secret() {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    let form_id = db
        .save_form_record(&daypress_server::models::CreateFormRequest {
            id: None,
            title: "Rate Limited Form".to_string(),
            fields: Vec::new(),
            settings: daypress_server::models::FormSettings {
                rate_limit_per_minute: 2,
                ..Default::default()
            },
        })
        .unwrap();

    // 1. Remote client with ConnectInfo cannot bypass form rate limiter by rotating X-Forwarded-For!
    for spoofed in ["10.0.0.1", "10.0.0.2"] {
        let mut req = Request::builder()
            .method("POST")
            .uri(format!("/wp-json/daypress/v1/forms/{}/submit", form_id))
            .header("Content-Type", "application/json")
            .header("X-Forwarded-For", spoofed)
            .body(Body::from(r#"{"data":{"name":"Spam"}}"#))
            .unwrap();
        req.extensions_mut()
            .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
                [198, 51, 100, 77],
                41234,
            ))));
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
    let mut req3 = Request::builder()
        .method("POST")
        .uri(format!("/wp-json/daypress/v1/forms/{}/submit", form_id))
        .header("Content-Type", "application/json")
        .header("X-Forwarded-For", "10.0.0.3")
        .body(Body::from(r#"{"data":{"name":"Spam3"}}"#))
        .unwrap();
    req3.extensions_mut()
        .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
            [198, 51, 100, 77],
            41234,
        ))));
    let res3 = app.oneshot(req3).await.unwrap();
    assert_eq!(
        res3.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Untrusted X-Forwarded-For rotation must not bypass form rate limiting"
    );

    // 2. SSRF URL guard + DNS-pinning client reject loopback/metadata/hex/IPv6-mapped targets
    for ssrf_url in [
        "http://127.0.0.1:8080/hook",
        "http://[::ffff:169.254.169.254]/latest/meta-data/",
        "http://2130706433/admin",
        "http://0x7f.0.0.1/admin",
    ] {
        assert!(
            Database::validate_webhook_target_url_ssrf(ssrf_url).is_err(),
            "Expected SSRF rejection for {ssrf_url}"
        );
    }

    // DNS-pinning reqwest client rejects localhost resolution when DAYPRESS_ALLOW_PRIVATE_WEBHOOKS != 1
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_port = listener.local_addr().unwrap().port();
    let ssrf_client =
        daypress_server::adapters::database::webhooks_shortlinks::build_ssrf_safe_http_client(
            std::time::Duration::from_secs(2),
        )
        .unwrap();
    let blocked_req = ssrf_client
        .post(format!("http://localhost:{local_port}/metadata"))
        .body("test")
        .send()
        .await;
    assert!(
        blocked_req.is_err(),
        "DNS-pinning HTTP client must block localhost resolution"
    );

    // 3. Webhook delivery with missing/cleared secret fails closed rather than signing with daypress-default-secret
    let wh_db_path = temp_db_path("step5_webhook_secret");
    let _ = std::fs::remove_file(&wh_db_path);
    let wh_db = Database::new(&wh_db_path).expect("temp db");
    let hook = wh_db
        .save_webhook_endpoint(&daypress_server::models::CreateWebhookEndpointRequest {
            id: None,
            name: "No Default Secret".to_string(),
            target_url: "local://test".to_string(),
            preset: "generic".to_string(),
            events: vec!["post.published".to_string()],
            secret: "real-random-secret-abc123".to_string(),
            is_active: true,
        })
        .unwrap();
    wh_db
        .dispatch_outbound_webhook_event("post.published", json!({ "id": 1 }))
        .unwrap();
    let deliveries = wh_db.list_webhook_deliveries(Some(hook.id), 10).unwrap();
    let deliv_id = deliveries[0].id;
    {
        let raw_conn = rusqlite::Connection::open(&wh_db_path).unwrap();
        raw_conn
            .execute(
                "UPDATE daypress_webhooks SET secret = '' WHERE id = ?1",
                rusqlite::params![hook.id],
            )
            .unwrap();
    }
    assert!(
        wh_db.execute_http_webhook_delivery(deliv_id).await.is_err(),
        "Webhook delivery must fail closed when signing secret is missing"
    );
    assert!(
        wh_db.replay_webhook_delivery(deliv_id).await.is_err(),
        "Webhook replay must fail closed when signing secret is missing"
    );
    drop(wh_db);
    let _ = std::fs::remove_file(&wh_db_path);
}

#[tokio::test]
async fn step6_wasm_get_option_allowlist_blocks_sensitive_secrets() {
    let db_path = temp_db_path("step6_wasm_opts");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::new(&db_path).expect("file db");
    db.seed_data().expect("seed");

    db.set_option("blogname", "Public Site Title")
        .expect("set blogname");
    db.set_option("daypress_smtp_pass", "TOP-SECRET-SMTP-PASS")
        .expect("set smtp");
    db.set_option("daypress_s3_secret_key", "TOP-SECRET-S3-KEY")
        .expect("set s3");
    db.set_option("daypress_jwt_secret", "TOP-SECRET-JWT")
        .expect("set jwt");

    let req: daypress_server::InstallWasmExtensionRequest = serde_json::from_value(json!({
        "slug": "sec-opt-test",
        "name": "Option Audit Ext",
        "shortcode": "sec_opt",
        "template_source": "SITE={{option:blogname}}|SMTP={{option:daypress_smtp_pass}}|S3={{option:daypress_s3_secret_key}}|JWT={{option:daypress_jwt_secret}}",
        "capabilities": {
            "shortcodes": true,
            "read_options": true
        }
    }))
    .expect("valid install request");
    db.install_or_update_wasm_extension(&req)
        .expect("install wasm extension");

    let out = db
        .execute_wasm_extension(
            "sec-opt-test",
            "render_shortcode",
            &std::collections::HashMap::new(),
            "",
        )
        .expect("execute wasm");

    assert!(
        out.output_html.contains("SITE=Public Site Title"),
        "Safe public blogname option must be readable: {}",
        out.output_html
    );
    assert!(
        !out.output_html.contains("TOP-SECRET-SMTP-PASS")
            && !out.output_html.contains("TOP-SECRET-S3-KEY")
            && !out.output_html.contains("TOP-SECRET-JWT"),
        "Sensitive secrets leaked through Wasm get_option: {}",
        out.output_html
    );

    drop(db);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn step6_contributor_post_and_layout_xss_sanitized() {
    let db_path = temp_db_path("step6_xss_posts");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::new(&db_path).expect("file db");
    db.seed_data().expect("seed");
    db.create_user(&CreateUserRequest {
        username: "contrib_xss".to_string(),
        email: "contrib_xss@example.com".to_string(),
        password: "ContribPassword123!".to_string(),
        role: Some("contributor".to_string()),
        ..Default::default()
    })
    .expect("create contributor");

    let app = create_app(db.clone());
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Authorization", basic_auth("contrib_xss", "ContribPassword123!"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "<script>t()</script>Safe Title",
                        "content": "<p>Body</p><script>evil()</script><img src=x onerror=alert(1)><svg onload=alert(2)></svg><a href=\"javascript:alert(3)\">Go</a>",
                        "excerpt": "<img src=x onerror=alert(4)>Summary",
                        "status": "draft"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created = body_to_json(res).await;
    let post_id = created["id"].as_i64().expect("post id");

    let title_s = created["title"]["rendered"].as_str().unwrap_or("");
    let content_s = created["content"]["rendered"].as_str().unwrap_or("");
    let excerpt_s = created["excerpt"]["rendered"].as_str().unwrap_or("");
    for field in [title_s, content_s, excerpt_s] {
        let lower = field.to_ascii_lowercase();
        assert!(!lower.contains("<script"), "script tag leaked: {field}");
        assert!(!lower.contains("<svg"), "svg tag leaked: {field}");
        assert!(!lower.contains("onerror"), "onerror leaked: {field}");
        assert!(!lower.contains("onload"), "onload leaked: {field}");
        assert!(
            !lower.contains("javascript:"),
            "javascript: leaked: {field}"
        );
    }

    // Also attempt raw rendered_html + malicious AST payload via save_layout_handler as contributor
    let layout_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/daypress/v1/layout/{post_id}"))
                .header("Authorization", basic_auth("contrib_xss", "ContribPassword123!"))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "layout": {
                            "version": "1.0",
                            "root": {
                                "id": "root",
                                "type": "Section",
                                "styles": {},
                                "children": [
                                    {
                                        "id": "c1",
                                        "type": "CustomHtml",
                                        "styles": {},
                                        "data": { "raw": "<p>Hi</p><svg onload=alert(8)></svg><script>bad()</script>" },
                                        "children": []
                                    },
                                    {
                                        "id": "b1",
                                        "type": "Button",
                                        "styles": {},
                                        "data": { "text": "Click", "url": "javascript:alert(7)", "target": "_self" },
                                        "children": []
                                    }
                                ]
                            }
                        },
                        "rendered_html": "<img src=x onerror=alert(9)>"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(layout_res.status(), StatusCode::OK);

    let updated_post = db.get_post_by_id(post_id).unwrap().expect("post exists");
    let compiled = updated_post.content.rendered.to_ascii_lowercase();
    assert!(
        !compiled.contains("<script"),
        "script in compiled layout: {compiled}"
    );
    assert!(
        !compiled.contains("<svg"),
        "svg in compiled layout: {compiled}"
    );
    assert!(
        !compiled.contains("onerror"),
        "onerror in compiled layout: {compiled}"
    );
    assert!(
        !compiled.contains("javascript:alert"),
        "javascript: URL in compiled layout: {compiled}"
    );

    drop(db);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn step6_svg_avatar_upload_blocked_and_svg_sanitizer_hardened() {
    let db_path = temp_db_path("step6_svg");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::new(&db_path).expect("file db");
    db.seed_data().expect("seed");
    db.set_option("daypress_setup_completed", "1")
        .expect("complete setup");
    let admin = db
        .create_user(&CreateUserRequest {
            username: "sec_admin".to_string(),
            email: "sec_admin@example.com".to_string(),
            password: "AdminPassword123!".to_string(),
            role: Some("administrator".to_string()),
            ..Default::default()
        })
        .expect("create admin");
    let app = create_app(db.clone());

    for (fname, ct) in [
        ("evil.svg", "image/svg+xml"),
        ("evil.png", "image/svg+xml"),
        ("evil.svg", "image/png"),
    ] {
        let boundary = "----DaypressSecSvgBoundary";
        let body_str = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{fname}\"\r\nContent-Type: {ct}\r\n\r\n<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>\r\n--{boundary}--\r\n"
        );
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/wp-json/daypress/v1/users/{}/avatar", admin.id))
                    .header(
                        "Authorization",
                        basic_auth("sec_admin", "AdminPassword123!"),
                    )
                    .header(
                        "Content-Type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body_str))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "SVG avatar upload ({fname}, {ct}) must be rejected"
        );
    }

    let malicious_svg = br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><animate attributeName="href" to="javascript:alert(1)"/><set attributeName="onbegin" to="alert(1)"/><feimage href="data:image/svg+xml,<svg onload=alert(1)>"/> <a xlink:href="javascript:alert(2)"><rect width="10" height="10" style="width:expression(alert(3))"/></a></svg>"#;
    let clean = daypress_server::media::sanitize_svg_bytes(malicious_svg).expect("valid svg");
    let clean_str = String::from_utf8(clean)
        .expect("utf8 svg")
        .to_ascii_lowercase();
    assert!(
        !clean_str.contains("<animate"),
        "animate leaked: {clean_str}"
    );
    assert!(!clean_str.contains("<set"), "set leaked: {clean_str}");
    assert!(
        !clean_str.contains("<feimage"),
        "feimage leaked: {clean_str}"
    );
    assert!(
        !clean_str.contains("javascript:"),
        "javascript leaked: {clean_str}"
    );
    assert!(
        !clean_str.contains("expression("),
        "expression leaked: {clean_str}"
    );
    assert!(
        clean_str.contains("<rect"),
        "benign rect stripped: {clean_str}"
    );

    drop(db);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn step6_shortcodes_style_breakout_and_languages_onclick_hardened() {
    let cleaned = daypress_server::templates::utils::clean_post_content(
        r#"[audio src="javascript:alert(1)"] [video src="https://cdn.example.com/ok.mp4"]"#,
    );
    assert!(
        !cleaned.to_ascii_lowercase().contains("javascript:"),
        "audio shortcode emitted javascript: URL: {cleaned}"
    );
    assert!(
        cleaned.contains(r#"<video controls src="https://cdn.example.com/ok.mp4""#),
        "valid video shortcode should render: {cleaned}"
    );

    let safe_css = daypress_server::templates::utils::escape_style_tag_contents(
        "body { color: red; }\0</StYlE><script>alert(1)</script>",
    );
    assert!(
        !safe_css.to_ascii_lowercase().contains("</style"),
        "style breakout not neutralized: {safe_css}"
    );
    assert!(!safe_css.contains('\0'));

    // Also verify languages admin HTML uses data-* attributes rather than unescaped inline JS strings
    let db_path = temp_db_path("step6_langs");
    let _ = std::fs::remove_file(&db_path);
    let db = Database::new(&db_path).expect("file db");
    db.seed_data().expect("seed");
    db.set_option("daypress_setup_completed", "1")
        .expect("complete setup");
    db.create_user(&CreateUserRequest {
        username: "sec_admin".to_string(),
        email: "sec_admin@example.com".to_string(),
        password: "AdminPassword123!".to_string(),
        role: Some("administrator".to_string()),
        ..Default::default()
    })
    .expect("create admin");
    db.upsert_language(&daypress_server::DaypressLanguage {
        code: "de".to_string(),
        locale: "de_DE".to_string(),
        name: "Deutsch');alert(1);//".to_string(),
        native_name: "Deutsch".to_string(),
        flag_emoji: "DE".to_string(),
        is_default: false,
        is_rtl: false,
        is_active: true,
        sort_order: 2,
    })
    .expect("upsert language");

    let app = create_app(db.clone());
    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/d-admin/languages")
                .header(
                    "Authorization",
                    basic_auth("sec_admin", "AdminPassword123!"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = String::from_utf8(
        to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        body.contains("setAsDefaultLangFromBtn(this)")
            && body.contains("removeLang(this.dataset.code)")
            && !body.contains("onclick=\"setAsDefaultLang('")
            && !body.contains("onclick=\"removeLang('"),
        "Languages admin must bind language params safely via dataset rather than raw onclick JS interpolation"
    );

    drop(db);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn step7_cors_origin_allowlist_blocks_arbitrary_third_party_origins() {
    let db = Database::new_in_memory().expect("in-memory db");
    db.set_option("daypress_setup_completed", "1")
        .expect("complete setup");
    db.set_option("siteurl", "https://site.example.com")
        .expect("set siteurl");
    let app = create_app(db.clone());

    for bad_origin in ["https://evil.com", "http://evil.com:8080", "null"] {
        let preflight = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/wp-json/wp/v2/posts")
                    .header("Host", "site.example.com")
                    .header("Origin", bad_origin)
                    .header("Access-Control-Request-Method", "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            preflight
                .headers()
                .get("access-control-allow-origin")
                .is_none(),
            "CORS must not echo Access-Control-Allow-Origin for untrusted origin `{bad_origin}`"
        );
    }

    let good = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/wp-json/")
                .header("Host", "site.example.com")
                .header("Origin", "https://site.example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(good.status(), StatusCode::OK);
    assert_eq!(
        good.headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok()),
        Some("https://site.example.com")
    );
    assert!(
        good.headers()
            .get("access-control-allow-credentials")
            .is_none(),
        "CORS must not enable allow-credentials globally"
    );
}

#[tokio::test]
async fn step7_internal_500_errors_masked_and_csv_formula_injection_neutralized() {
    use axum::response::IntoResponse;

    let io_resp = daypress_server::errors::AppError::Io(std::io::Error::other(
        "permission denied on /var/lib/daypress/secret.sqlite",
    ))
    .into_response();
    assert_eq!(io_resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let io_body = String::from_utf8(
        to_bytes(io_resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        io_body.contains("Internal server error.") && !io_body.contains("secret.sqlite"),
        "500 Io response leaked internal details: {io_body}"
    );

    let int_resp = daypress_server::errors::AppError::Internal(
        "SQLITE_CONSTRAINT hidden_column_secret".to_string(),
    )
    .into_response();
    assert_eq!(int_resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let int_body = String::from_utf8(
        to_bytes(int_resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        int_body.contains("Internal server error.") && !int_body.contains("hidden_column_secret"),
        "500 Internal response leaked internal details: {int_body}"
    );

    // CSV export formula neutralization across both FormStudio and Newsletters
    let db = Database::new_in_memory().expect("in-memory db");
    db.set_option("daypress_setup_completed", "1")
        .expect("complete setup");
    db.create_user(&CreateUserRequest {
        username: "csv_admin".to_string(),
        email: "csv_admin@example.com".to_string(),
        password: "AdminPassword123!".to_string(),
        role: Some("administrator".to_string()),
        ..Default::default()
    })
    .expect("create admin");

    let form_id = db
        .create_form("=cmd|'/C calc'!A0", "[]", "{}")
        .expect("create form");
    db.submit_form_entry_full(
        form_id,
        "  =HYPERLINK(\"http://evil.com\",\"Go\")",
        "[]",
        "unread",
        "queued",
        "none",
        "+1+1",
        "hash1",
        "ua",
    )
    .expect("submit form entry");

    db.subscribe_newsletter_contact(
        &daypress_server::models::NewsletterSubscribeRequest {
            email: "formula-sub@example.com".to_string(),
            name: "@SUM(1,1)".to_string(),
            lists: vec!["general".to_string()],
            source_url: "\t=2+2".to_string(),
            skip_double_opt_in: true,
        },
        "hash2",
    )
    .expect("subscribe contact");

    let app = create_app(db.clone());
    let forms_csv_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/d-admin/api/forms/export-csv")
                .header(
                    "Authorization",
                    basic_auth("csv_admin", "AdminPassword123!"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forms_csv_res.status(), StatusCode::OK);
    let forms_csv = String::from_utf8(
        to_bytes(forms_csv_res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        forms_csv.contains("\"'=cmd|'/C calc'!A0\"")
            && forms_csv.contains("\"'+1+1\"")
            && forms_csv.contains("\"'  =HYPERLINK"),
        "Forms CSV must neutralize leading spreadsheet formula triggers: {forms_csv}"
    );

    let nl_csv_res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/d-admin/api/newsletters/export-csv")
                .header(
                    "Authorization",
                    basic_auth("csv_admin", "AdminPassword123!"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(nl_csv_res.status(), StatusCode::OK);
    let nl_csv = String::from_utf8(
        to_bytes(nl_csv_res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        nl_csv.contains("\"'@SUM(1,1)\""),
        "Newsletters CSV must neutralize leading spreadsheet formula triggers: {nl_csv}"
    );
    assert_eq!(
        daypress_server::templates::utils::csv_safe_cell("\t=2+2"),
        "\"'\t=2+2\""
    );
    assert_eq!(
        daypress_server::templates::utils::csv_safe_cell("-2+3"),
        "\"'-2+3\""
    );
}

#[tokio::test]
async fn step7_open_redirect_and_unsigned_updater_fail_closed() {
    for bad in [
        "//evil.com",
        "/\\evil.com",
        "/\tevil.com",
        "/path\r\nSet-Cookie: a=1",
        "/sub\\evil?next=1",
    ] {
        assert_eq!(
            daypress_server::auth::sanitize_redirect_target(Some(bad)),
            "/d-admin/",
            "Redirect target `{bad:?}` must be rejected"
        );
    }
    assert_eq!(
        daypress_server::auth::sanitize_redirect_target(Some("/d-admin/edit.php?post_type=page")),
        "/d-admin/edit.php?post_type=page"
    );

    let db = Database::new_in_memory().expect("in-memory db");
    db.set_option("daypress_setup_completed", "1")
        .expect("complete setup");
    db.create_user(&CreateUserRequest {
        username: "upd_admin".to_string(),
        email: "upd_admin@example.com".to_string(),
        password: "AdminPassword123!".to_string(),
        role: Some("administrator".to_string()),
        ..Default::default()
    })
    .expect("create admin");

    let sub = db
        .subscribe_newsletter_contact(
            &daypress_server::models::NewsletterSubscribeRequest {
                email: "redir-test@example.com".to_string(),
                name: "Redir".to_string(),
                lists: vec!["general".to_string()],
                source_url: "/\t//evil.com".to_string(),
                skip_double_opt_in: false,
            },
            "hash3",
        )
        .expect("subscribe");
    let tok = sub.confirm_token.expect("confirm token");

    let app = create_app(db.clone());
    let confirm_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/newsletter/confirm/{tok}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(confirm_res.status(), StatusCode::OK);
    let confirm_html = String::from_utf8(
        to_bytes(confirm_res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        !confirm_html.contains("evil.com"),
        "Newsletter confirmation must never reflect untrusted redirect URL: {confirm_html}"
    );

    // Manual update archive upload without valid Ed25519 signature must fail closed (400 Bad Request)
    let boundary = "----DaypressSecUpdBoundary";
    let body_bytes = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"archive\"; filename=\"daypress-v9.9.9-x86_64-unknown-linux-gnu.tar.gz\"\r\nContent-Type: application/gzip\r\n\r\nFAKE_PAYLOAD\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"sha256\"\r\n\r\n0000000000000000000000000000000000000000000000000000000000000000\r\n--{boundary}--\r\n"
    );
    let upd_res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/update/upload")
                .header(
                    "Authorization",
                    basic_auth("upd_admin", "AdminPassword123!"),
                )
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body_bytes))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        upd_res.status(),
        StatusCode::BAD_REQUEST,
        "Unsigned update archive upload must fail closed with 400 Bad Request"
    );
}
