//! WordPress Admin Bar Component for Authenticated Users.

use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

/// Renders the floating WordPress Admin Bar and returns (html, body_class).
pub fn render_admin_bar(
    user: Option<&AuthUser>,
    current_post: Option<&WpPostResponse>,
    settings: &WpSettingsResponse,
    site_name: &str,
) -> (String, &'static str) {
    if let Some(user) = user {
        let username = escape_html(&user.username);
        let post_actions = if let Some(post) = current_post {
            let label = if post.post_type == "page" {
                "Page"
            } else {
                "Post"
            };
            format!(
                r##"
                <li id="wp-admin-bar-edit">
                    <a class="ab-item" href="/n-admin/post.php?post={id}&action=edit">
                        <span class="ab-icon"><i class="fa-solid fa-pencil"></i></span>
                        <span class="ab-label">Edit {label}</span>
                    </a>
                </li>
                <li id="wp-admin-bar-elementor-edit">
                    <a class="ab-item ab-btn-builder" href="/n-admin/builder?post={id}">
                        <span class="ab-icon"><i class="fa-solid fa-cubes"></i></span>
                        <span class="ab-label">Edit with Visual Builder</span>
                    </a>
                </li>
                "##,
                id = post.id,
                label = label
            )
        } else {
            r##"
            <li id="wp-admin-bar-customize">
                <a class="ab-item" href="/n-admin/customize.php">
                    <span class="ab-icon"><i class="fa-solid fa-paintbrush"></i></span>
                    <span class="ab-label">Customize</span>
                </a>
            </li>
            "##
            .to_string()
        };

        let bar = format!(
            r##"
            <div id="wpadminbar" class="nojq">
                <div class="quicklinks" id="wp-toolbar" role="navigation" aria-label="Toolbar">
                    <ul id="wp-admin-bar-root-default" class="ab-top-menu">
                        <li id="wp-admin-bar-wp-logo" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/">
                                <span class="ab-icon"><i class="fa-solid fa-bolt"></i></span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/">Dashboard</a></li>
                                    <li><a class="ab-item" href="/n-admin/neopress-engine.php">System Health</a></li>
                                </ul>
                            </div>
                        </li>
                        <li id="wp-admin-bar-site-name" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/">{site_name}</a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/">Dashboard</a></li>
                                    <li><a class="ab-item" href="/n-admin/themes.php">Themes</a></li>
                                    <li><a class="ab-item" href="/n-admin/widgets.php">Widgets</a></li>
                                    <li><a class="ab-item" href="/n-admin/nav-menus.php">Menus</a></li>
                                </ul>
                            </div>
                        </li>
                        <li id="wp-admin-bar-comments">
                            <a class="ab-item" href="/n-admin/edit-comments.php">
                                <span class="ab-icon"><i class="fa-solid fa-comment"></i></span>
                                <span class="ab-label">{pending_comments}</span>
                            </a>
                        </li>
                        <li id="wp-admin-bar-new-content" class="menupop">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/post-new.php">
                                <span class="ab-icon"><i class="fa-solid fa-plus"></i></span>
                                <span class="ab-label">New</span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/post-new.php">Post</a></li>
                                    <li><a class="ab-item" href="/n-admin/upload.php">Media</a></li>
                                    <li><a class="ab-item" href="/n-admin/post-new.php?post_type=page">Page</a></li>
                                    <li><a class="ab-item" href="/n-admin/user-new.php">User</a></li>
                                </ul>
                            </div>
                        </li>
                        {post_actions}
                    </ul>
                    <ul id="wp-admin-bar-top-secondary" class="ab-top-secondary ab-top-menu">
                        <li id="wp-admin-bar-my-account" class="menupop with-avatar">
                            <a class="ab-item" aria-haspopup="true" href="/n-admin/profile.php">
                                Howdy, <span class="display-name">{username}</span>
                                <span class="ab-avatar" style="margin-left: 6px;"><i class="fa-solid fa-user"></i></span>
                            </a>
                            <div class="ab-sub-wrapper">
                                <ul class="ab-submenu">
                                    <li><a class="ab-item" href="/n-admin/profile.php">Edit Profile</a></li>
                                    <li><a class="ab-item" href="/wp-login.php?action=logout" style="color: #f87171;">Log Out</a></li>
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
            username = username
        );
        (bar, " admin-bar")
    } else {
        (String::new(), "")
    }
}
