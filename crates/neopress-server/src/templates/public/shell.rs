//! Public Base HTML Document Shell (`templates/public/shell.rs`).
//!
//! Renders SEO meta tags, OpenGraph cards, Schema.org JSON-LD structured data,
//! Cookieless Analytics beacon, Admin Bar, Header, Footer, and Lightbox/Popup handlers.

use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::{clean_wp_content, escape_html, escape_json};

/// Strips HTML tags cleanly and truncates on UTF-8 character boundaries.
pub(crate) fn plain_text_excerpt(html: &str, max_chars: usize) -> String {
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

    let site_header_html = if is_blank_template {
        String::new()
    } else if !settings.elementor_header_html.trim().is_empty() {
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

    let site_footer_html = if is_blank_template {
        String::new()
    } else if !settings.elementor_footer_html.trim().is_empty() {
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
		{public_theme_css}
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
	<div id="np-lightbox-modal" class="np-lightbox-modal" role="dialog" aria-modal="true">
		<div class="np-lightbox-backdrop" onclick="this.parentElement.classList.remove('active')"></div>
		<button type="button" class="np-lightbox-close" aria-label="Close" onclick="document.getElementById('np-lightbox-modal').classList.remove('active')">&times;</button>
		<div class="np-lightbox-content">
			<img id="np-lightbox-img" src="" alt="" />
		</div>
	</div>
	<script>
	(function(){{
		document.addEventListener('click', function(e) {{
			var diviPopupLink = e.target.closest('a[href^="#"]');
			if (diviPopupLink) {{
				var hash = diviPopupLink.getAttribute('href');
				if (hash && hash.length > 1 && !hash.startsWith('#np-popup-')) {{
					var targetEl = document.getElementById(hash.slice(1));
					if (targetEl && targetEl.classList.contains('popup')) {{
						e.preventDefault();
						if (!targetEl.querySelector('.np-divi-popup-close')) {{
							var closeBtn = document.createElement('button');
							closeBtn.type = 'button';
							closeBtn.className = 'np-divi-popup-close';
							closeBtn.setAttribute('aria-label', 'Close');
							closeBtn.innerHTML = '&times;';
							closeBtn.onclick = function() {{ targetEl.classList.remove('is-open'); }};
							targetEl.appendChild(closeBtn);
						}}
						targetEl.classList.toggle('is-open');
						return;
					}}
				}}
			}}
			var popupLink = e.target.closest('a[href^="#np-popup-"]');
			if (popupLink) {{
				e.preventDefault();
				var targetId = popupLink.getAttribute('href').slice(1);
				var modal = document.getElementById(targetId) || document.querySelector('.np-popup-modal');
				if (modal) {{
					modal.classList.add('active');
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
					lbModal.classList.add('active');
				}}
			}}
		}});
		document.addEventListener('keydown', function(e) {{
			if (e.key === 'Escape') {{
				document.querySelectorAll('.np-popup-modal, .np-lightbox-modal').forEach(function(m) {{
					m.classList.remove('active');
				}});
				document.querySelectorAll('.et_pb_section.popup.is-open').forEach(function(m) {{
					m.classList.remove('is-open');
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
        public_theme_css = super::styles::render_public_theme_css(&escape_html(brand_color)),
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
