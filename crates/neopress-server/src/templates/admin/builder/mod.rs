pub mod styles;
pub mod markup;
pub mod scripts;

use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

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
            layout.trim()
        } else {
            "null"
        }
    } else {
        "null"
    };

    let is_blank_template = post.template == "page-template-blank.php"
        || post.template == "canvas"
        || post.template == "elementor_canvas";

    let preview_header_html = if is_blank_template {
        String::new()
    } else if !settings.elementor_header_html.trim().is_empty() {
        format!(
            r#"<header class="np-elementor-header">{}</header>"#,
            settings.elementor_header_html
        )
    } else {
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
                    <i class="fa-solid fa-bolt" style="color: #ff4f00;"></i> {site_title}
                </a>
                <nav class="preview-site-nav">
                    {preview_nav_html}
                </nav>
            </header>"#,
            site_title = site_title,
            preview_nav_html = preview_nav_html
        )
    };

    let preview_footer_html = if is_blank_template {
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

    markup::get_markup(
        post_id,
        &post_title,
        &post_slug,
        &post_type,
        &site_title,
        &preview_header_html,
        &preview_footer_html,
        &extra_head_css,
        layout_json_raw,
    )
}
