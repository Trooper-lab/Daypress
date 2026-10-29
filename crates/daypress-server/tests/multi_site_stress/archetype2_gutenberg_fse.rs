//! Archetype 2: Gutenberg FSE / Block Theme Ingestion & SSR Parity.

use super::common::{
    assert_zero_comment_leaks, body_to_string, load_fixture_or_fallback, setup_test_db,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use daypress_ast::{
    compile_theme_json_to_css, parse_theme_json, render_layout_to_html, transpile_gutenberg_content,
};
use daypress_server::create_app;
use daypress_server::domain::models::CreatePostRequest;
use tower::ServiceExt;

const FALLBACK_THEME_JSON: &str = r##"{
  "version": 3,
  "settings": {
    "color": {
      "palette": [
        { "name": "Primary", "slug": "primary", "color": "#1d4ed8" },
        { "name": "Secondary", "slug": "secondary", "color": "#f59e0b" },
        { "name": "Accent", "slug": "accent", "color": "#10b981" }
      ]
    },
    "typography": {
      "fontSizes": [
        { "name": "Small", "slug": "small", "size": "14px" },
        { "name": "Medium", "slug": "medium", "size": "1rem" },
        { "name": "Hero", "slug": "hero", "size": "48px" }
      ]
    },
    "spacing": {
      "spacingSizes": [
        { "name": "Small", "slug": "30", "size": "1rem" },
        { "name": "Large", "slug": "70", "size": "3rem" }
      ]
    },
    "layout": {
      "contentSize": "800px",
      "wideSize": "1200px"
    }
  }
}"##;

pub async fn run_test() {
    let raw_theme_json = load_fixture_or_fallback(
        "archetype2_gutenberg_fse/theme/theme.json",
        FALLBACK_THEME_JSON,
    );

    // 1. theme.json Parser & CSS Custom Properties Compilation
    let theme = parse_theme_json(&raw_theme_json).expect("theme.json must parse successfully");
    assert_eq!(theme.version, 3);

    let css = compile_theme_json_to_css(&raw_theme_json).expect("compile_theme_json_to_css failed");
    assert!(css.contains("--wp--preset--color--primary: #1d4ed8;"));
    assert!(css.contains("--wp--preset--color--accent:"));
    assert!(css.contains("--wp--style--global--content-size: 800px;"));
    assert!(
        css.contains(".has-primary-color { color: var(--wp--preset--color--primary) !important; }")
    );

    // 2. FSE Block Template with Template Part Transpilation
    let fse_template = r#"
<!-- wp:template-part {"slug":"header","tagName":"header"} /-->
<!-- wp:group {"layout":{"type":"constrained"}} -->
<div class="wp-block-group">
    <!-- wp:heading {"level":1} -->
    <h2>Twenty Twenty-Four FSE</h2>
    <!-- /wp:heading -->
    <!-- wp:paragraph -->
    <p class="has-primary-color has-secondary-color">Twenty Twenty-Four FSE block theme verification.</p>
    <!-- /wp:paragraph -->
</div>
<!-- /wp:group -->
<!-- wp:template-part {"slug":"footer","tagName":"footer"} /-->
"#;

    let layout =
        transpile_gutenberg_content(fse_template).expect("FSE content transpilation succeeds");
    let html = render_layout_to_html(&layout);
    assert_zero_comment_leaks(&html, "Archetype 2 FSE Layout");
    assert!(html.contains("Twenty Twenty-Four FSE"));
    assert!(html.contains("has-primary-color"));

    // 3. SSR Route with Enqueued FSE Custom Properties
    let db = setup_test_db();
    let _ = db.set_option("daypress_fse_theme_json_css", &css);

    let _ = db
        .create_post(&CreatePostRequest {
            title: "FSE Page".to_string(),
            content: html.clone(),
            status: "publish".to_string(),
            slug: Some("fse-page".to_string()),
            post_type: "page".to_string(),
            ..Default::default()
        })
        .expect("Failed to create FSE post");

    let app = create_app(db.clone());
    let req = Request::builder()
        .uri("/fse-page/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ssr_html = body_to_string(resp).await;

    assert_zero_comment_leaks(&ssr_html, "Archetype 2 SSR /fse-page/");
    assert!(ssr_html.contains("Twenty Twenty-Four FSE"));
}
