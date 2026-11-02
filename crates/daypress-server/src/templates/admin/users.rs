//! Admin Users & Profile Views (`/d-admin/users`, `/d-admin/user-new`, `/d-admin/profile`).

use super::layout::render_admin_shell;
use crate::models::{DaypressSettingsResponse, DaypressUserDetail, UserListItem};
use crate::templates::utils::escape_html;

// 8. Users View (/d-admin/users)
pub fn render_users_view(settings: &DaypressSettingsResponse, users: &[UserListItem]) -> String {
    let mut rows_html = String::new();
    for u in users {
        let initial = u
            .username
            .chars()
            .next()
            .unwrap_or('U')
            .to_uppercase()
            .to_string();
        let is_admin = u.id == 1;
        let delete_btn = if !is_admin {
            format!(
                r#"<button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px; color: var(--oxide); border-color: var(--oxide);" onclick="deleteUser({}, '{}')">Delete</button>"#,
                u.id,
                escape_html(&u.username)
            )
        } else {
            String::new()
        };

        rows_html.push_str(&format!(
            r##"
            <tr id="user-row-{id}">
                <td>
                    <div style="display: flex; align-items: center; gap: 10px;">
                        <div class="user-avatar" style="width: 32px; height: 32px; font-size: 12px; font-weight: 700;">{initial}</div>
                        <div>
                            <strong style="color: var(--ink);">{username}</strong>
                            <div style="font-size: 12px; color: var(--ink-muted);">{display_name}</div>
                        </div>
                    </div>
                </td>
                <td>{email}</td>
                <td><span class="badge badge-published">{role}</span></td>
                <td><strong>{post_count}</strong></td>
                <td>
                    <div style="display: flex; gap: 6px;">
                        <a href="/d-admin/profile?user_id={id}" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;">Edit Profile</a>
                        {delete_btn}
                    </div>
                </td>
            </tr>
            "##,
            id = u.id,
            initial = initial,
            username = escape_html(&u.username),
            display_name = escape_html(&u.display_name),
            email = escape_html(&u.email),
            role = escape_html(&u.role),
            post_count = u.post_count,
            delete_btn = delete_btn,
        ));
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Users</h1>
                <p class="page-subtitle">Manage user accounts and author permissions</p>
            </div>
            <div style="display: flex; gap: 10px;">
                <a href="/d-admin/user-new" class="btn btn-primary">+ Add New User</a>
                <a href="/d-admin/profile" class="btn btn-secondary">Edit My Profile</a>
            </div>
        </div>

        <div class="day-table-wrapper">
            <table class="day-table">
                <thead>
                    <tr>
                        <th>User</th>
                        <th>Email</th>
                        <th>Role</th>
                        <th>Posts</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
            </table>
        </div>

        <script>
            async function deleteUser(id, username) {{
                if (!confirm('Are you sure you want to delete user ' + username + '? Any authored posts will be reassigned to the administrator.')) {{
                    return;
                }}
                try {{
                    const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || '';
                    const res = await fetch('/wp-json/wp/v2/users/' + id, {{
                        method: 'DELETE',
                        headers: {{ 'X-WP-Nonce': nonce }}
                    }});
                    if (res.ok) {{
                        const row = document.getElementById('user-row-' + id);
                        if (row) row.remove();
                        showToast('✓ User ' + username + ' deleted successfully');
                    }} else {{
                        const data = await res.json();
                        alert('Failed to delete user: ' + (data.message || JSON.stringify(data)));
                    }}
                }} catch (e) {{
                    alert('Network error: ' + e);
                }}
            }}
        </script>
        "##,
        rows_html = rows_html
    );

    render_admin_shell("users", "Users", &body, settings)
}

// 8c. Add New User View (/d-admin/user-new)
pub fn render_user_new_view(settings: &DaypressSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Add New User</h1>
                <p class="page-subtitle">Create a brand new user and assign permissions</p>
            </div>
            <div>
                <a href="/d-admin/users" class="btn btn-secondary">&larr; Back to Users</a>
            </div>
        </div>

        <div class="day-card" style="max-width: 720px;">
            <div class="day-card-body">
                <form id="newUserForm" method="post" action="/d-admin/user-new" onsubmit="handleCreateUser(event)" style="display: flex; flex-direction: column; gap: 16px;">
                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                            Username <span style="color: var(--oxide);">*</span>
                        </label>
                        <input type="text" id="newUsername" name="username" required class="input" style="width: 100%; max-width: 400px; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px;" />
                    </div>

                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                            Email Address <span style="color: var(--oxide);">*</span>
                        </label>
                        <input type="email" id="newEmail" name="email" required class="input" style="width: 100%; max-width: 400px; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px;" />
                    </div>

                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px; max-width: 400px;">
                        <div>
                            <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">First Name</label>
                            <input type="text" id="newFirstName" name="first_name" class="input" style="width: 100%; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px;" />
                        </div>
                        <div>
                            <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">Last Name</label>
                            <input type="text" id="newLastName" name="last_name" class="input" style="width: 100%; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px;" />
                        </div>
                    </div>

                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">Website</label>
                        <input type="url" id="newUrl" name="url" placeholder="https://" class="input" style="width: 100%; max-width: 400px; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px;" />
                    </div>

                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                            Password <span style="color: var(--oxide);">*</span>
                        </label>
                        <div style="display: flex; gap: 8px; max-width: 400px;">
                            <input type="text" id="newPassword" name="password" required minlength="8" value="" class="input" style="flex: 1; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px; font-family: monospace;" />
                            <button type="button" class="btn btn-secondary" onclick="generateRandomPass()">Generate</button>
                        </div>
                    </div>

                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">Role</label>
                        <select id="newRole" name="role" style="padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px; width: 100%; max-width: 400px; background: white;">
                            <option value="Administrator">Administrator</option>
                            <option value="Editor">Editor</option>
                            <option value="Author">Author</option>
                            <option value="Contributor">Contributor</option>
                            <option value="Subscriber" selected>Subscriber</option>
                        </select>
                    </div>

                    <div style="margin-top: 12px;">
                        <button type="submit" class="btn btn-primary" id="btnCreateUser">Add New User</button>
                    </div>
                </form>
            </div>
        </div>

        <script>
            function generateRandomPass() {
                const chars = 'abcdefghjkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@#$%&*';
                const bytes = crypto.getRandomValues(new Uint8Array(16));
                let pass = '';
                for (let i = 0; i < bytes.length; i++) {
                    pass += chars.charAt(bytes[i] % chars.length);
                }
                document.getElementById('newPassword').value = pass;
            }
            // Every new user starts with their own random password, never a shared default.
            generateRandomPass();

            async function handleCreateUser(e) {
                e.preventDefault();
                const btn = document.getElementById('btnCreateUser');
                btn.disabled = true;
                btn.textContent = 'Creating User...';

                const username = document.getElementById('newUsername').value.trim();
                const email = document.getElementById('newEmail').value.trim();
                const firstName = document.getElementById('newFirstName').value.trim();
                const lastName = document.getElementById('newLastName').value.trim();
                const url = document.getElementById('newUrl').value.trim();
                const password = document.getElementById('newPassword').value;
                const role = document.getElementById('newRole').value;

                let displayName = username;
                if (firstName || lastName) {
                    displayName = (firstName + ' ' + lastName).trim();
                }

                try {
                    const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || '';
                    const res = await fetch('/wp-json/wp/v2/users', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'X-WP-Nonce': nonce
                        },
                        body: JSON.stringify({
                            username: username,
                            email: email,
                            password: password,
                            first_name: firstName,
                            last_name: lastName,
                            display_name: displayName,
                            role: role,
                            url: url
                        })
                    });

                    const data = await res.json();
                    if (res.ok) {
                        window.location.href = '/d-admin/users';
                    } else {
                        alert('Error creating user: ' + (data.message || JSON.stringify(data)));
                    }
                } catch (err) {
                    alert('Network error: ' + err);
                } finally {
                    btn.disabled = false;
                    btn.textContent = 'Add New User';
                }
            }
        </script>
        "##.to_string();

    render_admin_shell("users", "Add New User", &body, settings)
}

// 8b. User Profile View (/d-admin/profile and /d-admin/user-edit)
pub fn render_profile_view(
    settings: &DaypressSettingsResponse,
    user: &DaypressUserDetail,
) -> String {
    let initial = user
        .display_name
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();
    let current_ctx = crate::auth::current_admin();
    let is_admin_viewer = current_ctx
        .as_ref()
        .map(|a| a.role.eq_ignore_ascii_case("administrator"))
        .unwrap_or(true);
    let is_self = current_ctx
        .as_ref()
        .map(|a| a.user_id == user.id)
        .unwrap_or(true);
    let save_endpoint = if is_admin_viewer {
        format!("/wp-json/wp/v2/users/{}", user.id)
    } else {
        "/wp-json/wp/v2/users/me".to_string()
    };

    let disable_visual_checked = if !user.rich_editing { "checked" } else { "" };
    let disable_syntax_checked = if !user.syntax_highlighting {
        "checked"
    } else {
        ""
    };
    let comment_shortcuts_checked = if user.comment_shortcuts {
        "checked"
    } else {
        ""
    };
    let show_toolbar_checked = if user.show_admin_bar_front {
        "checked"
    } else {
        ""
    };

    let schemes: &[(&str, &str, [&str; 4])] = &[
        (
            "default",
            "Default",
            ["#1d2327", "#2c3338", "#2271b1", "#72aee6"],
        ),
        (
            "light",
            "Light",
            ["#e5e5e5", "#999999", "#d64e07", "#04a4cc"],
        ),
        (
            "modern",
            "Modern",
            ["#1e1e1e", "#3858e9", "#33f078", "#2145e6"],
        ),
        ("blue", "Blue", ["#096484", "#4796b3", "#52accc", "#74b6ce"]),
        (
            "coffee",
            "Coffee",
            ["#46403c", "#59524c", "#c7a589", "#9ea476"],
        ),
        (
            "ectoplasm",
            "Ectoplasm",
            ["#413256", "#523f6d", "#a3b745", "#d46f15"],
        ),
        (
            "midnight",
            "Midnight",
            ["#25282b", "#363b3f", "#69a8bb", "#e14d43"],
        ),
        (
            "ocean",
            "Ocean",
            ["#627c83", "#738e96", "#9ebaa0", "#aa9d88"],
        ),
        (
            "sunrise",
            "Sunrise",
            ["#b43c38", "#cf4944", "#dd823b", "#ccaf0b"],
        ),
    ];
    let active_color = if user.admin_color.trim().is_empty() {
        "default"
    } else {
        user.admin_color.trim()
    };
    let mut color_scheme_cards = String::new();
    for (id, label, swatches) in schemes {
        let checked = if id.eq_ignore_ascii_case(active_color) {
            "checked"
        } else {
            ""
        };
        let border_style = if !checked.is_empty() {
            "border-color: var(--press); background: var(--press-wash);"
        } else {
            "border-color: var(--rule);"
        };
        color_scheme_cards.push_str(&format!(
            r#"<label class="admin-color-option" style="display:flex;flex-direction:column;gap:6px;padding:8px 10px;border:1.5px solid var(--rule);border-radius:6px;cursor:pointer;transition:all 0.15s;{border_style}">
                <div style="display:flex;align-items:center;gap:6px;font-size:12px;font-weight:600;color:var(--ink);">
                    <input type="radio" name="admin_color" value="{id}" {checked} onchange="previewAdminColorScheme('{id}')" />
                    <span>{label}</span>
                </div>
                <div style="display:grid;grid-template-columns:repeat(4,1fr);height:14px;border-radius:3px;overflow:hidden;">
                    <span style="background:{c0};"></span>
                    <span style="background:{c1};"></span>
                    <span style="background:{c2};"></span>
                    <span style="background:{c3};"></span>
                </div>
            </label>"#,
            border_style = border_style,
            id = id,
            checked = checked,
            label = label,
            c0 = swatches[0],
            c1 = swatches[1],
            c2 = swatches[2],
            c3 = swatches[3],
        ));
    }

    let locales = [
        ("", "Site Default"),
        ("en_US", "English (United States)"),
        ("en_GB", "English (UK)"),
        ("nl_NL", "Nederlands"),
        ("de_DE", "Deutsch"),
        ("fr_FR", "Français"),
        ("es_ES", "Español"),
    ];
    let mut locale_options = String::new();
    for (code, label) in locales {
        let sel = if user.locale == code { "selected" } else { "" };
        locale_options.push_str(&format!(
            r#"<option value="{}" {}>{}</option>"#,
            escape_html(code),
            sel,
            escape_html(label)
        ));
    }

    let roles = [
        ("administrator", "Administrator"),
        ("editor", "Editor"),
        ("author", "Author"),
        ("contributor", "Contributor"),
        ("subscriber", "Subscriber"),
    ];
    let user_role_lower = user.role.to_ascii_lowercase();
    let mut role_options = String::new();
    for (val, label) in roles {
        let sel = if user_role_lower == val {
            "selected"
        } else {
            ""
        };
        role_options.push_str(&format!(
            r#"<option value="{}" {}>{}</option>"#,
            label, sel, label
        ));
    }
    let role_control_html = if is_admin_viewer {
        format!(
            r#"<select id="role" class="form-select" style="max-width: 320px;">{}</select>"#,
            role_options
        )
    } else {
        format!(
            r#"<input type="text" id="role" class="form-input" value="{}" disabled style="max-width: 320px; background: var(--paper-raised); color: var(--ink-muted);" />"#,
            escape_html(&user.role)
        )
    };

    let nickname_val = if user.nickname.trim().is_empty() {
        &user.username
    } else {
        &user.nickname
    };

    let mut display_candidates: Vec<String> = Vec::new();
    let mut push_unique = |s: String| {
        let t = s.trim().to_string();
        if !t.is_empty() && !display_candidates.iter().any(|c| c == &t) {
            display_candidates.push(t);
        }
    };
    push_unique(user.display_name.clone());
    push_unique(user.username.clone());
    push_unique(nickname_val.to_string());
    push_unique(user.first_name.clone());
    push_unique(user.last_name.clone());
    if !user.first_name.trim().is_empty() && !user.last_name.trim().is_empty() {
        push_unique(format!(
            "{} {}",
            user.first_name.trim(),
            user.last_name.trim()
        ));
        push_unique(format!(
            "{} {}",
            user.last_name.trim(),
            user.first_name.trim()
        ));
    }
    let mut display_name_options = String::new();
    for cand in &display_candidates {
        let sel = if cand == &user.display_name {
            "selected"
        } else {
            ""
        };
        display_name_options.push_str(&format!(
            r#"<option value="{0}" {1}>{0}</option>"#,
            escape_html(cand),
            sel
        ));
    }

    let gravatar_default =
        crate::adapters::database::users::gravatar_url_for_email(&user.email, 128);
    let resolved_avatar = if !user.avatar_url.trim().is_empty() {
        user.avatar_url.trim().to_string()
    } else {
        gravatar_default.clone()
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">{page_heading}</h1>
                <p class="page-subtitle">Personal options, public author identity, security credentials, active sessions, and REST Application Passwords</p>
            </div>
            <div style="display: flex; gap: 10px;">
                <a href="/author/{username}/" target="_blank" class="btn btn-secondary">View Author Archive &nearr;</a>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('profileFormSubmit').click()">Update Profile</button>
            </div>
        </div>

        <form id="profileForm" onsubmit="saveProfile(event)">
            <button type="submit" id="profileFormSubmit" style="display: none;"></button>

            <!-- User Summary Header Card -->
            <div class="day-card" style="padding: 24px; margin-bottom: 24px; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 16px;">
                <div style="display: flex; align-items: center; gap: 18px;">
                    <img id="headerAvatarImg" src="{resolved_avatar}" alt="{display_name}" onerror="this.style.display='none';document.getElementById('headerAvatarFallback').style.display='flex';" style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover; border: 2px solid var(--rule);" />
                    <div id="headerAvatarFallback" style="display: none; width: 64px; height: 64px; border-radius: 50%; background: var(--press); color: var(--paper-raised); align-items: center; justify-content: center; font-size: 26px; font-weight: 700;">
                        {initial}
                    </div>
                    <div>
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <h2 style="font-size: 20px; font-weight: 700; color: var(--ink); margin: 0;">{display_name}</h2>
                            <span class="badge badge-published">{role}</span>
                        </div>
                        <div style="color: var(--ink-muted); font-size: 13px; margin-top: 4px;">
                            Username: <code style="font-family: var(--font-mono); background: var(--paper-sunk); padding: 2px 6px; border-radius: 4px; color: var(--ink);">{username}</code> &bull; Registered: {registered}
                        </div>
                    </div>
                </div>
                <div style="text-align: right;">
                    <span style="font-size: 12px; color: var(--ink-muted); display: block;">User ID</span>
                    <strong style="font-family: var(--font-mono); font-size: 18px; color: var(--ink);">#{id}</strong>
                </div>
            </div>

            <!-- 1. Personal Options -->
            <div class="day-card" style="margin-bottom: 24px;">
                <div class="day-card-header">
                    <h2 class="day-card-title">Personal Options</h2>
                </div>
                <div class="day-card-body" style="display: flex; flex-direction: column; gap: 16px;">
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 14px;">
                        <label style="display: flex; align-items: center; gap: 10px; font-size: 13.5px; cursor: pointer;">
                            <input type="checkbox" id="disable_visual_editor" {disable_visual_checked} />
                            <span><strong>Visual Editor:</strong> Disable the visual editor when writing</span>
                        </label>
                        <label style="display: flex; align-items: center; gap: 10px; font-size: 13.5px; cursor: pointer;">
                            <input type="checkbox" id="disable_syntax_highlighting" {disable_syntax_checked} />
                            <span><strong>Syntax Highlighting:</strong> Disable syntax highlighting when editing code</span>
                        </label>
                        <label style="display: flex; align-items: center; gap: 10px; font-size: 13.5px; cursor: pointer;">
                            <input type="checkbox" id="comment_shortcuts" {comment_shortcuts_checked} />
                            <span><strong>Keyboard Shortcuts:</strong> Enable keyboard shortcuts for comment moderation</span>
                        </label>
                        <label style="display: flex; align-items: center; gap: 10px; font-size: 13.5px; cursor: pointer;">
                            <input type="checkbox" id="show_admin_bar_front" {show_toolbar_checked} />
                            <span><strong>Toolbar:</strong> Show Toolbar when viewing site</span>
                        </label>
                    </div>

                    <div class="form-group" style="margin-top: 4px; margin-bottom: 0;">
                        <label class="form-label">Admin Color Scheme</label>
                        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(155px, 1fr)); gap: 10px;">
                            {color_scheme_cards}
                        </div>
                    </div>

                    <div class="form-group" style="margin-bottom: 0; max-width: 340px;">
                        <label class="form-label" for="locale">Language</label>
                        <select id="locale" class="form-select">
                            {locale_options}
                        </select>
                    </div>
                </div>
            </div>

            <div style="display: grid; grid-template-columns: 3fr 2fr; gap: 24px;">
                <!-- Left Column: Name, Contact Info, About Yourself -->
                <div style="display: flex; flex-direction: column; gap: 24px;">
                    <div class="day-card">
                        <div class="day-card-header">
                            <h2 class="day-card-title">Name</h2>
                        </div>
                        <div class="day-card-body">
                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                                <div class="form-group">
                                    <label class="form-label">Username</label>
                                    <input type="text" id="profile_username" class="form-input" value="{username}" disabled style="background: var(--paper-raised); color: var(--ink-muted); cursor: not-allowed;" />
                                    <small style="color: var(--ink-muted); font-size: 11.5px; margin-top: 4px; display: block;">Usernames cannot be changed.</small>
                                </div>
                                <div class="form-group">
                                    <label class="form-label" for="role">Role</label>
                                    {role_control_html}
                                </div>
                            </div>

                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                                <div class="form-group">
                                    <label class="form-label" for="first_name">First Name</label>
                                    <input type="text" id="first_name" class="form-input" value="{first_name}" oninput="rebuildDisplayNameOptions()" />
                                </div>
                                <div class="form-group">
                                    <label class="form-label" for="last_name">Last Name</label>
                                    <input type="text" id="last_name" class="form-input" value="{last_name}" oninput="rebuildDisplayNameOptions()" />
                                </div>
                            </div>

                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                                <div class="form-group" style="margin-bottom: 0;">
                                    <label class="form-label" for="nickname">Nickname (Required)</label>
                                    <input type="text" id="nickname" class="form-input" value="{nickname}" required oninput="rebuildDisplayNameOptions()" />
                                </div>
                                <div class="form-group" style="margin-bottom: 0;">
                                    <label class="form-label" for="display_name">Display name publicly as</label>
                                    <select id="display_name" class="form-select">
                                        {display_name_options}
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="day-card">
                        <div class="day-card-header">
                            <h2 class="day-card-title">Contact Info</h2>
                        </div>
                        <div class="day-card-body">
                            <div class="form-group">
                                <label class="form-label" for="email">Email (Required)</label>
                                <input type="email" id="email" class="form-input" value="{email}" required />
                                <small style="color: var(--ink-muted); font-size: 11.5px; margin-top: 4px; display: block;">If you change this, we will update your Gravatar and administrative notifications immediately.</small>
                            </div>

                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" for="url">Website</label>
                                <input type="url" id="url" class="form-input" value="{url}" placeholder="https://yourwebsite.com" />
                            </div>
                        </div>
                    </div>

                    <div class="day-card">
                        <div class="day-card-header">
                            <h2 class="day-card-title">About Yourself</h2>
                        </div>
                        <div class="day-card-body">
                            <div class="form-group">
                                <label class="form-label" for="description">Biographical Info</label>
                                <textarea id="description" rows="4" class="form-textarea" placeholder="Share a little biographical information to fill out your profile. This is shown on your public Author Archive page.">{description}</textarea>
                                <small style="color: var(--ink-muted); font-size: 11.5px; margin-top: 4px; display: block;">Displayed publicly on <code>/author/{username}/</code> and in REST user responses.</small>
                            </div>

                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" for="avatar_url">Profile Picture</label>
                                <div style="display: flex; align-items: center; gap: 16px;">
                                    <img id="avatarPreviewImg" src="{resolved_avatar}" alt="Profile Picture" style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover; border: 1px solid var(--rule); flex-shrink: 0;" />
                                    <div style="flex: 1;">
                                        <input type="text" id="avatar_url" class="form-input" value="{avatar_url}" placeholder="Leave blank for automatic Gravatar ({gravatar_default})" oninput="updateAvatarPreview()" />
                                        <div style="display: flex; gap: 8px; align-items: center; margin-top: 6px;">
                                            <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 4px 10px;" onclick="resetAvatarToGravatar()">Use Default Gravatar</button>
                                            <span style="font-size: 11.5px; color: var(--ink-muted);">Paste any image or `/uploads/...` URL, or leave blank for Gravatar.</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Right Column: Account Management, Sessions & Application Passwords -->
                <div style="display: flex; flex-direction: column; gap: 24px;">
                    <div class="day-card">
                        <div class="day-card-header">
                            <h2 class="day-card-title">Account Management</h2>
                        </div>
                        <div class="day-card-body">
                            <div class="form-group">
                                <label class="form-label" for="password">New Password</label>
                                <input type="password" id="password" class="form-input" placeholder="Leave blank to keep current password" oninput="evaluatePasswordStrength(this.value)" />
                                <div id="passwordStrengthBar" style="display: none; margin-top: 8px; padding: 6px 10px; border-radius: 4px; font-size: 12px; font-weight: 600;"></div>
                            </div>
                            <div style="display: flex; gap: 8px; margin-bottom: 18px;">
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 6px 10px;" onclick="generatePassword()">Generate Password</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 6px 10px;" onclick="togglePasswordVisibility()">Show / Hide</button>
                            </div>

                            <div style="border-top: 1px solid var(--rule); padding-top: 16px;">
                                <label class="form-label">Sessions</label>
                                <p style="font-size: 12.5px; color: var(--ink-muted); margin: 0 0 10px 0; line-height: 1.5;">
                                    Did you lose your phone or leave your account logged in at a public computer? You can log out everywhere else, leaving only this browser signed in.
                                </p>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="destroyOtherSessions()">Log Out Everywhere Else</button>
                            </div>
                        </div>
                    </div>

                    <div class="day-card">
                        <div class="day-card-header" style="display: flex; align-items: center; justify-content: space-between;">
                            <h2 class="day-card-title">Two-Factor Authentication (2FA)</h2>
                            <span id="twoFactorBadge" class="badge {totp_badge_class}">{totp_badge_label}</span>
                        </div>
                        <div class="day-card-body">
                            <p style="font-size: 12.5px; color: var(--ink-muted); margin: 0 0 12px 0; line-height: 1.5;">
                                Protect this account with an RFC 6238 authenticator app (1Password, Google Authenticator, Authy, Apple Passwords, Bitwarden) and single-use backup recovery codes.
                            </p>
                            <div id="twoFactorStatusInfo" style="font-size: 12.5px; color: var(--ink); margin-bottom: 12px;"></div>
                            <div id="twoFactorSetupBox" style="display: none; margin-bottom: 14px; padding: 14px; border: 1px solid var(--rule-strong); border-radius: 6px; background: var(--paper-sunk);">
                                <div style="font-size: 12px; font-weight: 600; color: var(--ink); margin-bottom: 6px;">1. Add this secret key to your authenticator app:</div>
                                <code id="twoFactorSecretDisplay" style="display: block; padding: 8px 10px; background: var(--paper-raised); border: 1px solid var(--rule); border-radius: 4px; font-family: var(--font-mono); font-size: 13px; letter-spacing: 0.08em; word-break: break-all; margin-bottom: 8px;"></code>
                                <div style="margin-bottom: 12px;">
                                    <a id="twoFactorUriLink" href="#" style="font-size: 12px; color: var(--oxide); text-decoration: underline;">Open in Authenticator App (otpauth://)</a>
                                </div>
                                <div style="font-size: 12px; font-weight: 600; color: var(--ink); margin-bottom: 6px;">2. Enter the 6-digit verification code from your app:</div>
                                <div style="display: flex; gap: 8px; align-items: center;">
                                    <input type="text" id="twoFactorVerifyCode" class="form-input" placeholder="123456" maxlength="10" style="max-width: 160px; font-family: var(--font-mono); letter-spacing: 0.1em;" />
                                    <button type="button" class="btn btn-primary" style="font-size: 12px;" onclick="verifyAndEnableTwoFactor()">Verify &amp; Enable 2FA</button>
                                </div>
                            </div>
                            <div id="twoFactorRecoveryBox" style="display: none; margin-bottom: 14px; padding: 12px; background: var(--ink); color: var(--paper); border-radius: 6px; font-family: var(--font-mono); font-size: 12px;">
                                <div style="font-weight: 600; margin-bottom: 6px; color: #fcd34d;">Save these single-use recovery codes now (each code works once):</div>
                                <div id="twoFactorRecoveryList" style="display: grid; grid-template-columns: 1fr 1fr; gap: 6px;"></div>
                            </div>
                            <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                                <button type="button" id="btnSetup2fa" class="btn btn-secondary" style="font-size: 12px;" onclick="startTwoFactorSetup()">Set Up Authenticator App</button>
                                <button type="button" id="btnRegenRecovery2fa" class="btn btn-secondary" style="font-size: 12px; display: none;" onclick="regenerateRecoveryCodes()">Regenerate Recovery Codes</button>
                                <button type="button" id="btnDisable2fa" class="btn btn-secondary" style="font-size: 12px; color: var(--oxide); display: none;" onclick="disableTwoFactor()">Disable 2FA</button>
                            </div>
                        </div>
                    </div>

                    <div class="day-card">
                        <div class="day-card-header">
                            <h2 class="day-card-title">Application Passwords</h2>
                        </div>
                        <div class="day-card-body">
                            <p style="font-size: 12.5px; color: var(--ink-muted); margin-bottom: 12px; line-height: 1.5;">
                                Application passwords allow authentication via non-interactive systems, such as the REST API or WP-CLI, without providing your actual password.
                            </p>
                            <div style="display: flex; gap: 8px; align-items: center; margin-bottom: 12px;">
                                <input type="text" id="new_app_pass_name" class="form-input" placeholder="New Application Password Name (e.g. Headless CLI)" style="flex: 1;" />
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; white-space: nowrap;" onclick="generateApiToken()">Add New Application Password</button>
                            </div>
                            <div id="tokenOutput" style="display: none; margin-bottom: 14px; padding: 10px 12px; background: var(--ink); color: var(--paper); font-family: var(--font-mono); font-size: 12px; border-radius: 6px; word-break: break-all;"></div>

                            <div id="appPasswordsTableWrap" style="margin-top: 12px;">
                                <table class="day-table" style="font-size: 12px;">
                                    <thead>
                                        <tr>
                                            <th>Name</th>
                                            <th>Created</th>
                                            <th>Last Used</th>
                                            <th style="text-align: right;">Action</th>
                                        </tr>
                                    </thead>
                                    <tbody id="appPasswordsTbody">
                                        <tr><td colspan="4" style="color: var(--ink-muted); text-align: center;">Loading application passwords...</td></tr>
                                    </tbody>
                                </table>
                                <div id="revokeAllAppPassWrap" style="display: none; margin-top: 10px; text-align: right;">
                                    <button type="button" class="btn btn-secondary" style="font-size: 11.5px; color: var(--oxide);" onclick="revokeAllAppPasswords()">Revoke All Application Passwords</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </form>

        <script>
            const profileUserId = {id};
            const profileSaveEndpoint = '{save_endpoint}';
            const defaultGravatarUrl = '{gravatar_default}';

            function previewAdminColorScheme(scheme) {{
                document.body.setAttribute('data-admin-color', scheme);
                document.querySelectorAll('.admin-color-option').forEach(lbl => {{
                    const radio = lbl.querySelector('input[name="admin_color"]');
                    if (radio && radio.value === scheme) {{
                        lbl.style.borderColor = 'var(--press)';
                        lbl.style.background = 'var(--press-wash)';
                    }} else {{
                        lbl.style.borderColor = 'var(--rule)';
                        lbl.style.background = 'transparent';
                    }}
                }});
            }}

            function rebuildDisplayNameOptions() {{
                const sel = document.getElementById('display_name');
                if (!sel) return;
                const currentVal = sel.value;
                const username = document.getElementById('profile_username').value.trim();
                const first = document.getElementById('first_name').value.trim();
                const last = document.getElementById('last_name').value.trim();
                const nick = document.getElementById('nickname').value.trim();

                const candidates = [];
                const addUnique = (v) => {{
                    const t = (v || '').trim();
                    if (t && !candidates.includes(t)) candidates.push(t);
                }};
                addUnique(username);
                addUnique(nick);
                addUnique(first);
                addUnique(last);
                if (first && last) {{
                    addUnique(first + ' ' + last);
                    addUnique(last + ' ' + first);
                }}
                addUnique(currentVal);

                sel.innerHTML = candidates.map(c => {{
                    const esc = c.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');
                    const selected = c === currentVal ? ' selected' : '';
                    return '<option value="' + esc + '"' + selected + '>' + esc + '</option>';
                }}).join('');
            }}

            function updateAvatarPreview() {{
                const raw = document.getElementById('avatar_url').value.trim();
                const target = raw || defaultGravatarUrl;
                const img1 = document.getElementById('avatarPreviewImg');
                const img2 = document.getElementById('headerAvatarImg');
                if (img1) img1.src = target;
                if (img2) {{
                    img2.style.display = 'block';
                    img2.src = target;
                }}
                const fb = document.getElementById('headerAvatarFallback');
                if (fb) fb.style.display = 'none';
            }}

            function resetAvatarToGravatar() {{
                document.getElementById('avatar_url').value = '';
                updateAvatarPreview();
                showToast('✓ Reverted to automatic Gravatar');
            }}

            function evaluatePasswordStrength(pw) {{
                const bar = document.getElementById('passwordStrengthBar');
                if (!bar) return;
                if (!pw) {{
                    bar.style.display = 'none';
                    return;
                }}
                bar.style.display = 'block';
                let score = 0;
                if (pw.length >= 8) score++;
                if (pw.length >= 14) score++;
                if (/[A-Z]/.test(pw) && /[a-z]/.test(pw)) score++;
                if (/[0-9]/.test(pw) && /[^A-Za-z0-9]/.test(pw)) score++;

                if (score <= 1) {{
                    bar.style.background = 'rgba(220,38,38,0.12)';
                    bar.style.color = '#dc2626';
                    bar.textContent = 'Strength: Weak — use at least 12 characters with mixed symbols';
                }} else if (score <= 2) {{
                    bar.style.background = 'rgba(217,119,6,0.12)';
                    bar.style.color = '#d97706';
                    bar.textContent = 'Strength: Medium';
                }} else {{
                    bar.style.background = 'rgba(5,150,105,0.12)';
                    bar.style.color = '#059669';
                    bar.textContent = 'Strength: Strong';
                }}
            }}

            function generatePassword() {{
                const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+';
                const bytes = crypto.getRandomValues(new Uint8Array(20));
                let pass = '';
                for (let i = 0; i < bytes.length; i++) {{
                    pass += chars.charAt(bytes[i] % chars.length);
                }}
                const input = document.getElementById('password');
                input.type = 'text';
                input.value = pass;
                evaluatePasswordStrength(pass);
                showToast('✓ Generated strong 20-character password');
            }}

            function togglePasswordVisibility() {{
                const input = document.getElementById('password');
                input.type = input.type === 'password' ? 'text' : 'password';
            }}

            async function destroyOtherSessions() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/sessions/destroy-others', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    const data = await res.json().catch(() => ({{}}));
                    if (res.ok) {{
                        showToast('✓ ' + (data.message || 'Logged out of all other sessions!'));
                    }} else {{
                        showToast('Failed to log out other sessions', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function loadApplicationPasswords() {{
                const tbody = document.getElementById('appPasswordsTbody');
                const revokeAllWrap = document.getElementById('revokeAllAppPassWrap');
                if (!tbody) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/users/' + profileUserId + '/application-passwords');
                    if (!res.ok) return;
                    const list = await res.json();
                    if (!list || list.length === 0) {{
                        tbody.innerHTML = '<tr><td colspan="4" style="color: var(--ink-muted); text-align: center;">No active application passwords.</td></tr>';
                        if (revokeAllWrap) revokeAllWrap.style.display = 'none';
                        return;
                    }}
                    if (revokeAllWrap) revokeAllWrap.style.display = 'block';
                    tbody.innerHTML = list.map(item => {{
                        const nameEsc = (item.name || '').replace(/&/g, '&amp;').replace(/</g, '&lt;');
                        const created = (item.created || '').replace('T', ' ');
                        const lastUsed = item.last_used ? item.last_used.replace('T', ' ') : 'Never';
                        return '<tr>'
                            + '<td><strong>' + nameEsc + '</strong></td>'
                            + '<td style="font-family:var(--font-mono);font-size:11px;">' + created + '</td>'
                            + '<td style="font-family:var(--font-mono);font-size:11px;">' + lastUsed + '</td>'
                            + '<td style="text-align:right;"><button type="button" class="btn btn-secondary" style="font-size:11px;padding:3px 8px;color:var(--oxide);" onclick="revokeAppPassword(\'' + item.uuid + '\')">Revoke</button></td>'
                            + '</tr>';
                    }}).join('');
                }} catch (_) {{}}
            }}

            async function generateApiToken() {{
                const nameInput = document.getElementById('new_app_pass_name');
                const appName = (nameInput && nameInput.value.trim()) ? nameInput.value.trim() : 'Daypress API Client';
                try {{
                    const res = await fetch('/wp-json/wp/v2/users/' + profileUserId + '/application-passwords', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ name: appName }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.password) {{
                        const out = document.getElementById('tokenOutput');
                        out.style.display = 'block';
                        out.textContent = 'New password for "' + appName + '": ' + data.password + ' (copied to clipboard — save this now, it will not be shown again)';
                        if (nameInput) nameInput.value = '';
                        if (navigator.clipboard) navigator.clipboard.writeText(data.password);
                        loadApplicationPasswords();
                        showToast('✓ Application Password created and persisted in SQLite!');
                    }} else {{
                        showToast('Failed to create application password', true);
                    }}
                }} catch (err) {{
                    showToast('Failed to generate token: ' + err, true);
                }}
            }}

            async function revokeAppPassword(uuid) {{
                if (!uuid) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/users/' + profileUserId + '/application-passwords/' + uuid, {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        showToast('✓ Application password revoked');
                        loadApplicationPasswords();
                    }} else {{
                        showToast('Failed to revoke application password', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function revokeAllAppPasswords() {{
                if (!confirm('Revoke all application passwords for this user?')) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/users/' + profileUserId + '/application-passwords', {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        showToast('✓ All application passwords revoked');
                        loadApplicationPasswords();
                    }} else {{
                        showToast('Failed to revoke application passwords', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function saveProfile(e) {{
                e.preventDefault();
                const display_name = document.getElementById('display_name').value.trim();
                const first_name = document.getElementById('first_name').value.trim();
                const last_name = document.getElementById('last_name').value.trim();
                const nickname = document.getElementById('nickname').value.trim();
                const email = document.getElementById('email').value.trim();
                const url = document.getElementById('url').value.trim();
                const description = document.getElementById('description').value;
                const avatar_url = document.getElementById('avatar_url').value.trim();
                const locale = document.getElementById('locale').value;
                const roleEl = document.getElementById('role');
                const role = (roleEl && !roleEl.disabled) ? roleEl.value : undefined;
                const rich_editing = !document.getElementById('disable_visual_editor').checked;
                const syntax_highlighting = !document.getElementById('disable_syntax_highlighting').checked;
                const comment_shortcuts = document.getElementById('comment_shortcuts').checked;
                const show_admin_bar_front = document.getElementById('show_admin_bar_front').checked;
                const colorRadio = document.querySelector('input[name="admin_color"]:checked');
                const admin_color = colorRadio ? colorRadio.value : 'default';
                const password = document.getElementById('password').value;

                const payload = {{
                    display_name: display_name || undefined,
                    first_name,
                    last_name,
                    nickname: nickname || undefined,
                    email: email || undefined,
                    url,
                    description,
                    avatar_url,
                    locale,
                    role,
                    rich_editing,
                    syntax_highlighting,
                    comment_shortcuts,
                    show_admin_bar_front,
                    admin_color,
                    password: password.length > 0 ? password : undefined
                }};

                try {{
                    const res = await fetch(profileSaveEndpoint, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});

                    if (res.ok) {{
                        showToast('✓ Profile updated and persisted in SQLite!');
                        setTimeout(() => window.location.reload(), 500);
                    }} else {{
                        const err = await res.text();
                        showToast('Failed to update profile: ' + err, true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function loadTwoFactorStatus() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/2fa/status');
                    if (!res.ok) return;
                    const data = await res.json();
                    const badge = document.getElementById('twoFactorBadge');
                    const info = document.getElementById('twoFactorStatusInfo');
                    const btnSetup = document.getElementById('btnSetup2fa');
                    const btnRegen = document.getElementById('btnRegenRecovery2fa');
                    const btnDisable = document.getElementById('btnDisable2fa');
                    if (data.enabled) {{
                        if (badge) {{
                            badge.className = 'badge badge-published';
                            badge.textContent = 'Enabled';
                        }}
                        if (info) {{
                            info.textContent = 'Authenticator app active • ' + (data.recovery_codes_remaining || 0) + ' single-use recovery codes remaining.';
                        }}
                        if (btnSetup) btnSetup.textContent = 'Reconfigure Authenticator';
                        if (btnRegen) btnRegen.style.display = 'inline-block';
                        if (btnDisable) btnDisable.style.display = 'inline-block';
                    }} else {{
                        if (badge) {{
                            badge.className = 'badge badge-draft';
                            badge.textContent = 'Not Enabled';
                        }}
                        if (info) {{
                            info.textContent = 'Two-factor authentication is currently disabled for this account.';
                        }}
                        if (btnSetup) btnSetup.textContent = 'Set Up Authenticator App';
                        if (btnRegen) btnRegen.style.display = 'none';
                        if (btnDisable) btnDisable.style.display = 'none';
                    }}
                }} catch (_) {{}}
            }}

            async function startTwoFactorSetup() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/2fa/setup', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    const data = await res.json();
                    if (res.ok && data.secret) {{
                        const box = document.getElementById('twoFactorSetupBox');
                        const sec = document.getElementById('twoFactorSecretDisplay');
                        const link = document.getElementById('twoFactorUriLink');
                        if (box) box.style.display = 'block';
                        if (sec) sec.textContent = data.secret;
                        if (link) link.href = data.otpauth_uri || '#';
                        showToast('✓ Generated new TOTP secret — verify with a 6-digit code to activate');
                    }} else {{
                        showToast(data.message || 'Failed to start 2FA setup', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function verifyAndEnableTwoFactor() {{
                const input = document.getElementById('twoFactorVerifyCode');
                const code = input ? input.value.trim() : '';
                if (!code) {{
                    showToast('Enter the 6-digit code from your authenticator app', true);
                    return;
                }}
                try {{
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/2fa/enable', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ code }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.enabled) {{
                        const setupBox = document.getElementById('twoFactorSetupBox');
                        if (setupBox) setupBox.style.display = 'none';
                        if (input) input.value = '';
                        renderRecoveryCodesList(data.recovery_codes || []);
                        loadTwoFactorStatus();
                        showToast('✓ Two-Factor Authentication enabled!');
                    }} else {{
                        showToast(data.message || 'Invalid verification code', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            function renderRecoveryCodesList(codes) {{
                const box = document.getElementById('twoFactorRecoveryBox');
                const list = document.getElementById('twoFactorRecoveryList');
                if (!box || !list || !codes || !codes.length) return;
                box.style.display = 'block';
                list.innerHTML = codes.map(c => '<div>' + c + '</div>').join('');
                if (navigator.clipboard) {{
                    navigator.clipboard.writeText(codes.join('\n')).catch(() => {{}});
                }}
            }}

            async function regenerateRecoveryCodes() {{
                if (!confirm('Generate 8 new backup recovery codes? Any previous unused recovery codes will be invalidated immediately.')) return;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/2fa/recovery-codes', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    const data = await res.json();
                    if (res.ok && data.recovery_codes) {{
                        renderRecoveryCodesList(data.recovery_codes);
                        loadTwoFactorStatus();
                        showToast('✓ Generated 8 fresh backup recovery codes (copied to clipboard)');
                    }} else {{
                        showToast(data.message || 'Failed to regenerate recovery codes', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function disableTwoFactor() {{
                if (!confirm('Disable Two-Factor Authentication for this account?')) return;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/2fa/disable', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    if (res.ok) {{
                        const setupBox = document.getElementById('twoFactorSetupBox');
                        const recBox = document.getElementById('twoFactorRecoveryBox');
                        if (setupBox) setupBox.style.display = 'none';
                        if (recBox) recBox.style.display = 'none';
                        loadTwoFactorStatus();
                        showToast('✓ Two-Factor Authentication disabled');
                    }} else {{
                        showToast('Failed to disable Two-Factor Authentication', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            document.addEventListener('DOMContentLoaded', () => {{
                loadApplicationPasswords();
                loadTwoFactorStatus();
            }});
        </script>
        "##,
        page_heading = if is_self {
            "Profile".to_string()
        } else {
            format!("Edit User: {}", escape_html(&user.username))
        },
        totp_badge_class = if user.totp_enabled {
            "badge-published"
        } else {
            "badge-draft"
        },
        totp_badge_label = if user.totp_enabled {
            "Enabled"
        } else {
            "Not Enabled"
        },
        initial = initial,
        id = user.id,
        username = escape_html(&user.username),
        display_name = escape_html(&user.display_name),
        first_name = escape_html(&user.first_name),
        last_name = escape_html(&user.last_name),
        nickname = escape_html(nickname_val),
        description = escape_html(&user.description),
        avatar_url = escape_html(&user.avatar_url),
        resolved_avatar = escape_html(&resolved_avatar),
        gravatar_default = escape_html(&gravatar_default),
        email = escape_html(&user.email),
        url = escape_html(&user.url),
        registered = escape_html(&user.registered),
        role = escape_html(&user.role),
        role_control_html = role_control_html,
        disable_visual_checked = disable_visual_checked,
        disable_syntax_checked = disable_syntax_checked,
        comment_shortcuts_checked = comment_shortcuts_checked,
        show_toolbar_checked = show_toolbar_checked,
        color_scheme_cards = color_scheme_cards,
        locale_options = locale_options,
        display_name_options = display_name_options,
        save_endpoint = save_endpoint,
    );

    render_admin_shell("profile", "Profile", &body, settings)
}
