//! WordPress Admin Bar Component for Authenticated Users.

use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

/// Renders the floating WordPress Admin Bar (`#wpadminbar`) matching the backend admin bar and returns `(html, body_class)`.
pub fn render_admin_bar(
    user: Option<&AuthUser>,
    current_post: Option<&WpPostResponse>,
    settings: &WpSettingsResponse,
    site_name: &str,
) -> (String, &'static str) {
    if let Some(user) = user {
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
                    <a class="ab-item" href="/n-admin/post?post={id}&action=edit">
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
                        <span>Edit {label}</span>
                    </a>
                </li>
                <li>
                    <a class="ab-item" href="/n-admin/builder?post={id}" style="color: #60a5fa !important; font-weight: 500 !important;">
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
                <a class="ab-item" href="/n-admin/customize">
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
                    <span>Customize</span>
                </a>
            </li>
            <li>
                <a class="ab-item" href="/n-admin/builder" style="color: #60a5fa !important; font-weight: 500 !important;">
                    <span>⚡ NeoStudio</span>
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
            </style>
            <div id="wpadminbar" class="nojq">
                <div class="ab-top-menu-wrap" id="wp-toolbar" role="navigation" aria-label="Toolbar">
                    <ul id="wp-admin-bar-root-default" class="ab-top-menu">
                        <li id="wp-admin-bar-wp-logo" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/" title="About Neopress" style="padding: 0 12px !important;">
                                <img src="/static/neopress-icon.svg" style="width: 18px; height: 18px; border-radius: 4px;" alt="Neopress" />
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/">About Neopress</a></li>
                                    <li><a class="ab-item" href="/n-admin/site-health">Site Health</a></li>
                                    <li><a class="ab-item" href="/n-admin/neopress-engine">Engine Telemetry</a></li>
                                </ul>
                            </div>
                        </li>
                        <li id="wp-admin-bar-site-name" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/">
                                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
                                <span style="font-weight: 500;">{site_name}</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/">Dashboard</a></li>
                                    <li><a class="ab-item" href="/n-admin/nav-menus">Menus</a></li>
                                    <li><a class="ab-item" href="/n-admin/customize">Customize</a></li>
                                </ul>
                            </div>
                        </li>
                        <li>
                            <a class="ab-item" href="/n-admin/update-core" title="Updates">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><polyline points="23 4 23 10 17 10"></polyline><polyline points="1 20 1 14 7 14"></polyline><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
                                <span>0</span>
                            </a>
                        </li>
                        <li>
                            <a class="ab-item" href="/n-admin/edit" title="Quick Search (Ctrl+K)">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
                                <span>Ctrl+K</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-comments">
                            <a class="ab-item" href="/n-admin/edit-comments" title="Comments">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
                                <span>{pending_comments}</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-new-content" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/post-new">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                                <span>New</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/post-new">Post</a></li>
                                    <li><a class="ab-item" href="/n-admin/media-new">Media</a></li>
                                    <li><a class="ab-item" href="/n-admin/post-new?post_type=page">Page</a></li>
                                    <li><a class="ab-item" href="/n-admin/user-new">User</a></li>
                                </ul>
                            </div>
                        </li>
                        {post_actions}
                    </ul>
                    <ul id="wp-admin-bar-top-secondary" class="ab-top-secondary ab-top-menu">
                        <li id="wp-admin-bar-my-account" class="menupop with-avatar">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/profile">
                                <span>Howdy, <span class="display-name">{username}</span></span>
                                <span class="ab-avatar">{avatar_initial}</span>
                            </a>
                            <div class="ab-sub-wrapper ab-sub-right">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/profile">Edit Profile</a></li>
                                    <li><a class="ab-item" href="/login?action=logout" style="color: #f87171 !important;">Log Out</a></li>
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
            username = username,
            avatar_initial = avatar_initial
        );
        (bar, " admin-bar")
    } else {
        (String::new(), "")
    }
}
