use neopress_ast::render::{
    clean_empty_class_artifacts, render_layout_to_html,
    sanitize_ssr_html, strip_editor_classes,
};
use neopress_ast::schema::{
    NeopressLayout, NeopressNode, NodeKind, PriceFeatureItem, StyleProperties,
};

#[test]
fn test_attribute_escaping_xss_vectors() {
    let node = NeopressNode {
        id: "sec_\"1\"".to_string(),
        kind: NodeKind::SearchForm {
            placeholder: "Search \"articles\" & <guides> '2026'".to_string(),
            button_text: "Find & Submit >".to_string(),
            endpoint: "/search?q=\"test\"&filter=<all>".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    let html = render_layout_to_html(&layout);

    // Verify attribute quotes escaped
    assert!(
        html.contains("placeholder=\"Search &quot;articles&quot; &amp; &lt;guides&gt; '2026'\""),
        "Placeholder attribute was not properly escaped; got: {}",
        html
    );
    assert!(
        html.contains("action=\"/search?q=&quot;test&quot;&amp;filter=&lt;all&gt;\""),
        "Action attribute was not properly escaped; got: {}",
        html
    );
    assert!(
        html.contains(">Find &amp; Submit &gt;</button>"),
        "Button text was not properly escaped; got: {}",
        html
    );
    assert!(
        !html.contains("id=\"sec_\"1\"\""),
        "ID attribute has unescaped quotes"
    );
    assert!(
        html.contains("id=\"sec_&quot;1&quot;\""),
        "Expected escaped ID attribute"
    );
}

#[test]
fn test_login_form_attribute_escaping() {
    let node = NeopressNode {
        id: "login_\"form\"".to_string(),
        kind: NodeKind::LoginForm {
            label_username: "Username & \"Email\"".to_string(),
            label_password: "Password <Secret>".to_string(),
            redirect_url: "/dashboard?ref=\"hero\"&tab=<main>".to_string(),
            remember_me: true,
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    let html = render_layout_to_html(&layout);

    assert!(
        html.contains("value=\"/dashboard?ref=&quot;hero&quot;&amp;tab=&lt;main&gt;\""),
        "Login redirect value attribute was not properly escaped; got: {}",
        html
    );
    assert!(
        html.contains("<label>Username &amp; &quot;Email&quot;</label>"),
        "Username label was not escaped; got: {}",
        html
    );
    assert!(
        html.contains("<label>Password &lt;Secret&gt;</label>"),
        "Password label was not escaped; got: {}",
        html
    );
}

#[test]
fn test_firewall_case_insensitivity_and_unquoted() {
    let raw = r#"<div cOnTeNtEdItAbLe="true" DRAGGABLE='false' contenteditable=true contenteditable data-elementor-type="widget" data-neostudio-block="heading" class="card elementor-grid-handle neostudio-canvas"><p>Content</p></div>"#;
    let sanitized = sanitize_ssr_html(raw);

    assert!(!sanitized.to_ascii_lowercase().contains("contenteditable"), "Leaked contenteditable");
    assert!(!sanitized.to_ascii_lowercase().contains("draggable"), "Leaked draggable");
    assert!(!sanitized.contains("data-elementor-type"), "Leaked data-elementor-type");
    assert!(!sanitized.contains("data-neostudio-block"), "Leaked data-neostudio-block");
    assert!(!sanitized.contains("elementor-grid-handle"), "Leaked elementor-grid-handle");
    assert!(!sanitized.contains("neostudio-canvas"), "Leaked neostudio-canvas");
    assert!(sanitized.contains("class=\"card\""), "Legitimate class 'card' missing");
}

#[test]
fn test_firewall_empty_class_cleanup() {
    let raw = r#"<div class="elementor-grid-handle neostudio-handle ui-resizable"><span class="   ">Text</span></div>"#;
    let sanitized = sanitize_ssr_html(raw);

    assert!(!sanitized.contains("class=\"\""), "Found empty class=\"\"");
    assert!(!sanitized.contains("class=\"   \""), "Found whitespace class");
    assert!(!sanitized.contains("class="), "Should have removed all class attributes when empty");
}

#[test]
fn test_firewall_preserves_legitimate_attributes() {
    let raw = r#"<div class="elementor-section elementor-container et_pb_section et_pb_row" data-post-id="42" data-track="xyz"><p>Safe</p></div>"#;
    let sanitized = sanitize_ssr_html(raw);

    assert!(sanitized.contains("elementor-section"), "Legitimate Elementor frontend class stripped");
    assert!(sanitized.contains("elementor-container"), "Legitimate Elementor container class stripped");
    assert!(sanitized.contains("et_pb_section"), "Legitimate Divi section class stripped");
    assert!(sanitized.contains("et_pb_row"), "Legitimate Divi row class stripped");
    assert!(sanitized.contains("data-post-id=\"42\""), "Legitimate data attribute stripped");
    assert!(sanitized.contains("data-track=\"xyz\""), "Legitimate data attribute stripped");
}

#[test]
fn test_strip_editor_classes_standalone() {
    let classes = "card elementor-grid-handle btn neostudio-handle ui-sortable active";
    let cleaned = strip_editor_classes(classes);
    assert_eq!(cleaned, "card btn active");

    let only_editor = "elementor-grid-handle ui-resizable visual-builder-active";
    let cleaned_empty = strip_editor_classes(only_editor);
    assert_eq!(cleaned_empty, "");
}

#[test]
fn test_clean_empty_class_artifacts_standalone() {
    let html = r#"<div class="" id="1"><span class="   ">Text</span><p class="card   active  ">Paragraph</p></div>"#;
    let cleaned = clean_empty_class_artifacts(html);
    assert!(!cleaned.contains("class=\"\""), "Should not contain empty class");
    assert!(!cleaned.contains("class=\"   \""), "Should not contain whitespace class");
    assert!(cleaned.contains("class=\"card active\""), "Should normalize whitespace in remaining classes");
}

#[test]
fn test_commercial_pro_block_attribute_breakout_resilience() {
    let node = NeopressNode {
        id: "price_card_\"hack\"".to_string(),
        kind: NodeKind::PriceCard {
            title: "Pro \"Enterprise\" & <Tier>".to_string(),
            subtitle: Some("For teams \"ready\" to scale & grow".to_string()),
            currency: Some("$".to_string()),
            price: "99".to_string(),
            original_price: Some("149".to_string()),
            period: Some("month & \"annual\"".to_string()),
            badge: Some("Popular & \"Best Value\" <Top>".to_string()),
            featured: true,
            features: vec![
                PriceFeatureItem {
                    text: "Feature 1 & \"unlimited\"".to_string(),
                    included: true,
                    icon: None,
                },
                PriceFeatureItem {
                    text: "Feature 2 <beta>".to_string(),
                    included: false,
                    icon: None,
                },
            ],
            button_text: "Upgrade \"Now\" & Start".to_string(),
            button_url: "/checkout?plan=pro\" onmouseover=\"alert('xss')".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    let html = render_layout_to_html(&layout);

    // Verify button URL cannot break out of href attribute
    assert!(!html.contains("onmouseover=\"alert('xss')\""), "Attribute breakout in PriceCard button URL!");
    assert!(html.contains("href=\"/checkout?plan=pro&quot; onmouseover=&quot;alert('xss')\""));
    assert!(html.contains("Popular &amp; &quot;Best Value&quot; &lt;Top&gt;"));
    assert!(html.contains("For teams &quot;ready&quot; to scale &amp; grow"));
    assert!(html.contains("Feature 1 &amp; &quot;unlimited&quot;"));
    assert!(html.contains("Feature 2 &lt;beta&gt;"));
}

#[test]
fn test_call_to_action_attribute_breakout_resilience() {
    let node = NeopressNode {
        id: "cta_\"1\"".to_string(),
        kind: NodeKind::CallToAction {
            skin: Some("classic".to_string()),
            title: "Ready to \"Deploy\"?".to_string(),
            description: "Join > 10,000 developers & teams".to_string(),
            button_text: "Get Started \"Free\"".to_string(),
            button_url: "https://example.com/start\" onclick=\"evil()".to_string(),
            secondary_button_text: Some("Docs & \"API\"".to_string()),
            secondary_button_url: Some("/docs\" onfocus=\"evil()".to_string()),
            ribbon: Some("New \"2026\" & Fast".to_string()),
            image_url: Some("https://example.com/img.png\" onerror=\"evil()".to_string()),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    let html = render_layout_to_html(&layout);

    assert!(!html.contains("onclick=\"evil()\""), "Breakout in CTA button_url");
    assert!(!html.contains("onfocus=\"evil()\""), "Breakout in CTA secondary_button_url");
    assert!(!html.contains("onerror=\"evil()\""), "Breakout in CTA image_url");
    assert!(html.contains("https://example.com/img.png&quot; onerror=&quot;evil()"));
}

#[test]
fn test_nav_menu_and_animated_headline_resilience() {
    let node = NeopressNode {
        id: "nav_\"1\"".to_string(),
        kind: NodeKind::AnimatedHeadline {
            before_text: "Build \"faster\" &".to_string(),
            words: vec!["Smart \"Apps\"".to_string(), "Better <Sites>".to_string()],
            after_text: "with Neopress & \"Rust\"".to_string(),
            animation_style: "rotate".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    let html = render_layout_to_html(&layout);

    // Verify data-words JSON attribute does not break out
    assert!(!html.contains("data-words=\"[\""), "Unescaped quotes inside data-words attribute");
    assert!(html.contains("data-words=\"[&quot;Smart \\&quot;Apps\\&quot;&quot;,&quot;Better &lt;Sites&gt;&quot;]\""));
}

#[test]
fn test_sanitizer_edge_contenteditable_and_draggable_behavior() {
    // Test that editor attributes are stripped without touching non-editor content
    let raw = r#"<div contenteditable="true" draggable="false" data-elementor-settings="{}"><p>Clean text</p></div>"#;
    let sanitized = sanitize_ssr_html(raw);
    assert_eq!(sanitized, "<div><p>Clean text</p></div>");

    // Test with boolean / valueless draggable and contenteditable
    let raw_bare = r#"<div contenteditable draggable><p>Bare</p></div>"#;
    let sanitized_bare = sanitize_ssr_html(raw_bare);
    assert_eq!(sanitized_bare, "<div><p>Bare</p></div>");

    // Test with mixed whitespace, newline and uppercase
    let raw_whitespace = "<div\n  CONTENTEDITABLE=\"true\"\n  DRAGGABLE=\"true\"\n  class=\"elementor-grid-handle\"\n><p>Whitespace</p></div>";
    let sanitized_ws = sanitize_ssr_html(raw_whitespace);
    assert!(!sanitized_ws.to_ascii_lowercase().contains("contenteditable"));
    assert!(!sanitized_ws.to_ascii_lowercase().contains("draggable"));
    assert!(!sanitized_ws.contains("elementor-grid-handle"));
}
