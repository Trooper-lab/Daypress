//! Public Base HTML Document Shell (`templates/public/shell.rs`).
//!
//! Renders SEO meta tags, OpenGraph cards, Schema.org JSON-LD structured data,
//! Cookieless Analytics beacon, GDPR Cookie Consent Banner, Admin Bar,
//! dynamic Theme Builder Header/Footer, scoped responsive stylesheets,
//! and 3-tier DOM leak sanitizer.

use super::admin_bar::render_admin_bar;
pub use super::head_meta::plain_text_excerpt;
use super::head_meta::{render_head_meta, HeadMetaOutput};
use super::site_chrome;
use crate::auth::AuthUser;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::utils::escape_html;
use crate::theme_builder::models::ResolvedThemePartials;
use daypress_ast::sanitize_ssr_html;

fn render_cookie_consent_banner(settings: &DaypressSettingsResponse) -> String {
    let priv_cfg = &settings.privacy_settings;
    if !priv_cfg.cookie_banner_enabled {
        return String::new();
    }

    let pos_class = match priv_cfg.cookie_banner_position.as_str() {
        "floating-right" => "dp-cc-floating-right",
        "floating-left" => "dp-cc-floating-left",
        _ => "dp-cc-bottom-bar",
    };

    let title = if priv_cfg.cookie_banner_title.trim().is_empty() {
        "Privacy & Cookie Preferences"
    } else {
        priv_cfg.cookie_banner_title.trim()
    };
    let msg = if priv_cfg.cookie_banner_message.trim().is_empty() {
        "We use essential cookies to keep this site reliable and cookieless first-party analytics to improve your experience."
    } else {
        priv_cfg.cookie_banner_message.trim()
    };
    let lbl_accept = if priv_cfg.cookie_accept_all_label.trim().is_empty() {
        "Accept All"
    } else {
        priv_cfg.cookie_accept_all_label.trim()
    };
    let lbl_reject = if priv_cfg.cookie_reject_label.trim().is_empty() {
        "Essential Only"
    } else {
        priv_cfg.cookie_reject_label.trim()
    };
    let lbl_manage = if priv_cfg.cookie_manage_label.trim().is_empty() {
        "Preferences"
    } else {
        priv_cfg.cookie_manage_label.trim()
    };

    let analytics_tpl = if !priv_cfg.analytics_scripts_html.trim().is_empty() {
        format!(
            r#"<template id="dp-cc-tpl-analytics">{}</template>"#,
            priv_cfg.analytics_scripts_html
        )
    } else {
        String::new()
    };
    let marketing_tpl = if !priv_cfg.marketing_scripts_html.trim().is_empty() {
        format!(
            r#"<template id="dp-cc-tpl-marketing">{}</template>"#,
            priv_cfg.marketing_scripts_html
        )
    } else {
        String::new()
    };

    format!(
        r##"<style id="dp-cookie-consent-css">
.dp-cc-banner{{position:fixed;z-index:99998;background:var(--paper-raised,#fbf8f2);color:var(--ink,#16171a);border:1px solid var(--rule,#16171a2e);border-top:2px solid var(--rule-strong,#16171a);box-shadow:0 16px 40px rgba(22,23,26,.16);font-family:var(--font-sans,system-ui,-apple-system,sans-serif);font-size:13.5px;line-height:1.5;display:none;box-sizing:border-box}}
.dp-cc-bottom-bar{{left:16px;right:16px;bottom:16px;max-width:980px;margin:0 auto;padding:14px 20px;border-radius:2px;align-items:center;justify-content:space-between;gap:18px;flex-wrap:wrap}}
.dp-cc-floating-right{{right:20px;bottom:20px;max-width:400px;padding:18px;border-radius:2px;flex-direction:column;gap:14px}}
.dp-cc-floating-left{{left:20px;bottom:20px;max-width:400px;padding:18px;border-radius:2px;flex-direction:column;gap:14px}}
.dp-cc-title{{font-family:var(--font-mono,monospace);font-weight:600;font-size:12px;text-transform:uppercase;letter-spacing:.08em;margin:0 0 4px;color:var(--ink,#16171a)}}
.dp-cc-msg{{margin:0;color:var(--ink-muted,#5a5852);font-size:13px}}
.dp-cc-actions{{display:flex;gap:8px;flex-wrap:wrap;align-items:center;flex-shrink:0}}
.dp-cc-btn{{border:1px solid var(--rule,#16171a2e);background:var(--paper,#f3eee4);color:var(--ink,#16171a);padding:7px 13px;border-radius:2px;font-family:var(--font-mono,monospace);font-size:11.5px;font-weight:600;text-transform:uppercase;letter-spacing:.05em;cursor:pointer}}
.dp-cc-btn-primary{{background:var(--oxide,#a8401b);border-color:var(--oxide,#a8401b);color:var(--on-oxide,#fff8f0)}}
.dp-cc-modal{{position:fixed;inset:0;z-index:99999;background:rgba(22,23,26,.55);display:none;align-items:center;justify-content:center;padding:16px}}
.dp-cc-dialog{{background:var(--paper-raised,#fbf8f2);color:var(--ink,#16171a);border:1px solid var(--rule,#16171a2e);border-top:2px solid var(--rule-strong,#16171a);border-radius:2px;max-width:460px;width:100%;padding:20px;box-sizing:border-box}}
.dp-cc-cat{{display:flex;justify-content:space-between;align-items:center;padding:10px 0;border-bottom:1px solid var(--rule,#16171a2e);font-size:13px}}
</style>
<div id="dp-cookie-banner" class="dp-cc-banner {pos_class}" role="region" aria-label="{title}">
	<div style="flex:1;min-width:240px;">
		<div class="dp-cc-title">{title}</div>
		<p class="dp-cc-msg">{msg}</p>
	</div>
	<div class="dp-cc-actions">
		<button type="button" class="dp-cc-btn dp-cc-btn-primary" onclick="window.daypressSetCookieConsent(true,true)">{lbl_accept}</button>
		<button type="button" class="dp-cc-btn" onclick="window.daypressSetCookieConsent(false,false)">{lbl_reject}</button>
		<button type="button" class="dp-cc-btn" onclick="window.daypressOpenCookiePreferences()">{lbl_manage}</button>
	</div>
</div>
<div id="dp-cookie-prefs-modal" class="dp-cc-modal" role="dialog" aria-modal="true" aria-label="Cookie Preferences">
	<div class="dp-cc-dialog">
		<div class="dp-cc-title" style="margin-bottom:8px;">{title}</div>
		<p class="dp-cc-msg" style="margin-bottom:12px;">Choose which cookie categories you allow on this device.</p>
		<div class="dp-cc-cat"><span><strong>Essential</strong> (Security &amp; Session)</span><input type="checkbox" checked disabled /></div>
		<div class="dp-cc-cat"><span><strong>Analytics</strong> (Traffic &amp; Usage)</span><input type="checkbox" id="dp-cc-chk-analytics" checked /></div>
		<div class="dp-cc-cat" style="border-bottom:none;"><span><strong>Marketing</strong> (Campaigns &amp; Pixels)</span><input type="checkbox" id="dp-cc-chk-marketing" /></div>
		<div class="dp-cc-actions" style="margin-top:14px;justify-content:flex-end;">
			<button type="button" class="dp-cc-btn" onclick="document.getElementById('dp-cookie-prefs-modal').style.display='none'">Cancel</button>
			<button type="button" class="dp-cc-btn dp-cc-btn-primary" onclick="window.daypressSaveModalConsent()">Save Preferences</button>
		</div>
	</div>
</div>
{analytics_tpl}{marketing_tpl}
<script>
(function(){{
	var KEY = 'daypress_cookie_consent';
	var activated = {{}};
	function activateCategory(cat) {{
		if (activated[cat]) return;
		activated[cat] = true;
		document.querySelectorAll('script[type="text/plain"][data-cookie-category="' + cat + '"]').forEach(function(oldScript) {{
			var s = document.createElement('script');
			Array.from(oldScript.attributes).forEach(function(attr) {{
				if (attr.name !== 'type' && attr.name !== 'data-cookie-category') s.setAttribute(attr.name, attr.value);
			}});
			s.text = oldScript.textContent || '';
			oldScript.parentNode.replaceChild(s, oldScript);
		}});
		var tpl = document.getElementById('dp-cc-tpl-' + cat);
		if (tpl && tpl.content) {{
			document.head.appendChild(tpl.content.cloneNode(true));
		}}
	}}
	function applyConsent(c) {{
		if (!c) return;
		if (c.analytics) activateCategory('analytics');
		if (c.marketing) activateCategory('marketing');
		try {{ window.dispatchEvent(new CustomEvent('daypress:consent', {{ detail: c }})); }} catch(e) {{}}
	}}
	window.daypressGetCookieConsent = function() {{
		try {{
			var raw = localStorage.getItem(KEY);
			return raw ? JSON.parse(raw) : null;
		}} catch(e) {{ return null; }}
	}};
	window.daypressSetCookieConsent = function(analytics, marketing) {{
		var c = {{ essential: true, analytics: !!analytics, marketing: !!marketing, ts: Date.now() }};
		try {{
			localStorage.setItem(KEY, JSON.stringify(c));
			document.cookie = 'daypress_consent=' + (c.analytics ? '1' : '0') + (c.marketing ? '1' : '0') + '; path=/; max-age=31536000; SameSite=Lax';
		}} catch(e) {{}}
		var b = document.getElementById('dp-cookie-banner');
		if (b) b.style.display = 'none';
		var m = document.getElementById('dp-cookie-prefs-modal');
		if (m) m.style.display = 'none';
		applyConsent(c);
	}};
	window.daypressOpenCookiePreferences = function() {{
		var c = window.daypressGetCookieConsent() || {{ analytics: true, marketing: false }};
		var ca = document.getElementById('dp-cc-chk-analytics');
		var cm = document.getElementById('dp-cc-chk-marketing');
		if (ca) ca.checked = !!c.analytics;
		if (cm) cm.checked = !!c.marketing;
		var m = document.getElementById('dp-cookie-prefs-modal');
		if (m) m.style.display = 'flex';
	}};
	window.daypressSaveModalConsent = function() {{
		var ca = document.getElementById('dp-cc-chk-analytics');
		var cm = document.getElementById('dp-cc-chk-marketing');
		window.daypressSetCookieConsent(ca ? ca.checked : false, cm ? cm.checked : false);
	}};
	var existing = window.daypressGetCookieConsent();
	if (existing) {{
		applyConsent(existing);
	}} else {{
		var b = document.getElementById('dp-cookie-banner');
		if (b) b.style.display = 'flex';
	}}
}})();
</script>"##,
        pos_class = pos_class,
        title = escape_html(title),
        msg = escape_html(msg),
        lbl_accept = escape_html(lbl_accept),
        lbl_reject = escape_html(lbl_reject),
        lbl_manage = escape_html(lbl_manage),
        analytics_tpl = analytics_tpl,
        marketing_tpl = marketing_tpl,
    )
}

/// Renders the drop-in `Cmd+K` / `Ctrl+K` SQLite FTS5 Instant Search Modal in the Daypress Editorial Design System.
fn render_cmdk_search_modal() -> &'static str {
    r##"<style id="dp-cmdk-search-css">
.dp-search-trigger {
	display: inline-flex;
	align-items: center;
	gap: 7px;
	height: 32px;
	padding: 0 10px;
	background: var(--paper, #f3eee4);
	color: var(--ink-muted, #5a5852);
	border: 1px solid var(--rule, #16171a2e);
	border-radius: 2px;
	font-family: var(--font-sans, sans-serif);
	font-size: 12.5px;
	font-weight: 500;
	cursor: pointer;
	transition: border-color 0.15s ease, color 0.15s ease;
}
.dp-search-trigger:hover {
	border-color: var(--rule-strong, #16171a);
	color: var(--ink, #16171a);
}
.dp-search-kbd {
	font-family: var(--font-mono, monospace);
	font-size: 10.5px;
	padding: 1px 5px;
	background: var(--paper-raised, #fbf8f2);
	border: 1px solid var(--rule, #16171a2e);
	border-radius: 2px;
	color: var(--ink-muted, #5a5852);
}
.dp-cmdk-backdrop {
	position: fixed;
	inset: 0;
	z-index: 99990;
	background: rgba(16, 17, 20, 0.62);
	backdrop-filter: blur(4px);
	-webkit-backdrop-filter: blur(4px);
	display: none;
	align-items: flex-start;
	justify-content: center;
	padding: 9vh 16px 24px;
}
.dp-cmdk-backdrop.is-open {
	display: flex;
}
.dp-cmdk-dialog {
	width: 100%;
	max-width: 640px;
	background: var(--paper-raised, #fbf8f2);
	color: var(--ink, #16171a);
	border: 1px solid var(--rule, #16171a2e);
	border-top: 2px solid var(--rule-strong, #16171a);
	border-radius: 2px;
	box-shadow: 0 24px 60px rgba(0, 0, 0, 0.28);
	overflow: hidden;
	display: flex;
	flex-direction: column;
	max-height: 78vh;
}
.dp-cmdk-header {
	display: flex;
	align-items: center;
	gap: 12px;
	padding: 14px 18px;
	border-bottom: 1px solid var(--rule, #16171a2e);
	background: var(--paper-raised, #fbf8f2);
}
.dp-cmdk-icon {
	color: var(--oxide, #a8401b);
	font-size: 15px;
	flex-shrink: 0;
}
.dp-cmdk-input {
	flex: 1;
	border: none;
	outline: none;
	background: transparent;
	color: var(--ink, #16171a);
	font-family: var(--font-sans, sans-serif);
	font-size: 16px;
	line-height: 1.4;
}
.dp-cmdk-input::placeholder {
	color: var(--ink-muted, #5a5852);
	opacity: 0.75;
}
.dp-cmdk-latency {
	font-family: var(--font-mono, monospace);
	font-size: 10.5px;
	text-transform: uppercase;
	letter-spacing: 0.06em;
	color: var(--ink-muted, #5a5852);
	padding: 2px 7px;
	background: var(--paper, #f3eee4);
	border: 1px solid var(--rule, #16171a2e);
	border-radius: 2px;
	white-space: nowrap;
}
.dp-cmdk-filters {
	display: flex;
	align-items: center;
	gap: 6px;
	padding: 8px 18px;
	background: var(--paper, #f3eee4);
	border-bottom: 1px solid var(--rule, #16171a2e);
	overflow-x: auto;
}
.dp-cmdk-filter-pill {
	font-family: var(--font-mono, monospace);
	font-size: 11px;
	text-transform: uppercase;
	letter-spacing: 0.06em;
	padding: 3px 9px;
	border: 1px solid var(--rule, #16171a2e);
	border-radius: 2px;
	background: var(--paper-raised, #fbf8f2);
	color: var(--ink-muted, #5a5852);
	cursor: pointer;
}
.dp-cmdk-filter-pill.is-active {
	background: var(--ink, #16171a);
	color: var(--paper-raised, #fbf8f2);
	border-color: var(--ink, #16171a);
}
.dp-cmdk-typo-bar {
	display: none;
	padding: 7px 18px;
	font-family: var(--font-mono, monospace);
	font-size: 11.5px;
	background: color-mix(in srgb, var(--ochre, #d9a42a) 16%, var(--paper-raised, #fbf8f2));
	color: var(--ink, #16171a);
	border-bottom: 1px solid var(--rule, #16171a2e);
}
.dp-cmdk-results {
	overflow-y: auto;
	padding: 6px 0;
	flex: 1;
}
.dp-cmdk-item {
	display: block;
	padding: 12px 18px;
	text-decoration: none;
	color: var(--ink, #16171a);
	border-bottom: 1px solid var(--rule, #16171a2e);
	transition: background 0.1s ease;
}
.dp-cmdk-item:last-child {
	border-bottom: none;
}
.dp-cmdk-item:hover,
.dp-cmdk-item.is-selected {
	background: var(--paper, #f3eee4);
}
.dp-cmdk-item-top {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 8px;
	margin-bottom: 4px;
}
.dp-cmdk-item-title {
	font-family: var(--font-serif, Georgia, serif);
	font-size: 17px;
	font-weight: 500;
	color: var(--ink, #16171a);
	line-height: 1.25;
}
.dp-cmdk-item-meta {
	display: inline-flex;
	align-items: center;
	gap: 6px;
	font-family: var(--font-mono, monospace);
	font-size: 10.5px;
	text-transform: uppercase;
	letter-spacing: 0.06em;
	color: var(--ink-muted, #5a5852);
	flex-shrink: 0;
}
.dp-cmdk-type-badge {
	padding: 1px 6px;
	background: var(--paper, #f3eee4);
	border: 1px solid var(--rule, #16171a2e);
	color: var(--oxide, #a8401b);
	border-radius: 2px;
	font-weight: 600;
}
.dp-cmdk-item-snippet {
	font-family: var(--font-sans, sans-serif);
	font-size: 13px;
	color: var(--ink-muted, #5a5852);
	line-height: 1.5;
}
mark.dp-search-hl {
	background: color-mix(in srgb, var(--ochre, #d9a42a) 34%, transparent);
	color: var(--ink, #16171a);
	padding: 0 2px;
	border-radius: 1px;
	font-weight: 600;
}
.dp-cmdk-empty {
	padding: 28px 18px;
	text-align: center;
	font-size: 13.5px;
	color: var(--ink-muted, #5a5852);
}
.dp-cmdk-footer {
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding: 8px 18px;
	background: var(--paper, #f3eee4);
	border-top: 1px solid var(--rule, #16171a2e);
	font-family: var(--font-mono, monospace);
	font-size: 11px;
	color: var(--ink-muted, #5a5852);
}
</style>
<div id="dp-cmdk-modal" class="dp-cmdk-backdrop" role="dialog" aria-modal="true" aria-label="Site Instant Search">
	<div class="dp-cmdk-dialog">
		<div class="dp-cmdk-header">
			<i class="fa-solid fa-magnifying-glass dp-cmdk-icon" aria-hidden="true"></i>
			<input type="search" id="dp-cmdk-input" class="dp-cmdk-input" placeholder="Search articles, pages, features... (SQLite FTS5)" autocomplete="off" />
			<span id="dp-cmdk-latency" class="dp-cmdk-latency">FTS5 READY</span>
		</div>
		<div class="dp-cmdk-filters" id="dp-cmdk-filters">
			<button type="button" class="dp-cmdk-filter-pill is-active" data-pt="all">All</button>
			<button type="button" class="dp-cmdk-filter-pill" data-pt="page">Pages</button>
			<button type="button" class="dp-cmdk-filter-pill" data-pt="post">Articles</button>
			<button type="button" class="dp-cmdk-filter-pill" data-pt="feature">Features</button>
		</div>
		<div id="dp-cmdk-typo" class="dp-cmdk-typo-bar"></div>
		<div id="dp-cmdk-results" class="dp-cmdk-results">
			<div class="dp-cmdk-empty">Type at least 2 characters to search with BM25 ranking &amp; typo tolerance.</div>
		</div>
		<div class="dp-cmdk-footer">
			<span><kbd class="dp-search-kbd">&uarr;&darr;</kbd> Navigate &nbsp;&bull;&nbsp; <kbd class="dp-search-kbd">&crarr;</kbd> Open &nbsp;&bull;&nbsp; <kbd class="dp-search-kbd">ESC</kbd> Close</span>
			<span>SQLite FTS5 &bull; BM25</span>
		</div>
	</div>
</div>
<script>
(function(){
	var modal = document.getElementById('dp-cmdk-modal');
	var input = document.getElementById('dp-cmdk-input');
	var resultsEl = document.getElementById('dp-cmdk-results');
	var latencyEl = document.getElementById('dp-cmdk-latency');
	var typoEl = document.getElementById('dp-cmdk-typo');
	var filtersEl = document.getElementById('dp-cmdk-filters');
	if (!modal || !input || !resultsEl) return;

	var activeType = 'all';
	var selectedIdx = 0;
	var currentItems = [];
	var debounceTimer = null;

	function openSearch(initialQuery) {
		modal.classList.add('is-open');
		if (typeof initialQuery === 'string' && initialQuery.trim()) {
			input.value = initialQuery.trim();
		}
		setTimeout(function(){ input.focus(); input.select(); }, 20);
		if (input.value.trim().length >= 2) {
			runSearch(input.value.trim());
		}
	}
	function closeSearch() {
		modal.classList.remove('is-open');
	}
	window.daypressOpenSearch = openSearch;
	window.daypressCloseSearch = closeSearch;

	modal.addEventListener('click', function(e) {
		if (e.target === modal) closeSearch();
	});

	if (filtersEl) {
		filtersEl.addEventListener('click', function(e) {
			var btn = e.target.closest('.dp-cmdk-filter-pill');
			if (!btn) return;
			filtersEl.querySelectorAll('.dp-cmdk-filter-pill').forEach(function(b){ b.classList.remove('is-active'); });
			btn.classList.add('is-active');
			activeType = btn.getAttribute('data-pt') || 'all';
			if (input.value.trim().length >= 2) {
				runSearch(input.value.trim());
			}
		});
	}

	document.addEventListener('click', function(e) {
		var trig = e.target && e.target.closest ? e.target.closest('[data-dp-search-open], .dp-search-trigger, a[href="#search"]') : null;
		if (trig) {
			e.preventDefault();
			openSearch('');
		}
	});

	document.addEventListener('keydown', function(e) {
		if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) {
			e.preventDefault();
			if (modal.classList.contains('is-open')) closeSearch();
			else openSearch('');
			return;
		}
		if (!modal.classList.contains('is-open')) return;
		if (e.key === 'Escape') {
			e.preventDefault();
			closeSearch();
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (currentItems.length > 0) {
				selectedIdx = (selectedIdx + 1) % currentItems.length;
				updateSelection();
			}
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (currentItems.length > 0) {
				selectedIdx = (selectedIdx - 1 + currentItems.length) % currentItems.length;
				updateSelection();
			}
		} else if (e.key === 'Enter') {
			if (currentItems[selectedIdx]) {
				e.preventDefault();
				window.location.href = currentItems[selectedIdx].link;
			}
		}
	});

	input.addEventListener('input', function() {
		clearTimeout(debounceTimer);
		var q = input.value.trim();
		if (q.length < 2) {
			typoEl.style.display = 'none';
			resultsEl.innerHTML = '<div class="dp-cmdk-empty">Type at least 2 characters to search with BM25 ranking &amp; typo tolerance.</div>';
			currentItems = [];
			return;
		}
		debounceTimer = setTimeout(function(){ runSearch(q); }, 70);
	});

	function updateSelection() {
		var rows = resultsEl.querySelectorAll('.dp-cmdk-item');
		rows.forEach(function(r, idx) {
			if (idx === selectedIdx) {
				r.classList.add('is-selected');
				if (r.scrollIntoView) r.scrollIntoView({ block: 'nearest' });
			} else {
				r.classList.remove('is-selected');
			}
		});
	}

	function escHtml(s) {
		return String(s || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
	}

	function renderSearchResults(data, q) {
		var ms = ((data.took_us || 0) / 1000).toFixed(2);
		latencyEl.textContent = 'FTS5 • ' + ms + 'ms';
		currentItems = data.results || [];
		selectedIdx = 0;
		if (data.typo_corrected) {
			typoEl.style.display = 'block';
			typoEl.textContent = 'Fuzzy typo-tolerant match for "' + data.typo_corrected + '"';
		} else {
			typoEl.style.display = 'none';
		}
		if (currentItems.length === 0) {
			resultsEl.innerHTML = '<div class="dp-cmdk-empty">No matching results found for <strong>' + escHtml(q) + '</strong>.</div>';
			return;
		}
		var html = '';
		for (var i = 0; i < currentItems.length; i++) {
			var it = currentItems[i];
			var sel = i === 0 ? ' is-selected' : '';
			html += '<a class="dp-cmdk-item' + sel + '" href="' + escHtml(it.link) + '">'
				+ '<div class="dp-cmdk-item-top">'
				+ '<span class="dp-cmdk-item-title">' + escHtml(it.title) + '</span>'
				+ '<span class="dp-cmdk-item-meta"><span class="dp-cmdk-type-badge">' + escHtml(it.post_type_label || it.post_type) + '</span><span>' + (it.reading_time_minutes || 1) + ' min read</span></span>'
				+ '</div>'
				+ '<div class="dp-cmdk-item-snippet">' + (it.snippet_html || escHtml(it.excerpt)) + '</div>'
				+ '</a>';
		}
		resultsEl.innerHTML = html;
	}
	window.daypressRenderSearchResults = renderSearchResults;

	function runSearch(q) {
		if (window.DAYPRESS_PRELOADED_SEARCH_RESPONSE) {
			renderSearchResults(window.DAYPRESS_PRELOADED_SEARCH_RESPONSE, q);
			return;
		}
		var url = '/wp-json/daypress/v1/search?q=' + encodeURIComponent(q) + '&limit=12';
		if (activeType && activeType !== 'all') {
			url += '&post_type=' + encodeURIComponent(activeType);
		}
		fetch(url)
			.then(function(r){ return r.json(); })
			.then(function(data) {
				renderSearchResults(data, q);
			})
			.catch(function(){});
	}
})();
</script>"##
}

/// Renders the unified Daypress HTML shell driven 100% by database settings,
/// dynamic Theme Builder partials, and Daypress AST element styles.
pub fn render_base_layout(
    title: &str,
    settings: &DaypressSettingsResponse,
    content: &str,
    current_user: Option<&AuthUser>,
    current_post: Option<&DaypressPostResponse>,
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
        schema_type: _,
        schema_json_ld,
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
            crate::templates::utils::escape_style_tag_contents(typography_css)
        )
    };
    let custom_css_block = if !settings.custom_css.trim().is_empty() {
        format!(
            "{}\n\t<style id=\"daypress-custom-css\">\n{}\n\t</style>",
            typography_block,
            crate::templates::utils::escape_style_tag_contents(&settings.custom_css)
        )
    } else {
        typography_block
    };

    let is_blank_template = site_chrome::is_blank_template(current_post);

    let daystudio_kit_css_block = if !settings.daystudio_kit_css.trim().is_empty() {
        format!(
            "\n\t<style id=\"daystudio-kit-css\">\n{}\n\t</style>",
            crate::templates::utils::escape_style_tag_contents(&settings.daystudio_kit_css)
        )
    } else {
        String::new()
    };

    let favicon_href = {
        let safe = daypress_ast::sanitize_media_url(settings.site_favicon_url.trim());
        if !safe.is_empty() {
            escape_html(&safe)
        } else {
            "/static/favicon.svg".to_string()
        }
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
                crate::templates::utils::escape_style_tag_contents(css)
            )
        })
        .unwrap_or_default();

    let theme_footer_css = partials
        .and_then(|p| p.footer_css.as_ref())
        .map(|css| {
            format!(
                "\n\t<style id=\"dp-theme-footer-css\">\n{}\n\t</style>",
                crate::templates::utils::escape_style_tag_contents(css)
            )
        })
        .unwrap_or_default();

    let theme_popups_css = partials
        .and_then(|p| p.popups_css.as_ref())
        .filter(|css| !css.trim().is_empty())
        .map(|css| {
            format!(
                "\n\t<style id=\"dp-theme-popups-css\">\n{}\n\t</style>",
                crate::templates::utils::escape_style_tag_contents(css)
            )
        })
        .unwrap_or_default();

    let theme_popups_html = partials
        .and_then(|p| p.popups_html.as_ref())
        .map(|html| html.as_str())
        .unwrap_or("");

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

    let gate_analytics = if settings.privacy_settings.cookie_banner_enabled
        && settings.privacy_settings.gate_builtin_analytics
    {
        "true"
    } else {
        "false"
    };
    let cookie_consent_html = render_cookie_consent_banner(settings);
    let cmdk_search_html = render_cmdk_search_modal();
    let html_lang = current_post
        .map(|p| p.lang_code.trim())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let s = settings.language.trim();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        })
        .unwrap_or("en")
        .replace('_', "-");
    let lang_lower = html_lang.to_ascii_lowercase();
    let html_dir = if lang_lower.starts_with("ar")
        || lang_lower.starts_with("he")
        || lang_lower.starts_with("fa")
        || lang_lower.starts_with("ur")
        || lang_lower.starts_with("yi")
    {
        "rtl"
    } else {
        "ltr"
    };

    let doc = format!(
        r##"<!DOCTYPE html>
<html lang="{html_lang}" dir="{html_dir}" class="{html_theme_class}">
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
{schema_json_ld}
	</script>

	<script>
	(function(){{
		var gate = {gate_analytics};
		function canTrack() {{
			if (!gate) return true;
			try {{
				var raw = localStorage.getItem('daypress_cookie_consent');
				if (!raw) return false;
				var c = JSON.parse(raw);
				return !!(c && c.analytics);
			}} catch(e) {{ return false; }}
		}}
		function sendEvent(eventName, goalName) {{
			if (!canTrack() || !window.fetch) return;
			try {{
				var sp = new URLSearchParams(window.location.search || '');
				fetch('/wp-json/daypress/v1/analytics/event', {{
					method: 'POST',
					headers: {{ 'Content-Type': 'application/json' }},
					body: JSON.stringify({{
						page_url: window.location.pathname,
						referrer: document.referrer || '',
						device_type: window.innerWidth < 768 ? 'mobile' : (window.innerWidth < 1024 ? 'tablet' : 'desktop'),
						utm_source: sp.get('utm_source') || '',
						utm_medium: sp.get('utm_medium') || '',
						utm_campaign: sp.get('utm_campaign') || '',
						event_name: eventName || 'pageview',
						goal_name: goalName || ''
					}}),
					keepalive: true
				}}).catch(function(){{}});
			}} catch(e) {{}}
		}}
		window.daypressTrack = function(goalName) {{
			if (goalName) sendEvent('goal', String(goalName));
		}};
		var pvSent = false;
		function trackPageviewOnce() {{
			if (pvSent || !canTrack()) return;
			pvSent = true;
			sendEvent('pageview', '');
		}}
		trackPageviewOnce();
		window.addEventListener('daypress:consent', function() {{ trackPageviewOnce(); }});
		document.addEventListener('click', function(e) {{
			var el = e.target && e.target.closest ? e.target.closest('[data-dp-goal]') : null;
			if (el) {{
				var g = el.getAttribute('data-dp-goal');
				if (g) window.daypressTrack(g);
			}}
		}});
		if (window.PerformanceObserver) {{
			var vitals = {{ lcp_ms: null, inp_ms: null, cls_score: 0, ttfb_ms: null }};
			var vitalsSent = false;
			try {{
				var navEntry = performance.getEntriesByType && performance.getEntriesByType('navigation')[0];
				if (navEntry && navEntry.responseStart > 0) vitals.ttfb_ms = Math.round(navEntry.responseStart);
				new PerformanceObserver(function(list) {{
					var entries = list.getEntries();
					var last = entries[entries.length - 1];
					if (last) vitals.lcp_ms = Math.round(last.startTime);
				}}).observe({{ type: 'largest-contentful-paint', buffered: true }});
				new PerformanceObserver(function(list) {{
					list.getEntries().forEach(function(entry) {{
						if (!entry.hadRecentInput) vitals.cls_score = Math.round((vitals.cls_score + entry.value) * 1000) / 1000;
					}});
				}}).observe({{ type: 'layout-shift', buffered: true }});
				new PerformanceObserver(function(list) {{
					list.getEntries().forEach(function(entry) {{
						if (entry.duration && (!vitals.inp_ms || entry.duration > vitals.inp_ms)) vitals.inp_ms = Math.round(entry.duration);
					}});
				}}).observe({{ type: 'event', buffered: true, durationThreshold: 16 }});
			}} catch(e) {{}}
			function flushVitals() {{
				if (vitalsSent || !canTrack() || !window.fetch) return;
				if (vitals.lcp_ms === null && vitals.ttfb_ms === null) return;
				vitalsSent = true;
				fetch('/wp-json/daypress/v1/analytics/vitals', {{
					method: 'POST',
					headers: {{ 'Content-Type': 'application/json' }},
					body: JSON.stringify({{
						page_url: window.location.pathname,
						device_type: window.innerWidth < 768 ? 'mobile' : (window.innerWidth < 1024 ? 'tablet' : 'desktop'),
						lcp_ms: vitals.lcp_ms,
						inp_ms: vitals.inp_ms,
						cls_score: vitals.cls_score,
						ttfb_ms: vitals.ttfb_ms
					}}),
					keepalive: true
				}}).catch(function(){{}});
			}}
			document.addEventListener('visibilitychange', function() {{
				if (document.visibilityState === 'hidden') flushVitals();
			}});
		}}
	}})();
	</script>

	<style>
		{public_theme_css}
	</style>{daystudio_kit_css_block}{custom_css_block}{theme_header_css}{theme_footer_css}{theme_popups_css}{theme_head_assets}
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

	{daystudio_popups_html}
	{theme_popups_html}
	<div id="dp-lightbox-modal" class="dp-lightbox-modal" role="dialog" aria-modal="true">
		<div class="dp-lightbox-backdrop" onclick="this.parentElement.classList.remove('active')"></div>
		<button type="button" class="dp-lightbox-close" aria-label="Close" onclick="document.getElementById('dp-lightbox-modal').classList.remove('active')">&times;</button>
		<div class="dp-lightbox-content">
			<img id="dp-lightbox-img" src="" alt="" />
		</div>
	</div>
	{cookie_consent_html}
	{cmdk_search_html}
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
        schema_json_ld = schema_json_ld,
        gate_analytics = gate_analytics,
        public_theme_css = super::styles::render_public_theme_css(&escape_html(brand_color)),
        daystudio_kit_css_block = daystudio_kit_css_block,
        custom_css_block = custom_css_block,
        theme_header_css = theme_header_css,
        theme_footer_css = theme_footer_css,
        theme_popups_css = theme_popups_css,
        theme_head_assets = theme_head_assets,
        site_header_html = site_header_html,
        site_footer_html = site_footer_html,
        daystudio_popups_html = settings.daystudio_popups_html,
        theme_popups_html = theme_popups_html,
        content = content,
        full_body_class = full_body_class,
        admin_bar_html = admin_bar_html,
        cookie_consent_html = cookie_consent_html,
        cmdk_search_html = cmdk_search_html,
        html_lang = escape_html(&html_lang),
        html_dir = html_dir
    );

    sanitize_ssr_html(&doc)
}
