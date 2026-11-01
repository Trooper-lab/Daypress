use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Json, Router,
};
use daypress_server::{
    auth::{create_versioned_session_token, SESSION_COOKIE},
    create_app,
    db::Database,
    publisher::MASKED_SECRET,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

fn setup_authenticated_app() -> (Router, Database, String) {
    let db = Database::new_in_memory().expect("in-memory db");
    let _ = db.set_option("daypress_onboarded", "1");
    let secret = db.get_auth_secret().expect("auth secret");
    let token = create_versioned_session_token(1, &secret, 86400, 0);
    let cookie = format!("{SESSION_COOKIE}={token}");
    let app = create_app(db.clone());
    (app, db, cookie)
}

#[tokio::test]
async fn test_publish_admin_screen_and_settings_persistence_with_secret_masking() {
    let (app, db, cookie) = setup_authenticated_app();

    // 1. Render /d-admin/publish
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/d-admin/publish")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body_bytes);
    assert!(html.contains("Publish"));
    assert!(html.contains("Cloudflare Pages"));
    assert!(html.contains("Netlify"));
    assert!(html.contains("Deployment History"));

    // 2. Save settings with Cloudflare & Netlify tokens
    let save_payload = json!({
        "provider": "cloudflare",
        "production_url": "https://edge.daypress.dev",
        "cf_account_id": "cf_acc_12345",
        "cf_project_name": "daypress-edge-site",
        "cf_api_token": "cf_super_secret_token_987654321",
        "cf_custom_domain": "edge.daypress.dev",
        "netlify_site_id": "nl_site_111",
        "netlify_token": "nl_secret_token_abcdef123456",
        "netlify_custom_domain": "edge.daypress.dev",
        "local_export_dir": "data/dist-test",
        "static_search_enabled": true,
        "static_redirects_enabled": true,
        "static_form_mode": "webhook",
        "static_form_webhook_url": "https://hooks.daypress.dev/form",
        "neutralize_admin_links": true
    });

    let save_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/settings")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(save_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_res.status(), StatusCode::OK);
    let saved_json: Value = serde_json::from_slice(
        &axum::body::to_bytes(save_res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(saved_json["success"], true);
    // Token should be masked in response
    assert_eq!(
        saved_json["settings"]["cf_api_token"].as_str().unwrap(),
        MASKED_SECRET
    );
    assert_eq!(saved_json["settings"]["has_cf_token"], true);
    assert_eq!(
        saved_json["settings"]["netlify_token"].as_str().unwrap(),
        MASKED_SECRET
    );
    assert_eq!(saved_json["settings"]["has_netlify_token"], true);

    // 3. Updating settings with the masked token preserves the real secret in SQLite
    let mut update_payload = saved_json["settings"].clone();
    update_payload["production_url"] = json!("https://updated.daypress.dev");
    let res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/settings")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    let full_settings = db.get_publish_settings();
    assert_eq!(
        full_settings.cf_api_token,
        "cf_super_secret_token_987654321"
    );
    assert_eq!(full_settings.netlify_token, "nl_secret_token_abcdef123456");
    assert_eq!(full_settings.production_url, "https://updated.daypress.dev");
}

#[tokio::test]
async fn test_static_compile_anonymous_crawl_enhancements_diff_queue_and_zip_download() {
    let (app, _db, cookie) = setup_authenticated_app();

    // Add a redirect rule via Admin Redirects API
    let redir_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/d-admin/api/redirects")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "source_path": "/old-pricing",
                        "target_url": "/pricing",
                        "status_code": 301
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(redir_res.status().is_success());

    // Create a published page containing a form and a /d-admin/ link
    let create_page = json!({
        "title": "Pricing & Plans",
        "slug": "pricing",
        "status": "publish",
        "post_type": "page",
        "content": "<section><h1>Pricing</h1><p>Visit <a href=\"http://localhost:8090/about\">About</a> or <a href=\"/d-admin/posts\">Admin</a>.</p><form class=\"daypress-form\" method=\"post\"><input name=\"email\" type=\"email\" /><button type=\"submit\">Join</button></form></section>"
    });
    let create_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/pages")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(create_page.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let created_page: Value = serde_json::from_slice(
        &axum::body::to_bytes(create_res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    let pricing_id = created_page["id"].as_i64().unwrap();

    // Configure local folder export in a unique temp folder
    let temp_dir =
        std::env::temp_dir().join(format!("daypress_static_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);

    let settings = json!({
        "provider": "folder",
        "production_url": "https://static-demo.daypress.dev",
        "local_export_dir": temp_dir.to_string_lossy(),
        "static_search_enabled": true,
        "static_redirects_enabled": true,
        "static_form_mode": "webhook",
        "static_form_webhook_url": "https://hooks.daypress.dev/lead",
        "neutralize_admin_links": true
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/settings")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(settings.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check status before publish: should report unpublished changes and preflight warning for /d-admin/
    let status_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/publish/status")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_json: Value = serde_json::from_slice(
        &axum::body::to_bytes(status_res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(!status_json["unpublished_changes"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!status_json["preflight_warnings"]
        .as_array()
        .unwrap()
        .is_empty());

    // Execute publish
    let run_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/run")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run_res.status(), StatusCode::OK);
    let run_json: Value = serde_json::from_slice(
        &axum::body::to_bytes(run_res.into_body(), 10 * 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(run_json["success"], true);
    let dep = &run_json["deployment"];
    assert_eq!(dep["status"], "live");
    let deploy_id = dep["id"].as_i64().unwrap();
    assert!(dep["pages_count"].as_i64().unwrap() >= 3);

    // Verify exported files on disk
    let pricing_html = std::fs::read_to_string(temp_dir.join("pricing/index.html")).unwrap();
    // 1. Zero admin chrome leaked
    assert!(!pricing_html.contains("id=\"wpadminbar\""));
    assert!(!pricing_html.contains("id=\"daypress-welcome-dock\""));
    // 2. Localhost rewritten to canonical production URL
    assert!(pricing_html.contains("https://static-demo.daypress.dev/about"));
    assert!(!pricing_html.contains("http://localhost:8090"));
    // 3. /d-admin/ link stripped safely
    assert!(pricing_html.contains("data-daypress-admin-link-stripped=\"true\""));
    // 4. Form transformed to static webhook + client search script injected
    assert!(pricing_html.contains("data-static-webhook=\"https://hooks.daypress.dev/lead\""));
    assert!(pricing_html.contains("/static/daypress-static-search.js"));

    // 5. Static-ready enhancement artifacts exist
    let redirects = std::fs::read_to_string(temp_dir.join("_redirects")).unwrap();
    assert!(redirects.contains("/old-pricing /pricing 301"));
    assert!(temp_dir.join("404.html").exists());
    assert!(temp_dir.join("sitemap.xml").exists());
    assert!(temp_dir.join("robots.txt").exists());
    assert!(temp_dir.join("search-index.json").exists());
    assert!(temp_dir.join("static/daypress-static-search.js").exists());

    // 6. Status after publish should have 0 unpublished changes
    let status_after_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/publish/status")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_after: Value = serde_json::from_slice(
        &axum::body::to_bytes(status_after_res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        status_after["unpublished_changes"]
            .as_array()
            .unwrap()
            .len(),
        0,
        "Unpublished changes queue should be empty immediately after deploy"
    );

    // 7. Download the ZIP bundle and verify PKZIP signature
    let zip_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/wp-json/daypress/v1/publish/deployments/{deploy_id}/download"
                ))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(zip_res.status(), StatusCode::OK);
    let zip_bytes = axum::body::to_bytes(zip_res.into_body(), 50 * 1024 * 1024)
        .await
        .unwrap();
    assert!(zip_bytes.starts_with(b"PK\x03\x04"));
    let unpacked = daypress_server::publisher::archive::read_zip_archive(&zip_bytes).unwrap();
    assert!(unpacked.contains_key("pricing/index.html"));
    assert!(unpacked.contains_key("_redirects"));
    assert!(unpacked.contains_key("search-index.json"));

    // 8. Edit the Pricing page and confirm it re-appears in unpublished_changes as "Modified"
    std::thread::sleep(std::time::Duration::from_millis(1100));
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/wp/v2/pages/{pricing_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "title": "Pricing & Enterprise Plans" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status_edited_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/publish/status")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_edited: Value = serde_json::from_slice(
        &axum::body::to_bytes(status_edited_res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    let changes = status_edited["unpublished_changes"].as_array().unwrap();
    assert!(changes
        .iter()
        .any(|c| c["slug_or_path"] == "/pricing/" && c["change_type"] == "modified"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_cloudflare_pages_auto_dns_netlify_digest_and_one_click_rollback() {
    // Spin up a local mock Edge Provider API server handling both Cloudflare and Netlify endpoints
    let recorded_calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_ref = recorded_calls.clone();

    let mock_router = Router::new()
        .route(
            "/accounts/{acc}/pages/projects/{proj}",
            get({
                let c = calls_ref.clone();
                move || {
                    c.lock().unwrap().push("cf_get_project".into());
                    async { Json(json!({ "success": true, "result": { "name": "demo-proj" } })) }
                }
            }),
        )
        .route(
            "/accounts/{acc}/pages/projects/{proj}/deployments",
            post({
                let c = calls_ref.clone();
                move |req: axum::extract::Request| {
                    c.lock().unwrap().push("cf_create_deployment".into());
                    async move {
                        let _ = axum::body::to_bytes(req.into_body(), 100 * 1024 * 1024).await;
                        Json(json!({
                            "success": true,
                            "result": {
                                "id": "cf_deploy_999",
                                "url": "https://999.demo-proj.pages.dev"
                            }
                        }))
                    }
                }
            }),
        )
        .route(
            "/accounts/{acc}/pages/projects/{proj}/domains",
            post({
                let c = calls_ref.clone();
                move || {
                    c.lock().unwrap().push("cf_bind_domain".into());
                    async { Json(json!({ "success": true, "result": { "name": "www.acme.com" } })) }
                }
            }),
        )
        .route(
            "/zones",
            get({
                let c = calls_ref.clone();
                move || {
                    c.lock().unwrap().push("cf_list_zones".into());
                    async {
                        Json(json!({
                            "success": true,
                            "result": [{ "id": "zone_acme_1", "name": "acme.com" }]
                        }))
                    }
                }
            }),
        )
        .route(
            "/zones/{zone_id}/dns_records",
            post({
                let c = calls_ref.clone();
                move || {
                    c.lock().unwrap().push("cf_create_cname".into());
                    async { Json(json!({ "success": true, "result": { "id": "dns_1" } })) }
                }
            }),
        )
        // Netlify mock endpoints
        .route(
            "/sites/{site_id}/deploys",
            post({
                let c = calls_ref.clone();
                move |req: axum::extract::Request| {
                    c.lock().unwrap().push("nl_create_deploy".into());
                    async move {
                        let _ = axum::body::to_bytes(req.into_body(), 100 * 1024 * 1024).await;
                        Json(json!({
                            "id": "nl_deploy_777",
                            "required": [],
                            "ssl_url": "https://demo-netlify.netlify.app"
                        }))
                    }
                }
            }),
        )
        .route(
            "/sites/{site_id}",
            axum::routing::patch({
                let c = calls_ref.clone();
                move || {
                    c.lock().unwrap().push("nl_patch_domain".into());
                    async { Json(json!({ "id": "nl_site_1" })) }
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mock_router).await.unwrap();
    });

    let base_url = format!("http://{addr}");
    std::env::set_var("DAYPRESS_CF_API_BASE", &base_url);
    std::env::set_var("DAYPRESS_NETLIFY_API_BASE", &base_url);

    let (app, _db, cookie) = setup_authenticated_app();

    // 1. Configure Cloudflare Pages for www.acme.com
    let cf_settings = json!({
        "provider": "cloudflare",
        "production_url": "https://www.acme.com",
        "cf_account_id": "acc_1",
        "cf_project_name": "demo-proj",
        "cf_api_token": "tok_cf_12345678",
        "cf_custom_domain": "www.acme.com"
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/settings")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(cf_settings.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Trigger 1-Click Auto-DNS for www.acme.com
    let dom_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/domain/configure")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "domain": "www.acme.com", "provider": "cloudflare" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dom_res.status(), StatusCode::OK);
    let dom_json: Value = serde_json::from_slice(
        &axum::body::to_bytes(dom_res.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(dom_json["auto_dns_provisioned"], true);
    assert_eq!(dom_json["record_target"], "demo-proj.pages.dev");

    // Run Cloudflare deploy
    let cf_run = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/run")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    let cf_status = cf_run.status();
    let cf_body_bytes = axum::body::to_bytes(cf_run.into_body(), 5 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(
        cf_status,
        StatusCode::OK,
        "cf_run failed: {}",
        String::from_utf8_lossy(&cf_body_bytes)
    );
    let cf_deploy_json: Value = serde_json::from_slice(&cf_body_bytes).unwrap();
    let first_deploy_id = cf_deploy_json["deployment"]["id"].as_i64().unwrap();
    assert_eq!(cf_deploy_json["deployment"]["provider"], "cloudflare");
    assert_eq!(cf_deploy_json["deployment"]["status"], "live");

    {
        let calls = recorded_calls.lock().unwrap();
        assert!(calls.contains(&"cf_get_project".to_string()));
        assert!(calls.contains(&"cf_create_deployment".to_string()));
        assert!(calls.contains(&"cf_bind_domain".to_string()));
        assert!(calls.contains(&"cf_list_zones".to_string()));
        assert!(calls.contains(&"cf_create_cname".to_string()));
    }

    // 2. Configure and run Netlify deploy + domain bind
    let nl_settings = json!({
        "provider": "netlify",
        "production_url": "https://www.acme.com",
        "netlify_site_id": "nl_site_1",
        "netlify_token": "tok_nl_12345678",
        "netlify_custom_domain": "www.acme.com"
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/settings")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(nl_settings.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/domain/configure")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "domain": "www.acme.com", "provider": "netlify" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let nl_run = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/publish/run")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(nl_run.status(), StatusCode::OK);
    {
        let calls = recorded_calls.lock().unwrap();
        assert!(calls.contains(&"nl_create_deploy".to_string()));
        assert!(calls.contains(&"nl_patch_domain".to_string()));
    }

    // 3. Test 1-Click Rollback to the first deployment snapshot
    let rb_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/wp-json/daypress/v1/publish/rollback/{first_deploy_id}"
                ))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rb_res.status(), StatusCode::OK);
    let rb_json: Value = serde_json::from_slice(
        &axum::body::to_bytes(rb_res.into_body(), 5 * 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(rb_json["deployment"]["status"], "live");
    assert!(rb_json["deployment"]["commit_message"]
        .as_str()
        .unwrap()
        .contains(&format!("Rollback to Deployment #{}", first_deploy_id)));
}
