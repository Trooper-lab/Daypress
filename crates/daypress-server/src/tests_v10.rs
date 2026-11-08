use super::*;
use crate::adapters::mcp::{handle_mcp_rpc, JsonRpcRequest};
use base64::Engine;
use serde_json::{json, Value};

fn basic_admin_header() -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode("admin:admin");
    format!("Basic {encoded}")
}

#[tokio::test]
async fn test_v10_pillar1_security_audit_headers_ssrf_and_shortlink_hardening() {
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let auth = basic_admin_header();

    // 1. Run automated 10-point Security & Production Readiness Audit via REST API
    let res_audit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/security/audit")
                .header(header::AUTHORIZATION, &auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_audit.status(), StatusCode::OK);

    // Verify hardened baseline HTTP security headers are present on responses
    let headers = res_audit.headers();
    assert_eq!(
        headers
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    assert_eq!(
        headers
            .get(header::X_FRAME_OPTIONS)
            .and_then(|v| v.to_str().ok()),
        Some("SAMEORIGIN")
    );
    assert_eq!(
        headers
            .get("cross-origin-opener-policy")
            .and_then(|v| v.to_str().ok()),
        Some("same-origin")
    );
    assert!(headers
        .get(header::CONTENT_SECURITY_POLICY)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .contains("frame-ancestors 'self'"));
    assert!(headers
        .get("permissions-policy")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .contains("camera=()"));

    let audit_bytes = to_bytes(res_audit.into_body(), 1024 * 1024).await.unwrap();
    let audit_json: Value = serde_json::from_slice(&audit_bytes).unwrap();
    assert_eq!(audit_json["overall_status"], "production_ready");
    assert_eq!(audit_json["score"], 100);
    assert_eq!(audit_json["failed_count"], 0);
    assert_eq!(audit_json["checks"].as_array().unwrap().len(), 10);

    // 2. Verify FormStudio SSRF guard rejects loopback/metadata webhook_url on save
    let ssrf_form = CreateFormRequest {
        id: None,
        title: "SSRF Attempt Form".to_string(),
        fields: vec![],
        settings: FormSettings {
            webhook_url: "http://169.254.169.254/latest/meta-data/".to_string(),
            ..Default::default()
        },
    };
    assert!(
        db.save_form_record(&ssrf_form).is_err(),
        "FormStudio must reject cloud metadata SSRF webhook_url"
    );

    let loopback_form = CreateFormRequest {
        id: None,
        title: "Loopback Attempt Form".to_string(),
        fields: vec![],
        settings: FormSettings {
            webhook_url: "http://127.0.0.1:8080/internal".to_string(),
            ..Default::default()
        },
    };
    assert!(
        db.save_form_record(&loopback_form).is_err(),
        "FormStudio must reject loopback SSRF webhook_url"
    );

    let safe_form = CreateFormRequest {
        id: None,
        title: "Safe Webhook Form".to_string(),
        fields: vec![],
        settings: FormSettings {
            webhook_url: "https://hooks.slack.com/services/T000/B000/X000".to_string(),
            ..Default::default()
        },
    };
    assert!(db.save_form_record(&safe_form).is_ok());

    // 3. Verify Shortlinks reject protocol-relative (`//evil.com`) and CRLF injection URLs
    let open_redirect_req = crate::models::CreateShortlinkRequest {
        slug: "evil-redirect".to_string(),
        target_url: "//evil.example.com/phishing".to_string(),
        title: "Evil".to_string(),
        status_code: 302,
        utm_source: String::new(),
        utm_medium: String::new(),
        utm_campaign: String::new(),
        is_active: true,
    };
    assert!(
        db.save_shortlink(&open_redirect_req).is_err(),
        "Protocol-relative //evil.example.com must be rejected"
    );

    let crlf_req = crate::models::CreateShortlinkRequest {
        slug: "crlf-redirect".to_string(),
        target_url: "https://example.com/ok\r\nSet-Cookie: hacked=1".to_string(),
        title: "CRLF".to_string(),
        status_code: 302,
        utm_source: String::new(),
        utm_medium: String::new(),
        utm_campaign: String::new(),
        is_active: true,
    };
    assert!(
        db.save_shortlink(&crlf_req).is_err(),
        "CRLF characters in shortlink target_url must be rejected"
    );
}

#[tokio::test]
async fn test_v10_pillar2_stable_schema_v100_contract_and_wasm_abi_v1() {
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let auth = basic_admin_header();

    let res = app
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/schema/contract")
                .header(header::AUTHORIZATION, &auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let contract: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(contract["schema_version"], 100);
    assert_eq!(contract["expected_schema_version"], 100);
    assert_eq!(contract["user_version_matches"], true);
    assert_eq!(contract["backward_compatible"], true);
    assert_eq!(contract["foreign_key_violations"], 0);
    assert_eq!(contract["fts5_search_ready"], true);
    assert!(contract["missing_tables"].as_array().unwrap().is_empty());
    assert_eq!(contract["wasm_abi_version"], 1);
    assert!(
        contract["migrations_applied"].as_array().unwrap().len() >= 8,
        "Expected all 8 milestone migrations in daypress_schema_migrations"
    );
}

#[tokio::test]
async fn test_v10_pillar3_reproducible_benchmark_suite_and_mcp_tools() {
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let auth = basic_admin_header();

    // 1. Run Benchmark Suite via REST API
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/benchmarks/run")
                .header(header::AUTHORIZATION, &auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"iterations": 15}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let report: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(report["iterations_per_workload"], 15);
    assert!(report["summary_speedup_vs_wp"].as_f64().unwrap() > 1.0);
    let workloads = report["workloads"].as_array().unwrap();
    assert_eq!(workloads.len(), 5);

    // 2. Verify MCP tools `daypress_security_audit` and `daypress_run_benchmark`
    let service = CmsService::from_adapter(db.clone());
    let list_resp = handle_mcp_rpc(
        &service,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(1)),
            method: "tools/list".into(),
            params: json!({}),
        },
    );
    let tools = list_resp.result.unwrap()["tools"]
        .as_array()
        .unwrap()
        .clone();
    assert!(tools.iter().any(|t| t["name"] == "daypress_security_audit"));
    assert!(tools.iter().any(|t| t["name"] == "daypress_run_benchmark"));

    let mcp_audit = handle_mcp_rpc(
        &service,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_security_audit",
                "arguments": {}
            }),
        },
    );
    let mcp_audit_res = mcp_audit.result.unwrap();
    assert_eq!(mcp_audit_res["isError"], false);
    let parsed_audit: Value =
        serde_json::from_str(mcp_audit_res["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        parsed_audit["security_audit"]["overall_status"],
        "production_ready"
    );
    assert_eq!(
        parsed_audit["migration_corpus_signoff"]["passes_v1_signoff"],
        true
    );

    let mcp_bench = handle_mcp_rpc(
        &service,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(3)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_run_benchmark",
                "arguments": { "iterations": 10 }
            }),
        },
    );
    let mcp_bench_res = mcp_bench.result.unwrap();
    assert_eq!(mcp_bench_res["isError"], false);
}

#[tokio::test]
async fn test_v10_pillar4_migration_corpus_signoff_and_site_health_ui() {
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let auth = basic_admin_header();

    // 1. Verify 70%+ WordPress 1-Click Migration Corpus Sign-Off across all 4 archetypes
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/migration/signoff")
                .header(header::AUTHORIZATION, &auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let signoff: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(signoff["passes_v1_signoff"], true);
    assert_eq!(signoff["archetypes_tested"], 4);
    assert_eq!(signoff["archetypes_passing"], 4);
    assert!(
        signoff["aggregate_fidelity_pct"].as_f64().unwrap() >= 70.0,
        "Aggregate fidelity must exceed 70% threshold"
    );

    // 2. Verify `/d-admin/site-health` renders live v1.0.0-beta.1 readiness dashboard
    let secret = db.get_auth_secret().expect("auth secret");
    let session_token = crate::auth::create_session_token(1, &secret, 86400);
    let session_cookie = format!("{}={}", crate::auth::SESSION_COOKIE, session_token);
    let res_health = app
        .oneshot(
            Request::builder()
                .uri("/d-admin/site-health")
                .header(header::COOKIE, session_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_health.status(), StatusCode::OK);
    let html = String::from_utf8(
        to_bytes(res_health.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(html.contains("Site Health &amp; 1.0 Production Readiness"));
    assert!(html.contains(env!("CARGO_PKG_VERSION")));
    assert!(!html.contains("Daypress v0.1.0"));
}
