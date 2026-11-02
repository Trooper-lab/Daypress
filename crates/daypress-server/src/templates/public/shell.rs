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
use crate::templates::utils::{escape_html, escape_json};
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
.dp-cc-banner{{position:fixed;z-index:99998;background:#16181d;color:#f4f5f8;border:1px solid rgba(255,255,255,.12);box-shadow:0 16px 40px rgba(0,0,0,.35);font-family:system-ui,-apple-system,sans-serif;font-size:13.5px;line-height:1.5;display:none;box-sizing:border-box}}
.dp-cc-bottom-bar{{left:16px;right:16px;bottom:16px;max-width:980px;margin:0 auto;padding:16px 20px;border-radius:12px;align-items:center;justify-content:space-between;gap:18px;flex-wrap:wrap}}
.dp-cc-floating-right{{right:20px;bottom:20px;max-width:400px;padding:18px;border-radius:12px;flex-direction:column;gap:14px}}
.dp-cc-floating-left{{left:20px;bottom:20px;max-width:400px;padding:18px;border-radius:12px;flex-direction:column;gap:14px}}
.dp-cc-title{{font-weight:700;font-size:14.5px;margin:0 0 4px;color:#fff}}
.dp-cc-msg{{margin:0;color:#c9cdd8;font-size:13px}}
.dp-cc-actions{{display:flex;gap:8px;flex-wrap:wrap;align-items:center;flex-shrink:0}}
.dp-cc-btn{{border:1px solid rgba(255,255,255,.18);background:rgba(255,255,255,.07);color:#fff;padding:7px 13px;border-radius:7px;font-size:12.5px;font-weight:600;cursor:pointer}}
.dp-cc-btn-primary{{background:#2271b1;border-color:#2271b1;color:#fff}}
.dp-cc-modal{{position:fixed;inset:0;z-index:99999;background:rgba(0,0,0,.6);display:none;align-items:center;justify-content:center;padding:16px}}
.dp-cc-dialog{{background:#16181d;color:#f4f5f8;border:1px solid rgba(255,255,255,.14);border-radius:12px;max-width:460px;width:100%;padding:20px;box-sizing:border-box}}
.dp-cc-cat{{display:flex;justify-content:space-between;align-items:center;padding:10px 0;border-bottom:1px solid rgba(255,255,255,.08);font-size:13px}}
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

    let daystudio_kit_css_block = if !settings.daystudio_kit_css.trim().is_empty() {
        format!(
            "\n\t<style id=\"daystudio-kit-css\">\n{}\n\t</style>",
            settings.daystudio_kit_css
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

    let gate_analytics = if settings.privacy_settings.cookie_banner_enabled
        && settings.privacy_settings.gate_builtin_analytics
    {
        "true"
    } else {
        "false"
    };
    let cookie_consent_html = render_cookie_consent_banner(settings);

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
	}})();
	</script>

	<style>
		{public_theme_css}
	</style>{daystudio_kit_css_block}{custom_css_block}{theme_header_css}{theme_footer_css}{theme_head_assets}
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
	<div id="dp-lightbox-modal" class="dp-lightbox-modal" role="dialog" aria-modal="true">
		<div class="dp-lightbox-backdrop" onclick="this.parentElement.classList.remove('active')"></div>
		<button type="button" class="dp-lightbox-close" aria-label="Close" onclick="document.getElementById('dp-lightbox-modal').classList.remove('active')">&times;</button>
		<div class="dp-lightbox-content">
			<img id="dp-lightbox-img" src="" alt="" />
		</div>
	</div>
	{cookie_consent_html}
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
        gate_analytics = gate_analytics,
        public_theme_css = super::styles::render_public_theme_css(&escape_html(brand_color)),
        daystudio_kit_css_block = daystudio_kit_css_block,
        custom_css_block = custom_css_block,
        theme_header_css = theme_header_css,
        theme_footer_css = theme_footer_css,
        theme_head_assets = theme_head_assets,
        site_header_html = site_header_html,
        site_footer_html = site_footer_html,
        daystudio_popups_html = settings.daystudio_popups_html,
        content = content,
        full_body_class = full_body_class,
        admin_bar_html = admin_bar_html,
        cookie_consent_html = cookie_consent_html
    );

    sanitize_ssr_html(&doc)
}
