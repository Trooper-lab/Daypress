use neopress_ast::*;

#[test]
fn test_scoped_css_and_media_query_compilation() {
    let mut hover_styles = StyleProperties::default();
    hover_styles.background_color = Some("#ff0000".to_string());

    let mut tab_styles = StyleProperties::default();
    tab_styles.font_size = Some("18px".to_string());

    let mut mob_styles = StyleProperties::default();
    mob_styles.font_size = Some("14px".to_string());

    let mut node_styles = StyleProperties::default();
    node_styles.hover = Some(Box::new(hover_styles));
    node_styles.tablet = Some(Box::new(tab_styles));
    node_styles.mobile = Some(Box::new(mob_styles));
    node_styles.hide_mobile = Some(true);

    let layout = NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "test_hero_node".to_string(),
            kind: NodeKind::Section,
            styles: node_styles,
            children: Vec::new(),
        },
        global_styles: None,
    };

    let css = compile_layout_css(&layout);
    assert!(css.contains("[id=\"test_hero_node\"]:hover"));
    assert!(css.contains("background-color: #ff0000;"));
    assert!(css.contains("@media (max-width: 1024px)"));
    assert!(css.contains("font-size: 18px;"));
    assert!(css.contains("@media (max-width: 767px)"));
    assert!(css.contains("font-size: 14px;"));
    assert!(css.contains("display: none !important;"));
}

#[test]
fn test_3_tier_dom_leak_sanitizer() {
    let leaky_html = "<div class=\"elementor-grid-handle active-element ui-resizable\" contenteditable=\"true\" draggable=\"true\"><h1 contenteditable=\"true\">Safe Content</h1></div>";

    let layout = NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "leaky_root".to_string(),
            kind: NodeKind::CustomHtml {
                raw: leaky_html.to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        global_styles: None,
    };

    let ssr_output = render_layout_to_html(&layout);
    assert!(!ssr_output.contains("contenteditable"));
    assert!(!ssr_output.contains("draggable"));
    assert!(!ssr_output.contains("elementor-grid-handle"));
    assert!(!ssr_output.contains("active-element"));
    assert!(ssr_output.contains("Safe Content"));
}

#[test]
fn test_pro_blocks_html_rendering() {
    let price_card = NeopressNode {
        id: "card_1".to_string(),
        kind: NodeKind::PriceCard {
            title: "Pro Tier".to_string(),
            subtitle: Some("For fast sites".to_string()),
            currency: Some("$".to_string()),
            price: "49".to_string(),
            original_price: Some("99".to_string()),
            period: Some("mo".to_string()),
            badge: Some("Popular".to_string()),
            featured: true,
            features: vec![
                PriceFeatureItem {
                    text: "Sub-millisecond latency".to_string(),
                    included: true,
                    icon: None,
                },
                PriceFeatureItem {
                    text: "PHP Engine".to_string(),
                    included: false,
                    icon: None,
                },
            ],
            button_text: "Get Started".to_string(),
            button_url: "/checkout".to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let cta = NeopressNode {
        id: "cta_1".to_string(),
        kind: NodeKind::CallToAction {
            title: "Migrate Today".to_string(),
            description: "One click migration with zero regressions.".to_string(),
            button_text: "Start Migration".to_string(),
            button_url: "/start".to_string(),
            skin: Some("cover".to_string()),
            ribbon: Some("NEW".to_string()),
            image_url: Some("https://example.com/banner.jpg".to_string()),
            secondary_button_text: Some("Learn More".to_string()),
            secondary_button_url: Some("/docs".to_string()),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "pro_root".to_string(),
            kind: NodeKind::Container,
            styles: StyleProperties::default(),
            children: vec![price_card, cta],
        },
        global_styles: None,
    };

    let html = render_layout_to_html(&layout);
    assert!(html.contains("np-price-card-featured"));
    assert!(html.contains("np-price-badge"));
    assert!(html.contains("Popular"));
    assert!(html.contains("Sub-millisecond latency"));
    assert!(html.contains("included"));
    assert!(html.contains("excluded"));
    assert!(html.contains("np-cta-ribbon"));
    assert!(html.contains("NEW"));
    assert!(html.contains("np-cta-sec-btn"));
    assert!(html.contains("Learn More"));
}
