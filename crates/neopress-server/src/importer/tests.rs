use super::*;
use rusqlite::{params, Connection};
use std::io::Cursor;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
            CREATE TABLE wp_posts (
                ID INTEGER PRIMARY KEY AUTOINCREMENT,
                post_author INTEGER NOT NULL DEFAULT 1,
                post_date TEXT NOT NULL,
                post_date_gmt TEXT NOT NULL,
                post_content TEXT NOT NULL DEFAULT '',
                post_title TEXT NOT NULL DEFAULT '',
                post_excerpt TEXT NOT NULL DEFAULT '',
                post_status TEXT NOT NULL DEFAULT 'publish',
                comment_status TEXT NOT NULL DEFAULT 'open',
                ping_status TEXT NOT NULL DEFAULT 'open',
                post_password TEXT NOT NULL DEFAULT '',
                post_name TEXT NOT NULL DEFAULT '',
                to_ping TEXT NOT NULL DEFAULT '',
                pinged TEXT NOT NULL DEFAULT '',
                post_modified TEXT NOT NULL,
                post_modified_gmt TEXT NOT NULL,
                post_content_filtered TEXT NOT NULL DEFAULT '',
                post_parent INTEGER NOT NULL DEFAULT 0,
                guid TEXT NOT NULL DEFAULT '',
                menu_order INTEGER NOT NULL DEFAULT 0,
                post_type TEXT NOT NULL DEFAULT 'post',
                post_mime_type TEXT NOT NULL DEFAULT '',
                comment_count INTEGER NOT NULL DEFAULT 0,
                is_sticky INTEGER NOT NULL DEFAULT 0,
                layout_json TEXT DEFAULT NULL
            );
            CREATE TABLE wp_postmeta (
                meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
                post_id INTEGER NOT NULL DEFAULT 0,
                meta_key TEXT DEFAULT NULL,
                meta_value TEXT DEFAULT NULL
            );
            CREATE TABLE wp_options (
                option_id INTEGER PRIMARY KEY AUTOINCREMENT,
                option_name TEXT NOT NULL UNIQUE,
                option_value TEXT NOT NULL,
                autoload TEXT NOT NULL DEFAULT 'yes'
            );
            CREATE TABLE wp_terms (
                term_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                term_group INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE wp_term_taxonomy (
                term_taxonomy_id INTEGER PRIMARY KEY AUTOINCREMENT,
                term_id INTEGER NOT NULL,
                taxonomy TEXT NOT NULL DEFAULT 'category',
                description TEXT NOT NULL DEFAULT '',
                parent INTEGER NOT NULL DEFAULT 0,
                count INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE wp_term_relationships (
                object_id INTEGER NOT NULL,
                term_taxonomy_id INTEGER NOT NULL,
                term_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (object_id, term_taxonomy_id)
            );
            CREATE TABLE wp_comments (
                comment_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                comment_post_ID INTEGER NOT NULL DEFAULT 0,
                comment_author TEXT NOT NULL,
                comment_author_email TEXT NOT NULL DEFAULT '',
                comment_author_url TEXT NOT NULL DEFAULT '',
                comment_author_IP TEXT NOT NULL DEFAULT '',
                comment_date TEXT NOT NULL,
                comment_date_gmt TEXT NOT NULL,
                comment_content TEXT NOT NULL,
                comment_karma INTEGER NOT NULL DEFAULT 0,
                comment_approved TEXT NOT NULL DEFAULT '1',
                comment_agent TEXT NOT NULL DEFAULT '',
                comment_type TEXT NOT NULL DEFAULT 'comment',
                comment_parent INTEGER NOT NULL DEFAULT 0,
                user_id INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE wp_neopress_forms (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                schema_json TEXT NOT NULL DEFAULT '{}',
                settings_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE wp_neopress_plugins (
                slug TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL DEFAULT '1.0.0',
                description TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT 'general',
                is_builtin INTEGER NOT NULL DEFAULT 1,
                is_active INTEGER NOT NULL DEFAULT 1,
                manifest_json TEXT NOT NULL DEFAULT '{}',
                config_json TEXT NOT NULL DEFAULT '{}',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE wp_neopress_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                template_type TEXT NOT NULL,
                layout_json TEXT NOT NULL DEFAULT '{}',
                content TEXT NOT NULL DEFAULT '',
                priority INTEGER NOT NULL DEFAULT 100,
                is_active INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'publish',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE wp_neopress_template_conditions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template_id INTEGER NOT NULL,
                rule_target TEXT NOT NULL,
                rule_operator TEXT NOT NULL,
                rule_value TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 100,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
    )
    .unwrap();
    conn
}

#[test]
fn test_wxr_stream_parsing_and_ingestion() {
    let mut conn = setup_test_db();
    let sample_xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wfw="http://wellformedweb.org/CommentAPI/"
	xmlns:dc="http://purl.org/dc/elements/1.1/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>Exported WP Site</title>
	<description>Testing streaming WXR</description>
	<wp:category>
		<wp:term_id>1</wp:term_id>
		<wp:category_nicename>engineering</wp:category_nicename>
		<wp:cat_name>Engineering</wp:cat_name>
	</wp:category>
	<wp:tag>
		<wp:term_id>2</wp:term_id>
		<wp:tag_slug>rust</wp:tag_slug>
		<wp:tag_name>Rust</wp:tag_name>
	</wp:tag>
	<item>
		<title>Hello Streaming World</title>
		<wp:post_name>hello-streaming-world</wp:post_name>
		<content:encoded><![CDATA[<p>Streaming import works seamlessly!</p>]]></content:encoded>
		<wp:post_date>2026-10-21 14:00:00</wp:post_date>
		<wp:status>publish</wp:status>
		<wp:post_type>post</wp:post_type>
		<category domain="category" nicename="engineering"><![CDATA[Engineering]]></category>
		<category domain="post_tag" nicename="rust"><![CDATA[Rust]]></category>
		<wp:postmeta>
			<wp:meta_key>_test_meta_key</wp:meta_key>
			<wp:meta_value>meta_value_123</wp:meta_value>
		</wp:postmeta>
		<wp:comment>
			<wp:comment_author>Alice</wp:comment_author>
			<wp:comment_content>Awesome WXR stream!</wp:comment_content>
			<wp:comment_date>2026-10-21 14:05:00</wp:comment_date>
		</wp:comment>
	</item>
	<item>
		<title>About This Site</title>
		<wp:post_name>about-this-site</wp:post_name>
		<content:encoded><![CDATA[<p>About page content.</p>]]></content:encoded>
		<wp:status>publish</wp:status>
		<wp:post_type>page</wp:post_type>
	</item>
	<item>
		<title>Diagram PNG</title>
		<wp:post_name>diagram-png</wp:post_name>
		<wp:post_type>attachment</wp:post_type>
		<wp:attachment_url>http://example.com/uploads/diagram.png</wp:attachment_url>
	</item>
</channel>
</rss>"#;

    let res = import_wxr_stream(Cursor::new(sample_xml), &mut conn).unwrap();
    assert_eq!(res.imported_posts, 1);
    assert_eq!(res.imported_pages, 1);
    assert_eq!(res.imported_media, 1);
    assert_eq!(res.imported_categories, 1);
    assert_eq!(res.imported_tags, 1);
    assert_eq!(res.imported_comments, 1);

    let post_title: String = conn
        .query_row(
            "SELECT post_title FROM wp_posts WHERE post_name = 'hello-streaming-world'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(post_title, "Hello Streaming World");

    let attachment_mime: String = conn
        .query_row(
            "SELECT post_mime_type FROM wp_posts WHERE post_name = 'diagram-png'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(attachment_mime, "image/png");

    let blog_title: String = conn
        .query_row(
            "SELECT option_value FROM wp_options WHERE option_name = 'blogname'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(blog_title, "Exported WP Site");
}

#[test]
fn test_wxr_id_remapping_and_ast_transpilation() {
    let mut conn = setup_test_db();
    let sample_xml = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>Relational WXR Test</title>
	<description>Verify post_parent, _thumbnail_id, and AST transpilation</description>
	<item>
		<title>Hero Image</title>
		<wp:post_id>500</wp:post_id>
		<wp:post_name>hero-image</wp:post_name>
		<wp:post_type>attachment</wp:post_type>
		<wp:attachment_url>http://example.com/wp-content/uploads/hero.webp</wp:attachment_url>
	</item>
	<item>
		<title>Services Parent Page</title>
		<wp:post_id>100</wp:post_id>
		<wp:post_name>services</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[[et_pb_section background_color="#0f172a"][et_pb_row][et_pb_column type="4_4"][et_pb_text]<p>Divi Services Overview</p>[/et_pb_text][/et_pb_column][/et_pb_row][/et_pb_section]]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_thumbnail_id</wp:meta_key>
			<wp:meta_value>500</wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Rust Migration Child Page</title>
		<wp:post_id>200</wp:post_id>
		<wp:post_name>rust-migration</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:post_parent>100</wp:post_parent>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"sec1","elType":"section","settings":{"background_color":"#111827"},"elements":[{"id":"col1","elType":"column","settings":{"_column_size":100},"elements":[{"id":"h1","elType":"widget","widgetType":"heading","settings":{"title":"Sub-Millisecond Rust SSR","header_size":"h1"}}]}]}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
</channel>
</rss>"##;

    let res = import_wxr_xml(sample_xml, &mut conn).unwrap();
    assert_eq!(res.imported_pages, 2);
    assert_eq!(res.imported_media, 1);

    let (media_new_id, media_mime): (i64, String) = conn
        .query_row(
            "SELECT ID, post_mime_type FROM wp_posts WHERE post_name = 'hero-image'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(media_mime, "image/webp");

    let (parent_new_id, parent_content, parent_layout): (i64, String, Option<String>) = conn
        .query_row(
            "SELECT ID, post_content, layout_json FROM wp_posts WHERE post_name = 'services'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert!(parent_layout.is_some());
    assert!(parent_content.contains("Divi Services Overview"));

    // Verify _thumbnail_id was remapped from 500 -> media_new_id
    let thumb_meta: String = conn
        .query_row(
            "SELECT meta_value FROM wp_postmeta WHERE post_id = ?1 AND meta_key = '_thumbnail_id'",
            params![parent_new_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(thumb_meta.parse::<i64>().unwrap(), media_new_id);

    // Verify child page post_parent was remapped from 100 -> parent_new_id and Elementor AST transpiled
    let (child_parent_id, child_content, child_layout): (i64, String, Option<String>) = conn
            .query_row(
                "SELECT post_parent, post_content, layout_json FROM wp_posts WHERE post_name = 'rust-migration'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
    assert_eq!(child_parent_id, parent_new_id);
    assert!(child_layout.is_some());
    assert!(child_content.contains("Sub-Millisecond Rust SSR"));
}
