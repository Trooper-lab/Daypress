//! Empirical Challenger 2 Test Suite for Milestone 2 R2 (`tests/empirical_challenger_m2_r2_test.rs`)
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use neopress_server::domain::models::NavMenuItemDto;
use neopress_server::templates::public::menus::{
    build_nav_menu_tree, render_hierarchical_nav_menu,
};
use neopress_server::{create_app, Database};
use std::io::Cursor;
use tower::ServiceExt;

#[test]
fn test_empirical_nav_menu_isolated_tree_builder_and_ssr_renderer() {
    let items = vec![
        NavMenuItemDto {
            id: 1,
            title: "Solutions".to_string(),
            url: "/solutions/".to_string(),
            parent: 0,
            menu_order: 1,
            ..Default::default()
        },
        NavMenuItemDto {
            id: 2,
            title: "Cloud".to_string(),
            url: "/solutions/cloud/".to_string(),
            parent: 1,
            menu_order: 1,
            ..Default::default()
        },
        NavMenuItemDto {
            id: 3,
            title: "Serverless".to_string(),
            url: "/solutions/cloud/serverless/".to_string(),
            parent: 2,
            menu_order: 1,
            ..Default::default()
        },
        NavMenuItemDto {
            id: 4,
            title: "Rust MicroVM".to_string(),
            url: "/solutions/cloud/serverless/rust/".to_string(),
            parent: 3,
            menu_order: 1,
            ..Default::default()
        },
    ];

    let tree = build_nav_menu_tree(&items);
    assert_eq!(tree.len(), 1, "Level 1 has 1 root node ('Solutions')");
    assert_eq!(tree[0].children.len(), 1, "Level 2 has 1 child ('Cloud')");
    assert_eq!(
        tree[0].children[0].children.len(),
        1,
        "Level 3 has 1 child ('Serverless')"
    );
    assert_eq!(
        tree[0].children[0].children[0].children.len(),
        1,
        "Level 4 has 1 child ('Rust MicroVM')"
    );

    let markup = render_hierarchical_nav_menu(&tree, false);
    assert!(markup.contains(r#"<ul class="menu np-primary-menu">"#));
    assert!(markup.contains("menu-item-has-children"));
    assert_eq!(
        markup.matches(r#"<ul class="sub-menu">"#).count(),
        3,
        "Expected 3 nested <ul class=\"sub-menu\">"
    );
}

#[test]
fn test_empirical_nav_menu_cycle_and_orphan_resilience() {
    let malformed_items = vec![
        NavMenuItemDto {
            id: 10,
            title: "Cyclic Node A".to_string(),
            url: "/a/".to_string(),
            parent: 20,
            menu_order: 1,
            ..Default::default()
        },
        NavMenuItemDto {
            id: 20,
            title: "Cyclic Node B".to_string(),
            url: "/b/".to_string(),
            parent: 10,
            menu_order: 2,
            ..Default::default()
        },
        NavMenuItemDto {
            id: 30,
            title: "Orphan Node".to_string(),
            url: "/orphan/".to_string(),
            parent: 9999,
            menu_order: 3,
            ..Default::default()
        },
    ];
    let tree = build_nav_menu_tree(&malformed_items);
    assert_eq!(tree.len(), 0, "No root items exist in cycle/orphan input");
    let html = render_hierarchical_nav_menu(&tree, false);
    assert!(
        html.is_empty(),
        "Empty tree produces empty HTML without panicking"
    );
}

#[tokio::test]
async fn test_empirical_wxr_end_to_end_nav_menu_ssr_defect() {
    let db = Database::new_in_memory().expect("in-memory db");
    let wxr_data = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<rss version=\"2.0\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\" xmlns:wp=\"http://wordpress.org/export/1.2/\">\n<channel>\n<title>Enterprise Multi-Level Site</title>\n",
        "<item><title>Solutions</title><wp:post_id>1001</wp:post_id><wp:post_type>nav_menu_item</wp:post_type><wp:status>publish</wp:status><wp:menu_order>1</wp:menu_order><category domain=\"nav_menu\" nicename=\"primary\">Primary</category><wp:postmeta><wp:meta_key>_menu_item_url</wp:meta_key><wp:meta_value>/solutions/</wp:meta_value></wp:postmeta><wp:postmeta><wp:meta_key>_menu_item_menu_item_parent</wp:meta_key><wp:meta_value>0</wp:meta_value></wp:postmeta></item>\n",
        "<item><title>Cloud</title><wp:post_id>1002</wp:post_id><wp:post_type>nav_menu_item</wp:post_type><wp:status>publish</wp:status><wp:menu_order>1</wp:menu_order><category domain=\"nav_menu\" nicename=\"primary\">Primary</category><wp:postmeta><wp:meta_key>_menu_item_url</wp:meta_key><wp:meta_value>/solutions/cloud/</wp:meta_value></wp:postmeta><wp:postmeta><wp:meta_key>_menu_item_menu_item_parent</wp:meta_key><wp:meta_value>1001</wp:meta_value></wp:postmeta></item>\n",
        "<item><title>Serverless</title><wp:post_id>1003</wp:post_id><wp:post_type>nav_menu_item</wp:post_type><wp:status>publish</wp:status><wp:menu_order>1</wp:menu_order><category domain=\"nav_menu\" nicename=\"primary\">Primary</category><wp:postmeta><wp:meta_key>_menu_item_url</wp:meta_key><wp:meta_value>/solutions/cloud/serverless/</wp:meta_value></wp:postmeta><wp:postmeta><wp:meta_key>_menu_item_menu_item_parent</wp:meta_key><wp:meta_value>1002</wp:meta_value></wp:postmeta></item>\n",
        "<item><title>Rust MicroVM</title><wp:post_id>1004</wp:post_id><wp:post_type>nav_menu_item</wp:post_type><wp:status>publish</wp:status><wp:menu_order>1</wp:menu_order><category domain=\"nav_menu\" nicename=\"primary\">Primary</category><wp:postmeta><wp:meta_key>_menu_item_url</wp:meta_key><wp:meta_value>/solutions/cloud/serverless/rust/</wp:meta_value></wp:postmeta><wp:postmeta><wp:meta_key>_menu_item_menu_item_parent</wp:meta_key><wp:meta_value>1003</wp:meta_value></wp:postmeta></item>\n",
        "</channel></rss>"
    );

    let _res = db
        .import_wxr(Cursor::new(wxr_data))
        .expect("WXR import succeeds");
    let settings = db.get_settings().expect("settings read");
    assert_eq!(
        settings.nav_menu_items.len(),
        4,
        "4 items in settings.nav_menu_items"
    );

    let app = create_app(db);
    let req = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);

    // Assert that public SSR header must render child menu items and nested sub-menu markup
    let header_start = body_str.find("<header").expect("header exists");
    let header_end = body_str[header_start..]
        .find("</header>")
        .map(|idx| header_start + idx)
        .expect("header close exists");
    let header_markup = &body_str[header_start..header_end];

    println!(
        "\n=== RENDERED WXR HEADER SSR MARKUP ===\n{}\n======================================\n",
        header_markup
    );

    assert!(
        header_markup.contains("Cloud"),
        "Header markup must contain level-2 item 'Cloud'"
    );
    assert!(
        header_markup.contains("Serverless"),
        "Header markup must contain level-3 item 'Serverless'"
    );
    assert!(
        header_markup.contains("Rust MicroVM"),
        "Header markup must contain level-4 item 'Rust MicroVM'"
    );
    assert!(
        header_markup.contains("menu-item-has-children"),
        "Header markup must have menu-item-has-children class"
    );
    assert_eq!(
        header_markup.matches(r#"<ul class="sub-menu">"#).count(),
        3,
        "Header markup must render 3 nested <ul class=\"sub-menu\">"
    );
}

#[test]
fn test_empirical_threaded_comments_reverse_order_and_counts() {
    let db = Database::new_in_memory().expect("in-memory db");
    let wxr_data = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<rss version=\"2.0\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\" xmlns:wp=\"http://wordpress.org/export/1.2/\">\n<channel>\n<item>\n",
        "<title>Discussion Post</title><wp:post_id>301</wp:post_id><wp:post_name>discussion-post</wp:post_name><wp:post_type>post</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[<p>Content</p>]]></content:encoded>\n",
        "<wp:comment><wp:comment_id>202</wp:comment_id><wp:comment_author>Bob</wp:comment_author><wp:comment_content>Reply to Alice</wp:comment_content><wp:comment_approved>1</wp:comment_approved><wp:comment_parent>201</wp:comment_parent></wp:comment>\n",
        "<wp:comment><wp:comment_id>201</wp:comment_id><wp:comment_author>Alice</wp:comment_author><wp:comment_content>Root comment</wp:comment_content><wp:comment_approved>1</wp:comment_approved><wp:comment_parent>0</wp:comment_parent></wp:comment>\n",
        "<wp:comment><wp:comment_id>203</wp:comment_id><wp:comment_author>Charlie</wp:comment_author><wp:comment_content>Reply to Bob</wp:comment_content><wp:comment_approved>1</wp:comment_approved><wp:comment_parent>202</wp:comment_parent></wp:comment>\n",
        "<wp:comment><wp:comment_id>204</wp:comment_id><wp:comment_author>Spam</wp:comment_author><wp:comment_content>Spam</wp:comment_content><wp:comment_approved>0</wp:comment_approved><wp:comment_parent>0</wp:comment_parent></wp:comment>\n",
        "</item></channel></rss>"
    );

    let res = db
        .import_wxr(Cursor::new(wxr_data))
        .expect("WXR import succeeds");
    assert_eq!(res.imported_comments, 4);

    let post = db
        .get_post_by_slug("discussion-post")
        .expect("query")
        .expect("exists");
    assert_eq!(
        post.comments.len(),
        3,
        "Only approved comments (3) hydrated on public post view"
    );

    let alice = post
        .comments
        .iter()
        .find(|c| c.author_name == "Alice")
        .unwrap();
    let bob = post
        .comments
        .iter()
        .find(|c| c.author_name == "Bob")
        .unwrap();
    let charlie = post
        .comments
        .iter()
        .find(|c| c.author_name == "Charlie")
        .unwrap();

    assert_eq!(alice.parent, 0);
    assert_eq!(
        bob.parent, alice.id,
        "Bob's parent must be remapped to Alice's new ID"
    );
    assert_eq!(
        charlie.parent, bob.id,
        "Charlie's parent must be remapped to Bob's new ID"
    );

    let counts = db.get_comment_counts().unwrap();
    assert_eq!(
        counts.all, 5,
        "Total 5 comments (4 imported + 1 seed comment)"
    );
    assert_eq!(
        counts.approved, 4,
        "4 comments approved (3 imported + 1 seed comment)"
    );
    assert_eq!(counts.pending, 1, "1 comment pending");
}

#[tokio::test]
async fn test_empirical_sticky_posts_ordering_and_ssr_styling() {
    let db = Database::new_in_memory().expect("in-memory db");
    let wxr_data = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<rss version=\"2.0\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\" xmlns:wp=\"http://wordpress.org/export/1.2/\">\n<channel>\n",
        "<item><title>Recent Post</title><wp:post_id>401</wp:post_id><wp:post_name>recent-post</wp:post_name><wp:post_date>2026-10-23 12:00:00</wp:post_date><wp:post_type>post</wp:post_type><wp:status>publish</wp:status><wp:is_sticky>0</wp:is_sticky><content:encoded><![CDATA[<p>Recent</p>]]></content:encoded></item>\n",
        "<item><title>Oldest Sticky</title><wp:post_id>402</wp:post_id><wp:post_name>oldest-sticky</wp:post_name><wp:post_date>2024-01-01 10:00:00</wp:post_date><wp:post_type>post</wp:post_type><wp:status>publish</wp:status><wp:is_sticky>1</wp:is_sticky><content:encoded><![CDATA[<p>Old sticky</p>]]></content:encoded></item>\n",
        "<item><title>Newer Sticky</title><wp:post_id>403</wp:post_id><wp:post_name>newer-sticky</wp:post_name><wp:post_date>2025-06-01 10:00:00</wp:post_date><wp:post_type>post</wp:post_type><wp:status>publish</wp:status><wp:is_sticky>1</wp:is_sticky><content:encoded><![CDATA[<p>New sticky</p>]]></content:encoded></item>\n",
        "</channel></rss>"
    );

    db.import_wxr(Cursor::new(wxr_data))
        .expect("WXR import succeeds");

    let posts = db
        .get_posts("post", Some("publish"), None, 10)
        .expect("get_posts succeeds");
    assert!(posts.len() >= 3);
    assert_eq!(
        posts[0].slug, "newer-sticky",
        "Newer sticky post must be 1st"
    );
    assert!(posts[0].sticky);
    assert_eq!(
        posts[1].slug, "oldest-sticky",
        "Older sticky post must be 2nd"
    );
    assert!(posts[1].sticky);

    let recent_pos = posts.iter().position(|p| p.slug == "recent-post").unwrap();
    assert!(
        recent_pos >= 2,
        "recent-post must be positioned after sticky posts"
    );
    assert!(!posts[recent_pos].sticky);

    let app = create_app(db);
    let req = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("class=\"np-card sticky\""),
        "Sticky post must have .np-card.sticky CSS class"
    );
    assert!(
        body_str.contains("class=\"np-card\""),
        "Normal post must have .np-card CSS class"
    );
}

#[test]
fn test_empirical_safe_utf8_slugification_adversarial_scripts() {
    use generated_slugify::slugify;

    assert_eq!(
        slugify("Привет, Прекрасный Мир 2026!"),
        "привет-прекрасный-мир-2026"
    );
    assert_eq!(
        slugify("Ελληνικά & Αρχαία Κείμενα"),
        "ελληνικά-αρχαία-κείμενα"
    );
    assert_eq!(
        slugify("超高速 Rust エンジン: バージョン 2"),
        "超高速-rust-エンジン-バージョン-2"
    );
    assert_eq!(
        slugify("الذكاء الاصطناعي والتكنولوجيا"),
        "الذكاء-الاصطناعي-والتكنولوجيا"
    );
    assert_eq!(
        slugify("Hà Nội Nghìn Năm Văn Hiến"),
        "ha-nội-nghin-nam-van-hiến"
    );
    assert_eq!(
        slugify("Neopress: 東京 & Москва (v2.0)"),
        "neopress-東京-москва-v2-0"
    );
    assert_eq!(
        slugify("<h1>Hello <strong>World</strong></h1>"),
        "hello-world"
    );

    let long_title = "Привет ".repeat(50);
    let capped = slugify(&long_title);
    assert!(capped.chars().count() <= 200, "Slug capped at 200 chars");
    assert!(!capped.ends_with('-'), "Trailing dash stripped");

    assert_eq!(slugify("   ???   !!!   ###   "), "n-a");
    assert_eq!(slugify(""), "n-a");
}

#[tokio::test]
async fn test_empirical_dynamic_custom_taxonomy_archive_ssr() {
    let db = Database::new_in_memory().expect("in-memory db");
    let wxr_data = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<rss version=\"2.0\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\" xmlns:wp=\"http://wordpress.org/export/1.2/\">\n<channel>\n",
        "<wp:term><wp:term_id>801</wp:term_id><wp:term_taxonomy>industry</wp:term_taxonomy><wp:term_slug>fintech</wp:term_slug><wp:term_name><![CDATA[Financial Technology]]></wp:term_name><wp:term_description><![CDATA[Next-gen payment rails.]]></wp:term_description></wp:term>\n",
        "<wp:term><wp:term_id>802</wp:term_id><wp:term_taxonomy>industry</wp:term_taxonomy><wp:term_slug>healthtech</wp:term_slug><wp:term_name><![CDATA[Healthcare Technology]]></wp:term_name><wp:term_description><![CDATA[Telemedicine solutions.]]></wp:term_description></wp:term>\n",
        "<item><title>Stripe Settlement Engine</title><wp:post_id>901</wp:post_id><wp:post_name>stripe-settlement-engine</wp:post_name><wp:post_type>post</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[<p>Payment clearance.</p>]]></content:encoded><category domain=\"industry\" nicename=\"fintech\"><![CDATA[Financial Technology]]></category></item>\n",
        "<item><title>Medtronic Telemetry Core</title><wp:post_id>902</wp:post_id><wp:post_name>medtronic-telemetry-core</wp:post_name><wp:post_type>post</wp:post_type><wp:status>publish</wp:status><content:encoded><![CDATA[<p>Patient telemetry.</p>]]></content:encoded><category domain=\"industry\" nicename=\"healthtech\"><![CDATA[Healthcare Technology]]></category></item>\n",
        "</channel></rss>"
    );

    db.import_wxr(Cursor::new(wxr_data))
        .expect("WXR import succeeds");
    let app = create_app(db);

    let req1 = Request::builder()
        .uri("/industry/fintech/")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp1 = app.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let body1 = to_bytes(resp1.into_body(), usize::MAX).await.unwrap();
    let str1 = String::from_utf8_lossy(&body1);
    assert!(str1.contains("Industry: Financial Technology"));
    assert!(str1.contains("Next-gen payment rails."));
    assert!(str1.contains("Stripe Settlement Engine"));
    assert!(!str1.contains("Medtronic Telemetry Core"));

    let req2 = Request::builder()
        .uri("/industry/healthtech/")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let body2 = to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
    let str2 = String::from_utf8_lossy(&body2);
    assert!(str2.contains("Industry: Healthcare Technology"));
    assert!(str2.contains("Medtronic Telemetry Core"));
    assert!(!str2.contains("Stripe Settlement Engine"));

    let req3 = Request::builder()
        .uri("/industry/non-existent-slug/")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let resp3 = app.oneshot(req3).await.unwrap();
    assert_eq!(resp3.status(), StatusCode::NOT_FOUND);
}
