use neopress_ast::render::{compile_layout_css, render_layout_to_html, render_node_to_html, sanitize_ssr_html};
use neopress_ast::schema::{
    NavMenuItem, NeopressLayout, NeopressNode, NodeKind, PriceFeatureItem,
    SharePlatformItem, SocialIconItem, StyleProperties, TestimonialSlide,
};
use neopress_ast::{transpile_content, transpile_divi_content};
use serde_json::json;
use std::collections::HashMap;

// =========================================================================
// 1. Pro NodeKind Serialization / Deserialization Round-Trip Tests (All 24+)
// =========================================================================

#[test]
fn test_roundtrip_all_pro_node_kinds() {
    let pro_nodes = vec![
        NeopressNode {
            id: "price_card_test".to_string(),
            kind: NodeKind::PriceCard {
                title: "Scale Enterprise".to_string(),
                subtitle: Some("High availability clusters".to_string()),
                currency: Some("$".to_string()),
                price: "299".to_string(),
                original_price: Some("399".to_string()),
                period: Some("month".to_string()),
                badge: Some("Best Value".to_string()),
                featured: true,
                features: vec![
                    PriceFeatureItem {
                        text: "Unlimited edge nodes".to_string(),
                        included: true,
                        icon: Some("fa-solid fa-check".to_string()),
                    },
                    PriceFeatureItem {
                        text: "Legacy PHP runtime".to_string(),
                        included: false,
                        icon: None,
                    },
                ],
                button_text: "Deploy Cluster".to_string(),
                button_url: "/deploy".to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "price_table_test".to_string(),
            kind: NodeKind::PriceTable {
                title: "Standard Plan".to_string(),
                price: "$49".to_string(),
                period: "mo".to_string(),
                features: vec!["10 Sites".to_string(), "Daily Backups".to_string()],
                button_text: "Sign Up".to_string(),
                button_url: "/signup".to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "cta_test".to_string(),
            kind: NodeKind::CallToAction {
                title: "Migrate in 60 Seconds".to_string(),
                description: "Sub-millisecond WordPress replacement.".to_string(),
                button_text: "Start Free Trial".to_string(),
                button_url: "/trial".to_string(),
                skin: Some("cover".to_string()),
                ribbon: Some("HOT".to_string()),
                image_url: Some("https://example.com/banner.webp".to_string()),
                secondary_button_text: Some("Read Docs".to_string()),
                secondary_button_url: Some("/docs".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "testimonial_test".to_string(),
            kind: NodeKind::Testimonial {
                content: "Neopress handles 95,000 req/sec effortlessly!".to_string(),
                name: "Linus Torvalds".to_string(),
                job: "Principal Engineer".to_string(),
                image_url: Some("https://example.com/avatar.jpg".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "testimonial_carousel_test".to_string(),
            kind: NodeKind::TestimonialCarousel {
                slides: vec![
                    TestimonialSlide {
                        quote: "Zero memory leaks.".to_string(),
                        author: "Alice Smith".to_string(),
                        role: Some("CTO".to_string()),
                        company: Some("Acme Corp".to_string()),
                        avatar_url: Some("https://example.com/alice.png".to_string()),
                        rating: Some(5.0),
                    },
                    TestimonialSlide {
                        quote: "Sub-millisecond TTFB.".to_string(),
                        author: "Bob Jones".to_string(),
                        role: Some("Lead Architect".to_string()),
                        company: None,
                        avatar_url: None,
                        rating: Some(4.8),
                    },
                ],
                skin: Some("card".to_string()),
                autoplay: true,
                loop_slides: true,
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "countdown_test".to_string(),
            kind: NodeKind::Countdown {
                target_date: "2026-12-31 23:59:59".to_string(),
                mode: "due_date".to_string(),
                show_days: true,
                show_hours: true,
                show_minutes: true,
                show_seconds: true,
                expiry_message: Some("Special offer expired!".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "flipbox_test".to_string(),
            kind: NodeKind::FlipBox {
                front_icon: Some("fa-solid fa-cube".to_string()),
                front_image: None,
                front_title: "Native Architecture".to_string(),
                front_description: "Rust + Tokio + Axum".to_string(),
                back_title: "Extreme Speed".to_string(),
                back_description: "Zero GC pauses".to_string(),
                button_text: Some("Learn Architecture".to_string()),
                button_url: Some("/architecture".to_string()),
                direction: "flip_left".to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "animated_headline_test".to_string(),
            kind: NodeKind::AnimatedHeadline {
                before_text: "Build websites that are ".to_string(),
                words: vec!["Instant".to_string(), "Unbreakable".to_string(), "Scalable".to_string()],
                after_text: " with Neopress.".to_string(),
                animation_style: "typing".to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "rating_test".to_string(),
            kind: NodeKind::Rating {
                rating: 4.8,
                max_rating: 5,
                title: Some("User Score".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "star_rating_test".to_string(),
            kind: NodeKind::StarRating {
                rating: 4.5,
                max_rating: 5,
                title: Some("Overall Satisfaction".to_string()),
                star_color: Some("#f59e0b".to_string()),
                star_size: Some(20),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "progress_bar_test".to_string(),
            kind: NodeKind::ProgressBar {
                title: "Server Migration".to_string(),
                percent: 92,
                display_text: Some("92% Complete".to_string()),
                track_color: Some("#1e293b".to_string()),
                bar_color: Some("#10b981".to_string()),
                height: Some(16),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "alert_test".to_string(),
            kind: NodeKind::Alert {
                alert_type: "warning".to_string(),
                title: "SSL Certificate Expiring".to_string(),
                description: "Auto-renewing via Let's Encrypt in 2 days.".to_string(),
                dismissible: true,
                icon: Some("fa-solid fa-triangle-exclamation".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "counter_test".to_string(),
            kind: NodeKind::Counter {
                number: 95000,
                prefix: Some(">".to_string()),
                suffix: Some(" req/s".to_string()),
                title: Some("Throughput".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "benchmark_test".to_string(),
            kind: NodeKind::Benchmark {
                label: "Requests / Sec".to_string(),
                value: "95,000".to_string(),
                comparison: "39x faster than PHP 8.2".to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "social_icons_test".to_string(),
            kind: NodeKind::SocialIcons {
                items: vec![
                    SocialIconItem {
                        icon: "fa-brands fa-github".to_string(),
                        link: "https://github.com/neopress".to_string(),
                    },
                    SocialIconItem {
                        icon: "fa-brands fa-x-twitter".to_string(),
                        link: "https://x.com/neopress".to_string(),
                    },
                ],
                skin: Some("rounded".to_string()),
                color_type: Some("custom".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "share_buttons_test".to_string(),
            kind: NodeKind::ShareButtons {
                platforms: vec![
                    SharePlatformItem {
                        platform: "twitter".to_string(),
                        label: Some("Tweet".to_string()),
                        url_template: Some("https://twitter.com/intent/tweet?url={url}".to_string()),
                    },
                    SharePlatformItem {
                        platform: "linkedin".to_string(),
                        label: Some("Share".to_string()),
                        url_template: None,
                    },
                ],
                skin: Some("flat".to_string()),
                official_colors: true,
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "nav_menu_test".to_string(),
            kind: NodeKind::NavMenu {
                menu_slug: "main-nav".to_string(),
                layout: "horizontal".to_string(),
                align: "center".to_string(),
                mobile_hamburger: true,
                items: vec![
                    NavMenuItem {
                        title: "Home".to_string(),
                        url: "/".to_string(),
                        order: 0,
                        ..Default::default()
                    },
                    NavMenuItem {
                        title: "Features".to_string(),
                        url: "/features".to_string(),
                        order: 1,
                        ..Default::default()
                    },
                    NavMenuItem {
                        title: "Pricing".to_string(),
                        url: "/pricing".to_string(),
                        order: 2,
                        ..Default::default()
                    },
                ],
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "megamenu_test".to_string(),
            kind: NodeKind::MegaMenuContainer {
                trigger: ".nav-products-link".to_string(),
                width: Some("1200px".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "popup_modal_test".to_string(),
            kind: NodeKind::PopupModal {
                modal_id: "subscribe_modal".to_string(),
                trigger_selector: Some("#open-sub-btn".to_string()),
                overlay_bg: Some("rgba(0,0,0,0.75)".to_string()),
                close_button: true,
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "loop_grid_test".to_string(),
            kind: NodeKind::LoopGrid {
                post_type: "post".to_string(),
                posts_per_page: 9,
                columns: 3,
                show_thumbnail: true,
                show_excerpt: true,
                show_meta: true,
                show_pagination: true,
                taxonomy: Some("category:news".to_string()),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "toc_test".to_string(),
            kind: NodeKind::TableOfContents {
                title: "In This Article".to_string(),
                min_depth: 2,
                max_depth: 4,
                collapsible: true,
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "breadcrumbs_test".to_string(),
            kind: NodeKind::Breadcrumbs {
                separator: " / ".to_string(),
                home_text: "Home".to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "search_form_test".to_string(),
            kind: NodeKind::SearchForm {
                placeholder: "Search docs...".to_string(),
                button_text: "Search".to_string(),
                endpoint: "/api/search".to_string(),
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
        NeopressNode {
            id: "login_form_test".to_string(),
            kind: NodeKind::LoginForm {
                label_username: "Username or Email".to_string(),
                label_password: "Your Password".to_string(),
                redirect_url: "/admin".to_string(),
                remember_me: true,
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        },
    ];

    for (i, node) in pro_nodes.into_iter().enumerate() {
        // Test direct serialization and deserialization
        let json = serde_json::to_string(&node)
            .unwrap_or_else(|e| panic!("Failed to serialize node #{}: {:?}", i, e));
        
        let deserialized: NeopressNode = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("Failed to deserialize node #{}: {:?}\nJSON: {}", i, e, json));

        assert_eq!(
            node.id, deserialized.id,
            "ID mismatch on node variant #{}",
            i
        );
        assert_eq!(
            node.kind, deserialized.kind,
            "NodeKind mismatch on node variant #{}\nExpected: {:?}\nGot: {:?}",
            i, node.kind, deserialized.kind
        );

        // Test rendering to HTML without error
        let html = render_node_to_html(&deserialized);
        assert!(
            !html.is_empty(),
            "Empty HTML output for node variant #{}",
            i
        );
    }
}

// =========================================================================
// 2. Flat Visual Builder Format vs Nested AST Format Interchangeability
// =========================================================================

#[test]
fn test_flat_visual_builder_data_deserialization() {
    // 1. Flat PriceCard without "data" wrapper
    let flat_price_card = r#"{
        "id": "flat_pc_1",
        "type": "PriceCard",
        "title": "Startup Bundle",
        "price": "49",
        "featured": true,
        "features": [
            { "text": "SSL Certificate", "included": true }
        ],
        "styles": {}
    }"#;

    let node: NeopressNode = serde_json::from_str(flat_price_card)
        .expect("Flat PriceCard should deserialize cleanly");
    match &node.kind {
        NodeKind::PriceCard { title, price, featured, features, .. } => {
            assert_eq!(title, "Startup Bundle");
            assert_eq!(price, "49");
            assert!(*featured);
            assert_eq!(features.len(), 1);
            assert!(features[0].included);
        }
        _ => panic!("Expected PriceCard variant"),
    }

    // 2. Flat Counter with formatted number string and styles
    let flat_counter = r#"{
        "id": "flat_cnt_1",
        "type": "Counter",
        "number": "99.9%",
        "label": "Uptime SLA",
        "styles": {}
    }"#;

    let node_cnt: NeopressNode = serde_json::from_str(flat_counter)
        .expect("Flat Counter should deserialize cleanly");
    match &node_cnt.kind {
        NodeKind::Counter { number, suffix, title, .. } => {
            assert_eq!(*number, 99); // String "99.9%" integer part is parsed as 99
            assert_eq!(suffix.as_deref(), Some("%"));
            assert_eq!(title.as_deref(), Some("Uptime SLA"));
        }
        _ => panic!("Expected Counter variant"),
    }

    // 3. Flat Spacer taking height from style height
    let flat_spacer = r#"{
        "id": "flat_sp_1",
        "type": "Spacer",
        "styles": { "height": "64px" }
    }"#;

    let node_sp: NeopressNode = serde_json::from_str(flat_spacer)
        .expect("Flat Spacer should extract height from style");
    match &node_sp.kind {
        NodeKind::Spacer { height } => assert_eq!(*height, 64),
        _ => panic!("Expected Spacer with 64px height"),
    }

    // 4. Flat Video with "aspect" alias and ratio normalization
    let flat_vid = r#"{
        "id": "flat_vid_1",
        "type": "Video",
        "url": "https://youtu.be/dQw4w9WgXcQ",
        "aspect": "4:3",
        "loop": true,
        "styles": {}
    }"#;

    let node_vid: NeopressNode = serde_json::from_str(flat_vid)
        .expect("Flat Video should deserialize cleanly");
    match &node_vid.kind {
        NodeKind::Video { url, aspect_ratio, loop_video, .. } => {
            assert_eq!(url, "https://youtu.be/dQw4w9WgXcQ");
            assert_eq!(aspect_ratio, "4/3");
            assert!(*loop_video);
        }
        _ => panic!("Expected Video variant"),
    }
}

// =========================================================================
// 3. Deep StyleProperties: Responsive, Hover & Lossless raw_settings
// =========================================================================

#[test]
fn test_deep_style_properties_with_responsive_hover_and_raw_settings() {
    let mut custom_raw = HashMap::new();
    custom_raw.insert(
        "elementor_motion_fx".to_string(),
        json!({
            "mouse_track": { "speed": 1.5, "direction": "opposite" },
            "opacity_in": [0, 1]
        }),
    );
    custom_raw.insert(
        "divi_custom_addon_property".to_string(),
        json!("unrecognized_third_party_token_v42"),
    );

    let hover_style = StyleProperties {
        background_color: Some("#3b82f6".to_string()),
        transform: Some("translateY(-4px)".to_string()),
        box_shadow: Some("0 10px 25px rgba(0,0,0,0.2)".to_string()),
        ..Default::default()
    };

    let tablet_style = StyleProperties {
        grid_template_columns: Some("repeat(2, 1fr)".to_string()),
        gap: Some("16px".to_string()),
        padding_top: Some("32px".to_string()),
        padding_bottom: Some("32px".to_string()),
        ..Default::default()
    };

    let mobile_style = StyleProperties {
        grid_template_columns: Some("1fr".to_string()),
        gap: Some("12px".to_string()),
        padding_left: Some("16px".to_string()),
        padding_right: Some("16px".to_string()),
        hide_mobile: Some(false),
        ..Default::default()
    };

    let root_style = StyleProperties {
        display: Some("grid".to_string()),
        grid_template_columns: Some("repeat(4, 1fr)".to_string()),
        grid_gap: Some("24px".to_string()),
        position: Some("relative".to_string()),
        z_index: Some("10".to_string()),
        width: Some("100%".to_string()),
        max_width: Some("1440px".to_string()),
        padding_top: Some("64px".to_string()),
        padding_bottom: Some("64px".to_string()),
        background_color: Some("#0f172a".to_string()),
        border_radius: Some("16px".to_string()),
        transition: Some("all 0.25s ease".to_string()),
        custom_css: Some(".np-card { backdrop-filter: blur(8px); }".to_string()),
        hover: Some(Box::new(hover_style)),
        tablet: Some(Box::new(tablet_style)),
        mobile: Some(Box::new(mobile_style)),
        raw_settings: Some(custom_raw),
        ..Default::default()
    };

    let node = NeopressNode {
        id: "deep_grid_node".to_string(),
        kind: NodeKind::Container,
        styles: root_style,
        children: Vec::new(),
    };

    let layout = NeopressLayout {
        version: 1,
        root: node,
        global_styles: None,
    };

    // 1. Serialize to JSON and back
    let json_str = serde_json::to_string_pretty(&layout).expect("Layout serialization failed");
    let roundtripped: NeopressLayout = serde_json::from_str(&json_str).expect("Layout deserialization failed");

    let styles = &roundtripped.root.styles;
    assert_eq!(styles.display.as_deref(), Some("grid"));
    assert_eq!(styles.grid_template_columns.as_deref(), Some("repeat(4, 1fr)"));
    assert_eq!(styles.position.as_deref(), Some("relative"));
    assert_eq!(styles.z_index.as_deref(), Some("10"));

    // Verify raw_settings lossless retention
    let raw = styles.raw_settings.as_ref().expect("raw_settings should be preserved");
    assert!(raw.contains_key("elementor_motion_fx"));
    assert_eq!(
        raw.get("divi_custom_addon_property").and_then(|v| v.as_str()),
        Some("unrecognized_third_party_token_v42")
    );

    // Verify hover styles
    let hov = styles.hover.as_ref().expect("Hover styles missing");
    assert_eq!(hov.background_color.as_deref(), Some("#3b82f6"));
    assert_eq!(hov.transform.as_deref(), Some("translateY(-4px)"));

    // Verify tablet styles
    let tab = styles.tablet.as_ref().expect("Tablet styles missing");
    assert_eq!(tab.grid_template_columns.as_deref(), Some("repeat(2, 1fr)"));
    assert_eq!(tab.padding_top.as_deref(), Some("32px"));

    // Verify mobile styles
    let mob = styles.mobile.as_ref().expect("Mobile styles missing");
    assert_eq!(mob.grid_template_columns.as_deref(), Some("1fr"));
    assert_eq!(mob.padding_left.as_deref(), Some("16px"));

    // 2. Test CSS compiler output
    let compiled_css = compile_layout_css(&roundtripped);
    assert!(compiled_css.contains("[id=\"deep_grid_node\"]:hover"));
    assert!(compiled_css.contains("background-color: #3b82f6;"));
    assert!(compiled_css.contains("@media (max-width: 1024px)"));
    assert!(compiled_css.contains("grid-template-columns: repeat(2, 1fr);"));
    assert!(compiled_css.contains("@media (max-width: 767px)"));
    assert!(compiled_css.contains("grid-template-columns: 1fr;"));

    // 3. Test inline HTML SSR generation
    let html = render_layout_to_html(&roundtripped);
    assert!(html.contains("id=\"deep_grid_node\""));
    assert!(html.contains("display: grid;"));
    assert!(html.contains("grid-template-columns: repeat(4, 1fr);"));
}

// =========================================================================
// 4. Unicode Safety & Multi-Byte UTF-8 String Slicing Stress Tests
// =========================================================================

#[test]
fn test_multi_byte_utf8_safety_and_no_slicing_panic() {
    let adversarial_strings = vec![
        "🦀 Pure Rust Engine ⚡ 95k req/s 🚀 100% Native",
        "日本語テスト：東京タワーと桜の花びら 🌸",
        "العربية: اختبار شامل لنظام نيوبيرس بدون أي أخطاء",
        "Русский: Проверка многобайтовой кодировки UTF-8 в заголовках и описаниях",
        "한국어: 네오프레스 웹사이트 빌더 1클릭 마이그레이션 테스트",
        "Zürich Grüezi: Höchstgeschwindigkeit & Präzision Café résumé ñañe",
        "Multi-family Emojis: 👨‍👩‍👧‍👦 🏳️‍🌈 🧑🏽‍💻 🛰️ 🛸 🪐",
        "Currency & Math: €99.99 / ¥12,500 / £45.00 / ₹1,499 / ₿0.0042 / ∑(x) ≥ ∞",
        "XML Entities: <script>alert('xss')</script> &amp; \"double quotes\" 'single' `backticks`",
    ];

    for (idx, text) in adversarial_strings.iter().enumerate() {
        let node = NeopressNode {
            id: format!("utf8_node_{}", idx),
            kind: NodeKind::PriceCard {
                title: text.to_string(),
                subtitle: Some(format!("Sub: {}", text)),
                currency: Some("€".to_string()),
                price: "99".to_string(),
                original_price: Some("149".to_string()),
                period: Some("月".to_string()),
                badge: Some("⭐ 特選".to_string()),
                featured: true,
                features: vec![
                    PriceFeatureItem {
                        text: format!("Feature: {}", text),
                        included: true,
                        icon: Some("fa-solid fa-check".to_string()),
                    },
                ],
                button_text: "購入する 🛒".to_string(),
                button_url: format!("https://example.com/checkout?item={}", idx),
            },
            styles: StyleProperties {
                font_family: Some("'Noto Sans CJK JP', 'Noto Sans Arabic', sans-serif".to_string()),
                text_color: Some("#f8fafc".to_string()),
                ..Default::default()
            },
            children: vec![
                NeopressNode {
                    id: format!("utf8_child_{}", idx),
                    kind: NodeKind::Heading {
                        text: text.to_string(),
                        level: 2,
                    },
                    styles: StyleProperties::default(),
                    children: Vec::new(),
                },
                NeopressNode {
                    id: format!("utf8_text_{}", idx),
                    kind: NodeKind::Text {
                        content_html: format!("<p>{}</p>", text),
                    },
                    styles: StyleProperties::default(),
                    children: Vec::new(),
                },
            ],
        };

        let layout = NeopressLayout {
            version: 1,
            root: node,
            global_styles: None,
        };

        // 1. Serialization & Deserialization must never panic
        let json = serde_json::to_string(&layout)
            .unwrap_or_else(|e| panic!("Failed to serialize UTF-8 string #{}: {:?}", idx, e));
        let roundtrip: NeopressLayout = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("Failed to deserialize UTF-8 string #{}: {:?}", idx, e));

        let mut expected = layout.clone();
        expected.root.styles.sync_flat_and_nested();
        assert_eq!(expected, roundtrip);

        // 2. SSR HTML compilation must never panic on multi-byte UTF-8
        let html = render_layout_to_html(&roundtrip);
        assert!(!html.is_empty());
        assert!(html.contains(&format!("id=\"utf8_node_{}\"", idx)));

        // 3. Scoped CSS compilation must never panic on multi-byte font-families
        let css = compile_layout_css(&roundtrip);
        let _ = css; // Ensure no panic
    }
}

// =========================================================================
// 5. Transpilation Safety on Malformed / Adversarial Content
// =========================================================================

#[test]
fn test_transpile_content_utf8_and_malformed_inputs() {
    let mut meta = HashMap::new();

    // 1. Elementor JSON with multi-byte strings and edge-case URLs
    let elem_utf8_json = r##"[
        {
            "id": "sec_utf8",
            "elType": "section",
            "settings": {
                "background_color": "#0d0f12"
            },
            "elements": [
                {
                    "id": "col_utf8",
                    "elType": "column",
                    "elements": [
                        {
                            "id": "hdg_utf8",
                            "elType": "widget",
                            "widgetType": "heading",
                            "settings": {
                                "title": "🚀 Neopress 高性能 ⚡ 中文 & 日本語 & العربية ",
                                "link": { "url": "https://neopress.org/тест-кириллицы/?параметр=значение#якорь" }
                            }
                        },
                        {
                            "id": "vid_utf8",
                            "elType": "widget",
                            "widgetType": "video",
                            "settings": {
                                "youtube_url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s&feature=share"
                            }
                        }
                    ]
                }
            ]
        }
    ]"##;

    meta.insert("_elementor_data".to_string(), elem_utf8_json.to_string());
    let layout = transpile_content("", &meta).expect("Elementor UTF-8 transpilation failed");
    assert_eq!(layout.root.children.len(), 1);
    let sec = &layout.root.children[0];
    let col = &sec.children[0];
    assert_eq!(col.children.len(), 2);

    match &col.children[0].kind {
        NodeKind::Heading { text, .. } => {
            assert!(text.contains("🚀 Neopress 高性能 ⚡"));
        }
        _ => panic!("Expected Heading node"),
    }

    // 2. Divi 4 shortcode with multi-byte attributes and content
    let divi_utf8 = r##"
        [et_pb_section background_color="#15181e"]
            [et_pb_row]
                [et_pb_column type="4_4"]
                    [et_pb_text header_font="Noto Sans|700||on|||||"]
                        <h1>التحول السريع إلى نيوبريس ⚡</h1>
                        <p>Rust-powered WordPress replacement with zero regressions 🦀.</p>
                    [/et_pb_text]
                    [et_pb_button button_text="시작하기 🚀" button_url="/kr/start"][/et_pb_button]
                    [et_pb_countdown_timer title="الوقت المتبقي للأطلاق" date_time="2026-12-31 23:59:59" /]
                [/et_pb_column]
            [/et_pb_row]
        [/et_pb_section]
    "##;

    let empty_meta = HashMap::new();
    let divi_layout = transpile_content(divi_utf8, &empty_meta).expect("Divi UTF-8 transpilation failed");
    let d_sec = &divi_layout.root.children[0];
    let d_row = &d_sec.children[0];
    let d_col = &d_row.children[0];
    assert_eq!(d_col.children.len(), 3);

    // 3. Divi 5 block comments with UTF-8
    let divi5_utf8 = r#"
        <!-- wp:divi/section {"module":{"props":{"attrs":{"module_id":"divi5-кириллица"}}}} -->
        <!-- wp:divi/row -->
        <!-- wp:divi/column {"type":"1_1"} -->
        <!-- wp:divi/text --><h2>Быстрый перенос без ошибок</h2><!-- /wp:divi/text -->
        <!-- /wp:divi/column -->
        <!-- /wp:divi/row -->
        <!-- /wp:divi/section -->
    "#;

    // 3a. Direct call to transpile_divi_content succeeds:
    let divi5_direct = transpile_divi_content(divi5_utf8).expect("Divi 5 direct transpilation failed");
    let d5_dir_sec = &divi5_direct.root.children[0];
    assert_eq!(d5_dir_sec.id, "divi5-кириллица");

    // 3b. Universal transpile_content MUST route Divi 5 content to Divi transpiler, not Gutenberg!
    let divi5_layout = transpile_content(divi5_utf8, &empty_meta).expect("Divi 5 UTF-8 transpilation failed");
    let d5_sec = &divi5_layout.root.children[0];
    assert_eq!(
        d5_sec.id, "divi5-кириллица",
        "transpile_content in src/lib.rs fails to route '<!-- wp:divi/' to transpile_divi_content, falling through to Gutenberg!"
    );

    // 4. Gutenberg block comments with UTF-8
    let gb_utf8 = r#"
        <!-- wp:heading {"level":1} -->
        <h1>Transpilation sans faille pour les développeurs exigeants 🥐</h1>
        <!-- /wp:heading -->
    "#;

    let gb_layout = transpile_content(gb_utf8, &empty_meta).expect("Gutenberg UTF-8 transpilation failed");
    assert_eq!(gb_layout.root.children.len(), 1);
}

// =========================================================================
// 6. DOM Leak Sanitizer Stress & Evasion Resistance Matrix
// =========================================================================

#[test]
fn test_dom_leak_sanitizer_adversarial_matrix() {
    let adversarial_snippets = vec![
        // Standard leaks
        ("<div contenteditable=\"true\">Text</div>", "<div>Text</div>"),
        ("<div contenteditable='true'>Text</div>", "<div>Text</div>"),
        ("<div contenteditable=\"false\">Text</div>", "<div>Text</div>"),
        ("<div contenteditable>Text</div>", "<div>Text</div>"),
        ("<div draggable=\"true\">Text</div>", "<div>Text</div>"),
        ("<div draggable='true'>Text</div>", "<div>Text</div>"),
        // Mixed classes
        (
            "<span class=\"btn elementor-grid-handle active-element ui-resizable\">Click</span>",
            "<span class=\"btn    \">Click</span>",
        ),
        (
            "<div class=\"elementor-element-edit-mode elementor-editor-active ui-draggable ui-sortable\">Inside</div>",
            "<div class=\"    \">Inside</div>",
        ),
        // Compound malicious leaks
        (
            "<div class=\"elementor-grid-handle\" contenteditable=\"true\" draggable=\"true\"><h1 contenteditable=\"true\">Safe Title</h1></div>",
            "<div class=\"\"  ><h1 >Safe Title</h1></div>",
        ),
    ];

    for (raw_input, _expected_sanitized) in adversarial_snippets {
        let cleaned = sanitize_ssr_html(raw_input);
        assert!(
            !cleaned.contains("contenteditable"),
            "Failed to strip contenteditable in: {}\nGot: {}",
            raw_input,
            cleaned
        );
        assert!(
            !cleaned.contains("draggable"),
            "Failed to strip draggable in: {}\nGot: {}",
            raw_input,
            cleaned
        );
        assert!(
            !cleaned.contains("elementor-grid-handle"),
            "Failed to strip elementor-grid-handle in: {}\nGot: {}",
            raw_input,
            cleaned
        );
        assert!(
            !cleaned.contains("active-element"),
            "Failed to strip active-element in: {}\nGot: {}",
            raw_input,
            cleaned
        );
    }
}
