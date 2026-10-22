pub mod divi;
pub mod elementor;
pub mod gutenberg;
pub mod kit;
pub mod render;
pub mod schema;

use std::collections::HashMap;

pub use divi::{
    render_spb_calculator_widget, transpile_divi_content, transpile_divi_content_with_context,
    DiviContext, SpbServiceItem,
};
pub use elementor::{
    normalize_video_embed_url, resolve_dynamic_tag, transpile_elementor_json,
    transpile_elementor_json_with_context, ElementorContext, NavMenuItem,
};
pub use gutenberg::transpile_gutenberg_content;
pub use kit::{ parse_php_serialized_to_json, ElementorKit, GlobalColor, GlobalTypography };
pub use render::{render_layout_to_html, render_node_to_html};
pub use schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};

/// Analyzes post content and postmeta to automatically detect legacy page builder formats
/// (Elementor JSON, Divi shortcodes, or Gutenberg block comments) and transpiles them into
/// a canonical NeopressLayout AST.
pub fn transpile_content(content: &str, meta: &HashMap<String, String>) -> Option<NeopressLayout> {
    // 1. Check for Elementor payload in postmeta
    if let Some(elem_json) = meta.get("_elementor_data") {
        if !elem_json.trim().is_empty() {
            let ctx = ElementorContext::from_meta(meta);
            if let Ok(mut layout) = transpile_elementor_json_with_context(elem_json, &ctx) {
                // Wire ElementorKit global styles when _elementor_kit_settings or _elementor_page_settings is present
                if let Some(settings_raw) = meta
                    .get("_elementor_kit_settings")
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| {
                        meta.get("_elementor_page_settings")
                            .filter(|s| !s.trim().is_empty())
                    })
                {
                    if let Some(kit) = ElementorKit::from_raw_settings(settings_raw) {
                        layout.global_styles = serde_json::to_value(kit).ok();
                    } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(settings_raw)
                    {
                        if val.is_object() {
                            layout.global_styles = Some(val);
                        }
                    }
                }
                return Some(layout);
            }
        }
    }

    // 2. Check for Divi shortcode markup in content
    if content.contains("[et_pb_")
        || content.contains("[spb_kw_calculator_module")
        || content.contains("[spb_kitwerk_calculator")
        || content.contains("[dvppl_cf7_styler")
    {
        let ctx = DiviContext::from_meta(meta);
        if let Ok(layout) = transpile_divi_content_with_context(content, &ctx) {
            return Some(layout);
        }
    }

    // 3. Check for Gutenberg block comments in content
    if content.contains("<!-- wp:") {
        if let Ok(layout) = transpile_gutenberg_content(content) {
            return Some(layout);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elementor_transpilation_and_rendering() {
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
                                    "link": { "url": "https://neopress.org", "is_external": true }
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
        assert_eq!(
            section.styles.background_color.as_deref(),
            Some("#0f172a")
        );
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
                assert_eq!(url, "https://neopress.org");
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
        assert!(html.contains("<a href=\"https://neopress.org\" target=\"_blank\""));
    }

    #[test]
    fn test_elementor_container_globals_and_youtube() {
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
        assert_eq!(cont.styles.justify_content.as_deref(), Some("space-between"));
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
        assert!(matches!(left_col.children[0].kind, NodeKind::Divider { .. }));
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
    fn test_gutenberg_transpilation_and_rendering() {
        let gb_content = r##"
<!-- wp:cover {"url":"https://example.com/hero.jpg","customOverlayColor":"#0f172a"} -->
<div class="wp-block-cover">
    <!-- wp:heading {"level":1,"textAlign":"center"} -->
    <h1 class="wp-block-heading has-text-align-center">Gutenberg to Native Rust</h1>
    <!-- /wp:heading -->
    <!-- wp:paragraph -->
    <p>Zero-overhead block transpilation.</p>
    <!-- /wp:paragraph -->
    <!-- wp:buttons -->
    <div class="wp-block-buttons">
        <!-- wp:button -->
        <div class="wp-block-button"><a class="wp-block-button__link" href="/start">Start Now</a></div>
        <!-- /wp:button -->
    </div>
    <!-- /wp:buttons -->
</div>
<!-- /wp:cover -->
<!-- wp:spacer {"height":"60px"} /-->
        "##;

        let meta = HashMap::new();
        let layout = transpile_content(gb_content, &meta).expect("Gutenberg transpilation failed");
        assert_eq!(layout.root.children.len(), 2);

        let cover = &layout.root.children[0];
        assert_eq!(cover.kind, NodeKind::Section);
        assert_eq!(cover.children.len(), 3);

        let html = render_layout_to_html(&layout);
        assert!(html.contains("Gutenberg to Native Rust</h1>"));
        assert!(html.contains("Zero-overhead block transpilation."));
        assert!(html.contains("<a href=\"/start\" target=\"_self\" class=\"np-button\""));
        assert!(html.contains("height: 60px;"));
    }

    #[test]
    fn test_visual_builder_and_seed_schema_unification() {
        let vb_json = r##"{
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": [
                    {
                        "id": "sec_1",
                        "type": "Section",
                        "data": {},
                        "styles": {
                            "background_color": "#0f172a",
                            "padding_top": "80px",
                            "border": "1px solid #e2e8f0",
                            "margin": "0 auto"
                        },
                        "children": [
                            {
                                "id": "col_1",
                                "type": "Column",
                                "styles": { "flex": "1" },
                                "children": [
                                    {
                                        "id": "cnt_1",
                                        "type": "Counter",
                                        "data": { "number": "100%", "label": "Lighthouse Performance" },
                                        "styles": {}
                                    },
                                    {
                                        "id": "sp_1",
                                        "type": "Spacer",
                                        "styles": { "height": "40px" }
                                    },
                                    {
                                        "id": "frm_1",
                                        "type": "NativeForm",
                                        "data": { "title": "Contact Us", "button_text": "Submit" }
                                    },
                                    {
                                        "id": "bm_1",
                                        "type": "Benchmark",
                                        "data": { "text": "95,000 req/sec", "subtitle": "Tokio Engine" }
                                    },
                                    {
                                        "id": "ic_1",
                                        "type": "Icon",
                                        "data": { "icon": "fa-solid fa-rocket" }
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }"##;

        let layout: NeopressLayout =
            serde_json::from_str(vb_json).expect("Visual Builder JSON should deserialize cleanly");
        assert_eq!(layout.root.kind, NodeKind::Root);
        let sec = &layout.root.children[0];
        assert_eq!(sec.kind, NodeKind::Section);
        let col = &sec.children[0];
        assert_eq!(col.kind, NodeKind::Column);

        match &col.children[0].kind {
            NodeKind::Counter {
                number,
                suffix,
                title,
                ..
            } => {
                assert_eq!(*number, 100);
                assert_eq!(suffix.as_deref(), Some("%"));
                assert_eq!(title.as_deref(), Some("Lighthouse Performance"));
            }
            _ => panic!("Expected Counter"),
        }

        match &col.children[1].kind {
            NodeKind::Spacer { height } => assert_eq!(*height, 40),
            _ => panic!("Expected Spacer with 40px height"),
        }

        let html = render_layout_to_html(&layout);
        assert!(html.contains("class=\"np-column\""));
        assert!(html.contains("100%"));
        assert!(html.contains("Lighthouse Performance"));
        assert!(html.contains("class=\"np-form\""));
        assert!(html.contains("95,000 req/sec"));
        assert!(html.contains("<i class=\"fa-solid fa-rocket\""));
    }

    #[test]
    fn test_elementor_extended_widgets_transpilation_and_rendering() {
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
                                    "link": { "url": "https://neopress.org" }
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
                                    "testimonial_content": "Neopress loads pages in 0.2ms!",
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
                assert_eq!(link.as_deref(), Some("https://neopress.org"));
            }
            _ => panic!("Expected IconBox"),
        }

        // Check ProgressBar
        match &col.children[1].kind {
            NodeKind::ProgressBar {
                title,
                percent,
                display_text,
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
                assert_eq!(content, "Neopress loads pages in 0.2ms!");
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
        assert!(html.contains("np-iconbox"));
        assert!(html.contains("High Performance"));
        assert!(html.contains("np-progressbar"));
        assert!(html.contains("width: 95%;"));
        assert!(html.contains("np-testimonial"));
        assert!(html.contains("Satya Nadella"));
        assert!(html.contains("np-alert-success"));
        assert!(html.contains("np-rating"));
        assert!(html.contains("fa-solid fa-star"));
    }
}
