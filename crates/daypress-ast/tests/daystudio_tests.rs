use daypress_ast::*;
use std::collections::HashMap;

#[test]
fn test_daystudio_transpilation_and_rendering() {
    let elementor_json = r##"[
        {
            "id": "sec_1",
            "elType": "section",
            "settings": {
                "background_background": "classic",
                "background_color": "#0f172a",
                "padding": { "top": "60", "bottom": "60", "unit": "px" }
            },
            "elements": [
                {
                    "id": "col_1",
                    "elType": "column",
                    "settings": { "_column_size": 100 },
                    "elements": [
                        {
                            "id": "hdg_1",
                            "elType": "widget",
                            "widgetType": "heading",
                            "settings": {
                                "title": "Built with Native Rust",
                                "header_size": "h1",
                                "title_color": "#38bdf8"
                            }
                        },
                        {
                            "id": "btn_1",
                            "elType": "widget",
                            "widgetType": "button",
                            "settings": {
                                "text": "Get Started",
                                "link": { "url": "https://daypress.org", "is_external": true }
                            }
                        }
                    ]
                }
            ]
        }
    ]"##;

    let mut meta = HashMap::new();
    meta.insert("_elementor_data".to_string(), elementor_json.to_string());

    let layout = transpile_content("", &meta).expect("Transpilation failed");
    assert_eq!(layout.version, 1);
    assert_eq!(layout.root.children.len(), 1);

    let section = &layout.root.children[0];
    assert_eq!(section.kind, NodeKind::Section);
    assert_eq!(section.styles.background_color.as_deref(), Some("#0f172a"));
    assert_eq!(section.styles.padding_top.as_deref(), Some("60px"));
    assert_eq!(section.children.len(), 1);

    let col = &section.children[0];
    match &col.kind {
        NodeKind::Flex { basis, .. } => assert_eq!(basis.as_deref(), Some("100%")),
        _ => panic!("Expected Flex column"),
    }

    let heading = &col.children[0];
    match &heading.kind {
        NodeKind::Heading { text, level } => {
            assert_eq!(text, "Built with Native Rust");
            assert_eq!(*level, 1);
        }
        _ => panic!("Expected Heading widget"),
    }
    assert_eq!(heading.styles.text_color.as_deref(), Some("#38bdf8"));

    let button = &col.children[1];
    match &button.kind {
        NodeKind::Button { text, url, target } => {
            assert_eq!(text, "Get Started");
            assert_eq!(url, "https://daypress.org");
            assert_eq!(target, "_blank");
        }
        _ => panic!("Expected Button widget"),
    }

    // Render to HTML (ensure no redundant elem_root <section> wrapper)
    let html = render_layout_to_html(&layout);
    assert!(!html.contains("id=\"elem_root\""));
    assert!(html.contains("<section id=\"sec_1\""));
    assert!(html.contains("<h1 id=\"hdg_1\""));
    assert!(html.contains("Built with Native Rust</h1>"));
    assert!(html.contains("<a href=\"https://daypress.org\" target=\"_blank\""));
}

#[test]
fn test_daystudio_container_globals_and_youtube() {
    let elementor_json = r##"[
        {
            "id": "cont_flex_1",
            "elType": "container",
            "settings": {
                "flex_direction": "row",
                "flex_justify_content": "space-between",
                "flex_align_items": "center",
                "flex_gap": { "size": 24, "unit": "px" },
                "background_background": "classic",
                "__globals__": {
                    "background_color": "globals/colors?id=primary"
                }
            },
            "elements": [
                {
                    "id": "vid_1",
                    "elType": "widget",
                    "widgetType": "video",
                    "settings": {
                        "youtube_url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
                    }
                },
                {
                    "id": "cta_1",
                    "elType": "widget",
                    "widgetType": "call-to-action",
                    "settings": {
                        "title": "Migrate in 1 Click",
                        "description": "Zero downtime migration.",
                        "button": "Start Free",
                        "link": { "url": "/migrate" }
                    }
                }
            ]
        }
    ]"##;

    let mut meta = HashMap::new();
    meta.insert("_elementor_data".to_string(), elementor_json.to_string());
    meta.insert(
        "_elementor_page_settings".to_string(),
        r#"{"content_width": 1280}"#.to_string(),
    );

    let layout = transpile_content("", &meta).expect("Elementor 3.6+ transpilation failed");
    assert!(layout.global_styles.is_some());

    let cont = &layout.root.children[0];
    assert_eq!(cont.kind, NodeKind::Container);
    assert_eq!(cont.styles.display.as_deref(), Some("flex"));
    assert_eq!(
        cont.styles.justify_content.as_deref(),
        Some("space-between")
    );
    assert_eq!(cont.styles.gap.as_deref(), Some("24px"));
    assert_eq!(
        cont.styles.background_color.as_deref(),
        Some("var(--e-global-color-primary)")
    );

    match &cont.children[0].kind {
        NodeKind::Video { url, .. } => {
            assert_eq!(url, "https://www.youtube.com/embed/dQw4w9WgXcQ");
        }
        _ => panic!("Expected Video node"),
    }

    let html = render_layout_to_html(&layout);
    assert!(html.contains("--e-content-width: 1280px;"));
    assert!(html.contains("https://www.youtube.com/embed/dQw4w9WgXcQ"));
    assert!(html.contains("Migrate in 1 Click"));
}

#[test]
fn test_daystudio_extended_widgets_transpilation_and_rendering() {
    let elementor_json = r##"[
        {
            "id": "sec_ext",
            "elType": "section",
            "settings": { "background_color": "#ffffff" },
            "elements": [
                {
                    "id": "col_ext",
                    "elType": "column",
                    "elements": [
                        {
                            "id": "ib_1",
                            "elType": "widget",
                            "widgetType": "icon-box",
                            "settings": {
                                "title_text": "High Performance",
                                "description_text": "Built in pure native Rust with zero PHP memory leaks.",
                                "selected_icon": { "value": "fa-solid fa-bolt" },
                                "link": { "url": "https://daypress.org" }
                            }
                        },
                        {
                            "id": "prog_1",
                            "elType": "widget",
                            "widgetType": "progress",
                            "settings": {
                                "title": "Throughput",
                                "percent": { "size": 95 },
                                "display_percentage": "95%"
                            }
                        },
                        {
                            "id": "testim_1",
                            "elType": "widget",
                            "widgetType": "testimonial",
                            "settings": {
                                "testimonial_content": "Daypress loads pages in 0.2ms!",
                                "testimonial_name": "Satya Nadella",
                                "testimonial_job": "Tech Executive"
                            }
                        },
                        {
                            "id": "alrt_1",
                            "elType": "widget",
                            "widgetType": "alert",
                            "settings": {
                                "alert_type": "success",
                                "alert_title": "Database Synced",
                                "alert_description": "Embedded SQLite WAL write completed.",
                                "show_dismiss": "show"
                            }
                        },
                        {
                            "id": "rate_1",
                            "elType": "widget",
                            "widgetType": "rating",
                            "settings": {
                                "rating": 4.9,
                                "rating_scale": 5,
                                "title": "Lighthouse Score"
                            }
                        }
                    ]
                }
            ]
        }
    ]"##;

    let mut meta = HashMap::new();
    meta.insert("_elementor_data".to_string(), elementor_json.to_string());

    let layout = transpile_content("", &meta).expect("Transpilation failed");
    assert_eq!(layout.version, 1);
    let sec = &layout.root.children[0];
    let col = &sec.children[0];
    assert_eq!(col.children.len(), 5);

    // Check IconBox
    match &col.children[0].kind {
        NodeKind::IconBox {
            icon,
            title,
            description,
            link,
        } => {
            assert_eq!(icon, "fa-solid fa-bolt");
            assert_eq!(title, "High Performance");
            assert_eq!(
                description,
                "Built in pure native Rust with zero PHP memory leaks."
            );
            assert_eq!(link.as_deref(), Some("https://daypress.org"));
        }
        _ => panic!("Expected IconBox"),
    }

    // Check ProgressBar
    match &col.children[1].kind {
        NodeKind::ProgressBar {
            title,
            percent,
            display_text,
            ..
        } => {
            assert_eq!(title, "Throughput");
            assert_eq!(*percent, 95);
            assert_eq!(display_text.as_deref(), Some("95%"));
        }
        _ => panic!("Expected ProgressBar"),
    }

    // Check Testimonial
    match &col.children[2].kind {
        NodeKind::Testimonial {
            content, name, job, ..
        } => {
            assert_eq!(content, "Daypress loads pages in 0.2ms!");
            assert_eq!(name, "Satya Nadella");
            assert_eq!(job, "Tech Executive");
        }
        _ => panic!("Expected Testimonial"),
    }

    // Check Alert
    match &col.children[3].kind {
        NodeKind::Alert {
            alert_type,
            title,
            description,
            dismissible,
            ..
        } => {
            assert_eq!(alert_type, "success");
            assert_eq!(title, "Database Synced");
            assert_eq!(description, "Embedded SQLite WAL write completed.");
            assert!(*dismissible);
        }
        _ => panic!("Expected Alert"),
    }

    // Check Rating
    match &col.children[4].kind {
        NodeKind::Rating {
            rating,
            max_rating,
            title,
        } => {
            assert!((*rating - 4.9).abs() < 0.01);
            assert_eq!(*max_rating, 5);
            assert_eq!(title.as_deref(), Some("Lighthouse Score"));
        }
        _ => panic!("Expected Rating"),
    }

    // Test HTML SSR
    let html = render_layout_to_html(&layout);
    assert!(html.contains("dp-iconbox"));
    assert!(html.contains("High Performance"));
    assert!(html.contains("dp-progressbar"));
    assert!(html.contains("width: 95%;"));
    assert!(html.contains("dp-testimonial"));
    assert!(html.contains("Satya Nadella"));
    assert!(html.contains("dp-alert-success"));
    assert!(html.contains("dp-rating"));
    assert!(html.contains("fa-solid fa-star"));
}

#[test]
fn test_daystudio_css_grid_container() {
    let json = r#"[
        {
            "id": "grid_sec",
            "elType": "container",
            "settings": {
                "container_type": "grid",
                "grid_columns_grid": { "size": 3, "unit": "fr" },
                "grid_gaps": { "row": "24", "column": "24", "unit": "px" },
                "grid_auto_flow": "row",
                "grid_columns_grid_tablet": { "size": 2, "unit": "fr" },
                "grid_columns_grid_mobile": { "size": 1, "unit": "fr" }
            },
            "elements": [
                {
                    "id": "item_1",
                    "elType": "widget",
                    "widgetType": "heading",
                    "settings": {
                        "title": "Grid Item 1",
                        "_grid_column_span": 2
                    }
                }
            ]
        }
    ]"#;

    let mut meta = HashMap::new();
    meta.insert("_elementor_data".to_string(), json.to_string());

    let layout = transpile_content("", &meta).expect("Grid transpilation failed");
    let grid_node = &layout.root.children[0];
    assert_eq!(grid_node.styles.display.as_deref(), Some("grid"));
    assert_eq!(
        grid_node.styles.grid_template_columns.as_deref(),
        Some("repeat(3, 1fr)")
    );
    assert_eq!(grid_node.styles.grid_gap.as_deref(), Some("24px"));
    assert_eq!(grid_node.styles.grid_auto_flow.as_deref(), Some("row"));

    // Responsive overrides
    let tab = grid_node
        .styles
        .tablet
        .as_ref()
        .expect("Missing tablet styles");
    assert_eq!(tab.grid_template_columns.as_deref(), Some("repeat(2, 1fr)"));
    let mob = grid_node
        .styles
        .mobile
        .as_ref()
        .expect("Missing mobile styles");
    assert_eq!(mob.grid_template_columns.as_deref(), Some("repeat(1, 1fr)"));

    // Grid item column span
    let item = &grid_node.children[0];
    assert_eq!(item.styles.grid_column.as_deref(), Some("span 2"));
}
