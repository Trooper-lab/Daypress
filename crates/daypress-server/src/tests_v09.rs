use super::*;
use crate::adapters::mcp::{handle_mcp_rpc, JsonRpcRequest};
use serde_json::{json, Value};

#[tokio::test]
async fn test_v09_pillar1_newsletters_member_gate_and_auto_digests() {
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());

    // 1. Create a subscriber-gated post (`member_gate: "subscribers_only"`) with a `[daypress_newsletter]` shortcode on a public post
    let gated_post = db
        .create_post(&CreatePostRequest {
            title: "Deep Rust Architecture Secrets".to_string(),
            slug: Some("rust-secrets".to_string()),
            content:
                "<p>Public teaser paragraph.</p><!--more--><p>SECRET_SUBSCRIBER_PAYWALL_BODY_42</p>"
                    .to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            member_gate: Some("subscribers_only".to_string()),
            ..Default::default()
        })
        .expect("create gated post");
    assert_eq!(gated_post.member_gate, "subscribers_only");

    let public_post = db
            .create_post(&CreatePostRequest {
                title: "Public Newsletter Landing".to_string(),
                slug: Some("join-newsletter".to_string()),
                content: r#"<p>Welcome!</p>[daypress_newsletter title="Join the Dispatch" list="general" button="Join Now"]"#.to_string(),
                status: "publish".to_string(),
                post_type: "page".to_string(),
                ..Default::default()
            })
            .expect("create public page with newsletter shortcode");

    // Verify `[daypress_newsletter]` shortcode expands on public SSR
    let res_landing = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{}/", public_post.slug))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_landing.status(), StatusCode::OK);
    let landing_html = String::from_utf8(
        to_bytes(res_landing.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(landing_html.contains("dp-newsletter-box"));
    assert!(landing_html.contains("Join the Dispatch"));

    // 2. Anonymous visitor requests gated post -> sees teaser + Member Gate card, NOT the secret body
    let res_anon = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/rust-secrets/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_anon.status(), StatusCode::OK);
    let anon_html = String::from_utf8(
        to_bytes(res_anon.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(anon_html.contains("Public teaser paragraph."));
    assert!(anon_html.contains("dp-member-gate-card"));
    assert!(
        !anon_html.contains("SECRET_SUBSCRIBER_PAYWALL_BODY_42"),
        "Anonymous visitor must not see subscriber-gated content"
    );

    // 3. Visitor subscribes via `POST /wp-json/daypress/v1/newsletters/subscribe` (Double Opt-In enabled by default)
    let sub_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/newsletters/subscribe")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "alice@example.com",
                        "name": "Alice Rustacean",
                        "lists": ["general"],
                        "source_url": "/rust-secrets/"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub_res.status(), StatusCode::OK);
    let sub_json: Value =
        serde_json::from_slice(&to_bytes(sub_res.into_body(), 64 * 1024).await.unwrap()).unwrap();
    assert_eq!(sub_json["status"], "pending");
    assert_eq!(sub_json["unlocked"], false);

    // Fetch the pending subscriber from SQLite to get `confirm_token`
    let subs = db
        .list_subscribers(Some("pending"), None, Some("alice@example.com"), true)
        .expect("list pending subscribers");
    assert_eq!(subs.len(), 1);
    let confirm_token = subs[0].confirm_token.clone().expect("confirm_token");
    let unsub_token = subs[0]
        .unsubscribe_token
        .clone()
        .expect("unsubscribe_token");

    // 4. Visitor clicks `/newsletter/confirm/{token}` -> status becomes `subscribed` & sets `daypress_subscriber` cookie
    let confirm_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/newsletter/confirm/{}", confirm_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(confirm_res.status(), StatusCode::OK);
    let set_cookie = confirm_res
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .expect("Set-Cookie header on confirmation")
        .to_string();
    assert!(set_cookie.contains("daypress_subscriber="));

    // 5. Confirmed subscriber requests `/rust-secrets/` with `Cookie: daypress_subscriber=<token>` -> FULL UNLOCKED CONTENT!
    let cookie_pair = set_cookie.split(';').next().unwrap().to_string();
    let res_unlocked = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/rust-secrets/")
                .header(header::COOKIE, &cookie_pair)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_unlocked.status(), StatusCode::OK);
    let unlocked_html = String::from_utf8(
        to_bytes(res_unlocked.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        unlocked_html.contains("SECRET_SUBSCRIBER_PAYWALL_BODY_42"),
        "Confirmed subscriber with cookie must see full gated post body"
    );
    assert!(!unlocked_html.contains("dp-member-gate-card"));

    // 6. Publish a new post with `send_newsletter_digest: Some(true)` -> automatically sends digest campaign to Alice!
    let digest_post = db
        .create_post(&CreatePostRequest {
            title: "Daypress 0.9 Release Notes".to_string(),
            slug: Some("daypress-0-9-release".to_string()),
            content: "<p>Built-in newsletters, webhooks, A/B testing, and Wasm plugins!</p>"
                .to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            send_newsletter_digest: Some(true),
            ..Default::default()
        })
        .expect("create post with auto digest");
    let campaigns = db.list_newsletter_campaigns().expect("list campaigns");
    assert!(
        campaigns.iter().any(|c| c.post_id == Some(digest_post.id)
            && c.campaign_type == "digest"
            && c.status == "sent"
            && c.sent_count == 1),
        "Automated new-post digest campaign must be created and sent to 1 confirmed subscriber"
    );

    // 7. Verify MCP tools `daypress_list_subscribers` and `daypress_create_newsletter_campaign`
    let service = crate::application::CmsService::from_adapter(db.clone());
    let mcp_subs = handle_mcp_rpc(
        &service,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(901)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_list_subscribers",
                "arguments": { "status": "subscribed" }
            }),
        },
    );
    let mcp_subs_payload: Value = serde_json::from_str(
        mcp_subs.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(mcp_subs_payload["stats"]["confirmed_subscribers"], 1);

    let mcp_camp = handle_mcp_rpc(
        &service,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(902)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_create_newsletter_campaign",
                "arguments": {
                    "subject": "Weekly Rust Dispatch #1",
                    "body_html": "<p>Hello from MCP!</p>",
                    "target_list": "general",
                    "send_now": true
                }
            }),
        },
    );
    let mcp_camp_payload: Value = serde_json::from_str(
        mcp_camp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(mcp_camp_payload["status"], "sent");
    assert_eq!(mcp_camp_payload["sent_count"], 1);

    // 8. 1-Click Unsubscribe via `POST /newsletter/unsubscribe/{token}` revokes access
    let unsub_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/newsletter/unsubscribe/{}", unsub_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unsub_res.status(), StatusCode::OK);

    // Requesting gated post with the old token after unsubscribing is now locked again
    let res_after_unsub = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/rust-secrets/")
                .header(header::COOKIE, &cookie_pair)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let after_unsub_html = String::from_utf8(
        to_bytes(res_after_unsub.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        !after_unsub_html.contains("SECRET_SUBSCRIBER_PAYWALL_BODY_42"),
        "Unsubscribed token must no longer unlock gated post"
    );
}

#[tokio::test]
async fn test_v09_pillar2_shortlinks_and_hmac_webhooks() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let cms = crate::application::services::CmsService::from_adapter(db.clone());

    // 1. Create a branded `/go/partner-cloud` shortlink via MCP `daypress_manage_shortlinks`
    let mcp_sl = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(201)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_shortlinks",
                "arguments": {
                    "action": "create",
                    "slug": "partner-cloud",
                    "target_url": "https://cloud.example.com/offer",
                    "title": "Partner Cloud Deal",
                    "status_code": 307,
                    "utm_source": "daypress",
                    "utm_medium": "affiliate",
                    "utm_campaign": "spring_launch"
                }
            }),
        },
    );
    let sl_created: crate::models::ShortlinkRecord = serde_json::from_str(
        mcp_sl.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(sl_created.slug, "partner-cloud");
    assert_eq!(sl_created.status_code, 307);

    // 2. Hit `GET /go/partner-cloud` from two distinct visitors and verify redirect + UTM + X-Robots-Tag + click counts
    let go_res1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/go/partner-cloud")
                .header(header::USER_AGENT, "VisitorOne/1.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(go_res1.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
            go_res1
                .headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("https://cloud.example.com/offer?utm_source=daypress&utm_medium=affiliate&utm_campaign=spring_launch")
        );
    assert_eq!(
        go_res1
            .headers()
            .get("X-Robots-Tag")
            .and_then(|v| v.to_str().ok()),
        Some("nofollow, sponsored")
    );

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/go/partner-cloud")
                .header(header::USER_AGENT, "VisitorOne/1.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/go/partner-cloud")
                .header(header::USER_AGENT, "VisitorTwo/2.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let links = db.list_shortlinks().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].clicks, 3);

    // 3. Register an HMAC-signed outbound webhook endpoint (`preset: "slack"`, `mock://` transport for deterministic testing)
    let wh = db
        .save_webhook_endpoint(&crate::models::CreateWebhookEndpointRequest {
            id: None,
            name: "Slack Marketing Channel".to_string(),
            target_url: "mock://hooks.slack.local/services/T000/B000/X000".to_string(),
            secret: "whsec_daypress_unit_test_secret".to_string(),
            events: vec![
                "post.published".to_string(),
                "subscriber.confirmed".to_string(),
            ],
            preset: "slack".to_string(),
            is_active: true,
        })
        .unwrap();
    assert_eq!(wh.preset, "slack");

    // 4. Publishing a new post automatically triggers `post.published` outbound webhook
    let published_post = db
        .create_post(&CreatePostRequest {
            title: "Daypress 0.9 Outbound Webhooks".to_string(),
            content: "<p>Real-time event streaming.</p>".to_string(),
            status: "publish".to_string(),
            slug: Some("daypress-09-webhooks".to_string()),
            post_type: "post".to_string(),
            send_newsletter_digest: Some(false),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(published_post.slug, "daypress-09-webhooks");

    // 5. Inspect logged webhook delivery & verify HMAC-SHA256 signature (`X-Daypress-Signature-256`)
    let deliveries = db.list_webhook_deliveries(Some(wh.id), 10).unwrap();
    assert_eq!(deliveries.len(), 1);
    let d = &deliveries[0];
    assert_eq!(d.event_name, "post.published");
    assert_eq!(d.status, "delivered");
    assert_eq!(d.response_status, 200);
    let parsed_payload: serde_json::Value = serde_json::from_str(&d.payload_json).unwrap();
    let ts = parsed_payload["timestamp"].as_i64().unwrap();
    let expected_sig = crate::adapters::database::webhooks_shortlinks::compute_webhook_hmac_sha256(
        "whsec_daypress_unit_test_secret",
        ts,
        &d.payload_json,
    );
    assert_eq!(d.signature_header, expected_sig);
    assert!(d.payload_json.contains("Daypress 0.9 Outbound Webhooks"));

    // 6. Replay the webhook delivery via MCP `daypress_manage_webhooks` (`action: "replay"`)
    let mcp_replay = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(202)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_webhooks",
                "arguments": {
                    "action": "replay",
                    "delivery_id": d.id
                }
            }),
        },
    );
    let replayed: crate::models::WebhookDeliveryRecord = serde_json::from_str(
        mcp_replay.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(replayed.id, d.id);
    assert_eq!(replayed.status, "replayed");
    assert_eq!(replayed.attempt_count, 2);
}

#[tokio::test]
async fn test_v09_pillar3_ab_testing_and_core_web_vitals() {
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let cms = crate::application::services::CmsService::from_adapter(db.clone());

    // 1. Create a landing page with Control headline
    let landing = db
        .create_post(&CreatePostRequest {
            title: "Daypress Cloud Hosting".to_string(),
            slug: Some("ab-landing".to_string()),
            content: "<h1>HERO_CONTROL_HEADLINE_ALPHA</h1><p>Sub-millisecond Rust CMS.</p>"
                .to_string(),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            ..Default::default()
        })
        .expect("create landing page");

    // 2. Create a 50/50 server-side A/B split test via MCP `daypress_manage_ab_tests`
    let mcp_create = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(301)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_ab_tests",
                "arguments": {
                    "action": "create",
                    "name": "Hero Headline Experiment",
                    "post_id": landing.id,
                    "control_text": "HERO_CONTROL_HEADLINE_ALPHA",
                    "variant_text": "HERO_VARIANT_HEADLINE_BETA",
                    "goal_event": "conversion",
                    "traffic_split": 50
                }
            }),
        },
    );
    let ab_created: crate::models::AbTestRecord = serde_json::from_str(
        mcp_create.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(ab_created.post_id, landing.id);
    assert_eq!(ab_created.status, "running");

    // 3. Simulate 40 unique visitors requesting `/ab-landing/` and record which variant each visitor received
    let mut group_a_uas = Vec::new();
    let mut group_b_uas = Vec::new();
    for i in 0..40 {
        let ua = format!("AbVisitorAgent/{}", i);
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/ab-landing/")
                    .header(header::USER_AGENT, &ua)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let html = String::from_utf8(
            to_bytes(res.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        if html.contains("HERO_VARIANT_HEADLINE_BETA") {
            assert!(!html.contains("HERO_CONTROL_HEADLINE_ALPHA"));
            assert!(html.contains(&format!("data-dp-ab=\"{}:B\"", ab_created.id)));
            group_b_uas.push(ua);
        } else {
            assert!(html.contains("HERO_CONTROL_HEADLINE_ALPHA"));
            assert!(html.contains(&format!("data-dp-ab=\"{}:A\"", ab_created.id)));
            group_a_uas.push(ua);
        }
    }
    assert!(!group_a_uas.is_empty(), "Control A should receive traffic");
    assert!(!group_b_uas.is_empty(), "Variant B should receive traffic");
    assert_eq!(group_a_uas.len() + group_b_uas.len(), 40);

    // Repeat request for first visitor to verify sticky assignment without double-counting views
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/ab-landing/")
                .header(header::USER_AGENT, &group_b_uas[0])
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 4. Trigger conversions: 1 conversion in Control A, and almost all in Variant B -> statistically significant uplift
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/analytics/event")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::USER_AGENT, &group_a_uas[0])
                .body(Body::from(
                    serde_json::json!({
                        "page_url": "/ab-landing/",
                        "event_type": "conversion",
                        "goal_name": "signup"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    for ua in group_b_uas
        .iter()
        .take(group_b_uas.len().saturating_sub(2).max(1))
    {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/daypress/v1/analytics/event")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::USER_AGENT, ua)
                    .body(Body::from(
                        serde_json::json!({
                            "page_url": "/ab-landing/",
                            "event_type": "conversion",
                            "goal_name": "signup"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let tests_after = db.list_ab_tests(Some(landing.id)).unwrap();
    let stats = &tests_after[0];
    assert_eq!(stats.control_views + stats.variant_views, 40);
    assert_eq!(stats.control_conversions, 1);
    assert!(stats.variant_conversions > 5);
    assert!(stats.variant_cr_pct > stats.control_cr_pct);
    assert!(stats.uplift_pct > 100.0);
    assert!(
        stats.confidence_pct >= 95.0,
        "Expected >=95% statistical confidence, got {}%",
        stats.confidence_pct
    );
    assert_eq!(stats.recommended_winner.as_deref(), Some("B"));

    // 5. 1-Click Promote Winner B via MCP `daypress_manage_ab_tests` (`action: "promote"`)
    let mcp_promote = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(302)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_ab_tests",
                "arguments": {
                    "action": "promote",
                    "id": ab_created.id,
                    "winner": "B"
                }
            }),
        },
    );
    let promoted: crate::models::AbTestRecord = serde_json::from_str(
        mcp_promote.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(promoted.status, "completed");
    assert_eq!(promoted.winner.as_deref(), Some("B"));

    let updated_landing = db.get_post_by_id(landing.id).unwrap().unwrap();
    assert!(
        updated_landing
            .content
            .rendered
            .contains("HERO_VARIANT_HEADLINE_BETA"),
        "Promoting Variant B must permanently update the post content in SQLite"
    );

    // 6. Record Real-User Core Web Vitals beacons and verify p75 calculation via `daypress_get_web_vitals`
    for (lcp, inp, cls, ttfb) in [
        (1100.0, 45.0, 0.01, 18.0),
        (1450.0, 70.0, 0.03, 24.0),
        (1820.0, 95.0, 0.04, 31.0),
        (2100.0, 120.0, 0.06, 40.0),
    ] {
        let v_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/daypress/v1/analytics/vitals")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "page_url": "/ab-landing/",
                            "device_type": "desktop",
                            "lcp_ms": lcp,
                            "inp_ms": inp,
                            "cls_score": cls,
                            "ttfb_ms": ttfb
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(v_res.status(), StatusCode::OK);
    }

    let mcp_vitals = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(303)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_get_web_vitals",
                "arguments": {}
            }),
        },
    );
    let vitals_summary: crate::models::CoreWebVitalsSummary = serde_json::from_str(
        mcp_vitals.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(vitals_summary.sample_count, 4);
    assert_eq!(vitals_summary.lcp_p75_ms, 1820.0);
    assert_eq!(vitals_summary.inp_p75_ms, 95.0);
    assert_eq!(vitals_summary.cls_p75, 0.04);
    assert_eq!(vitals_summary.overall_status, "good");
}

#[tokio::test]
async fn test_v09_pillar4_wasm_sandboxed_extensions() {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use base64::Engine;
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let _ = db.set_option("blogname", "Daypress Wasm Lab");
    let app = create_app(db.clone());
    let cms = crate::application::services::CmsService::from_adapter(db.clone());

    // 1. Install a `.wasm` extension WITHOUT `kv_storage` permission first
    let mcp_install_restricted = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(401)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_wasm_extensions",
                "arguments": {
                    "action": "install",
                    "slug": "wasm-hit-counter",
                    "name": "Wasm Hit Counter",
                    "shortcode": "wasm_counter",
                    "template_source": "<div class=\"wasm-counter\" data-site=\"{{option:blogname}}\">Hello {{attr:label|Visitor}} #{{kv_incr:page_hits}}</div>",
                    "capabilities": {
                        "shortcodes": true,
                        "kv_storage": false,
                        "read_options": true
                    }
                }
            }),
        },
    );
    let ext_restricted: crate::models::WasmExtensionRecord = serde_json::from_str(
        mcp_install_restricted.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(ext_restricted.slug, "wasm-hit-counter");
    assert!(ext_restricted
        .exports
        .contains(&"render_shortcode".to_string()));
    assert!(!ext_restricted.capabilities.kv_storage);

    // 2. Attempting to execute while `kv_storage` is false must trap with `Capability denied`
    let mcp_exec_denied = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(402)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_wasm_extensions",
                "arguments": {
                    "action": "execute",
                    "slug": "wasm-hit-counter",
                    "attrs": { "label": "DeniedUser" }
                }
            }),
        },
    );
    let denied_res = mcp_exec_denied.result.unwrap();
    assert_eq!(denied_res["isError"], true);
    assert!(denied_res["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Capability denied: extension is not granted 'kv_storage' permission"));

    // 3. Grant `kv_storage: true` and `read_options: true`, then verify SSR shortcode expansion & persistent KV state
    let _ = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(403)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_wasm_extensions",
                "arguments": {
                    "action": "install",
                    "slug": "wasm-hit-counter",
                    "name": "Wasm Hit Counter",
                    "shortcode": "wasm_counter",
                    "template_source": "<div class=\"wasm-counter\" data-site=\"{{option:blogname}}\">Hello {{attr:label|Visitor}} #{{kv_incr:page_hits}}</div>",
                    "capabilities": {
                        "shortcodes": true,
                        "kv_storage": true,
                        "read_options": true
                    }
                }
            }),
        },
    );

    let demo_post = db
        .create_post(&CreatePostRequest {
            title: "Wasm Shortcode Demo".to_string(),
            slug: Some("wasm-demo".to_string()),
            content: r#"<p>Testing Wasm:</p>[wasm_counter label="Explorer"]"#.to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            ..Default::default()
        })
        .expect("create wasm demo post");

    let res1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{}/", demo_post.slug))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::OK);
    let html1 = String::from_utf8(
        to_bytes(res1.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        html1.contains(
            r#"<div class="wasm-counter" data-site="Daypress Wasm Lab">Hello Explorer #1</div>"#
        ),
        "First Wasm shortcode render should output #1"
    );

    // Execute again via MCP and verify KV counter incremented to #2
    let mcp_exec_ok = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(404)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_manage_wasm_extensions",
                "arguments": {
                    "action": "execute",
                    "slug": "wasm-hit-counter",
                    "attrs": { "label": "Explorer" }
                }
            }),
        },
    );
    let exec_out: daypress_wasm::WasmExecutionOutput = serde_json::from_str(
        mcp_exec_ok.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        exec_out.output_html,
        r#"<div class="wasm-counter" data-site="Daypress Wasm Lab">Hello Explorer #2</div>"#
    );
    assert!(exec_out.fuel_consumed > 0);

    // 4. Install a malicious infinite-loop `.wasm` binary (`loop br 0 end`) with `fuel_limit = 350`
    let infinite_wasm_b64 = base64::engine::general_purpose::STANDARD
        .encode(daypress_wasm::compile_infinite_loop_wasm_binary());
    let _ = db
        .install_or_update_wasm_extension(&crate::models::InstallWasmExtensionRequest {
            slug: "wasm-infinite".to_string(),
            name: "Infinite Loop Wasm".to_string(),
            version: "1.0.0".to_string(),
            description: "Tests instruction fuel trap".to_string(),
            shortcode: "wasm_infinite".to_string(),
            capabilities: daypress_wasm::WasmCapabilities::default(),
            fuel_limit: Some(350),
            max_memory_pages: Some(4),
            wasm_base64: Some(infinite_wasm_b64),
            template_source: None,
            is_active: true,
        })
        .expect("install infinite loop wasm");

    let trap_err = db
        .execute_wasm_extension(
            "wasm-infinite",
            "render_shortcode",
            &std::collections::HashMap::new(),
            "",
        )
        .expect_err("infinite loop must trap on fuel exhaustion");
    assert!(trap_err
        .to_string()
        .contains("instruction fuel exhausted (limit: 350 instructions)"));

    // Verify public SSR gracefully isolates the trap without hanging or 500ing
    let trapped_post = db
        .create_post(&CreatePostRequest {
            title: "Trapped Wasm Post".to_string(),
            slug: Some("wasm-trapped".to_string()),
            content: "<p>Before trap</p>[wasm_infinite]<p>After trap</p>".to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            ..Default::default()
        })
        .expect("create trapped post");
    let res_trap = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{}/", trapped_post.slug))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_trap.status(), StatusCode::OK);
    let trap_html = String::from_utf8(
        to_bytes(res_trap.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(trap_html.contains("Before trap"));
    assert!(trap_html.contains("daypress-wasm [wasm-infinite] trapped"));
    assert!(trap_html.contains("After trap"));
}

#[tokio::test]
async fn test_v09_security_hardening_ssrf_csrf_and_secret_protection() {
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    let cms = crate::application::services::CmsService::from_adapter(db.clone());

    // 1. Outbound Webhook SSRF guard blocks cloud metadata, localhost, and private RFC1918 IPs
    for forbidden_url in [
        "http://169.254.169.254/latest/meta-data/",
        "http://127.0.0.1:8090/d-admin",
        "http://localhost:6379/",
        "http://10.0.0.1/internal",
        "http://192.168.1.1/router",
        "file:///etc/passwd",
    ] {
        let res = db.save_webhook_endpoint(&crate::models::CreateWebhookEndpointRequest {
            id: None,
            name: "Malicious SSRF Hook".to_string(),
            target_url: forbidden_url.to_string(),
            preset: "generic".to_string(),
            secret: String::new(),
            events: vec!["post.published".to_string()],
            is_active: true,
        });
        assert!(
            res.is_err(),
            "Expected SSRF validation to reject {}",
            forbidden_url
        );
    }

    // 2. MCP `daypress_update_option` refuses to overwrite cryptographic `auth_secret`
    let mcp_secret_attempt = crate::adapters::mcp::handle_mcp_rpc(
        &cms,
        crate::adapters::mcp::JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(serde_json::json!(999)),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "daypress_update_option",
                "arguments": {
                    "name": "auth_secret",
                    "value": "forged_known_secret"
                }
            }),
        },
    );
    let mcp_res = mcp_secret_attempt.result.expect("mcp result");
    assert_eq!(
        mcp_res["isError"], true,
        "MCP must reject overwriting auth_secret"
    );
    assert!(
        mcp_res["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("protected cryptographic secret"),
        "MCP error message should explain why auth_secret is blocked"
    );

    // 3. Baseline security headers are attached to responses
    let home_res = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        home_res
            .headers()
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    assert_eq!(
        home_res
            .headers()
            .get(header::X_FRAME_OPTIONS)
            .and_then(|v| v.to_str().ok()),
        Some("SAMEORIGIN")
    );

    // 4. CSRF protection rejects cross-site cookie-authenticated state mutations
    let secret = db.get_auth_secret().expect("auth secret");
    let session_token = crate::auth::create_session_token(1, &secret, 86400);
    let cookie_header = format!("{}={}", crate::auth::SESSION_COOKIE, session_token);

    let csrf_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header(header::COOKIE, &cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .header("sec-fetch-site", "cross-site")
                .body(Body::from(
                    serde_json::json!({
                        "title": "CSRF Forged Post",
                        "status": "publish"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        csrf_res.status(),
        StatusCode::FORBIDDEN,
        "Cross-site cookie mutation without valid origin must be rejected"
    );
}
