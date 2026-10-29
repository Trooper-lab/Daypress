//! Markup submodules for DayStudio 2026 Visual Builder.

pub mod head;
pub mod modals;
pub mod shell;

use super::scripts::get_scripts;
use super::styles::get_styles;

pub fn get_markup(
    post_id: i64,
    post_title: &str,
    post_slug: &str,
    post_type: &str,
    site_title: &str,
    preview_header_html: &str,
    preview_footer_html: &str,
    extra_head_css: &str,
    extra_head_assets: &str,
    layout_json_raw: &str,
) -> String {
    let styles = get_styles();
    let scripts = get_scripts();

    let head_html = head::render_head();
    let shell_html = shell::render_shell();
    let modals_html = modals::render_modals();

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="en" class="daystudio-theme-active">
{head_html}
<body>
{shell_html}
{modals_html}
<script>{scripts}</script>
</body>
</html>"#
    );

    let rest_nonce: String = crate::auth::current_admin()
        .map(|a| a.nonce)
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    full_html
        .replace("__STYLES__", styles)
        .replace("__REST_NONCE__", &rest_nonce)
        .replace("__EXTRA_HEAD_CSS__", extra_head_css)
        .replace("__EXTRA_HEAD_ASSETS__", extra_head_assets)
        .replace("__POST_ID__", &post_id.to_string())
        .replace("__POST_TITLE__", post_title)
        .replace("__POST_SLUG__", post_slug)
        .replace("__POST_TYPE__", post_type)
        .replace("__SITE_TITLE__", site_title)
        .replace("__PREVIEW_HEADER_HTML__", preview_header_html)
        .replace("__PREVIEW_FOOTER_HTML__", preview_footer_html)
        .replace("__LAYOUT_JSON_RAW__", layout_json_raw)
}
