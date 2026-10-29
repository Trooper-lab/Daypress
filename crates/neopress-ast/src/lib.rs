pub mod divi;
pub mod elementor;
pub mod gutenberg;
pub mod html_transpiler;
pub mod kit;
pub mod render;
pub mod schema;
pub mod url_utils;

use std::collections::HashMap;

pub use divi::{
    render_spb_calculator_widget, transpile_divi_content, transpile_divi_content_with_context,
    DiviContext, SpbServiceItem,
};
pub use elementor::{
    normalize_video_embed_url, resolve_dynamic_tag, transpile_elementor_json,
    transpile_elementor_json_with_context, ElementorContext,
};
pub use gutenberg::theme_json::{self, compile_theme_json_to_css, parse_theme_json, ThemeJson};
pub use gutenberg::{transpile_gutenberg_content, transpile_gutenberg_content_with_context};
pub use html_transpiler::{
    transpile_html_document, transpile_html_to_layout, HtmlDesignImportOptions,
    HtmlDesignImportResult,
};
pub use kit::{
    parse_php_serialized_to_json, ButtonStyle, ElementorKit, GlobalColor, GlobalTypography,
};
pub use render::{
    compile_layout_css, render_layout_to_html, render_node_to_html, resolve_semantic_tag,
    sanitize_editor_html, sanitize_ssr_html, strip_editor_classes,
};
pub use schema::{
    AccordionItem, BackgroundConfig, BorderConfig, Breakpoint, ConditionScope, ConditionType,
    DisplayCondition, IconListItem, NavMenuItem, NeopressLayout, NeopressNode, NodeKind,
    PriceFeatureItem, PseudoState, SharePlatformItem, SocialIconItem, Spacing, StyleProperties,
    TabItem, TemplateType, TestimonialSlide, ThemeTemplate, ToggleItem, TypographyConfig,
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
                if let Some(kit) = ElementorKit::from_raw_settings(settings_raw) {
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
        let src_url = meta.get("__np_source_site_url").map(|s| s.as_str());
        if let Ok(layout) = transpile_gutenberg_content_with_context(content, src_url) {
            return Some(layout);
        }
    }

    None
}
