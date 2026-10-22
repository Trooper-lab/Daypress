use super::utils::{clean_wp_content, escape_html, escape_json};
use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use neopress_ast::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};

/// Strips HTML tags cleanly and truncates on UTF-8 character boundaries.
fn plain_text_excerpt(html: &str, max_chars: usize) -> String {
    let cleaned = clean_wp_content(html);
    let mut out = String::with_capacity(cleaned.len());
    let mut in_tag = false;
    for ch in cleaned.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let normalized = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() > max_chars {
        let prefix: String = normalized.chars().take(max_chars).collect();
        format!("{}...", escape_html(&prefix))
    } else {
        escape_html(&normalized)
    }
}

/// Renders the unified Neopress HTML shell driven 100% by database settings and Neopress AST element styles.
pub fn render_base_layout(
    title: &str,
    settings: &WpSettingsResponse,
    content: &str,
    current_user: Option<&AuthUser>,
    current_post: Option<&WpPostResponse>,
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

    let page_title = if let Some(post) = current_post {
        if !post.seo_title.trim().is_empty() {
            post.seo_title.trim().to_string()
        } else {
            format!("{} – {}", title, site_name)
        }
    } else if title == site_name {
        if !settings.description.trim().is_empty() {
            format!("{} – {}", site_name, site_desc)
        } else {
            site_name.to_string()
        }
    } else {
        format!("{} – {}", title, site_name)
    };

    let meta_desc = if let Some(post) = current_post {
        if !post.seo_description.trim().is_empty() {
            post.seo_description.trim().to_string()
        } else {
            let ex = plain_text_excerpt(&post.content.rendered, 155);
            if !ex.is_empty() {
                ex
            } else {
                site_desc.to_string()
            }
        }
    } else {
        site_desc.to_string()
    };

    let robots_meta = if current_post.map(|p| p.seo_noindex).unwrap_or(false) {
        r#"<meta name="robots" content="noindex, follow" />"#
    } else {
        r#"<meta name="robots" content="index, follow" />"#
    };

    let canonical_tag = if let Some(post) = current_post {
        let can_url = if !post.seo_canonical.trim().is_empty() {
            post.seo_canonical.trim().to_string()
        } else {
            post.link.clone()
        };
        if !can_url.is_empty() {
            format!(r#"<link rel="canonical" href="{}" />"#, escape_html(&can_url))
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let og_image_tags = if let Some(post) = current_post {
        let img = if !post.seo_og_image.trim().is_empty() {
            Some(post.seo_og_image.trim())
        } else {
            post.featured_image_url.as_deref()
        };
        if let Some(url) = img.filter(|u| !u.is_empty()) {
            format!(
                r#"<meta property="og:image" content="{0}" /><meta name="twitter:image" content="{0}" />"#,
                escape_html(url)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let brand_color = if settings.brand_color.trim().is_empty() {
        "#ff4f00"
    } else {
        settings.brand_color.trim()
    };

    let (admin_bar_html, body_class_admin) = if let Some(user) = current_user {
        let username = escape_html(&user.username);
        let post_actions = if let Some(post) = current_post {
            let label = if post.post_type == "page" {
                "Page"
            } else {
                "Post"
            };
            format!(
                r##"
                <li id="wp-admin-bar-edit">
                    <a class="ab-item" href="/n-admin/post.php?post={id}&action=edit">
                        <span class="ab-icon"><i class="fa-solid fa-pencil"></i></span>
                        <span class="ab-label">Edit {label}</span>
                    </a>
                </li>
                <li id="wp-admin-bar-elementor-edit">
                    <a class="ab-item ab-btn-builder" href="/n-admin/builder?post={id}">
                        <span class="ab-icon"><i class="fa-solid fa-cubes"></i></span>
                        <span class="ab-label">Edit with Visual Builder</span>
                    </a>
                </li>
                "##,
                id = post.id,
                label = label
            )
        } else {
            r##"
            <li id="wp-admin-bar-customize">
                <a class="ab-item" href="/n-admin/customize.php">
                    <span class="ab-icon"><i class="fa-solid fa-paintbrush"></i></span>
                    <span class="ab-label">Customize</span>
                </a>
            </li>
            "##
            .to_string()
        };

        let bar = format!(
            r##"
            <div id="wpadminbar" class="nojq">
                <div class="quicklinks" id="wp-toolbar" role="navigation" aria-label="Toolbar">
                    <ul id="wp-admin-bar-root-default" class="ab-top-menu">
                        <li id="wp-admin-bar-wp-logo" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/">
                                <span class="ab-icon"><i class="fa-solid fa-bolt"></i></span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/">Dashboard</a></li>
                                    <li><a class="ab-item" href="/n-admin/neopress-engine.php">System Health</a></li>
                                </ul>
                            </div>
                        </li>
                        <li id="wp-admin-bar-site-name" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/">{site_name}</a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/">Dashboard</a></li>
                                    <li><a class="ab-item" href="/n-admin/themes.php">Themes</a></li>
                                    <li><a class="ab-item" href="/n-admin/widgets.php">Widgets</a></li>
                                    <li><a class="ab-item" href="/n-admin/nav-menus.php">Menus</a></li>
                                </ul>
                            </div>
                        </li>
                        <li id="wp-admin-bar-comments">
                            <a class="ab-item" href="/n-admin/edit-comments.php">
                                <span class="ab-icon"><i class="fa-solid fa-comment"></i></span>
                                <span class="ab-label">{pending_comments}</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-new-content" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/post-new.php">
                                <span class="ab-icon"><i class="fa-solid fa-plus"></i></span>
                                <span class="ab-label">New</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/post-new.php">Post</a></li>
                                    <li><a class="ab-item" href="/n-admin/upload.php">Media</a></li>
                                    <li><a class="ab-item" href="/n-admin/post-new.php?post_type=page">Page</a></li>
                                    <li><a class="ab-item" href="/n-admin/user-new.php">User</a></li>
                                </ul>
                            </div>
                        </li>
                        {post_actions}
                    </ul>
                    <ul id="wp-admin-bar-top-secondary" class="ab-top-secondary ab-top-menu">
                        <li id="wp-admin-bar-my-account" class="menupop with-avatar">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/profile.php">
                                Howdy, <span class="display-name">{username}</span>
                                <span class="ab-avatar" style="margin-left: 6px;"><i class="fa-solid fa-user"></i></span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/profile.php">Edit Profile</a></li>
                                    <li><a class="ab-item" href="/wp-login.php?action=logout" style="color: #f87171;">Log Out</a></li>
                                </ul>
                            </div>
                        </li>
                    </ul>
                </div>
            </div>
            "##,
            site_name = escape_html(site_name),
            pending_comments = settings.pending_comments_count,
            post_actions = post_actions,
            username = username
        );
        (bar, " admin-bar")
    } else {
        (String::new(), "")
    };

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

    let has_elementor_theme = !settings.elementor_kit_css.trim().is_empty()
        || !settings.elementor_header_html.trim().is_empty()
        || !settings.elementor_footer_html.trim().is_empty();

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
        let mut items_html = String::new();
        let mut f_html = String::new();
        for item in &settings.nav_menu_items {
            let target_attr = if !item.target.is_empty() {
                format!(r#" target="{}""#, escape_html(&item.target))
            } else {
                String::new()
            };
            items_html.push_str(&format!(
                r#"<a href="{url}" class="nav-link"{target}>{title}</a>"#,
                url = escape_html(&item.url),
                target = target_attr,
                title = escape_html(&item.title),
            ));
            f_html.push_str(&format!(
                r#"<a href="{url}"{target}>{title}</a>"#,
                url = escape_html(&item.url),
                target = target_attr,
                title = escape_html(&item.title),
            ));
        }
        (items_html, f_html)
    } else {
        (
            r#"<a href="/" class="nav-link">Home</a><a href="/#posts" class="nav-link">Articles</a>"#
                .to_string(),
            r#"<a href="/">Home</a><a href="/#posts">Articles</a>"#.to_string(),
        )
    };

    let schema_type = if let Some(post) = current_post {
        if !post.seo_schema_type.trim().is_empty() {
            post.seo_schema_type.trim().to_string()
        } else if post.post_type == "post" {
            "BlogPosting".to_string()
        } else {
            "WebPage".to_string()
        }
    } else {
        "WebSite".to_string()
    };

    let site_header_html = if !settings.elementor_header_html.trim().is_empty() {
        format!(
            r#"<header class="np-elementor-header">{}</header>"#,
            settings.elementor_header_html
        )
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

    let site_footer_html = if !settings.elementor_footer_html.trim().is_empty() {
        format!(
            r#"<footer class="np-elementor-footer">{}</footer>"#,
            settings.elementor_footer_html
        )
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

    format!(
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

	<!-- Typography: Fira Sans, Roboto, Inter & JetBrains Mono -->
	<link rel="preconnect" href="https://fonts.googleapis.com">
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
	<link href="https://fonts.googleapis.com/css2?family=Fira+Sans:ital,wght@0,300;0,400;0,500;0,600;0,700;1,300;1,400;1,600&family=Inter:wght@400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500;600;700&family=Roboto:ital,wght@0,300;0,400;0,500;0,700;1,300;1,400&display=swap" rel="stylesheet">
	<!-- FontAwesome 6 -->
	<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" crossorigin="anonymous" referrerpolicy="no-referrer" />

	<!-- Native SEO: Schema.org JSON-LD Structured Data -->
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

	<!-- Native Cookieless Analytics Ingest -->
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
		:root {{
			--bg-base: #07090e;
			--bg-panel: #0b0f19;
			--bg-card: #111827;
			--bg-card-hover: #1e293b;
			--border: #1e293b;
			--border-subtle: #162032;
			--border-focus: {brand_color};
			--text-main: #f8fafc;
			--text-muted: #94a3b8;
			--text-dim: #64748b;
			--accent: {brand_color};
			--accent-hover: #3b82f6;
			--accent-glow: rgba(37, 99, 235, 0.25);
			--e-global-color-primary: #2563eb;
			--e-global-color-secondary: #0f172a;
			--e-global-color-text: #e2e8f0;
			--e-global-color-accent: #ff4f00;
			--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
			--font-mono: 'JetBrains Mono', monospace;
		}}

		*, *::before, *::after {{
			box-sizing: border-box;
			margin: 0;
			padding: 0;
		}}

		body {{
			background-color: var(--bg-base);
			color: var(--text-main);
			font-family: var(--font-sans);
			font-size: 15px;
			line-height: 1.65;
			-webkit-font-smoothing: antialiased;
			min-height: 100vh;
			display: flex;
			flex-direction: column;
			overflow-x: hidden;
		}}

		a {{
			color: inherit;
			text-decoration: none;
			transition: all 0.15s ease;
		}}

		/* Global Header */
		header.neo-header {{
			position: sticky;
			top: 0;
			z-index: 1000;
			background: rgba(11, 15, 25, 0.9);
			backdrop-filter: blur(12px);
			-webkit-backdrop-filter: blur(12px);
			border-bottom: 1px solid var(--border);
			min-height: 68px;
			display: flex;
			align-items: center;
		}}

		.header-container {{
			max-width: 1240px;
			margin: 0 auto;
			padding: 10px 24px;
			width: 100%;
			display: flex;
			align-items: center;
			justify-content: space-between;
			gap: 20px;
		}}

		.logo-brand {{
			display: inline-flex;
			align-items: center;
			gap: 10px;
			font-weight: 800;
			font-size: 17px;
			letter-spacing: -0.02em;
			color: #ffffff;
			white-space: nowrap;
		}}

		.logo-brand .badge-neo {{
			background: linear-gradient(135deg, #2563eb 0%, #ff4f00 100%);
			color: #ffffff;
			font-size: 10px;
			font-weight: 800;
			padding: 3px 7px;
			border-radius: 4px;
			letter-spacing: 0.05em;
			text-transform: uppercase;
		}}

		nav.neo-nav {{
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 18px;
		}}

		.nav-link {{
			font-size: 13.5px;
			font-weight: 500;
			color: var(--text-muted);
			transition: color 0.15s ease;
		}}

		.nav-link:hover {{
			color: #ffffff;
		}}

		.header-actions {{
			display: flex;
			align-items: center;
			gap: 12px;
		}}

		.header-search {{
			display: flex;
			align-items: center;
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 6px;
			padding: 4px 10px;
		}}

		.header-search input {{
			background: transparent;
			border: none;
			color: #ffffff;
			font-size: 12.5px;
			width: 140px;
			outline: none;
		}}

		.btn-login {{
			font-size: 13px;
			font-weight: 600;
			color: var(--text-muted);
			padding: 6px 12px;
			border-radius: 6px;
			border: 1px solid var(--border);
		}}

		.btn-login:hover {{
			color: #ffffff;
			border-color: #475569;
		}}

		main.neo-main {{
			flex: 1;
		}}

		/* ==========================================================================
		   Canonical Neopress AST Element System (.np-*)
		   ========================================================================== */
		.np-section {{
			width: 100%;
			padding: 64px 24px;
			position: relative;
			border-bottom: 1px solid var(--border-subtle);
		}}

		.np-container {{
			max-width: 1200px;
			margin: 0 auto;
			width: 100%;
		}}

		.np-grid {{
			display: grid;
			gap: 24px;
			width: 100%;
		}}

		.np-flex {{
			display: flex;
			gap: 20px;
		}}

		.np-section:has(> .np-flex + .np-flex),
		.np-container:has(> .np-flex + .np-flex) {{
			display: flex;
			flex-direction: row;
			gap: 28px;
			align-items: flex-start;
		}}

		.np-section > .np-flex + .np-flex,
		.np-section > .np-flex:has(+ .np-flex),
		.np-container > .np-flex + .np-flex,
		.np-container > .np-flex:has(+ .np-flex) {{
			flex: 1 1 0%;
			min-width: 0;
		}}

		.np-column {{
			display: flex;
			flex-direction: column;
			gap: 16px;
			min-width: 0;
		}}

		.np-heading {{
			color: #ffffff;
			font-weight: 800;
			letter-spacing: -0.025em;
			line-height: 1.2;
			margin-bottom: 12px;
		}}

		h1.np-heading {{ font-size: 44px; }}
		h2.np-heading {{ font-size: 32px; }}
		h3.np-heading {{ font-size: 22px; }}
		h4.np-heading {{ font-size: 18px; }}

		.np-text {{
			color: #cbd5e1;
			font-size: 16px;
			line-height: 1.75;
		}}

		.np-text p {{
			margin-bottom: 16px;
		}}

		.np-text p:last-child {{
			margin-bottom: 0;
		}}

		.np-image {{
			max-width: 100%;
			height: auto;
			border-radius: 10px;
			display: block;
		}}

		.np-button, .btn-primary-neon {{
			display: inline-flex;
			align-items: center;
			justify-content: center;
			gap: 8px;
			background: #2563eb;
			color: #ffffff;
			font-size: 14px;
			font-weight: 600;
			padding: 11px 22px;
			border-radius: 8px;
			border: 1px solid rgba(255,255,255,0.1);
			cursor: pointer;
			transition: all 0.15s ease;
			width: fit-content;
		}}

		.np-button:hover, .btn-primary-neon:hover {{
			background: #1d4ed8;
			transform: translateY(-1px);
		}}

		.np-btn-sm {{
			padding: 6px 14px;
			font-size: 12.5px;
		}}

		.btn-secondary-edge {{
			display: inline-flex;
			align-items: center;
			justify-content: center;
			gap: 8px;
			background: var(--bg-card);
			color: #ffffff;
			font-size: 14px;
			font-weight: 600;
			padding: 10px 20px;
			border-radius: 8px;
			border: 1px solid var(--border);
		}}

		.np-divider {{
			border: none;
			border-top: 1px solid var(--border);
			margin: 24px 0;
			width: 100%;
		}}

		.np-iconbox, .np-imagebox {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 12px;
			padding: 28px;
			display: flex;
			flex-direction: column;
			gap: 10px;
		}}

		.np-iconbox-icon {{
			width: 44px;
			height: 44px;
			border-radius: 10px;
			background: rgba(37, 99, 235, 0.15);
			color: #60a5fa;
			display: flex;
			align-items: center;
			justify-content: center;
			font-size: 20px;
			margin-bottom: 4px;
		}}

		.np-iconbox-title, .np-imagebox-title {{
			font-size: 19px;
			font-weight: 700;
			color: #ffffff;
		}}

		.np-iconbox-desc, .np-imagebox-desc {{
			font-size: 14.5px;
			color: var(--text-muted);
			line-height: 1.6;
		}}

		.np-counter {{
			background: rgba(15, 23, 42, 0.75);
			border: 1px solid var(--border);
			border-radius: 12px;
			padding: 24px;
			text-align: center;
		}}

		.np-counter-val {{
			font-size: 38px;
			font-weight: 900;
			color: #38bdf8;
			font-family: var(--font-mono);
			line-height: 1.1;
			margin-bottom: 6px;
		}}

		.np-counter-title {{
			font-size: 13px;
			font-weight: 600;
			color: var(--text-muted);
			text-transform: uppercase;
			letter-spacing: 0.05em;
		}}

		.np-accordion {{
			display: flex;
			flex-direction: column;
			gap: 10px;
		}}

		.np-accordion-item {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 10px;
			overflow: hidden;
		}}

		.np-accordion-item summary {{
			padding: 16px 20px;
			font-weight: 700;
			font-size: 15px;
			color: #ffffff;
			cursor: pointer;
			list-style: none;
		}}

		.np-accordion-body {{
			padding: 0 20px 18px 20px;
			color: var(--text-muted);
			font-size: 14.5px;
			line-height: 1.65;
		}}

		.np-price-table {{
			background: linear-gradient(180deg, #111827 0%, #0f172a 100%);
			border: 1px solid #334155;
			border-radius: 14px;
			padding: 32px;
			display: flex;
			flex-direction: column;
			gap: 20px;
		}}

		.np-price-title {{
			font-size: 20px;
			font-weight: 800;
			color: #ffffff;
		}}

		.np-price-amount {{
			display: flex;
			align-items: baseline;
			gap: 6px;
		}}

		.np-price-val {{
			font-size: 44px;
			font-weight: 900;
			color: #ffffff;
			font-family: var(--font-mono);
		}}

		.np-price-period {{
			font-size: 14px;
			color: var(--text-muted);
		}}

		.np-price-features {{
			list-style: none;
			display: flex;
			flex-direction: column;
			gap: 10px;
		}}

		.np-price-feature {{
			font-size: 14.5px;
			color: #cbd5e1;
			display: flex;
			align-items: center;
			gap: 8px;
		}}

		.np-price-feature::before {{
			content: "✓";
			color: #10b981;
			font-weight: 800;
		}}

		.np-progressbar {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 10px;
			padding: 18px 20px;
		}}

		.np-progress-title {{
			display: flex;
			justify-content: space-between;
			font-size: 13.5px;
			font-weight: 700;
			color: #ffffff;
			margin-bottom: 10px;
		}}

		.np-progress-track {{
			height: 10px;
			background: #090d16;
			border-radius: 999px;
			overflow: hidden;
		}}

		.np-progress-fill {{
			height: 100%;
			background: linear-gradient(90deg, #2563eb, #38bdf8);
			border-radius: 999px;
		}}

		/* Neopress Post & Page Card Elements */
		.np-card {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 12px;
			overflow: hidden;
			display: flex;
			flex-direction: column;
			transition: transform 0.15s ease, border-color 0.15s ease;
		}}

		.np-card:hover {{
			border-color: #3b82f6;
			transform: translateY(-2px);
		}}

		.np-card-media {{
			width: 100%;
			height: 200px;
			overflow: hidden;
			background: #0f172a;
			border-bottom: 1px solid var(--border);
		}}

		.np-card-media img {{
			width: 100%;
			height: 100%;
			object-fit: cover;
			display: block;
		}}

		.np-card-body {{
			padding: 24px;
			display: flex;
			flex-direction: column;
			flex: 1;
		}}

		.np-card-meta {{
			font-size: 12px;
			color: var(--text-dim);
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 8px;
			margin-bottom: 10px;
		}}

		.np-badge {{
			background: rgba(37, 99, 235, 0.18);
			color: #60a5fa;
			padding: 2px 8px;
			border-radius: 4px;
			font-weight: 700;
			font-size: 11px;
			text-transform: uppercase;
		}}

		.np-card-title {{
			font-size: 20px;
			font-weight: 700;
			color: #ffffff;
			line-height: 1.35;
			margin-bottom: 10px;
		}}

		.np-card-excerpt {{
			font-size: 14px;
			color: var(--text-muted);
			line-height: 1.6;
			margin-bottom: 18px;
			flex: 1;
		}}

		.np-card-link {{
			font-size: 13px;
			font-weight: 600;
			color: #60a5fa;
		}}

		/* Global Footer */
		footer.neo-footer {{
			background: #04060b;
			border-top: 1px solid var(--border);
			padding: 56px 0 36px 0;
			color: var(--text-muted);
			font-size: 13.5px;
		}}

		.footer-grid {{
			max-width: 1200px;
			margin: 0 auto;
			padding: 0 24px;
			display: grid;
			grid-template-columns: 2fr 1.5fr 1fr;
			gap: 40px;
			margin-bottom: 40px;
		}}

		.footer-brand p {{
			color: var(--text-dim);
			margin-top: 12px;
			max-width: 420px;
			line-height: 1.6;
		}}

		.footer-col-title {{
			color: #ffffff;
			font-size: 12px;
			font-weight: 700;
			text-transform: uppercase;
			letter-spacing: 0.06em;
			margin-bottom: 16px;
		}}

		.footer-links {{
			display: flex;
			flex-direction: column;
			gap: 8px;
		}}

		.footer-links a {{
			color: var(--text-muted);
		}}

		.footer-links a:hover {{
			color: #ffffff;
		}}

		.footer-bottom {{
			max-width: 1200px;
			margin: 0 auto;
			padding: 24px 24px 0 24px;
			border-top: 1px solid var(--border-subtle);
			display: flex;
			justify-content: space-between;
			align-items: center;
			font-size: 12.5px;
			color: var(--text-dim);
		}}

		.engine-chip {{
			display: inline-flex;
			align-items: center;
			gap: 6px;
			background: #0c1220;
			border: 1px solid #1e293b;
			padding: 3px 10px;
			border-radius: 4px;
			color: #38bdf8;
			font-family: var(--font-mono);
			font-size: 11px;
		}}

		/* WordPress Live Admin Bar */
		#wpadminbar {{
			direction: ltr;
			color: #c3c4c7;
			font-size: 13px;
			font-weight: 400;
			font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
			line-height: 32px;
			height: 32px;
			position: fixed;
			top: 0;
			left: 0;
			width: 100%;
			z-index: 999999;
			background: #1d2327;
			box-shadow: 0 1px 4px rgba(0,0,0,0.5);
		}}
		body.admin-bar {{
			margin-top: 32px !important;
		}}
		body.admin-bar header.neo-header {{
			top: 32px !important;
		}}
		#wpadminbar .quicklinks {{
			display: flex;
			justify-content: space-between;
			align-items: center;
			height: 32px;
			max-width: 100%;
			padding: 0 12px;
		}}
		#wpadminbar ul {{
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			align-items: center;
			height: 32px;
		}}
		#wpadminbar li {{
			position: relative;
			display: inline-block;
			height: 32px;
		}}
		#wpadminbar .ab-item {{
			display: flex;
			align-items: center;
			gap: 6px;
			padding: 0 10px;
			color: #c3c4c7;
			font-size: 13px;
			height: 32px;
			line-height: 32px;
		}}
		#wpadminbar li:hover > .ab-item {{
			background: #2c3338;
			color: #72aee6;
		}}
		#wpadminbar .ab-btn-builder {{
			background: rgba(224, 38, 96, 0.2);
			color: #f472b6 !important;
			font-weight: 600;
			border-radius: 4px;
			margin: 0 4px;
			height: 24px;
			line-height: 24px;
		}}
		#wpadminbar .ab-sub-wrapper {{
			display: none;
			position: absolute;
			top: 32px;
			left: 0;
			min-width: 190px;
			background: #2c3338;
			box-shadow: 0 4px 14px rgba(0,0,0,0.6);
			border-radius: 0 0 6px 6px;
			z-index: 1000000;
			padding: 4px 0;
		}}
		#wpadminbar li:hover > .ab-sub-wrapper {{
			display: block;
		}}
		#wpadminbar .ab-sub-wrapper ul {{
			display: flex;
			flex-direction: column;
			height: auto;
		}}
		#wpadminbar .ab-sub-wrapper li {{
			width: 100%;
			height: 28px;
		}}
		#wpadminbar .ab-sub-wrapper .ab-item {{
			height: 28px;
			line-height: 28px;
			padding: 0 14px;
			font-size: 12px;
		}}
		#wpadminbar .ab-top-secondary {{
			margin-left: auto;
		}}
		#wpadminbar #wp-admin-bar-my-account .ab-sub-wrapper {{
			right: 0;
			left: auto;
		}}

		/* ==========================================================================
		   Elementor Theme Builder, Flexbox Container, Menu, Form & Modal System
		   ========================================================================== */
		html.elementor-theme-active body {{
			background-color: #ffffff;
			color: var(--e-global-color-text, #1F1C17);
		}}

		.np-elementor-header {{
			width: 100%;
			background: #ffffff;
			position: relative;
			z-index: 1000;
		}}

		.np-elementor-footer {{
			width: 100%;
			position: relative;
		}}

		.np-visual-canvas .np-container,
		.np-elementor-header .np-container,
		.np-elementor-footer .np-container,
		.np-popup-modal .np-container {{
			max-width: 100%;
			margin: 0;
			position: relative;
			background-size: cover;
			background-position: center;
			background-repeat: no-repeat;
		}}

		.np-container.np-container-boxed {{
			max-width: 100%;
			width: 100%;
		}}

		.np-container-boxed > .np-container-inner {{
			max-width: var(--e-content-width, 1140px);
			width: 100%;
			margin: 0 auto;
			display: flex;
			flex-direction: inherit;
			justify-content: inherit;
			align-items: inherit;
			gap: inherit;
			flex-wrap: inherit;
			min-height: inherit;
		}}

		.np-container[style*="flex-direction: row"] > .np-container:not([style*="width:"]),
		.np-container[style*="flex-direction: row"] > .np-container-inner > .np-container:not([style*="width:"]) {{
			flex: 1 1 0%;
			min-width: 0;
		}}

		html.elementor-theme-active .np-image {{
			border-radius: 0;
		}}

		html.elementor-theme-active .np-imagebox,
		html.elementor-theme-active .np-iconbox {{
			background: transparent;
			border: none;
			border-radius: 0;
			padding: 0;
			align-items: center;
		}}

		html.elementor-theme-active .np-imagebox-title,
		html.elementor-theme-active .np-iconbox-title {{
			color: var(--e-global-color-secondary, #1F1C17);
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
		}}

		html.elementor-theme-active .np-imagebox-desc,
		html.elementor-theme-active .np-iconbox-desc {{
			color: var(--e-global-color-text, #1F1C17);
		}}

		html.elementor-theme-active .np-divider {{
			border-top-color: rgba(31, 28, 23, 0.15);
		}}

		.np-nav-menu .np-nav-list {{
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			flex-wrap: wrap;
			gap: 28px;
			align-items: center;
		}}

		.np-nav-menu.np-nav-layout-vertical .np-nav-list {{
			flex-direction: column;
			align-items: flex-start;
			gap: 8px;
		}}

		.np-nav-menu .np-nav-link {{
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
			font-size: 16px;
			font-weight: 400;
			color: var(--e-global-color-secondary, #1F1C17);
			text-decoration: none;
			transition: color 0.15s ease;
		}}

		.np-nav-menu .np-nav-link:hover {{
			color: var(--e-global-color-primary, #9CC13C);
		}}

		.np-icon-list {{
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			flex-direction: column;
			gap: 8px;
		}}

		.np-icon-list-item {{
			display: flex;
			align-items: center;
			gap: 10px;
			line-height: 1.65;
		}}

		.np-icon-list-item i {{
			color: var(--e-global-color-primary, #9CC13C);
			font-size: 16px;
			width: 18px;
			text-align: center;
			flex-shrink: 0;
		}}

		.np-icon-list-link {{
			display: inline-flex;
			align-items: center;
			gap: 10px;
			color: inherit;
			text-decoration: none;
		}}

		.np-icon-list-link:hover {{
			color: var(--e-global-color-primary, #9CC13C);
		}}

		.np-social-icons {{
			display: flex;
			align-items: center;
			gap: 12px;
			flex-wrap: wrap;
		}}

		.np-social-icon {{
			display: inline-flex;
			align-items: center;
			justify-content: center;
			width: 38px;
			height: 38px;
			border-radius: 50%;
			background: var(--e-global-color-secondary, #1F1C17);
			color: #ffffff !important;
			font-size: 17px;
			transition: opacity 0.15s ease, transform 0.15s ease;
		}}

		.np-social-icon:hover {{
			opacity: 0.85;
			transform: translateY(-1px);
		}}

		.np-elementor-form-grid {{
			display: flex;
			flex-wrap: wrap;
			gap: 16px;
			width: 100%;
		}}

		.np-elementor-form-grid .np-field-group {{
			display: flex;
			flex-direction: column;
			gap: 6px;
		}}

		.np-elementor-form-grid .np-field-w100 {{
			width: 100%;
		}}

		.np-elementor-form-grid .np-field-w50 {{
			width: calc(50% - 8px);
		}}

		.np-elementor-form-grid .np-field-label {{
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
			font-weight: 500;
			font-size: 15px;
			color: var(--e-global-color-secondary, #1F1C17);
		}}

		.np-elementor-form-grid .np-form-input {{
			width: 100%;
			padding: 12px 16px;
			border: 1px solid #d4d4d8;
			border-radius: 4px;
			background: #ffffff;
			color: #1F1C17;
			font-family: inherit;
			font-size: 15px;
		}}

		.np-elementor-form-grid .np-Subgroup {{
			display: flex;
			flex-direction: column;
			gap: 8px;
			padding: 6px 0;
		}}

		.np-elementor-form-grid .np-Subgroup.np-Subgroup-inline {{
			flex-direction: row;
			flex-wrap: wrap;
			gap: 18px;
		}}

		.np-elementor-form-grid .np-checkbox-label {{
			display: inline-flex;
			align-items: center;
			gap: 8px;
			font-size: 15px;
			cursor: pointer;
		}}

		.np-popup-modal, .np-lightbox-modal {{
			position: fixed;
			inset: 0;
			z-index: 99999;
			display: flex;
			align-items: center;
			justify-content: center;
			padding: 24px;
		}}

		.np-popup-backdrop, .np-lightbox-backdrop {{
			position: absolute;
			inset: 0;
			background: rgba(0, 0, 0, 0.72);
			backdrop-filter: blur(3px);
		}}

		.np-popup-dialog {{
			position: relative;
			z-index: 1;
			background: #ffffff;
			border-radius: 8px;
			max-width: 680px;
			width: 100%;
			max-height: 90vh;
			overflow-y: auto;
			box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
			padding: 32px;
		}}

		.np-popup-close, .np-lightbox-close {{
			position: absolute;
			top: 12px;
			right: 16px;
			z-index: 2;
			background: transparent;
			border: none;
			font-size: 28px;
			line-height: 1;
			color: #1F1C17;
			cursor: pointer;
		}}

		.np-lightbox-close {{
			color: #ffffff;
			font-size: 36px;
			top: 20px;
			right: 28px;
		}}

		.np-lightbox-content {{
			position: relative;
			z-index: 1;
			max-width: 92vw;
			max-height: 88vh;
		}}

		.np-lightbox-content img {{
			max-width: 92vw;
			max-height: 88vh;
			object-fit: contain;
			border-radius: 4px;
			box-shadow: 0 20px 50px rgba(0,0,0,0.5);
			display: block;
		}}

		@media (min-width: 1025px) {{
			.np-hide-desktop {{
				display: none !important;
			}}
		}}

		@media (min-width: 768px) and (max-width: 1024px) {{
			.np-hide-tablet {{
				display: none !important;
			}}
		}}

		@media (max-width: 767px) {{
			.np-hide-mobile {{
				display: none !important;
			}}
			.np-elementor-form-grid .np-field-w50 {{
				width: 100%;
			}}
			.np-container[style*="flex-direction: row"],
			.np-container[style*="flex-direction: row"] > .np-container-inner {{
				flex-wrap: wrap;
			}}
			.np-container[style*="flex-direction: row"] > .np-container,
			.np-container[style*="flex-direction: row"] > .np-container-inner > .np-container {{
				width: 100% !important;
				flex: 1 1 100% !important;
			}}
		}}

		@media (max-width: 900px) {{
			.footer-grid, .np-grid {{
				grid-template-columns: 1fr !important;
			}}
			nav.neo-nav {{
				display: none;
			}}
		}}
	</style>{elementor_kit_css_block}{custom_css_block}
</head>
<body class="{body_class_admin}">
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
	<div id="np-lightbox-modal" class="np-lightbox-modal" style="display:none;" role="dialog" aria-modal="true">
		<div class="np-lightbox-backdrop" onclick="this.parentElement.style.display='none'"></div>
		<button type="button" class="np-lightbox-close" aria-label="Close" onclick="document.getElementById('np-lightbox-modal').style.display='none'">&times;</button>
		<div class="np-lightbox-content">
			<img id="np-lightbox-img" src="" alt="" />
		</div>
	</div>
	<script>
	(function(){{
		document.addEventListener('click', function(e) {{
			var popupLink = e.target.closest('a[href^="#np-popup-"]');
			if (popupLink) {{
				e.preventDefault();
				var targetId = popupLink.getAttribute('href').slice(1);
				var modal = document.getElementById(targetId) || document.querySelector('.np-popup-modal');
				if (modal) {{
					modal.style.display = 'flex';
				}}
				return;
			}}
			var lbLink = e.target.closest('a.np-lightbox-trigger');
			if (lbLink) {{
				e.preventDefault();
				var imgUrl = lbLink.getAttribute('href');
				var lbModal = document.getElementById('np-lightbox-modal');
				var lbImg = document.getElementById('np-lightbox-img');
				if (lbModal && lbImg && imgUrl) {{
					lbImg.src = imgUrl;
					lbModal.style.display = 'flex';
				}}
			}}
		}});
		document.addEventListener('keydown', function(e) {{
			if (e.key === 'Escape') {{
				document.querySelectorAll('.np-popup-modal, .np-lightbox-modal').forEach(function(m) {{
					m.style.display = 'none';
				}});
			}}
		}});
	}})();
	</script>
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
        brand_color = escape_html(brand_color),
        elementor_kit_css_block = elementor_kit_css_block,
        custom_css_block = custom_css_block,
        site_header_html = site_header_html,
        site_footer_html = site_footer_html,
        elementor_popups_html = settings.elementor_popups_html,
        content = content,
        body_class_admin = body_class_admin,
        admin_bar_html = admin_bar_html
    )
}

/// Renders the dynamic Front Page / Blog Index 100% using Neopress AST elements (`neopress_ast`)
/// and live database content (`settings`, `posts`). Zero hardcoded marketing copy.
pub fn render_home_page(
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
) -> String {
    let site_title = if settings.title.trim().is_empty() {
        "Neopress"
    } else {
        settings.title.trim()
    };
    let site_tagline = if settings.description.trim().is_empty() {
        "Published with Neopress Native Rust CMS"
    } else {
        settings.description.trim()
    };

    // 1. Construct the Hero Section as a genuine NeopressLayout AST and compile it to HTML
    let first_post = posts.iter().find(|p| p.post_type == "post").or_else(|| posts.first());
    let cta_url = first_post
        .map(|p| format!("/{}/", p.slug))
        .unwrap_or_else(|| "#posts".to_string());
    let cta_label = first_post
        .map(|p| format!("Read Latest: {}", p.title.rendered))
        .unwrap_or_else(|| "Browse Articles".to_string());
    let hero_img_url = posts
        .iter()
        .find_map(|p| p.featured_image_url.clone())
        .unwrap_or_default();

    let mut hero_left_children = vec![
        NeopressNode {
            id: "np_home_hero_title".to_string(),
            kind: NodeKind::Heading {
                text: escape_html(site_title),
                level: 1,
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
        NeopressNode {
            id: "np_home_hero_desc".to_string(),
            kind: NodeKind::Text {
                content_html: format!("<p>{}</p>", escape_html(site_tagline)),
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
        NeopressNode {
            id: "np_home_hero_cta".to_string(),
            kind: NodeKind::Button {
                text: escape_html(&cta_label),
                url: cta_url,
                target: "_self".to_string(),
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
    ];

    let hero_grid_node = if !hero_img_url.is_empty() {
        NeopressNode {
            id: "np_home_hero_grid".to_string(),
            kind: NodeKind::Grid {
                columns: 2,
                gap: 40,
            },
            styles: StyleProperties::default(),
            children: vec![
                NeopressNode {
                    id: "np_home_hero_col_left".to_string(),
                    kind: NodeKind::Column,
                    styles: StyleProperties::default(),
                    children: hero_left_children,
                },
                NeopressNode {
                    id: "np_home_hero_col_right".to_string(),
                    kind: NodeKind::Column,
                    styles: StyleProperties::default(),
                    children: vec![NeopressNode {
                        id: "np_home_hero_image".to_string(),
                        kind: NodeKind::Image {
                            src: hero_img_url,
                            alt: site_title.to_string(),
                            width: None,
                            height: None,
                            link: None,
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    }],
                },
            ],
        }
    } else {
        NeopressNode {
            id: "np_home_hero_col_single".to_string(),
            kind: NodeKind::Column,
            styles: StyleProperties::default(),
            children: std::mem::take(&mut hero_left_children),
        }
    };

    let hero_layout = NeopressLayout {
        version: 1,
        global_styles: None,
        root: NeopressNode {
            id: "np_home_hero_section".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: vec![NeopressNode {
                id: "np_home_hero_container".to_string(),
                kind: NodeKind::Container,
                styles: StyleProperties::default(),
                children: vec![hero_grid_node],
            }],
        },
    };
    let hero_html = neopress_ast::render_layout_to_html(&hero_layout);

    // 2. Render all published posts using Neopress Element Cards (.np-section -> .np-container -> .np-grid -> .np-card)
    let max_posts = settings.posts_per_page.max(1) as usize;
    let mut post_cards_html = String::new();
    for post in posts
        .iter()
        .filter(|p| p.post_type == "post" || p.post_type.is_empty())
        .take(max_posts)
    {
        let excerpt = if !post.excerpt.rendered.trim().is_empty() {
            plain_text_excerpt(&post.excerpt.rendered, 140)
        } else {
            plain_text_excerpt(&post.content.rendered, 140)
        };
        let date_short: String = post.date.chars().take(10).collect();
        let cat_label = post
            .category_names
            .first()
            .map(|c| escape_html(c))
            .unwrap_or_else(|| "Article".to_string());
        let author_label = if post.author_name.trim().is_empty() {
            "admin".to_string()
        } else {
            escape_html(&post.author_name)
        };

        let media_html = if let Some(ref img_url) = post.featured_image_url {
            if !img_url.is_empty() {
                format!(
                    r#"<div class="np-card-media"><img src="{}" alt="{}" loading="lazy" /></div>"#,
                    escape_html(img_url),
                    escape_html(&post.title.rendered)
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        post_cards_html.push_str(&format!(
            r##"
            <a href="/{slug}/" class="np-card">
                {media_html}
                <div class="np-card-body">
                    <div class="np-card-meta">
                        <span class="np-badge">{cat_label}</span>
                        <span>&bull;</span>
                        <span>{date}</span>
                        <span>&bull;</span>
                        <span>{author}</span>
                    </div>
                    <h3 class="np-card-title">{title}</h3>
                    <p class="np-card-excerpt">{excerpt}</p>
                    <div class="np-card-link">Read article &rarr;</div>
                </div>
            </a>
            "##,
            slug = escape_html(&post.slug),
            media_html = media_html,
            cat_label = cat_label,
            date = escape_html(&date_short),
            author = author_label,
            title = post.title.rendered,
            excerpt = excerpt,
        ));
    }

    // 3. Render site navigation / pages directory section using Neopress Elements if menu items exist
    let pages_section_html = if !settings.nav_menu_items.is_empty() {
        let mut page_cards = String::new();
        for item in settings
            .nav_menu_items
            .iter()
            .filter(|i| i.url != "/" && !i.title.eq_ignore_ascii_case("Home"))
        {
            page_cards.push_str(&format!(
                r##"
                <a href="{url}" class="np-iconbox">
                    <div class="np-iconbox-icon"><i class="fa-solid fa-file-lines"></i></div>
                    <h3 class="np-iconbox-title">{title}</h3>
                    <p class="np-iconbox-desc">Explore {title} &rarr;</p>
                </a>
                "##,
                url = escape_html(&item.url),
                title = escape_html(&item.title),
            ));
        }
        if !page_cards.is_empty() {
            format!(
                r##"
                <section id="np-site-pages" class="np-section">
                    <div class="np-container">
                        <h2 class="np-heading" style="margin-bottom: 24px;">Site Pages</h2>
                        <div class="np-grid" style="grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));">
                            {page_cards}
                        </div>
                    </div>
                </section>
                "##,
                page_cards = page_cards
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let home_content = format!(
        r##"
        {hero_html}
        <section id="posts" class="np-section">
            <div class="np-container">
                <h2 class="np-heading" style="margin-bottom: 28px;">Latest Articles</h2>
                <div class="np-grid" style="grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));">
                    {post_cards_html}
                </div>
            </div>
        </section>
        {pages_section_html}
        "##,
        hero_html = hero_html,
        post_cards_html = post_cards_html,
        pages_section_html = pages_section_html,
    );

    render_base_layout(&settings.title, settings, &home_content, current_user, None)
}

/// Renders individual articles and pages using Neopress AST elements and database metadata.
pub fn render_single_post(
    post: &WpPostResponse,
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
    layout_json: Option<&str>,
) -> String {
    let ast_rendered = layout_json
        .and_then(|s| serde_json::from_str::<NeopressLayout>(s).ok())
        .filter(|l| !l.root.children.is_empty())
        .map(|l| neopress_ast::render_layout_to_html(&l))
        .filter(|h| !h.trim().is_empty());

    let has_ast_layout = ast_rendered.is_some();
    let clean_content = ast_rendered.unwrap_or_else(|| clean_wp_content(&post.content.rendered));
    let is_page = post.post_type == "page";
    let site_name = if settings.title.trim().is_empty() {
        "Home"
    } else {
        settings.title.trim()
    };

    let fallback_kit_css = if !settings.elementor_kit_css.trim().is_empty() {
        String::new()
    } else {
        neopress_ast::ElementorKit::default().compile_to_css()
    };

    let global_css = if let Some(layout_str) = layout_json {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(layout_str) {
            if let Some(gs) = val.get("global_styles") {
                if !gs.is_null() {
                    if let Ok(kit) = serde_json::from_value::<neopress_ast::ElementorKit>(gs.clone()) {
                        kit.compile_to_css()
                    } else {
                        fallback_kit_css
                    }
                } else {
                    fallback_kit_css
                }
            } else {
                fallback_kit_css
            }
        } else {
            fallback_kit_css
        }
    } else {
        fallback_kit_css
    };

    // Full-bleed Visual Builder page vs standard article/page with header shell
    let is_full_bleed_visual = has_ast_layout
        || (is_page
            && (clean_content.contains("elem_hero_sec")
                || clean_content.contains("divi_section_")
                || clean_content.contains("divi_node_")
                || clean_content.contains("elementor-section")
                || post.template == "canvas"
                || post.template == "full-width"));

    let has_elementor_theme = !settings.elementor_kit_css.trim().is_empty()
        || !settings.elementor_header_html.trim().is_empty()
        || !settings.elementor_footer_html.trim().is_empty();

    let content = if is_full_bleed_visual {
        format!(
            r##"
            <style>{global_css}</style>
            <div class="np-visual-canvas">
                {body}
            </div>
            "##,
            global_css = global_css,
            body = clean_content
        )
    } else if is_page && has_elementor_theme {
        format!(
            r##"
            <style>{global_css}</style>
            <section class="np-section" style="border-bottom: none; padding: 48px 24px;">
                <div class="np-container" style="max-width: var(--e-content-width, 1140px); margin: 0 auto;">
                    <h1 class="np-heading entry-title" style="margin-bottom: 24px;">{title}</h1>
                    <div class="np-text entry-content">
                        {body}
                    </div>
                </div>
            </section>
            "##,
            global_css = global_css,
            title = post.title.rendered,
            body = clean_content
        )
    } else {
        let author_display = if post.author_name.is_empty() {
            "admin".to_string()
        } else {
            escape_html(&post.author_name)
        };

        let categories_html = if !post.category_names.is_empty() {
            let badges: Vec<String> = post
                .category_names
                .iter()
                .map(|c| {
                    let slug = generated_slugify::slugify(c);
                    format!(
                        r#"<a href="/category/{}" class="np-badge">{}</a>"#,
                        escape_html(&slug),
                        escape_html(c)
                    )
                })
                .collect();
            format!("<span>&bull;</span>{}", badges.join(" "))
        } else {
            String::new()
        };

        let featured_img_html = if let Some(ref img_url) = post.featured_image_url {
            if !img_url.is_empty() && !clean_content.contains(img_url.as_str()) {
                format!(
                    r#"<div style="margin-bottom: 28px; border-radius: 12px; overflow: hidden; border: 1px solid var(--border);"><img src="{url}" alt="{alt}" class="np-image" style="width: 100%; max-height: 460px; object-fit: cover;" /></div>"#,
                    url = escape_html(img_url),
                    alt = escape_html(&post.title.rendered),
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let tags_html = if !post.tag_names.is_empty() {
            let pills: Vec<String> = post
                .tag_names
                .iter()
                .map(|t| {
                    let slug = generated_slugify::slugify(t);
                    format!(
                        r#"<a href="/tag/{}" style="background: var(--bg-card); border: 1px solid var(--border); color: var(--text-muted); padding: 4px 12px; border-radius: 9999px; font-size: 12px;">#{}</a>"#,
                        escape_html(&slug),
                        escape_html(t)
                    )
                })
                .collect();
            format!(
                r#"<div style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: 28px;">{}</div>"#,
                pills.join("")
            )
        } else {
            String::new()
        };

        let comments_section_html = if !is_page {
            let mut comment_items = String::new();
            if post.comments.is_empty() {
                comment_items.push_str(
                    r#"<p id="no-comments-placeholder" style="color: var(--text-dim); font-size: 14px; margin-bottom: 20px;">No comments yet. Be the first to join the discussion!</p>"#,
                );
            } else {
                for c in &post.comments {
                    let c_author = if c.author_name.is_empty() {
                        "Anonymous".to_string()
                    } else {
                        escape_html(&c.author_name)
                    };
                    let c_date: String = c.date.chars().take(10).collect();
                    let c_body = clean_wp_content(&c.content.rendered);
                    let indent_style = if c.parent > 0 {
                        "margin-left: 32px; border-left: 3px solid #2563eb;"
                    } else {
                        ""
                    };
                    comment_items.push_str(&format!(
                        r##"<div class="comment-card" style="background: var(--bg-card); border: 1px solid var(--border); border-radius: 8px; padding: 18px 20px; margin-bottom: 14px; {indent}">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                                <strong style="color: #ffffff; font-size: 14px;">{author}</strong>
                                <span style="color: var(--text-dim); font-size: 12px; font-family: var(--font-mono);">{date}</span>
                            </div>
                            <div style="color: #d4d4d8; font-size: 14.5px; line-height: 1.6;">{body}</div>
                        </div>"##,
                        indent = indent_style,
                        author = c_author,
                        date = escape_html(&c_date),
                        body = c_body,
                    ));
                }
            }

            format!(
                r##"<section id="comments" style="margin-top: 56px; padding-top: 36px; border-top: 1px solid var(--border);">
                    <h3 class="np-heading" style="font-size: 22px; margin-bottom: 20px;">Comments ({count})</h3>
                    <div id="comments-list">
                        {comment_items}
                    </div>
                    <div style="background: var(--bg-card); border: 1px solid var(--border); border-radius: 10px; padding: 24px; margin-top: 24px;">
                        <h4 style="font-size: 16px; font-weight: 700; color: #ffffff; margin-bottom: 16px;">Leave a Comment</h4>
                        <form id="comment-form" onsubmit="submitPublicComment(event)" style="display: flex; flex-direction: column; gap: 14px;">
                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;">
                                <input type="text" id="comment-author" placeholder="Your Name *" required style="background: #09090b; border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: #ffffff; font-size: 14px;" />
                                <input type="email" id="comment-email" placeholder="Your Email *" required style="background: #09090b; border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: #ffffff; font-size: 14px;" />
                            </div>
                            <textarea id="comment-content" rows="4" placeholder="Share your thoughts..." required style="background: #09090b; border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: #ffffff; font-size: 14px; font-family: inherit;"></textarea>
                            <div>
                                <button type="submit" id="comment-submit-btn" class="np-button">Post Comment</button>
                            </div>
                        </form>
                    </div>
                </section>
                <script>
                async function submitPublicComment(e) {{
                    e.preventDefault();
                    const btn = document.getElementById('comment-submit-btn');
                    const author_name = document.getElementById('comment-author').value.trim();
                    const author_email = document.getElementById('comment-email').value.trim();
                    const content = document.getElementById('comment-content').value.trim();
                    if (!author_name || !content) return;
                    btn.disabled = true;
                    btn.textContent = 'Posting...';
                    try {{
                        const res = await fetch('/wp-json/wp/v2/comments', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ post: {post_id}, author_name, author_email, content }})
                        }});
                        if (res.ok) {{
                            window.location.reload();
                        }} else {{
                            btn.disabled = false;
                            btn.textContent = 'Post Comment';
                        }}
                    }} catch (err) {{
                        btn.disabled = false;
                        btn.textContent = 'Post Comment';
                    }}
                }}
                </script>"##,
                count = post.comments.len(),
                comment_items = comment_items,
                post_id = post.id,
            )
        } else {
            String::new()
        };

        let date_short: String = post.date.chars().take(10).collect();

        format!(
            r##"
            <style>{global_css}</style>
            <section class="np-section">
                <div class="np-container" style="max-width: 880px;">
                    <div style="margin-bottom: 32px; border-bottom: 1px solid var(--border); padding-bottom: 24px;">
                        <a href="/" style="font-size: 13px; font-weight: 600; color: #60a5fa; display: inline-block; margin-bottom: 16px;">&larr; {site_name}</a>
                        <div class="np-card-meta">
                            <span class="np-badge">{post_type}</span>
                            <span>&bull;</span>
                            <time datetime="{date}">{date}</time>
                            <span>&bull;</span>
                            <span>{author_display}</span>
                            {categories_html}
                        </div>
                        <h1 class="np-heading">{title}</h1>
                    </div>

                    {featured_img_html}

                    <div class="np-text">
                        {body}
                    </div>

                    {tags_html}

                    {comments_section_html}
                </div>
            </section>
            "##,
            global_css = global_css,
            site_name = escape_html(site_name),
            title = post.title.rendered,
            date = date_short,
            post_type = escape_html(&post.post_type),
            author_display = author_display,
            categories_html = categories_html,
            featured_img_html = featured_img_html,
            tags_html = tags_html,
            comments_section_html = comments_section_html,
            body = clean_content
        )
    };

    render_base_layout(
        &post.title.rendered,
        settings,
        &content,
        current_user,
        Some(post),
    )
}

pub fn render_archive_page(
    archive_title: &str,
    archive_subtitle: &str,
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
) -> String {
    let mut cards_html = String::new();
    if posts.is_empty() {
        cards_html.push_str(
            r#"<div style="grid-column: 1 / -1; text-align: center; padding: 60px 24px; background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; color: var(--text-muted);">No matching articles or pages found.</div>"#,
        );
    } else {
        for post in posts {
            let excerpt = plain_text_excerpt(&post.content.rendered, 140);
            let date_short: String = post.date.chars().take(10).collect();
            let author_display = if post.author_name.is_empty() {
                "admin".to_string()
            } else {
                escape_html(&post.author_name)
            };

            cards_html.push_str(&format!(
                r##"
                <a href="/{slug}/" class="np-card">
                    <div class="np-card-body">
                        <div class="np-card-meta">
                            <span class="np-badge">{post_type}</span>
                            <span>&bull;</span>
                            <span>{date}</span>
                            <span>&bull;</span>
                            <span>{author}</span>
                        </div>
                        <h3 class="np-card-title">{title}</h3>
                        <p class="np-card-excerpt">{excerpt}</p>
                        <div class="np-card-link">Read more &rarr;</div>
                    </div>
                </a>
                "##,
                slug = escape_html(&post.slug),
                post_type = escape_html(&post.post_type),
                date = escape_html(&date_short),
                author = author_display,
                title = post.title.rendered,
                excerpt = excerpt,
            ));
        }
    }

    let body = format!(
        r##"
        <section class="np-section">
            <div class="np-container">
                <div style="max-width: 760px; margin-bottom: 40px;">
                    <span class="np-badge" style="margin-bottom: 12px; display: inline-block;">{subtitle}</span>
                    <h1 class="np-heading">{title}</h1>
                </div>
                <div class="np-grid" style="grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));">
                    {cards_html}
                </div>
            </div>
        </section>
        "##,
        subtitle = escape_html(archive_subtitle),
        title = escape_html(archive_title),
        cards_html = cards_html,
    );

    render_base_layout(archive_title, settings, &body, current_user, None)
}

pub fn render_search_page(
    query: &str,
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
) -> String {
    render_archive_page(
        &format!("Search Results for: {}", query),
        "Search Archive",
        posts,
        settings,
        current_user,
    )
}

pub fn render_not_found_page(
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
) -> String {
    let body = r##"
        <section class="np-section">
            <div class="np-container" style="max-width: 720px; text-align: center;">
                <span class="np-badge" style="margin-bottom: 14px; display: inline-block;">HTTP 404</span>
                <h1 class="np-heading">404 — Page Not Found</h1>
                <p class="np-text" style="margin-bottom: 24px;">The requested resource could not be found.</p>
                <a href="/" class="np-button">Return Home</a>
            </div>
        </section>
    "##;

    render_base_layout("404 Not Found", settings, body, current_user, None)
}
