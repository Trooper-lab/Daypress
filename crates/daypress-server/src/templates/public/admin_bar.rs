//! WordPress Admin Bar Component for Authenticated Users.

use crate::auth::AuthUser;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::utils::escape_html;

/// Renders the floating WordPress Admin Bar (`#wpadminbar`) matching the backend admin bar and returns `(html, body_class)`.
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

        let post_actions = if let Some(post) = current_post {
            let label = if post.post_type == "page" {
                "Page"
            } else {
                "Post"
            };
            format!(
                r##"
                <li>
                    <a class="ab-item" href="/d-admin/post?post={id}&action=edit">
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
                        <span>Edit {label}</span>
                    </a>
                </li>
                <li>
                    <a class="ab-item" href="/d-admin/builder?post={id}" style="color: #60a5fa !important; font-weight: 500 !important;">
                        <span>⚡ Edit with Visual Builder</span>
                    </a>
                </li>
                "##,
                id = post.id,
                label = label
            )
        } else {
            r##"
            <li>
                <a class="ab-item" href="/d-admin/customize">
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
                    <span>Customize</span>
                </a>
            </li>
            <li>
                <a class="ab-item" href="/d-admin/builder" style="color: #60a5fa !important; font-weight: 500 !important;">
                    <span>⚡ DayStudio</span>
                </a>
            </li>
            "##
            .to_string()
        };

        let bar = format!(
            r##"
            <style>
                #wpadminbar {{
                    position: fixed !important;
                    top: 0 !important;
                    left: 0 !important;
                    right: 0 !important;
                    width: 100% !important;
                    height: 32px !important;
                    background: #141923 !important;
                    color: #e2e8f0 !important;
                    z-index: 999999 !important;
                    font-family: -apple-system, BlinkMacSystemFont, "Inter", "Segoe UI", Roboto, sans-serif !important;
                    font-size: 13px !important;
                    line-height: 32px !important;
                    user-select: none !important;
                    box-shadow: none !important;
                }}
                body.admin-bar {{
                    margin-top: 32px !important;
                }}
                #wpadminbar .ab-top-menu-wrap {{
                    display: flex !important;
                    align-items: center !important;
                    justify-content: space-between !important;
                    height: 32px !important;
                    padding: 0 4px !important;
                }}
                #wpadminbar .ab-top-menu {{
                    display: flex !important;
                    align-items: center !important;
                    list-style: none !important;
                    height: 32px !important;
                    margin: 0 !important;
                    padding: 0 !important;
                }}
                #wpadminbar .ab-top-menu > li {{
                    position: relative !important;
                    height: 32px !important;
                    display: flex !important;
                    align-items: center !important;
                }}
                #wpadminbar .ab-item {{
                    display: inline-flex !important;
                    align-items: center !important;
                    gap: 6px !important;
                    height: 32px !important;
                    padding: 0 10px !important;
                    color: #e2e8f0 !important;
                    text-decoration: none !important;
                    font-size: 13px !important;
                    line-height: 32px !important;
                    transition: background 0.12s, color 0.12s !important;
                }}
                #wpadminbar .ab-top-menu > li:hover > .ab-item {{
                    background: #1e2636 !important;
                    color: #60a5fa !important;
                }}
                #wpadminbar .ab-avatar {{
                    width: 18px !important;
                    height: 18px !important;
                    border-radius: 3px !important;
                    background: #3858e9 !important;
                    color: #fff !important;
                    font-size: 10px !important;
                    font-weight: 700 !important;
                    display: inline-flex !important;
                    align-items: center !important;
                    justify-content: center !important;
                }}
                #wpadminbar .ab-sub-wrapper {{
                    display: none !important;
                    position: absolute !important;
                    top: 32px !important;
                    left: 0 !important;
                    min-width: 180px !important;
                    background: #1e2636 !important;
                    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.45) !important;
                    padding: 4px 0 !important;
                    z-index: 1000000 !important;
                }}
                #wpadminbar .ab-sub-wrapper.ab-sub-right {{
                    left: auto !important;
                    right: 0 !important;
                }}
                #wpadminbar .menupop:hover > .ab-sub-wrapper {{
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
                    display: block !important;
                    height: auto !important;
                    line-height: 1.4 !important;
                    padding: 6px 14px !important;
                    color: #cbd5e1 !important;
                    font-size: 12.5px !important;
                }}
                #wpadminbar .ab-submenu .ab-item:hover {{
                    background: #141923 !important;
                    color: #60a5fa !important;
                }}
                #daypress-welcome-dock {{
                    position: fixed;
                    bottom: 24px;
                    right: 24px;
                    z-index: 999998;
                    max-width: 410px;
                    background: #16171a;
                    color: #f3eee4;
                    border: 1px solid #3c3a34;
                    border-radius: 10px;
                    padding: 18px 20px;
                    box-shadow: 0 20px 48px rgba(0,0,0,0.28);
                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
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
                    border-radius: 6px;
                    font-size: 13px;
                    font-weight: 600;
                }}
                #daypress-welcome-dock .dock-btn-secondary {{
                    display: inline-flex;
                    align-items: center;
                    background: rgba(255,255,255,0.08);
                    color: #f3eee4 !important;
                    text-decoration: none !important;
                    padding: 8px 12px;
                    border-radius: 6px;
                    font-size: 12.5px;
                    font-weight: 500;
                }}
            </style>
            <div id="wpadminbar" class="nojq">
                <div class="ab-top-menu-wrap" id="wp-toolbar" role="navigation" aria-label="Toolbar">
                    <ul id="wp-admin-bar-root-default" class="ab-top-menu">
                        <li id="wp-admin-bar-wp-logo" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/d-admin/" title="About Daypress" style="padding: 0 12px !important;">
                                <img src="/static/daypress-icon.svg" style="width: 18px; height: 18px; border-radius: 4px;" alt="Daypress" />
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/getting-started">✨ Get Started &amp; Connect AI</a></li>
                                    <li><a class="ab-item" href="/d-admin/">About Daypress</a></li>
                                    <li><a class="ab-item" href="/d-admin/site-health">Site Health</a></li>
                                    <li><a class="ab-item" href="/d-admin/daypress-engine">Engine Telemetry</a></li>
                                </ul>
                            </div>
                        </li>
                        <li id="wp-admin-bar-site-name" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/d-admin/">
                                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
                                <span style="font-weight: 500;">{site_name}</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/">Dashboard</a></li>
                                    <li><a class="ab-item" href="/d-admin/getting-started">✨ Get Started &amp; AI</a></li>
                                    <li><a class="ab-item" href="/d-admin/nav-menus">Menus</a></li>
                                    <li><a class="ab-item" href="/d-admin/customize">Customize</a></li>
                                </ul>
                            </div>
                        </li>
                        <li>
                            <a class="ab-item" href="/d-admin/getting-started" style="color: #fbbf24 !important; font-weight: 600 !important;">
                                <span>✨ Connect AI Agent</span>
                            </a>
                        </li>
                        <li>
                            <a class="ab-item" href="/d-admin/publish" style="color: #34d399 !important; font-weight: 600 !important;" title="Publish to Cloudflare Pages, Netlify, or .ZIP">
                                <span>&#128640; Publish</span>
                            </a>
                        </li>
                        <li>
                            <a class="ab-item" href="#" onclick="event.preventDefault();fetch('/wp-json/daypress/v1/cache/purge',{{method:'POST',headers:{{'Content-Type':'application/json'}},body:'{{}}'}}).then(()=>{{this.innerHTML='<span>&#10003; Cache Purged</span>';setTimeout(()=>location.reload(),350);}});" title="Purge In-Memory SSR Hot Cache &amp; Edge CDN">
                                <span>&#9889; Purge Cache</span>
                            </a>
                        </li>
                        <li>
                            <a class="ab-item" href="/d-admin/update-core" title="Updates">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><polyline points="23 4 23 10 17 10"></polyline><polyline points="1 20 1 14 7 14"></polyline><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
                                <span>0</span>
                            </a>
                        </li>
                        <li>
                            <a class="ab-item" href="/d-admin/edit" title="Quick Search (Ctrl+K)">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
                                <span>Ctrl+K</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-comments">
                            <a class="ab-item" href="/d-admin/edit-comments" title="Comments">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
                                <span>{pending_comments}</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-new-content" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/d-admin/post-new">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                                <span>New</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/post-new">Post</a></li>
                                    <li><a class="ab-item" href="/d-admin/media-new">Media</a></li>
                                    <li><a class="ab-item" href="/d-admin/post-new?post_type=page">Page</a></li>
                                    <li><a class="ab-item" href="/d-admin/user-new">User</a></li>
                                </ul>
                            </div>
                        </li>
                        {post_actions}
                    </ul>
                    <ul id="wp-admin-bar-top-secondary" class="ab-top-secondary ab-top-menu">
                        <li id="wp-admin-bar-my-account" class="menupop with-avatar">
                            <a class="ab-item" aria-haspopup="true" href="/d-admin/profile">
                                <span>Howdy, <span class="display-name">{username}</span></span>
                                <span class="ab-avatar">{avatar_initial}</span>
                            </a>
                            <div class="ab-sub-wrapper ab-sub-right">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/d-admin/profile">Edit Profile</a></li>
                                    <li><a class="ab-item" href="/login?action=logout" style="color: #f87171 !important;">Log Out</a></li>
                                </ul>
                            </div>
                        </li>
                    </ul>
                </div>
            </div>
            <div id="daypress-welcome-dock" role="region" aria-label="Welcome to Daypress">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px;">
                    <span style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;color:#f59e0b;">✨ Your new homepage is live</span>
                    <button type="button" onclick="document.getElementById('daypress-welcome-dock').style.display='none'" style="background:none;border:none;color:#9ca3af;cursor:pointer;font-size:15px;">&times;</button>
                </div>
                <div style="font-size:14px;line-height:1.45;color:#e5e7eb;margin-bottom:14px;">
                    Welcome to <strong>{site_name}</strong>! Open your Admin Launchpad to connect your AI agent (Claude Code, Codex, or Cursor) in one click.
                </div>
                <div style="display:flex;gap:8px;flex-wrap:wrap;">
                    <a href="/d-admin/getting-started" class="dock-btn-primary">Open Admin &amp; Connect AI &rarr;</a>
                    <a href="/d-admin/publish" class="dock-btn-secondary">Publish Site</a>
                    <a href="/d-admin/customize" class="dock-btn-secondary">Customize Theme</a>
                </div>
            </div>
            <script>
                if (window.location.search.includes('welcome=1')) {{
                    const dock = document.getElementById('daypress-welcome-dock');
                    if (dock) dock.style.display = 'block';
                }}
            </script>
            "##,
            site_name = escape_html(site_name),
            pending_comments = settings.pending_comments_count,
            post_actions = post_actions,
            username = username,
            avatar_initial = avatar_initial
        );
        (bar, " admin-bar")
    } else {
        (String::new(), "")
    }
}
