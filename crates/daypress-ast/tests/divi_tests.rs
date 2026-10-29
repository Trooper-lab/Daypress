use daypress_ast::*;
use std::collections::HashMap;

#[test]
fn test_divi_transpilation_and_leaf_shortcodes() {
    let divi_content = r##"
        [et_pb_section background_color="#1e293b"]
            [et_pb_row]
                [et_pb_column type="1_2"]
                    [et_pb_divider color="#334155"]
                    [et_pb_text]
                        <p>Left Column Content</p>
                    [/et_pb_text]
                    [et_pb_counter percent="92"]Rust Speed[/et_pb_counter]
                [/et_pb_column]
                [et_pb_column type="1_2"]
                    [et_pb_button button_text="Learn More" button_url="/features"][/et_pb_button]
                    [et_pb_accordion]
                        [et_pb_accordion_item title="FAQ 1"]Answer 1[/et_pb_accordion_item]
                        [et_pb_accordion_item title="FAQ 2"]Answer 2[/et_pb_accordion_item]
                    [/et_pb_accordion]
                [/et_pb_column]
            [/et_pb_row]
        [/et_pb_section]
    "##;

    let meta = HashMap::new();
    let layout = transpile_content(divi_content, &meta).expect("Divi transpilation failed");

    assert_eq!(layout.version, 1);
    assert_eq!(layout.root.children.len(), 1);

    let section = &layout.root.children[0];
    assert_eq!(section.kind, NodeKind::Section);
    assert!(section.id.starts_with("divi_section_"));

    let row = &section.children[0];
    assert_eq!(row.kind, NodeKind::Container);
    assert_eq!(row.children.len(), 2);

    // Verify unclosed [et_pb_divider] did NOT swallow [et_pb_text] or [et_pb_counter]
    let left_col = &row.children[0];
    assert_eq!(left_col.children.len(), 3);
    assert!(matches!(
        left_col.children[0].kind,
        NodeKind::Divider { .. }
    ));
    assert!(matches!(left_col.children[1].kind, NodeKind::Text { .. }));
    match &left_col.children[2].kind {
        NodeKind::ProgressBar { title, percent, .. } => {
            assert_eq!(title, "Rust Speed");
            assert_eq!(*percent, 92);
        }
        _ => panic!("Expected ProgressBar for et_pb_counter"),
    }

    let right_col = &row.children[1];
    assert_eq!(right_col.children.len(), 2);
    match &right_col.children[0].kind {
        NodeKind::Button { text, url, .. } => {
            assert_eq!(text, "Learn More");
            assert_eq!(url, "/features");
        }
        _ => panic!("Expected Button module"),
    }
    match &right_col.children[1].kind {
        NodeKind::Accordion { items } => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].title, "FAQ 1");
            assert_eq!(items[0].content_html, "Answer 1");
        }
        _ => panic!("Expected Accordion module"),
    }

    // Render to HTML
    let html = render_layout_to_html(&layout);
    assert!(html.contains("Left Column Content"));
    assert!(html.contains("<a href=\"/features\" target=\"_self\""));
    assert!(html.contains("FAQ 1"));
}

#[test]
fn test_divi5_gutenberg_block_comments() {
    let divi5_markup = r#"
<!-- wp:divi/section {"module":{"props":{"attrs":{"module_id":"hero-sec","custom_padding":"40px|20px|40px|20px"}}}} -->
<!-- wp:divi/row -->
<!-- wp:divi/column {"type":"1_2"} -->
<!-- wp:divi/text --><h2>Hello from Divi 5</h2><!-- /wp:divi/text -->
<!-- /wp:divi/column -->
<!-- /wp:divi/row -->
<!-- /wp:divi/section -->
"#;

    let layout = transpile_divi_content(divi5_markup).expect("Divi 5 transpilation failed");
    assert_eq!(layout.version, 1);
    let sec = &layout.root.children[0];
    assert_eq!(sec.id, "hero-sec");
    assert_eq!(
        sec.styles.padding.as_ref().and_then(|p| p.top.as_deref()),
        Some("40px")
    );
    assert_eq!(
        sec.styles.padding.as_ref().and_then(|p| p.right.as_deref()),
        Some("20px")
    );

    let row = &sec.children[0];
    let col = &row.children[0];
    match &col.kind {
        NodeKind::Flex { basis, .. } => {
            assert_eq!(basis.as_deref(), Some("50%"));
        }
        _ => panic!("Expected Flex column with 50% basis"),
    }

    let text_node = &col.children[0];
    match &text_node.kind {
        NodeKind::Text { content_html } => {
            assert!(content_html.contains("Hello from Divi 5"));
        }
        _ => panic!("Expected Text node"),
    }
}

#[test]
fn test_divi_font_composite_and_countdown() {
    let divi_shortcode = r#"
[et_pb_section]
  [et_pb_row]
    [et_pb_column type="4_4"]
      [et_pb_text header_font="Montserrat|700||on|||||" header_font_size="32px"]
        <h1>LANCEERDATUM</h1>
      [/et_pb_text]
      [et_pb_countdown_timer title="Tot de opening" date_time="2026-12-31 23:59:59" /]
    [/et_pb_column]
  [/et_pb_row]
[/et_pb_section]
"#;

    let layout =
        transpile_divi_content(divi_shortcode).expect("Divi shortcode transpilation failed");
    let sec = &layout.root.children[0];
    let row = &sec.children[0];
    let col = &row.children[0];

    let text_node = &col.children[0];
    assert_eq!(
        text_node.styles.font_family.as_deref(),
        Some("'Montserrat', sans-serif")
    );
    assert_eq!(text_node.styles.font_weight.as_deref(), Some("700"));
    assert_eq!(
        text_node.styles.text_transform.as_deref(),
        Some("uppercase")
    );

    let timer_node = &col.children[1];
    match &timer_node.kind {
        NodeKind::Countdown {
            target_date,
            show_days,
            ..
        } => {
            assert_eq!(target_date, "2026-12-31 23:59:59");
            assert!(*show_days);
        }
        _ => panic!("Expected Countdown node"),
    }
}

#[test]
fn test_divi_slider_slide_and_signup() {
    let divi_markup = r#"
[et_pb_section]
  [et_pb_row]
    [et_pb_column type="4_4"]
      [et_pb_slider]
        [et_pb_slide heading="Hero Slide 1" button_text="Get Started" button_link="/start" image="https://example.com/slide.png"]
          Welcome to high performance Daypress.
        [/et_pb_slide]
      [/et_pb_slider]
      [et_pb_signup title="Join the Newsletter" button_text="Subscribe Now" provider="mailchimp" /]
    [/et_pb_column]
  [/et_pb_row]
[/et_pb_section]
"#;

    let layout =
        transpile_divi_content(divi_markup).expect("Divi slider/signup transpilation failed");
    let sec = &layout.root.children[0];
    let row = &sec.children[0];
    let col = &row.children[0];

    // Check slider container and slide child
    let slider = &col.children[0];
    assert_eq!(slider.kind, NodeKind::Container);
    assert_eq!(slider.children.len(), 1);

    match &slider.children[0].kind {
        NodeKind::CallToAction {
            title,
            button_text,
            button_url,
            image_url,
            ..
        } => {
            assert_eq!(title, "Hero Slide 1");
            assert_eq!(button_text, "Get Started");
            assert_eq!(button_url, "/start");
            assert_eq!(image_url.as_deref(), Some("https://example.com/slide.png"));
        }
        _ => panic!("Expected CallToAction node for et_pb_slide"),
    }

    // Check signup form
    let signup = &col.children[1];
    match &signup.kind {
        NodeKind::NativeForm {
            title,
            button_text,
            endpoint,
        } => {
            assert_eq!(title, "Join the Newsletter");
            assert_eq!(button_text, "Subscribe Now");
            assert_eq!(endpoint, "mailchimp");
        }
        _ => panic!("Expected NativeForm node for et_pb_signup"),
    }

    let html = render_layout_to_html(&layout);
    assert!(html.contains("Hero Slide 1"));
    assert!(html.contains("Get Started"));
    assert!(html.contains("Join the Newsletter"));
    assert!(html.contains("Subscribe Now"));
}
