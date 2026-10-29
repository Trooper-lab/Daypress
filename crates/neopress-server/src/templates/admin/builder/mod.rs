mod css_tokens;
pub mod markup;
pub mod scoping;
pub mod scripts;
pub mod site_preview;
pub mod styles;

use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;
use crate::theme_builder::models::ResolvedThemePartials;

/// Synthesizes a 100% native `NeopressLayout` AST (zero `CustomHtml` fallbacks)
/// from arbitrary post HTML + `<style>` blocks using `neopress_ast::transpile_html_to_layout`.
pub fn synthesize_ast_from_post_html(_post_id: i64, html: &str) -> Option<serde_json::Value> {
    let trimmed = html.trim();
    if trimmed.is_empty() {
        return None;
    }
    let layout = neopress_ast::transpile_html_to_layout(trimmed);
    if layout.root.children.is_empty() {
        return None;
    }
    serde_json::to_value(&layout).ok()
}

/// Renders NeoStudio for `post` with the default site chrome (no Theme Builder partials).
pub fn render_wp_builder_view(
    settings: &WpSettingsResponse,
    post: &WpPostResponse,
    existing_layout_json: Option<&str>,
) -> String {
    render_wp_builder_view_with_partials(settings, post, existing_layout_json, None)
}

/// Renders NeoStudio for `post`, framing the artboard with the header, footer and theme CSS the
/// public page resolves for it (Theme Builder `partials` included).
pub fn render_wp_builder_view_with_partials(
    settings: &WpSettingsResponse,
    post: &WpPostResponse,
    existing_layout_json: Option<&str>,
    partials: Option<&ResolvedThemePartials>,
) -> String {
    let post_id = post.id;
    let post_title = escape_html(&post.title.rendered);
    let post_slug = escape_html(&post.slug);
    let post_type = escape_html(&post.post_type);
    let site_title = escape_html(&settings.title);

    let layout_json_raw = if let Some(layout) = existing_layout_json {
        if !layout.trim().is_empty() {
            layout
                .trim()
                .replace("</script", "<\\/script")
                .replace("</SCRIPT", "<\\/SCRIPT")
        } else if let Some(synth) = synthesize_ast_from_post_html(post_id, &post.content.rendered) {
            synth
                .to_string()
                .replace("</script", "<\\/script")
                .replace("</SCRIPT", "<\\/SCRIPT")
        } else {
            "null".to_string()
        }
    } else if let Some(synth) = synthesize_ast_from_post_html(post_id, &post.content.rendered) {
        synth
            .to_string()
            .replace("</script", "<\\/script")
            .replace("</SCRIPT", "<\\/SCRIPT")
    } else {
        "null".to_string()
    };

    let site = site_preview::build_site_preview(settings, post, &layout_json_raw, partials);
    let extra_head_assets = &settings.theme_assets_html;

    markup::get_markup(
        post_id,
        &post_title,
        &post_slug,
        &post_type,
        &site_title,
        &site.header_html,
        &site.footer_html,
        &site.scoped_css,
        extra_head_assets,
        &layout_json_raw,
    )
}
