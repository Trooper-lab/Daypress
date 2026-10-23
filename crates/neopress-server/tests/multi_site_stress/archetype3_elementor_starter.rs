//! Archetype 3: Elementor Starter Site Ingestion, Kit CSS, and URL Remapping.

use super::common::{assert_zero_comment_leaks, body_to_string, load_fixture_or_fallback, setup_test_db};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use neopress_server::create_app;
use std::io::Cursor;
use tower::ServiceExt;

const FALLBACK_ELEMENTOR_STARTER_WXR: &str = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<wp:base_blog_url>https://websitedemos.net/brandstore-02/</wp:base_blog_url>
	<!-- Elementor Kit Post -->
	<item>
		<title>Default Kit</title>
		<wp:post_id>10</wp:post_id>
		<wp:post_name>default-kit</wp:post_name>
		<wp:post_type>elementor_kit</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_page_settings</wp:meta_key>
			<wp:meta_value><![CDATA[{"system_colors":[{"_id":"primary","title":"Primary","color":"#ff4f00"},{"_id":"secondary","title":"Secondary","color":"#212529"}],"system_typography":[{"_id":"primary","title":"Primary","typography_font_family":"Montserrat","typography_font_weight":"700"}],"container_width":{"size":1200,"unit":"px"}}]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<!-- Starter Home Page with Containers & Links -->
	<item>
		<title>Home</title>
		<wp:post_id>20</wp:post_id>
		<wp:post_name>home</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"c1","elType":"container","settings":{"flex_direction":"row"},"elements":[{"id":"w1","elType":"widget","widgetType":"heading","settings":{"title":"BrandStore Starter"}},{"id":"w2","elType":"widget","widgetType":"button","settings":{"text":"About Us","link":{"url":"https:\/\/websitedemos.net\/brandstore-02\/about\/"}}},{"id":"w3","elType":"widget","widgetType":"button","settings":{"text":"GitHub","link":{"url":"https:\/\/github.com\/rust-lang\/rust"}}}]}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<!-- Starter Blog Post -->
	<item>
		<title>Featured Post</title>
		<wp:post_id>30</wp:post_id>
		<wp:post_name>featured-post</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>BrandStore news article.</p>]]></content:encoded>
	</item>
</channel>
</rss>"##;

pub async fn run_test() {
    let db = setup_test_db();
    let wxr_data = load_fixture_or_fallback(
        "archetype3_elementor_starter/elementor_starter.xml",
        FALLBACK_ELEMENTOR_STARTER_WXR,
    );

    let res = db.import_wxr(Cursor::new(wxr_data)).expect("Elementor starter WXR import failed");
    assert!(res.imported_posts + res.imported_pages >= 2, "Expected at least kit and home page");

    // 1. Kit CSS Dual-Write Options Persistence
    let kit_css = db.get_option("neopress_elementor_kit_css")
        .unwrap()
        .or_else(|| db.get_option("elementor_active_kit_css").unwrap())
        .expect("Kit CSS option missing from wp_options");
    assert!(kit_css.contains("--e-global-color-primary: #ff4f00;"), "Primary color variable missing");
    assert!(kit_css.contains("--e-global-color-secondary: #212529;"), "Secondary color variable missing");
    assert!(kit_css.contains("1200px"), "Content width 1200px missing");

    // 2. Public SSR Route & <head> Style Injection
    let app = create_app(db.clone());
    let req = Request::builder().uri("/home/").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let html = body_to_string(resp).await;

    assert_zero_comment_leaks(&html, "Archetype 3 /home/");
    assert!(html.contains(r#"<style id="neopress-elementor-kit-css">"#), "SSR must contain kit css block");
    assert!(html.contains("--e-global-color-primary: #ff4f00;"));
    assert!(html.contains("BrandStore") || html.contains("Precision Engineering"));

    // 3. Link Remapping Verification
    assert!(html.contains(r#"href="/about/""#), "Internal demo link must be remapped to /about/");
    assert!(html.contains("https://github.com/rust-lang/rust"), "External GitHub link must remain untouched");
}
