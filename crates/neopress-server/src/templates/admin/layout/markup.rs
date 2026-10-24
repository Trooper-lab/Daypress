//! Admin Shell HTML Markup and template rendering.

use crate::models::WpSettingsResponse;
use crate::templates::admin::layout::navigation::{
    render_cpt_navigation, render_wp_admin_bar, NavState,
};
use crate::templates::admin::layout::styles_components::get_styles_components;
use crate::templates::admin::layout::styles_forms::get_styles_forms;
use crate::templates::admin::layout::styles_shell::get_styles_shell;

pub fn render_wp_admin_shell(
    current_tab: &str,
    page_title: &str,
    body_content: &str,
    settings: &WpSettingsResponse,
) -> String {
    let site_name = if settings.title.is_empty() {
        "Neopress"
    } else {
        &settings.title
    };

    let nav = NavState::compute(current_tab);
    let custom_post_types_nav = render_cpt_navigation(current_tab, &settings.custom_post_types);
    let wp_admin_bar = render_wp_admin_bar(site_name, settings.pending_comments_count);
    let styles_shell = get_styles_shell();
    let styles_comp = get_styles_components();
    let styles_forms = get_styles_forms();

    format!(
        r##"<!DOCTYPE html>
<html lang="en-US">
<head>
	<meta charset="UTF-8" />
	<title>{page_title} &bull; {site_name}</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="icon" type="image/svg+xml" href="/static/favicon.svg" />
	<style>
{styles_shell}
{styles_comp}
{styles_forms}
	</style>
	<script>
		// Standard WordPress REST API settings & session initialization
		window.wpApiSettings = window.wpApiSettings || {{
			root: '/wp-json/',
			nonce: 'wp_rest_admin',
			versionString: 'wp/v2/'
		}};
	</script>
</head>
<body>
{wp_admin_bar}
<div id="neo-app">
	<!-- Left 240px Navigation Sidebar -->
	<aside id="neo-sidebar">
		<div class="sidebar-header">
			<a href="/n-admin/" class="brand-link">
				<img src="/static/neopress-icon.svg" class="brand-icon" alt="Neopress Icon" />
				<span class="brand-name">Neopress</span>
			</a>
		</div>

		<nav class="sidebar-nav">
			<!-- 1. DASHBOARD -->
			<div class="nav-group {is_dash_open}">
				<a href="/n-admin/index.php" class="nav-item {is_dash}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
					<span>Dashboard</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/index.php" class="nav-subitem {sub_dash_home}">Home</a>
					<a href="/n-admin/update-core.php" class="nav-subitem {sub_dash_updates}">Updates</a>
				</div>
			</div>

			<!-- 2. POSTS -->
			<div class="nav-group {is_posts_open}">
				<a href="/n-admin/edit.php" class="nav-item {is_posts}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
					<span>Posts</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/edit.php" class="nav-subitem {sub_posts_all}">All Posts</a>
					<a href="/n-admin/post-new.php" class="nav-subitem {sub_posts_new}">Add New</a>
					<a href="/n-admin/edit-tags.php?taxonomy=category" class="nav-subitem {sub_posts_cats}">Categories</a>
					<a href="/n-admin/edit-tags.php?taxonomy=post_tag" class="nav-subitem {sub_posts_tags}">Tags</a>
				</div>
			</div>

			<!-- 3. MEDIA -->
			<div class="nav-group {is_media_open}">
				<a href="/n-admin/upload.php" class="nav-item {is_media}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
					<span>Media</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/upload.php" class="nav-subitem {sub_media_lib}">Library</a>
					<a href="/n-admin/media-new.php" class="nav-subitem {sub_media_new}">Add New</a>
				</div>
			</div>

			<!-- 4. PAGES -->
			<div class="nav-group {is_pages_open}">
				<a href="/n-admin/edit.php?post_type=page" class="nav-item {is_pages}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path><polyline points="13 2 13 9 20 9"></polyline></svg>
					<span>Pages</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/edit.php?post_type=page" class="nav-subitem {sub_pages_all}">All Pages</a>
					<a href="/n-admin/post-new.php?post_type=page" class="nav-subitem {sub_pages_new}">Add New</a>
				</div>
			</div>

			<!-- 5. DYNAMIC CUSTOM POST TYPES -->
			{custom_post_types_nav}

			<!-- 6. COMMENTS -->
			<div class="nav-group">
				<a href="/n-admin/edit-comments.php" class="nav-item {is_comments}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path></svg>
					<span>Comments</span>
				</a>
			</div>

			<div class="nav-section-title">CONFIGURATION</div>

			<!-- 7. APPEARANCE -->
			<div class="nav-group {is_app_open}">
				<a href="/n-admin/themes.php" class="nav-item {is_appearance}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
					<span>Appearance</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/themes.php" class="nav-subitem {sub_app_themes}">Themes</a>
					<a href="/n-admin/customize.php" class="nav-subitem {sub_app_cust}">Customize</a>
					<a href="/n-admin/widgets.php" class="nav-subitem {sub_app_widg}">Widgets</a>
					<a href="/n-admin/nav-menus.php" class="nav-subitem {sub_app_menus}">Menus</a>
					<a href="/n-admin/theme-editor.php" class="nav-subitem {sub_app_editor}">Theme File Editor</a>
				</div>
			</div>

			<!-- 8. PLUGINS -->
			<div class="nav-group {is_plugins_open}">
				<a href="/n-admin/plugins.php" class="nav-item {is_plugins}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
					<span>Plugins</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/plugins.php" class="nav-subitem {sub_plug_list}">Installed Plugins</a>
					<a href="/n-admin/plugin-install.php" class="nav-subitem {sub_plug_new}">Add New &amp; SDK</a>
					<a href="/n-admin/plugin-editor.php" class="nav-subitem {sub_plug_editor}">Plugin File Editor</a>
				</div>
			</div>

			<!-- 8b. SEO & REDIRECTS -->
			<div class="nav-group">
				<a href="/n-admin/seo.php" class="nav-item {is_seo}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line><line x1="11" y1="8" x2="11" y2="14"></line><line x1="8" y1="11" x2="14" y2="11"></line></svg>
					<span>SEO &amp; Redirects</span>
				</a>
			</div>

			<!-- 8c. FORMS STUDIO -->
			<div class="nav-group">
				<a href="/n-admin/forms.php" class="nav-item {is_forms}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"></path><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path></svg>
					<span>Forms Studio</span>
				</a>
			</div>

			<!-- 8d. SNAPSHOT BACKUPS -->
			<div class="nav-group">
				<a href="/n-admin/backups.php" class="nav-item {is_backups}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path></svg>
					<span>Backups</span>
				</a>
			</div>

			<!-- 9. USERS -->
			<div class="nav-group {is_users_open}">
				<a href="/n-admin/users.php" class="nav-item {is_users}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
					<span>Users</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/users.php" class="nav-subitem {sub_users_all}">All Users</a>
					<a href="/n-admin/user-new.php" class="nav-subitem {sub_users_new}">Add New</a>
					<a href="/n-admin/profile.php" class="nav-subitem {sub_users_prof}">Profile</a>
				</div>
			</div>

			<!-- 10. TOOLS -->
			<div class="nav-group {is_tools_open}">
				<a href="/n-admin/tools.php" class="nav-item {is_tools}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path></svg>
					<span>Tools</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/tools.php" class="nav-subitem {sub_tools_avail}">Available Tools</a>
					<a href="/n-admin/import.php" class="nav-subitem {sub_tools_import}">Import</a>
					<a href="/n-admin/export.php" class="nav-subitem {sub_tools_export}">Export</a>
					<a href="/n-admin/site-health.php" class="nav-subitem {sub_tools_health}">Site Health</a>
					<a href="/n-admin/export-personal-data.php" class="nav-subitem {sub_tools_exp_data}">Export Personal Data</a>
					<a href="/n-admin/erase-personal-data.php" class="nav-subitem {sub_tools_era_data}">Erase Personal Data</a>
				</div>
			</div>

			<!-- 11. SETTINGS -->
			<div class="nav-group {is_settings_open}">
				<a href="/n-admin/options-general.php" class="nav-item {is_settings}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
					<span>Settings</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/options-general.php" class="nav-subitem {sub_set_gen}">General</a>
					<a href="/n-admin/options-writing.php" class="nav-subitem {sub_set_writ}">Writing</a>
					<a href="/n-admin/options-reading.php" class="nav-subitem {sub_set_read}">Reading</a>
					<a href="/n-admin/options-discussion.php" class="nav-subitem {sub_set_disc}">Discussion</a>
					<a href="/n-admin/options-media.php" class="nav-subitem {sub_set_media}">Media</a>
					<a href="/n-admin/options-permalink.php" class="nav-subitem {sub_set_perm}">Permalinks</a>
					<a href="/n-admin/options-privacy.php" class="nav-subitem {sub_set_priv}">Privacy</a>
				</div>
			</div>

			<!-- SYSTEM TELEMETRY -->
			<div class="nav-group">
				<a href="/n-admin/neopress-engine.php" class="nav-item {is_engine}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
					<span>Engine Telemetry</span>
				</a>
			</div>
		</nav>

		<div class="sidebar-footer">
			<a href="/" target="_blank" class="view-site-btn">
				<span>View Public Site</span>
				<span style="font-size: 13px;">&nearr;</span>
			</a>
			<a href="/n-admin/profile.php" class="user-profile" style="text-decoration: none;">
				<div class="user-avatar">A</div>
				<div class="user-details">
					<span class="user-name">admin</span>
					<span class="user-role">Administrator</span>
				</div>
			</a>
			<div style="margin-top: 10px; padding-top: 10px; border-top: 1px solid rgba(255, 255, 255, 0.08); display: flex; justify-content: space-between; align-items: center;">
				<a href="/n-admin/profile.php" style="color: #94a3b8; font-size: 12px; text-decoration: none;">Edit Profile</a>
				<a href="/wp-login.php?action=logout" style="color: #f87171; font-size: 12px; text-decoration: none; font-weight: 600; display: inline-flex; align-items: center; gap: 4px;">
					<span>Log Out</span>
					<span>&rarr;</span>
				</a>
			</div>
		</div>
	</aside>

	<!-- Main Workspace -->
	<main id="neo-main">
		<header id="neo-topbar">
			<div class="breadcrumbs">
				<a href="/n-admin/" class="crumb-root">Neopress</a>
				<span class="crumb-sep">/</span>
				<span class="crumb-active">{page_title}</span>
			</div>

			<div class="topbar-actions">
				<div class="latency-chip">
					<span class="latency-dot"></span>
					<span>⚡ &lt;0.15ms in-process</span>
				</div>
				<a href="/n-admin/post-new.php" class="btn-new-post">
					<span>+</span>
					<span>New Post</span>
				</a>
				<a href="/wp-login.php?action=logout" title="Log Out" style="background: #f1f5f9; color: #475569; text-decoration: none; font-size: 12px; font-weight: 500; padding: 6px 12px; border-radius: 6px; border: 1px solid #cbd5e1; display: inline-flex; align-items: center; gap: 4px; transition: all 0.15s;">
					<span>Log Out</span>
				</a>
			</div>
		</header>

		<div id="neo-canvas">
			{body_content}
		</div>
	</main>
</div>

<!-- Global Toast -->
<div id="neo-toast"></div>

<script>
	function showToast(message, isError = false) {{
		const toast = document.getElementById('neo-toast');
		toast.textContent = message;
		toast.style.background = isError ? '#b91c1c' : '#0f172a';
		toast.style.display = 'flex';
		setTimeout(() => {{
			toast.style.display = 'none';
		}}, 3500);
	}}

	document.querySelectorAll('#neo-sidebar .nav-group').forEach(group => {{
		group.addEventListener('mouseenter', () => {{
			if (!group.classList.contains('open')) {{
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
</script>
</body>
</html>
"##,
        page_title = page_title,
        site_name = site_name,
        wp_admin_bar = wp_admin_bar,
        styles_shell = styles_shell,
        styles_comp = styles_comp,
        styles_forms = styles_forms,
        is_dash = nav.is_dash,
        is_dash_open = nav.is_dash_open,
        sub_dash_home = nav.sub_dash_home,
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
        sub_app_themes = nav.sub_app_themes,
        sub_app_cust = nav.sub_app_cust,
        sub_app_widg = nav.sub_app_widg,
        sub_app_menus = nav.sub_app_menus,
        sub_app_editor = nav.sub_app_editor,
        is_plugins = nav.is_plugins,
        is_plugins_open = nav.is_plugins_open,
        sub_plug_list = nav.sub_plug_list,
        sub_plug_new = nav.sub_plug_new,
        sub_plug_editor = nav.sub_plug_editor,
        is_seo = nav.is_seo,
        is_forms = nav.is_forms,
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
        is_engine = nav.is_engine,
        body_content = body_content
    )
}
