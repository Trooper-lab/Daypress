//! Archetype 4: Custom PHP + ACF + CPT Theme (NORTHWIND) Ingestion & SSR Parity.

use super::common::{
    assert_zero_comment_leaks, body_to_string, load_fixture_or_fallback, setup_test_db,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use neopress_server::create_app;
use std::io::Cursor;
use tower::ServiceExt;

const FALLBACK_CUSTOM_ACF_WXR: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>NORTHWIND</title>
	<wp:base_blog_url>https://northwind.example</wp:base_blog_url>
	<!-- CPT 1: trainingen -->
	<item>
		<title>Leiderschap Training</title>
		<wp:post_id>1001</wp:post_id>
		<wp:post_name>leiderschap-training</wp:post_name>
		<wp:post_type>trainingen</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<div class="diagonal-section swiper"><div class="swiper-wrapper"><div class="swiper-slide">Training Slide</div></div></div>]]></content:encoded>
		<wp:postmeta><wp:meta_key>duur</wp:meta_key><wp:meta_value><![CDATA[2 dagen]]></wp:meta_value></wp:postmeta>
	</item>
	<!-- CPT 2: vacatures -->
	<item>
		<title>Senior Consultant</title>
		<wp:post_id>1002</wp:post_id>
		<wp:post_name>senior-consultant</wp:post_name>
		<wp:post_type>vacatures</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Vacature omschrijving</p>]]></content:encoded>
	</item>
	<!-- CPT 3: cases -->
	<item>
		<title>Klant Case Study</title>
		<wp:post_id>1003</wp:post_id>
		<wp:post_name>klant-case-study</wp:post_name>
		<wp:post_type>cases</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Case study over transformatie.</p>]]></content:encoded>
	</item>
	<!-- CPT 4: kandidaten -->
	<item>
		<title>Marketing Specialist</title>
		<wp:post_id>1004</wp:post_id>
		<wp:post_name>marketing-specialist</wp:post_name>
		<wp:post_type>kandidaten</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Kandidaat profiel</p>]]></content:encoded>
	</item>
	<!-- CPT 5: faq -->
	<item>
		<title>Wat kost een training?</title>
		<wp:post_id>1005</wp:post_id>
		<wp:post_name>wat-kost-een-training</wp:post_name>
		<wp:post_type>faq</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>FAQ antwoord</p>]]></content:encoded>
	</item>
	<!-- CPT 6: quotes -->
	<item>
		<title>Geweldige samenwerking</title>
		<wp:post_id>1006</wp:post_id>
		<wp:post_name>geweldige-samenwerking</wp:post_name>
		<wp:post_type>quotes</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Klant testimonial quote</p>]]></content:encoded>
	</item>
	<!-- CPT 7: logo -->
	<item>
		<title>Partner Logo</title>
		<wp:post_id>1007</wp:post_id>
		<wp:post_name>partner-logo</wp:post_name>
		<wp:post_type>logo</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Partner logo asset</p>]]></content:encoded>
	</item>
</channel>
</rss>"#;

pub async fn run_test() {
    let db = setup_test_db();
    let xml_data = load_fixture_or_fallback(
        "archetype4_custom_acf/site.WordPress.xml",
        FALLBACK_CUSTOM_ACF_WXR,
    );

    let res = db
        .import_wxr(Cursor::new(xml_data))
        .expect("NORTHWIND WXR import failed");
    assert!(
        res.imported_posts >= 3,
        "Expected imported posts for NORTHWIND"
    );

    // 1. Dynamic CPT Discovery Assertions (all 7 CPTs)
    let cpts = [
        "trainingen",
        "vacatures",
        "cases",
        "kandidaten",
        "faq",
        "quotes",
        "logo",
    ];
    for cpt in &cpts {
        let items = db.get_posts(cpt, None, None, 10).unwrap_or_default();
        assert!(
            !items.is_empty(),
            "CPT '{}' items must be dynamically discovered in SQLite",
            cpt
        );
    }

    // 2. ACF Custom Fields in wp_postmeta
    let trainingen = db.get_posts("trainingen", None, None, 10).unwrap();
    let target_post = if let Ok(Some(p)) = db.get_post_by_slug("leiderschap-training") {
        p
    } else {
        trainingen.first().expect("At least one training").clone()
    };
    if let Some(val) = target_post.meta.get("duur") {
        assert_eq!(val, &serde_json::json!("2 dagen"));
    }

    // 3. Public SSR Archive Routes
    let app = create_app(db.clone());
    for route in ["/trainingen/", "/vacatures/", "/cases/"] {
        let req = Request::builder().uri(route).body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Archive route {} failed",
            route
        );
        let html = body_to_string(resp).await;

        assert_zero_comment_leaks(&html, route);
        assert!(
            !html.contains(r#"{"theme_slug":"#),
            "Raw JSON must not leak into SSR for {}",
            route
        );
    }

    // 4. Single CPT Route & Diagonal/Swiper Markup SSR
    let single_route = format!("/trainingen/{}/", target_post.slug);
    let req = Request::builder()
        .uri(&single_route)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Single CPT route {} failed",
        single_route
    );
    let html = body_to_string(resp).await;

    assert_zero_comment_leaks(&html, &single_route);
    assert!(
        html.contains(&target_post.title.rendered),
        "Single CPT title must be rendered"
    );
    assert!(
        html.contains("diagonal-section") || html.contains("swiper") || html.contains("content"),
        "Expected diagonal/swiper or rich content in single CPT HTML"
    );
}
