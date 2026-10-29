//! Public site chrome around the NeoStudio artboard (`templates/admin/builder/site_preview.rs`).
//!
//! The canvas shows the same header, footer and theme stylesheet the public page renders with
//! (`templates::public::site_chrome` + `render_public_theme_css`), scoped to the artboard so the
//! editor sees an exact copy of the live page without the site CSS touching the studio chrome.

use super::scoping::scope_site_css;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::public::site_chrome;
use crate::templates::utils::escape_html;
use crate::theme_builder::models::ResolvedThemePartials;
use neopress_ast::NeopressLayout;

/// Site header/footer markup and scoped site CSS for one builder session.
pub struct SitePreview {
    pub header_html: String,
    pub footer_html: String,
    pub scoped_css: String,
}

/// Templates that render the page without the global site header and footer.
fn hides_site_chrome(post: &WpPostResponse) -> bool {
    post.template == "page-template-blank.php"
        || post.template == "canvas"
        || post.template == "elementor_canvas"
        || post.template == "leeg.php"
        || post.post_type == "template:header"
        || post.post_type == "template:footer"
}

pub fn build_site_preview(
    settings: &WpSettingsResponse,
    post: &WpPostResponse,
    layout_json: &str,
    partials: Option<&ResolvedThemePartials>,
) -> SitePreview {
    let hidden = hides_site_chrome(post);
    // Visitors see the logged-out header ("Log In"), so the canvas previews that one.
    let header_html = if hidden {
        String::new()
    } else {
        site_chrome::render_site_header(settings, None, partials, false)
    };
    let footer_html = if hidden {
        String::new()
    } else {
        site_chrome::render_site_footer(settings, partials, false)
    };

    let is_blank = site_chrome::is_blank_template(Some(post));
    let theme_probe = format!("{}{}", post.content.rendered, layout_json);
    let html_class = site_chrome::html_theme_class(settings, &theme_probe, is_blank);

    SitePreview {
        header_html,
        footer_html,
        scoped_css: scope_site_css(&site_css(settings, layout_json, partials), Some(html_class)),
    }
}

/// The stylesheets of the public page in the order the public shell emits them.
fn site_css(
    settings: &WpSettingsResponse,
    layout_json: &str,
    partials: Option<&ResolvedThemePartials>,
) -> String {
    let layout = serde_json::from_str::<NeopressLayout>(layout_json)
        .ok()
        .filter(|l| !l.root.children.is_empty());
    let brand = escape_html(site_chrome::brand_color(settings));
    let parts = [
        crate::templates::public::render_public_theme_css(&brand),
        settings.elementor_kit_css.clone(),
        site_chrome::typography_css(settings).to_string(),
        settings.custom_css.clone(),
        partials
            .and_then(|p| p.header_css.clone())
            .unwrap_or_default(),
        partials
            .and_then(|p| p.footer_css.clone())
            .unwrap_or_default(),
        site_chrome::page_kit_css(settings, layout.as_ref()),
    ];
    let mut css = String::new();
    for part in parts.iter().filter(|p| !p.trim().is_empty()) {
        css.push_str(part);
        css.push('\n');
    }
    css
}
