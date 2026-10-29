//! Adversarial Milestone 3 Verification Suite
//!
//! Stress-tests:
//! 1. External link preservation (GitHub, Twitter, Google Fonts, mailto, tel, anchors).
//! 2. Internal link remapping (root base, subdirectories, queries, anchors, escaped JSON).
//! 3. Theme builder display conditions priority and specificity (front_page vs general, exclusions).

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_server::{create_app, Database};
use std::io::Cursor;
use tower::ServiceExt;

#[tokio::test]
async fn test_adversarial_external_url_preservation_and_link_remapping() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<wp:base_blog_url>https://websitedemos.net/brandstore-02/</wp:base_blog_url>
	<item>
		<title>Adversarial Links Test</title>
		<wp:post_id>501</wp:post_id>
		<wp:post_name>adv-links</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[
			<p>Internal Root: <a href="https://websitedemos.net/brandstore-02/">Home</a></p>
			<p>Internal Subpage: <a href="https://websitedemos.net/brandstore-02/about/">About</a></p>
			<p>Internal Query and Fragment: <a href="https://websitedemos.net/brandstore-02/shop/?sort=desc&color=blue#grid">Shop Query</a></p>
			<p>Media absolute: <img src="https://websitedemos.net/brandstore-02/wp-content/uploads/2026/02/hero.webp" /></p>
			<p>Media relative: <img src="/wp-content/uploads/2026/02/banner.webp" /></p>
			<p>External GitHub: <a href="https://github.com/rust-lang/rust">GitHub</a></p>
			<p>External Twitter: <a href="https://twitter.com/rustlang?lang=en">Twitter</a></p>
			<p>External Fonts: <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;700&display=swap" rel="stylesheet" /></p>
			<p>Mailto link: <a href="mailto:admin@daypress.io?subject=Inquiry">Email Us</a></p>
			<p>Tel link: <a href="tel:+18005550199">Call Us</a></p>
			<p>Anchor only: <a href="#details-section">Jump to Details</a></p>
			<p>External WP Uploads: <a href="https://external-wp.org/wp-content/uploads/2026/02/guide.pdf">Guide</a></p>
		]]></content:encoded>
	</item>
	<item>
		<title>Adversarial Elementor Links Test</title>
		<wp:post_id>502</wp:post_id>
		<wp:post_name>adv-elem-links</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"w1","elType":"widget","widgetType":"button","settings":{"link":{"url":"https:\/\/github.com\/rust-lang\/rust"}}},{"id":"w2","elType":"widget","widgetType":"button","settings":{"link":{"url":"https:\/\/websitedemos.net\/brandstore-02\/contact\/"}}}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("WXR import failed");

    let post = db
        .get_post_by_slug("adv-links")
        .unwrap()
        .expect("post found");
    let content = &post.content.rendered;

    // 1. Assert internal links are cleanly remapped
    assert!(
        content.contains(r#"href="/""#),
        "Internal root link must be /"
    );
    assert!(
        content.contains(r#"href="/about/""#),
        "Internal subpage link must be /about/"
    );
    assert!(
        content.contains(r#"href="/shop/?sort=desc&color=blue#grid""#),
        "Query and fragment on internal link must be preserved relative"
    );
    assert!(
        content.contains(r#"src="/uploads/2026/02/hero.webp""#),
        "Absolute uploads URL must become /uploads/"
    );
    assert!(
        content.contains(r#"src="/uploads/2026/02/banner.webp""#),
        "Relative uploads URL must become /uploads/"
    );

    // 2. Adversarial assertions: External URLs must be untouched
    assert!(
        content.contains(r#"href="https://github.com/rust-lang/rust""#),
        "External GitHub URL must not be altered"
    );
    assert!(
        content.contains(r#"href="https://twitter.com/rustlang?lang=en""#),
        "External Twitter URL with query must not be altered"
    );
    assert!(
        content.contains(
            "https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;700&display=swap"
        ),
        "External Google Fonts link must not be altered"
    );
    assert!(
        content.contains(r#"href="mailto:admin@daypress.io?subject=Inquiry""#),
        "Mailto link must not be altered"
    );
    assert!(
        content.contains(r#"href="tel:+18005550199""#),
        "Tel link must not be altered"
    );
    assert!(
        content.contains(r##"href="#details-section""##),
        "Anchor link must not be altered"
    );
    assert!(
        content.contains(r#"href="https://external-wp.org/wp-content/uploads/2026/02/guide.pdf""#),
        "External WordPress uploads URL must not be altered"
    );

    // 3. Verify in Elementor JSON AST
    let elem_post = db
        .get_post_by_slug("adv-elem-links")
        .unwrap()
        .expect("elem post found");
    let elem_content = &elem_post.content.rendered;
    assert!(
        elem_content.contains("https://github.com/rust-lang/rust"),
        "Elementor AST widget must retain external GitHub link"
    );
    assert!(
        elem_content.contains(r#"/contact/"#),
        "Elementor AST widget internal link must be remapped to /contact/"
    );
}

#[tokio::test]
async fn test_adversarial_theme_builder_display_conditions_and_specificity() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // Import 2 header templates:
    // Header 1: General (entire site) -> "Default Global Header"
    // Header 2: Front Page only -> "Dedicated Front Page Header"
    // And 1 footer template: General with exclusion of page 602 -> "Universal Footer"
    let wxr = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<item>
		<title>Default Global Header</title>
		<wp:post_id>6001</wp:post_id>
		<wp:post_name>default-header</wp:post_name>
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
			<wp:meta_value><![CDATA[[{"id":"h1","elType":"widget","widgetType":"heading","settings":{"title":"Global Header Default"}}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Front Page Header</title>
		<wp:post_id>6002</wp:post_id>
		<wp:post_name>front-header</wp:post_name>
		<wp:post_type>elementor_library</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_template_type</wp:meta_key>
			<wp:meta_value><![CDATA[header]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_conditions</wp:meta_key>
			<wp:meta_value><![CDATA[["include/singular/front_page"]]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"h2","elType":"widget","widgetType":"heading","settings":{"title":"Dedicated Front Page Header"}}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Excluded Footer</title>
		<wp:post_id>6003</wp:post_id>
		<wp:post_name>excluded-footer</wp:post_name>
		<wp:post_type>elementor_library</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_template_type</wp:meta_key>
			<wp:meta_value><![CDATA[footer]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_conditions</wp:meta_key>
			<wp:meta_value><![CDATA[["include/general", "exclude/singular/page/602"]]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"f1","elType":"widget","widgetType":"heading","settings":{"title":"Universal Footer Landmark"}}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Home Page</title>
		<wp:post_id>601</wp:post_id>
		<wp:post_name>home</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Front page body text</p>]]></content:encoded>
	</item>
	<item>
		<title>Excluded Page</title>
		<wp:post_id>602</wp:post_id>
		<wp:post_name>no-footer-page</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Special Landing without Footer</p>]]></content:encoded>
	</item>
	<item>
		<title>Standard Page</title>
		<wp:post_id>603</wp:post_id>
		<wp:post_name>standard-page</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Standard Page with Footer</p>]]></content:encoded>
	</item>
</channel>
</rss>"##;

    db.import_wxr(Cursor::new(wxr)).expect("WXR import failed");

    // Configure static front page
    let _ = db.set_option("show_on_front", "page");
    let _ = db.set_option("page_on_front", "601");

    let no_footer_post = db
        .get_post_by_slug("no-footer-page")
        .unwrap()
        .expect("post found");
    println!(
        "DEBUG: no_footer_post.id = {}, slug = {}",
        no_footer_post.id, no_footer_post.slug
    );
    let tpl3 = db
        .theme_templates()
        .get_by_id(3)
        .unwrap()
        .expect("template 3 exists");
    let excl_cond = tpl3
        .conditions
        .iter()
        .find(|c| {
            c.condition_type == daypress_server::theme_builder::models::ConditionType::Exclude
        })
        .expect("exclude cond exists");
    assert_eq!(
        excl_cond.target_id,
        no_footer_post.id.to_string(),
        "WXR import must remap condition target_id to new SQLite post id"
    );

    let app = create_app(db);

    // 1. Request front page: Must render Dedicated Front Page Header (due to specificity 40 vs 10)
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains("Dedicated Front Page Header"),
        "Front page must render dedicated front page header"
    );
    assert!(
        !html.contains("Global Header Default"),
        "Front page must not render lower-specificity general header"
    );
    assert!(
        html.contains("Universal Footer Landmark"),
        "Front page must render universal footer"
    );

    // 2. Request standard page (/standard-page/): Must render Default Global Header AND Universal Footer
    let req = Request::builder()
        .uri("/standard-page/")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains("Global Header Default"),
        "Standard page must render default global header"
    );
    assert!(
        !html.contains("Dedicated Front Page Header"),
        "Standard page must NOT render front page header"
    );
    assert!(
        html.contains("Universal Footer Landmark"),
        "Standard page must render universal footer"
    );

    // 3. Request excluded page (/no-footer-page/): Must NOT render Universal Footer because of exclude condition!
    let req = Request::builder()
        .uri("/no-footer-page/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    println!("DEBUG excluded page html snippet around footer:\n{}", {
        let idx = html.find("<footer").unwrap_or(0);
        &html[idx..std::cmp::min(idx + 300, html.len())]
    });

    assert!(
        html.contains("Global Header Default"),
        "Excluded page must still render default global header"
    );
    assert!(
        !html.contains("Universal Footer Landmark"),
        "Excluded page must NOT render footer due to exclusion condition rule!"
    );
}
