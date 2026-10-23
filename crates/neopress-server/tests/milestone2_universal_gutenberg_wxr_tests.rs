//! Milestone 2 Integration & Adversarial Verification Suite
//! (`tests/milestone2_universal_gutenberg_wxr_tests.rs`)

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use neopress_ast::{render_layout_to_html, transpile_gutenberg_content};
use neopress_server::{create_app, Database};
use neopress_server::domain::models::NavMenuItemDto;
use neopress_server::templates::public::menus::{build_nav_menu_tree, render_hierarchical_nav_menu};
use std::io::Cursor;
use tower::ServiceExt;

#[test]
fn test_balanced_brace_nested_gutenberg_comments_without_leaks() {
    let gutenberg_content = r#"
<!-- wp:group {"metadata":{"name":"Hero Group"},"style":{"spacing":{"padding":{"top":"50px","bottom":"50px"}}}} -->
<div class="wp-block-group">
    <!-- wp:heading {"level":1} -->
    <h1>Universal Gutenberg Test</h1>
    <!-- /wp:heading -->
    <!-- wp:paragraph {"align":"center","placeholder":"Content with {embedded: braces} in string"} -->
    <p class="has-text-align-center">Testing complex braces {foo: "bar"}.</p>
    <!-- /wp:paragraph -->
</div>
<!-- /wp:group -->
"#;

    let layout = transpile_gutenberg_content(gutenberg_content).expect("Transpilation succeeds");
    assert!(!layout.root.children.is_empty(), "Transpiled nodes should not be empty");
    let html = render_layout_to_html(&layout);

    // Verify zero Gutenberg raw comment leaks
    assert!(!html.contains("<!-- wp:"), "HTML must not leak raw <!-- wp: comments");
    assert!(!html.contains("<!-- /wp:"), "HTML must not leak raw <!-- /wp: comments");
    assert!(html.contains("Universal Gutenberg Test"), "Heading text must be preserved");
    assert!(html.contains("Testing complex braces"), "Paragraph text must be preserved");
}

#[test]
fn test_core_blocks_gallery_quote_embed_list_rendering() {
    let raw = r#"
<!-- wp:quote {"className":"is-style-large"} -->
<blockquote class="wp-block-quote is-style-large"><p>Neopress delivers sub-millisecond WordPress performance.</p><cite>Core Team</cite></blockquote>
<!-- /wp:quote -->

<!-- wp:list {"ordered":true} -->
<ol class="wp-block-list">
    <!-- wp:list-item -->
    <li>First Item</li>
    <!-- /wp:list-item -->
    <!-- wp:list-item -->
    <li>Second Item</li>
    <!-- /wp:list-item -->
</ol>
<!-- /wp:list -->

<!-- wp:embed {"url":"https://www.youtube.com/watch?v=dQw4w9WgXcQ","type":"video","providerNameSlug":"youtube","responsive":true} -->
<figure class="wp-block-embed is-type-video is-provider-youtube wp-block-embed-youtube wp-embed-aspect-16-9 wp-has-aspect-ratio"><div class="wp-block-embed__wrapper">
https://www.youtube.com/watch?v=dQw4w9WgXcQ
</div></figure>
<!-- /wp:embed -->
"#;

    let layout = transpile_gutenberg_content(raw).expect("Transpilation succeeds");
    let html = render_layout_to_html(&layout);

    assert!(html.contains("<blockquote"), "Must render blockquote");
    assert!(html.contains("Neopress delivers sub-millisecond"), "Must render quote content");
    assert!(html.contains("<ol") || html.contains("<ul"), "Must render list");
    assert!(html.contains("<li>First Item</li>"), "Must render list items");
    assert!(html.contains("youtube.com") || html.contains("iframe") || html.contains("video"), "Must render video embed");
    assert!(!html.contains("<!-- wp:"), "Must zero comment leaks");
}

#[test]
fn test_safe_unicode_slugification_no_na_collapse() {
    // Russian / Cyrillic
    let cyrillic = generated_slugify::slugify("Привет Мир");
    assert_ne!(cyrillic, "n-a", "Cyrillic title must not collapse to n-a");
    assert!(cyrillic.contains("привет-мир") || cyrillic.chars().any(|c| c.is_alphanumeric()));

    // Japanese / Kanji & Kana
    let japanese = generated_slugify::slugify("こんにちは 世界");
    assert_ne!(japanese, "n-a", "Japanese title must not collapse to n-a");
    assert!(!japanese.is_empty());

    // Greek
    let greek = generated_slugify::slugify("Γειά σου Κόσμε");
    assert_ne!(greek, "n-a", "Greek title must not collapse to n-a");

    // Hebrew
    let hebrew = generated_slugify::slugify("שלום עולם");
    assert_ne!(hebrew, "n-a", "Hebrew title must not collapse to n-a");

    // Arabic
    let arabic = generated_slugify::slugify("مرحبا بالعالم");
    assert_ne!(arabic, "n-a", "Arabic title must not collapse to n-a");
}

#[test]
fn test_hierarchical_3_plus_level_navigation_ssr_markup() {
    let items = vec![
        NavMenuItemDto {
            id: 1,
            title: "Parent Level 1".to_string(),
            url: "/parent/".to_string(),
            parent: 0,
            menu_order: 1,
            ..Default::default()
        },
        NavMenuItemDto {
            id: 2,
            title: "Child Level 2".to_string(),
            url: "/parent/child/".to_string(),
            parent: 1,
            menu_order: 1,
            ..Default::default()
        },
        NavMenuItemDto {
            id: 3,
            title: "Grandchild Level 3".to_string(),
            url: "/parent/child/grandchild/".to_string(),
            parent: 2,
            menu_order: 1,
            ..Default::default()
        },
    ];

    let tree = build_nav_menu_tree(&items);
    assert_eq!(tree.len(), 1, "Root should have 1 top-level parent");
    assert_eq!(tree[0].children.len(), 1, "Level 1 should have 1 child");
    assert_eq!(tree[0].children[0].children.len(), 1, "Level 2 should have 1 grandchild");

    let markup = render_hierarchical_nav_menu(&tree, false);
    assert!(markup.contains(r#"<ul class="menu np-primary-menu">"#));
    assert!(markup.contains("menu-item-has-children"), "Parent must have menu-item-has-children class");
    assert!(markup.contains(r#"<ul class="sub-menu">"#), "Sub-menu must have sub-menu class");
    assert!(markup.contains("Parent Level 1"));
    assert!(markup.contains("Child Level 2"));
    assert!(markup.contains("Grandchild Level 3"));
}

#[test]
fn test_sticky_posts_and_threaded_comments_wxr_remapping() {
    let db = Database::new_in_memory().unwrap();

    let wxr_data = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<item>
		<title>Normal Post</title>
		<wp:post_id>1</wp:post_id>
		<wp:post_name>normal-post</wp:post_name>
		<content:encoded><![CDATA[<p>Normal post content</p>]]></content:encoded>
		<wp:post_date>2026-10-21 10:00:00</wp:post_date>
		<wp:status>publish</wp:status>
		<wp:post_type>post</wp:post_type>
		<wp:is_sticky>0</wp:is_sticky>
	</item>
	<item>
		<title>Sticky Older Post</title>
		<wp:post_id>2</wp:post_id>
		<wp:post_name>sticky-older-post</wp:post_name>
		<content:encoded><![CDATA[<p>Sticky post content</p>]]></content:encoded>
		<wp:post_date>2026-10-01 10:00:00</wp:post_date>
		<wp:status>publish</wp:status>
		<wp:post_type>post</wp:post_type>
		<wp:is_sticky>1</wp:is_sticky>
		<wp:comment>
			<wp:comment_id>101</wp:comment_id>
			<wp:comment_author>Alice</wp:comment_author>
			<wp:comment_content>Top comment</wp:comment_content>
			<wp:comment_date>2026-10-02 10:00:00</wp:comment_date>
			<wp:comment_parent>0</wp:comment_parent>
		</wp:comment>
		<wp:comment>
			<wp:comment_id>102</wp:comment_id>
			<wp:comment_author>Bob</wp:comment_author>
			<wp:comment_content>Threaded reply</wp:comment_content>
			<wp:comment_date>2026-10-02 11:00:00</wp:comment_date>
			<wp:comment_parent>101</wp:comment_parent>
		</wp:comment>
	</item>
</channel>
</rss>"#;

    let res = db.import_wxr(Cursor::new(wxr_data)).unwrap();
    assert_eq!(res.imported_posts, 2);
    assert_eq!(res.imported_comments, 2);

    // Verify Sticky Post Query Order: sticky post must come first despite being older!
    let posts = db.get_posts("post", Some("publish"), None, 10).unwrap();
    assert!(posts.len() >= 2);
    assert_eq!(posts[0].slug, "sticky-older-post", "Sticky post must be ordered first");
    assert!(posts[0].sticky, "Sticky post flag must be true");
    let normal_post = posts.iter().find(|p| p.slug == "normal-post").expect("normal post found");
    assert!(!normal_post.sticky, "Normal post flag must be false");

    // Verify Threaded Comments Remapping
    let sticky_id = posts[0].id;
    let post_hydrated = db.get_post_by_id(sticky_id).unwrap().expect("Post exists");
    assert_eq!(post_hydrated.comments.len(), 2);
    let parent_comment = post_hydrated.comments.iter().find(|c| c.author_name == "Alice").unwrap();
    let child_comment = post_hydrated.comments.iter().find(|c| c.author_name == "Bob").unwrap();
    assert_eq!(child_comment.parent, parent_comment.id, "Child comment parent must match remapped parent ID");
}

#[tokio::test]
async fn test_dynamic_taxonomy_archive_route_ssr() {
    let db = Database::new_in_memory().unwrap();

    let wxr_data = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<wp:term>
		<wp:term_id>501</wp:term_id>
		<wp:term_taxonomy>genre</wp:term_taxonomy>
		<wp:term_slug>cyberpunk</wp:term_slug>
		<wp:term_name><![CDATA[Cyberpunk]]></wp:term_name>
		<wp:term_description><![CDATA[Stories of high tech and low life.]]></wp:term_description>
	</wp:term>
	<item>
		<title>Neuromancer Tribute</title>
		<wp:post_id>601</wp:post_id>
		<wp:post_name>neuromancer-tribute</wp:post_name>
		<wp:post_type>post</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>The sky above the port was the color of television...</p>]]></content:encoded>
		<category domain="genre" nicename="cyberpunk"><![CDATA[Cyberpunk]]></category>
	</item>
</channel>
</rss>"#;

    db.import_wxr(Cursor::new(wxr_data)).unwrap();

    let app = create_app(db);

    let req = Request::builder()
        .uri("/genre/cyberpunk/")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    assert!(body_str.contains("Genre: Cyberpunk"), "Title should show genre name");
    assert!(body_str.contains("Stories of high tech and low life."), "Description should be rendered");
    assert!(body_str.contains("Neuromancer Tribute"), "Associated post title should be rendered");
}
