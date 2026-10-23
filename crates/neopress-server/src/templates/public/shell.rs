//! Public Base HTML Document Shell (`templates/public/shell.rs`).
//!
//! Renders SEO meta tags, OpenGraph cards, Schema.org JSON-LD structured data,
//! Cookieless Analytics beacon, Admin Bar, dynamic Theme Builder Header/Footer,
//! scoped responsive stylesheets, and 3-tier DOM leak sanitizer.

use super::admin_bar::render_admin_bar;
use super::head_meta::{render_head_meta, HeadMetaOutput};
pub use super::head_meta::plain_text_excerpt;
use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::{escape_html, escape_json};
use crate::theme_builder::models::ResolvedThemePartials;
use neopress_ast::sanitize_ssr_html;

/// Renders the unified Neopress HTML shell driven 100% by database settings,
/// dynamic Theme Builder partials, and Neopress AST element styles.
pub fn render_base_layout(
    title: &str,
    settings: &WpSettingsResponse,
    content: &str,
    current_user: Option<&AuthUser>,
    current_post: Option<&WpPostResponse>,
    partials: Option<&ResolvedThemePartials>,
) -> String {
    let site_name = if settings.title.trim().is_empty() {
        "Neopress"
    } else {
        settings.title.trim()
    };
    let site_desc = if settings.description.trim().is_empty() {
        site_name
    } else {
        settings.description.trim()
    };

    let HeadMetaOutput {
        page_title,
        meta_desc,
        robots_meta,
        canonical_tag,
        og_image_tags,
        schema_type,
    } = render_head_meta(title, settings, current_post, site_name, site_desc);

    let brand_color = if settings.brand_color.trim().is_empty() {
        "#ff4f00"
    } else {
        settings.brand_color.trim()
    };

    let (admin_bar_html, body_class_admin) = render_admin_bar(current_user, current_post, settings, site_name);

    let header_login_btn = if current_user.is_some() {
        r##"<a href="/n-admin/" class="np-button np-btn-sm"><i class="fa-solid fa-gauge-high"></i> Dashboard</a>"##
    } else {
        r##"<a href="/n-admin/login.php" class="btn-login">Log In</a>"##
    };

    let brand_display = escape_html(site_name);

    let custom_css_block = if !settings.custom_css.trim().is_empty() {
        format!(
            "\n\t<style id=\"neopress-custom-css\">\n{}\n\t</style>",
            settings.custom_css
        )
    } else {
        String::new()
    };

    let is_blank_template = current_post
        .map(|p| {
            p.template == "page-template-blank.php"
                || p.template == "canvas"
                || p.template == "elementor_canvas"
        })
        .unwrap_or(false);

    let has_elementor_theme = !settings.elementor_kit_css.trim().is_empty()
        || !settings.elementor_header_html.trim().is_empty()
        || !settings.elementor_footer_html.trim().is_empty()
        || is_blank_template
        || content.contains("divi_section_");

    let elementor_kit_css_block = if !settings.elementor_kit_css.trim().is_empty() {
        format!(
            "\n\t<style id=\"neopress-elementor-kit-css\">\n{}\n\t</style>",
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

    let html_theme_class = if has_elementor_theme {
        "elementor-theme-active"
    } else {
        "dark"
    };

    let (nav_links_html, footer_nav_html) = if !settings.nav_menu_items.is_empty() {
        let tree = super::menus::build_nav_menu_tree(&settings.nav_menu_items);
        let menu_html = super::menus::render_hierarchical_nav_menu(&tree, false);
        let footer_html = super::menus::render_footer_nav_links(&settings.nav_menu_items);
        (menu_html, footer_html)
    } else {
        (
            r#"<ul class="menu np-primary-menu"><li class="menu-item np-nav-item"><a href="/" class="nav-link">Home</a></li><li class="menu-item np-nav-item"><a href="/#posts" class="nav-link">Articles</a></li></ul>"#.to_string(),
            r#"<a href="/">Home</a><a href="/#posts">Articles</a>"#.to_string(),
        )
    };

    // Header Resolution Pipeline
    let has_tb_header = partials.map(|p| p.has_theme_builder_header).unwrap_or(false);
    let site_header_html = if is_blank_template {
        String::new()
    } else if let Some(ref h_html) = partials.and_then(|p| p.header_html.as_ref()) {
        let sticky_cls = if partials.map(|p| p.header_sticky).unwrap_or(false) { " np-sticky" } else { "" };
        let trans_cls = if partials.map(|p| p.header_transparent).unwrap_or(false) { " np-transparent" } else { "" };
        format!(r#"<header class="np-theme-header{}{}">{}</header>"#, sticky_cls, trans_cls, h_html)
    } else if !has_tb_header && !settings.elementor_header_html.trim().is_empty() {
        format!(r#"<header class="np-elementor-header">{}</header>"#, settings.elementor_header_html)
    } else {
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
            brand_display = brand_display,
            nav_links_html = nav_links_html,
            header_login_btn = header_login_btn
        )
    };

    // Footer Resolution Pipeline
    let has_tb_footer = partials.map(|p| p.has_theme_builder_footer).unwrap_or(false);
    let site_footer_html = if is_blank_template {
        String::new()
    } else if let Some(ref f_html) = partials.and_then(|p| p.footer_html.as_ref()) {
        format!(r#"<footer class="np-theme-footer">{}</footer>"#, f_html)
    } else if !has_tb_footer && !settings.elementor_footer_html.trim().is_empty() {
        format!(r#"<footer class="np-elementor-footer">{}</footer>"#, settings.elementor_footer_html)
    } else {
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
				&copy; 2026 {brand_display}. Powered by Neopress Native Rust CMS.
			</div>
			<div class="engine-chip">
				<span>⚡</span> Neopress AST &bull; Embedded SQLite WAL
			</div>
		</div>
	</footer>"##,
            brand_display = brand_display,
            site_desc = escape_html(site_desc),
            footer_nav_html = footer_nav_html
        )
    };

    let theme_header_css = partials
        .and_then(|p| p.header_css.as_ref())
        .map(|css| format!("\n\t<style id=\"np-theme-header-css\">\n{}\n\t</style>", css))
        .unwrap_or_default();

    let theme_footer_css = partials
        .and_then(|p| p.footer_css.as_ref())
        .map(|css| format!("\n\t<style id=\"np-theme-footer-css\">\n{}\n\t</style>", css))
        .unwrap_or_default();

    let theme_head_assets = if !settings.theme_assets_html.trim().is_empty() {
        settings.theme_assets_html.as_str()
    } else {
        ""
    };

    let theme_slug = if settings.active_theme.trim().is_empty() {
        "neopress".to_string()
    } else {
        settings.active_theme.trim().to_lowercase().replace(' ', "-")
    };
    let theme_body_class = format!("wp-theme-{}", theme_slug);
    let post_classes = if let Some(p) = current_post {
        if p.post_type == "archive" {
            format!("archive post-type-archive-{} {}", p.slug, theme_body_class)
        } else if p.post_type == "page" {
            if settings.page_on_front > 0 && p.id == settings.page_on_front {
                format!("home wp-singular page-template-default page page-id-{} {}", p.id, theme_body_class)
            } else {
                format!("wp-singular page-template-default page page-id-{} {}", p.id, theme_body_class)
            }
        } else {
            format!("single single-{} wp-singular {}", p.post_type, theme_body_class)
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
	<meta name="generator" content="Neopress Rust CMS" />

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
				fetch('/wp-json/neopress/v1/analytics/event', {{
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

	<!-- Main Content Area (Rendered via Neopress AST Elements) -->
	<main class="neo-main">
		{content}
	</main>

	<!-- Global Site Footer (100% Database-Driven) -->
	{site_footer_html}

	{elementor_popups_html}
	<div id="np-lightbox-modal" class="np-lightbox-modal" role="dialog" aria-modal="true">
		<div class="np-lightbox-backdrop" onclick="this.parentElement.classList.remove('active')"></div>
		<button type="button" class="np-lightbox-close" aria-label="Close" onclick="document.getElementById('np-lightbox-modal').classList.remove('active')">&times;</button>
		<div class="np-lightbox-content">
			<img id="np-lightbox-img" src="" alt="" />
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
