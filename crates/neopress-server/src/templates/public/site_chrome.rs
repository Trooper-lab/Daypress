//! Global site header, footer and page-level theme CSS (`templates/public/site_chrome.rs`).
//!
//! Shared by the public SSR shell and the NeoStudio builder canvas, so the header, footer and
//! theme stylesheet an editor sees around the artboard are the ones visitors get.

use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;
use crate::theme_builder::models::ResolvedThemePartials;
use neopress_ast::{ElementorKit, NeopressLayout};

/// Site name shown in the header and footer brand.
pub fn site_name(settings: &WpSettingsResponse) -> &str {
    if settings.title.trim().is_empty() {
        "Neopress"
    } else {
        settings.title.trim()
    }
}

/// Site tagline, falling back to the site name.
pub fn site_description(settings: &WpSettingsResponse) -> &str {
    if settings.description.trim().is_empty() {
        site_name(settings)
    } else {
        settings.description.trim()
    }
}

/// Customizer brand color, falling back to the Neopress orange.
pub fn brand_color(settings: &WpSettingsResponse) -> &str {
    if settings.brand_color.trim().is_empty() {
        "#ff4f00"
    } else {
        settings.brand_color.trim()
    }
}

/// Whether the post uses a blank/canvas template that renders without site header and footer.
pub fn is_blank_template(post: Option<&WpPostResponse>) -> bool {
    post.map(|p| {
        p.template == "page-template-blank.php"
            || p.template == "canvas"
            || p.template == "elementor_canvas"
    })
    .unwrap_or(false)
}

/// Class the public shell puts on `<html>`: `elementor-theme-active` for imported Elementor/Divi
/// sites and blank templates, `dark` for the native Neopress theme.
pub fn html_theme_class(
    settings: &WpSettingsResponse,
    content: &str,
    is_blank_template: bool,
) -> &'static str {
    let has_elementor_theme = !settings.elementor_kit_css.trim().is_empty()
        || !settings.elementor_header_html.trim().is_empty()
        || !settings.elementor_footer_html.trim().is_empty()
        || is_blank_template
        || content.contains("divi_section_");
    if has_elementor_theme {
        "elementor-theme-active"
    } else {
        "dark"
    }
}

/// Customizer → Typography: CSS that swaps the site font stack (empty for the default stack).
pub fn typography_css(settings: &WpSettingsResponse) -> &'static str {
    match settings.typography_scale.as_str() {
        "serif" => ":root { --font-sans: \"Newsreader\", Georgia, \"Times New Roman\", serif; }",
        "mono" => ":root { --font-sans: \"JetBrains Mono\", ui-monospace, SFMono-Regular, Menlo, monospace; }",
        _ => "",
    }
}

/// Page-level Elementor kit CSS (global colors, fonts, heading/button defaults): the layout's own
/// `global_styles` kit when present, otherwise the default kit unless the site imported one.
pub fn page_kit_css(settings: &WpSettingsResponse, layout: Option<&NeopressLayout>) -> String {
    let fallback_kit_css = if !settings.elementor_kit_css.trim().is_empty() {
        String::new()
    } else {
        ElementorKit::default().compile_to_css()
    };
    match layout.and_then(|l| l.global_styles.as_ref()) {
        Some(gs) if !gs.is_null() => serde_json::from_value::<ElementorKit>(gs.clone())
            .map(|kit| kit.compile_to_css())
            .unwrap_or(fallback_kit_css),
        _ => fallback_kit_css,
    }
}

/// Resolves the global site header: Theme Builder template, imported Elementor header, or the
/// native Neopress header.
pub fn render_site_header(
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
    partials: Option<&ResolvedThemePartials>,
    is_blank_template: bool,
) -> String {
    let has_tb_header = partials
        .map(|p| p.has_theme_builder_header)
        .unwrap_or(false);
    let header_hidden = partials.map(|p| p.header_hidden).unwrap_or(false);
    let header_force_fallback = partials.map(|p| p.header_force_fallback).unwrap_or(false);
    if is_blank_template || header_hidden {
        return String::new();
    }
    if !header_force_fallback {
        if let Some(h_html) = partials.and_then(|p| p.header_html.as_ref()) {
            let sticky_cls = if partials.map(|p| p.header_sticky).unwrap_or(false) {
                " np-sticky"
            } else {
                ""
            };
            let trans_cls = if partials.map(|p| p.header_transparent).unwrap_or(false) {
                " np-transparent"
            } else {
                ""
            };
            return format!(
                r#"<header class="np-theme-header{}{}">{}</header>"#,
                sticky_cls, trans_cls, h_html
            );
        }
    }
    if !header_force_fallback && !has_tb_header && !settings.elementor_header_html.trim().is_empty()
    {
        return format!(
            r#"<header class="np-elementor-header">{}</header>"#,
            settings.elementor_header_html
        );
    }

    let header_login_btn = if current_user.is_some() {
        r##"<a href="/n-admin/" class="np-button np-btn-sm"><i class="fa-solid fa-gauge-high"></i> Dashboard</a>"##
    } else {
        r##"<a href="/login" class="btn-login">Log In</a>"##
    };
    let nav_links_html = if !settings.nav_menu_items.is_empty() {
        let tree = super::menus::build_nav_menu_tree(&settings.nav_menu_items);
        super::menus::render_hierarchical_nav_menu(&tree, false)
    } else {
        r#"<ul class="menu np-primary-menu"><li class="menu-item np-nav-item"><a href="/" class="nav-link">Home</a></li><li class="menu-item np-nav-item"><a href="/#posts" class="nav-link">Articles</a></li></ul>"#.to_string()
    };
    format!(
        r##"<header class="neo-header">
		<div class="header-container">
			<a href="/" class="logo-brand">
				<span class="badge-neo">CMS</span>
				<span>{brand_display}</span>
			</a>
			<nav class="neo-nav">
				{nav_links_html}
			</nav>
			<div class="header-actions">
				<form action="/" method="GET" class="header-search">
					<input type="text" name="s" placeholder="Search site..." aria-label="Search" />
				</form>
				{header_login_btn}
			</div>
		</div>
	</header>"##,
        brand_display = escape_html(site_name(settings)),
        nav_links_html = nav_links_html,
        header_login_btn = header_login_btn
    )
}

/// Resolves the global site footer: Theme Builder template, imported Elementor footer, or the
/// native Neopress footer.
pub fn render_site_footer(
    settings: &WpSettingsResponse,
    partials: Option<&ResolvedThemePartials>,
    is_blank_template: bool,
) -> String {
    let has_tb_footer = partials
        .map(|p| p.has_theme_builder_footer)
        .unwrap_or(false);
    let footer_hidden = partials.map(|p| p.footer_hidden).unwrap_or(false);
    let footer_force_fallback = partials.map(|p| p.footer_force_fallback).unwrap_or(false);
    if is_blank_template || footer_hidden {
        return String::new();
    }
    if !footer_force_fallback {
        if let Some(f_html) = partials.and_then(|p| p.footer_html.as_ref()) {
            return format!(r#"<footer class="np-theme-footer">{}</footer>"#, f_html);
        }
    }
    if !footer_force_fallback && !has_tb_footer && !settings.elementor_footer_html.trim().is_empty()
    {
        return format!(
            r#"<footer class="np-elementor-footer">{}</footer>"#,
            settings.elementor_footer_html
        );
    }

    let brand_display = escape_html(site_name(settings));
    let privacy_link_html = if settings.privacy_policy_page_id > 0 {
        format!(
            r#" &bull; <a href="/?page_id={}">Privacy Policy</a>"#,
            settings.privacy_policy_page_id
        )
    } else {
        String::new()
    };
    let footer_nav_html = if !settings.nav_menu_items.is_empty() {
        super::menus::render_footer_nav_links(&settings.nav_menu_items)
    } else {
        r#"<a href="/">Home</a><a href="/#posts">Articles</a>"#.to_string()
    };
    format!(
        r##"<footer class="neo-footer">
		<div class="footer-grid">
			<div class="footer-brand">
				<div class="logo-brand">
					<span class="badge-neo">CMS</span>
					<span>{brand_display}</span>
				</div>
				<p>{site_desc}</p>
			</div>
			<div>
				<div class="footer-col-title">Navigation</div>
				<div class="footer-links">
					{footer_nav_html}
				</div>
			</div>
			<div>
				<div class="footer-col-title">Feeds &amp; Endpoints</div>
				<div class="footer-links">
					<a href="/sitemap.xml">XML Sitemap</a>
					<a href="/wp-json/wp/v2/posts">REST API v2</a>
					<a href="/n-admin/">Admin Portal</a>
				</div>
			</div>
		</div>
		<div class="footer-bottom">
			<div>
				&copy; 2026 {brand_display}. Powered by Neopress Native Rust CMS.{privacy_link_html}
			</div>
			<div class="engine-chip">
				<span>⚡</span> Neopress AST &bull; Embedded SQLite WAL
			</div>
		</div>
	</footer>"##,
        brand_display = brand_display,
        privacy_link_html = privacy_link_html,
        site_desc = escape_html(site_description(settings)),
        footer_nav_html = footer_nav_html
    )
}
