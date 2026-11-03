//! Daypress Live Site Top Bar (`#wpadminbar`) for Authenticated Users.
//!
//! Styled in the Daypress Night Desk palette (`--desk`, `--desk-raised`, `--desk-ink`,
//! `--desk-muted`, `--desk-rule`, `--desk-oxide`) with Schibsted Grotesk, Newsreader,
//! and IBM Plex Mono typography and crisp vector iconography (zero emojis).

use crate::auth::AuthUser;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::format_cpt_title;
use crate::templates::utils::escape_html;

/// Renders the floating Daypress Top Bar (`#wpadminbar`) matching the Daypress Desk brand guide
/// and returns `(html, body_class)`.
pub fn render_admin_bar(
    user: Option<&AuthUser>,
    current_post: Option<&DaypressPostResponse>,
    settings: &DaypressSettingsResponse,
    site_name: &str,
) -> (String, &'static str) {
    if let Some(user) = user.filter(|u| crate::auth::should_show_admin_bar_front(u.user_id)) {
        let username = escape_html(&user.username);
        let avatar_initial = username
            .chars()
            .next()
            .unwrap_or('A')
            .to_uppercase()
            .collect::<String>();

        let avatar_html = if let Some(url) = crate::auth::cached_user_avatar_url(user.user_id) {
            format!(
                r#"<span class="ab-avatar"><img src="{}" alt="{}" class="ab-avatar-img" /></span>"#,
                escape_html(&url),
                username
            )
        } else {
            format!(r#"<span class="ab-avatar">{}</span>"#, avatar_initial)
        };

        let post_actions = if let Some(post) = current_post {
            let label = match post.post_type.as_str() {
                "page" => "Page".to_string(),
                "post" => "Post".to_string(),
                other => escape_html(&format_cpt_title(other)),
            };
            format!(
                r##"
                <li id="wp-admin-bar-edit">
                    <a class="ab-item" href="/d-admin/post?post={id}&amp;action=edit" title="Open in Editorial Desk Editor">
                        <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
                        <span>Edit {label}</span>
                    </a>
                </li>
                <li id="wp-admin-bar-builder">
                    <a class="ab-item ab-item-studio" href="/d-admin/builder?post={id}" title="Open in DayStudio Visual Builder">
                        <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
                        <span>Edit with Visual Builder</span>
                    </a>
                </li>
                "##,
                id = post.id,
                label = label
            )
        } else {
            let builder_href = if settings.page_on_front > 0 {
                format!("/d-admin/builder?post={}", settings.page_on_front)
            } else {
                "/d-admin/builder".to_string()
            };
            format!(
                r##"
                <li id="wp-admin-bar-customize">
                    <a class="ab-item" href="/d-admin/customize" title="Customize Theme &amp; Brand Tokens">
                        <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
                        <span>Customize</span>
                    </a>
                </li>
                <li id="wp-admin-bar-builder">
                    <a class="ab-item ab-item-studio" href="{builder_href}" title="Open DayStudio Visual Builder">
                        <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
                        <span>DayStudio</span>
                    </a>
                </li>
                "##,
                builder_href = builder_href
            )
        };

        let cpt_new_items: String = settings
            .custom_post_types
            .iter()
            .map(|slug| {
                format!(
                    r#"<li><a class="ab-item" href="/d-admin/post-new?post_type={}">{}</a></li>"#,
                    escape_html(slug),
                    escape_html(&format_cpt_title(slug))
                )
            })
            .collect();

        let pending_comments = settings.pending_comments_count;
        let comments_badge_class = if pending_comments > 0 {
            "ab-count is-alert"
        } else {
            "ab-count"
        };

        let (updates_count, updates_badge_class, updates_title) =
            if let Some(ref ver) = settings.update_available {
                (
                    1,
                    "ab-count is-alert",
                    format!("Daypress v{} update available", escape_html(ver)),
                )
            } else {
                (
                    0,
                    "ab-count",
                    format!("Daypress v{} is up to date", env!("CARGO_PKG_VERSION")),
                )
            };

        let users_submenu_item = if user.is_admin() {
            r#"<li><a class="ab-item" href="/d-admin/users">Manage Users</a></li>"#
        } else {
            ""
        };

        let bar = format!(
            r##"
            <style>
                #wpadminbar {{
                    --ab-bg: #1a1c1f;
                    --ab-raised: #26282c;
                    --ab-ink: #efe9de;
                    --ab-muted: #a7a297;
                    --ab-rule: rgba(239, 233, 222, 0.16);
                    --ab-oxide: #e8744a;
                    --ab-oxide-solid: #a8401b;
                    --ab-on-oxide: #fff8f0;
                    --ab-press: #93a1ff;
                    --ab-moss: #93b98a;
                    --ab-ochre: #e6b84a;
                    --ab-font-sans: "Schibsted Grotesk", "Helvetica Neue", Arial, sans-serif;
                    --ab-font-serif: "Newsreader", "Iowan Old Style", Georgia, serif;
                    --ab-font-mono: "IBM Plex Mono", ui-monospace, Menlo, monospace;

                    position: fixed !important;
                    top: 0 !important;
                    left: 0 !important;
                    right: 0 !important;
                    width: 100% !important;
                    height: 36px !important;
                    background: var(--ab-bg) !important;
                    color: var(--ab-ink) !important;
                    border-bottom: 1px solid var(--ab-rule) !important;
                    z-index: 999999 !important;
                    font-family: var(--ab-font-sans) !important;
                    font-size: 13px !important;
                    line-height: 36px !important;
                    user-select: none !important;
                    box-shadow: none !important;
                    -webkit-font-smoothing: antialiased !important;
                }}
                body.admin-bar {{
                    margin-top: 36px !important;
                }}
                body.admin-bar header.day-header,
                body.admin-bar header.dp-theme-header.dp-sticky {{
                    top: 36px !important;
                }}
                #wpadminbar .ab-top-menu-wrap {{
                    display: flex !important;
                    align-items: center !important;
                    justify-content: space-between !important;
                    height: 35px !important;
                    padding: 0 8px !important;
                }}
                #wpadminbar .ab-top-menu {{
                    display: flex !important;
                    align-items: center !important;
                    list-style: none !important;
                    height: 35px !important;
                    margin: 0 !important;
                    padding: 0 !important;
                    gap: 2px !important;
                }}
                #wpadminbar .ab-top-menu > li {{
                    position: relative !important;
                    height: 35px !important;
                    display: flex !important;
                    align-items: center !important;
                }}
                #wpadminbar .ab-divider {{
                    width: 1px !important;
                    height: 16px !important;
                    background: var(--ab-rule) !important;
                    margin: 0 4px !important;
                    pointer-events: none !important;
                }}
                #wpadminbar .ab-item {{
                    display: inline-flex !important;
                    align-items: center !important;
                    gap: 6px !important;
                    height: 28px !important;
                    padding: 0 9px !important;
                    border-radius: 3px !important;
                    color: var(--ab-ink) !important;
                    text-decoration: none !important;
                    font-family: var(--ab-font-sans) !important;
                    font-size: 12.5px !important;
                    font-weight: 500 !important;
                    line-height: 28px !important;
                    transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease !important;
                    white-space: nowrap !important;
                }}
                #wpadminbar .ab-icon {{
                    width: 14px !important;
                    height: 14px !important;
                    flex-shrink: 0 !important;
                    color: var(--ab-muted) !important;
                    transition: color 0.12s ease !important;
                }}
                #wpadminbar .ab-top-menu > li:hover > .ab-item,
                #wpadminbar .ab-top-menu > li > .ab-item:focus-visible {{
                    background: var(--ab-raised) !important;
                    color: var(--ab-ink) !important;
                }}
                #wpadminbar .ab-top-menu > li:hover > .ab-item .ab-icon {{
                    color: var(--ab-oxide) !important;
                }}
                #wpadminbar .ab-brand-item {{
                    padding: 0 10px !important;
                    gap: 8px !important;
                }}
                #wpadminbar .ab-day-dot {{
                    width: 8px !important;
                    height: 8px !important;
                    border-radius: 999px !important;
                    background: var(--ab-oxide) !important;
                    flex-shrink: 0 !important;
                }}
                #wpadminbar .ab-wordmark {{
                    font-family: var(--ab-font-serif) !important;
                    font-size: 16.5px !important;
                    font-weight: 450 !important;
                    letter-spacing: -0.02em !important;
                    color: var(--ab-ink) !important;
                    line-height: 1 !important;
                }}
                #wpadminbar .ab-item-studio {{
                    background: rgba(232, 116, 74, 0.12) !important;
                    border: 1px solid rgba(232, 116, 74, 0.35) !important;
                    color: var(--ab-ink) !important;
                    font-weight: 600 !important;
                    height: 26px !important;
                    line-height: 24px !important;
                    margin-left: 2px !important;
                }}
                #wpadminbar .ab-item-studio .ab-icon {{
                    color: var(--ab-oxide) !important;
                }}
                #wpadminbar .ab-top-menu > li:hover > .ab-item-studio {{
                    background: var(--ab-oxide-solid) !important;
                    border-color: var(--ab-oxide) !important;
                    color: var(--ab-on-oxide) !important;
                }}
                #wpadminbar .ab-top-menu > li:hover > .ab-item-studio .ab-icon {{
                    color: var(--ab-on-oxide) !important;
                }}
                #wpadminbar .ab-count {{
                    font-family: var(--ab-font-mono) !important;
                    font-size: 11px !important;
                    font-weight: 500 !important;
                    font-variant-numeric: tabular-nums !important;
                    color: var(--ab-muted) !important;
                    line-height: 1 !important;
                }}
                #wpadminbar .ab-count.is-alert {{
                    background: rgba(230, 184, 74, 0.18) !important;
                    color: var(--ab-ochre) !important;
                    padding: 2px 5px !important;
                    border-radius: 3px !important;
                }}
                #wpadminbar .ab-kbd {{
                    font-family: var(--ab-font-mono) !important;
                    font-size: 10.5px !important;
                    font-weight: 500 !important;
                    color: var(--ab-muted) !important;
                    border: 1px solid var(--ab-rule) !important;
                    border-radius: 2px !important;
                    padding: 1px 5px !important;
                    line-height: 1.2 !important;
                }}
                #wpadminbar .ab-avatar {{
                    width: 20px !important;
                    height: 20px !important;
                    border-radius: 3px !important;
                    background: var(--ab-oxide-solid) !important;
                    color: var(--ab-on-oxide) !important;
                    font-family: var(--ab-font-mono) !important;
                    font-size: 11px !important;
                    font-weight: 600 !important;
                    display: inline-flex !important;
                    align-items: center !important;
                    justify-content: center !important;
                    overflow: hidden !important;
                    flex-shrink: 0 !important;
                }}
                #wpadminbar .ab-avatar-img {{
                    width: 100% !important;
                    height: 100% !important;
                    object-fit: cover !important;
                    display: block !important;
                }}
                #wpadminbar .ab-sub-wrapper {{
                    display: none !important;
                    position: absolute !important;
                    top: 35px !important;
                    left: 0 !important;
                    min-width: 196px !important;
                    background: var(--ab-bg) !important;
                    border: 1px solid var(--ab-rule) !important;
                    border-top: 2px solid var(--ab-oxide) !important;
                    border-radius: 0 0 4px 4px !important;
                    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.42) !important;
                    padding: 5px 0 !important;
                    z-index: 1000000 !important;
                }}
                #wpadminbar .ab-sub-wrapper.ab-sub-right {{
                    left: auto !important;
                    right: 0 !important;
                }}
                #wpadminbar .menupop:hover > .ab-sub-wrapper,
                #wpadminbar .menupop:focus-within > .ab-sub-wrapper {{
                    display: block !important;
                }}
                #wpadminbar .ab-submenu {{
                    list-style: none !important;
                    margin: 0 !important;
                    padding: 0 !important;
                    display: flex !important;
                    flex-direction: column !important;
                }}
                #wpadminbar .ab-submenu .ab-item {{
                    display: flex !important;
                    justify-content: space-between !important;
                    height: auto !important;
                    line-height: 1.4 !important;
                    padding: 6px 14px !important;
                    border-radius: 0 !important;
                    color: var(--ab-muted) !important;
                    font-size: 12.5px !important;
                    font-weight: 400 !important;
                }}
                #wpadminbar .ab-submenu .ab-item:hover {{
                    background: var(--ab-raised) !important;
                    color: var(--ab-ink) !important;
                }}
                #wpadminbar .ab-submenu .ab-item-logout {{
                    color: var(--ab-oxide) !important;
                    border-top: 1px solid var(--ab-rule) !important;
                    margin-top: 4px !important;
                    padding-top: 8px !important;
                }}
                #wpadminbar .ab-submenu .ab-item-logout:hover {{
                    background: var(--ab-raised) !important;
                    color: var(--ab-on-oxide) !important;
                }}
                #day-ab-qs-backdrop {{
                    display: none;
                    position: fixed;
                    inset: 0;
                    background: rgba(22, 23, 26, 0.55);
                    z-index: 1000001;
                    align-items: flex-start;
                    justify-content: center;
                    padding-top: 12vh;
                    font-family: "Schibsted Grotesk", "Helvetica Neue", Arial, sans-serif;
                }}
                .day-ab-qs-dialog {{
                    width: min(620px, 92vw);
                    background: #fbf8f2;
                    color: #16171a;
                    border: 1px solid #16171a;
                    border-radius: 6px;
                    overflow: hidden;
                    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.32);
                }}
                #day-ab-qs-input {{
                    width: 100%;
                    border: 0;
                    border-bottom: 1px solid #16171a;
                    padding: 15px 18px;
                    font: 400 18px/1.4 "Newsreader", "Iowan Old Style", Georgia, serif;
                    background: transparent;
                    color: #16171a;
                    outline: none;
                }}
                #day-ab-qs-results {{
                    max-height: 50vh;
                    overflow-y: auto;
                    padding: 6px 0;
                }}
                .day-ab-qs-item {{
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    gap: 12px;
                    padding: 9px 18px;
                    text-decoration: none;
                    color: #16171a;
                    font-size: 14px;
                }}
                .day-ab-qs-item.is-active,
                .day-ab-qs-item:hover {{
                    background: rgba(42, 59, 208, 0.12);
                }}
                .day-ab-qs-kind,
                .day-ab-qs-hint,
                .day-ab-qs-empty {{
                    font: 500 11.5px/16px "IBM Plex Mono", ui-monospace, Menlo, monospace;
                    color: #5a5852;
                }}
                .day-ab-qs-empty {{
                    padding: 14px 18px;
                }}
                .day-ab-qs-hint {{
                    padding: 8px 18px;
                    border-top: 1px solid rgba(22, 23, 26, 0.18);
                    background: #f3eee4;
                }}
                #daypress-welcome-dock {{
                    position: fixed;
                    bottom: 24px;
                    right: 24px;
                    z-index: 999998;
                    max-width: 410px;
                    background: #1a1c1f;
                    color: #efe9de;
                    border: 1px solid rgba(239, 233, 222, 0.18);
                    border-top: 2px solid #e8744a;
                    border-radius: 6px;
                    padding: 18px 20px;
                    box-shadow: 0 20px 48px rgba(0,0,0,0.32);
                    font-family: "Schibsted Grotesk", "Helvetica Neue", Arial, sans-serif;
                    display: none;
                }}
                #daypress-welcome-dock .dock-btn-primary {{
                    display: inline-flex;
                    align-items: center;
                    gap: 6px;
                    background: #a8401b;
                    color: #fff8f0 !important;
                    text-decoration: none !important;
                    padding: 8px 14px;
                    border-radius: 4px;
                    font-size: 13px;
                    font-weight: 600;
                }}
                #daypress-welcome-dock .dock-btn-secondary {{
                    display: inline-flex;
                    align-items: center;
                    background: #26282c;
                    color: #efe9de !important;
                    border: 1px solid rgba(239, 233, 222, 0.16);
                    text-decoration: none !important;
                    padding: 8px 12px;
                    border-radius: 4px;
                    font-size: 12.5px;
                    font-weight: 500;
                }}
                @media (max-width: 780px) {{
                    #wpadminbar .ab-hide-mobile {{
                        display: none !important;
                    }}
                }}
            </style>
            <div id="wpadminbar" class="nojq">
                <div class="ab-top-menu-wrap" id="wp-toolbar" role="navigation" aria-label="Daypress Admin Toolbar">
                    <ul id="wp-admin-bar-root-default" class="ab-top-menu">
                        <li id="wp-admin-bar-wp-logo" class="menupop">
                            <a class="ab-item ab-brand-item" aria-haspopup="true" href="/d-admin/" title="Daypress Desk">
                                <span class="ab-day-dot" aria-hidden="true"></span>
                                <span class="ab-wordmark">daypress</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/">Admin Desk</a></li>
                                    <li><a class="ab-item" href="/d-admin/analytics">Analytics</a></li>
                                    <li><a class="ab-item" href="/d-admin/site-health">Site Health</a></li>
                                    <li><a class="ab-item" href="/d-admin/daypress-engine">Engine Telemetry</a></li>
                                </ul>
                            </div>
                        </li>
                        <li class="ab-divider" role="separator" aria-hidden="true"></li>
                        <li id="wp-admin-bar-site-name" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/d-admin/">
                                <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
                                <span>{site_name}</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/">Dashboard</a></li>
                                    <li><a class="ab-item" href="/d-admin/edit?post_type=page">Pages</a></li>
                                    <li><a class="ab-item" href="/d-admin/edit">Posts</a></li>
                                    <li><a class="ab-item" href="/d-admin/upload">Media Library</a></li>
                                    <li><a class="ab-item" href="/d-admin/headers-footers">Headers &amp; Footers</a></li>
                                    <li><a class="ab-item" href="/d-admin/theme-layouts">Theme Layouts</a></li>
                                    <li><a class="ab-item" href="/d-admin/nav-menus">Menus</a></li>
                                    <li><a class="ab-item" href="/d-admin/customize">Customize</a></li>
                                </ul>
                            </div>
                        </li>
                        <li id="wp-admin-bar-new-content" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/d-admin/post-new" title="Create New Content">
                                <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                                <span class="ab-hide-mobile">New</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/post-new">Post</a></li>
                                    <li><a class="ab-item" href="/d-admin/post-new?post_type=page">Page</a></li>
                                    {cpt_new_items}
                                    <li><a class="ab-item" href="/d-admin/media-new">Media</a></li>
                                    <li><a class="ab-item" href="/d-admin/forms">Form</a></li>
                                    <li><a class="ab-item" href="/d-admin/user-new">User</a></li>
                                </ul>
                            </div>
                        </li>
                        {post_actions}
                    </ul>
                    <ul id="wp-admin-bar-top-secondary" class="ab-top-secondary ab-top-menu">
                        <li id="wp-admin-bar-search">
                            <a class="ab-item" href="/d-admin/edit" onclick="event.preventDefault();window.dayAbOpenQuickSearch();" title="Quick Search Posts, Pages &amp; Desk Screens (Ctrl+K)">
                                <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
                                <span class="ab-hide-mobile">Search</span>
                                <kbd class="ab-kbd ab-hide-mobile">Ctrl K</kbd>
                            </a>
                        </li>
                        <li id="wp-admin-bar-purge-cache">
                            <a class="ab-item" href="#purge-cache" onclick="event.preventDefault();window.dayAbPurgeCache(this);" title="Purge In-Memory SSR Hot Cache &amp; Edge CDN">
                                <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5z"></path><path d="M2 17l10 5 10-5"></path><path d="M2 12l10 5 10-5"></path></svg>
                                <span class="ab-hide-mobile">Purge Cache</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-comments">
                            <a class="ab-item" href="/d-admin/edit-comments" title="{pending_comments} pending comments">
                                <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
                                <span class="{comments_badge_class}">{pending_comments}</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-updates">
                            <a class="ab-item" href="/d-admin/update-core" title="{updates_title}">
                                <svg class="ab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"></polyline><polyline points="1 20 1 14 7 14"></polyline><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
                                <span class="{updates_badge_class}">{updates_count}</span>
                            </a>
                        </li>
                        <li class="ab-divider" role="separator" aria-hidden="true"></li>
                        <li id="wp-admin-bar-my-account" class="menupop with-avatar">
                            <a class="ab-item" aria-haspopup="true" href="/d-admin/profile">
                                <span>Howdy, <span class="display-name">{username}</span></span>
                                {avatar_html}
                            </a>
                            <div class="ab-sub-wrapper ab-sub-right">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/profile">Edit Profile</a></li>
                                    {users_submenu_item}
                                    <li><a class="ab-item ab-item-logout" href="/login?action=logout">Log Out</a></li>
                                </ul>
                            </div>
                        </li>
                    </ul>
                </div>
            </div>
            <div id="day-ab-qs-backdrop" onclick="if (event.target === this) window.dayAbCloseQuickSearch();">
                <div class="day-ab-qs-dialog" role="dialog" aria-modal="true" aria-label="Quick search">
                    <input id="day-ab-qs-input" type="search" autocomplete="off" placeholder="Search posts, pages and desk screens…" />
                    <div id="day-ab-qs-results"></div>
                    <div class="day-ab-qs-hint">Up/Down to move &middot; Enter to open &middot; Esc to close</div>
                </div>
            </div>
            <div id="daypress-welcome-dock" role="region" aria-label="Welcome to Daypress">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px;">
                    <span style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.12em;color:#e8744a;">Your new homepage is live</span>
                    <button type="button" onclick="document.getElementById('daypress-welcome-dock').style.display='none'" style="background:none;border:none;color:#a7a297;cursor:pointer;font-size:15px;">&times;</button>
                </div>
                <div style="font-size:14px;line-height:1.45;color:#efe9de;margin-bottom:14px;">
                    Welcome to <strong>{site_name}</strong>. Open the Admin Desk to manage content, headers, and theme templates.
                </div>
                <div style="display:flex;gap:8px;flex-wrap:wrap;">
                    <a href="/d-admin/" class="dock-btn-primary">Open Admin Desk &rarr;</a>
                    <a href="/d-admin/customize" class="dock-btn-secondary">Customize Theme</a>
                </div>
            </div>
            <script>
            (function() {{
                if (window.location.search.includes('welcome=1')) {{
                    var dock = document.getElementById('daypress-welcome-dock');
                    if (dock) dock.style.display = 'block';
                }}

                window.dayAbPurgeCache = async function(btn) {{
                    if (!btn || btn.dataset.busy === '1') return;
                    btn.dataset.busy = '1';
                    var orig = btn.innerHTML;
                    try {{
                        var res = await fetch('/wp-json/daypress/v1/cache/purge', {{
                            method: 'POST',
                            credentials: 'same-origin',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: '{{}}'
                        }});
                        if (res.ok) {{
                            btn.innerHTML = '<svg class="ab-icon" style="color:#93b98a !important;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4"><polyline points="20 6 9 17 4 12"></polyline></svg><span style="color:#93b98a;">Cache Purged</span>';
                            setTimeout(function() {{ window.location.reload(); }}, 320);
                            return;
                        }}
                    }} catch (e) {{}}
                    btn.dataset.busy = '0';
                    btn.innerHTML = orig;
                }};

                var QS_SCREENS = [
                    ['Dashboard', '/d-admin/'],
                    ['DayStudio Visual Builder', '/d-admin/builder'],
                    ['All Pages', '/d-admin/edit?post_type=page'],
                    ['Add New Page', '/d-admin/post-new?post_type=page'],
                    ['All Posts', '/d-admin/edit'],
                    ['Add New Post', '/d-admin/post-new'],
                    ['Media Library', '/d-admin/upload'],
                    ['Headers & Footers', '/d-admin/headers-footers'],
                    ['Theme Layouts', '/d-admin/theme-layouts'],
                    ['Synced Symbols', '/d-admin/symbols'],
                    ['Popups & Bars', '/d-admin/popups'],
                    ['Menus', '/d-admin/nav-menus'],
                    ['Customize', '/d-admin/customize'],
                    ['Forms Studio', '/d-admin/forms'],
                    ['Analytics', '/d-admin/analytics'],
                    ['Comments', '/d-admin/edit-comments'],
                    ['Users', '/d-admin/users'],
                    ['Profile', '/d-admin/profile'],
                    ['SEO & Redirects', '/d-admin/seo'],
                    ['Site Health', '/d-admin/site-health'],
                    ['General Settings', '/d-admin/options-general'],
                    ['Engine Telemetry', '/d-admin/daypress-engine']
                ];
                var qsItems = [];
                var qsActive = 0;
                var qsTimer = null;
                var qsSeq = 0;

                function qsEsc(s) {{
                    return String(s == null ? '' : s).replace(/[&<>"']/g, function(c) {{
                        return {{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}}[c];
                    }});
                }}

                function qsRender(query, contentResults) {{
                    var q = query.trim().toLowerCase();
                    var screens = QS_SCREENS
                        .filter(function(pair) {{ return !q || pair[0].toLowerCase().includes(q); }})
                        .slice(0, q ? 6 : 8)
                        .map(function(pair) {{ return {{ label: pair[0], url: pair[1], kind: 'Desk Screen' }}; }});
                    qsItems = contentResults.concat(screens);
                    qsActive = 0;
                    var box = document.getElementById('day-ab-qs-results');
                    if (!box) return;
                    if (!qsItems.length) {{
                        box.innerHTML = '<div class="day-ab-qs-empty">No matching content or screens</div>';
                        return;
                    }}
                    box.innerHTML = qsItems.map(function(it, i) {{
                        return '<a href="' + qsEsc(it.url) + '" class="day-ab-qs-item' + (i === 0 ? ' is-active' : '') + '">' +
                            '<span>' + qsEsc(it.label) + '</span><span class="day-ab-qs-kind">' + qsEsc(it.kind) + '</span></a>';
                    }}).join('');
                }}

                function qsHighlight() {{
                    var els = document.querySelectorAll('#day-ab-qs-results .day-ab-qs-item');
                    els.forEach(function(el, idx) {{
                        var active = idx === qsActive;
                        el.classList.toggle('is-active', active);
                        if (active) el.scrollIntoView({{ block: 'nearest' }});
                    }});
                }}

                async function qsSearch(query) {{
                    var seq = ++qsSeq;
                    var q = query.trim();
                    if (q.length < 2) {{ qsRender(query, []); return; }}
                    var content = [];
                    try {{
                        var res = await fetch('/wp-json/wp/v2/search?per_page=6&search=' + encodeURIComponent(q), {{ credentials: 'same-origin' }});
                        if (res.ok) {{
                            var rows = await res.json();
                            content = (Array.isArray(rows) ? rows : []).map(function(r) {{
                                return {{
                                    label: String(r.title || '(no title)').replace(/<[^>]*>/g, ''),
                                    url: '/d-admin/post?post=' + encodeURIComponent(r.id) + '&action=edit',
                                    kind: 'Edit ' + (r.subtype || 'post')
                                }};
                            }});
                        }}
                    }} catch (e) {{}}
                    if (seq === qsSeq) qsRender(query, content);
                }}

                window.dayAbOpenQuickSearch = function() {{
                    var bd = document.getElementById('day-ab-qs-backdrop');
                    var input = document.getElementById('day-ab-qs-input');
                    if (!bd || !input) return;
                    bd.style.display = 'flex';
                    input.value = '';
                    qsRender('', []);
                    setTimeout(function() {{ input.focus(); }}, 0);
                }};

                window.dayAbCloseQuickSearch = function() {{
                    var bd = document.getElementById('day-ab-qs-backdrop');
                    if (bd) bd.style.display = 'none';
                }};

                document.addEventListener('DOMContentLoaded', function() {{
                    var input = document.getElementById('day-ab-qs-input');
                    if (!input) return;
                    input.addEventListener('input', function() {{
                        clearTimeout(qsTimer);
                        qsRender(input.value, []);
                        qsTimer = setTimeout(function() {{ qsSearch(input.value); }}, 160);
                    }});
                    input.addEventListener('keydown', function(e) {{
                        if (e.key === 'ArrowDown') {{ e.preventDefault(); qsActive = Math.min(qsActive + 1, Math.max(qsItems.length - 1, 0)); qsHighlight(); }}
                        else if (e.key === 'ArrowUp') {{ e.preventDefault(); qsActive = Math.max(qsActive - 1, 0); qsHighlight(); }}
                        else if (e.key === 'Enter') {{ e.preventDefault(); var it = qsItems[qsActive]; if (it) window.location.href = it.url; }}
                        else if (e.key === 'Escape') {{ window.dayAbCloseQuickSearch(); }}
                    }});
                }});

                document.addEventListener('keydown', function(e) {{
                    if (e.defaultPrevented) return;
                    var mod = e.ctrlKey || e.metaKey;
                    if (mod && (e.key === 'k' || e.key === 'K')) {{
                        e.preventDefault();
                        window.dayAbOpenQuickSearch();
                    }} else if (e.key === 'Escape') {{
                        window.dayAbCloseQuickSearch();
                    }}
                }});
            }})();
            </script>
            "##,
            site_name = escape_html(site_name),
            cpt_new_items = cpt_new_items,
            post_actions = post_actions,
            pending_comments = pending_comments,
            comments_badge_class = comments_badge_class,
            updates_count = updates_count,
            updates_badge_class = updates_badge_class,
            updates_title = updates_title,
            username = username,
            avatar_html = avatar_html,
            users_submenu_item = users_submenu_item
        );
        (bar, " admin-bar")
    } else {
        (String::new(), "")
    }
}
