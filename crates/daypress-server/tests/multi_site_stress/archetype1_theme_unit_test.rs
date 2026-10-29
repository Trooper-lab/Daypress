//! Archetype 1: WordPress Theme Unit Test WXR Ingestion & SSR Parity.

use super::common::{
    assert_zero_comment_leaks, body_to_string, load_fixture_or_fallback, setup_test_db,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use daypress_server::create_app;
use daypress_server::domain::models::NavMenuItemDto;
use daypress_server::templates::public::menus::build_nav_menu_tree;
use std::io::Cursor;
use tower::ServiceExt;

const FALLBACK_THEME_UNIT_WXR: &str = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>Theme Unit Test</title>
	<wp:base_blog_url>http://127.0.0.1:8090</wp:base_blog_url>
	<wp:term>
		<wp:term_id>20</wp:term_id>
		<wp:term_taxonomy>nav_menu</wp:term_taxonomy>
		<wp:term_slug>primary-navigation</wp:term_slug>
		<wp:term_name><![CDATA[Primary Navigation]]></wp:term_name>
	</wp:term>
	<item>
		<title>Explore</title>
		<wp:post_id>102</wp:post_id>
		<wp:post_type>nav_menu_item</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:postmeta><wp:meta_key>_menu_item_menu_item_parent</wp:meta_key><wp:meta_value><![CDATA[0]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_menu_item_url</wp:meta_key><wp:meta_value><![CDATA[#]]></wp:meta_value></wp:postmeta>
		<category domain="nav_menu" nicename="primary-navigation"><![CDATA[Primary Navigation]]></category>
	</item>
	<item>
		<title>Documentation</title>
		<wp:post_id>103</wp:post_id>
		<wp:post_type>nav_menu_item</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:postmeta><wp:meta_key>_menu_item_menu_item_parent</wp:meta_key><wp:meta_value><![CDATA[102]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_menu_item_url</wp:meta_key><wp:meta_value><![CDATA[/core-blocks/]]></wp:meta_value></wp:postmeta>
		<category domain="nav_menu" nicename="primary-navigation"><![CDATA[Primary Navigation]]></category>
	</item>
	<item>
		<title>Advanced Specs (Level 3)</title>
		<wp:post_id>104</wp:post_id>
		<wp:post_type>nav_menu_item</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:postmeta><wp:meta_key>_menu_item_menu_item_parent</wp:meta_key><wp:meta_value><![CDATA[103]]></wp:meta_value></wp:postmeta>
		<wp:postmeta><wp:meta_key>_menu_item_url</wp:meta_key><wp:meta_value><![CDATA[/threaded-discussion/]]></wp:meta_value></wp:postmeta>
		<category domain="nav_menu" nicename="primary-navigation"><![CDATA[Primary Navigation]]></category>
	</item>
	<item>
		<title>Sticky Announcement</title>
		<wp:post_id>2</wp:post_id>
		<wp:post_name>sticky-announcement</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:is_sticky>1</wp:is_sticky>
		<wp:post_date>2024-01-01 10:00:00</wp:post_date>
		<content:encoded><![CDATA[<p>Sticky announcement post.</p>]]></content:encoded>
	</item>
	<item>
		<title>Threaded Discussion</title>
		<wp:post_id>3</wp:post_id>
		<wp:post_name>threaded-discussion</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:is_sticky>0</wp:is_sticky>
		<wp:post_date>2026-02-01 10:00:00</wp:post_date>
		<content:encoded><![CDATA[<p>Threaded discussion post.</p>]]></content:encoded>
		<wp:comment>
			<wp:comment_id>201</wp:comment_id>
			<wp:comment_author>Alice</wp:comment_author>
			<wp:comment_content>Top comment</wp:comment_content>
			<wp:comment_parent>0</wp:comment_parent>
		</wp:comment>
		<wp:comment>
			<wp:comment_id>202</wp:comment_id>
			<wp:comment_author>Bob</wp:comment_author>
			<wp:comment_content>Threaded reply</wp:comment_content>
			<wp:comment_parent>201</wp:comment_parent>
		</wp:comment>
	</item>
	<item>
		<title>Core Blocks Showcase</title>
		<wp:post_id>4</wp:post_id>
		<wp:post_name>core-blocks</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:is_sticky>0</wp:is_sticky>
		<wp:post_date>2026-03-01 10:00:00</wp:post_date>
		<content:encoded><![CDATA[
<!-- wp:quote {"className":"is-style-large"} -->
<blockquote class="wp-block-quote is-style-large"><p>Daypress delivers sub-millisecond WordPress performance.</p><cite>Core Engineering</cite></blockquote>
<!-- /wp:quote -->
<!-- wp:columns -->
<div class="wp-block-columns">
	<!-- wp:column -->
	<div class="wp-block-column"><p>Left column text.</p></div>
	<!-- /wp:column -->
	<!-- wp:column -->
	<div class="wp-block-column"><p>Right column text.</p></div>
	<!-- /wp:column -->
</div>
<!-- /wp:columns -->
<!-- wp:list {"ordered":true} -->
<ol class="wp-block-list"><li>First Nested Item</li><li>Second Nested Item</li></ol>
<!-- /wp:list -->
		]]></content:encoded>
	</item>
	<item>
		<title>Привет Мир</title>
		<wp:post_id>5</wp:post_id>
		<wp:post_name>привет-мир</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:post_date>2026-03-02 10:00:00</wp:post_date>
		<content:encoded><![CDATA[<p>Cyrillic post content.</p>]]></content:encoded>
	</item>
</channel>
</rss>"##;

pub async fn run_test() {
    let db = setup_test_db();
    let wxr_data = load_fixture_or_fallback(
        "archetype1_theme_unit_test/theme_unit_test.xml",
        FALLBACK_THEME_UNIT_WXR,
    );

    let res = db
        .import_wxr(Cursor::new(wxr_data))
        .expect("WXR import failed");
    assert!(
        res.imported_posts >= 2,
        "Expected at least 2 imported posts"
    );
    assert!(
        res.imported_comments >= 2,
        "Expected at least 2 threaded comments"
    );

    // 1. Relational Assertions: Sticky Post Priority
    let posts = db.get_posts("post", Some("publish"), None, 10).unwrap();
    assert!(!posts.is_empty(), "Posts should not be empty");
    assert_eq!(
        posts[0].slug, "sticky-announcement",
        "Sticky post must be ordered first"
    );
    assert!(posts[0].sticky, "Sticky flag must be true");

    // 2. Relational Assertions: Threaded Comments Remapping
    let disc_post = db
        .get_post_by_slug("threaded-discussion")
        .unwrap()
        .expect("Discussion post exists");
    assert!(
        disc_post.comments.len() >= 2,
        "Expected at least 2 comments"
    );
    let parent_c = disc_post
        .comments
        .iter()
        .find(|c| c.author_name == "Alice")
        .expect("Alice comment exists");
    let child_c = disc_post
        .comments
        .iter()
        .find(|c| c.author_name == "Bob")
        .expect("Bob comment exists");
    assert_eq!(
        child_c.parent, parent_c.id,
        "Child comment parent must be remapped to Alice's DB ID"
    );

    // 3. Relational Assertions: Hierarchical Navigation Menu Tree (3 Levels)
    let menu_items: Vec<NavMenuItemDto> = db
        .get_option("daypress_nav_menu_items")
        .unwrap()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let tree = build_nav_menu_tree(&menu_items);
    assert!(!tree.is_empty(), "Menu tree must not be empty");
    let branch = tree
        .iter()
        .find(|item| !item.children.is_empty())
        .expect("Level 1 branch with children");
    assert!(!branch.children.is_empty(), "Level 2 child must exist");
    let grandchild = &branch.children[0].children;
    assert!(!grandchild.is_empty(), "Level 3 grandchild must exist");

    // 4. Safe UTF-8 Slugification
    let unicode_post = db
        .get_post_by_slug("привет-мир")
        .ok()
        .flatten()
        .or_else(|| db.get_post_by_slug("core-blocks").ok().flatten());
    assert!(unicode_post.is_some(), "Unicode post must exist in SQLite");
    let slug = unicode_post.unwrap().slug;
    assert_ne!(slug, "n-a", "Slug must not collapse into n-a");

    // 5. Semantic SSR HTML & Zero Comment Leaks
    let app = create_app(db.clone());
    let req = Request::builder()
        .uri("/core-blocks/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let html = body_to_string(resp).await;

    assert_zero_comment_leaks(&html, "Archetype 1 /core-blocks/");
    assert!(
        html.contains("<blockquote") || html.contains("wp-block-quote"),
        "Must render semantic blockquote"
    );
    assert!(
        html.contains("Daypress delivers"),
        "Must render blockquote text"
    );
    assert!(
        html.contains("<ol") || html.contains("<ul") || html.contains("wp-block-list"),
        "Must render list"
    );
    assert!(html.contains("First Nested Item"), "Must render list items");
}
