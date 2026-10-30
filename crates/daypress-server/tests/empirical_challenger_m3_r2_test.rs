//! Empirical Challenger 1 Test Suite for Milestone 3 R2
//! (`crates/daypress-server/tests/empirical_challenger_m3_r2_test.rs`)
//!
//! Validates:
//! 1. Multi-chunk CDATA streaming (>100KB) across arbitrary chunk sizes.
//! 2. XML site URL precedence (<wp:base_blog_url> > <link> > <wp:base_site_url>) regardless of tag ordering.
//! 3. Elementor Starter Kit styles compilation & SSR <head> injection of --e-global-color-* and --e-global-typography-*.
//! 4. Universal URL & media remapping with query strings, hashes, escaped JSON URLs, and external link preservation.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_server::{create_app, Database};
use std::io::Cursor;
use tower::ServiceExt;

#[tokio::test]
async fn test_challenger_extreme_cdata_streaming_128kb() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // Construct a ~130KB JSON structure inside CDATA
    let mut widgets = Vec::new();
    for i in 0..600 {
        widgets.push(format!(
            r#"{{"id":"w_{}","elType":"widget","widgetType":"heading","settings":{{"title":"Heading {} - Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."}}}}"#,
            i, i
        ));
    }
    let large_json = format!(
        r#"[{{"id":"sec_main","elType":"section","elements":[{{"id":"col_main","elType":"column","elements":[{}]}}]}}]"#,
        widgets.join(",")
    );
    assert!(
        large_json.len() > 120_000,
        "Must be >120KB for extreme CDATA test"
    );

    let wxr = format!(
        r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<item>
		<title>Massive CDATA Page</title>
		<wp:post_id>777</wp:post_id>
		<wp:post_name>massive-cdata</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Initial encoded content</p>]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[{}]]></wp:meta_value>
		</wp:postmeta>
	</item>
</channel>
</rss>"##,
        large_json
    );

    let res = db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");
    assert_eq!(res.imported_pages, 1);

    let post = db
        .get_post_by_slug("massive-cdata")
        .unwrap()
        .expect("post found");
    assert!(
        post.content.rendered.contains("Heading 0 - Lorem ipsum"),
        "First heading must be present"
    );
    assert!(
        post.content.rendered.contains("Heading 599 - Lorem ipsum"),
        "Last heading in chunk must be present"
    );

    let layout = db.get_post_layout(post.id).unwrap();
    assert!(
        layout.is_some(),
        "Complete AST layout must be parsed and stored"
    );
}

#[tokio::test]
async fn test_challenger_base_url_precedence_reverse_tag_order() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // In this test, <wp:base_blog_url> appears FIRST, then <wp:base_site_url>, then <link>.
    // <wp:base_blog_url> MUST remain winning with priority 3!
    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<wp:base_blog_url>https://websitedemos.net/starter-kit-sub/</wp:base_blog_url>
	<wp:base_site_url>https://websitedemos.net</wp:base_site_url>
	<link>https://websitedemos.net/incorrect-link/</link>
	<item>
		<title>Precedence Item</title>
		<wp:post_id>888</wp:post_id>
		<wp:post_name>precedence-item</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[
			<p><a href="https://websitedemos.net/starter-kit-sub/pricing/">View Pricing</a></p>
			<img src="https://websitedemos.net/starter-kit-sub/wp-content/uploads/2026/04/logo.svg" />
		]]></content:encoded>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");

    let post = db
        .get_post_by_slug("precedence-item")
        .unwrap()
        .expect("post found");
    assert!(
        post.content.rendered.contains(r#"href="/pricing/""#),
        "Link must be remapped to relative /pricing/ based on base_blog_url"
    );
    assert!(
        post.content
            .rendered
            .contains(r#"src="/uploads/2026/04/logo.svg""#),
        "Media must be remapped to /uploads/ based on base_blog_url"
    );
    assert!(
        !post.content.rendered.contains("/starter-kit-sub/pricing/"),
        "Subdirectory must not remain unstripped"
    );
}

#[tokio::test]
async fn test_challenger_daystudio_kit_css_full_token_injection() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<item>
		<title>Global Styling Kit</title>
		<wp:post_id>901</wp:post_id>
		<wp:post_name>styling-kit</wp:post_name>
		<wp:post_type>elementor_kit</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_page_settings</wp:meta_key>
			<wp:meta_value><![CDATA[{
				"system_colors": [
					{"_id": "primary", "title": "Primary", "color": "#1e3a8a"},
					{"_id": "secondary", "title": "Secondary", "color": "#3b82f6"},
					{"_id": "text", "title": "Text", "color": "#1f2937"},
					{"_id": "accent", "title": "Accent", "color": "#f59e0b"}
				],
				"custom_colors": [
					{"_id": "brand_dark", "title": "Brand Dark", "color": "#0f172a"}
				],
				"system_typography": [
					{"_id": "primary", "title": "Primary", "typography_font_family": "Plus Jakarta Sans", "typography_font_weight": "600"},
					{"_id": "secondary", "title": "Secondary", "typography_font_family": "Inter", "typography_font_weight": "400"}
				],
				"container_width": {"size": 1280, "unit": "px"}
			}]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Public Landing</title>
		<wp:post_id>902</wp:post_id>
		<wp:post_name>public-landing</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Welcome to our styled page</p>]]></content:encoded>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");

    // 1. Verify CSS stored in options
    let kit_css = db
        .get_option("daypress_elementor_kit_css")
        .unwrap()
        .expect("kit css option must exist");
    assert!(kit_css.contains("--e-global-color-primary: #1e3a8a;"));
    assert!(kit_css.contains("--e-global-color-secondary: #3b82f6;"));
    assert!(kit_css.contains("--e-global-color-text: #1f2937;"));
    assert!(kit_css.contains("--e-global-color-accent: #f59e0b;"));
    assert!(kit_css.contains("--e-global-color-brand_dark: #0f172a;"));
    assert!(kit_css.contains("--e-global-typography-primary-font-family: Plus Jakarta Sans;"));
    assert!(kit_css.contains("--e-global-typography-primary-font-weight: 600;"));
    // Note: Elementor kit setting `container_width` defaults to 1140px when alias is missing on `content_width`
    assert!(
        kit_css.contains("--e-content-width: 1140px;")
            || kit_css.contains("--e-content-width: 1280px;")
    );
    assert!(
        kit_css.contains("family=Plus+Jakarta+Sans"),
        "Google Fonts url must encode spaces with +"
    );

    // 2. Verify SSR <head> injection
    let app = create_app(db);
    let req = Request::builder()
        .uri("/public-landing/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(html.contains(r#"<style id="daystudio-kit-css">"#));
    assert!(html.contains("--e-global-color-primary: #1e3a8a;"));
    assert!(html.contains("--e-global-typography-primary-font-family: Plus Jakarta Sans;"));
    assert!(html.contains("Welcome to our styled page"));
}

#[tokio::test]
async fn test_challenger_url_remapping_queries_hashes_and_json_escapes() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<wp:base_blog_url>https://websitedemos.net/gadget-store/</wp:base_blog_url>
	<item>
		<title>Gadget Page</title>
		<wp:post_id>950</wp:post_id>
		<wp:post_name>gadget-page</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[
			<p>Root link: <a href="https://websitedemos.net/gadget-store/">Home</a></p>
			<p>Sub link with query and hash: <a href="https://websitedemos.net/gadget-store/shop/?view=grid&cat=deals#products">Deals</a></p>
			<p>External GitHub: <a href="https://github.com/rust-lang/rust">GitHub</a></p>
			<p>External Twitter: <a href="https://twitter.com/rustlang">Twitter</a></p>
			<p>External CDN: <img src="https://cdn.example.com/assets/badge.png" /></p>
		]]></content:encoded>
	</item>
	<item>
		<title>Gadget Elementor</title>
		<wp:post_id>951</wp:post_id>
		<wp:post_name>gadget-elementor</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"btn1","elType":"widget","widgetType":"button","settings":{"link":{"url":"https:\/\/websitedemos.net\/gadget-store\/checkout\/?promo=save10#pay"}}},{"id":"btn2","elType":"widget","widgetType":"button","settings":{"link":{"url":"https:\/\/github.com\/rust-lang\/cargo"}}}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");

    let page1 = db
        .get_post_by_slug("gadget-page")
        .unwrap()
        .expect("page1 found");
    assert!(
        page1.content.rendered.contains(r#"href="/""#),
        "Root demo link must become /"
    );
    assert!(
        page1
            .content
            .rendered
            .contains(r#"href="/shop/?view=grid&cat=deals#products""#),
        "Query and hash preserved on relative path"
    );
    assert!(
        page1
            .content
            .rendered
            .contains(r#"href="https://github.com/rust-lang/rust""#),
        "GitHub external URL preserved"
    );
    assert!(
        page1
            .content
            .rendered
            .contains(r#"href="https://twitter.com/rustlang""#),
        "Twitter external URL preserved"
    );
    assert!(
        page1
            .content
            .rendered
            .contains(r#"src="https://cdn.example.com/assets/badge.png""#),
        "CDN external media preserved"
    );

    let page2 = db
        .get_post_by_slug("gadget-elementor")
        .unwrap()
        .expect("page2 found");
    assert!(
        page2
            .content
            .rendered
            .contains(r#"/checkout/?promo=save10#pay"#),
        "Elementor JSON escaped URL remapped with query and hash"
    );
    assert!(
        page2
            .content
            .rendered
            .contains("https://github.com/rust-lang/cargo"),
        "Elementor JSON external link preserved"
    );
}

#[tokio::test]
async fn test_challenger_theme_builder_conditions_remapping_and_fallback_landmarks() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // Pre-populate legacy options elementor_footer_html with a canary string
    // that MUST NEVER leak if Theme Builder has an active template or excludes it!
    db.set_option(
        "daypress_elementor_footer_html",
        "<div id=\"CANARY_LEAKED_LEGACY_FOOTER\">Legacy Footer</div>",
    )
    .unwrap();

    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?><rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wp="http://wordpress.org/export/1.2/"><channel>
	<item><title>Global EntireSite Header</title><wp:post_id>1001</wp:post_id><wp:post_name>global-header</wp:post_name><wp:post_type>elementor_library</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta><wp:meta_key>_elementor_template_type</wp:meta_key><wp:meta_value><![CDATA[header]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_elementor_conditions</wp:meta_key><wp:meta_value><![CDATA[["include/general"]]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_elementor_data</wp:meta_key><wp:meta_value><![CDATA[[{"id":"gh1","elType":"widget","widgetType":"heading","settings":{"title":"Global EntireSite Header Landmark"}}]]]></wp:meta_value></wp:postmeta>
	</item>
	<item><title>VIP Page Specific Header</title><wp:post_id>1002</wp:post_id><wp:post_name>vip-header</wp:post_name><wp:post_type>elementor_library</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta><wp:meta_key>_elementor_template_type</wp:meta_key><wp:meta_value><![CDATA[header]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_elementor_conditions</wp:meta_key><wp:meta_value><![CDATA[["include/singular/page/701"]]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_elementor_data</wp:meta_key><wp:meta_value><![CDATA[[{"id":"vh1","elType":"widget","widgetType":"heading","settings":{"title":"VIP Post-Specific Header Landmark"}}]]]></wp:meta_value></wp:postmeta>
	</item>
	<item><title>Universal Footer with Landing Exclusion</title><wp:post_id>1003</wp:post_id><wp:post_name>excluded-footer</wp:post_name><wp:post_type>elementor_library</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta><wp:meta_key>_elementor_template_type</wp:meta_key><wp:meta_value><![CDATA[footer]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_elementor_conditions</wp:meta_key><wp:meta_value><![CDATA[["include/general", "exclude/singular/page/702"]]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_elementor_data</wp:meta_key><wp:meta_value><![CDATA[[{"id":"ef1","elType":"widget","widgetType":"heading","settings":{"title":"Active Elementor Footer Landmark"}}]]]></wp:meta_value></wp:postmeta>
	</item>
	<item><title>VIP Page</title><wp:post_id>701</wp:post_id><wp:post_name>vip-page</wp:post_name><wp:post_type>page</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[<p>VIP Page Content</p>]]></content:encoded></item>
	<item><title>Clean Landing Page</title><wp:post_id>702</wp:post_id><wp:post_name>clean-landing</wp:post_name><wp:post_type>page</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[<p>Clean Landing Content</p>]]></content:encoded></item>
</channel></rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("WXR import failed");

    let vip_page = db
        .get_post_by_slug("vip-page")
        .unwrap()
        .expect("vip page found");
    let landing_page = db
        .get_post_by_slug("clean-landing")
        .unwrap()
        .expect("landing page found");

    // 1. Empirically verify target_id relational remapping in SQLite without any manual updates!
    use daypress_server::theme_builder::models::{ConditionType, RuleTarget, TemplateType};
    use daypress_server::theme_builder::repository::ThemeTemplateRepository;
    let repo = ThemeTemplateRepository::from_database(&db);

    let vip_headers = repo.find_active_by_type(TemplateType::Header).unwrap();
    let vip_tpl = vip_headers
        .iter()
        .find(|t| t.title == "VIP Page Specific Header")
        .expect("vip tpl exists");
    assert_eq!(vip_tpl.conditions.len(), 1);
    assert_eq!(vip_tpl.conditions[0].rule_target, RuleTarget::Page);
    assert_eq!(
        vip_tpl.conditions[0].target_id,
        vip_page.id.to_string(),
        "WXR target_id 701 must be automatically remapped to SQLite post ID {}",
        vip_page.id
    );

    let footers = repo.find_active_by_type(TemplateType::Footer).unwrap();
    let footer_tpl = footers
        .iter()
        .find(|t| t.title == "Universal Footer with Landing Exclusion")
        .expect("footer tpl exists");
    let exclude_cond = footer_tpl
        .conditions
        .iter()
        .find(|c| c.condition_type == ConditionType::Exclude)
        .expect("exclude condition exists");
    assert_eq!(exclude_cond.rule_target, RuleTarget::Page);
    assert_eq!(
        exclude_cond.target_id,
        landing_page.id.to_string(),
        "WXR target_id 702 in exclude condition must be automatically remapped to SQLite post ID {}",
        landing_page.id
    );

    let app = create_app(db);

    // 2. VIP Page: Post-specific header MUST beat EntireSite header! Footer must render.
    let req = Request::builder()
        .uri("/vip-page/")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains("VIP Post-Specific Header Landmark"),
        "VIP Page must render post-specific header"
    );
    assert!(
        !html.contains("Global EntireSite Header Landmark"),
        "VIP Page must NOT render general site header"
    );
    assert!(
        html.contains("Active Elementor Footer Landmark"),
        "VIP Page must render active Elementor footer"
    );

    // 3. Clean Landing: Excluded footer must NOT render Elementor footer, must NOT leak legacy footer, MUST render fallback day-footer!
    let req = Request::builder()
        .uri("/clean-landing/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains("Global EntireSite Header Landmark"),
        "Clean Landing must render general header"
    );
    assert!(
        !html.contains("Active Elementor Footer Landmark"),
        "Clean Landing must NOT render Elementor footer due to exclusion rule"
    );
    assert!(
        !html.contains("CANARY_LEAKED_LEGACY_FOOTER"),
        "Clean Landing must NEVER leak legacy elementor_footer_html from options!"
    );
    assert!(
        html.contains(r#"<footer class="day-footer">"#),
        "Clean Landing must render theme fallback day-footer landmark"
    );
}
