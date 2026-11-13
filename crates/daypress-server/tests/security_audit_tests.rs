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
        ("POST", "/wp-json/daypress/v1/cache/purge", r#"{"cf_api_token":"evil"}"#),
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
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "Expected 401 on {uri}");
    }

    // Also get_template, get_conditions, preview_template, assign-page require AdminUser (Editor rejected with 403)
    for (method, uri, body) in [
        ("GET", format!("/wp-json/daypress/v1/templates/{}", tpl.id), String::new()),
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
            format!("/wp-json/daypress/v1/posts/{}/preview-links", admin_draft.id),
            r#"{"label":"hijack"}"#.to_string(),
        ),
        (
            "GET",
            format!("/wp-json/daypress/v1/posts/{}/preview-links", admin_draft.id),
            String::new(),
        ),
        (
            "GET",
            format!("/wp-json/daypress/v1/posts/{}/revisions/diff", admin_draft.id),
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
                .uri(format!("/wp-json/daypress/v1/templates/active?post_id={}", admin_draft.id))
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
    let deliveries = db.list_webhook_deliveries(Some(wh.id), 5).expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    let d = &deliveries[0];
    let parsed: Value = serde_json::from_str(&d.payload_json).unwrap();
    let ts = parsed["timestamp"].as_i64().unwrap();
    let expected = daypress_server::adapters::database::webhooks_shortlinks::compute_webhook_hmac_sha256(
        raw_secret,
        ts,
        &d.payload_json,
    );
    assert_eq!(d.signature_header, expected);
}
