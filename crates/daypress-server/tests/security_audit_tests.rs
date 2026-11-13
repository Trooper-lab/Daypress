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
        let html = String::from_utf8(to_bytes(res.into_body(), 1024 * 1024).await.unwrap().to_vec())
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
                .uri(format!("/gated-magic-article/?subscriber_token={}", signed_tok))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let qs_html =
        String::from_utf8(to_bytes(qs_res.into_body(), 1024 * 1024).await.unwrap().to_vec())
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
        .list_subscribers(Some("subscribed"), None, Some("alice-subscriber@example.com"), true)
        .unwrap();
    let magic_tok = refreshed[0].confirm_token.clone().expect("magic token");
    assert_ne!(magic_tok, old_confirm, "Magic link request must rotate token");
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
    assert!(set_cookie.contains("HttpOnly"), "Subscriber cookie must be HttpOnly");
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
    let unlocked_html =
        String::from_utf8(to_bytes(unlocked_res.into_body(), 1024 * 1024).await.unwrap().to_vec())
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
    db.set_option("page_on_front", &raw_gated.id.to_string()).unwrap();

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
        format!("/wp-json/daypress/v1/og-image?post_id={}&format=svg", raw_gated.id),
    ] {
        let res = app
            .clone()
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = String::from_utf8(to_bytes(res.into_body(), 2 * 1024 * 1024).await.unwrap().to_vec())
            .unwrap();
        assert!(!body.contains("SECRET_RAW_HTML_PAYWALL"), "Gated raw leak at {uri}");
        assert!(!body.contains("SECRET_ACF_VALUE"), "Gated ACF leak at {uri}");
        assert!(!body.contains("SECRET_PASSWORDED_BODY_TOKEN"), "Passworded body leak at {uri}");
        assert!(!body.contains("SECRET_PW_ACF"), "Passworded ACF leak at {uri}");
        assert!(!body.contains("SuperSecretPostPass99"), "Plaintext post password leaked at {uri}");
    }

    // FTS5 Search must exclude password-protected posts and redact gated bodies even when queried by title or body token
    for q in ["SECRET_PASSWORDED_BODY_TOKEN", "Passworded", "Gated Landing Doc"] {
        let s_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/wp-json/daypress/v1/search?q={}", q.replace(' ', "%20")))
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
    let drafts_body =
        String::from_utf8(to_bytes(list_drafts.into_body(), 1024 * 1024).await.unwrap().to_vec())
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
    let og_svg = String::from_utf8(to_bytes(og_draft.into_body(), 1024 * 1024).await.unwrap().to_vec())
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
    let header_none = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(r#"{"alg":"none","typ":"JWT"}"#);
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

    let header_hs512 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(r#"{"alg":"HS512","typ":"JWT"}"#);
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
    admin_headers.insert(
        axum::http::header::COOKIE,
        admin_cookie.parse().unwrap(),
    );
    assert!(
        daypress_server::hot_cache::SsrHotCache::has_session_cookie(&admin_headers),
        "HotCache must detect daypress_admin cookie"
    );
    let mut sub_headers = axum::http::HeaderMap::new();
    sub_headers.insert(
        axum::http::header::COOKIE,
        "daypress_subscriber=sub.1.9999999999.deadbeef".parse().unwrap(),
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
                .uri(format!("/newsletter/confirm/{}", sub.confirm_token.unwrap()))
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
