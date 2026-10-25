pub mod markup;
pub mod scoping;
pub mod scripts;
pub mod styles;

use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

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

pub fn render_wp_builder_view(
    settings: &WpSettingsResponse,
    post: &WpPostResponse,
    existing_layout_json: Option<&str>,
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

    let is_blank_template = post.template == "page-template-blank.php"
        || post.template == "canvas"
        || post.template == "elementor_canvas"
        || post.template == "leeg.php";
    let is_editing_header = post.post_type == "template:header";
    let is_editing_footer = post.post_type == "template:footer";

    let preview_header_html = if is_blank_template || is_editing_header || is_editing_footer {
        String::new()
    } else if !settings.elementor_header_html.trim().is_empty() {
        format!(
            r#"<header class="np-elementor-header">{}</header>"#,
            settings.elementor_header_html
        )
    } else {
        let logo_markup = if !settings.site_logo_url.is_empty() {
            format!(
                r#"<img src="{}" alt="{}" style="height: 28px;" />"#,
                escape_html(&settings.site_logo_url),
                site_title
            )
        } else {
            format!(
                r#"<i class="fa-solid fa-bolt" style="color: #ff4f00;"></i> {}"#,
                site_title
            )
        };
        let preview_nav_html = if settings.nav_menu_items.is_empty() {
            r#"<a href="/">Home</a><a href="/n-admin/">Dashboard</a>"#.to_string()
        } else {
            let mut links = Vec::with_capacity(settings.nav_menu_items.len() + 1);
            for item in &settings.nav_menu_items {
                links.push(format!(
                    r#"<a href="{}">{}</a>"#,
                    escape_html(&item.url),
                    escape_html(&item.title)
                ));
            }
            links.push(r#"<a href="/n-admin/">Dashboard</a>"#.to_string());
            links.join("\n                    ")
        };
        format!(
            r#"<header class="preview-site-header">
                <a href="/" class="preview-site-brand">
                    {logo_markup}
                </a>
                <nav class="preview-site-nav">
                    {preview_nav_html}
                </nav>
            </header>"#,
            logo_markup = logo_markup,
            preview_nav_html = preview_nav_html
        )
    };

    let preview_footer_html = if is_blank_template || is_editing_header || is_editing_footer {
        String::new()
    } else if !settings.elementor_footer_html.trim().is_empty() {
        format!(
            r#"<footer class="np-elementor-footer">{}</footer>"#,
            settings.elementor_footer_html
        )
    } else {
        format!(
            r#"<footer class="preview-site-footer">
                <p>&copy; 2026 {}. Built in Native Rust with Zero JavaScript Overhead.</p>
            </footer>"#,
            site_title
        )
    };

    let mut extra_head_css = String::new();
    if !settings.elementor_kit_css.trim().is_empty() {
        extra_head_css.push_str(&settings.elementor_kit_css);
        extra_head_css.push('\n');
    }
    if !settings.custom_css.trim().is_empty() {
        extra_head_css.push_str(&settings.custom_css);
        extra_head_css.push('\n');
    }
    let extra_head_assets = &settings.theme_assets_html;
    let scoped_head_css = scoping::scope_theme_css(&extra_head_css);

    markup::get_markup(
        post_id,
        &post_title,
        &post_slug,
        &post_type,
        &site_title,
        &preview_header_html,
        &preview_footer_html,
        &scoped_head_css,
        extra_head_assets,
        &layout_json_raw,
    )
}
