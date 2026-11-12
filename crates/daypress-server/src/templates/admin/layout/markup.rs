//! Admin Shell HTML Markup and template rendering.

use crate::models::DaypressSettingsResponse;
use crate::templates::admin::layout::navigation::{render_cpt_navigation_full, NavState};
use crate::templates::admin::layout::styles_components::get_styles_components;
use crate::templates::admin::layout::styles_forms::get_styles_forms;
use crate::templates::admin::layout::styles_shell::{
    get_styles_shell, DESK_FONT_FACES, DESK_FOUNDATION,
};
use crate::templates::utils::escape_html;

pub fn render_admin_shell(
    current_tab: &str,
    page_title: &str,
    body_content: &str,
    settings: &DaypressSettingsResponse,
) -> String {
    let site_name = escape_html(if settings.title.is_empty() {
        "Daypress"
    } else {
        &settings.title
    });
    let account_name = escape_html(
        &crate::auth::current_admin()
            .map(|a| a.display_name)
            .unwrap_or_else(|| "admin".to_string()),
    );
    let pending_comments_num = settings.pending_comments_count;
    let comments_badge = if settings.pending_comments_count > 0 {
        format!(
            r#"<span class="nav-badge">{}</span>"#,
            settings.pending_comments_count
        )
    } else {
        String::new()
    };
    let updates_badge = if let Some(ref ver) = settings.update_available {
        format!(
            r#"<span class="nav-badge" title="Daypress v{} available">1</span>"#,
            escape_html(ver)
        )
    } else {
        String::new()
    };

    let nav = NavState::compute(current_tab);
    let custom_post_types_nav = render_cpt_navigation_full(current_tab, settings);
    let styles_shell = get_styles_shell();
    let styles_comp = get_styles_components();
    let admin_color = escape_html(
        &crate::auth::current_admin()
            .map(|a| a.admin_color)
            .unwrap_or_else(|| "default".to_string()),
    );
    let html_lang = escape_html(
        &crate::auth::current_admin()
            .map(|a| a.locale.trim().replace('_', "-"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "en-US".to_string()),
    );
    let tr = |key: &'static str| crate::i18n::t(&html_lang, key);
    let styles_forms = get_styles_forms();
    let rest_nonce: String = crate::auth::current_admin()
        .map(|a| a.nonce)
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    let has_plugin = |slug: &str| settings.active_plugins.iter().any(|p| p == slug);

    let nav_sub_wasm = format!(
        r#"<a href="/d-admin/wasm" class="nav-subitem {}">Wasm Extensions (.wasm)</a>"#,
        nav.is_wasm
    );

    let nav_sub_forms = if has_plugin("daypress-forms") || current_tab == "forms" {
        format!(
            r#"<a href="/d-admin/forms" class="nav-subitem {}">{}</a>"#,
            nav.is_forms,
            tr("nav.forms")
        )
    } else {
        String::new()
    };

    let nav_sub_newsletters = if has_plugin("daypress-newsletters") || current_tab == "newsletters"
    {
        format!(
            r#"<a href="/d-admin/newsletters" class="nav-subitem {}">Newsletters</a>"#,
            nav.is_newsletters
        )
    } else {
        String::new()
    };

    let nav_sub_whatsapp = if has_plugin("whatsapp-chat") || current_tab == "whatsapp-chat" {
        format!(
            r#"<a href="/d-admin/plugin-settings?plugin=whatsapp-chat" class="nav-subitem {}">WhatsApp Chat</a>"#,
            nav.is_whatsapp_chat
        )
    } else {
        String::new()
    };

    let nav_sub_webhooks = if has_plugin("daypress-webhooks")
        || current_tab == "webhooks"
        || current_tab == "shortlinks"
    {
        format!(
            r#"<a href="/d-admin/webhooks" class="nav-subitem {}">Shortlinks &amp; Webhooks</a>"#,
            nav.is_webhooks
        )
    } else {
        String::new()
    };

    let nav_sub_ab_testing = if has_plugin("daypress-ab-testing") || current_tab == "ab-testing" {
        format!(
            r#"<a href="/d-admin/ab-testing" class="nav-subitem {}">A/B Testing &amp; Vitals</a>"#,
            nav.is_ab_testing
        )
    } else {
        String::new()
    };

    let nav_sub_code_snippets = if has_plugin("code-snippets") || current_tab == "code-snippets" {
        format!(
            r#"<a href="/d-admin/plugin-settings?plugin=code-snippets" class="nav-subitem {}">Header &amp; Footer Snippets</a>"#,
            nav.is_code_snippets
        )
    } else {
        String::new()
    };

    let nav_sub_seo = format!(
        r#"<a href="/d-admin/seo" class="nav-subitem {}">SEO &amp; Redirects</a>"#,
        nav.is_seo
    );

    let nav_sub_backups = format!(
        r#"<a href="/d-admin/backups" class="nav-subitem {}">Backups</a>"#,
        nav.is_backups
    );

    let mut qs_plugin_screens = String::new();
    qs_plugin_screens.push_str("['Wasm Extensions (.wasm)', '/d-admin/wasm'], ");
    qs_plugin_screens.push_str("['SEO & Redirects', '/d-admin/seo'], ");
    qs_plugin_screens.push_str("['Backups', '/d-admin/backups'], ");
    if has_plugin("daypress-forms") {
        qs_plugin_screens.push_str("['Forms Studio', '/d-admin/forms'], ");
    }
    if has_plugin("daypress-newsletters") {
        qs_plugin_screens.push_str("['Newsletters & Member Gate', '/d-admin/newsletters'], ");
    }
    if has_plugin("whatsapp-chat") {
        qs_plugin_screens
            .push_str("['WhatsApp Chat', '/d-admin/plugin-settings?plugin=whatsapp-chat'], ");
    }
    if has_plugin("code-snippets") {
        qs_plugin_screens.push_str(
            "['Header & Footer Snippets', '/d-admin/plugin-settings?plugin=code-snippets'], ",
        );
    }
    if has_plugin("daypress-webhooks") {
        qs_plugin_screens.push_str("['Shortlinks & Webhooks', '/d-admin/webhooks'], ");
    }
    if has_plugin("daypress-ab-testing") {
        qs_plugin_screens.push_str("['A/B Testing & Vitals', '/d-admin/ab-testing'], ");
    }

    format!(
        r##"<!DOCTYPE html>
<html lang="{html_lang}" data-admin-color="{admin_color}">
<head>
	<meta charset="UTF-8" />
	<title>{page_title} &bull; {site_name}</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="icon" type="image/svg+xml" href="/static/favicon.svg" />
	<style>
{DESK_FONT_FACES}
{DESK_FOUNDATION}
{styles_shell}
{styles_comp}
{styles_forms}
/* Daypress Admin Color Schemes (matching Daylight & Ink architecture: distinct desk, clean paper canvas, --oxide buttons/active, --press links) */
:root[data-admin-color="linen"], body[data-admin-color="linen"],
:root[data-admin-color="light"], body[data-admin-color="light"] {{
	--desk: #e5e0d5; --desk-raised: #d8d1c3; --desk-ink: #2b2621; --desk-muted: #5c544b; --desk-rule: rgba(43,38,33,0.12); --desk-oxide: #b04a25;
	--paper: #f5f2eb; --paper-raised: #ffffff; --paper-sunk: #ece7dc; --ink: #221f1c; --ink-muted: #5e574f; --rule: #dcd5c7; --rule-strong: #c4bba9;
	--oxide: #b04a25; --on-oxide: #ffffff; --primary: #b04a25; --primary-hover: #8c3718; --oxide-wash: #fbf0ec;
	--press: #226096; --press-hover: #164670; --press-wash: #eef5fb; --focus: #226096; --moss: #3d7047;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
:root[data-admin-color="studio"], body[data-admin-color="studio"],
:root[data-admin-color="modern"], body[data-admin-color="modern"] {{
	--desk: #1e1f26; --desk-raised: #2b2d38; --desk-ink: #f0f1f5; --desk-muted: #b4b7c5; --desk-rule: rgba(255,255,255,0.08); --desk-oxide: #7b86ff;
	--paper: #f1f2f6; --paper-raised: #ffffff; --paper-sunk: #e6e8f0; --ink: #1b1e26; --ink-muted: #555b6e; --rule: #d8dbe6; --rule-strong: #c0c4d4;
	--oxide: #3858e9; --on-oxide: #ffffff; --primary: #3858e9; --primary-hover: #2145e6; --oxide-wash: #eef1fe;
	--press: #3858e9; --press-hover: #1d39b8; --press-wash: #eef1fe; --focus: #3858e9; --moss: #2d7d5e;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
:root[data-admin-color="nordic"], body[data-admin-color="nordic"],
:root[data-admin-color="blue"], body[data-admin-color="blue"] {{
	--desk: #1e2d3b; --desk-raised: #293d50; --desk-ink: #eef3f7; --desk-muted: #b0c2d1; --desk-rule: rgba(255,255,255,0.08); --desk-oxide: #66b3e6;
	--paper: #f0f4f7; --paper-raised: #ffffff; --paper-sunk: #e4ebf0; --ink: #1c262e; --ink-muted: #526370; --rule: #d5e0e8; --rule-strong: #bccad6;
	--oxide: #2b6cb0; --on-oxide: #ffffff; --primary: #2b6cb0; --primary-hover: #1e4e8c; --oxide-wash: #ebf4fc;
	--press: #1d5f96; --press-hover: #12436b; --press-wash: #ebf4fc; --focus: #1d5f96; --moss: #36705a;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
:root[data-admin-color="espresso"], body[data-admin-color="espresso"],
:root[data-admin-color="coffee"], body[data-admin-color="coffee"] {{
	--desk: #362f2b; --desk-raised: #463d38; --desk-ink: #f4eee9; --desk-muted: #c2b6ad; --desk-rule: rgba(255,255,255,0.08); --desk-oxide: #dda175;
	--paper: #f3efe9; --paper-raised: #ffffff; --paper-sunk: #e8e1d7; --ink: #26201c; --ink-muted: #61554d; --rule: #dcd3c8; --rule-strong: #c5b9ab;
	--oxide: #9e5a34; --on-oxide: #ffffff; --primary: #9e5a34; --primary-hover: #7d4525; --oxide-wash: #fbf1eb;
	--press: #286090; --press-hover: #1b4569; --press-wash: #eef5fb; --focus: #286090; --moss: #4d6e46;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
:root[data-admin-color="sage"], body[data-admin-color="sage"],
:root[data-admin-color="ectoplasm"], body[data-admin-color="ectoplasm"] {{
	--desk: #233029; --desk-raised: #2f4037; --desk-ink: #edf3ef; --desk-muted: #b0c2b8; --desk-rule: rgba(255,255,255,0.08); --desk-oxide: #82c49b;
	--paper: #f1f4f1; --paper-raised: #ffffff; --paper-sunk: #e4ebe5; --ink: #1c2520; --ink-muted: #526358; --rule: #d5dfd7; --rule-strong: #bccac0;
	--oxide: #3b7052; --on-oxide: #ffffff; --primary: #3b7052; --primary-hover: #2b563e; --oxide-wash: #ecf6f0;
	--press: #25637c; --press-hover: #18475b; --press-wash: #ecf5f9; --focus: #25637c; --moss: #3b7052;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
:root[data-admin-color="dusk"], body[data-admin-color="dusk"],
:root[data-admin-color="midnight"], body[data-admin-color="midnight"] {{
	--desk: #282433; --desk-raised: #363045; --desk-ink: #f1eef6; --desk-muted: #b8b0c8; --desk-rule: rgba(255,255,255,0.08); --desk-oxide: #e0826d;
	--paper: #f3f1f6; --paper-raised: #ffffff; --paper-sunk: #e7e3ed; --ink: #221e2b; --ink-muted: #5d556b; --rule: #dad5e3; --rule-strong: #c2bbd0;
	--oxide: #7b4685; --on-oxide: #ffffff; --primary: #7b4685; --primary-hover: #5e3366; --oxide-wash: #f6eff8;
	--press: #3558a8; --press-hover: #233e7d; --press-wash: #eef2fb; --focus: #3558a8; --moss: #3f6e55;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
:root[data-admin-color="harbor"], body[data-admin-color="harbor"],
:root[data-admin-color="ocean"], body[data-admin-color="ocean"] {{
	--desk: #26353a; --desk-raised: #33474d; --desk-ink: #eef4f4; --desk-muted: #b0c4c2; --desk-rule: rgba(255,255,255,0.08); --desk-oxide: #7fc4bd;
	--paper: #f0f4f4; --paper-raised: #ffffff; --paper-sunk: #e3eceb; --ink: #1f282b; --ink-muted: #536467; --rule: #d5e0df; --rule-strong: #bccbca;
	--oxide: #356e6b; --on-oxide: #ffffff; --primary: #356e6b; --primary-hover: #265250; --oxide-wash: #ebf5f4;
	--press: #226382; --press-hover: #16465e; --press-wash: #ecf4f8; --focus: #226382; --moss: #3c6e56;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
:root[data-admin-color="terracotta"], body[data-admin-color="terracotta"],
:root[data-admin-color="sunrise"], body[data-admin-color="sunrise"] {{
	--desk: #3b2521; --desk-raised: #4d312c; --desk-ink: #f7edea; --desk-muted: #ccb2aa; --desk-rule: rgba(255,255,255,0.08); --desk-oxide: #ea9066;
	--paper: #f6f0ea; --paper-raised: #ffffff; --paper-sunk: #ebe1d7; --ink: #291d1a; --ink-muted: #66524d; --rule: #dfd3c8; --rule-strong: #c8b8aa;
	--oxide: #b84a32; --on-oxide: #ffffff; --primary: #b84a32; --primary-hover: #913723; --oxide-wash: #fcf0ed;
	--press: #245e9c; --press-hover: #174270; --press-wash: #eef4fb; --focus: #245e9c; --moss: #486e48;
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk);
}}
	</style>
	<script>
		// Standard WordPress REST API settings & session initialization
		window.wpApiSettings = window.wpApiSettings || {{
			root: '/wp-json/',
			nonce: '{rest_nonce}',
			versionString: 'wp/v2/'
		}};
		if (window.fetch && !window.__daypressNonceFetchInstalled) {{
			window.__daypressNonceFetchInstalled = true;
			const _origFetch = window.fetch.bind(window);
			window.fetch = function(input, init) {{
				if (typeof input === 'string' && (input.startsWith('/d-admin/') || input.startsWith('/wp-json/')) && init && init.method) {{
					const m = String(init.method).toUpperCase();
					if (m !== 'GET' && m !== 'HEAD' && m !== 'OPTIONS' && window.wpApiSettings && window.wpApiSettings.nonce) {{
						const hdrs = new Headers(init.headers || undefined);
						if (!hdrs.has('X-WP-Nonce')) {{
							hdrs.set('X-WP-Nonce', window.wpApiSettings.nonce);
						}}
						init = Object.assign({{}}, init, {{ headers: hdrs }});
					}}
				}}
				return _origFetch(input, init);
			}};
		}}
	</script>
</head>
<body data-admin-color="{admin_color}">
<div id="day-app">
	<!-- Full-Width Top Admin Bar (WordPress #wpadminbar) -->
	<header id="day-adminbar" role="region" aria-label="Toolbar">
		<ul class="ab-top-menu">
			<li>
				<button type="button" class="adminbar-menu-btn" onclick="dayToggleMobileNav()" aria-label="Toggle navigation menu">
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="6" x2="21" y2="6"></line><line x1="3" y1="12" x2="21" y2="12"></line><line x1="3" y1="18" x2="21" y2="18"></line></svg>
					<span>Menu</span>
				</button>
			</li>
			<li id="wp-admin-bar-wp-logo" class="menupop">
				<a href="/d-admin/" class="ab-item" aria-haspopup="true" aria-label="About Daypress" title="About Daypress">
					<svg class="ab-brand-icon" viewBox="0 0 1000 1000" width="20" height="20" aria-hidden="true">
						<path fill="currentColor" fill-rule="evenodd" d="M425 182 V156 L575 125 H592 V712 Q592 742 616 748 L660 756 V780 L508 802 L494 720 Q432 798 334 798 C212 798 136 700 136 568 C136 430 222 332 350 332 C418 332 466 360 492 396 V218 Q492 188 458 182 Z M492 466 C470 410 422 376 360 376 C268 376 220 450 220 562 C220 674 276 740 364 740 C422 740 470 704 492 652 Z"/>
						<circle cx="785" cy="734" r="78" fill="var(--desk-oxide)"/>
					</svg>
					<span class="sr-only">Daypress</span>
				</a>
				<div class="ab-sub-wrapper">
					<ul class="ab-submenu">
						<li><a href="/d-admin/getting-started">About Daypress &amp; AI Key</a></li>
						<li><a href="/d-admin/update-core">Updates</a></li>
						<li><a href="/d-admin/site-health">Site Health</a></li>
						<li><a href="/d-admin/daypress-engine">Engine Telemetry</a></li>
					</ul>
				</div>
			</li>
			<li id="wp-admin-bar-site-name" class="menupop">
				<a href="/" target="_blank" rel="noopener" class="ab-item" aria-haspopup="true" title="Visit site">
					<svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
					<span>{site_name}</span>
				</a>
				<div class="ab-sub-wrapper">
					<ul class="ab-submenu">
						<li><a href="/" target="_blank" rel="noopener">Visit Site</a></li>
						<li><a href="/d-admin/customize">Customize</a></li>
						<li><a href="/d-admin/headers-footers">Header &amp; Footer</a></li>
						<li><a href="/d-admin/theme-layouts">Theme Layouts</a></li>
						<li><a href="/d-admin/nav-menus">Menus</a></li>
					</ul>
				</div>
			</li>
			<li id="wp-admin-bar-comments" class="adminbar-hide-mobile">
				<a href="/d-admin/edit-comments" class="ab-item" title="{pending_comments_num} Comments in moderation">
					<svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path></svg>
					<span class="ab-label">{pending_comments_num}</span>
				</a>
			</li>
			<li id="wp-admin-bar-new-content" class="menupop">
				<a href="/d-admin/post-new" class="ab-item" aria-haspopup="true" title="Add New">
					<svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
					<span class="ab-label">New</span>
				</a>
				<div class="ab-sub-wrapper">
					<ul class="ab-submenu">
						<li><a href="/d-admin/post-new">Post</a></li>
						<li><a href="/d-admin/media-new">Media</a></li>
						<li><a href="/d-admin/post-new?post_type=page">Page</a></li>
						<li><a href="/d-admin/forms">Form</a></li>
						<li><a href="/d-admin/user-new">User</a></li>
					</ul>
				</div>
			</li>
			<li id="wp-admin-bar-search-btn" class="adminbar-hide-mobile">
				<button type="button" class="ab-item" onclick="dayOpenQuickSearch()" title="Quick search (Ctrl+K)">
					<svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
					<span class="ab-label">Search</span>
					<kbd class="ab-kbd">Ctrl+K</kbd>
				</button>
			</li>
		</ul>
		<ul id="wp-admin-bar-top-secondary" class="ab-top-secondary">
			<li id="wp-admin-bar-my-account" class="menupop with-avatar">
				<a href="/d-admin/profile" class="ab-item" aria-haspopup="true" title="My Account">
					<span>Howdy, <span class="display-name">{account_name}</span></span>
					<span class="ab-avatar" aria-hidden="true">
						<svg viewBox="0 0 24 24" width="13" height="13" fill="currentColor"><path d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z"/></svg>
					</span>
				</a>
				<div class="ab-sub-wrapper">
					<div class="ab-user-card">
						<div class="ab-user-avatar-lg" aria-hidden="true">
							<svg viewBox="0 0 24 24" width="28" height="28" fill="currentColor"><path d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z"/></svg>
						</div>
						<div class="ab-user-meta">
							<strong>{account_name}</strong>
							<span>Administrator</span>
						</div>
					</div>
					<ul id="wp-admin-bar-user-actions" class="ab-submenu">
						<li id="wp-admin-bar-user-info"><a href="/d-admin/profile">{account_name}</a></li>
						<li id="wp-admin-bar-edit-profile"><a href="/d-admin/profile">Edit Profile</a></li>
						<li id="wp-admin-bar-logout"><a href="/login?action=logout">{nav_logout}</a></li>
					</ul>
				</div>
			</li>
		</ul>
	</header>

	<!-- Left Admin Menu Sidebar (WordPress #adminmenuwrap — no duplicate account/footer) -->
	<aside id="day-sidebar" aria-label="Main menu">
		<nav class="sidebar-nav">
			<!-- 1. DASHBOARD -->
			<div class="nav-group {is_dash_open}">
				<a href="/d-admin/" class="nav-item {is_dash}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
					<span>{nav_dashboard}</span>
					{updates_badge}
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/" class="nav-subitem {sub_dash_home}">{nav_home}</a>
					<a href="/d-admin/getting-started" class="nav-subitem {sub_dash_gs}">{nav_get_started}</a>
					<a href="/d-admin/update-core" class="nav-subitem {sub_dash_updates}">{nav_updates} {updates_badge}</a>
				</div>
			</div>

			<!-- 2. POSTS -->
			<div class="nav-group {is_posts_open}">
				<a href="/d-admin/edit" class="nav-item {is_posts}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
					<span>{nav_posts}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/edit" class="nav-subitem {sub_posts_all}">{nav_all_posts}</a>
					<a href="/d-admin/post-new" class="nav-subitem {sub_posts_new}">{nav_add_new}</a>
					<a href="/d-admin/edit-tags?taxonomy=category" class="nav-subitem {sub_posts_cats}">{nav_categories}</a>
					<a href="/d-admin/edit-tags?taxonomy=post_tag" class="nav-subitem {sub_posts_tags}">{nav_tags}</a>
				</div>
			</div>

			<!-- 3. MEDIA -->
			<div class="nav-group {is_media_open}">
				<a href="/d-admin/upload" class="nav-item {is_media}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
					<span>{nav_media}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/upload" class="nav-subitem {sub_media_lib}">{nav_library}</a>
					<a href="/d-admin/media-new" class="nav-subitem {sub_media_new}">{nav_add_new}</a>
				</div>
			</div>

			<!-- 4. PAGES -->
			<div class="nav-group {is_pages_open}">
				<a href="/d-admin/edit?post_type=page" class="nav-item {is_pages}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path><polyline points="13 2 13 9 20 9"></polyline></svg>
					<span>{nav_pages}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/edit?post_type=page" class="nav-subitem {sub_pages_all}">{nav_all_pages}</a>
					<a href="/d-admin/post-new?post_type=page" class="nav-subitem {sub_pages_new}">{nav_add_new}</a>
				</div>
			</div>

			<!-- 5. DYNAMIC CUSTOM POST TYPES -->
			{custom_post_types_nav}

			<!-- 5a. CONTENT MODEL (CPTs, Taxonomies & Structured Fields) -->
			<div class="nav-group">
				<a href="/d-admin/content-types" class="nav-item {is_content_model}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path></svg>
					<span>{nav_content_model}</span>
				</a>
			</div>

			<!-- 5b. DAYSTUDIO -->
			<div class="nav-group">
				<a href="/d-admin/builder" class="nav-item">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
					<span>{nav_daystudio}</span>
				</a>
			</div>

			<!-- 6. COMMENTS -->
			<div class="nav-group">
				<a href="/d-admin/edit-comments" class="nav-item {is_comments}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path></svg>
					<span>{nav_comments}</span>
					{comments_badge}
				</a>
			</div>

			<!-- 7. APPEARANCE -->
			<div class="nav-group {is_app_open}">
				<a href="/d-admin/headers-footers" class="nav-item {is_appearance}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
					<span>{nav_appearance}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/headers-footers" class="nav-subitem {sub_app_hf}">Header &amp; Footer</a>
					<a href="/d-admin/theme-layouts" class="nav-subitem {sub_app_layouts}">Theme Layouts</a>
					<a href="/d-admin/symbols" class="nav-subitem {sub_app_symbols}">Synced Symbols</a>
					<a href="/d-admin/popups" class="nav-subitem {sub_app_popups}">Popups &amp; Bars</a>
					<a href="/d-admin/customize" class="nav-subitem {sub_app_cust}">Customize</a>
					<a href="/d-admin/nav-menus" class="nav-subitem {sub_app_menus}">Menus</a>
				</div>
			</div>

			<!-- 8. PLUGINS -->
			<div class="nav-group {is_plugins_open}">
				<a href="/d-admin/plugins" class="nav-item {is_plugins}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
					<span>{nav_plugins}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/plugins" class="nav-subitem {sub_plug_list}">Installed Plugins</a>
					<a href="/d-admin/plugin-install" class="nav-subitem {sub_plug_new}">Add New &amp; SDK</a>
					{nav_sub_wasm}
				</div>
			</div>

			<!-- 8b. MARKETING (Overview, SEO & Redirects, Forms, Newsletters, Analytics, WhatsApp Chat) -->
			<div class="nav-group {is_marketing_open}">
				<a href="/d-admin/marketing" class="nav-item {is_marketing}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 11l18-5v12L3 14v-3z"></path><path d="M11.6 16.8a3 3 0 1 1-5.8-1.6"></path></svg>
					<span>Marketing</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/marketing" class="nav-subitem {sub_mkt_overview}">Overview</a>
					{nav_sub_seo}
					{nav_sub_forms}
					{nav_sub_newsletters}
					<a href="/d-admin/analytics" class="nav-subitem {is_analytics}">{nav_analytics}</a>
					{nav_sub_whatsapp}
				</div>
			</div>

			<!-- 9. USERS -->
			<div class="nav-group {is_users_open}">
				<a href="/d-admin/users" class="nav-item {is_users}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
					<span>{nav_users}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/users" class="nav-subitem {sub_users_all}">All Users</a>
					<a href="/d-admin/user-new" class="nav-subitem {sub_users_new}">{nav_add_new}</a>
					<a href="/d-admin/profile" class="nav-subitem {sub_users_prof}">Profile</a>
				</div>
			</div>

			<!-- 10. TOOLS -->
			<div class="nav-group {is_tools_open}">
				<a href="/d-admin/tools" class="nav-item {is_tools}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path></svg>
					<span>{nav_tools}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/tools" class="nav-subitem {sub_tools_avail}">Available Tools</a>
					<a href="/d-admin/import" class="nav-subitem {sub_tools_import}">Import</a>
					<a href="/d-admin/export" class="nav-subitem {sub_tools_export}">Export</a>
					{nav_sub_backups}
					{nav_sub_webhooks}
					{nav_sub_ab_testing}
					{nav_sub_code_snippets}
					<a href="/d-admin/site-health" class="nav-subitem {sub_tools_health}">Site Health</a>
					<a href="/d-admin/export-personal-data" class="nav-subitem {sub_tools_exp_data}">Export Personal Data</a>
					<a href="/d-admin/erase-personal-data" class="nav-subitem {sub_tools_era_data}">Erase Personal Data</a>
				</div>
			</div>

			<!-- 11. SETTINGS -->
			<div class="nav-group {is_settings_open}">
				<a href="/d-admin/options-general" class="nav-item {is_settings}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
					<span>{nav_settings}</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/options-general" class="nav-subitem {sub_set_gen}">General</a>
					<a href="/d-admin/languages" class="nav-subitem {sub_set_lang}">{nav_languages}</a>
					<a href="/d-admin/options-writing" class="nav-subitem {sub_set_writ}">Writing</a>
					<a href="/d-admin/options-reading" class="nav-subitem {sub_set_read}">Reading</a>
					<a href="/d-admin/options-discussion" class="nav-subitem {sub_set_disc}">Discussion</a>
					<a href="/d-admin/options-media" class="nav-subitem {sub_set_media}">Media</a>
					<a href="/d-admin/options-permalink" class="nav-subitem {sub_set_perm}">Permalinks</a>
					<a href="/d-admin/options-privacy" class="nav-subitem {sub_set_priv}">Privacy</a>
				</div>
			</div>

			<!-- 11b. PUBLISH & HOSTING -->
			<div class="nav-group">
				<a href="/d-admin/publish" class="nav-item {is_publish}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5z"></path><path d="M2 17l10 5 10-5"></path><path d="M2 12l10 5 10-5"></path></svg>
					<span>{nav_publish}</span>
				</a>
			</div>

			<!-- SYSTEM TELEMETRY -->
			<div class="nav-group">
				<a href="/d-admin/daypress-engine" class="nav-item {is_engine}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
					<span>{nav_engine}</span>
				</a>
			</div>

			<button type="button" id="btnCollapseSidebar" class="desk-collapse-btn" onclick="dayToggleSidebarCollapse()" title="Collapse / Expand Admin Menu">
				<span class="desk-collapse-icon-circle" id="deskCollapseIcon">&#9666;</span>
				<span class="desk-collapse-label">Collapse menu</span>
			</button>
		</nav>
	</aside>

	<div id="day-mobile-backdrop" onclick="dayToggleMobileNav(false)"></div>

	<!-- Main Workspace (WordPress #wpbody-content) -->
	<main id="day-main">
		<div id="screen-meta" role="region" aria-label="Screen Options and Help">
			<div id="screen-meta-options-panel" style="display:none;">
				<div style="font-weight:600; font-size:13.5px; color:var(--ink); margin-bottom:10px;">Screen Options</div>
				<div style="display:flex; flex-wrap:wrap; gap:18px; align-items:center; font-size:13px; color:var(--ink-muted);">
					<label style="display:inline-flex; align-items:center; gap:6px; cursor:pointer;">
						<input type="checkbox" checked onchange="document.body.classList.toggle('dp-compact-density', !this.checked)" />
						Comfortable row spacing
					</label>
					<label style="display:inline-flex; align-items:center; gap:6px; cursor:pointer;">
						<input type="checkbox" id="chkGlobalFoldMenu" onchange="dayToggleSidebarCollapse()" />
						Fold navigation menu
					</label>
				</div>
			</div>
			<div id="screen-meta-help-panel" style="display:none;">
				<div style="font-weight:600; font-size:13.5px; color:var(--ink); margin-bottom:6px;">Contextual Help &mdash; {page_title}</div>
				<p style="font-size:13px; color:var(--ink-muted); max-width:72ch; margin:0;">
					Use the left navigation menu to switch between content types, visual templates, and site configuration. Press <kbd style="padding:1px 5px; background:var(--paper-sunk); border:1px solid var(--rule-strong); border-radius:3px;">Ctrl+K</kbd> at any time to jump to any post, page, or admin screen. For REST API &amp; MCP automation docs, open <a href="/d-admin/getting-started">Get Started</a>.
				</p>
			</div>
		</div>
		<div id="screen-meta-links">
			<button type="button" id="show-settings-link" class="screen-meta-toggle" aria-expanded="false" onclick="dayToggleScreenMeta('options')">Screen Options</button>
			<button type="button" id="contextual-help-link" class="screen-meta-toggle" aria-expanded="false" onclick="dayToggleScreenMeta('help')">Help</button>
		</div>
		<div id="day-canvas">
			{body_content}
		</div>
		<footer id="wpfooter" role="contentinfo">
			<span id="footer-thankyou">Thank you for creating with <a href="/d-admin/getting-started">Daypress</a>.</span>
			<span id="footer-upgrade">Version {version}</span>
		</footer>
	</main>
</div>

<!-- Global Toast -->
<div id="day-toast"></div>

<!-- Quick Search (Ctrl+K) -->
<div id="day-qs-backdrop" onclick="if (event.target === this) dayCloseQuickSearch();">
	<div class="day-qs-dialog" role="dialog" aria-modal="true" aria-label="Quick search">
		<input id="day-qs-input" type="search" autocomplete="off" placeholder="Search posts, pages and desk screens..." />
		<div id="day-qs-results"></div>
		<div class="day-qs-hint">&uarr; &darr; to move &middot; Enter to open &middot; Esc to close</div>
	</div>
</div>

<script>
	const DAY_QS_SCREENS = [
		['Dashboard', '/d-admin/'], ['Get Started & AI Agent Key', '/d-admin/getting-started'],
		['Updates & Self-Updater', '/d-admin/update-core'],
		['All Posts', '/d-admin/edit'], ['Add New Post', '/d-admin/post-new'],
		['Categories', '/d-admin/edit-tags?taxonomy=category'], ['Tags', '/d-admin/edit-tags?taxonomy=post_tag'],
		['Media Library', '/d-admin/upload'], ['Upload Media', '/d-admin/media-new'],
		['All Pages', '/d-admin/edit?post_type=page'], ['Add New Page', '/d-admin/post-new?post_type=page'],
		['Comments', '/d-admin/edit-comments'], ['Themes', '/d-admin/themes'],
		['Header & Footer Manager', '/d-admin/headers-footers'],
		['Page Assignment Matrix', '/d-admin/headers-footers?tab=matrix'],
		['Theme Layouts', '/d-admin/theme-layouts'],
		['Synced Symbols', '/d-admin/symbols'],
		['Popups & Announcement Bars', '/d-admin/popups'],
		['Customize', '/d-admin/customize'],
		['Widgets', '/d-admin/widgets'], ['Menus', '/d-admin/nav-menus'], ['Theme File Editor', '/d-admin/theme-editor'],
		['Installed Plugins', '/d-admin/plugins'], ['Add New Plugin', '/d-admin/plugin-install'],
		['Marketing Hub', '/d-admin/marketing'],
		{qs_plugin_screens}['Users', '/d-admin/users'], ['Add New User', '/d-admin/user-new'],
		['Profile', '/d-admin/profile'], ['Tools', '/d-admin/tools'], ['Import', '/d-admin/import'],
		['Export', '/d-admin/export'],
		['Site Health (1.0 Security Audit & Benchmarks)', '/d-admin/site-health'], ['Export Personal Data', '/d-admin/export-personal-data'],
		['Erase Personal Data', '/d-admin/erase-personal-data'], ['General Settings', '/d-admin/options-general'],
		['Writing Settings', '/d-admin/options-writing'], ['Reading Settings', '/d-admin/options-reading'],
		['Discussion Settings', '/d-admin/options-discussion'], ['Media Settings', '/d-admin/options-media'],
		['Permalink Settings', '/d-admin/options-permalink'], ['Privacy Settings', '/d-admin/options-privacy'],
		['Languages & Multilingual (i18n)', '/d-admin/languages'],
		['Publish & Hosting', '/d-admin/publish'],
		['Content Model (CPTs & ACF)', '/d-admin/content-types'],
		['Engine Telemetry', '/d-admin/daypress-engine'], ['Analytics', '/d-admin/analytics']
	];
	let dayQsItems = [];
	let dayQsActive = 0;
	let dayQsTimer = null;
	let dayQsSeq = 0;

	function dayQsEscape(s) {{
		return String(s == null ? '' : s).replace(/[&<>"']/g, c => ({{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}}[c]));
	}}
	function dayOpenQuickSearch() {{
		const bd = document.getElementById('day-qs-backdrop');
		const input = document.getElementById('day-qs-input');
		if (!bd || !input) return;
		bd.style.display = 'flex';
		input.value = '';
		dayQsRender('', []);
		setTimeout(() => input.focus(), 0);
	}}
	function dayCloseQuickSearch() {{
		const bd = document.getElementById('day-qs-backdrop');
		if (bd) bd.style.display = 'none';
	}}
	function dayQsRender(query, contentResults) {{
		const q = query.trim().toLowerCase();
		const screens = DAY_QS_SCREENS
			.filter(([label]) => !q || label.toLowerCase().includes(q))
			.slice(0, q ? 6 : 8)
			.map(([label, url]) => ({{ label, url, kind: 'Screen' }}));
		dayQsItems = contentResults.concat(screens);
		dayQsActive = 0;
		const box = document.getElementById('day-qs-results');
		if (!box) return;
		if (!dayQsItems.length) {{
			box.innerHTML = '<div class="day-qs-empty">No matches</div>';
			return;
		}}
		box.innerHTML = dayQsItems.map((it, i) =>
			'<a href="' + dayQsEscape(it.url) + '" data-qs-idx="' + i + '" class="day-qs-item' + (i === 0 ? ' is-active' : '') + '">' +
			'<span>' + dayQsEscape(it.label) + '</span><span class="day-qs-kind">' + dayQsEscape(it.kind) + '</span></a>'
		).join('');
	}}
	function dayQsHighlight() {{
		document.querySelectorAll('#day-qs-results [data-qs-idx]').forEach(el => {{
			const active = Number(el.dataset.qsIdx) === dayQsActive;
			el.classList.toggle('is-active', active);
			if (active) el.scrollIntoView({{ block: 'nearest' }});
		}});
	}}
	async function dayQsSearch(query) {{
		const seq = ++dayQsSeq;
		const q = query.trim();
		if (q.length < 2) {{ dayQsRender(query, []); return; }}
		let content = [];
		try {{
			const res = await fetch('/wp-json/wp/v2/search?per_page=8&search=' + encodeURIComponent(q), {{ credentials: 'same-origin' }});
			if (res.ok) {{
				const rows = await res.json();
				content = (Array.isArray(rows) ? rows : []).map(r => ({{
					label: (r.title || '(no title)').replace(/<[^>]*>/g, ''),
					url: '/d-admin/post?post=' + encodeURIComponent(r.id),
					kind: r.subtype || 'post'
				}}));
			}}
		}} catch (err) {{ /* network errors fall back to screen matches */ }}
		if (seq === dayQsSeq) dayQsRender(query, content);
	}}
	document.addEventListener('DOMContentLoaded', () => {{
		const input = document.getElementById('day-qs-input');
		if (!input) return;
		input.addEventListener('input', () => {{
			clearTimeout(dayQsTimer);
			dayQsRender(input.value, []);
			dayQsTimer = setTimeout(() => dayQsSearch(input.value), 180);
		}});
		input.addEventListener('keydown', (e) => {{
			if (e.key === 'ArrowDown') {{ e.preventDefault(); dayQsActive = Math.min(dayQsActive + 1, dayQsItems.length - 1); dayQsHighlight(); }}
			else if (e.key === 'ArrowUp') {{ e.preventDefault(); dayQsActive = Math.max(dayQsActive - 1, 0); dayQsHighlight(); }}
			else if (e.key === 'Enter') {{ e.preventDefault(); const it = dayQsItems[dayQsActive]; if (it) window.location.href = it.url; }}
			else if (e.key === 'Escape') {{ dayCloseQuickSearch(); }}
		}});
	}});
	// Registered last: screens with their own Ctrl+K (e.g. the editor's "insert link") call preventDefault first.
	document.addEventListener('keydown', (e) => {{
		if (e.defaultPrevented) return;
		const mod = e.ctrlKey || e.metaKey;
		if (mod && (e.key === 'k' || e.key === 'K')) {{
			e.preventDefault();
			dayOpenQuickSearch();
		}} else if (e.key === 'Escape') {{
			dayCloseQuickSearch();
		}}
	}});
</script>

<script>
	function showToast(message, isError = false) {{
		const toast = document.getElementById('day-toast');
		toast.textContent = message;
		toast.classList.toggle('is-error', isError);
		toast.style.display = 'flex';
		setTimeout(() => {{
			toast.style.display = 'none';
		}}, 3500);
	}}

	document.querySelectorAll('#day-sidebar .nav-group').forEach(group => {{
		group.addEventListener('mouseenter', () => {{
			if (!group.classList.contains('open') || document.body.classList.contains('dp-sidebar-collapsed')) {{
				const sub = group.querySelector('.nav-submenu');
				if (sub) {{
					const rect = group.getBoundingClientRect();
					const maxTop = Math.max(36, window.innerHeight - 220);
					sub.style.top = Math.min(rect.top, maxTop) + 'px';
				}}
			}} else {{
				const sub = group.querySelector('.nav-submenu');
				if (sub) sub.style.top = '';
			}}
		}});
	}});

	function dayToggleMobileNav(force) {{
		const open = typeof force === 'boolean' ? force : !document.body.classList.contains('dp-mobile-nav-open');
		document.body.classList.toggle('dp-mobile-nav-open', open);
	}}

	function dayToggleScreenMeta(kind) {{
		const customOptions = document.getElementById('dashScreenOptionsPanel') || document.getElementById('editorScreenOptionsPanel') || document.getElementById('listScreenOptionsPanel') || document.getElementById('screenOptionsDrawer');
		const metaWrap = document.getElementById('screen-meta');
		const optPanel = document.getElementById('screen-meta-options-panel');
		const helpPanel = document.getElementById('screen-meta-help-panel');
		const btnOpt = document.getElementById('show-settings-link');
		const btnHelp = document.getElementById('contextual-help-link');
		if (kind === 'options') {{
			if (customOptions) {{
				const isOpen = customOptions.style.display === 'block';
				customOptions.style.display = isOpen ? 'none' : 'block';
				if (btnOpt) btnOpt.setAttribute('aria-expanded', isOpen ? 'false' : 'true');
				if (metaWrap) metaWrap.classList.remove('is-open');
				if (btnHelp) btnHelp.setAttribute('aria-expanded', 'false');
				return;
			}}
			if (!metaWrap || !optPanel || !helpPanel) return;
			const wasOpen = metaWrap.classList.contains('is-open') && optPanel.style.display === 'block';
			helpPanel.style.display = 'none';
			if (btnHelp) btnHelp.setAttribute('aria-expanded', 'false');
			optPanel.style.display = wasOpen ? 'none' : 'block';
			metaWrap.classList.toggle('is-open', !wasOpen);
			if (btnOpt) btnOpt.setAttribute('aria-expanded', wasOpen ? 'false' : 'true');
		}} else {{
			if (customOptions) customOptions.style.display = 'none';
			if (btnOpt) btnOpt.setAttribute('aria-expanded', 'false');
			if (!metaWrap || !optPanel || !helpPanel) return;
			const wasOpen = metaWrap.classList.contains('is-open') && helpPanel.style.display === 'block';
			optPanel.style.display = 'none';
			helpPanel.style.display = wasOpen ? 'none' : 'block';
			metaWrap.classList.toggle('is-open', !wasOpen);
			if (btnHelp) btnHelp.setAttribute('aria-expanded', wasOpen ? 'false' : 'true');
		}}
	}}

	function dayApplySidebarCollapsed(collapsed) {{
		document.body.classList.toggle('dp-sidebar-collapsed', !!collapsed);
		const icon = document.getElementById('deskCollapseIcon');
		if (icon) icon.textContent = collapsed ? '\u25b8' : '\u25c2';
		const chk = document.getElementById('chkGlobalFoldMenu');
		if (chk) chk.checked = !!collapsed;
	}}

	function dayToggleSidebarCollapse() {{
		const next = !document.body.classList.contains('dp-sidebar-collapsed');
		dayApplySidebarCollapsed(next);
		try {{ localStorage.setItem('daypress_sidebar_collapsed', next ? '1' : '0'); }} catch (_) {{}}
		fetch('/wp-json/daypress/v1/user-preferences', {{
			method: 'POST',
			headers: {{ 'Content-Type': 'application/json' }},
			body: JSON.stringify({{ sidebar_collapsed: next }})
		}}).catch(() => {{}});
	}}

	(function initSidebarCollapse() {{
		try {{
			if (localStorage.getItem('daypress_sidebar_collapsed') === '1') {{
				dayApplySidebarCollapsed(true);
			}}
		}} catch (_) {{}}
		fetch('/wp-json/daypress/v1/user-preferences')
			.then(r => r.ok ? r.json() : null)
			.then(data => {{
				if (data && data.preferences && typeof data.preferences.sidebar_collapsed === 'boolean') {{
					dayApplySidebarCollapsed(data.preferences.sidebar_collapsed);
					try {{ localStorage.setItem('daypress_sidebar_collapsed', data.preferences.sidebar_collapsed ? '1' : '0'); }} catch (_) {{}}
				}}
			}})
			.catch(() => {{}});
	}})();
</script>
</body>
</html>
"##,
        page_title = page_title,
        site_name = site_name,
        rest_nonce = rest_nonce,
        account_name = account_name,
        comments_badge = comments_badge,
        pending_comments_num = pending_comments_num,
        version = env!("CARGO_PKG_VERSION"),
        styles_shell = styles_shell,
        styles_comp = styles_comp,
        styles_forms = styles_forms,
        is_dash = nav.is_dash,
        is_dash_open = nav.is_dash_open,
        sub_dash_home = nav.sub_dash_home,
        sub_dash_gs = nav.sub_dash_gs,
        sub_dash_updates = nav.sub_dash_updates,
        is_posts = nav.is_posts,
        is_posts_open = nav.is_posts_open,
        sub_posts_all = nav.sub_posts_all,
        sub_posts_new = nav.sub_posts_new,
        sub_posts_cats = nav.sub_posts_cats,
        sub_posts_tags = nav.sub_posts_tags,
        is_media = nav.is_media,
        is_media_open = nav.is_media_open,
        sub_media_lib = nav.sub_media_lib,
        sub_media_new = nav.sub_media_new,
        is_pages = nav.is_pages,
        is_pages_open = nav.is_pages_open,
        sub_pages_all = nav.sub_pages_all,
        sub_pages_new = nav.sub_pages_new,
        custom_post_types_nav = custom_post_types_nav,
        is_comments = nav.is_comments,
        is_appearance = nav.is_appearance,
        is_app_open = nav.is_app_open,
        sub_app_hf = nav.sub_app_hf,
        sub_app_layouts = nav.sub_app_layouts,
        sub_app_symbols = nav.sub_app_symbols,
        sub_app_popups = nav.sub_app_popups,
        sub_app_cust = nav.sub_app_cust,
        sub_app_menus = nav.sub_app_menus,
        is_plugins = nav.is_plugins,
        is_plugins_open = nav.is_plugins_open,
        sub_plug_list = nav.sub_plug_list,
        sub_plug_new = nav.sub_plug_new,
        is_marketing = nav.is_marketing,
        is_marketing_open = nav.is_marketing_open,
        sub_mkt_overview = nav.sub_mkt_overview,
        is_analytics = nav.is_analytics,
        nav_sub_wasm = nav_sub_wasm,
        nav_sub_forms = nav_sub_forms,
        nav_sub_newsletters = nav_sub_newsletters,
        nav_sub_whatsapp = nav_sub_whatsapp,
        nav_sub_webhooks = nav_sub_webhooks,
        nav_sub_ab_testing = nav_sub_ab_testing,
        nav_sub_code_snippets = nav_sub_code_snippets,
        nav_sub_seo = nav_sub_seo,
        nav_sub_backups = nav_sub_backups,
        qs_plugin_screens = qs_plugin_screens,
        is_users = nav.is_users,
        is_users_open = nav.is_users_open,
        sub_users_all = nav.sub_users_all,
        sub_users_new = nav.sub_users_new,
        sub_users_prof = nav.sub_users_prof,
        is_tools = nav.is_tools,
        is_tools_open = nav.is_tools_open,
        sub_tools_avail = nav.sub_tools_avail,
        sub_tools_import = nav.sub_tools_import,
        sub_tools_export = nav.sub_tools_export,
        sub_tools_health = nav.sub_tools_health,
        sub_tools_exp_data = nav.sub_tools_exp_data,
        sub_tools_era_data = nav.sub_tools_era_data,
        is_settings = nav.is_settings,
        is_settings_open = nav.is_settings_open,
        sub_set_gen = nav.sub_set_gen,
        sub_set_writ = nav.sub_set_writ,
        sub_set_read = nav.sub_set_read,
        sub_set_disc = nav.sub_set_disc,
        sub_set_media = nav.sub_set_media,
        sub_set_perm = nav.sub_set_perm,
        sub_set_priv = nav.sub_set_priv,
        sub_set_lang = nav.sub_set_lang,
        is_publish = nav.is_publish,
        is_engine = nav.is_engine,
        is_content_model = nav.is_content_model,
        admin_color = admin_color,
        html_lang = html_lang,
        updates_badge = updates_badge,
        body_content = body_content,
        nav_dashboard = tr("nav.dashboard"),
        nav_home = tr("nav.home"),
        nav_get_started = tr("nav.get_started"),
        nav_updates = tr("nav.updates"),
        nav_posts = tr("nav.posts"),
        nav_all_posts = tr("nav.all_posts"),
        nav_add_new = tr("nav.add_new"),
        nav_categories = tr("nav.categories"),
        nav_tags = tr("nav.tags"),
        nav_media = tr("nav.media"),
        nav_library = tr("nav.library"),
        nav_pages = tr("nav.pages"),
        nav_all_pages = tr("nav.all_pages"),
        nav_content_model = tr("nav.content_model"),
        nav_daystudio = tr("nav.daystudio"),
        nav_comments = tr("nav.comments"),
        nav_appearance = tr("nav.appearance"),
        nav_plugins = tr("nav.plugins"),
        nav_analytics = tr("nav.analytics"),
        nav_users = tr("nav.users"),
        nav_tools = tr("nav.tools"),
        nav_settings = tr("nav.settings"),
        nav_languages = tr("nav.languages"),
        nav_publish = tr("nav.publish"),
        nav_engine = tr("nav.engine"),
        nav_logout = tr("nav.logout"),
    )
}
