//! Public Base HTML Document Shell (`templates/public/shell.rs`).
//!
//! Renders SEO meta tags, OpenGraph cards, Schema.org JSON-LD structured data,
//! Cookieless Analytics beacon, Admin Bar, dynamic Theme Builder Header/Footer,
//! scoped responsive stylesheets, and 3-tier DOM leak sanitizer.

use super::admin_bar::render_admin_bar;
pub use super::head_meta::plain_text_excerpt;
use super::head_meta::{render_head_meta, HeadMetaOutput};
use super::site_chrome;
use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::{escape_html, escape_json};
use crate::theme_builder::models::ResolvedThemePartials;
use daypress_ast::sanitize_ssr_html;

/// Renders the unified Daypress HTML shell driven 100% by database settings,
/// dynamic Theme Builder partials, and Daypress AST element styles.
pub fn render_base_layout(
    title: &str,
    settings: &WpSettingsResponse,
    content: &str,
    current_user: Option<&AuthUser>,
    current_post: Option<&WpPostResponse>,
    partials: Option<&ResolvedThemePartials>,
) -> String {
    let site_name = site_chrome::site_name(settings);
    let site_desc = site_chrome::site_description(settings);

    let HeadMetaOutput {
        page_title,
        meta_desc,
        robots_meta,
        canonical_tag,
        og_image_tags,
        schema_type,
        feed_links,
    } = render_head_meta(title, settings, current_post, site_name, site_desc);

    let brand_color = site_chrome::brand_color(settings);

    let (admin_bar_html, body_class_admin) =
        render_admin_bar(current_user, current_post, settings, site_name);

    // Customizer → Typography: swap the site font stack (custom CSS below can still override it).
    let typography_css = site_chrome::typography_css(settings);
    let typography_block = if typography_css.is_empty() {
        String::new()
    } else {
        format!(
            "\n\t<style id=\"daypress-typography\">{}</style>",
            typography_css
        )
    };
    let custom_css_block = if !settings.custom_css.trim().is_empty() {
        format!(
            "{}\n\t<style id=\"daypress-custom-css\">\n{}\n\t</style>",
            typography_block, settings.custom_css
        )
    } else {
        typography_block
    };

    let is_blank_template = site_chrome::is_blank_template(current_post);

    let elementor_kit_css_block = if !settings.elementor_kit_css.trim().is_empty() {
        format!(
            "\n\t<style id=\"daypress-elementor-kit-css\">\n{}\n\t</style>",
            settings.elementor_kit_css
        )
    } else {
        String::new()
    };

    let favicon_href = if !settings.site_favicon_url.trim().is_empty() {
        escape_html(settings.site_favicon_url.trim())
    } else {
        "/static/favicon.svg".to_string()
    };

    let html_theme_class = site_chrome::html_theme_class(settings, content, is_blank_template);

    // Header & Footer Resolution Pipelines (Theme Builder → Elementor → native Daypress)
    let site_header_html =
        site_chrome::render_site_header(settings, current_user, partials, is_blank_template);
    let site_footer_html = site_chrome::render_site_footer(settings, partials, is_blank_template);

    let theme_header_css = partials
        .and_then(|p| p.header_css.as_ref())
        .map(|css| {
            format!(
                "\n\t<style id=\"dp-theme-header-css\">\n{}\n\t</style>",
                css
            )
        })
        .unwrap_or_default();

    let theme_footer_css = partials
        .and_then(|p| p.footer_css.as_ref())
        .map(|css| {
            format!(
                "\n\t<style id=\"dp-theme-footer-css\">\n{}\n\t</style>",
                css
            )
        })
        .unwrap_or_default();

    let theme_head_assets = if !settings.theme_assets_html.trim().is_empty() {
        settings.theme_assets_html.as_str()
    } else {
        ""
    };

    let theme_slug = if settings.active_theme.trim().is_empty() {
        "daypress".to_string()
    } else {
        settings
            .active_theme
            .trim()
            .to_lowercase()
            .replace(' ', "-")
    };
    let theme_body_class = format!("wp-theme-{}", theme_slug);
    let post_classes = if let Some(p) = current_post {
        if p.post_type == "archive" {
            format!("archive post-type-archive-{} {}", p.slug, theme_body_class)
        } else if p.post_type == "page" {
            if settings.page_on_front > 0 && p.id == settings.page_on_front {
                format!(
                    "home wp-singular page-template-default page page-id-{} {}",
                    p.id, theme_body_class
                )
            } else {
                format!(
                    "wp-singular page-template-default page page-id-{} {}",
                    p.id, theme_body_class
                )
            }
        } else {
            format!(
                "single single-{} wp-singular {}",
                p.post_type, theme_body_class
            )
        }
    } else {
        format!("home wp-singular page {}", theme_body_class)
    };
    let full_body_class = if body_class_admin.trim().is_empty() {
        post_classes
    } else {
        format!("{} {}", body_class_admin.trim(), post_classes)
    };

    let doc = format!(
        r##"<!DOCTYPE html>
<html lang="nl-NL" class="{html_theme_class}">
<head>
	<meta charset="UTF-8" />
	<meta name="viewport" content="width=device-width, initial-scale=1" />
	<title>{page_title}</title>
	<meta name="description" content="{meta_desc}" />
	{robots_meta}
	{canonical_tag}
	{feed_links}
	<meta name="generator" content="Daypress Rust CMS" />

	<!-- Native SEO: OpenGraph & Twitter Cards -->
	<meta property="og:title" content="{page_title}" />
	<meta property="og:description" content="{meta_desc}" />
	<meta property="og:type" content="website" />
	<meta property="og:site_name" content="{site_name}" />
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content="{page_title}" />
	<meta name="twitter:description" content="{meta_desc}" />
	{og_image_tags}
	<link rel="icon" href="{favicon_href}" />

	<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" crossorigin="anonymous" referrerpolicy="no-referrer" />

	<script type="application/ld+json">
	{{
		"@context": "https://schema.org",
		"@type": "{schema_type}",
		"name": "{site_name_json}",
		"headline": "{title_json}",
		"description": "{meta_desc_json}",
		"publisher": {{
			"@type": "Organization",
			"name": "{site_name_json}"
		}}
	}}
	</script>

	<script>
	(function(){{
		try {{
			if (window.fetch) {{
				fetch('/wp-json/daypress/v1/analytics/event', {{
					method: 'POST',
					headers: {{ 'Content-Type': 'application/json' }},
					body: JSON.stringify({{
						page_url: window.location.pathname,
						referrer: document.referrer || '',
						device_type: window.innerWidth < 768 ? 'mobile' : (window.innerWidth < 1024 ? 'tablet' : 'desktop')
					}}),
					keepalive: true
				}}).catch(function(){{}});
			}}
		}} catch(e) {{}}
	}})();
	</script>

	<style>
		{public_theme_css}
	</style>{elementor_kit_css_block}{custom_css_block}{theme_header_css}{theme_footer_css}{theme_head_assets}
</head>
<body class="{full_body_class}">
	{admin_bar_html}

	<!-- Global Site Header (100% Database-Driven) -->
	{site_header_html}

	<!-- Main Content Area (Rendered via Daypress AST Elements) -->
	<main class="day-main">
		{content}
	</main>

	<!-- Global Site Footer (100% Database-Driven) -->
	{site_footer_html}

	{elementor_popups_html}
	<div id="dp-lightbox-modal" class="dp-lightbox-modal" role="dialog" aria-modal="true">
		<div class="dp-lightbox-backdrop" onclick="this.parentElement.classList.remove('active')"></div>
		<button type="button" class="dp-lightbox-close" aria-label="Close" onclick="document.getElementById('dp-lightbox-modal').classList.remove('active')">&times;</button>
		<div class="dp-lightbox-content">
			<img id="dp-lightbox-img" src="" alt="" />
		</div>
	</div>
</body>
</html>
"##,
        html_theme_class = html_theme_class,
        page_title = escape_html(&page_title),
        site_name = escape_html(site_name),
        meta_desc = escape_html(&meta_desc),
        robots_meta = robots_meta,
        canonical_tag = canonical_tag,
        feed_links = feed_links,
        og_image_tags = og_image_tags,
        favicon_href = favicon_href,
        schema_type = schema_type,
        site_name_json = escape_json(site_name),
        title_json = escape_json(title),
        meta_desc_json = escape_json(&meta_desc),
        public_theme_css = super::styles::render_public_theme_css(&escape_html(brand_color)),
        elementor_kit_css_block = elementor_kit_css_block,
        custom_css_block = custom_css_block,
        theme_header_css = theme_header_css,
        theme_footer_css = theme_footer_css,
        theme_head_assets = theme_head_assets,
        site_header_html = site_header_html,
        site_footer_html = site_footer_html,
        elementor_popups_html = settings.elementor_popups_html,
        content = content,
        full_body_class = full_body_class,
        admin_bar_html = admin_bar_html
    );

    sanitize_ssr_html(&doc)
}
