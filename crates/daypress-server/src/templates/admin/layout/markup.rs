//! Admin Shell HTML Markup and template rendering.

use crate::models::WpSettingsResponse;
use crate::templates::admin::layout::navigation::{render_cpt_navigation, NavState};
use crate::templates::admin::layout::styles_components::get_styles_components;
use crate::templates::admin::layout::styles_forms::get_styles_forms;
use crate::templates::admin::layout::styles_shell::{
    get_styles_shell, DESK_FONT_FACES, DESK_FOUNDATION,
};
use crate::templates::utils::escape_html;

pub fn render_wp_admin_shell(
    current_tab: &str,
    page_title: &str,
    body_content: &str,
    settings: &WpSettingsResponse,
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

    let nav = NavState::compute(current_tab);
    let custom_post_types_nav = render_cpt_navigation(current_tab, &settings.custom_post_types);
    let styles_shell = get_styles_shell();
    let styles_comp = get_styles_components();
    let styles_forms = get_styles_forms();
    let rest_nonce: String = crate::auth::current_admin()
        .map(|a| a.nonce)
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    format!(
        r##"<!DOCTYPE html>
<html lang="en-US">
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
<body>
<div id="day-app">
	<!-- The desk: wordmark, search, navigation, account -->
	<aside id="day-sidebar">
		<div class="desk-brand">
			<a href="/d-admin/" class="desk-wordmark" aria-label="Daypress desk"><span class="day-dot" aria-hidden="true"></span>daypress</a>
			<a href="/" target="_blank" rel="noopener" class="desk-site" title="View the site">{site_name} &nearr;</a>
		</div>
		<button type="button" class="desk-search" onclick="dayOpenQuickSearch()">Search <kbd>Ctrl K</kbd></button>
		<nav class="sidebar-nav">
			<!-- 1. DASHBOARD -->
			<div class="nav-group {is_dash_open}">
				<a href="/d-admin/" class="nav-item {is_dash}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
					<span>Dashboard</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/" class="nav-subitem {sub_dash_home}">Home</a>
					<a href="/d-admin/update-core" class="nav-subitem {sub_dash_updates}">Updates</a>
				</div>
			</div>

			<!-- 2. POSTS -->
			<div class="nav-group {is_posts_open}">
				<a href="/d-admin/edit" class="nav-item {is_posts}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
					<span>Posts</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/edit" class="nav-subitem {sub_posts_all}">All Posts</a>
					<a href="/d-admin/post-new" class="nav-subitem {sub_posts_new}">Add New</a>
					<a href="/d-admin/edit-tags?taxonomy=category" class="nav-subitem {sub_posts_cats}">Categories</a>
					<a href="/d-admin/edit-tags?taxonomy=post_tag" class="nav-subitem {sub_posts_tags}">Tags</a>
				</div>
			</div>

			<!-- 3. MEDIA -->
			<div class="nav-group {is_media_open}">
				<a href="/d-admin/upload" class="nav-item {is_media}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
					<span>Media</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/upload" class="nav-subitem {sub_media_lib}">Library</a>
					<a href="/d-admin/media-new" class="nav-subitem {sub_media_new}">Add New</a>
				</div>
			</div>

			<!-- 4. PAGES -->
			<div class="nav-group {is_pages_open}">
				<a href="/d-admin/edit?post_type=page" class="nav-item {is_pages}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path><polyline points="13 2 13 9 20 9"></polyline></svg>
					<span>Pages</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/edit?post_type=page" class="nav-subitem {sub_pages_all}">All Pages</a>
					<a href="/d-admin/post-new?post_type=page" class="nav-subitem {sub_pages_new}">Add New</a>
				</div>
			</div>

			<!-- 5. DYNAMIC CUSTOM POST TYPES -->
			{custom_post_types_nav}

			<!-- 5b. DAYSTUDIO -->
			<div class="nav-group">
				<a href="/d-admin/builder" class="nav-item">
					<span>DayStudio</span>
				</a>
			</div>

			<!-- 6. COMMENTS -->
			<div class="nav-group">
				<a href="/d-admin/edit-comments" class="nav-item {is_comments}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path></svg>
					<span>Comments</span>
					{comments_badge}
				</a>
			</div>

			<!-- 7. APPEARANCE -->
			<div class="nav-group {is_app_open}">
				<a href="/d-admin/headers-footers" class="nav-item {is_appearance}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
					<span>Appearance</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/headers-footers" class="nav-subitem {sub_app_hf}">Header &amp; Footer</a>
					<a href="/d-admin/customize" class="nav-subitem {sub_app_cust}">Customize</a>
					<a href="/d-admin/nav-menus" class="nav-subitem {sub_app_menus}">Menus</a>
				</div>
			</div>

			<!-- 8. PLUGINS -->
			<div class="nav-group {is_plugins_open}">
				<a href="/d-admin/plugins" class="nav-item {is_plugins}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
					<span>Plugins</span>
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
					<span>Forms Studio</span>
				</a>
			</div>

			<!-- 9. USERS -->
			<div class="nav-group {is_users_open}">
				<a href="/d-admin/users" class="nav-item {is_users}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
					<span>Users</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/users" class="nav-subitem {sub_users_all}">All Users</a>
					<a href="/d-admin/user-new" class="nav-subitem {sub_users_new}">Add New</a>
					<a href="/d-admin/profile" class="nav-subitem {sub_users_prof}">Profile</a>
				</div>
			</div>

			<!-- 10. TOOLS -->
			<div class="nav-group {is_tools_open}">
				<a href="/d-admin/tools" class="nav-item {is_tools}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path></svg>
					<span>Tools</span>
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
					<span>Settings</span>
				</a>
				<div class="nav-submenu">
					<a href="/d-admin/options-general" class="nav-subitem {sub_set_gen}">General</a>
					<a href="/d-admin/options-writing" class="nav-subitem {sub_set_writ}">Writing</a>
					<a href="/d-admin/options-reading" class="nav-subitem {sub_set_read}">Reading</a>
					<a href="/d-admin/options-discussion" class="nav-subitem {sub_set_disc}">Discussion</a>
					<a href="/d-admin/options-media" class="nav-subitem {sub_set_media}">Media</a>
					<a href="/d-admin/options-permalink" class="nav-subitem {sub_set_perm}">Permalinks</a>
					<a href="/d-admin/options-privacy" class="nav-subitem {sub_set_priv}">Privacy</a>
				</div>
			</div>

			<!-- SYSTEM TELEMETRY -->
			<div class="nav-group">
				<a href="/d-admin/daypress-engine" class="nav-item {is_engine}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
					<span>Engine Telemetry</span>
				</a>
			</div>
		</nav>

		<div class="desk-footer">
			<div class="desk-account">
				<a href="/d-admin/profile" title="Edit your profile">{account_name}</a>
				<a href="/login?action=logout" class="desk-logout">Log out</a>
			</div>
			<div class="desk-version">v{version} &middot; single binary</div>
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
		['Comments', '/d-admin/edit-comments'], ['Themes', '/d-admin/themes'], ['Header & Footer Manager', '/d-admin/headers-footers'], ['Customize', '/d-admin/customize'],
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
        sub_app_cust = nav.sub_app_cust,
        sub_app_menus = nav.sub_app_menus,
        is_plugins = nav.is_plugins,
        is_plugins_open = nav.is_plugins_open,
        sub_plug_list = nav.sub_plug_list,
        sub_plug_new = nav.sub_plug_new,
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
