//! Starter content for a new install: a first post with a comment, a sample page, and one
//! page built with DayStudio, styled with the default Daypress theme.

use chrono::Utc;
use rusqlite::{params, Connection, Result};

pub fn seed_starter_site(conn: &Connection) -> Result<()> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // stderr, so `--mcp-stdio` keeps a clean JSON-RPC stdout.
    eprintln!("⚡ Adding starter content (a sample post and pages)...");

    // 1. Start from an empty content set
    conn.execute_batch(
        r#"
        DELETE FROM posts;
        DELETE FROM term_relationships;
        DELETE FROM postmeta;
        DELETE FROM comments;
        "#,
    )?;

    // 2. Site options & default Daypress theme configuration
    let default_menu = r#"[{"id":1,"title":"Home","url":"/","parent_id":0,"order":0},{"id":2,"title":"Hello world!","url":"/hello-world/","parent_id":0,"order":1},{"id":3,"title":"Sample Page","url":"/sample-page/","parent_id":0,"order":2},{"id":4,"title":"DayStudio","url":"/daystudio-sample/","parent_id":0,"order":3}]"#;
    let options = [
        ("blogname", "Daypress"),
        ("blogdescription", "A self-hosted CMS written in Rust."),
        ("siteurl", "http://localhost:8090"),
        ("home", "http://localhost:8090"),
        ("admin_email", "admin@example.com"),
        ("posts_per_page", "10"),
        ("date_format", "F j, Y"),
        ("time_format", "g:i a"),
        ("daypress_active_theme", "Daypress Daylight"),
        ("stylesheet", "Daypress Daylight"),
        ("daypress_brand_color", "#a8401b"),
        ("daypress_typography_scale", "serif"),
        ("daypress_nav_menu_items", default_menu),
    ];

    for (name, val) in options {
        conn.execute(
            "INSERT OR REPLACE INTO options (option_name, option_value) VALUES (?1, ?2);",
            params![name, val],
        )?;
    }

    // 3. Ensure admin user. The placeholder hash matches no password; first-run onboarding (`/setup`)
    // or `DAYPRESS_ADMIN_PASSWORD` sets a real password.
    conn.execute(
        r#"INSERT OR IGNORE INTO users (ID, user_login, user_pass, user_nicename, user_email, user_registered, display_name)
           VALUES (1, 'admin', ?2, 'admin', 'admin@example.com', ?1, 'admin');"#,
        params![now, crate::auth::LEGACY_SEED_PASSWORD_HASH],
    )?;

    // 4. The default category
    conn.execute(
        "INSERT OR REPLACE INTO terms (term_id, name, slug) VALUES (1, 'Uncategorized', 'uncategorized');",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO term_taxonomy (term_taxonomy_id, term_id, taxonomy, count) VALUES (1, 1, 'category', 1);",
        [],
    )?;

    insert_hello_world(conn, &now)?;
    insert_sample_page(conn, &now)?;
    insert_daystudio_page(conn, &now)?;

    eprintln!("✓ Added starter content. Edit or delete it under Posts and Pages.");
    Ok(())
}

fn insert_post(
    conn: &Connection,
    id: i64,
    post_type: &str,
    slug: &str,
    title: &str,
    content: &str,
    excerpt: &str,
    now: &str,
) -> Result<()> {
    let guid = if post_type == "page" {
        format!("http://localhost:8090/?page_id={id}")
    } else {
        format!("http://localhost:8090/?p={id}")
    };
    conn.execute(
        r#"INSERT INTO posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
           VALUES (?1, 1, ?2, ?2, ?3, ?4, ?5, 'publish', ?6, ?2, ?2, ?7, ?8);"#,
        params![id, now, content, title, excerpt, slug, guid, post_type],
    )?;
    Ok(())
}

fn insert_hello_world(conn: &Connection, now: &str) -> Result<()> {
    let content = r#"<!-- wp:paragraph -->
<p>Welcome to Daypress. This is your first post. Edit or delete it, then start writing!</p>
<!-- /wp:paragraph -->

<!-- wp:paragraph -->
<p>Daypress is a self-hosted publishing platform written in Rust with an embedded SQLite database. Your posts, pages, media, and theme templates live together in a single portable database file on disk.</p>
<!-- /wp:paragraph -->

<!-- wp:heading -->
<h2>First steps for your new site</h2>
<!-- /wp:heading -->

<!-- wp:list -->
<ul>
<li><strong>Write and publish:</strong> Open the <a href="/d-admin/edit">Posts screen</a> in the admin desk to write new articles, organize categories and tags, or edit this post.</li>
<li><strong>Design pages visually:</strong> Open <a href="/d-admin/builder?post=3">DayStudio</a> to arrange sections, grids, and blocks directly on the canvas.</li>
<li><strong>Shape your theme:</strong> Visit the <a href="/d-admin/customize">Theme Customizer</a> or <a href="/d-admin/headers-footers">Headers &amp; Footers</a> to adjust your site title, brand accent color, typography, and navigation menus.</li>
<li><strong>Bring an existing site:</strong> Moving from WordPress? Open <a href="/d-admin/import">Tools &rarr; Import</a> to import a WXR XML file or a site bundle.</li>
</ul>
<!-- /wp:list -->

<!-- wp:quote -->
<blockquote class="wp-block-quote"><p>Good publishing software stays out of the way so your words, ideas, and design take center stage.</p></blockquote>
<!-- /wp:quote -->"#;
    insert_post(
        conn,
        1,
        "post",
        "hello-world",
        "Hello world!",
        content,
        "Welcome to Daypress. This is your first post. Edit or delete it, then start writing!",
        now,
    )?;
    conn.execute(
        "INSERT INTO term_relationships (object_id, term_taxonomy_id) VALUES (1, 1);",
        [],
    )?;

    conn.execute(
        r#"INSERT INTO comments (comment_ID, comment_post_ID, comment_author, comment_author_email, comment_author_url, comment_author_IP, comment_date, comment_date_gmt, comment_content, comment_karma, comment_approved, comment_agent, comment_type, comment_parent, user_id)
           VALUES (1, 1, 'A Daypress Commenter', 'commenter@example.com', '', '127.0.0.1', ?1, ?1, ?2, 0, '1', '', 'comment', 0, 0);"#,
        params![
            now,
            "Hi, this is a comment.\nTo moderate, edit, or delete comments, visit the Comments screen in the admin."
        ],
    )?;
    conn.execute("UPDATE posts SET comment_count = 1 WHERE ID = 1;", [])?;
    Ok(())
}

fn insert_sample_page(conn: &Connection, now: &str) -> Result<()> {
    let content = r#"<!-- wp:paragraph -->
<p>This is an example page. Unlike a blog post, a page stays in one place and usually shows up in your site navigation. Many sites start with an About page that introduces the people behind it. It could say something like this:</p>
<!-- /wp:paragraph -->

<!-- wp:quote -->
<blockquote class="wp-block-quote"><p>Hi there! I run a small bakery and start baking bread every morning before sunrise. On this site I share new recipes and our opening hours.</p></blockquote>
<!-- /wp:quote -->

<!-- wp:paragraph -->
<p>...or something like this:</p>
<!-- /wp:paragraph -->

<!-- wp:quote -->
<blockquote class="wp-block-quote"><p>The Example Company was founded in 2010 and builds sturdy garden furniture for people who like to spend their time outside.</p></blockquote>
<!-- /wp:quote -->

<!-- wp:paragraph -->
<p>As a new Daypress user, go to <a href="/d-admin/">the admin dashboard</a> to delete this page and create new pages for your content. Have fun!</p>
<!-- /wp:paragraph -->"#;
    insert_post(
        conn,
        2,
        "page",
        "sample-page",
        "Sample Page",
        content,
        "This is an example page.",
        now,
    )
}

/// DayStudio layout for the sample builder page (ID 3).
pub const DAYSTUDIO_SAMPLE_LAYOUT: &str = r##"{
  "version": 1,
  "root": {
    "id": "root_sample",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_sample_intro",
        "type": "Section",
        "styles": { "background_color": "#fbf8f2", "text_color": "#16171a", "padding_top": "72px", "padding_bottom": "72px", "text_align": "center" },
        "children": [
          {
            "id": "cont_sample_intro",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_sample_title", "type": "Heading", "data": { "text": "A page built with DayStudio", "level": 1 }, "styles": { "font_size": "42px", "margin_bottom": "16px" } },
              { "id": "p_sample_intro", "type": "Text", "data": { "content_html": "<p>This sample page was made with DayStudio, the visual builder. Open it in the builder to see how it is put together, then change it or delete it.</p>" }, "styles": { "font_size": "18px", "text_color": "#5a5852", "margin_bottom": "28px" } },
              { "id": "btn_sample_edit", "type": "Button", "data": { "text": "Edit this page in DayStudio", "url": "/d-admin/builder?post=3" }, "styles": { "background_color": "#a8401b", "text_color": "#fff8f0", "padding_top": "12px", "padding_bottom": "12px", "padding_left": "28px", "padding_right": "28px", "border_radius": "6px" } }
            ]
          }
        ]
      },
      {
        "id": "sec_sample_grid",
        "type": "Section",
        "styles": { "background_color": "#f3eee4", "padding_top": "56px", "padding_bottom": "64px" },
        "children": [
          {
            "id": "cont_sample_grid",
            "type": "Container",
            "styles": {},
            "children": [
              {
                "id": "grid_sample",
                "type": "Grid",
                "data": { "columns": 3, "gap": 24 },
                "styles": {},
                "children": [
                  {
                    "id": "card_sample_sections",
                    "type": "Container",
                    "styles": { "background_color": "#fbf8f2", "padding_top": "24px", "padding_bottom": "24px", "padding_left": "20px", "padding_right": "20px", "border_radius": "6px", "border": "1px solid #d4cdc0" },
                    "children": [
                      { "id": "h_card_sections", "type": "Heading", "data": { "text": "Sections", "level": 3 }, "styles": { "font_size": "20px", "text_color": "#16171a" } },
                      { "id": "p_card_sections", "type": "Text", "data": { "content_html": "<p>Sections run across the full width of the page. The containers inside them hold your content.</p>" }, "styles": { "font_size": "15px", "text_color": "#5a5852", "margin_top": "8px" } }
                    ]
                  },
                  {
                    "id": "card_sample_grids",
                    "type": "Container",
                    "styles": { "background_color": "#fbf8f2", "padding_top": "24px", "padding_bottom": "24px", "padding_left": "20px", "padding_right": "20px", "border_radius": "6px", "border": "1px solid #d4cdc0" },
                    "children": [
                      { "id": "h_card_grids", "type": "Heading", "data": { "text": "Grids", "level": 3 }, "styles": { "font_size": "20px", "text_color": "#16171a" } },
                      { "id": "p_card_grids", "type": "Text", "data": { "content_html": "<p>These three cards sit in a grid with three columns. Change the number of columns in the builder.</p>" }, "styles": { "font_size": "15px", "text_color": "#5a5852", "margin_top": "8px" } }
                    ]
                  },
                  {
                    "id": "card_sample_widgets",
                    "type": "Container",
                    "styles": { "background_color": "#fbf8f2", "padding_top": "24px", "padding_bottom": "24px", "padding_left": "20px", "padding_right": "20px", "border_radius": "6px", "border": "1px solid #d4cdc0" },
                    "children": [
                      { "id": "h_card_widgets", "type": "Heading", "data": { "text": "Widgets", "level": 3 }, "styles": { "font_size": "20px", "text_color": "#16171a" } },
                      { "id": "p_card_widgets", "type": "Text", "data": { "content_html": "<p>Headings, text, buttons, images, video and more. Drag them onto the page and edit them in place.</p>" }, "styles": { "font_size": "15px", "text_color": "#5a5852", "margin_top": "8px" } }
                    ]
                  }
                ]
              }
            ]
          }
        ]
      }
    ]
  }
}"##;

fn insert_daystudio_page(conn: &Connection, now: &str) -> Result<()> {
    // Stored the way DayStudio saves a page: canonical layout JSON plus the server-rendered HTML.
    let layout: daypress_ast::DaypressLayout = serde_json::from_str(DAYSTUDIO_SAMPLE_LAYOUT)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let layout_json = serde_json::to_string(&layout)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let rendered_html = daypress_ast::render_layout_to_html(&layout);

    insert_post(
        conn,
        3,
        "page",
        "daystudio-sample",
        "A page built with DayStudio",
        &rendered_html,
        "A sample page made with the DayStudio visual builder.",
        now,
    )?;
    conn.execute(
        "UPDATE posts SET layout_json = ?1 WHERE ID = 3;",
        params![layout_json],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO postmeta (post_id, meta_key, meta_value) VALUES (3, '_daypress_layout', ?1);",
        params![layout_json],
    )?;
    Ok(())
}
