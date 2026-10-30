//! Milestone 3 Integration Suite
//! (`tests/milestone3_elementor_starter_wxr_tests.rs`)
//!
//! Validates:
//! 1. Elementor Starter Kit WXR ingestion, CSS variables compilation, options persistence & SSR <head> injection.
//! 2. Multi-chunk CDATA streaming (>64KB) without truncation.
//! 3. Elementor Theme Builder library templates (header/footer) & display conditions translation & SSR rendering.
//! 4. Universal internal link and media remapping while preserving external links (GitHub, Twitter, Google Fonts).
//! 5. Precedence of `<wp:base_blog_url>` over `<channel><link>` and `<wp:base_site_url>`.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_server::theme_builder::models::{
    ConditionType, RuleTarget, TemplateFilter, TemplateType,
};
use daypress_server::{create_app, Database};
use std::io::Cursor;
use tower::ServiceExt;

#[tokio::test]
async fn test_daystudio_starter_kit_wxr_ingestion_and_ssr_head_css() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<wp:base_blog_url>https://websitedemos.net/brandstore-02/</wp:base_blog_url>
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
	<item>
		<title>Home</title>
		<wp:post_id>20</wp:post_id>
		<wp:post_name>home</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Welcome to Brandstore</p>]]></content:encoded>
	</item>
	<item>
		<title>Featured Post</title>
		<wp:post_id>30</wp:post_id>
		<wp:post_name>featured-post</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>News article</p>]]></content:encoded>
	</item>
</channel>
</rss>"##;

    let res = db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");
    assert!(
        res.imported_posts >= 2,
        "Expected elementor_kit and post to count as posts"
    );
    assert_eq!(res.imported_pages, 1, "Expected 1 page");

    // Verify dual-write persistence to options
    let kit_css_1 = db.get_option("daypress_elementor_kit_css").unwrap();
    let kit_css_2 = db.get_option("elementor_active_kit_css").unwrap();
    assert!(
        kit_css_1.is_some(),
        "daypress_elementor_kit_css must be persisted"
    );
    assert!(
        kit_css_2.is_some(),
        "elementor_active_kit_css must be persisted"
    );

    let css = kit_css_1.unwrap();
    assert!(
        css.contains("--e-global-color-primary: #ff4f00;"),
        "Primary color variable missing"
    );
    assert!(
        css.contains("--e-global-color-secondary: #212529;"),
        "Secondary color variable missing"
    );
    assert!(
        css.contains("fonts.googleapis.com"),
        "Google Fonts @import missing"
    );
    assert!(css.contains("Montserrat"), "Montserrat font missing");
    assert!(
        css.contains("--e-global-typography-primary-font-family: Montserrat;"),
        "Typography font variable missing"
    );
    assert!(
        css.contains("--e-content-width: 1200px;"),
        "Content width variable missing or default"
    );
    assert!(
        css.contains("--content-width: 1200px;"),
        "Standard content width variable missing"
    );
    assert!(
        css.contains("--container-max-width: 1200px;"),
        "Container max width variable missing"
    );

    // Verify SSR <head> injection
    let app = create_app(db);
    let req = Request::builder()
        .uri("/home/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains(r#"<style id="daystudio-kit-css">"#),
        "SSR must contain kit css block"
    );
    assert!(
        html.contains("--e-global-color-primary: #ff4f00;"),
        "SSR head must contain primary color"
    );
    assert!(
        html.contains("Welcome to Brandstore"),
        "SSR must render page content"
    );
}

#[tokio::test]
async fn test_multi_chunk_cdata_streaming_without_truncation() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // Construct a payload where CDATA in _elementor_data exceeds 64KB (~85KB)
    let mut large_widgets = Vec::new();
    for i in 0..400 {
        large_widgets.push(format!(
            r#"{{"id":"w_{}","elType":"widget","widgetType":"heading","settings":{{"title":"Heading Widget Index {} with extensive padding text 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz repeating"}}}}"#,
            i, i
        ));
    }
    let large_json = format!(
        r#"[{{"id":"s1","elType":"section","elements":[{{"id":"c1","elType":"column","elements":[{}]}}]}}]"#,
        large_widgets.join(",")
    );
    assert!(
        large_json.len() > 65536,
        "Payload must be >64KB to trigger multi-chunk CDATA streaming"
    );

    let wxr = format!(
        r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<item>
		<title>Large Page</title>
		<wp:post_id>55</wp:post_id>
		<wp:post_name>large-page</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Initial content</p>]]></content:encoded>
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
        .get_post_by_slug("large-page")
        .unwrap()
        .expect("post found");
    assert!(
        post.content.rendered.contains("Heading Widget Index 0"),
        "First widget must be rendered"
    );
    assert!(
        post.content.rendered.contains("Heading Widget Index 399"),
        "Last widget in chunk must be rendered"
    );

    // Verify AST layout was parsed from the full, non-truncated JSON
    let layout = db.get_post_layout(post.id).unwrap();
    assert!(layout.is_some(), "Layout AST must be saved");
}

#[tokio::test]
async fn test_daystudio_library_theme_templates_and_display_conditions_ssr() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<item>
		<title>Site Header</title>
		<wp:post_id>101</wp:post_id>
		<wp:post_name>site-header</wp:post_name>
		<wp:post_type>elementor_library</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_template_type</wp:meta_key>
			<wp:meta_value><![CDATA[header]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_conditions</wp:meta_key>
			<wp:meta_value><![CDATA[["include/general"]]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"h_sec","elType":"section","elements":[{"id":"h_col","elType":"column","elements":[{"id":"h_w","elType":"widget","widgetType":"heading","settings":{"title":"Global Header Landmark"}}]}]}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Site Footer</title>
		<wp:post_id>102</wp:post_id>
		<wp:post_name>site-footer</wp:post_name>
		<wp:post_type>elementor_library</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_template_type</wp:meta_key>
			<wp:meta_value><![CDATA[footer]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_conditions</wp:meta_key>
			<wp:meta_value><![CDATA[a:1:{i:0;s:15:"include/general";}]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"f_sec","elType":"section","elements":[{"id":"f_col","elType":"column","elements":[{"id":"f_w","elType":"widget","widgetType":"heading","settings":{"title":"Global Footer Copyright"}}]}]}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>About Us</title>
		<wp:post_id>200</wp:post_id>
		<wp:post_name>about-us</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Our company story</p>]]></content:encoded>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");

    // Verify daypress_templates records and conditions
    let templates = db
        .theme_templates()
        .list(&TemplateFilter::default())
        .expect("list templates");
    assert_eq!(
        templates.len(),
        2,
        "Expected 1 header and 1 footer template"
    );

    let header_tpl = templates
        .iter()
        .find(|t| t.template_type == TemplateType::Header)
        .expect("Header exists");
    assert_eq!(header_tpl.title, "Site Header");
    assert_eq!(header_tpl.conditions.len(), 1);
    assert_eq!(
        header_tpl.conditions[0].condition_type,
        ConditionType::Include
    );
    assert_eq!(header_tpl.conditions[0].rule_target, RuleTarget::EntireSite);

    let footer_tpl = templates
        .iter()
        .find(|t| t.template_type == TemplateType::Footer)
        .expect("Footer exists");
    assert_eq!(footer_tpl.title, "Site Footer");
    assert_eq!(footer_tpl.conditions.len(), 1);
    assert_eq!(
        footer_tpl.conditions[0].condition_type,
        ConditionType::Include
    );
    assert_eq!(footer_tpl.conditions[0].rule_target, RuleTarget::EntireSite);

    // Verify dual-write to options
    let opt_hdr = db
        .get_option("daypress_elementor_header_html")
        .unwrap()
        .unwrap_or_default();
    let opt_ftr = db
        .get_option("daypress_elementor_footer_html")
        .unwrap()
        .unwrap_or_default();
    assert!(
        opt_hdr.contains("Global Header Landmark"),
        "Header HTML option missing"
    );
    assert!(
        opt_ftr.contains("Global Footer Copyright"),
        "Footer HTML option missing"
    );

    // Verify SSR renders header and footer landmarks
    let app = create_app(db);
    let req = Request::builder()
        .uri("/about-us/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains("Global Header Landmark"),
        "SSR must include Elementor header landmark"
    );
    assert!(
        html.contains("Global Footer Copyright"),
        "SSR must include Elementor footer landmark"
    );
    assert!(
        html.contains("Our company story"),
        "SSR must include page content"
    );
}

#[tokio::test]
async fn test_universal_link_and_media_remapping() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<wp:base_blog_url>https://websitedemos.net/brandstore-02/</wp:base_blog_url>
	<item>
		<title>Product Page</title>
		<wp:post_id>301</wp:post_id>
		<wp:post_name>product-showcase</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[
			<p>Explore <a href="https://websitedemos.net/brandstore-02/shop/">Shop Collection</a>.</p>
			<img src="https://websitedemos.net/brandstore-02/wp-content/uploads/2026/02/sneaker.png" alt="Sneaker" />
			<img src="/wp-content/uploads/2026/02/hat.png" alt="Hat" />
			<p>External: <a href="https://github.com/rust-lang/rust">Rust Repo</a></p>
			<p>Social: <a href="https://twitter.com/rustlang">Twitter</a></p>
			<link href="https://fonts.googleapis.com/css2?family=Roboto" rel="stylesheet" />
		]]></content:encoded>
	</item>
	<item>
		<title>Elementor Showcase</title>
		<wp:post_id>302</wp:post_id>
		<wp:post_name>elementor-showcase</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"w1","elType":"widget","widgetType":"image","settings":{"image":{"url":"https:\/\/websitedemos.net\/brandstore-02\/wp-content\/uploads\/2026\/02\/hero.jpg"},"link":{"url":"https:\/\/websitedemos.net\/brandstore-02\/about\/"}}}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Shop Nav Item</title>
		<wp:post_id>303</wp:post_id>
		<wp:post_name>shop-nav-item</wp:post_name>
		<wp:post_type>nav_menu_item</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_menu_item_url</wp:meta_key>
			<wp:meta_value><![CDATA[https://websitedemos.net/brandstore-02/categories/]]></wp:meta_value>
		</wp:postmeta>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");

    let post = db
        .get_post_by_slug("product-showcase")
        .unwrap()
        .expect("post found");

    // Internal demo link and media remapped in standard content
    assert!(
        post.content.rendered.contains(r#"href="/shop/""#),
        "Internal demo shop link must be remapped to relative"
    );
    assert!(
        post.content
            .rendered
            .contains(r#"src="/uploads/2026/02/sneaker.png""#),
        "Absolute uploads must be remapped to /uploads/"
    );
    assert!(
        post.content
            .rendered
            .contains(r#"src="/uploads/2026/02/hat.png""#),
        "Relative /wp-content/uploads/ must be remapped to /uploads/"
    );

    // External URLs preserved intact
    assert!(
        post.content
            .rendered
            .contains(r#"href="https://github.com/rust-lang/rust""#),
        "GitHub link must be preserved"
    );
    assert!(
        post.content
            .rendered
            .contains(r#"href="https://twitter.com/rustlang""#),
        "Twitter link must be preserved"
    );
    assert!(
        post.content
            .rendered
            .contains("https://fonts.googleapis.com/css2?family=Roboto"),
        "Google Fonts link must be preserved"
    );

    // Escaped URLs in Elementor JSON remapped
    let post_elem = db
        .get_post_by_slug("elementor-showcase")
        .unwrap()
        .expect("elem post found");
    assert!(
        post_elem
            .content
            .rendered
            .contains(r#"/uploads/2026/02/hero.jpg"#),
        "Elementor JSON media must be remapped to /uploads/"
    );
    assert!(
        post_elem.content.rendered.contains(r#"/about/"#),
        "Elementor JSON link must be remapped to relative"
    );
}

#[tokio::test]
async fn test_wxr_base_blog_url_precedence_over_base_site_url() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // Base site url is https://websitedemos.net, channel link is https://websitedemos.net/wrong-channel/,
    // but base_blog_url is https://websitedemos.net/brandstore-02/.
    // Base blog url MUST have top precedence so /brandstore-02/ prefix is stripped cleanly.
    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<link>https://websitedemos.net/wrong-channel/</link>
	<wp:base_site_url>https://websitedemos.net</wp:base_site_url>
	<wp:base_blog_url>https://websitedemos.net/brandstore-02/</wp:base_blog_url>
	<item>
		<title>Subdirectory Link Test</title>
		<wp:post_id>401</wp:post_id>
		<wp:post_name>sub-test</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[
			<p><a href="https://websitedemos.net/brandstore-02/checkout/">Checkout</a></p>
		]]></content:encoded>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("import_wxr failed");

    let post = db
        .get_post_by_slug("sub-test")
        .unwrap()
        .expect("post found");
    assert!(
        post.content.rendered.contains(r#"href="/checkout/""#),
        "Subdirectory prefix must be stripped using base_blog_url precedence"
    );
    assert!(
        !post.content.rendered.contains("/brandstore-02/checkout/"),
        "Old subdirectory path must not linger"
    );
}
