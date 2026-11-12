//! Admin Users & Profile Views (`/d-admin/users`, `/d-admin/user-new`, `/d-admin/profile`, `/d-admin/user-edit`).

use super::layout::render_admin_shell;
use crate::models::{DaypressSettingsResponse, DaypressUserDetail, UserListItem};
use crate::templates::utils::escape_html;

// 8. Users View (/d-admin/users)
pub fn render_users_view(settings: &DaypressSettingsResponse, users: &[UserListItem]) -> String {
    let user_count = users.len();
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
        let delete_action = if !is_admin {
            format!(
                r#" | <button type="button" class="row-action-btn trash-action" onclick="deleteUser({}, '{}')">Delete</button>"#,
                u.id,
                escape_html(&u.username)
            )
        } else {
            String::new()
        };

        rows_html.push_str(&format!(
            r##"
            <tr id="user-row-{id}" data-user-row>
                <td style="width: 32px;"><input type="checkbox" name="users[]" value="{id}" /></td>
                <td class="table-title-cell">
                    <div style="display: flex; align-items: flex-start; gap: 10px;">
                        <img src="{avatar_url}" alt="{username}" onerror="this.style.display='none';this.nextElementSibling.style.display='flex';" style="width: 32px; height: 32px; border-radius: 3px; object-fit: cover; border: 1px solid var(--rule);" />
                        <div class="user-avatar" style="display: none; width: 32px; height: 32px; font-size: 12px; font-weight: 700; align-items: center; justify-content: center; background: var(--paper-sunk); border: 1px solid var(--rule); border-radius: 3px;">{initial}</div>
                        <div>
                            <strong><a href="/d-admin/user-edit?user_id={id}">{username}</a></strong>
                            <div class="table-actions">
                                <a href="/d-admin/user-edit?user_id={id}">Edit</a>{delete_action}
                            </div>
                        </div>
                    </div>
                </td>
                <td>{display_name}</td>
                <td><a href="mailto:{email}" style="color: var(--oxide); text-decoration: none;">{email}</a></td>
                <td><span class="badge badge-published">{role}</span></td>
                <td><strong>{post_count}</strong></td>
            </tr>
            "##,
            id = u.id,
            initial = initial,
            avatar_url = escape_html(&u.avatar_url),
            username = escape_html(&u.username),
            display_name = escape_html(&u.display_name),
            email = escape_html(&u.email),
            role = escape_html(&u.role),
            post_count = u.post_count,
            delete_action = delete_action,
        ));
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Users</h1>
                <a href="/d-admin/user-new" class="page-title-action">+ Add New User</a>
                <a href="/d-admin/profile" class="page-title-action">Edit My Profile</a>
            </div>
        </div>

        <div class="wp-subsubsub-row">
            <div class="subsubsub">
                <a href="/d-admin/users" class="current">All <span class="count">({user_count})</span></a>
            </div>
            <div style="display: flex; gap: 6px; align-items: center;">
                <input type="search" id="userSearchInput" placeholder="Search users..." class="form-input" style="width: 190px; padding: 4px 10px; min-height: 30px; font-size: 13px;" oninput="filterUserRows(this.value)" />
                <button type="button" class="btn btn-secondary" style="padding: 4px 10px; min-height: 30px; font-size: 13px;" onclick="filterUserRows(document.getElementById('userSearchInput').value)">Search Users</button>
            </div>
        </div>

        <div class="tablenav top">
            <div class="tablenav-actions">
                <span style="font-size: 13px; color: var(--ink-muted);">Manage accounts, roles, and author permissions</span>
            </div>
            <div class="tablenav-pages">
                <span>{user_count} items</span>
            </div>
        </div>

        <div class="day-table-wrapper">
            <table class="day-table" id="usersTable">
                <thead>
                    <tr>
                        <th style="width: 32px;"><input type="checkbox" onchange="document.querySelectorAll('#usersTable input[name=&quot;users[]&quot;]').forEach(cb => cb.checked = this.checked)" /></th>
                        <th>Username</th>
                        <th>Name</th>
                        <th>Email</th>
                        <th>Role</th>
                        <th>Posts</th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
                <tfoot>
                    <tr>
                        <th style="width: 32px;"><input type="checkbox" onchange="document.querySelectorAll('#usersTable input[name=&quot;users[]&quot;]').forEach(cb => cb.checked = this.checked)" /></th>
                        <th>Username</th>
                        <th>Name</th>
                        <th>Email</th>
                        <th>Role</th>
                        <th>Posts</th>
                    </tr>
                </tfoot>
            </table>
        </div>

        <script>
            function filterUserRows(q) {{
                const filter = (q || '').toLowerCase();
                document.querySelectorAll('#usersTable tbody tr[data-user-row]').forEach(r => {{
                    r.style.display = (!filter || r.textContent.toLowerCase().includes(filter)) ? '' : 'none';
                }});
            }}

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
        rows_html = rows_html,
        user_count = user_count
    );

    render_admin_shell("users", "Users", &body, settings)
}

// 8c. Add New User View (/d-admin/user-new)
pub fn render_user_new_view(settings: &DaypressSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Add New User</h1>
                <a href="/d-admin/users" class="page-title-action">&larr; All Users</a>
                <p class="page-subtitle">Create a brand new user and add them to this site.</p>
            </div>
        </div>

        <form id="newUserForm" method="post" action="/d-admin/user-new" onsubmit="handleCreateUser(event)">
            <table class="form-table" role="presentation">
                <tbody>
                    <tr>
                        <th scope="row"><label for="newUsername">Username <span class="description">(required)</span></label></th>
                        <td><input type="text" id="newUsername" name="username" required class="form-input regular-text" /></td>
                    </tr>
                    <tr>
                        <th scope="row"><label for="newEmail">Email <span class="description">(required)</span></label></th>
                        <td><input type="email" id="newEmail" name="email" required class="form-input regular-text" /></td>
                    </tr>
                    <tr>
                        <th scope="row"><label for="newFirstName">First Name</label></th>
                        <td><input type="text" id="newFirstName" name="first_name" class="form-input regular-text" /></td>
                    </tr>
                    <tr>
                        <th scope="row"><label for="newLastName">Last Name</label></th>
                        <td><input type="text" id="newLastName" name="last_name" class="form-input regular-text" /></td>
                    </tr>
                    <tr>
                        <th scope="row"><label for="newUrl">Website</label></th>
                        <td><input type="text" id="newUrl" name="url" placeholder="https://" class="form-input regular-text" /></td>
                    </tr>
                    <tr>
                        <th scope="row"><label for="newPassword">Password <span class="description">(required)</span></label></th>
                        <td>
                            <div style="display: flex; gap: 8px; max-width: 380px;">
                                <input type="text" id="newPassword" name="password" required minlength="8" value="" class="form-input" style="flex: 1; font-family: var(--font-mono);" />
                                <button type="button" class="btn btn-secondary" onclick="generateRandomPass()">Generate password</button>
                            </div>
                        </td>
                    </tr>
                    <tr>
                        <th scope="row"><label for="newRole">Role</label></th>
                        <td>
                            <select id="newRole" name="role" class="form-select regular-text">
                                <option value="Administrator">Administrator</option>
                                <option value="Editor">Editor</option>
                                <option value="Author">Author</option>
                                <option value="Contributor">Contributor</option>
                                <option value="Subscriber" selected>Subscriber</option>
                            </select>
                        </td>
                    </tr>
                </tbody>
            </table>

            <p class="submit" style="margin-top: 12px;">
                <button type="submit" class="btn btn-primary" id="btnCreateUser">Add New User</button>
            </p>
        </form>

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
                let url = document.getElementById('newUrl').value.trim();
                if (url && !/^https?:\/\//i.test(url) && !url.startsWith('/')) {
                    url = 'https://' + url;
                }
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

    render_admin_shell("user-new", "Add New User", &body, settings)
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
            "Daylight & Ink",
            ["#1d2327", "#f0f0f1", "#b8431b", "#2271b1"],
        ),
        (
            "linen",
            "Warm Linen",
            ["#e5e0d5", "#f5f2eb", "#b04a25", "#226096"],
        ),
        (
            "studio",
            "Studio Slate",
            ["#1e1f26", "#f1f2f6", "#3858e9", "#7b86ff"],
        ),
        (
            "nordic",
            "Nordic Mist",
            ["#1e2d3b", "#f0f4f7", "#2b6cb0", "#66b3e6"],
        ),
        (
            "espresso",
            "Roast & Oat",
            ["#362f2b", "#f3efe9", "#9e5a34", "#286090"],
        ),
        (
            "sage",
            "Botanical Sage",
            ["#233029", "#f1f4f1", "#3b7052", "#25637c"],
        ),
        (
            "dusk",
            "Velvet Dusk",
            ["#282433", "#f3f1f6", "#7b4685", "#3558a8"],
        ),
        (
            "harbor",
            "Harbor Fog",
            ["#26353a", "#f0f4f4", "#356e6b", "#226382"],
        ),
        (
            "terracotta",
            "Sunlit Clay",
            ["#3b2521", "#f6f0ea", "#b84a32", "#245e9c"],
        ),
    ];
    let active_color =
        crate::adapters::database::users::canonical_daypress_theme(&user.admin_color);
    let mut color_scheme_cards = String::new();
    for (id, label, swatches) in schemes {
        let checked = if id.eq_ignore_ascii_case(active_color) {
            "checked"
        } else {
            ""
        };
        let border_style = if !checked.is_empty() {
            "border-color: var(--oxide); background: var(--oxide-wash);"
        } else {
            "border-color: var(--rule);"
        };
        color_scheme_cards.push_str(&format!(
            r#"<label class="admin-color-option" style="display:flex;flex-direction:column;gap:6px;padding:8px 10px;border:1.5px solid var(--rule);border-radius:6px;cursor:pointer;transition:all 0.15s;{border_style}">
                <div style="display:flex;align-items:center;gap:6px;font-size:12px;font-weight:600;color:var(--ink);">
                    <input type="radio" name="admin_color" value="{id}" {checked} onchange="previewAdminColorScheme('{id}')" />
                    <span>{label}</span>
                </div>
                <div style="display:grid;grid-template-columns:repeat(4,1fr);height:14px;border-radius:3px;overflow:hidden;border:1px solid rgba(0,0,0,0.08);">
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
    let raw_avatar = user.avatar_url.trim();
    let is_auto_gravatar = raw_avatar.is_empty()
        || (raw_avatar.starts_with("https://www.gravatar.com/avatar/")
            && raw_avatar.contains("d=identicon"));
    let custom_avatar_input = if is_auto_gravatar { "" } else { raw_avatar };
    let resolved_avatar = if is_auto_gravatar {
        gravatar_default.clone()
    } else {
        raw_avatar.to_string()
    };

    let back_to_users_btn = if !is_self {
        r#"<a href="/d-admin/users" class="btn btn-secondary">&larr; Back to Users</a>"#
    } else {
        ""
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">{page_heading}</h1>
                <p class="page-subtitle">Personal options, public author identity, security credentials, active sessions, and REST Application Passwords</p>
            </div>
            <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                {back_to_users_btn}
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
                            <h2 id="headerDisplayName" style="font-size: 20px; font-weight: 700; color: var(--ink); margin: 0;">{display_name}</h2>
                            <span id="headerRoleBadge" class="badge badge-published">{role}</span>
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
                                <input type="text" id="url" class="form-input" value="{url}" placeholder="https://yourwebsite.com" />
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
                                <div style="display: flex; align-items: flex-start; gap: 16px;">
                                    <img id="avatarPreviewImg" src="{resolved_avatar}" alt="Profile Picture" style="width: 68px; height: 68px; border-radius: 50%; object-fit: cover; border: 1.5px solid var(--rule); flex-shrink: 0;" />
                                    <div style="flex: 1;">
                                        <input type="file" id="avatarFileInput" accept="image/*" style="display: none;" onchange="handleAvatarFileUpload(event)" />
                                        <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center; margin-bottom: 8px;">
                                            <button type="button" id="btnUploadAvatar" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;" onclick="document.getElementById('avatarFileInput').click()">Upload Image</button>
                                            <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="openAvatarMediaModal()">Choose from Media Library</button>
                                            <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="resetAvatarToGravatar()">Use Default Gravatar</button>
                                        </div>
                                        <input type="text" id="avatar_url" class="form-input" value="{custom_avatar_input}" placeholder="Or paste an image / /uploads/... URL (leave blank for automatic Gravatar)" oninput="updateAvatarPreview()" />
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

        <!-- Media Library Picker Modal for Profile Picture -->
        <div id="avatarMediaModal" onclick="if(event.target === this) closeAvatarMediaModal();" style="display: none; position: fixed; inset: 0; background: rgba(22,23,26,0.55); z-index: 9999; align-items: center; justify-content: center; padding: 24px;">
            <div style="background: var(--paper-raised); color: var(--ink); border: 1px solid var(--rule-strong); border-radius: 8px; width: 100%; max-width: 760px; max-height: 82vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 20px 50px rgba(0,0,0,0.28);">
                <div style="padding: 16px 20px; border-bottom: 1px solid var(--rule); display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
                    <div>
                        <h3 style="margin: 0; font-size: 16px; font-weight: 600; font-family: var(--font-sans);">Select Profile Picture</h3>
                        <p style="margin: 2px 0 0 0; font-size: 12px; color: var(--ink-muted);">Choose an existing image from the Media Library or upload a new photo</p>
                    </div>
                    <div style="display: flex; gap: 8px; align-items: center;">
                        <input type="search" id="avatarMediaSearch" class="form-input" placeholder="Search media..." oninput="renderAvatarMediaGrid()" style="width: 190px; padding: 5px 10px; font-size: 12.5px;" />
                        <button type="button" class="btn btn-primary" style="font-size: 12px; padding: 6px 12px;" onclick="document.getElementById('avatarFileInput').click()">+ Upload New</button>
                        <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 6px 10px;" onclick="closeAvatarMediaModal()">Close</button>
                    </div>
                </div>
                <div id="avatarMediaGrid" style="padding: 18px 20px; overflow-y: auto; display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 12px; min-height: 220px;">
                    <div style="grid-column: 1 / -1; text-align: center; color: var(--ink-muted); padding: 40px 0;">Loading media library...</div>
                </div>
            </div>
        </div>

        <script>
            const profileUserId = {id};
            const profileSaveEndpoint = '{save_endpoint}';
            const defaultGravatarUrl = '{gravatar_default}';

            function restHeaders(withJson = false) {{
                const h = {{
                    'X-WP-Nonce': (window.wpApiSettings && window.wpApiSettings.nonce) || ''
                }};
                if (withJson) h['Content-Type'] = 'application/json';
                return h;
            }}

            function previewAdminColorScheme(scheme) {{
                document.documentElement.setAttribute('data-admin-color', scheme);
                document.body.setAttribute('data-admin-color', scheme);
                document.querySelectorAll('.admin-color-option').forEach(lbl => {{
                    const radio = lbl.querySelector('input[name="admin_color"]');
                    if (radio && radio.value === scheme) {{
                        lbl.style.borderColor = 'var(--oxide)';
                        lbl.style.background = 'var(--oxide-wash)';
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
                if (first && last) {{
                    addUnique(first + ' ' + last);
                    addUnique(last + ' ' + first);
                }}
                addUnique(first);
                addUnique(last);
                addUnique(nick);
                addUnique(username);
                addUnique(currentVal);

                const preferred = candidates.includes(currentVal) ? currentVal : (candidates[0] || username);
                sel.innerHTML = candidates.map(c => {{
                    const esc = c.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');
                    const selected = c === preferred ? ' selected' : '';
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
                persistAvatarUrl('');
                showToast('✓ Reverted to automatic Gravatar');
            }}

            async function persistAvatarUrl(avatarUrl) {{
                try {{
                    await fetch(profileSaveEndpoint, {{
                        method: 'POST',
                        credentials: 'same-origin',
                        headers: restHeaders(true),
                        body: JSON.stringify({{ avatar_url: avatarUrl }})
                    }});
                }} catch (_) {{}}
            }}

            async function handleAvatarFileUpload(event) {{
                const fileInput = event.target;
                if (!fileInput || !fileInput.files || !fileInput.files.length) return;
                const file = fileInput.files[0];
                const btn = document.getElementById('btnUploadAvatar');
                if (btn) {{
                    btn.disabled = true;
                    btn.textContent = 'Uploading...';
                }}
                try {{
                    const formData = new FormData();
                    formData.append('file', file);
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/avatar', {{
                        method: 'POST',
                        credentials: 'same-origin',
                        headers: restHeaders(false),
                        body: formData
                    }});
                    const data = await res.json().catch(() => ({{}}));
                    if (res.ok && data.url) {{
                        document.getElementById('avatar_url').value = data.url;
                        updateAvatarPreview();
                        closeAvatarMediaModal();
                        showToast('✓ Profile picture uploaded and saved!');
                    }} else {{
                        showToast(data.message || 'Failed to upload profile picture', true);
                    }}
                }} catch (err) {{
                    showToast('Upload error: ' + err, true);
                }} finally {{
                    fileInput.value = '';
                    if (btn) {{
                        btn.disabled = false;
                        btn.textContent = 'Upload Image';
                    }}
                }}
            }}

            let cachedAvatarMediaItems = [];

            async function openAvatarMediaModal() {{
                const modal = document.getElementById('avatarMediaModal');
                if (!modal) return;
                modal.style.display = 'flex';
                const grid = document.getElementById('avatarMediaGrid');
                if (grid) {{
                    grid.innerHTML = '<div style="grid-column: 1 / -1; text-align: center; color: var(--ink-muted); padding: 40px 0;">Loading media library...</div>';
                }}
                try {{
                    const res = await fetch('/wp-json/wp/v2/media?per_page=100', {{
                        credentials: 'same-origin',
                        headers: restHeaders(false)
                    }});
                    if (res.ok) {{
                        const items = await res.json();
                        cachedAvatarMediaItems = (Array.isArray(items) ? items : []).filter(m => {{
                            const mime = (m.mime_type || '').toLowerCase();
                            const url = (m.source_url || '').toLowerCase();
                            return mime.startsWith('image/') || /\.(png|jpe?g|webp|gif|svg|avif)$/i.test(url);
                        }});
                        renderAvatarMediaGrid();
                    }} else {{
                        if (grid) grid.innerHTML = '<div style="grid-column: 1 / -1; text-align: center; color: var(--oxide); padding: 40px 0;">Could not load media library.</div>';
                    }}
                }} catch (err) {{
                    if (grid) grid.innerHTML = '<div style="grid-column: 1 / -1; text-align: center; color: var(--oxide); padding: 40px 0;">Network error loading media library.</div>';
                }}
            }}

            function closeAvatarMediaModal() {{
                const modal = document.getElementById('avatarMediaModal');
                if (modal) modal.style.display = 'none';
            }}

            function renderAvatarMediaGrid() {{
                const grid = document.getElementById('avatarMediaGrid');
                if (!grid) return;
                const searchInput = document.getElementById('avatarMediaSearch');
                const q = (searchInput ? searchInput.value : '').trim().toLowerCase();
                const currentUrl = document.getElementById('avatar_url').value.trim();

                const filtered = cachedAvatarMediaItems.filter(m => {{
                    if (!q) return true;
                    const title = ((m.title && m.title.rendered) ? m.title.rendered : (m.title || '')).toLowerCase();
                    const url = (m.source_url || '').toLowerCase();
                    return title.includes(q) || url.includes(q);
                }});

                if (!filtered.length) {{
                    grid.innerHTML = '<div style="grid-column: 1 / -1; text-align: center; color: var(--ink-muted); padding: 40px 0;">No matching images in the Media Library. Click <strong>+ Upload New</strong> above to upload a photo.</div>';
                    return;
                }}

                grid.innerHTML = filtered.map(m => {{
                    const url = m.source_url || '';
                    const rawTitle = (m.title && m.title.rendered) ? m.title.rendered : (m.slug || 'Image');
                    const escUrl = url.replace(/"/g, '&quot;').replace(/'/g, '&#39;');
                    const escTitle = String(rawTitle).replace(/<[^>]*>/g, '').replace(/&/g, '&amp;').replace(/</g, '&lt;');
                    const isSelected = currentUrl === url;
                    const border = isSelected ? '2px solid var(--oxide)' : '1px solid var(--rule)';
                    return '<button type="button" onclick="selectAvatarFromMedia(\'' + escUrl + '\')" style="display:flex;flex-direction:column;gap:6px;padding:6px;background:var(--paper);border:' + border + ';border-radius:6px;cursor:pointer;text-align:left;transition:transform 0.12s, border-color 0.12s;">'
                        + '<div style="width:100%;aspect-ratio:1/1;border-radius:4px;overflow:hidden;background:var(--paper-sunk);display:flex;align-items:center;justify-content:center;">'
                        + '<img src="' + escUrl + '" alt="' + escTitle + '" loading="lazy" style="width:100%;height:100%;object-fit:cover;" />'
                        + '</div>'
                        + '<span style="font-size:11px;color:var(--ink);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;width:100%;">' + escTitle + '</span>'
                        + '</button>';
                }}).join('');
            }}

            function selectAvatarFromMedia(url) {{
                document.getElementById('avatar_url').value = url;
                updateAvatarPreview();
                persistAvatarUrl(url);
                closeAvatarMediaModal();
                showToast('✓ Profile picture selected from Media Library!');
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
                        credentials: 'same-origin',
                        headers: restHeaders(true)
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
                    const res = await fetch('/wp-json/wp/v2/users/' + profileUserId + '/application-passwords', {{
                        credentials: 'same-origin',
                        headers: restHeaders(false)
                    }});
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
                        credentials: 'same-origin',
                        headers: restHeaders(true),
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
                        method: 'DELETE',
                        credentials: 'same-origin',
                        headers: restHeaders(false)
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
                        method: 'DELETE',
                        credentials: 'same-origin',
                        headers: restHeaders(false)
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
                const username = document.getElementById('profile_username').value.trim();
                const display_name = document.getElementById('display_name').value.trim();
                const first_name = document.getElementById('first_name').value.trim();
                const last_name = document.getElementById('last_name').value.trim();
                const nickname = document.getElementById('nickname').value.trim() || username;
                const email = document.getElementById('email').value.trim();
                let url = document.getElementById('url').value.trim();
                if (url && !/^https?:\/\//i.test(url) && !url.startsWith('/')) {{
                    url = 'https://' + url;
                    document.getElementById('url').value = url;
                }}
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
                    display_name: display_name || nickname || username,
                    first_name,
                    last_name,
                    nickname,
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
                        credentials: 'same-origin',
                        headers: restHeaders(true),
                        body: JSON.stringify(payload)
                    }});

                    if (res.ok) {{
                        const saved = await res.json().catch(() => null);
                        if (saved) {{
                            const hName = document.getElementById('headerDisplayName');
                            if (hName && saved.display_name) hName.textContent = saved.display_name;
                            const hRole = document.getElementById('headerRoleBadge');
                            if (hRole && saved.role) hRole.textContent = saved.role;
                        }}
                        showToast('✓ Profile updated and persisted in SQLite!');
                        setTimeout(() => window.location.reload(), 450);
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
                    const res = await fetch('/wp-json/daypress/v1/users/' + profileUserId + '/2fa/status', {{
                        credentials: 'same-origin',
                        headers: restHeaders(false)
                    }});
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
                        credentials: 'same-origin',
                        headers: restHeaders(true)
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
                        credentials: 'same-origin',
                        headers: restHeaders(true),
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
                        credentials: 'same-origin',
                        headers: restHeaders(true)
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
                        credentials: 'same-origin',
                        headers: restHeaders(true)
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
                const activeRadio = document.querySelector('input[name="admin_color"]:checked');
                if (activeRadio) previewAdminColorScheme(activeRadio.value);
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
        back_to_users_btn = back_to_users_btn,
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
        custom_avatar_input = escape_html(custom_avatar_input),
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

    let active_tab = if is_self { "profile" } else { "users" };
    let title_label = if is_self { "Profile" } else { "Edit User" };
    render_admin_shell(active_tab, title_label, &body, settings)
}
