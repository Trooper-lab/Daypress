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
/* Daypress Soft Editorial Backend Themes (plus legacy alias selectors for backward compatibility) */
:root[data-admin-color="linen"], body[data-admin-color="linen"],
:root[data-admin-color="light"], body[data-admin-color="light"] {{
	--desk: #e8e1d3; --desk-raised: #ded5c4; --desk-ink: #2b2621; --desk-muted: #6e655c; --desk-rule: #2b262124; --desk-oxide: #b05a3c;
	--paper: #f8f5ee; --paper-raised: #fdfbf7; --paper-sunk: #ede7da; --ink: #221f1c; --ink-muted: #686159;
	--oxide: #b05a3c; --press: #4a6b9c; --moss: #4d6b53; --focus: #4a6b9c;
	--press-wash: rgba(74,107,156,0.10); --oxide-wash: rgba(176,90,60,0.10);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
:root[data-admin-color="studio"], body[data-admin-color="studio"],
:root[data-admin-color="modern"], body[data-admin-color="modern"] {{
	--desk: #252830; --desk-raised: #313540; --desk-ink: #eef0f6; --desk-muted: #9da3b4; --desk-rule: #eef0f624; --desk-oxide: #6c8ebf;
	--paper: #f4f5f8; --paper-raised: #fbfcfe; --paper-sunk: #e8eaf0; --ink: #1b1e26; --ink-muted: #5d6373;
	--oxide: #4f5bd5; --press: #4f5bd5; --moss: #3d8b6e; --focus: #4f5bd5;
	--press-wash: rgba(79,91,213,0.10); --oxide-wash: rgba(79,91,213,0.10);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
:root[data-admin-color="nordic"], body[data-admin-color="nordic"],
:root[data-admin-color="blue"], body[data-admin-color="blue"] {{
	--desk: #283642; --desk-raised: #334554; --desk-ink: #edf2f6; --desk-muted: #9cb0c0; --desk-rule: #edf2f624; --desk-oxide: #7ba6bc;
	--paper: #f1f5f7; --paper-raised: #f9fbfc; --paper-sunk: #e3ebf0; --ink: #1c262e; --ink-muted: #586975;
	--oxide: #3b6e8c; --press: #2f6690; --moss: #467362; --focus: #2f6690;
	--press-wash: rgba(47,102,144,0.10); --oxide-wash: rgba(59,110,140,0.10);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
:root[data-admin-color="espresso"], body[data-admin-color="espresso"],
:root[data-admin-color="coffee"], body[data-admin-color="coffee"] {{
	--desk: #362e29; --desk-raised: #473d36; --desk-ink: #f3ece4; --desk-muted: #b5a698; --desk-rule: #f3ece424; --desk-oxide: #d49b72;
	--paper: #f5efe8; --paper-raised: #fcf9f5; --paper-sunk: #e9dfd3; --ink: #26201c; --ink-muted: #695d54;
	--oxide: #9e5f3b; --press: #7c5840; --moss: #5e7153; --focus: #9e5f3b;
	--press-wash: rgba(158,95,59,0.11); --oxide-wash: rgba(158,95,59,0.11);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
:root[data-admin-color="sage"], body[data-admin-color="sage"],
:root[data-admin-color="ectoplasm"], body[data-admin-color="ectoplasm"] {{
	--desk: #26332c; --desk-raised: #33443b; --desk-ink: #ebf1ed; --desk-muted: #9eb0a5; --desk-rule: #ebf1ed24; --desk-oxide: #8fb399;
	--paper: #f1f4ef; --paper-raised: #f9fbf8; --paper-sunk: #e3e9e0; --ink: #1c2520; --ink-muted: #56665c;
	--oxide: #48755b; --press: #3b6952; --moss: #48755b; --focus: #3b6952;
	--press-wash: rgba(72,117,91,0.11); --oxide-wash: rgba(72,117,91,0.11);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
:root[data-admin-color="dusk"], body[data-admin-color="dusk"],
:root[data-admin-color="midnight"], body[data-admin-color="midnight"] {{
	--desk: #2e2938; --desk-raised: #3d364a; --desk-ink: #f1eef6; --desk-muted: #aca3bd; --desk-rule: #f1eef624; --desk-oxide: #c98a78;
	--paper: #f5f2f7; --paper-raised: #fcfafe; --paper-sunk: #e8e3ed; --ink: #221e2b; --ink-muted: #645c73;
	--oxide: #7b5285; --press: #684675; --moss: #4e6e5d; --focus: #7b5285;
	--press-wash: rgba(123,82,133,0.11); --oxide-wash: rgba(123,82,133,0.11);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
:root[data-admin-color="harbor"], body[data-admin-color="harbor"],
:root[data-admin-color="ocean"], body[data-admin-color="ocean"] {{
	--desk: #2f3e44; --desk-raised: #3d5057; --desk-ink: #eef3f2; --desk-muted: #a3b5b3; --desk-rule: #eef3f224; --desk-oxide: #8ab0ab;
	--paper: #f2f5f4; --paper-raised: #fafcfb; --paper-sunk: #e4eae8; --ink: #1f282b; --ink-muted: #5b6b6e;
	--oxide: #457371; --press: #3b696c; --moss: #527564; --focus: #457371;
	--press-wash: rgba(69,115,113,0.11); --oxide-wash: rgba(69,115,113,0.11);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
:root[data-admin-color="terracotta"], body[data-admin-color="terracotta"],
:root[data-admin-color="sunrise"], body[data-admin-color="sunrise"] {{
	--desk: #422b27; --desk-raised: #563833; --desk-ink: #f7ede8; --desk-muted: #c4a99e; --desk-rule: #f7ede824; --desk-oxide: #e08b63;
	--paper: #f7f0e8; --paper-raised: #fdfaf6; --paper-sunk: #ece1d4; --ink: #291d1a; --ink-muted: #6e5853;
	--oxide: #b8563d; --press: #9e4731; --moss: #597051; --focus: #b8563d;
	--press-wash: rgba(184,86,61,0.11); --oxide-wash: rgba(184,86,61,0.11);
	--bg-app: var(--paper); --bg-surface: var(--paper-raised); --bg-surface-alt: var(--paper-sunk); --primary: var(--press);
}}
	</style>
	<script>
		// Standard WordPress REST API settings & session initialization
		window.wpApiSettings = window.wpApiSettings || {{
			root: '/wp-json/',
			nonce: '{rest_nonce}',
			versionString: 'wp/v2/'
		}};
	</script>
</head>
<body data-admin-color="{admin_color}">
<div id="day-app">
	<!-- The desk: wordmark, search, navigation, account -->
	<aside id="day-sidebar">
		<div class="desk-brand">
			<a href="/d-admin/" class="desk-wordmark" aria-label="Daypress desk"><span class="day-dot" aria-hidden="true"></span>daypress</a>
			<a href="/" target="_blank" rel="noopener" class="desk-site" title="View the site">{site_name} &nearr;</a>
		</div>
		<button type="button" class="desk-search" onclick="dayOpenQuickSearch()"><span>Search</span> <kbd>Ctrl K</kbd></button>
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
				</div>
			</div>

			<!-- 8b. FORMS STUDIO -->
			<div class="nav-group">
				<a href="/d-admin/forms" class="nav-item {is_forms}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"></path><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path></svg>
					<span>{nav_forms}</span>
				</a>
			</div>

			<!-- 8c. COOKIELESS ANALYTICS -->
			<div class="nav-group">
				<a href="/d-admin/analytics" class="nav-item {is_analytics}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="20" x2="18" y2="10"></line><line x1="12" y1="20" x2="12" y2="4"></line><line x1="6" y1="20" x2="6" y2="14"></line></svg>
					<span>{nav_analytics}</span>
				</a>
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
					<a href="/d-admin/seo" class="nav-subitem {is_seo}">SEO &amp; Redirects</a>
					<a href="/d-admin/backups" class="nav-subitem {is_backups}">Backups</a>
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
		</nav>

		<div class="desk-footer">
			<div class="desk-account">
				<a href="/d-admin/profile" title="Edit your profile">{account_name}</a>
				<a href="/login?action=logout" class="desk-logout">{nav_logout}</a>
			</div>
			<div class="desk-version">v{version} &middot; single binary</div>
			<button type="button" id="btnCollapseSidebar" class="desk-collapse-btn" onclick="dayToggleSidebarCollapse()" title="Collapse / Expand Admin Menu">
				<span id="deskCollapseIcon">◂</span>
				<span class="desk-collapse-label">Collapse menu</span>
			</button>
		</div>
	</aside>

	<!-- Main Workspace -->
	<main id="day-main">
		<div id="day-canvas">
			{body_content}
		</div>
	</main>
</div>

<!-- Global Toast -->
<div id="day-toast"></div>

<!-- Quick Search (Ctrl+K) -->
<div id="day-qs-backdrop" onclick="if (event.target === this) dayCloseQuickSearch();">
	<div class="day-qs-dialog" role="dialog" aria-modal="true" aria-label="Quick search">
		<input id="day-qs-input" type="search" autocomplete="off" placeholder="Search posts, pages and desk screens…" />
		<div id="day-qs-results"></div>
		<div class="day-qs-hint">↑ ↓ to move · Enter to open · Esc to close</div>
	</div>
</div>

<script>
	const DAY_QS_SCREENS = [
		['Dashboard', '/d-admin/'], ['All Posts', '/d-admin/edit'], ['Add New Post', '/d-admin/post-new'],
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
		['Forms Studio', '/d-admin/forms'], ['Users', '/d-admin/users'], ['Add New User', '/d-admin/user-new'],
		['Profile', '/d-admin/profile'], ['Tools', '/d-admin/tools'], ['Import', '/d-admin/import'],
		['Export', '/d-admin/export'], ['SEO & Redirects', '/d-admin/seo'], ['Backups', '/d-admin/backups'],
		['Site Health', '/d-admin/site-health'], ['Export Personal Data', '/d-admin/export-personal-data'],
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

	function dayApplySidebarCollapsed(collapsed) {{
		document.body.classList.toggle('dp-sidebar-collapsed', !!collapsed);
		const icon = document.getElementById('deskCollapseIcon');
		if (icon) icon.textContent = collapsed ? '▸' : '◂';
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
        is_seo = nav.is_seo,
        is_forms = nav.is_forms,
        is_analytics = nav.is_analytics,
        is_backups = nav.is_backups,
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
        nav_forms = tr("nav.forms"),
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
