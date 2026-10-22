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

    markup::get_markup(
        post_id,
        &post_title,
        &post_slug,
        &post_type,
        &site_title,
        &preview_nav_html,
        layout_json_raw,
    )
}
