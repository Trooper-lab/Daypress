use crate::daystudio::{transpile_daystudio_json_with_context, DaystudioContext};
use crate::divi::{transpile_divi_content_with_context, DiviContext};
use crate::gutenberg::transpile_gutenberg_content_with_context;
use crate::kit::DaystudioKit;
use crate::schema::{DaypressLayout, DaypressNode, NodeKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Analyzes post content and postmeta to automatically detect legacy page builder formats
/// (Elementor JSON, Divi shortcodes, or Gutenberg block comments) and transpiles them into
/// a canonical DaypressLayout AST.
pub fn transpile_content(content: &str, meta: &HashMap<String, String>) -> Option<DaypressLayout> {
    // 1. Check for Elementor payload in postmeta
    if let Some(elem_json) = meta.get("_elementor_data") {
        if !elem_json.trim().is_empty() {
            let ctx = DaystudioContext::from_meta(meta);
            if let Ok(mut layout) = transpile_daystudio_json_with_context(elem_json, &ctx) {
                if let Some(settings_raw) = meta
                    .get("_elementor_kit_settings")
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| {
                        meta.get("_elementor_page_settings")
                            .filter(|s| !s.trim().is_empty())
                    })
                {
                    if let Some(kit) = DaystudioKit::from_raw_settings(settings_raw) {
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

    // 2. Check for Divi shortcode markup or Divi 5 block comments in content
    if content.contains("<!-- wp:divi/")
        || content.contains("[et_pb_")
        || content.contains("[spb_kw_calculator_module")
        || content.contains("[spb_kitwerk_calculator")
        || content.contains("[dvppl_cf7_styler")
    {
        let ctx = DiviContext::from_meta(meta);
        if let Ok(mut layout) = transpile_divi_content_with_context(content, &ctx) {
            if let Some(settings_raw) = meta
                .get("_elementor_kit_settings")
                .filter(|s| !s.trim().is_empty())
            {
                if let Some(kit) = DaystudioKit::from_raw_settings(settings_raw) {
                    layout.global_styles = serde_json::to_value(kit).ok();
                } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(settings_raw) {
                    if val.is_object() {
                        layout.global_styles = Some(val);
                    }
                }
            }
            return Some(layout);
        }
    }

    // 3. Check for Gutenberg block comments in content
    if content.contains("<!-- wp:") {
        let src_url = meta.get("__dp_source_site_url").map(|s| s.as_str());
        if let Ok(layout) = transpile_gutenberg_content_with_context(content, src_url) {
            return Some(layout);
        }
    }

    None
}

/// Per-post AST transpilation fidelity report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AstFidelityReport {
    pub builder_detected: String,
    pub total_nodes: usize,
    pub native_nodes: usize,
    pub fallback_html_nodes: usize,
    pub unresolved_shortcodes: Vec<String>,
    pub fidelity_score_pct: f32,
}

/// Detects the primary authoring builder for a WordPress post/page from content and postmeta.
pub fn detect_builder(content: &str, meta: &HashMap<String, String>) -> &'static str {
    if meta
        .get("_elementor_data")
        .is_some_and(|s| !s.trim().is_empty() && s.trim() != "[]")
    {
        return "elementor";
    }
    if content.contains("<!-- wp:divi/")
        || content.contains("[et_pb_")
        || content.contains("[spb_kw_calculator_module")
        || content.contains("[spb_kitwerk_calculator")
        || content.contains("[dvppl_cf7_styler")
    {
        return "divi";
    }
    if content.contains("<!-- wp:") {
        return "gutenberg";
    }
    "classic"
}

fn push_unique(list: &mut Vec<String>, item: String) {
    if !item.is_empty() && !list.contains(&item) {
        list.push(item);
    }
}

fn walk_node_fidelity(node: &DaypressNode, report: &mut AstFidelityReport) {
    report.total_nodes += 1;
    match &node.kind {
        NodeKind::ShortcodeFallback { tag, content, .. } => {
            report.fallback_html_nodes += 1;
            if tag == "shortcode" && content.starts_with('[') {
                let name: String = content
                    .trim_start_matches('[')
                    .chars()
                    .take_while(|c| !c.is_whitespace() && *c != ']' && *c != '/')
                    .collect();
                push_unique(
                    &mut report.unresolved_shortcodes,
                    if name.is_empty() { tag.clone() } else { name },
                );
            } else {
                push_unique(&mut report.unresolved_shortcodes, tag.clone());
            }
        }
        NodeKind::CustomHtml { raw } => {
            if let Some(rest) = raw.strip_prefix("<!-- Elementor widget '") {
                report.fallback_html_nodes += 1;
                let widget_name: String = rest.chars().take_while(|c| *c != '\'').collect();
                push_unique(
                    &mut report.unresolved_shortcodes,
                    format!("elementor:{}", widget_name),
                );
            } else if raw.contains("[et_pb_") {
                report.fallback_html_nodes += 1;
                push_unique(
                    &mut report.unresolved_shortcodes,
                    "et_pb_fallback".to_string(),
                );
            } else {
                report.native_nodes += 1;
            }
        }
        _ => {
            report.native_nodes += 1;
        }
    }

    for child in &node.children {
        walk_node_fidelity(child, report);
    }
}

/// Analyzes the AST conversion fidelity of a post or page.
/// If `precomputed_layout` is `None`, runs `transpile_content(content, meta)` on the fly.
pub fn analyze_post_fidelity(
    content: &str,
    meta: &HashMap<String, String>,
    precomputed_layout: Option<&DaypressLayout>,
) -> AstFidelityReport {
    let builder = detect_builder(content, meta);
    let mut report = AstFidelityReport {
        builder_detected: builder.to_string(),
        ..Default::default()
    };

    let owned_layout;
    let layout_ref = match precomputed_layout {
        Some(l) => Some(l),
        None => {
            owned_layout = transpile_content(content, meta);
            owned_layout.as_ref()
        }
    };

    if let Some(layout) = layout_ref {
        for node in &layout.root.children {
            walk_node_fidelity(node, &mut report);
        }
    }

    if report.total_nodes == 0 {
        report.fidelity_score_pct = if builder == "classic" { 100.0 } else { 0.0 };
    } else {
        report.fidelity_score_pct =
            ((report.native_nodes as f32 / report.total_nodes as f32) * 1000.0).round() / 10.0;
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elementor_fidelity_with_nested_tabs_and_unknown_widget() {
        let mut meta = HashMap::new();
        meta.insert(
            "_elementor_data".to_string(),
            r#"[
                {
                    "id": "sec1",
                    "elType": "container",
                    "settings": {},
                    "elements": [
                        {
                            "id": "w1",
                            "elType": "widget",
                            "widgetType": "nested-tabs",
                            "settings": {
                                "tabs": [
                                    { "tab_title": "Overview" },
                                    { "tab_title": "Specs" }
                                ]
                            },
                            "elements": [
                                {
                                    "id": "c1",
                                    "elType": "container",
                                    "settings": {},
                                    "elements": [
                                        {
                                            "id": "h1",
                                            "elType": "widget",
                                            "widgetType": "heading",
                                            "settings": { "title": "Overview Heading" },
                                            "elements": []
                                        }
                                    ]
                                },
                                {
                                    "id": "c2",
                                    "elType": "container",
                                    "settings": {},
                                    "elements": [
                                        {
                                            "id": "t1",
                                            "elType": "widget",
                                            "widgetType": "text-editor",
                                            "settings": { "editor": "<p>Specs body</p>" },
                                            "elements": []
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "id": "w2",
                            "elType": "widget",
                            "widgetType": "some-custom-3p-addon",
                            "settings": {},
                            "elements": []
                        }
                    ]
                }
            ]"#
            .to_string(),
        );

        let report = analyze_post_fidelity("", &meta, None);
        assert_eq!(report.builder_detected, "elementor");
        assert_eq!(report.total_nodes, 3); // container + nested-tabs + 3p widget
        assert_eq!(report.native_nodes, 2);
        assert_eq!(report.fallback_html_nodes, 1);
        assert_eq!(
            report.unresolved_shortcodes,
            vec!["elementor:some-custom-3p-addon".to_string()]
        );
    }

    #[test]
    fn test_divi_and_gutenberg_new_converters_score_100_percent() {
        let meta = HashMap::new();
        let divi_content = r#"[et_pb_section][et_pb_row][et_pb_column type="4_4"][et_pb_team_member name="Jane Doe" position="CTO" image_url="https://example.com/jane.jpg"]Bio here[/et_pb_team_member][et_pb_gallery gallery_ids="https://example.com/a.jpg,https://example.com/b.jpg"][/et_pb_gallery][et_pb_shop posts_number="4" columns_number="2"][/et_pb_shop][/et_pb_column][/et_pb_row][/et_pb_section]"#;
        let divi_report = analyze_post_fidelity(divi_content, &meta, None);
        assert_eq!(divi_report.builder_detected, "divi");
        assert_eq!(divi_report.fallback_html_nodes, 0);
        assert_eq!(divi_report.fidelity_score_pct, 100.0);

        let gb_content = r#"<!-- wp:details {"summary":"FAQ Question"} -->
<details class="wp-block-details"><summary>FAQ Question</summary><!-- wp:paragraph --><p>FAQ Answer</p><!-- /wp:paragraph --></details>
<!-- /wp:details -->
<!-- wp:search {"label":"Search","placeholder":"Find articles...","buttonText":"Go"} /-->
<!-- wp:social-links -->
<ul class="wp-block-social-links"><!-- wp:social-link {"url":"https://github.com","service":"github"} /--></ul>
<!-- /wp:social-links -->"#;
        let gb_report = analyze_post_fidelity(gb_content, &meta, None);
        assert_eq!(gb_report.builder_detected, "gutenberg");
        assert_eq!(gb_report.fallback_html_nodes, 0);
        assert_eq!(gb_report.fidelity_score_pct, 100.0);
    }
}
