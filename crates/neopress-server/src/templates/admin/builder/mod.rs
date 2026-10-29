mod css_tokens;
pub mod markup;
pub mod scoping;
pub mod scripts;
pub mod site_preview;
pub mod styles;

use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;
use crate::theme_builder::models::ResolvedThemePartials;

pub fn synthesize_ast_from_post_html(post_id: i64, html: &str) -> Option<serde_json::Value> {
    let trimmed = html.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut rest = trimmed;
    while !rest.is_empty() {
        if let Some(sec_start) = rest.find("<section") {
            let before = rest[..sec_start].trim();
            if !before.is_empty() {
                chunks.push(before.to_string());
            }
            let from_sec = &rest[sec_start..];
            if let Some(sec_end_rel) = from_sec.find("</section>") {
                let end_idx = sec_end_rel + "</section>".len();
                let sec_html = from_sec[..end_idx].trim();
                if !sec_html.is_empty() {
                    chunks.push(sec_html.to_string());
                }
                rest = from_sec[end_idx..].trim_start();
            } else {
                chunks.push(from_sec.trim().to_string());
                break;
            }
        } else {
            let rem = rest.trim();
            if !rem.is_empty() {
                chunks.push(rem.to_string());
            }
            break;
        }
    }
    if chunks.is_empty() {
        return None;
    }
    let children: Vec<serde_json::Value> = chunks
        .into_iter()
        .enumerate()
        .map(|(idx, raw)| {
            serde_json::json!({
                "id": format!("acf_sec_{}_{}", post_id, idx),
                "type": "CustomHtml",
                "data": { "raw": raw },
                "styles": {}
            })
        })
        .collect();
    Some(serde_json::json!({
        "version": 1,
        "root": {
            "id": "root",
            "type": "Root",
            "styles": {},
            "children": children
        }
    }))
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
