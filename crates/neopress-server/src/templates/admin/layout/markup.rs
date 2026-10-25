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
{wp_admin_bar}
<div id="neo-app">
	<!-- Left Navigation Sidebar (Unified with Top Admin Bar) -->
	<aside id="neo-sidebar">
		<nav class="sidebar-nav">
			<!-- 1. DASHBOARD -->
			<div class="nav-group {is_dash_open}">
				<a href="/n-admin/" class="nav-item {is_dash}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
					<span>Dashboard</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/" class="nav-subitem {sub_dash_home}">Home</a>
					<a href="/n-admin/update-core" class="nav-subitem {sub_dash_updates}">Updates</a>
				</div>
			</div>

			<!-- 2. POSTS -->
			<div class="nav-group {is_posts_open}">
				<a href="/n-admin/edit" class="nav-item {is_posts}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
					<span>Posts</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/edit" class="nav-subitem {sub_posts_all}">All Posts</a>
					<a href="/n-admin/post-new" class="nav-subitem {sub_posts_new}">Add New</a>
					<a href="/n-admin/edit-tags?taxonomy=category" class="nav-subitem {sub_posts_cats}">Categories</a>
					<a href="/n-admin/edit-tags?taxonomy=post_tag" class="nav-subitem {sub_posts_tags}">Tags</a>
				</div>
			</div>

			<!-- 3. MEDIA -->
			<div class="nav-group {is_media_open}">
				<a href="/n-admin/upload" class="nav-item {is_media}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
					<span>Media</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/upload" class="nav-subitem {sub_media_lib}">Library</a>
					<a href="/n-admin/media-new" class="nav-subitem {sub_media_new}">Add New</a>
				</div>
			</div>

			<!-- 4. PAGES -->
			<div class="nav-group {is_pages_open}">
				<a href="/n-admin/edit?post_type=page" class="nav-item {is_pages}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path><polyline points="13 2 13 9 20 9"></polyline></svg>
					<span>Pages</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/edit?post_type=page" class="nav-subitem {sub_pages_all}">All Pages</a>
					<a href="/n-admin/post-new?post_type=page" class="nav-subitem {sub_pages_new}">Add New</a>
				</div>
			</div>

			<!-- 5. DYNAMIC CUSTOM POST TYPES -->
			{custom_post_types_nav}

			<!-- 6. COMMENTS -->
			<div class="nav-group">
				<a href="/n-admin/edit-comments" class="nav-item {is_comments}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path></svg>
					<span>Comments</span>
				</a>
			</div>

			<!-- 7. APPEARANCE -->
			<div class="nav-group {is_app_open}">
				<a href="/n-admin/headers-footers" class="nav-item {is_appearance}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
					<span>Appearance</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/headers-footers" class="nav-subitem {sub_app_hf}">Header &amp; Footer</a>
					<a href="/n-admin/customize" class="nav-subitem {sub_app_cust}">Customize</a>
					<a href="/n-admin/nav-menus" class="nav-subitem {sub_app_menus}">Menus</a>
				</div>
			</div>

			<!-- 8. PLUGINS -->
			<div class="nav-group {is_plugins_open}">
				<a href="/n-admin/plugins" class="nav-item {is_plugins}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
					<span>Plugins</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/plugins" class="nav-subitem {sub_plug_list}">Installed Plugins</a>
					<a href="/n-admin/plugin-install" class="nav-subitem {sub_plug_new}">Add New &amp; SDK</a>
				</div>
			</div>

			<!-- 8b. FORMS STUDIO -->
			<div class="nav-group">
				<a href="/n-admin/forms" class="nav-item {is_forms}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"></path><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path></svg>
					<span>Forms Studio</span>
				</a>
			</div>

			<!-- 9. USERS -->
			<div class="nav-group {is_users_open}">
				<a href="/n-admin/users" class="nav-item {is_users}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
					<span>Users</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/users" class="nav-subitem {sub_users_all}">All Users</a>
					<a href="/n-admin/user-new" class="nav-subitem {sub_users_new}">Add New</a>
					<a href="/n-admin/profile" class="nav-subitem {sub_users_prof}">Profile</a>
				</div>
			</div>

			<!-- 10. TOOLS -->
			<div class="nav-group {is_tools_open}">
				<a href="/n-admin/tools" class="nav-item {is_tools}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path></svg>
					<span>Tools</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/tools" class="nav-subitem {sub_tools_avail}">Available Tools</a>
					<a href="/n-admin/import" class="nav-subitem {sub_tools_import}">Import</a>
					<a href="/n-admin/export" class="nav-subitem {sub_tools_export}">Export</a>
					<a href="/n-admin/seo" class="nav-subitem {is_seo}">SEO &amp; Redirects</a>
					<a href="/n-admin/backups" class="nav-subitem {is_backups}">Backups</a>
					<a href="/n-admin/site-health" class="nav-subitem {sub_tools_health}">Site Health</a>
					<a href="/n-admin/export-personal-data" class="nav-subitem {sub_tools_exp_data}">Export Personal Data</a>
					<a href="/n-admin/erase-personal-data" class="nav-subitem {sub_tools_era_data}">Erase Personal Data</a>
				</div>
			</div>

			<!-- 11. SETTINGS -->
			<div class="nav-group {is_settings_open}">
				<a href="/n-admin/options-general" class="nav-item {is_settings}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
					<span>Settings</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/options-general" class="nav-subitem {sub_set_gen}">General</a>
					<a href="/n-admin/options-writing" class="nav-subitem {sub_set_writ}">Writing</a>
					<a href="/n-admin/options-reading" class="nav-subitem {sub_set_read}">Reading</a>
					<a href="/n-admin/options-discussion" class="nav-subitem {sub_set_disc}">Discussion</a>
					<a href="/n-admin/options-media" class="nav-subitem {sub_set_media}">Media</a>
					<a href="/n-admin/options-permalink" class="nav-subitem {sub_set_perm}">Permalinks</a>
					<a href="/n-admin/options-privacy" class="nav-subitem {sub_set_priv}">Privacy</a>
				</div>
			</div>

			<!-- SYSTEM TELEMETRY -->
			<div class="nav-group">
				<a href="/n-admin/neopress-engine" class="nav-item {is_engine}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
					<span>Engine Telemetry</span>
				</a>
			</div>
		</nav>

		<div class="sidebar-footer" onclick="cycleFerrisTip()" title="Click for another tip">
			<div class="ferris-tip-header">
				<span class="ferris-crab">🦀</span>
				<span class="ferris-tip-label">Pro Tip</span>
				<span class="ferris-tip-next">↻</span>
			</div>
			<p id="ferrisTipText" class="ferris-tip-body">Open any page in <strong>⚡ NeoStudio</strong> to visually inspect and edit blocks in real time.</p>
		</div>
	</aside>

	<!-- Main Workspace -->
	<main id="neo-main">
		<div id="neo-canvas">
			{body_content}
		</div>
	</main>
</div>

<!-- Global Toast -->
<div id="neo-toast"></div>

<!-- Quick Search (Ctrl+K) -->
<div id="neo-qs-backdrop" onclick="if (event.target === this) neoCloseQuickSearch();" style="display:none; position:fixed; inset:0; background:rgba(15,23,42,0.45); z-index:100000; align-items:flex-start; justify-content:center; padding-top:12vh;">
	<div role="dialog" aria-modal="true" aria-label="Quick search" style="width:min(640px, 92vw); background:#fff; border-radius:12px; box-shadow:0 25px 50px -12px rgba(0,0,0,0.35); overflow:hidden;">
		<input id="neo-qs-input" type="search" autocomplete="off" placeholder="Search posts, pages and admin screens…" style="width:100%; box-sizing:border-box; border:0; border-bottom:1px solid #e2e8f0; padding:16px 18px; font-size:15px; outline:none;" />
		<div id="neo-qs-results" style="max-height:52vh; overflow-y:auto; padding:6px 0;"></div>
		<div style="padding:8px 14px; font-size:11px; color:#94a3b8; border-top:1px solid #f1f5f9;">↑ ↓ to navigate · Enter to open · Esc to close</div>
	</div>
</div>

<script>
	const NEO_QS_SCREENS = [
		['Dashboard', '/n-admin/'], ['All Posts', '/n-admin/edit'], ['Add New Post', '/n-admin/post-new'],
		['Categories', '/n-admin/edit-tags?taxonomy=category'], ['Tags', '/n-admin/edit-tags?taxonomy=post_tag'],
		['Media Library', '/n-admin/upload'], ['Upload Media', '/n-admin/media-new'],
		['All Pages', '/n-admin/edit?post_type=page'], ['Add New Page', '/n-admin/post-new?post_type=page'],
		['Comments', '/n-admin/edit-comments'], ['Themes', '/n-admin/themes'], ['Header & Footer Manager', '/n-admin/headers-footers'], ['Customize', '/n-admin/customize'],
		['Widgets', '/n-admin/widgets'], ['Menus', '/n-admin/nav-menus'], ['Theme File Editor', '/n-admin/theme-editor'],
		['Installed Plugins', '/n-admin/plugins'], ['Add New Plugin', '/n-admin/plugin-install'],
		['Forms Studio', '/n-admin/forms'], ['Users', '/n-admin/users'], ['Add New User', '/n-admin/user-new'],
		['Profile', '/n-admin/profile'], ['Tools', '/n-admin/tools'], ['Import', '/n-admin/import'],
		['Export', '/n-admin/export'], ['SEO & Redirects', '/n-admin/seo'], ['Backups', '/n-admin/backups'],
		['Site Health', '/n-admin/site-health'], ['Export Personal Data', '/n-admin/export-personal-data'],
		['Erase Personal Data', '/n-admin/erase-personal-data'], ['General Settings', '/n-admin/options-general'],
		['Writing Settings', '/n-admin/options-writing'], ['Reading Settings', '/n-admin/options-reading'],
		['Discussion Settings', '/n-admin/options-discussion'], ['Media Settings', '/n-admin/options-media'],
		['Permalink Settings', '/n-admin/options-permalink'], ['Privacy Settings', '/n-admin/options-privacy'],
		['Engine Telemetry', '/n-admin/neopress-engine'], ['Analytics', '/n-admin/analytics']
	];
	let neoQsItems = [];
	let neoQsActive = 0;
	let neoQsTimer = null;
	let neoQsSeq = 0;

	function neoQsEscape(s) {{
		return String(s == null ? '' : s).replace(/[&<>"']/g, c => ({{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}}[c]));
	}}
	function neoOpenQuickSearch() {{
		const bd = document.getElementById('neo-qs-backdrop');
		const input = document.getElementById('neo-qs-input');
		if (!bd || !input) return;
		bd.style.display = 'flex';
		input.value = '';
		neoQsRender('', []);
		setTimeout(() => input.focus(), 0);
	}}
	function neoCloseQuickSearch() {{
		const bd = document.getElementById('neo-qs-backdrop');
		if (bd) bd.style.display = 'none';
	}}
	function neoQsRender(query, contentResults) {{
		const q = query.trim().toLowerCase();
		const screens = NEO_QS_SCREENS
			.filter(([label]) => !q || label.toLowerCase().includes(q))
			.slice(0, q ? 6 : 8)
			.map(([label, url]) => ({{ label, url, kind: 'Screen' }}));
		neoQsItems = contentResults.concat(screens);
		neoQsActive = 0;
		const box = document.getElementById('neo-qs-results');
		if (!box) return;
		if (!neoQsItems.length) {{
			box.innerHTML = '<div style="padding:14px 18px; color:#94a3b8; font-size:13px;">No matches</div>';
			return;
		}}
		box.innerHTML = neoQsItems.map((it, i) =>
			'<a href="' + neoQsEscape(it.url) + '" data-qs-idx="' + i + '" style="display:flex; justify-content:space-between; gap:12px; padding:9px 18px; text-decoration:none; color:#0f172a; font-size:13.5px;' + (i === 0 ? ' background:#eff6ff;' : '') + '">' +
			'<span>' + neoQsEscape(it.label) + '</span><span style="color:#94a3b8; font-size:11.5px; text-transform:uppercase; letter-spacing:.04em;">' + neoQsEscape(it.kind) + '</span></a>'
		).join('');
	}}
	function neoQsHighlight() {{
		document.querySelectorAll('#neo-qs-results [data-qs-idx]').forEach(el => {{
			el.style.background = Number(el.dataset.qsIdx) === neoQsActive ? '#eff6ff' : '';
			if (Number(el.dataset.qsIdx) === neoQsActive) el.scrollIntoView({{ block: 'nearest' }});
		}});
	}}
	async function neoQsSearch(query) {{
		const seq = ++neoQsSeq;
		const q = query.trim();
		if (q.length < 2) {{ neoQsRender(query, []); return; }}
		let content = [];
		try {{
			const res = await fetch('/wp-json/wp/v2/search?per_page=8&search=' + encodeURIComponent(q), {{ credentials: 'same-origin' }});
			if (res.ok) {{
				const rows = await res.json();
				content = (Array.isArray(rows) ? rows : []).map(r => ({{
					label: (r.title || '(no title)').replace(/<[^>]*>/g, ''),
					url: '/n-admin/post?post=' + encodeURIComponent(r.id),
					kind: r.subtype || 'post'
				}}));
			}}
		}} catch (err) {{ /* network errors fall back to screen matches */ }}
		if (seq === neoQsSeq) neoQsRender(query, content);
	}}
	document.addEventListener('DOMContentLoaded', () => {{
		const input = document.getElementById('neo-qs-input');
		if (!input) return;
		input.addEventListener('input', () => {{
			clearTimeout(neoQsTimer);
			neoQsRender(input.value, []);
			neoQsTimer = setTimeout(() => neoQsSearch(input.value), 180);
		}});
		input.addEventListener('keydown', (e) => {{
			if (e.key === 'ArrowDown') {{ e.preventDefault(); neoQsActive = Math.min(neoQsActive + 1, neoQsItems.length - 1); neoQsHighlight(); }}
			else if (e.key === 'ArrowUp') {{ e.preventDefault(); neoQsActive = Math.max(neoQsActive - 1, 0); neoQsHighlight(); }}
			else if (e.key === 'Enter') {{ e.preventDefault(); const it = neoQsItems[neoQsActive]; if (it) window.location.href = it.url; }}
			else if (e.key === 'Escape') {{ neoCloseQuickSearch(); }}
		}});
	}});
	// Registered last: screens with their own Ctrl+K (e.g. the editor's "insert link") call preventDefault first.
	document.addEventListener('keydown', (e) => {{
		if (e.defaultPrevented) return;
		const mod = e.ctrlKey || e.metaKey;
		if (mod && (e.key === 'k' || e.key === 'K')) {{
			e.preventDefault();
			neoOpenQuickSearch();
		}} else if (e.key === 'Escape') {{
			neoCloseQuickSearch();
		}}
	}});
</script>

<script>
	const FERRIS_TIPS = [
		'Open any page in <strong>⚡ NeoStudio</strong> to visually inspect and edit blocks in real time.',
		'Drop any WordPress <code>.xml</code> export in <strong>Tools &rarr; Import</strong> for 1-click migration.',
		'Every change saves directly to embedded SQLite in <strong>&lt;0.15ms</strong> with zero cache bloat.',
		'Use <strong>SEO &amp; Redirects</strong> to preview Google SERP snippets and manage 301 rules.',
		'Take instant zero-downtime SQLite snapshots anytime in <strong>Backups</strong>.'
	];
	let ferrisTipIdx = 0;
	function cycleFerrisTip() {{
		ferrisTipIdx = (ferrisTipIdx + 1) % FERRIS_TIPS.length;
		const el = document.getElementById('ferrisTipText');
		if (el) el.innerHTML = FERRIS_TIPS[ferrisTipIdx];
	}}
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
        rest_nonce = rest_nonce,
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
