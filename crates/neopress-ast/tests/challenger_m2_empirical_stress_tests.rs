//! Challenger Empirical Stress & Verification Test Suite for Milestone 2
//! Tests Gutenberg balanced-brace tokenizer, core block mappers, theme.json compiler,
//! and leak prevention under adversarial conditions.

use neopress_ast::{
    compile_theme_json_to_css, parse_theme_json, render_layout_to_html, transpile_content,
    transpile_gutenberg_content, NodeKind,
};
use std::collections::HashMap;

#[test]
fn test_adversarial_multiline_json_and_spacing_attributes() {
    let input = r#"
<!-- wp:columns {
  "metadata": {
    "name": "Nested Test Columns"
  },
  "style": {
    "spacing": {
      "padding": {
        "top": "3.5rem",
        "bottom": "3.5rem",
        "left": "2rem",
        "right": "2rem"
      },
      "blockGap": {
        "top": "24px",
        "left": "24px"
      }
    }
  }
} -->
<div class="wp-block-columns">
  <!-- wp:column {
    "width": "66.66%"
  } -->
  <div class="wp-block-column">
    <!-- wp:paragraph {"fontSize":"large"} -->
    <p class="has-large-font-size">Main column body</p>
    <!-- /wp:paragraph -->
  </div>
  <!-- /wp:column -->
  <!-- wp:column {
    "width": "33.33%"
  } -->
  <div class="wp-block-column">
    <!-- wp:paragraph -->
    <p>Sidebar content</p>
    <!-- /wp:paragraph -->
  </div>
  <!-- /wp:column -->
</div>
<!-- /wp:columns -->
"#;

    let layout = transpile_gutenberg_content(input).expect("Should transpile multiline JSON");
    assert_eq!(
        layout.root.children.len(),
        1,
        "Root should have 1 columns block"
    );
    let columns = &layout.root.children[0];
    assert!(
        matches!(columns.kind, NodeKind::Flex { .. }),
        "Columns block should be mapped to Flex"
    );
    assert_eq!(
        columns.children.len(),
        2,
        "Columns should contain 2 column children"
    );

    // Check column 1
    let col1 = &columns.children[0];
    if let NodeKind::Flex { basis, .. } = &col1.kind {
        assert_eq!(basis.as_deref(), Some("66.66%"));
    } else {
        panic!("Col 1 should be NodeKind::Flex");
    }

    // Check column 2
    let col2 = &columns.children[1];
    if let NodeKind::Flex { basis, .. } = &col2.kind {
        assert_eq!(basis.as_deref(), Some("33.33%"));
    } else {
        panic!("Col 2 should be NodeKind::Flex");
    }

    // Render HTML and assert zero leaks
    let html = render_layout_to_html(&layout);
    assert!(!html.contains("<!-- wp:"), "Leaked raw open wp comment");
    assert!(!html.contains("<!-- /wp:"), "Leaked raw close wp comment");
    assert!(html.contains("Main column body"));
    assert!(html.contains("Sidebar content"));
}

#[test]
fn test_adversarial_json_string_literals_with_braces_and_escapes() {
    let input = r#"
<!-- wp:paragraph {"placeholder":"{user_name} has {profile: \"active\"}", "metadata":{"tags":["{mustache}", "{brace: inside}"]}} -->
<p>Hello {user_name} with \"escaped quote\" and backslash \\ and {closing: brace}.</p>
<!-- /wp:paragraph -->
<!-- wp:heading {"level":3,"placeholder":"Title with {brackets} and : colons"} -->
<h3>Title with {brackets} and : colons</h3>
<!-- /wp:heading -->
"#;

    let layout = transpile_gutenberg_content(input).expect("Should parse strings with braces");
    assert_eq!(
        layout.root.children.len(),
        2,
        "Should parse paragraph and heading"
    );

    let p = &layout.root.children[0];
    assert!(matches!(p.kind, NodeKind::Text { .. }));

    let h = &layout.root.children[1];
    if let NodeKind::Heading { text, level } = &h.kind {
        assert_eq!(*level, 3);
        assert_eq!(text, "Title with {brackets} and : colons");
    } else {
        panic!("Expected Heading node");
    }

    let html = render_layout_to_html(&layout);
    assert!(!html.contains("<!-- wp:"));
    assert!(!html.contains("<!-- /wp:"));
    assert!(html.contains("Hello {user_name}"));
    assert!(html.contains("Title with {brackets}"));
}

#[test]
fn test_deeply_nested_containers_and_embedded_html_comments() {
    let input = r#"
<!-- wp:columns -->
<div class="wp-block-columns">
  <!-- wp:column -->
  <div class="wp-block-column">
    <!-- Non-Gutenberg standard HTML comment: <!-- nested test --> should be preserved or ignored -->
    <!-- wp:columns -->
    <div class="wp-block-columns">
      <!-- wp:column -->
      <div class="wp-block-column">
        <!-- wp:paragraph -->
        <p>Deep level 3 content</p>
        <!-- /wp:paragraph -->
      </div>
      <!-- /wp:column -->
    </div>
    <!-- /wp:columns -->
  </div>
  <!-- /wp:column -->
</div>
<!-- /wp:columns -->
"#;

    let layout =
        transpile_gutenberg_content(input).expect("Should parse nested columns 3 levels deep");
    assert_eq!(layout.root.children.len(), 1);
    let level1 = &layout.root.children[0];
    assert_eq!(level1.children.len(), 1);
    let col1 = &level1.children[0];
    assert_eq!(col1.children.len(), 1);
    let level2 = &col1.children[0];
    assert_eq!(level2.children.len(), 1);
    let col2 = &level2.children[0];
    assert_eq!(col2.children.len(), 1);
    let p = &col2.children[0];
    assert!(matches!(p.kind, NodeKind::Text { .. }));

    let html = render_layout_to_html(&layout);
    assert!(!html.contains("<!-- wp:"));
    assert!(!html.contains("<!-- /wp:"));
    assert!(html.contains("Deep level 3 content"));
}

#[test]
fn test_core_gallery_modern_and_legacy() {
    // 1. Modern Gutenberg Gallery with child images
    let modern_gallery = r#"
<!-- wp:gallery {"columns":2,"linkTo":"media","style":{"spacing":{"blockGap":"20px"}}} -->
<figure class="wp-block-gallery has-nested-images columns-2">
  <!-- wp:image {"id":10,"sizeSlug":"large","linkDestination":"media"} -->
  <figure class="wp-block-image size-large"><a href="https://example.com/img1.jpg"><img src="https://example.com/img1.jpg" alt="Image 1"/></a></figure>
  <!-- /wp:image -->
  <!-- wp:image {"id":11,"sizeSlug":"large","linkDestination":"media"} -->
  <figure class="wp-block-image size-large"><a href="https://example.com/img2.jpg"><img src="https://example.com/img2.jpg" alt="Image 2"/></a></figure>
  <!-- /wp:image -->
</figure>
<!-- /wp:gallery -->
"#;

    let layout = transpile_gutenberg_content(modern_gallery).expect("Transpile modern gallery");
    let gallery_node = &layout.root.children[0];
    if let NodeKind::Grid { columns, gap } = &gallery_node.kind {
        assert_eq!(*columns, 2, "Columns should be 2");
        assert_eq!(*gap, 20, "Gap should be 20px");
    } else {
        panic!("Gallery should map to Grid node");
    }
    assert_eq!(
        gallery_node.children.len(),
        2,
        "Should have 2 image child nodes"
    );
    if let NodeKind::Image { src, alt, link, .. } = &gallery_node.children[0].kind {
        assert_eq!(src, "https://example.com/img1.jpg");
        assert_eq!(alt, "Image 1");
        assert_eq!(link.as_deref(), Some("https://example.com/img1.jpg"));
    } else {
        panic!("Child should be Image node");
    }

    // 2. Legacy Gutenberg Gallery with HTML images in inner_html
    let legacy_gallery = r#"
<!-- wp:gallery {"columns":3,"linkTo":"media"} -->
<ul class="wp-block-gallery columns-3 is-cropped">
  <li class="blocks-gallery-item"><figure><img src="https://example.com/photoA.jpg" alt="Photo A" width="600" height="400" /></figure></li>
  <li class="blocks-gallery-item"><figure><img src="https://example.com/photoB.jpg" alt="Photo B" width="600" height="400" /></figure></li>
  <li class="blocks-gallery-item"><figure><img src="https://example.com/photoC.jpg" alt="Photo C" width="600" height="400" /></figure></li>
</ul>
<!-- /wp:gallery -->
"#;

    let layout_legacy =
        transpile_gutenberg_content(legacy_gallery).expect("Transpile legacy gallery");
    let gallery_legacy_node = &layout_legacy.root.children[0];
    if let NodeKind::Grid { columns, .. } = &gallery_legacy_node.kind {
        assert_eq!(*columns, 3, "Columns should be 3");
    } else {
        panic!("Legacy gallery should map to Grid node");
    }
    assert_eq!(
        gallery_legacy_node.children.len(),
        3,
        "Extracted 3 images from legacy HTML"
    );
    if let NodeKind::Image { src, alt, link, .. } = &gallery_legacy_node.children[1].kind {
        assert_eq!(src, "https://example.com/photoB.jpg");
        assert_eq!(alt, "Photo B");
        assert_eq!(link.as_deref(), Some("https://example.com/photoB.jpg"));
    } else {
        panic!("Legacy child should be Image node");
    }
}

#[test]
fn test_core_embed_all_providers() {
    let input = r#"
<!-- wp:embed {"url":"https://www.youtube.com/watch?v=ScMzIvxBSi4","type":"video","providerNameSlug":"youtube","responsive":true} -->
<figure class="wp-block-embed is-type-video is-provider-youtube wp-block-embed-youtube wp-embed-aspect-16-9 wp-has-aspect-ratio"><div class="wp-block-embed__wrapper">
https://www.youtube.com/watch?v=ScMzIvxBSi4
</div></figure>
<!-- /wp:embed -->

<!-- wp:embed {"url":"https://vimeo.com/76979871","type":"video","providerNameSlug":"vimeo","responsive":true} -->
<figure class="wp-block-embed is-type-video is-provider-vimeo wp-block-embed-vimeo wp-embed-aspect-16-9 wp-has-aspect-ratio"><div class="wp-block-embed__wrapper">
https://vimeo.com/76979871
</div></figure>
<!-- /wp:embed -->

<!-- wp:embed {"url":"https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT","type":"rich","providerNameSlug":"spotify","responsive":true} -->
<figure class="wp-block-embed is-type-rich is-provider-spotify wp-block-embed-spotify"><div class="wp-block-embed__wrapper">
https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT
</div></figure>
<!-- /wp:embed -->

<!-- wp:embed {"url":"https://twitter.com/WordPress/status/123456789","type":"rich","providerNameSlug":"twitter","responsive":true} -->
<figure class="wp-block-embed is-type-rich is-provider-twitter wp-block-embed-twitter"><div class="wp-block-embed__wrapper">
https://twitter.com/WordPress/status/123456789
</div></figure>
<!-- /wp:embed -->
"#;

    let layout = transpile_gutenberg_content(input).expect("Transpile embeds");
    assert_eq!(layout.root.children.len(), 4, "Should have 4 embed nodes");

    // 1. YouTube
    if let NodeKind::Video {
        url, aspect_ratio, ..
    } = &layout.root.children[0].kind
    {
        assert!(
            url.contains("youtube.com/embed/ScMzIvxBSi4"),
            "YouTube embed URL normalized: {}",
            url
        );
        assert_eq!(aspect_ratio, "16/9");
    } else {
        panic!("YouTube should map to NodeKind::Video");
    }

    // 2. Vimeo
    if let NodeKind::Video {
        url, aspect_ratio, ..
    } = &layout.root.children[1].kind
    {
        assert!(
            url.contains("player.vimeo.com/video/76979871"),
            "Vimeo embed URL normalized: {}",
            url
        );
        assert_eq!(aspect_ratio, "16/9");
    } else {
        panic!("Vimeo should map to NodeKind::Video");
    }

    // 3. Spotify
    if let NodeKind::CustomHtml { raw } = &layout.root.children[2].kind {
        assert!(raw.contains("wp-block-embed-spotify"));
        assert!(
            raw.contains("open.spotify.com/embed/track/4cOdK2wGLETKBW3PvgPWqT"),
            "Spotify embed URL: {}",
            raw
        );
        assert!(raw.contains("border-radius:12px"));
    } else {
        panic!("Spotify should map to CustomHtml");
    }

    // 4. Twitter
    if let NodeKind::CustomHtml { raw } = &layout.root.children[3].kind {
        assert!(raw.contains("wp-block-embed-twitter"));
        assert!(raw.contains("platform.twitter.com/widgets.js"));
        assert!(raw.contains("https://twitter.com/WordPress/status/123456789"));
    } else {
        panic!("Twitter should map to CustomHtml");
    }
}

#[test]
fn test_core_list_multi_level_and_ordered_attributes() {
    let input = r#"
<!-- wp:list {"ordered":true,"start":5,"reversed":true} -->
<ol start="5" reversed>
  <!-- wp:list-item -->
  <li>Item 5 (Start)
    <!-- wp:list -->
    <ul>
      <!-- wp:list-item -->
      <li>Nested bullet A</li>
      <!-- /wp:list-item -->
      <!-- wp:list-item -->
      <li>Nested bullet B
        <!-- wp:list {"ordered":true} -->
        <ol>
          <!-- wp:list-item -->
          <li>Deep nested number 1</li>
          <!-- /wp:list-item -->
        </ol>
        <!-- /wp:list -->
      </li>
      <!-- /wp:list-item -->
    </ul>
    <!-- /wp:list -->
  </li>
  <!-- /wp:list-item -->
  <!-- wp:list-item -->
  <li>Item 4</li>
  <!-- /wp:list-item -->
</ol>
<!-- /wp:list -->
"#;

    let layout = transpile_gutenberg_content(input).expect("Transpile multi-level list");
    assert_eq!(layout.root.children.len(), 1);
    if let NodeKind::Text { content_html } = &layout.root.children[0].kind {
        assert!(
            content_html.starts_with("<ol start=\"5\" reversed>"),
            "HTML: {}",
            content_html
        );
        assert!(content_html.contains("Item 5 (Start)"));
        assert!(content_html.contains("<ul>"));
        assert!(content_html.contains("Nested bullet A"));
        assert!(content_html.contains("Nested bullet B"));
        assert!(content_html.contains("<ol>"));
        assert!(content_html.contains("Deep nested number 1"));
        assert!(content_html.ends_with("</ol>"));
    } else {
        panic!("List should map to Text with content_html");
    }
}

#[test]
fn test_core_quote_with_citation() {
    let input = r#"
<!-- wp:quote {"citation":"Ada Lovelace","className":"is-style-large"} -->
<blockquote class="wp-block-quote is-style-large">
<p>The Analytical Engine weaves algebraical patterns just as the Jacquard-loom weaves flowers and leaves.</p>
<cite>Ada Lovelace</cite>
</blockquote>
<!-- /wp:quote -->
"#;

    let layout = transpile_gutenberg_content(input).expect("Transpile quote");
    assert_eq!(layout.root.children.len(), 1);

    let html = render_layout_to_html(&layout);
    assert!(html.contains("<blockquote"), "Must contain blockquote");
    assert!(html.contains("The Analytical Engine weaves"), "Quote body");
    assert!(html.contains("<cite>Ada Lovelace</cite>"), "Citation");
    assert!(!html.contains("<!-- wp:"));
}

#[test]
fn test_theme_json_full_compilation_and_presets() {
    let raw_theme_json = r###"{
      "version": 3,
      "$schema": "https://schemas.wp.org/trunk/theme.json",
      "settings": {
        "color": {
          "palette": [
            { "name": "Primary", "slug": "primary", "color": "#1d4ed8" },
            { "name": "Accent", "slug": "accent", "color": "#f97316" }
          ],
          "gradients": [
            { "name": "Warm Sunset", "slug": "warm-sunset", "gradient": "linear-gradient(135deg, #f97316 0%, #dc2626 100%)" }
          ]
        },
        "typography": {
          "fontSizes": [
            { "name": "Small", "slug": "small", "size": "14px" },
            { "name": "Hero", "slug": "hero", "size": "48px" }
          ],
          "fontFamilies": [
            { "name": "Inter", "slug": "inter", "fontFamily": "\"Inter\", sans-serif" }
          ]
        },
        "spacing": {
          "spacingSizes": [
            { "name": "Small", "slug": "30", "size": "1rem" },
            { "name": "Large", "slug": "70", "size": "3rem" }
          ]
        },
        "shadow": {
          "presets": [
            { "name": "Card Shadow", "slug": "card", "shadow": "0 4px 6px -1px rgb(0 0 0 / 0.1)" }
          ]
        },
        "layout": {
          "contentSize": "800px",
          "wideSize": "1200px"
        }
      },
      "styles": {
        "color": {
          "background": "#fafafa",
          "text": "#171717"
        },
        "typography": {
          "fontSize": "16px",
          "fontFamily": "\"Inter\", sans-serif"
        },
        "css": "h1 { letter-spacing: -0.02em; }"
      }
    }"###;

    let theme = parse_theme_json(raw_theme_json).expect("Theme.json should parse");
    assert_eq!(theme.version, 3);

    let css = compile_theme_json_to_css(raw_theme_json).expect("Compile to CSS");

    // 1. Root variables
    assert!(
        css.contains("--wp--preset--color--primary: #1d4ed8;"),
        "Primary color var"
    );
    assert!(
        css.contains("--wp--preset--color--accent: #f97316;"),
        "Accent color var"
    );
    assert!(css.contains(
        "--wp--preset--gradient--warm-sunset: linear-gradient(135deg, #f97316 0%, #dc2626 100%);"
    ));
    assert!(css.contains("--wp--preset--font-size--small: 14px;"));
    assert!(css.contains("--wp--preset--font-size--hero: 48px;"));
    assert!(css.contains("--wp--preset--font-family--inter: \"Inter\", sans-serif;"));
    assert!(css.contains("--wp--preset--spacing--30: 1rem;"));
    assert!(css.contains("--wp--preset--spacing--70: 3rem;"));
    assert!(css.contains("--wp--preset--shadow--card: 0 4px 6px -1px rgb(0 0 0 / 0.1);"));
    assert!(css.contains("--wp--style--global--content-size: 800px;"));
    assert!(css.contains("--wp--style--global--wide-size: 1200px;"));

    // 2. Utility classes
    assert!(
        css.contains(".has-primary-color { color: var(--wp--preset--color--primary) !important; }")
    );
    assert!(css.contains(".has-primary-background-color { background-color: var(--wp--preset--color--primary) !important; }"));
    assert!(css.contains(
        ".has-primary-border-color { border-color: var(--wp--preset--color--primary) !important; }"
    ));
    assert!(css.contains(".has-warm-sunset-gradient-background { background: var(--wp--preset--gradient--warm-sunset) !important; }"));
    assert!(css.contains(
        ".has-hero-font-size { font-size: var(--wp--preset--font-size--hero) !important; }"
    ));
    assert!(css.contains(
        ".has-inter-font-family { font-family: var(--wp--preset--font-family--inter) !important; }"
    ));

    // 3. Layout constraints
    assert!(css.contains(".is-layout-constrained"));
    assert!(css.contains("max-width: var(--wp--style--global--content-size, 800px);"));
    assert!(css.contains("max-width: var(--wp--style--global--wide-size, 1200px);"));

    // 4. Styles body & custom css
    assert!(css.contains("body {"));
    assert!(css.contains("background-color: #fafafa;"));
    assert!(css.contains("color: #171717;"));
    assert!(css.contains("font-size: 16px;"));
    assert!(css.contains("font-family: \"Inter\", sans-serif;"));
    assert!(css.contains("h1 { letter-spacing: -0.02em; }"));
}

#[test]
fn test_transpile_content_dispatcher_handles_gutenberg_with_empty_meta() {
    let wp_content = r#"
<!-- wp:heading -->
<h2>Automatic Dispatcher Heading</h2>
<!-- /wp:heading -->
<!-- wp:paragraph -->
<p>Clean dispatch test</p>
<!-- /wp:paragraph -->
"#;
    let meta = HashMap::new();
    let res = transpile_content(wp_content, &meta);
    assert!(
        res.is_some(),
        "transpile_content should automatically detect <!-- wp: comments"
    );
    let layout = res.unwrap();
    assert_eq!(layout.root.children.len(), 2);
    let html = render_layout_to_html(&layout);
    assert!(html.contains("<h2") && html.contains("Automatic Dispatcher Heading"));
    assert!(html.contains("Clean dispatch test"));
    assert!(!html.contains("<!-- wp:"));
}

#[test]
fn test_multibyte_utf8_emoji_and_non_latin_in_block_attributes() {
    let input = r#"
<!-- wp:paragraph {"placeholder":"🚀 宇宙 Пространство {foo} 🌍", "customMeta":{"arabic":"مرحبا {user}"}} -->
<p>Multilingual paragraph: 🚀 宇宙 Пространство مرحبا</p>
<!-- /wp:paragraph -->
"#;

    let layout = transpile_gutenberg_content(input).expect("Should parse multi-byte UTF-8");
    assert_eq!(layout.root.children.len(), 1);
    let html = render_layout_to_html(&layout);
    assert!(html.contains("🚀 宇宙 Пространство مرحبا"));
    assert!(!html.contains("<!-- wp:"));
}

#[test]
fn test_escaped_quotes_and_braces_in_json_attributes() {
    let input = r#"
<!-- wp:paragraph {"customText":"He said, \"Hello, {world}!\" and \\escaped\\."} -->
<p>He said, "Hello, {world}!" and \escaped\.</p>
<!-- /wp:paragraph -->
"#;

    let layout =
        transpile_gutenberg_content(input).expect("Should parse escaped quotes in attributes");
    assert_eq!(layout.root.children.len(), 1);
    let html = render_layout_to_html(&layout);
    assert!(
        html.contains("He said, &quot;Hello, {world}!&quot;")
            || html.contains("He said, \"Hello, {world}!\"")
    );
    assert!(!html.contains("<!-- wp:"));
}

#[test]
fn test_unclosed_and_mismatched_block_recovery_without_panics() {
    // Unclosed paragraph inside columns, closed by /wp:columns
    let input = r#"
<!-- wp:columns -->
<div class="wp-block-columns">
  <!-- wp:column -->
  <div class="wp-block-column">
    <!-- wp:paragraph -->
    <p>Unclosed paragraph inside column</p>
    <!-- missing /wp:paragraph -->
  </div>
  <!-- /wp:column -->
</div>
<!-- /wp:columns -->
<!-- /wp:orphan_close -->
"#;

    let layout =
        transpile_gutenberg_content(input).expect("Should recover from unclosed tags cleanly");
    assert_eq!(layout.root.children.len(), 1);
    let col = &layout.root.children[0].children[0];
    assert_eq!(col.children.len(), 1);
    let html = render_layout_to_html(&layout);
    assert!(html.contains("Unclosed paragraph inside column"));
    assert!(!html.contains("<!-- wp:"));
}

#[test]
fn test_void_self_closing_blocks_with_attributes() {
    let input = r#"
<!-- wp:separator {"className":"is-style-dots"} /-->
<!-- wp:spacer {"height":"75px"} /-->
<!-- wp:separator /-->
"#;

    let layout = transpile_gutenberg_content(input).expect("Should parse self-closing void blocks");
    assert_eq!(layout.root.children.len(), 3, "Expected 3 void blocks");

    // Block 1: separator
    assert!(matches!(
        layout.root.children[0].kind,
        NodeKind::Divider { .. }
    ));

    // Block 2: spacer
    if let NodeKind::Spacer { height } = &layout.root.children[1].kind {
        assert_eq!(*height, 75);
    } else {
        panic!("Block 2 should be Spacer with height 75");
    }

    // Block 3: plain separator
    assert!(matches!(
        layout.root.children[2].kind,
        NodeKind::Divider { .. }
    ));

    let html = render_layout_to_html(&layout);
    assert!(!html.contains("<!-- wp:"));
    assert!(!html.contains("<!-- /wp:"));
}

#[test]
fn test_gutenberg_style_and_color_preset_extraction() {
    let input = r#"
<!-- wp:paragraph {"backgroundColor":"primary","textColor":"accent","fontSize":"large","style":{"spacing":{"padding":{"top":"16px","bottom":"16px"}}}} -->
<p class="has-primary-background-color has-accent-color has-large-font-size">Styled paragraph with presets</p>
<!-- /wp:paragraph -->
"#;

    let layout = transpile_gutenberg_content(input).expect("Should parse style presets");
    assert_eq!(layout.root.children.len(), 1);
    let node = &layout.root.children[0];

    // Verify background color mapped to preset var
    assert_eq!(
        node.styles
            .background
            .as_ref()
            .and_then(|b| b.color.as_deref()),
        Some("var(--wp--preset--color--primary)")
    );
    // Verify text color mapped to preset var
    assert_eq!(
        node.styles
            .typography
            .as_ref()
            .and_then(|t| t.color.as_deref()),
        Some("var(--wp--preset--color--accent)")
    );
    // Verify padding
    let pad = node.styles.padding.as_ref().unwrap();
    assert_eq!(pad.top.as_deref(), Some("16px"));
    assert_eq!(pad.bottom.as_deref(), Some("16px"));

    let html = render_layout_to_html(&layout);
    assert!(!html.contains("<!-- wp:"));
    assert!(html.contains("Styled paragraph with presets"));
}
