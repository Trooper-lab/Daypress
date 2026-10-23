use neopress_ast::render::{compile_layout_css, render_layout_to_html, render_node_to_html};
use neopress_ast::schema::{
    NeopressLayout, NeopressNode, NodeKind, Spacing, StyleProperties,
};
use neopress_ast::transpile_content;
use std::collections::HashMap;

// =========================================================================
// 1. Pro Blocks Boundary & Stress Tests (Edge values, NaN, extreme lengths)
// =========================================================================

#[test]
fn test_pro_blocks_boundary_and_extreme_values() {
    // 1. StarRating with edge ratings (0.0, 5.0, negative float, NaN-like float, 0 max_rating)
    let edge_ratings = vec![
        (0.0, 5),
        (5.0, 5),
        (-2.5, 5),
        (10.5, 10),
        (3.0, 0), // max_rating = 0: must not divide by zero or panic
    ];

    for (rating, max_rating) in edge_ratings {
        let node = NeopressNode {
            id: "star_edge".to_string(),
            kind: NodeKind::StarRating {
                rating,
                max_rating,
                title: Some("Edge Rating".to_string()),
                star_color: Some("#fbbf24".to_string()),
                star_size: Some(18),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        };

        let html = render_node_to_html(&node);
        assert!(!html.is_empty());
        assert!(html.contains("np-star-rating"));
    }

    // 2. ProgressBar with 0%, 100%, 255% (u8 boundary)
    for pct in [0, 50, 100, 255] {
        let pb_node = NeopressNode {
            id: format!("pb_{}", pct),
            kind: NodeKind::ProgressBar {
                title: format!("Progress {}%", pct),
                percent: pct,
                display_text: Some(format!("{} percent", pct)),
                track_color: Some("#334155".to_string()),
                bar_color: Some("#38bdf8".to_string()),
                height: Some(12),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        };
        let html = render_node_to_html(&pb_node);
        assert!(html.contains(&format!("width: {}%;", pct)));
    }

    // 3. AnimatedHeadline with empty words vs 50 words
    let empty_words_node = NeopressNode {
        id: "empty_anim".to_string(),
        kind: NodeKind::AnimatedHeadline {
            before_text: "Empty: ".to_string(),
            words: Vec::new(),
            after_text: " end.".to_string(),
            animation_style: "rotate".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };
    let html_empty = render_node_to_html(&empty_words_node);
    assert!(html_empty.contains("np-animated-headline"));

    let many_words: Vec<String> = (0..50).map(|i| format!("Speed-{}", i)).collect();
    let many_words_node = NeopressNode {
        id: "many_anim".to_string(),
        kind: NodeKind::AnimatedHeadline {
            before_text: "Neopress is ".to_string(),
            words: many_words,
            after_text: ".".to_string(),
            animation_style: "typing".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };
    let html_many = render_node_to_html(&many_words_node);
    assert!(html_many.contains("Speed-49"));

    // 4. FlipBox all 6 directions
    for dir in ["flip_left", "flip_right", "flip_up", "flip_down", "push", "slide"] {
        let fb_node = NeopressNode {
            id: format!("fb_{}", dir),
            kind: NodeKind::FlipBox {
                front_icon: None,
                front_image: None,
                front_title: "Title".to_string(),
                front_description: "Desc".to_string(),
                back_title: "Back".to_string(),
                back_description: "Back Desc".to_string(),
                button_text: None,
                button_url: None,
                direction: dir.to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        };
        let html_fb = render_node_to_html(&fb_node);
        assert!(html_fb.contains(&format!("np-flipbox-{}", dir)));
    }

    // 5. TableOfContents with inverted depths
    let toc_node = NeopressNode {
        id: "toc_inv".to_string(),
        kind: NodeKind::TableOfContents {
            title: "Quick Index".to_string(),
            min_depth: 5,
            max_depth: 2,
            collapsible: false,
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };
    let html_toc = render_node_to_html(&toc_node);
    assert!(html_toc.contains("np-toc"));
    assert!(html_toc.contains("data-min-depth=\"5\""));
    assert!(html_toc.contains("data-max-depth=\"2\""));
}

// =========================================================================
// 2. Flat Visual Builder Format Interop for Extended Blocks
// =========================================================================

#[test]
fn test_flat_visual_builder_extended_blocks() {
    // 1. Notice alias for Alert
    let flat_alert = r#"{
        "id": "alert_flat_1",
        "type": "Notice",
        "alert_type": "danger",
        "title": "System Alert",
        "description": "High memory consumption detected",
        "dismissible": false
    }"#;
    let node_al: NeopressNode = serde_json::from_str(flat_alert).expect("Notice alias failed");
    match &node_al.kind {
        NodeKind::Alert { alert_type, title, dismissible, .. } => {
            assert_eq!(alert_type, "danger");
            assert_eq!(title, "System Alert");
            assert!(!dismissible);
        }
        _ => panic!("Expected Alert variant for Notice alias"),
    }

    // 2. PostGrid alias for LoopGrid
    let flat_grid = r#"{
        "id": "grid_flat_1",
        "type": "PostGrid",
        "post_type": "portfolio",
        "posts_per_page": 12,
        "columns": 4,
        "taxonomy": "genre:sci-fi"
    }"#;
    let node_gr: NeopressNode = serde_json::from_str(flat_grid).expect("PostGrid alias failed");
    match &node_gr.kind {
        NodeKind::LoopGrid { post_type, posts_per_page, columns, taxonomy, .. } => {
            assert_eq!(post_type, "portfolio");
            assert_eq!(*posts_per_page, 12);
            assert_eq!(*columns, 4);
            assert_eq!(taxonomy.as_deref(), Some("genre:sci-fi"));
        }
        _ => panic!("Expected LoopGrid variant for PostGrid alias"),
    }

    // 3. Icon with "icon" alias (using raw literal with double hashes)
    let flat_icon = r##"{
        "id": "icon_flat_1",
        "type": "Icon",
        "icon": "fa-brands fa-rust",
        "size": 32,
        "color": "#e11d48"
    }"##;
    let node_ic: NeopressNode = serde_json::from_str(flat_icon).expect("Icon alias failed");
    match &node_ic.kind {
        NodeKind::Icon { name, size, color } => {
            assert_eq!(name, "fa-brands fa-rust");
            assert_eq!(*size, Some(32));
            assert_eq!(color.as_deref(), Some("#e11d48"));
        }
        _ => panic!("Expected Icon variant"),
    }
}

// =========================================================================
// 3. Deep Unicode & Multi-Script Adversarial Stress
// =========================================================================

#[test]
fn test_deep_unicode_and_multi_script_resilience() {
    let multi_script_samples = vec![
        ("arabic", "مرحباً بكم في نيوبيرس! أداء فائق وتوافق 100% مع قوالب ووردبريس."),
        ("cjk", "高性能なRust製CMS『Neopress』へようこそ。WordPressサイトの移行がワンクリックで完了します。"),
        ("hindi", "नियोप्रेस में आपका स्वागत है - अत्यधिक तीव्र और सुरक्षित वर्डप्रेस प्रतिस्थापन।"),
        ("thai", "ยินดีต้อนรับสู่ Neopress ระบบจัดการเนื้อหาความเร็วสูง ⚡"),
        ("combining_chars", "Z̸a̸l̸g̸o̸ ̸t̸e̸x̸t̸ ̸c̸h̸a̸o̸s̸ ̸t̸e̸s̸t̸ ̸t̸o̸ ̸v̸e̸r̸i̸f̸y̸ ̸n̸o̸ ̸p̸a̸n̸i̸c̸"),
        ("math_astral", "𝔘𝔫𝔦𝔠𝔬𝔡𝔢 𝕸𝖆𝖙𝖍 𝔞𝔫𝔡 𝒢ℯℴ𝓂ℯ𝓉𝓇𝓎: ∬_D f(x,y) dA = ∭_V ∇⋅F dV"),
    ];

    for (label, text) in multi_script_samples {
        let node = NeopressNode {
            id: format!("node_{}", label),
            kind: NodeKind::CallToAction {
                title: text.to_string(),
                description: format!("Sub-text: {}", text),
                button_text: "Action 🚀".to_string(),
                button_url: "/action".to_string(),
                skin: Some("cover".to_string()),
                ribbon: Some("新".to_string()),
                image_url: None,
                secondary_button_text: None,
                secondary_button_url: None,
            },
            styles: StyleProperties {
                custom_css: Some(format!("/* {} comment */", text)),
                ..Default::default()
            },
            children: Vec::new(),
        };

        let layout = NeopressLayout {
            version: 1,
            root: node,
            global_styles: None,
        };

        // Roundtrip JSON
        let json_str = serde_json::to_string(&layout).expect("Serialization failed");
        let deserialized: NeopressLayout = serde_json::from_str(&json_str).expect("Deserialization failed");
        assert_eq!(layout, deserialized);

        // SSR HTML rendering
        let html = render_layout_to_html(&deserialized);
        assert!(!html.is_empty());
        assert!(html.contains(&format!("id=\"node_{}\"", label)));

        // Scoped CSS
        let css = compile_layout_css(&deserialized);
        let _ = css;
    }
}

// =========================================================================
// 4. Nested Responsive Styles, Breakpoint Visibility & CSS Compiler
// =========================================================================

#[test]
fn test_nested_responsive_styles_and_breakpoint_visibility() {
    let mut desktop_hidden = StyleProperties::default();
    desktop_hidden.hide_desktop = Some(true);

    let mut tablet_hidden = StyleProperties::default();
    tablet_hidden.hide_tablet = Some(true);

    let mut mobile_hidden = StyleProperties::default();
    mobile_hidden.hide_mobile = Some(true);

    let node = NeopressNode {
        id: "multi_level_tree".to_string(),
        kind: NodeKind::Section,
        styles: StyleProperties {
            display: Some("flex".to_string()),
            flex_direction: Some("row".to_string()),
            tablet: Some(Box::new(StyleProperties {
                flex_direction: Some("column".to_string()),
                gap: Some("16px".to_string()),
                ..Default::default()
            })),
            mobile: Some(Box::new(StyleProperties {
                padding_top: Some("8px".to_string()),
                ..Default::default()
            })),
            hide_mobile: Some(true),
            ..Default::default()
        },
        children: vec![
            NeopressNode {
                id: "child_desktop_only".to_string(),
                kind: NodeKind::Heading { text: "Desktop Only".to_string(), level: 2 },
                styles: desktop_hidden,
                children: Vec::new(),
            },
            NeopressNode {
                id: "child_tablet_hidden".to_string(),
                kind: NodeKind::Text { content_html: "<p>Hidden on Tablet</p>".to_string() },
                styles: tablet_hidden,
                children: Vec::new(),
            },
            NeopressNode {
                id: "child_mobile_hidden".to_string(),
                kind: NodeKind::Button { text: "No Mobile".to_string(), url: "/desktop".to_string(), target: "_self".to_string() },
                styles: mobile_hidden,
                children: Vec::new(),
            },
        ],
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    let compiled_css = compile_layout_css(&layout);

    // Desktop rule for hide_desktop
    assert!(compiled_css.contains("[id=\"child_desktop_only\"] { display: none !important; }"));

    // Tablet media query
    assert!(compiled_css.contains("@media (max-width: 1024px)"));
    assert!(compiled_css.contains("[id=\"multi_level_tree\"] { flex-direction: column; gap: 16px; }"));
    assert!(compiled_css.contains("[id=\"child_tablet_hidden\"] { display: none !important; }"));

    // Mobile media query
    assert!(compiled_css.contains("@media (max-width: 767px)"));
    assert!(compiled_css.contains("[id=\"multi_level_tree\"] { padding-top: 8px; }"));
    assert!(compiled_css.contains("[id=\"multi_level_tree\"] { display: none !important; }"));
    assert!(compiled_css.contains("[id=\"child_mobile_hidden\"] { display: none !important; }"));
}

// =========================================================================
// 5. Divi 5 Routing & Parsing Robustness Under Adversarial Variations
// =========================================================================

#[test]
fn test_divi5_whitespace_and_mixed_routing_robustness() {
    // 1. Divi 5 with varied whitespace around comment markers
    let divi5_content = "   \n\n  <!-- wp:divi/section {\"module\":{\"props\":{\"attrs\":{\"module_id\":\"adv_divi5\"}}}} -->\n"
        .to_string()
        + "  <!-- wp:divi/row -->\n"
        + "    <!-- wp:divi/column {\"type\":\"4_4\"} -->\n"
        + "      <!-- wp:divi/button {\"button_text\":\"Explore Features\",\"button_url\":\"/features\"} /-->\n"
        + "    <!-- /wp:divi/column -->\n"
        + "  <!-- /wp:divi/row -->\n"
        + "<!-- /wp:divi/section -->\n   ";

    let empty_meta = HashMap::new();
    let layout = transpile_content(&divi5_content, &empty_meta).expect("Divi 5 transpilation with leading whitespace failed");
    assert_eq!(layout.root.children.len(), 1);
    let sec = &layout.root.children[0];
    assert_eq!(sec.id, "adv_divi5");

    let row = &sec.children[0];
    let col = &row.children[0];
    assert_eq!(col.children.len(), 1);
    match &col.children[0].kind {
        NodeKind::Button { text, url, .. } => {
            assert_eq!(text, "Explore Features");
            assert_eq!(url, "/features");
        }
        _ => panic!("Expected Button node from self-closing wp:divi/button"),
    }

    // 2. Priority check: When Elementor _elementor_data is present in meta, Elementor takes precedence
    let mut elem_meta = HashMap::new();
    elem_meta.insert("_elementor_data".to_string(), r#"[{"id":"elem_prio","elType":"section","elements":[]}]"#.to_string());
    let prio_layout = transpile_content(&divi5_content, &elem_meta).expect("Priority transpilation failed");
    assert_eq!(prio_layout.root.children[0].id, "elem_prio");
}

// =========================================================================
// 6. Deep Nested Style Sync: sync_flat_and_nested Bidirectional Integrity
// =========================================================================

#[test]
fn test_style_properties_sync_flat_and_nested_bidirectional() {
    // Scenario A: Flat properties populate nested Spacing
    let mut flat_style = StyleProperties {
        padding_top: Some("20px".to_string()),
        padding_right: Some("30px".to_string()),
        padding_bottom: Some("40px".to_string()),
        padding_left: Some("50px".to_string()),
        margin_top: Some("10px".to_string()),
        margin_bottom: Some("15px".to_string()),
        ..Default::default()
    };
    flat_style.sync_flat_and_nested();

    let pad = flat_style.padding.as_ref().expect("padding should be populated from flat");
    assert_eq!(pad.top.as_deref(), Some("20px"));
    assert_eq!(pad.right.as_deref(), Some("30px"));
    assert_eq!(pad.bottom.as_deref(), Some("40px"));
    assert_eq!(pad.left.as_deref(), Some("50px"));

    let mar = flat_style.margin.as_ref().expect("margin should be populated from flat");
    assert_eq!(mar.top.as_deref(), Some("10px"));
    assert_eq!(mar.bottom.as_deref(), Some("15px"));

    // Scenario B: Nested Spacing populates flat properties
    let mut nested_style = StyleProperties {
        padding: Some(Spacing {
            top: Some("1.5rem".to_string()),
            right: Some("2rem".to_string()),
            bottom: Some("1.5rem".to_string()),
            left: Some("2rem".to_string()),
        }),
        ..Default::default()
    };
    nested_style.sync_flat_and_nested();

    assert_eq!(nested_style.padding_top.as_deref(), Some("1.5rem"));
    assert_eq!(nested_style.padding_right.as_deref(), Some("2rem"));
    assert_eq!(nested_style.padding_bottom.as_deref(), Some("1.5rem"));
    assert_eq!(nested_style.padding_left.as_deref(), Some("2rem"));
}
