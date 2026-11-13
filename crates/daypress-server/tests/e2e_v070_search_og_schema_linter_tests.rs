//! End-to-End Integration Test Suite for Daypress v0.7.0:
//! 1. 4 Audit Bugfixes (Night Mode CPT Field Contrast, `/` Canonical + `/home/` 301 Redirect,
//!    Sitemap CPT Coverage + Homepage `og:image`, and Agent Launchpad MCP Tool Names)
//! 2. Native SQLite FTS5 Full-Text Search (BM25 Ranking, Prefix & Levenshtein Typo Tolerance,
//!    Highlighted Snippets, Post-Type Filters, and Public `Cmd+K` Search Modal)
//! 3. Automatic Rust-Rendered `1200x630` PNG & SVG OpenGraph Social Share Cards (`/og-image/{id}.png`)
//! 4. Rich Schema.org JSON-LD `@graph` (`Article`, `FAQPage`, `Product`, `LocalBusiness`, `BreadcrumbList`)
//!    + Auto Table of Contents (TOC) & Reading-Time Widget
//! 5. Real-Time Daystudio WCAG 2.1 Contrast, SQLite Broken-Link & SEO Linter (`/wp-json/daypress/v1/seo/audit`)

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_server::{create_app, CreatePostRequest, CustomPostTypeDefinition, Database};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn body_to_string(res: axum::response::Response) -> String {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

async fn body_to_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).expect("valid JSON response")
}

#[tokio::test]
async fn test_v070_four_audit_bugfixes_canonical_redirect_sitemap_cpt_night_css_and_mcp_names() {
    let db = Database::new_in_memory().expect("in-memory DB");

    // Configure static front page `/home/` -> `/`
    let home_page = db
        .create_post(&CreatePostRequest {
            title: "Daypress Editorial Front Page".to_string(),
            content: "<section class=\"dp-section\"><h1>Sub-Millisecond Rust CMS</h1></section>"
                .to_string(),
            slug: Some("home".to_string()),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            ..Default::default()
        })
        .unwrap();
    db.set_option("show_on_front", "page").unwrap();
    db.set_option("page_on_front", &home_page.id.to_string())
        .unwrap();

    // Register a `feature` Custom Post Type and create a published `/feature/fts5-search/` item with ACF fields
    db.save_post_type_definition(&CustomPostTypeDefinition {
        slug: "feature".to_string(),
        label_singular: "Feature".to_string(),
        label_plural: "Features".to_string(),
        public: true,
        has_archive: true,
        rewrite_slug: "feature".to_string(),
        ..Default::default()
    })
    .unwrap();

    let mut acf_map = serde_json::Map::new();
    acf_map.insert("latency_target".to_string(), json!("< 50 microseconds"));
    acf_map.insert("engine_tier".to_string(), json!("SQLite FTS5 BM25"));

    let feature_post = db
        .create_post(&CreatePostRequest {
            title: "FTS5 Instant Search Engine".to_string(),
            content: "<p>Full-text search with BM25 ranking and typo tolerance.</p>".to_string(),
            slug: Some("fts5-search".to_string()),
            status: "publish".to_string(),
            post_type: "feature".to_string(),
            acf: Some(Value::Object(acf_map)),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        feature_post.link,
        "http://localhost:8090/feature/fts5-search/"
    );

    let app = create_app(db.clone());

    // Bugfix 1: `/home/` redirects 301 to `/`, and `/` has canonical `http://localhost:8090/` + auto `og:image`
    let home_slug_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/home/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(home_slug_res.status(), StatusCode::MOVED_PERMANENTLY);
    assert_eq!(
        home_slug_res
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/"
    );

    let root_res = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(root_res.status(), StatusCode::OK);
    let root_html = body_to_string(root_res).await;
    assert!(
        root_html.contains(r#"<link rel="canonical" href="http://localhost:8090/" />"#),
        "Homepage canonical must point to root /"
    );
    assert!(
        root_html.contains(
            r#"<meta property="og:image" content="http://localhost:8090/og-image/home.png" />"#
        ),
        "Homepage must emit auto og:image when seo_og_image and featured_image are empty"
    );

    // Bugfix 2: Night mode `[data-theme="dark"]` CSS variables and CPT structured field summary contrast
    let feat_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/feature/fts5-search/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feat_res.status(), StatusCode::OK);
    let feat_html = body_to_string(feat_res).await;
    assert!(feat_html.contains(r#":root[data-theme="dark"]"#));
    assert!(feat_html.contains("color:var(--ink, var(--text-main))"));
    assert!(feat_html.contains("SQLite FTS5 BM25"));

    // Bugfix 3: `sitemap.xml` omits `/home/` duplicate and includes `/feature/fts5-search/` + `/feature/`
    let sitemap_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/sitemap.xml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sitemap_res.status(), StatusCode::OK);
    let sitemap_xml = body_to_string(sitemap_res).await;
    assert!(sitemap_xml.contains("<loc>http://localhost:8090/</loc>"));
    assert!(
        !sitemap_xml.contains("<loc>http://localhost:8090/home/</loc>"),
        "sitemap.xml must not list /home/ duplicate when home is page_on_front"
    );
    assert!(
        sitemap_xml.contains("<loc>http://localhost:8090/feature/fts5-search/</loc>"),
        "sitemap.xml must include published CPT items"
    );
    assert!(
        sitemap_xml.contains("<loc>http://localhost:8090/feature/</loc>"),
        "sitemap.xml must include public CPT archive routes"
    );

    // Bugfix 4: Agent connection prompt in `/d-admin/getting-started` lists `daypress_get_post_layout` & `daypress_update_post_layout`
    let dash_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/d-admin/getting-started")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dash_res.status(), StatusCode::OK);
    let dash_html = body_to_string(dash_res).await;
    assert!(dash_html.contains("daypress_get_post_layout"));
    assert!(dash_html.contains("daypress_update_post_layout"));
    assert!(!dash_html.contains("daypress_get_layout_ast"));
}

#[tokio::test]
async fn test_v070_sqlite_fts5_bm25_search_typo_tolerance_and_cmdk_modal() {
    let db = Database::new_in_memory().expect("in-memory DB");

    db.create_post(
        &CreatePostRequest {
            title: "Sub-Millisecond Rust Edge Architecture".to_string(),
            content: "<p>Daypress compiles Elementor, Divi, and Gutenberg into a zero-overhead unified AST backed by Rusqlite WAL.</p>".to_string(),
            slug: Some("rust-edge-architecture".to_string()),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            ..Default::default()
        })
    .unwrap();

    db.create_post(
        &CreatePostRequest {
            title: "Migrating Legacy WordPress WooCommerce Stores".to_string(),
            content: "<p>One-click WXR and JSON bundle migration with deterministic relational ID mapping.</p>".to_string(),
            slug: Some("migrating-wordpress".to_string()),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            ..Default::default()
        })
    .unwrap();

    let app = create_app(db.clone());

    // 1. Exact & prefix FTS5 BM25 query with `<mark class="dp-search-hl">` snippet highlighting
    let search_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/search?q=Rusqlite")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(search_res.status(), StatusCode::OK);
    let search_json = body_to_json(search_res).await;
    assert_eq!(search_json["engine"], "sqlite-fts5-bm25");
    assert_eq!(search_json["total"], 1);
    assert_eq!(search_json["results"][0]["slug"], "rust-edge-architecture");
    assert!(search_json["results"][0]["snippet_html"]
        .as_str()
        .unwrap()
        .contains("<mark class=\"dp-search-hl\">"));

    // 2. Levenshtein typo-tolerant query ("gutenbrg" -> matches "Gutenberg")
    let typo_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/search?q=gutenbrg")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(typo_res.status(), StatusCode::OK);
    let typo_json = body_to_json(typo_res).await;
    assert_eq!(typo_json["total"], 1);
    assert_eq!(typo_json["results"][0]["slug"], "rust-edge-architecture");
    assert_eq!(typo_json["results"][0]["typo_matched"], true);

    // 3. Post-type filter (`?post_type=page`)
    let page_filter_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/search?q=migration&post_type=page")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let page_filter_json = body_to_json(page_filter_res).await;
    assert_eq!(page_filter_json["total"], 1);
    assert_eq!(
        page_filter_json["results"][0]["slug"],
        "migrating-wordpress"
    );

    // 4. Public SSR page includes the Cmd+K Search Modal and header trigger button
    let pub_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/rust-edge-architecture/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let pub_html = body_to_string(pub_res).await;
    assert!(pub_html.contains("id=\"dp-cmdk-modal\""));
    assert!(pub_html.contains("id=\"dp-header-search-trigger\""));
    assert!(pub_html.contains("/wp-json/daypress/v1/search"));
}

#[tokio::test]
async fn test_v070_auto_og_social_cards_schema_json_ld_and_auto_toc_reading_time() {
    let db = Database::new_in_memory().expect("in-memory DB");

    let mut meta_map = serde_json::Map::new();
    meta_map.insert("seo_schema_type".to_string(), json!("FAQPage"));

    let article = db
        .create_post(
            &CreatePostRequest {
                title: "Deep Dive into Daypress 0.7 Architecture".to_string(),
                content: r#"
                    <p>Welcome to the comprehensive engineering guide for Daypress 0.7.</p>
                    <h2>Why SQLite FTS5 Beats External Search Clusters</h2>
                    <p>How does SQLite FTS5 achieve sub-millisecond latency? By running in-process alongside Rusqlite WAL with zero network hops.</p>
                    <h3>BM25 Weighting and Snippet Extraction</h3>
                    <p>What is the default BM25 column weighting? Title is weighted 10x, excerpt 5x, slug 4x, and body 1x.</p>
                    <h2>Automatic Rust OpenGraph Social Cards</h2>
                    <p>Every post and page renders a crisp 1200x630 editorial card on the fly.</p>
                "#
                .to_string(),
                slug: Some("daypress-07-architecture".to_string()),
                status: "publish".to_string(),
                post_type: "post".to_string(),
                meta: Some(Value::Object(meta_map)),
                ..Default::default()
            })
        .unwrap();

    let app = create_app(db.clone());

    // 1. Verify `/og-image/{id}.png` renders a valid 1200x630 PNG image
    let og_png_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/og-image/{}.png", article.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(og_png_res.status(), StatusCode::OK);
    assert_eq!(
        og_png_res.headers().get("content-type").unwrap(),
        "image/png"
    );
    let png_bytes = to_bytes(og_png_res.into_body(), usize::MAX).await.unwrap();
    assert!(
        png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "Response must be a valid PNG binary"
    );
    let decoded = image::load_from_memory(&png_bytes).expect("decode PNG");
    assert_eq!(decoded.width(), 1200);
    assert_eq!(decoded.height(), 630);

    // 2. Verify `/og-image/{id}.svg` renders the Daypress Editorial SVG card
    let og_svg_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/og-image/{}.svg", article.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(og_svg_res.status(), StatusCode::OK);
    let svg_str = body_to_string(og_svg_res).await;
    assert!(svg_str.contains("<svg"));
    assert!(svg_str.contains("Deep Dive into Daypress"));
    assert!(svg_str.contains("Architecture"));

    // 3. Verify Single Post SSR includes Auto-OG meta tags, Schema.org `@graph` (`BlogPosting` + `FAQPage` + `BreadcrumbList`),
    //    Reading-Time badge (`dp-reading-time`), Auto-TOC (`dp-auto-toc`), and injected heading anchor IDs
    let post_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/daypress-07-architecture/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(post_res.status(), StatusCode::OK);
    let post_html = body_to_string(post_res).await;

    assert!(post_html.contains(&format!(
        r#"<meta property="og:image" content="http://localhost:8090/og-image/{}.png" />"#,
        article.id
    )));
    assert!(post_html.contains(r#"<script type="application/ld+json">"#));
    assert!(post_html.contains(r#""@graph""#));
    assert!(post_html.contains(r#""@type": "FAQPage""#));
    assert!(post_html.contains(r#""@type": "BreadcrumbList""#));
    assert!(
        post_html.contains("class=\"dp-toc dp-auto-toc\""),
        "Blog post with >= 2 headings must auto-render Table of Contents"
    );
    assert!(
        post_html.contains("id=\"toc-why-sqlite-fts5-beats-external-search-clusters\""),
        "Headings must receive deterministic id anchor attributes"
    );
    assert!(
        post_html.contains("class=\"dp-reading-time\""),
        "Single post header must include reading-time badge"
    );
}

#[tokio::test]
async fn test_v070_daystudio_wcag_contrast_broken_link_and_seo_linter() {
    let db = Database::new_in_memory().expect("in-memory DB");

    let page = db
        .create_post(&CreatePostRequest {
            title: "Landing Page Under Audit".to_string(),
            content: "<p>Initial content</p>".to_string(),
            slug: Some("audit-landing".to_string()),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            ..Default::default()
        })
        .unwrap();

    // Layout AST containing:
    // 1. Low-contrast text (`#d1d5db` on `#ffffff` -> ~1.5:1 < 4.5:1 AA)
    // 2. Image missing `alt` text
    // 3. Skipped heading hierarchy (`H1` -> `H3`)
    // 4. Button with a broken internal link (`/missing-ghost-page/`) and a valid link (`/hello-world/`)
    let layout_json = json!({
        "version": 1,
        "root": {
            "id": "root",
            "type": "Root",
            "children": [
                {
                    "id": "sec_audit",
                    "type": "Section",
                    "styles": { "background_color": "#ffffff" },
                    "children": [
                        {
                            "id": "h1_main",
                            "type": "Heading",
                            "data": { "text": "Primary Hero Heading", "level": 1 },
                            "styles": { "text_color": "#16171a" }
                        },
                        {
                            "id": "h3_skipped",
                            "type": "Heading",
                            "data": { "text": "Skipped Subheading Level", "level": 3 },
                            "styles": { "text_color": "#16171a" }
                        },
                        {
                            "id": "txt_low_contrast",
                            "type": "Text",
                            "data": { "content_html": "<p>Unreadable pale gray copy on white background</p>" },
                            "styles": { "text_color": "#d1d5db" }
                        },
                        {
                            "id": "img_no_alt",
                            "type": "Image",
                            "data": { "src": "/uploads/editorial-diagram.webp", "alt": "" }
                        },
                        {
                            "id": "btn_broken",
                            "type": "Button",
                            "data": { "text": "Broken CTA", "url": "/missing-ghost-page/", "target": "_self" }
                        },
                        {
                            "id": "btn_valid",
                            "type": "Button",
                            "data": { "text": "Valid CTA", "url": "/hello-world/", "target": "_self" }
                        }
                    ]
                }
            ]
        }
    })
    .to_string();

    let app = create_app(db.clone());

    let audit_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/seo/audit")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "post_id": page.id,
                        "title": "Landing Page Under Audit",
                        "slug": "audit-landing",
                        "layout_json": layout_json,
                        "theme": "light"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_res.status(), StatusCode::OK);
    let report = body_to_json(audit_res).await;

    assert!(report["score"].as_u64().unwrap() < 90);
    let issues = report["issues"].as_array().unwrap();

    // Must flag low contrast on `txt_low_contrast` with Auto-Fix `#16171a`
    assert!(
        issues.iter().any(|i| i["node_id"] == "txt_low_contrast"
            && i["auto_fix_kind"] == "contrast_text"
            && i["auto_fix_value"] == "#16171a"),
        "Linter must flag low WCAG contrast on txt_low_contrast with auto-fix"
    );

    // Must flag missing image alt on `img_no_alt` with inferred Auto-Fix
    assert!(
        issues.iter().any(|i| i["node_id"] == "img_no_alt"
            && i["auto_fix_kind"] == "image_alt"
            && i["auto_fix_value"] == "editorial diagram"),
        "Linter must flag missing image alt on img_no_alt with inferred filename alt"
    );

    // Must flag skipped heading level `H1 -> H3` on `h3_skipped` with Auto-Fix `"2"`
    assert!(
        issues.iter().any(|i| i["node_id"] == "h3_skipped"
            && i["auto_fix_kind"] == "heading_level"
            && i["auto_fix_value"] == "2"),
        "Linter must flag skipped heading hierarchy H1 -> H3"
    );

    // Must flag broken internal link `/missing-ghost-page/` while passing `/hello-world/`
    assert!(
        issues
            .iter()
            .any(|i| i["node_id"] == "btn_broken" && i["category"] == "links"),
        "Linter must flag broken internal link /missing-ghost-page/"
    );
    let verified_links = report["verified_links"].as_array().unwrap();
    assert!(verified_links
        .iter()
        .any(|l| l["href"] == "/hello-world/" && l["valid"] == true));
    assert!(verified_links
        .iter()
        .any(|l| l["href"] == "/missing-ghost-page/" && l["valid"] == false));

    // Verify Daystudio Visual Builder shell renders the Linter topbar pill and left-rail button
    let builder_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/d-admin/builder?post_id={}", page.id))
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(builder_res.status(), StatusCode::OK);
    let builder_html = body_to_string(builder_res).await;
    assert!(builder_html.contains("id=\"btn-toggle-linter\""));
    assert!(builder_html.contains("id=\"rail-btn-linter\""));
    assert!(builder_html.contains("runDaystudioLinter"));
}

#[tokio::test]
async fn test_v070_visual_browser_screenshots() {
    let Ok(out_dir) = std::env::var("DAYPRESS_VISUAL_SCREENSHOT_DIR") else {
        return;
    };
    let out_path = std::path::PathBuf::from(&out_dir);
    std::fs::create_dir_all(&out_path).unwrap();

    let db = Database::new_in_memory().expect("in-memory DB");

    // 1. Create a rich blog post with multiple H2/H3 sections for Auto-TOC, Reading-Time, and FTS5 Search
    let post = db
        .create_post(&CreatePostRequest {
            title: "Daypress 0.7: SQLite FTS5 Search & Live WCAG Linter".to_string(),
            content: r#"
                <p>Daypress 0.7 introduces sub-millisecond full-text search powered by SQLite FTS5, automatic Rust-rendered OpenGraph social cards, rich Schema.org JSON-LD graphs, and a real-time WCAG 2.1 contrast and broken-link linter inside Daystudio.</p>
                <h2>Native SQLite FTS5 with BM25 Ranking</h2>
                <p>Every published post, page, and Custom Post Type is indexed automatically in an embedded FTS5 virtual table with prefix matching, Levenshtein typo tolerance, and highlighted context snippets.</p>
                <h3>Instant Cmd+K Public Search Modal</h3>
                <p>Visitors can press Cmd+K or Ctrl+K on any page to launch the editorial search dialog with post-type filter pills and keyboard navigation.</p>
                <h2>Automatic Rust-Rendered 1200x630 Social Cards</h2>
                <p>When a post or page does not specify a custom social image, Daypress synthesizes a crisp 1200x630 PNG card directly in Rust using the active Daylight or Night palette.</p>
                <h2>Real-Time Daystudio WCAG &amp; Broken-Link Linter</h2>
                <p>Authors get live feedback on AA/AAA contrast ratios, missing image alt text, heading hierarchy, and SQLite-verified internal links with 1-click Auto-Fix.</p>
                <script>
                if (window.location.search.indexOf('open_search=1') !== -1) {
                    window.addEventListener('DOMContentLoaded', function() {
                        if (window.daypressOpenSearch) window.daypressOpenSearch('FTS5 BM25');
                    });
                }
                if (window.location.search.indexOf('night=1') !== -1) {
                    document.documentElement.setAttribute('data-theme', 'dark');
                }
                </script>
            "#
            .to_string(),
            slug: Some("daypress-07-launch".to_string()),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            ..Default::default()
        })
        .unwrap();

    // 2. Register `feature` CPT and create a CPT item with structured fields for Night Mode verification
    db.save_post_type_definition(&CustomPostTypeDefinition {
        slug: "feature".to_string(),
        label_singular: "Feature".to_string(),
        label_plural: "Features".to_string(),
        public: true,
        has_archive: true,
        rewrite_slug: "feature".to_string(),
        ..Default::default()
    })
    .unwrap();

    let mut acf_map = serde_json::Map::new();
    acf_map.insert("search_engine".to_string(), json!("SQLite FTS5 + BM25"));
    acf_map.insert("p99_latency".to_string(), json!("42 microseconds"));
    acf_map.insert("wcag_compliance".to_string(), json!("WCAG 2.1 AA / AAA"));

    db.create_post(&CreatePostRequest {
        title: "Sub-Millisecond FTS5 Search & Auto-OG Cards".to_string(),
        content: r#"
            <p>Full-text search and structured field cards adapt seamlessly between Daylight Editorial and Night Desk modes.</p>
            <script>document.documentElement.setAttribute('data-theme', 'dark');</script>
        "#
        .to_string(),
        slug: Some("fts5-instant-search".to_string()),
        status: "publish".to_string(),
        post_type: "feature".to_string(),
        acf: Some(Value::Object(acf_map)),
        ..Default::default()
    })
    .unwrap();

    // 3. Save a Visual Builder layout on `post` with a deliberate contrast warning & broken link so the Daystudio Linter drawer shows live issues
    let builder_page = db
        .create_post(&CreatePostRequest {
            title: "Editorial Landing Page".to_string(),
            content: "<p>Visual builder page</p>".to_string(),
            slug: Some("editorial-landing".to_string()),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            ..Default::default()
        })
        .unwrap();

    let builder_ast = json!({
        "version": 1,
        "root": {
            "id": "root",
            "type": "Root",
            "children": [
                {
                    "id": "sec_hero",
                    "type": "Section",
                    "styles": { "background_color": "#fbf8f2", "padding_top": "56px", "padding_bottom": "56px" },
                    "children": [
                        {
                            "id": "cont_hero",
                            "type": "Container",
                            "children": [
                                {
                                    "id": "h1_hero",
                                    "type": "Heading",
                                    "data": { "text": "Precision Rust Publishing", "level": 1 },
                                    "styles": { "font_size": "44px", "text_color": "#16171a", "margin_bottom": "16px" }
                                },
                                {
                                    "id": "h3_skipped",
                                    "type": "Heading",
                                    "data": { "text": "Sub-Millisecond SQLite FTS5 & WCAG Linter", "level": 3 },
                                    "styles": { "font_size": "20px", "text_color": "#a8401b", "margin_bottom": "16px" }
                                },
                                {
                                    "id": "txt_low_contrast",
                                    "type": "Text",
                                    "data": { "content_html": "<p>Low contrast sample caption that triggers the WCAG 2.1 AA contrast linter with 1-click Auto-Fix.</p>" },
                                    "styles": { "text_color": "#b8b2a6", "margin_bottom": "20px" }
                                },
                                {
                                    "id": "btn_broken_demo",
                                    "type": "Button",
                                    "data": { "text": "Explore Documentation", "url": "/non-existent-docs-route/", "target": "_self" }
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    })
    .to_string();
    db.save_post_layout(builder_page.id, &builder_ast, None)
        .unwrap();

    // Save the generated 1200x630 PNG OG Card directly
    let app = create_app(db.clone());
    let og_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/og-image/{}.png", post.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let og_bytes = to_bytes(og_res.into_body(), usize::MAX).await.unwrap();
    std::fs::write(out_path.join("v070_og_card.png"), &og_bytes).unwrap();

    // Also save Night-mode 1200x630 PNG OG Card
    let og_dark_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/og-image/{}.png?theme=dark", post.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let og_dark_bytes = to_bytes(og_dark_res.into_body(), usize::MAX).await.unwrap();
    std::fs::write(out_path.join("v070_og_card_dark.png"), &og_dark_bytes).unwrap();

    // Fetch real FTS5 search results for "FTS5" so the Cmd+K modal preview shows real SQLite FTS5 results
    let search_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/search?q=FTS5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let search_json_str = body_to_string(search_res).await;

    // Fetch real SSR HTML for the public blog post (Auto-TOC + Reading-Time)
    let public_post_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/daypress-07-launch/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let public_post_html = body_to_string(public_post_res).await;

    // Fetch real SSR HTML for the CPT item in Night Mode
    let cpt_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/feature/fts5-instant-search/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let cpt_html = body_to_string(cpt_res).await;

    // Fetch real SSR HTML for Daystudio Builder with the Linter Drawer
    let builder_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/d-admin/builder?post_id={}", builder_page.id))
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let builder_html_raw = body_to_string(builder_res).await;

    // Fetch real SQLite Linter report for `builder_page` so the preview shows SQLite broken-link verification too
    let linter_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/seo/audit")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "post_id": builder_page.id,
                        "layout_json": builder_ast,
                        "theme": "light",
                        "links": ["/non-existent-docs-route/"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let linter_json_str = body_to_string(linter_res).await;

    // Build local @font-face CSS pointing to crates/daypress-server/assets/fonts/*.ttf
    let fonts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts");
    let fonts_url = fonts_dir.display().to_string().replace('\\', "/");
    let local_font_css = format!(
        r#"<style id="dp-local-fonts">
@font-face {{ font-family: 'Newsreader'; font-weight: 400 700; src: url('file:///{fonts_url}/Newsreader-SemiBold.ttf') format('truetype'); }}
@font-face {{ font-family: 'Schibsted Grotesk'; font-weight: 400 700; src: url('file:///{fonts_url}/SchibstedGrotesk-Medium.ttf') format('truetype'); }}
@font-face {{ font-family: 'IBM Plex Mono'; font-weight: 400 700; src: url('file:///{fonts_url}/IBMPlexMono-Medium.ttf') format('truetype'); }}
</style>"#
    );

    let strip_external = |html: &str| -> String {
        let re = regex::Regex::new(r#"<link[^>]*fonts\.(googleapis|gstatic)\.com[^>]*>|<link[^>]*cdnjs\.cloudflare\.com[^>]*>"#).unwrap();
        re.replace_all(html, "")
            .into_owned()
            .replace("<head>", &format!("<head>{local_font_css}"))
    };

    std::fs::write(
        out_path.join("v070_public_toc.html"),
        strip_external(&public_post_html),
    )
    .unwrap();

    let cmdk_html = strip_external(&public_post_html).replace(
        "</body>",
        &format!(
            r#"<script>window.DAYPRESS_PRELOADED_SEARCH_RESPONSE = {search_json_str}; window.addEventListener('DOMContentLoaded', function(){{ if (window.daypressOpenSearch) window.daypressOpenSearch('FTS5'); }});</script></body>"#
        ),
    );
    std::fs::write(out_path.join("v070_cmdk_search.html"), cmdk_html).unwrap();

    std::fs::write(
        out_path.join("v070_cpt_night_mode.html"),
        strip_external(&cpt_html),
    )
    .unwrap();

    let builder_preview_html = strip_external(&builder_html_raw).replace(
        "</body>",
        &format!(
            r#"<script>window.DAYPRESS_PRELOADED_LINTER_REPORT = {linter_json_str}; window.addEventListener('DOMContentLoaded', function(){{ if (window.runDaystudioLinter) window.runDaystudioLinter(); if (window.openLinterDrawer) window.openLinterDrawer(); }});</script></body>"#
        ),
    );
    std::fs::write(
        out_path.join("v070_builder_preview.html"),
        builder_preview_html,
    )
    .unwrap();
}
