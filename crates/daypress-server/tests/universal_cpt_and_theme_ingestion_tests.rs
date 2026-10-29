//! Empirical Verification Test Suite for Milestone 1:
//! Universal Theme/ACF Ingestion & Zero Site-Specific Hardcoding.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
use daypress_server::{create_app, Database};
use serde_json::json;
use std::io::Cursor;
use tower::ServiceExt;

const ARBITRARY_CPT_WXR: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>Universal Corp</title>
	<description>Arbitrary Custom Post Types Test</description>
	<wp:wxr_version>1.2</wp:wxr_version>

	<wp:term>
		<wp:term_id>101</wp:term_id>
		<wp:term_taxonomy>portfolio_category</wp:term_taxonomy>
		<wp:term_slug>commercial</wp:term_slug>
		<wp:term_name><![CDATA[Commercial]]></wp:term_name>
	</wp:term>
	<wp:term>
		<wp:term_id>201</wp:term_id>
		<wp:term_taxonomy>event_type</wp:term_taxonomy>
		<wp:term_slug>keynote</wp:term_slug>
		<wp:term_name><![CDATA[Keynote]]></wp:term_name>
	</wp:term>
	<wp:term>
		<wp:term_id>301</wp:term_id>
		<wp:term_taxonomy>service_tier</wp:term_taxonomy>
		<wp:term_slug>enterprise</wp:term_slug>
		<wp:term_name><![CDATA[Enterprise]]></wp:term_name>
	</wp:term>

	<!-- CPT 1: portfolio -->
	<item>
		<title>DayDesign System</title>
		<wp:post_id>1001</wp:post_id>
		<wp:post_name>day-design-system</wp:post_name>
		<wp:post_type>portfolio</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>High performance 2026 design system showcase.</p>]]></content:encoded>
		<excerpt:encoded><![CDATA[Design system showcase excerpt.]]></excerpt:encoded>
		<category domain="portfolio_category" nicename="commercial"><![CDATA[Commercial]]></category>
		<wp:postmeta>
			<wp:meta_key>banner_image</wp:meta_key>
			<wp:meta_value><![CDATA[/uploads/day-design-banner.webp]]></wp:meta_value>
		</wp:postmeta>
	</item>

	<!-- CPT 2: events -->
	<item>
		<title>Rust Global Summit 2026</title>
		<wp:post_id>1002</wp:post_id>
		<wp:post_name>rust-global-summit-2026</wp:post_name>
		<wp:post_type>events</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Annual worldwide Rust performance summit.</p>]]></content:encoded>
		<excerpt:encoded><![CDATA[Global Rust summit details.]]></excerpt:encoded>
		<category domain="event_type" nicename="keynote"><![CDATA[Keynote]]></category>
		<wp:postmeta>
			<wp:meta_key>event_location</wp:meta_key>
			<wp:meta_value><![CDATA[Amsterdam]]></wp:meta_value>
		</wp:postmeta>
	</item>

	<!-- CPT 3: services -->
	<item>
		<title>Cloud Architecture Migration</title>
		<wp:post_id>1003</wp:post_id>
		<wp:post_name>cloud-migration</wp:post_name>
		<wp:post_type>services</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Sub-millisecond native Rust CMS migration service.</p>]]></content:encoded>
		<excerpt:encoded><![CDATA[Native Rust CMS migration.]]></excerpt:encoded>
		<category domain="service_tier" nicename="enterprise"><![CDATA[Enterprise]]></category>
	</item>
</channel>
</rss>"#;

async fn body_to_string(res: axum::response::Response) -> String {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn make_custom_layout(id: &str, html: &str) -> DaypressLayout {
    DaypressLayout {
        version: 1,
        root: DaypressNode {
            id: id.to_string(),
            kind: NodeKind::CustomHtml {
                raw: html.to_string(),
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
        global_styles: None,
    }
}

#[tokio::test]
async fn test_universal_cpt_ingestion_and_permalink_generation() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let import_res = db
        .import_wxr(Cursor::new(ARBITRARY_CPT_WXR))
        .expect("WXR import should succeed");
    assert_eq!(import_res.imported_posts, 3);

    let portfolio_post = db
        .get_post_by_slug("day-design-system")
        .expect("db query")
        .expect("portfolio item found");
    assert_eq!(portfolio_post.post_type, "portfolio");
    assert_eq!(portfolio_post.slug, "day-design-system");

    let event_post = db
        .get_post_by_slug("rust-global-summit-2026")
        .expect("db query")
        .expect("event item found");
    assert_eq!(event_post.post_type, "events");

    let service_post = db
        .get_post_by_slug("cloud-migration")
        .expect("db query")
        .expect("service item found");
    assert_eq!(service_post.post_type, "services");
}

#[tokio::test]
async fn test_universal_cpt_archive_ssr_rendering() {
    let db = Database::new_in_memory().expect("in-memory DB");
    db.import_wxr(Cursor::new(ARBITRARY_CPT_WXR))
        .expect("WXR import should succeed");
    let app = create_app(db.clone());

    // 1. Query /portfolio/ archive route
    let req = Request::builder()
        .uri("/portfolio/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_to_string(res).await;
    assert!(html.contains("<h1 class=\"archive-title\">Portfolio</h1>"));
    assert!(html.contains("href=\"/portfolio/day-design-system/\""));
    assert!(html.contains("DayDesign System"));
    assert!(html.contains("Design system showcase excerpt"));
    assert!(html.contains("src=\"/uploads/day-design-banner.webp\""));

    // 2. Query /events/ archive route
    let req = Request::builder()
        .uri("/events/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_to_string(res).await;
    assert!(html.contains("<h1 class=\"archive-title\">Events</h1>"));
    assert!(html.contains("href=\"/events/rust-global-summit-2026/\""));
    assert!(html.contains("Rust Global Summit 2026"));

    // 3. Query /services/ archive route
    let req = Request::builder()
        .uri("/services/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_to_string(res).await;
    assert!(html.contains("<h1 class=\"archive-title\">Services</h1>"));
    assert!(html.contains("href=\"/services/cloud-migration/\""));
    assert!(html.contains("Cloud Architecture Migration"));

    // 4. Query Nonexistent CPT archive -> 404
    let req = Request::builder()
        .uri("/nonexistent-cpt/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cpt_single_post_routes_ssr() {
    let db = Database::new_in_memory().expect("in-memory DB");
    db.import_wxr(Cursor::new(ARBITRARY_CPT_WXR))
        .expect("WXR import should succeed");
    let app = create_app(db.clone());

    // 1. Single CPT: /portfolio/day-design-system/
    let req = Request::builder()
        .uri("/portfolio/day-design-system/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_to_string(res).await;
    assert!(html.contains("DayDesign System"));
    assert!(html.contains("High performance 2026 design system showcase."));

    // 2. Single CPT: /events/rust-global-summit-2026/
    let req = Request::builder()
        .uri("/events/rust-global-summit-2026/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_to_string(res).await;
    assert!(html.contains("Rust Global Summit 2026"));

    // 3. Nonexistent CPT post -> 404
    let req = Request::builder()
        .uri("/portfolio/non-existent-item/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_theme_header_footer_resolution_across_root_and_cpt_archives() {
    let db = Database::new_in_memory().expect("in-memory DB");
    db.import_wxr(Cursor::new(ARBITRARY_CPT_WXR))
        .expect("WXR import should succeed");
    let app = create_app(db.clone());

    let header_payload = json!({
        "title": "Challenger Theme Header",
        "template_type": "header",
        "status": "publish",
        "priority": 100,
        "is_active": true,
        "layout": make_custom_layout("hdr_root", "<div class=\"challenger-header-brand\">⚡ DAY-CHALLENGER-HEADER-2026</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "entire_site", "priority": 100 }]
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/templates")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(header_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let footer_payload = json!({
        "title": "Challenger Theme Footer",
        "template_type": "footer",
        "status": "publish",
        "priority": 100,
        "is_active": true,
        "layout": make_custom_layout("ftr_root", "<div class=\"challenger-footer-brand\">🔒 DAY-CHALLENGER-FOOTER-2026</div>"),
        "conditions": [{ "condition_type": "include", "rule_target": "entire_site", "priority": 100 }]
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/templates")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(footer_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 1. Verify Root Route (`/`) resolves both Header and Footer
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let root_html = body_to_string(res).await;
    assert!(
        root_html.contains("dp-theme-header"),
        "Root must contain .dp-theme-header"
    );
    assert!(
        root_html.contains("⚡ DAY-CHALLENGER-HEADER-2026"),
        "Root must render Challenger Theme Header"
    );
    assert!(
        root_html.contains("dp-theme-footer"),
        "Root must contain .dp-theme-footer"
    );
    assert!(
        root_html.contains("🔒 DAY-CHALLENGER-FOOTER-2026"),
        "Root must render Challenger Theme Footer"
    );

    // 2. Verify CPT Archive Route (`/portfolio/`) resolves both Header and Footer
    let req = Request::builder()
        .uri("/portfolio/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let portfolio_html = body_to_string(res).await;
    assert!(portfolio_html.contains("dp-theme-header"));
    assert!(portfolio_html.contains("⚡ DAY-CHALLENGER-HEADER-2026"));
    assert!(portfolio_html.contains("dp-theme-footer"));
    assert!(portfolio_html.contains("🔒 DAY-CHALLENGER-FOOTER-2026"));

    // 3. Verify CPT Archive Route (`/events/`) resolves both Header and Footer
    let req = Request::builder()
        .uri("/events/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let events_html = body_to_string(res).await;
    assert!(events_html.contains("⚡ DAY-CHALLENGER-HEADER-2026"));
    assert!(events_html.contains("🔒 DAY-CHALLENGER-FOOTER-2026"));

    // 4. Verify Single CPT Route (`/portfolio/day-design-system/`) resolves both Header and Footer
    let req = Request::builder()
        .uri("/portfolio/day-design-system/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let single_html = body_to_string(res).await;
    assert!(single_html.contains("⚡ DAY-CHALLENGER-HEADER-2026"));
    assert!(single_html.contains("🔒 DAY-CHALLENGER-FOOTER-2026"));
}

#[tokio::test]
async fn test_cpt_archive_dynamic_taxonomy_hydration() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let app = create_app(db.clone());

    // 1. Create a category term
    let cat_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/categories")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(
                    json!({ "name": "Mobile Engineering", "slug": "mobile-engineering" })
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cat_res.status(), StatusCode::CREATED);
    let cat_bytes = to_bytes(cat_res.into_body(), usize::MAX).await.unwrap();
    let cat_json: serde_json::Value = serde_json::from_slice(&cat_bytes).unwrap();
    let cat_id = cat_json["id"].as_i64().unwrap();

    // 2. Create a CPT post associated with this category
    let post_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(
                    json!({
                        "title": "FinTech iOS App",
                        "slug": "fintech-ios-app",
                        "content": "<p>FinTech banking application.</p>",
                        "status": "publish",
                        "post_type": "portfolio",
                        "categories": [cat_id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(post_res.status(), StatusCode::CREATED);

    // 3. Request /portfolio/ archive SSR
    let req = Request::builder()
        .uri("/portfolio/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_to_string(res).await;

    // Verify dynamic taxonomy facet filters are hydrated!
    assert!(
        html.contains("filters_desktop"),
        "Must render desktop filters container"
    );
    assert!(
        html.contains("Category"),
        "Must render Category facet group"
    );
    assert!(
        html.contains("data-value=\"mobile-engineering\""),
        "Must render term slug data-value"
    );
    assert!(
        html.contains("Mobile Engineering"),
        "Must render term display name"
    );
    assert!(html.contains("FinTech iOS App"), "Must render card title");
}
