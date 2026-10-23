use super::models::{ButtonStyle, ElementorKit, GlobalColor};

#[test]
fn test_elementor_kit_defaults() {
    let kit = ElementorKit::default();
    assert_eq!(kit.system_colors.len(), 4);
    assert_eq!(kit.system_colors[0].color, "#6EC1E4");
    assert_eq!(kit.system_typography.len(), 4);
    assert_eq!(kit.content_width, 1140);
    assert_eq!(kit.widgets_space, 20);

    let css = kit.compile_to_css();
    assert!(css.contains("--e-global-color-primary: #6EC1E4;"));
    assert!(css.contains("--e-global-color-accent: #61CE70;"));
    assert!(css.contains("--e-content-width: 1140px;"));
    assert!(css.contains("--e-global-typography-primary-font-family: Roboto, sans-serif;"));
}

#[test]
fn test_elementor_kit_custom_colors_and_css() {
    let mut kit = ElementorKit::default();
    kit.custom_colors.push(GlobalColor {
        id: "brand-purple".to_string(),
        title: "Brand Purple".to_string(),
        color: "#8b5cf6".to_string(),
    });
    kit.custom_css = Some(".custom-banner { padding: 40px; }".to_string());

    let css = kit.compile_to_css();
    assert!(css.contains("--e-global-color-brand-purple: #8b5cf6;"));
    assert!(css.contains(".custom-banner { padding: 40px; }"));
}

#[test]
fn test_elementor_kit_button_style() {
    let kit = ElementorKit {
        button_style: Some(ButtonStyle {
            background_color: Some("#ff4f00".to_string()),
            text_color: Some("#ffffff".to_string()),
            border_radius: Some("8px".to_string()),
        }),
        ..Default::default()
    };

    let css = kit.compile_to_css();
    assert!(css.contains("--e-button-bg: #ff4f00;"));
    assert!(css.contains("--e-button-color: #ffffff;"));
    assert!(css.contains("--e-button-border-radius: 8px;"));
}

#[test]
fn test_php_serialized_elementor_kit_parsing() {
    let php_serialized = r##"a:19:{s:21:"space_between_widgets";a:3:{s:4:"unit";s:2:"px";s:4:"size";s:0:"";s:5:"sizes";a:0:{}}s:13:"system_colors";a:4:{i:0;a:3:{s:3:"_id";s:7:"primary";s:5:"title";s:7:"Primair";s:5:"color";s:7:"#6A6351";}i:1;a:3:{s:3:"_id";s:9:"secondary";s:5:"title";s:9:"Secundair";s:5:"color";s:7:"#3A3633";}i:2;a:3:{s:3:"_id";s:4:"text";s:5:"title";s:5:"Tekst";s:5:"color";s:7:"#1F1C17";}i:3;a:3:{s:3:"_id";s:6:"accent";s:5:"title";s:6:"Accent";s:5:"color";s:7:"#B8A98A";}}s:13:"custom_colors";a:4:{i:0;a:3:{s:3:"_id";s:7:"2171565";s:5:"title";s:14:"Saved Color #5";s:5:"color";s:7:"#4054B2";}i:1;a:3:{s:3:"_id";s:8:"581fd4a7";s:5:"title";s:14:"Saved Color #6";s:5:"color";s:7:"#23A455";}i:2;a:3:{s:3:"_id";s:8:"4b94e36e";s:5:"title";s:14:"Saved Color #7";s:5:"color";s:4:"#000";}i:3;a:3:{s:3:"_id";s:8:"498af94a";s:5:"title";s:14:"Saved Color #8";s:5:"color";s:4:"#FFF";}}s:17:"system_typography";a:4:{i:0;a:5:{s:3:"_id";s:7:"primary";s:5:"title";s:17:"Primaire kopregel";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:9:"Fira Sans";s:22:"typography_font_weight";s:3:"600";}i:1;a:5:{s:3:"_id";s:9:"secondary";s:5:"title";s:19:"Secundaire kopregel";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:9:"Fira Sans";s:22:"typography_font_weight";s:3:"400";}i:2;a:5:{s:3:"_id";s:4:"text";s:5:"title";s:10:"Body-tekst";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:6:"Roboto";s:22:"typography_font_weight";s:3:"400";}i:3;a:5:{s:3:"_id";s:6:"accent";s:5:"title";s:12:"Accent-tekst";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:6:"Roboto";s:22:"typography_font_weight";s:3:"500";}}s:9:"site_name";s:18:"Acme Flooring Demo";s:16:"site_description";s:36:"Thuiskomen op de vloer van je dromen";s:17:"custom_typography";a:0:{}s:21:"default_generic_fonts";s:10:"Sans-serif";s:19:"page_title_selector";s:14:"h1.entry-title";s:27:"hello_footer_copyright_text";s:25:"Alle rechten voorbehouden";s:15:"activeItemIndex";i:1;s:11:"viewport_md";i:768;s:11:"viewport_lg";i:1025;s:25:"hello_header_logo_display";s:0:"";s:28:"hello_header_tagline_display";s:0:"";s:44:"settings_page_transitions_entrance_animation";s:8:"fade-out";s:40:"settings_page_transitions_preloader_type";s:9:"animation";s:50:"settings_page_transitions_preloader_animation_type";s:13:"circle-dashed";s:11:"__globals__";a:1:{s:42:"settings_page_transitions_background_color";s:27:"globals/colors?id=secondary";}}"##;

    let kit = ElementorKit::from_raw_settings(php_serialized).expect("Should parse PHP serialized ElementorKit");
    assert_eq!(kit.system_colors.len(), 4);
    assert_eq!(kit.system_colors[0].id, "primary");
    assert_eq!(kit.system_colors[0].color, "#6A6351");
    assert_eq!(kit.system_colors[1].color, "#3A3633");
    assert_eq!(kit.system_colors[2].color, "#1F1C17");
    assert_eq!(kit.system_colors[3].color, "#B8A98A");
    assert_eq!(kit.custom_colors.len(), 4);
    assert_eq!(kit.custom_colors[3].id, "498af94a");
    assert_eq!(kit.custom_colors[3].color, "#FFF");
    assert_eq!(kit.system_typography[0].font_family, "Fira Sans");
    assert_eq!(kit.system_typography[2].font_family, "Roboto");

    let css = kit.compile_to_css();
    assert!(css.contains("--e-global-color-primary: #6A6351;"));
    assert!(css.contains("--e-global-color-498af94a: #FFF;"));
    assert!(css.contains("--e-global-typography-primary-font-family: Fira Sans;"));
}

#[test]
fn test_elementor_kit_container_width_alias() {
    let json = r#"{"container_width":{"size":1200,"unit":"px"}}"#;
    let kit: ElementorKit = serde_json::from_str(json).expect("deserialize failed");
    assert_eq!(kit.content_width, 1200);

    let css = kit.compile_to_css();
    assert!(css.contains("--e-content-width: 1200px;"));
    assert!(css.contains("--content-width: 1200px;"));
    assert!(css.contains("--container-max-width: 1200px;"));
}

