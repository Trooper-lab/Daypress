use neopress_ast::render::{render_layout_to_html, sanitize_ssr_html};
use neopress_ast::schema::{
    NeopressLayout, NeopressNode, NodeKind, PriceFeatureItem, StyleProperties,
};
use neopress_ast::{transpile_content, transpile_divi_content};
use std::collections::HashMap;

// =========================================================================
// 1. Divi 5 Gutenberg Block Comments: Complex JSON & Inner Blocks
// =========================================================================

#[test]
fn test_divi5_multiline_json_and_nested_inner_blocks() {
    // Multiline JSON in block comments (standard formatted Gutenberg / Divi 5)
    let divi5_multiline = r#"
<!-- wp:divi/section {
  "module": {
    "props": {
      "attrs": {
        "module_id": "multiline_sec",
        "custom_padding": "50px|30px|50px|30px"
      }
    }
  }
} -->
<!-- wp:divi/row -->
<!-- wp:divi/column {"type":"1_3"} -->
<!-- wp:divi/text --><h3>Col 1</h3><!-- /wp:divi/text -->
<!-- /wp:divi/column -->
<!-- wp:divi/column {"type":"2_3"} -->
<!-- wp:divi/text -->
<p>Col 2 content with inner text</p>
<!-- /wp:divi/text -->
<!-- wp:divi/button {"button_text":"Click Me","button_url":"https://example.com"} /-->
<!-- /wp:divi/column -->
<!-- /wp:divi/row -->
<!-- /wp:divi/section -->
"#;

    // Full transpile_divi_content verification
    let layout_res = transpile_divi_content(divi5_multiline);
    assert!(
        layout_res.is_ok(),
        "transpile_divi_content failed on multiline Divi 5 JSON: {:?}",
        layout_res.err()
    );
    let layout = layout_res.unwrap();
    assert_eq!(layout.root.children.len(), 1, "Expected 1 root section");
    let sec_node = &layout.root.children[0];
    assert_eq!(sec_node.id, "multiline_sec", "Should extract module_id from multiline JSON");
    assert_eq!(
        sec_node.styles.padding.as_ref().and_then(|p| p.top.as_deref()),
        Some("50px")
    );
    assert_eq!(sec_node.children.len(), 1, "Section should have 1 child (Row)");
    let row_node = &sec_node.children[0];
    assert_eq!(row_node.children.len(), 2, "Row should have 2 children (Columns)");
    let col2 = &row_node.children[1];
    assert_eq!(col2.children.len(), 2, "Column 2 should have 2 children (Text + Button)");
}

#[test]
fn test_transpile_content_divi5_routing() {
    let divi5_content = r#"
<!-- wp:divi/section {"module":{"props":{"attrs":{"module_id":"routed_divi5"}}}} -->
<!-- wp:divi/row -->
<!-- wp:divi/column {"type":"1_1"} -->
<!-- wp:divi/text --><h2>Divi 5 Heading</h2><!-- /wp:divi/text -->
<!-- /wp:divi/column -->
<!-- /wp:divi/row -->
<!-- /wp:divi/section -->
"#;

    let empty_meta = HashMap::new();
    let layout_opt = transpile_content(divi5_content, &empty_meta);
    assert!(layout_opt.is_some(), "transpile_content returned None for Divi 5 content");
    let layout = layout_opt.unwrap();
    let sec = &layout.root.children[0];
    assert_eq!(
        sec.id, "routed_divi5",
        "transpile_content must route Divi 5 content to Divi transpiler, preserving module_id 'routed_divi5'"
    );
}

// =========================================================================
// 2. Divi 4 Quad Margins and Paddings with Varied Units and Edge Cases
// =========================================================================

#[test]
fn test_divi4_quad_margins_and_paddings_varied_units() {
    let divi_content = r#"
[et_pb_section custom_padding="50px|30px|50px|30px|false|false" custom_margin="-20px|auto|40px|auto"]
  [et_pb_row custom_padding="2.5rem|5vw|10%|auto"]
    [et_pb_column type="4_4"]
      [et_pb_text custom_margin="15px||30px|"]
        <p>Testing partial quad margins</p>
      [/et_pb_text]
      [et_pb_text custom_padding="|||"]
        <p>Testing empty quad padding</p>
      [/et_pb_text]
    [/et_pb_column]
  [/et_pb_row]
[/et_pb_section]
"#;

    let layout = transpile_divi_content(divi_content).expect("Divi 4 transpilation failed");
    let sec = &layout.root.children[0];

    // Section padding with flags at end
    let sec_pad = sec.styles.padding.as_ref().expect("Section padding missing");
    assert_eq!(sec_pad.top.as_deref(), Some("50px"));
    assert_eq!(sec_pad.right.as_deref(), Some("30px"));
    assert_eq!(sec_pad.bottom.as_deref(), Some("50px"));
    assert_eq!(sec_pad.left.as_deref(), Some("30px"));

    // Section margin with negative value and auto
    let sec_mar = sec.styles.margin.as_ref().expect("Section margin missing");
    assert_eq!(sec_mar.top.as_deref(), Some("-20px"));
    assert_eq!(sec_mar.right.as_deref(), Some("auto"));
    assert_eq!(sec_mar.bottom.as_deref(), Some("40px"));
    assert_eq!(sec_mar.left.as_deref(), Some("auto"));

    // Row padding with varied units (rem, vw, %, auto)
    let row = &sec.children[0];
    let row_pad = row.styles.padding.as_ref().expect("Row padding missing");
    assert_eq!(row_pad.top.as_deref(), Some("2.5rem"));
    assert_eq!(row_pad.right.as_deref(), Some("5vw"));
    assert_eq!(row_pad.bottom.as_deref(), Some("10%"));
    assert_eq!(row_pad.left.as_deref(), Some("auto"));

    // Column child 0: Partial margin (top and bottom only)
    let col = &row.children[0];
    let text1 = &col.children[0];
    let t1_mar = text1.styles.margin.as_ref().expect("Text 1 margin missing");
    assert_eq!(t1_mar.top.as_deref(), Some("15px"));
    assert_eq!(t1_mar.right, None);
    assert_eq!(t1_mar.bottom.as_deref(), Some("30px"));
    assert_eq!(t1_mar.left, None);

    // Verify rendered HTML CSS for partial margin
    let html = render_layout_to_html(&layout);
    assert!(html.contains("margin-top: 15px;"));
    assert!(html.contains("margin-bottom: 30px;"));

    // Column child 1: Empty quad "|||" should be None
    let text2 = &col.children[1];
    assert!(text2.styles.padding.is_none(), "Empty quad '|||' should parse as None");
}

// =========================================================================
// 3. Elementor CSS Grid Containers with Varied Track Templates & Spans
// =========================================================================

#[test]
fn test_elementor_css_grid_track_templates_and_spans() {
    let grid_json = r#"[
        {
            "id": "grid_parent",
            "elType": "container",
            "settings": {
                "container_type": "grid",
                "grid_columns_grid_custom": "repeat(auto-fill, minmax(280px, 1fr))",
                "grid_rows_grid_custom": "auto 1fr auto",
                "grid_gaps": { "row": "24", "column": "16", "unit": "px" },
                "grid_auto_flow": "dense",
                "grid_columns_grid_tablet": { "size": 2, "unit": "fr" },
                "grid_columns_grid_mobile": "1fr"
            },
            "elements": [
                {
                    "id": "child_u64_span",
                    "elType": "widget",
                    "widgetType": "heading",
                    "settings": {
                        "title": "Item 1",
                        "_grid_column_span": 2
                    }
                },
                {
                    "id": "child_custom_col",
                    "elType": "widget",
                    "widgetType": "button",
                    "settings": {
                        "text": "Item 2",
                        "_grid_column": "1 / span 3",
                        "_grid_row": "2 / 4"
                    }
                }
            ]
        }
    ]"#;

    let mut meta = HashMap::new();
    meta.insert("_elementor_data".to_string(), grid_json.to_string());
    let layout = transpile_content("", &meta).expect("Grid container transpilation failed");
    let grid_node = &layout.root.children[0];

    assert_eq!(grid_node.styles.display.as_deref(), Some("grid"));
    assert_eq!(
        grid_node.styles.grid_template_columns.as_deref(),
        Some("repeat(auto-fill, minmax(280px, 1fr))")
    );
    assert_eq!(
        grid_node.styles.grid_template_rows.as_deref(),
        Some("auto 1fr auto")
    );
    assert_eq!(grid_node.styles.grid_gap.as_deref(), Some("24px 16px"));
    assert_eq!(grid_node.styles.grid_auto_flow.as_deref(), Some("dense"));

    // Responsive overrides
    let tab = grid_node.styles.tablet.as_ref().expect("Missing tablet styles");
    assert_eq!(tab.grid_template_columns.as_deref(), Some("repeat(2, 1fr)"));
    let mob = grid_node.styles.mobile.as_ref().expect("Missing mobile styles");
    assert_eq!(mob.grid_template_columns.as_deref(), Some("1fr"));

    // Children placement
    assert_eq!(grid_node.children[0].styles.grid_column.as_deref(), Some("span 2"));
    assert_eq!(grid_node.children[1].styles.grid_column.as_deref(), Some("1 / span 3"));
    assert_eq!(grid_node.children[1].styles.grid_row.as_deref(), Some("2 / 4"));
}

// =========================================================================
// 4. SSR HTML Rendering & Strict Editor Attribute/Class Sanitization
// =========================================================================

#[test]
fn test_ssr_html_sanitizer_edge_cases() {
    let test_cases = vec![
        // Standard contenteditable variations
        "<h1 contenteditable=\"true\">Heading</h1>",
        "<p contenteditable='true'>Paragraph</p>",
        "<div contenteditable=\"false\">Non-editable</div>",
        "<span contenteditable>Bare attribute</span>",
        // Case variations (HTML5 case-insensitivity)
        "<h2 contentEditable=\"true\">Mixed Case</h2>",
        "<div CONTENTEDITABLE=\"true\">Upper Case</div>",
        // Draggable
        "<div draggable=\"true\">Drag me</div>",
        "<div draggable='false'>Don't drag</div>",
        // Editor classes
        "<div class=\"elementor-grid-handle active-element ui-resizable\">Handle</div>",
        "<section class=\"elementor-element-edit-mode elementor-editor-active ui-draggable\">Sec</section>",
        // Divi & NeoStudio editor classes
        "<div class=\"et_pb_element_editing visual-builder-active neostudio-handle\">Edit Mode</div>",
        // Editor data attributes
        "<div data-elementor-type=\"section\" data-drag-handle=\"true\" data-model-cid=\"c123\">Content</div>",
    ];

    for raw_input in test_cases {
        let sanitized = sanitize_ssr_html(raw_input);
        assert!(
            !sanitized.to_ascii_lowercase().contains("contenteditable"),
            "Sanitizer leaked 'contenteditable' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("draggable"),
            "Sanitizer leaked 'draggable' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("elementor-grid-handle"),
            "Sanitizer leaked 'elementor-grid-handle' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("active-element"),
            "Sanitizer leaked 'active-element' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("et_pb_element_editing"),
            "Sanitizer leaked 'et_pb_element_editing' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("visual-builder-active"),
            "Sanitizer leaked 'visual-builder-active' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("neostudio-handle"),
            "Sanitizer leaked 'neostudio-handle' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("data-elementor-type"),
            "Sanitizer leaked 'data-elementor-type' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("data-drag-handle"),
            "Sanitizer leaked 'data-drag-handle' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
        assert!(
            !sanitized.contains("data-model-cid"),
            "Sanitizer leaked 'data-model-cid' from input: {}\nResult: {}",
            raw_input,
            sanitized
        );
    }
}

#[test]
fn test_render_layout_to_html_zero_editor_leaks() {
    let node = NeopressNode {
        id: "hero_sec".to_string(),
        kind: NodeKind::CustomHtml {
            raw: "<div class=\"elementor-grid-handle active-element\" contenteditable=\"true\" data-drag-handle=\"true\"><h1 contentEditable=\"true\">Title</h1></div>".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    let public_html = render_layout_to_html(&layout);
    assert!(!public_html.to_ascii_lowercase().contains("contenteditable"));
    assert!(!public_html.contains("elementor-grid-handle"));
    assert!(!public_html.contains("active-element"));
    assert!(!public_html.contains("data-drag-handle"));
    assert!(public_html.contains("Title</h1>"));
}

// =========================================================================
// 5. Challenger Empirical Edge Cases: HTML Escaping & Advanced Leak Matrix
// =========================================================================

#[test]
fn test_challenger_html_attribute_escaping_and_advanced_leak_firewall() {
    // 1. Edge Case: HTML Attribute Escaping in Pro Widgets
    let escaping_node = NeopressNode {
        id: "escape_container".to_string(),
        kind: NodeKind::Container,
        styles: StyleProperties::default(),
        children: vec![
            NeopressNode {
                id: "escape_price_card".to_string(),
                kind: NodeKind::PriceCard {
                    title: "Pro & Enterprise <Tier>".to_string(),
                    subtitle: Some("Security \"Hardened\" Edition".to_string()),
                    currency: Some("&dollar;".to_string()),
                    price: "199\" onfocus=\"alert('xss')".to_string(),
                    original_price: Some("299".to_string()),
                    period: Some("month".to_string()),
                    badge: Some("Special \"Offer\"".to_string()),
                    featured: true,
                    features: vec![
                        PriceFeatureItem {
                            text: "Feature with <b>HTML</b> & \"quotes\"".to_string(),
                            included: true,
                            icon: Some("fa-solid fa-shield\" onerror=\"alert(1)".to_string()),
                        },
                    ],
                    button_text: "Deploy > Now".to_string(),
                    button_url: "/deploy?plan=pro&ref=hero\" onmouseover=\"evil()".to_string(),
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            },
            NeopressNode {
                id: "escape_img".to_string(),
                kind: NodeKind::Image {
                    src: "https://example.com/logo.png\" onerror=\"alert('img')".to_string(),
                    alt: "Company \"Logo\" & Brand <Icon>".to_string(),
                    link: Some("/home?ref=img&track=\"yes\"".to_string()),
                    width: None,
                    height: None,
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            },
            NeopressNode {
                id: "escape_search".to_string(),
                kind: NodeKind::SearchForm {
                    placeholder: "Search \"articles\" & docs...".to_string(),
                    button_text: "Search >".to_string(),
                    endpoint: "/api/search?lang=en&type=\"all\"".to_string(),
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            },
            NeopressNode {
                id: "escape_login".to_string(),
                kind: NodeKind::LoginForm {
                    label_username: "User \"Name\" & ID".to_string(),
                    label_password: "Password <Secret>".to_string(),
                    redirect_url: "/admin?user=\"root\"&status=active".to_string(),
                    remember_me: true,
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            },
        ],
    };

    let layout = NeopressLayout {
        version: 1,
        root: escaping_node,
        global_styles: None,
    };

    let rendered = render_layout_to_html(&layout);

    // Verify attribute quotes are properly escaped so attribute injection is prevented
    assert!(!rendered.contains("href=\"/deploy?plan=pro&ref=hero\" onmouseover="), "Attribute breakout occurred in button_url");
    assert!(rendered.contains("href=\"/deploy?plan=pro&amp;ref=hero&quot; onmouseover=&quot;evil()\""), "Expected escaped button_url");
    assert!(!rendered.contains("src=\"https://example.com/logo.png\" onerror="), "Attribute breakout occurred in img src");
    assert!(rendered.contains("src=\"https://example.com/logo.png&quot; onerror=&quot;alert('img')\""), "Expected escaped img src");
    assert!(rendered.contains("alt=\"Company &quot;Logo&quot; &amp; Brand &lt;Icon&gt;\""), "Expected escaped img alt");
    assert!(rendered.contains("placeholder=\"Search &quot;articles&quot; &amp; docs...\""), "Expected escaped search placeholder");
    assert!(rendered.contains("action=\"/api/search?lang=en&amp;type=&quot;all&quot;\""), "Expected escaped search endpoint");
    assert!(rendered.contains("value=\"/admin?user=&quot;root&quot;&amp;status=active\""), "Expected escaped login redirect");

    // 2. Advanced SSR Leak Sanitization Matrix
    let raw_html_with_leaks = r#"<div class="card elementor-grid-handle neostudio-handle visual-builder-active active-element et_pb_element_editing ui-resizable ui-draggable ui-sortable neostudio-canvas"
        cOnTeNtEdItAbLe="true"
        CONTENTEDITABLE='false'
        contenteditable
        draggable="true"
        DRAGGABLE='false'
        data-elementor-type="section"
        data-elementor-id="12345"
        data-neostudio-block="price_card"
        data-drag-handle="true"
        data-model-cid="c42"
        data-valid-custom="keep-me"
        data-track-id="987">
        <span class="elementor-element-edit-mode neostudio-resizer">
            <h2 contentEditable="true">Clean Header</h2>
        </span>
        <div class="elementor-grid-handle ui-resizable">
            <p>Only editor classes inside</p>
        </div>
    </div>"#;

    let sanitized = sanitize_ssr_html(raw_html_with_leaks);

    // Verify zero contenteditable leaks (case-insensitive, quoted or unquoted)
    assert!(!sanitized.to_ascii_lowercase().contains("contenteditable"), "Leaked contenteditable in sanitized output");
    // Verify zero draggable leaks
    assert!(!sanitized.to_ascii_lowercase().contains("draggable"), "Leaked draggable in sanitized output");
    // Verify editor data attributes stripped
    assert!(!sanitized.contains("data-elementor-type"), "Leaked data-elementor-type");
    assert!(!sanitized.contains("data-elementor-id"), "Leaked data-elementor-id");
    assert!(!sanitized.contains("data-neostudio-block"), "Leaked data-neostudio-block");
    assert!(!sanitized.contains("data-drag-handle"), "Leaked data-drag-handle");
    assert!(!sanitized.contains("data-model-cid"), "Leaked data-model-cid");
    // Verify valid data attributes preserved
    assert!(sanitized.contains("data-valid-custom=\"keep-me\""), "Wrongly stripped valid data attribute");
    assert!(sanitized.contains("data-track-id=\"987\""), "Wrongly stripped valid data attribute");

    // Verify editor classes stripped and legitimate classes preserved
    assert!(sanitized.contains("class=\"card\""), "Legitimate class 'card' was not preserved cleanly; got: {}", sanitized);
    assert!(!sanitized.contains("elementor-grid-handle"), "Leaked elementor-grid-handle");
    assert!(!sanitized.contains("neostudio-handle"), "Leaked neostudio-handle");
    assert!(!sanitized.contains("neostudio-canvas"), "Leaked neostudio-canvas");
    assert!(!sanitized.contains("visual-builder-active"), "Leaked visual-builder-active");
    assert!(!sanitized.contains("active-element"), "Leaked active-element");
    assert!(!sanitized.contains("et_pb_element_editing"), "Leaked et_pb_element_editing");
    assert!(!sanitized.contains("ui-resizable"), "Leaked ui-resizable");
    assert!(!sanitized.contains("ui-draggable"), "Leaked ui-draggable");
    assert!(!sanitized.contains("elementor-element-edit-mode"), "Leaked elementor-element-edit-mode");
    assert!(!sanitized.contains("neostudio-resizer"), "Leaked neostudio-resizer");

    // Verify empty class attribute cleanup (div with only editor classes should have NO class attribute)
    assert!(!sanitized.contains("class=\"\""), "Found empty class=\"\" artifact in output: {}", sanitized);
    assert!(!sanitized.contains("class=\"   \""), "Found whitespace class artifact in output: {}", sanitized);
}
