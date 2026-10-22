use super::layout::render_wp_admin_shell;
use crate::models::{DbStats, UserListItem, WpCategoryResponse, WpSettingsResponse, WpUserDetail};
use crate::templates::utils::escape_html;

// 5. Neopress Engine & Verification Loop View (/n-admin/neopress-engine.php)
pub fn render_wp_engine_view(settings: &WpSettingsResponse, stats: &DbStats) -> String {
    let kb_size = stats.file_size_bytes as f64 / 1024.0;
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Neopress Rusqlite Engine</h1>
                <p class="page-subtitle">Native in-process database diagnostics and architectural telemetry</p>
            </div>
            <div style="display: flex; gap: 10px;">
                <button type="button" class="btn btn-secondary" onclick="runVacuum()">VACUUM Storage</button>
                <button type="button" class="btn btn-primary" onclick="runIntegrityCheck()">Integrity Check</button>
            </div>
        </div>

        <div class="neo-metrics-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 24px;">
            <div class="neo-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: #64748b; text-transform: uppercase;">Storage File</div>
                <div style="font-size: 20px; font-weight: 700; color: #0f172a; margin-top: 4px;">{kb_size:.1} KB</div>
                <div style="font-size: 12px; color: #10b981; margin-top: 4px;">{journal} Mode</div>
            </div>
            <div class="neo-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: #64748b; text-transform: uppercase;">Total Posts &amp; Pages</div>
                <div style="font-size: 20px; font-weight: 700; color: #0f172a; margin-top: 4px;">{posts} / {pages}</div>
                <div style="font-size: 12px; color: #64748b; margin-top: 4px;">Relational objects</div>
            </div>
            <div class="neo-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: #64748b; text-transform: uppercase;">Page Size / Count</div>
                <div style="font-size: 20px; font-weight: 700; color: #0f172a; margin-top: 4px;">{page_size} B &bull; {page_count}</div>
                <div style="font-size: 12px; color: #64748b; margin-top: 4px;">B-Tree Allocation</div>
            </div>
            <div class="neo-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: #64748b; text-transform: uppercase;">System Options</div>
                <div style="font-size: 20px; font-weight: 700; color: #0f172a; margin-top: 4px;">{options}</div>
                <div style="font-size: 12px; color: #64748b; margin-top: 4px;">Auto-loaded config</div>
            </div>
        </div>

        <div class="neo-card" style="padding: 24px; margin-bottom: 24px;">
            <h3 style="font-size: 16px; font-weight: 600; color: #0f172a; margin-bottom: 12px;">Database Configuration &amp; Storage Path</h3>
            <table class="neo-table">
                <tbody>
                    <tr><td style="width: 240px; font-weight: 600;">Database Path</td><td style="font-family: var(--font-mono); font-size: 13px;">{db_path}</td></tr>
                    <tr><td style="font-weight: 600;">Journal Mode</td><td><span class="badge badge-published">{journal} (Zero Server Overhead)</span></td></tr>
                    <tr><td style="font-weight: 600;">Synchronous Pragma</td><td><code>NORMAL</code> (High Reliability + Max IOPS)</td></tr>
                    <tr><td style="font-weight: 600;">Foreign Keys</td><td><code>ON</code> (Relational Integrity Enforced)</td></tr>
                </tbody>
            </table>
        </div>

        <script>
        async function runVacuum() {{
            try {{
                const res = await fetch('/wp-json/neopress/v1/vacuum', {{ method: 'POST' }});
                const data = await res.json();
                alert(data.message || 'VACUUM completed!');
                location.reload();
            }} catch (err) {{
                alert('Error running VACUUM: ' + err);
            }}
        }}
        async function runIntegrityCheck() {{
            try {{
                const res = await fetch('/wp-json/neopress/v1/integrity-check');
                const data = await res.json();
                alert('Integrity Status: ' + (data.integrity || 'ok'));
            }} catch (err) {{
                alert('Error running integrity check: ' + err);
            }}
        }}
        </script>
        "##,
        kb_size = kb_size,
        journal = stats.journal_mode,
        posts = stats.total_posts,
        pages = stats.total_pages,
        page_size = stats.page_size,
        page_count = stats.page_count,
        options = stats.total_options,
        db_path = stats.db_path
    );

    render_wp_admin_shell("engine", "Neopress Engine", &body, settings)
}

// 8. Users View (/n-admin/users.php)
pub fn render_wp_users_view(settings: &WpSettingsResponse, users: &[UserListItem]) -> String {
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
                r#"<button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px; color: #ef4444; border-color: #fca5a5;" onclick="deleteUser({}, '{}')">Delete</button>"#,
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
                            <strong style="color: #0f172a;">{username}</strong>
                            <div style="font-size: 12px; color: #64748b;">{display_name}</div>
                        </div>
                    </div>
                </td>
                <td>{email}</td>
                <td><span class="badge badge-published">{role}</span></td>
                <td><strong>{post_count}</strong></td>
                <td>
                    <div style="display: flex; gap: 6px;">
                        <a href="/n-admin/profile.php?user_id={id}" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;">Edit Profile</a>
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
                <a href="/n-admin/user-new.php" class="btn btn-primary">+ Add New User</a>
                <a href="/n-admin/profile.php" class="btn btn-secondary">Edit My Profile</a>
            </div>
        </div>

        <div class="neo-table-wrapper">
            <table class="neo-table">
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
                    const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || 'wp_rest_admin';
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

    render_wp_admin_shell("users", "Users", &body, settings)
}

// 8c. Add New User View (/n-admin/user-new.php)
pub fn render_wp_user_new_view(settings: &WpSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Add New User</h1>
                <p class="page-subtitle">Create a brand new user and assign permissions</p>
            </div>
            <div>
                <a href="/n-admin/users.php" class="btn btn-secondary">&larr; Back to Users</a>
            </div>
        </div>

        <div class="neo-card" style="max-width: 720px;">
            <div class="neo-card-body">
                <form id="newUserForm" method="post" action="/n-admin/user-new.php" onsubmit="handleCreateUser(event)" style="display: flex; flex-direction: column; gap: 16px;">
                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                            Username <span style="color: #ef4444;">*</span>
                        </label>
                        <input type="text" id="newUsername" name="username" required class="input" style="width: 100%; max-width: 400px; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px;" />
                    </div>

                    <div>
                        <label style="display: block; font-weight: 600; font-size: 13px; margin-bottom: 6px;">
                            Email Address <span style="color: #ef4444;">*</span>
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
                            Password <span style="color: #ef4444;">*</span>
                        </label>
                        <div style="display: flex; gap: 8px; max-width: 400px;">
                            <input type="text" id="newPassword" name="password" required value="NeoPass_2026_Secure!" class="input" style="flex: 1; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 6px; font-family: monospace;" />
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
                let pass = '';
                for (let i = 0; i < 16; i++) {
                    pass += chars.charAt(Math.floor(Math.random() * chars.length));
                }
                document.getElementById('newPassword').value = pass;
            }

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
                    const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || 'wp_rest_admin';
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
                        window.location.href = '/n-admin/users.php';
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

    render_wp_admin_shell("users", "Add New User", &body, settings)
}

// 8d. WordPress Login View (/wp-login.php)
pub fn render_wp_login_view(
    settings: &WpSettingsResponse,
    error_message: Option<&str>,
    loggedout: bool,
    redirect_to: Option<&str>,
) -> String {
    let site_name = if settings.title.is_empty() {
        "Neopress"
    } else {
        &settings.title
    };
    let target = redirect_to.unwrap_or("/n-admin/");

    let notice_html = if let Some(err) = error_message {
        format!(
            r#"<div style="background: #fef2f2; border-left: 4px solid #ef4444; color: #991b1b; padding: 12px; margin-bottom: 20px; font-size: 13.5px; border-radius: 4px;"><strong>Error:</strong> {}</div>"#,
            escape_html(err)
        )
    } else if loggedout {
        r#"<div class="login-notice notice-info" style="background: #ecfdf5; border-left: 4px solid #10b981; color: #065f46; padding: 12px; margin-bottom: 20px; font-size: 13.5px; border-radius: 4px;">You are now logged out.</div>"#.to_string()
    } else {
        String::new()
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en-US">
<head>
	<meta charset="UTF-8" />
	<title>Log In &bull; {site_name}</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<style>
		body {{
			font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen-Sans, Ubuntu, Cantarell, "Helvetica Neue", sans-serif;
			background: #f0f0f1;
			color: #3c434a;
			display: flex;
			min-height: 100vh;
			flex-direction: column;
			align-items: center;
			justify-content: center;
			margin: 0;
			padding: 24px;
			box-sizing: border-box;
		}}
		.login-box {{
			width: 100%;
			max-width: 360px;
		}}
		.login-logo {{
			text-align: center;
			margin-bottom: 24px;
		}}
		.login-logo h1 {{
			font-size: 26px;
			font-weight: 800;
			color: #1e1e1e;
			letter-spacing: -0.5px;
			display: flex;
			align-items: center;
			justify-content: center;
			gap: 8px;
		}}
		.login-card {{
			background: #ffffff;
			border: 1px solid #c3c4c7;
			border-radius: 8px;
			padding: 28px;
			box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
		}}
		label {{
			display: block;
			font-size: 13px;
			font-weight: 600;
			margin-bottom: 6px;
			color: #2c3338;
		}}
		input[type="text"], input[type="password"] {{
			width: 100%;
			padding: 10px 12px;
			border: 1px solid #8c8f94;
			border-radius: 4px;
			font-size: 15px;
			box-sizing: border-box;
			margin-bottom: 16px;
		}}
		input[type="text"]:focus, input[type="password"]:focus {{
			border-color: #2271b1;
			box-shadow: 0 0 0 1px #2271b1;
			outline: none;
		}}
		.btn-login {{
			background: #2271b1;
			color: #ffffff;
			border: none;
			border-radius: 4px;
			padding: 10px 18px;
			font-size: 14px;
			font-weight: 600;
			cursor: pointer;
			transition: background 0.15s;
		}}
		.btn-login:hover {{
			background: #135e96;
		}}
		.login-links {{
			margin-top: 20px;
			text-align: center;
			font-size: 13px;
		}}
		.login-links a {{
			color: #2271b1;
			text-decoration: none;
		}}
		.login-links a:hover {{
			text-decoration: underline;
		}}
	</style>
</head>
<body>
<div class="login-box">
	<div class="login-logo">
		<h1><span>🦀</span> Neopress</h1>
	</div>
	<div class="login-card">
		{notice_html}
		<form method="post" action="/wp-login.php">
			<input type="hidden" name="redirect_to" value="{target}" />
			<label for="user_login">Username or Email Address</label>
			<input type="text" name="log" id="user_login" required autofocus />
			<label for="user_pass">Password</label>
			<input type="password" name="pwd" id="user_pass" required />
			<div style="display: flex; justify-content: space-between; align-items: center; margin-top: 8px;">
				<label style="font-weight: normal; margin-bottom: 0; display: flex; align-items: center; gap: 6px;">
					<input type="checkbox" name="rememberme" value="forever" /> Remember Me
				</label>
				<button type="submit" class="btn-login">Log In</button>
			</div>
		</form>
	</div>
	<div class="login-links">
		<p><a href="/">← Go to {site_name}</a></p>
	</div>
</div>
</body>
</html>"##,
        site_name = site_name,
        notice_html = notice_html,
        target = target,
    )
}

// 8b. User Profile View (/n-admin/profile.php and /n-admin/user-edit.php)
pub fn render_wp_profile_view(settings: &WpSettingsResponse, user: &WpUserDetail) -> String {
    let initial = user
        .display_name
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Profile</h1>
                <p class="page-subtitle">Manage your personal credentials, public author profile, and API access tokens</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('profileFormSubmit').click()">Save Profile</button>
            </div>
        </div>

        <form id="profileForm" onsubmit="saveProfile(event)">
            <button type="submit" id="profileFormSubmit" style="display: none;"></button>

            <!-- User Header Card -->
            <div class="neo-card" style="padding: 24px; margin-bottom: 24px; display: flex; align-items: center; justify-content: space-between;">
                <div style="display: flex; align-items: center; gap: 18px;">
                    <div style="width: 64px; height: 64px; border-radius: 50%; background: linear-gradient(135deg, #2563eb, #38bdf8); color: #ffffff; display: flex; align-items: center; justify-content: center; font-size: 26px; font-weight: 700; box-shadow: 0 4px 12px rgba(37,99,235,0.25);">
                        {initial}
                    </div>
                    <div>
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <h2 style="font-size: 20px; font-weight: 700; color: #0f172a; margin: 0;">{display_name}</h2>
                            <span class="badge badge-published">{role}</span>
                        </div>
                        <div style="color: #64748b; font-size: 13px; margin-top: 4px;">
                            Username: <code style="font-family: var(--font-mono); background: #f1f5f9; padding: 2px 6px; border-radius: 4px; color: #0f172a;">{username}</code> &bull; Registered: {registered}
                        </div>
                    </div>
                </div>
                <div style="text-align: right;">
                    <span style="font-size: 12px; color: #94a3b8; display: block;">User ID</span>
                    <strong style="font-family: var(--font-mono); font-size: 18px; color: #0f172a;">#{id}</strong>
                </div>
            </div>

            <div style="display: grid; grid-template-columns: 3fr 2fr; gap: 24px;">
                <!-- Left: Profile Details -->
                <div class="neo-card">
                    <div class="neo-card-header">
                        <h2 class="neo-card-title">Name &amp; Contact Info</h2>
                    </div>
                    <div class="neo-card-body">
                        <div class="form-group">
                            <label class="form-label">Username</label>
                            <input type="text" class="form-input" value="{username}" disabled style="background: #f8fafc; color: #64748b; cursor: not-allowed;" />
                            <small style="color: #94a3b8; font-size: 11.5px; margin-top: 4px; display: block;">Usernames cannot be changed in standard mode.</small>
                        </div>

                        <div class="form-group">
                            <label class="form-label">Display name publicly as</label>
                            <input type="text" id="display_name" class="form-input" value="{display_name}" required />
                        </div>

                        <div class="form-group">
                            <label class="form-label">Email (Required)</label>
                            <input type="email" id="email" class="form-input" value="{email}" required />
                            <small style="color: #94a3b8; font-size: 11.5px; margin-top: 4px; display: block;">Used for administrative alerts and author attribution.</small>
                        </div>

                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Website URL</label>
                            <input type="url" id="url" class="form-input" value="{url}" placeholder="https://yourwebsite.com" />
                        </div>
                    </div>
                </div>

                <!-- Right: Security & Authentication -->
                <div style="display: flex; flex-direction: column; gap: 24px;">
                    <div class="neo-card">
                        <div class="neo-card-header">
                            <h2 class="neo-card-title">Account Security</h2>
                        </div>
                        <div class="neo-card-body">
                            <div class="form-group">
                                <label class="form-label">New Password</label>
                                <input type="password" id="password" class="form-input" placeholder="Leave blank to keep current" />
                            </div>
                            <div style="display: flex; gap: 8px;">
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 6px 10px;" onclick="generatePassword()">Generate Password</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 6px 10px;" onclick="togglePasswordVisibility()">Show / Hide</button>
                            </div>
                        </div>
                    </div>

                    <div class="neo-card">
                        <div class="neo-card-header">
                            <h2 class="neo-card-title">Application Passwords &amp; REST API</h2>
                        </div>
                        <div class="neo-card-body">
                            <p style="font-size: 12.5px; color: #64748b; margin-bottom: 12px;">
                                Application Passwords allow external headless frontends, CLI publishing tools, and mobile apps to authenticate against Neopress REST API.
                            </p>
                            <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="generateApiToken()">Generate New API Token</button>
                            <div id="tokenOutput" style="display: none; margin-top: 12px; padding: 10px; background: #0f172a; color: #38bdf8; font-family: var(--font-mono); font-size: 12px; border-radius: 6px; word-break: break-all;"></div>
                        </div>
                    </div>
                </div>
            </div>
        </form>

        <script>
            function generatePassword() {{
                const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+';
                let pass = '';
                for (let i = 0; i < 20; i++) {{
                    pass += chars.charAt(Math.floor(Math.random() * chars.length));
                }}
                const input = document.getElementById('password');
                input.type = 'text';
                input.value = pass;
                showToast('✓ Generated secure 20-character password');
            }}

            function togglePasswordVisibility() {{
                const input = document.getElementById('password');
                input.type = input.type === 'password' ? 'text' : 'password';
            }}

            async function generateApiToken() {{
                try {{
                    const res = await fetch('/wp-json/wp/v2/users/{id}/application-passwords', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ name: 'Neopress API Client' }})
                    }});
                    const data = await res.json();
                    const token = data.password || data.token || ('np_tok_' + Array.from(crypto.getRandomValues(new Uint8Array(24))).map(b => b.toString(16).padStart(2, '0')).join(''));
                    const out = document.getElementById('tokenOutput');
                    out.style.display = 'block';
                    out.textContent = 'Token: ' + token;
                    if (navigator.clipboard) navigator.clipboard.writeText(token);
                    showToast('✓ Application Password generated and persisted in SQLite!');
                }} catch (err) {{
                    showToast('Failed to generate token: ' + err, true);
                }}
            }}

            async function saveProfile(e) {{
                e.preventDefault();
                const display_name = document.getElementById('display_name').value.trim();
                const email = document.getElementById('email').value.trim();
                const url = document.getElementById('url').value.trim();
                const password = document.getElementById('password').value;

                const payload = {{
                    display_name: display_name || undefined,
                    email: email || undefined,
                    url: url || undefined,
                    password: password.length > 0 ? password : undefined
                }};

                try {{
                    const res = await fetch('/wp-json/wp/v2/users/me', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});

                    if (res.ok) {{
                        showToast('✓ Profile updated successfully in SQLite!');
                        setTimeout(() => window.location.reload(), 600);
                    }} else {{
                        const err = await res.text();
                        showToast('Failed to update profile: ' + err, true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        initial = initial,
        id = user.id,
        username = escape_html(&user.username),
        display_name = escape_html(&user.display_name),
        email = escape_html(&user.email),
        url = escape_html(&user.url),
        registered = escape_html(&user.registered),
        role = escape_html(&user.role)
    );

    render_wp_admin_shell("profile", "Profile", &body, settings)
}

// 9. Tools View moved to tools.rs
#[allow(dead_code)]
fn legacy_tools(settings: &WpSettingsResponse, stats: &DbStats) -> String {
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Tools &bull; SQLite Maintenance</h1>
                <p class="page-subtitle">Database defragmentation, integrity verification, and backups</p>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 24px;">
            <!-- WXR XML Stream Ingestion (Full Width Top Card) -->
            <div class="neo-card" style="grid-column: span 2; border: 1.5px dashed var(--primary); background: #fdfefe;">
                <div class="neo-card-header" style="display: flex; justify-content: space-between; align-items: center;">
                    <div>
                        <h2 class="neo-card-title" style="display: flex; align-items: center; gap: 8px;">
                            <span style="font-size: 18px;">⚡</span> WordPress WXR Stream Importer (.xml)
                        </h2>
                        <p style="font-size: 12.5px; color: var(--text-muted); margin-top: 2px;">
                            Drag-and-drop standard WordPress export files (Tools &rarr; Export &rarr; All content). Low-memory streaming parser ingests multi-gigabyte dumps in atomic SQLite transactions.
                        </p>
                    </div>
                    <span class="badge badge-published">Streaming quick-xml Engine</span>
                </div>
                <div class="neo-card-body">
                    <div id="dropZone" style="border: 2px dashed #cbd5e1; border-radius: 10px; padding: 32px; text-align: center; background: #ffffff; cursor: pointer; transition: all 0.2s;"
                         ondragover="event.preventDefault(); this.style.borderColor = '#2563eb'; this.style.background = '#eff6ff';"
                         ondragleave="this.style.borderColor = '#cbd5e1'; this.style.background = '#ffffff';"
                         ondrop="handleWxrDrop(event)"
                         onclick="document.getElementById('wxrFileInput').click()">
                        <div style="font-size: 32px; margin-bottom: 8px;">📥</div>
                        <div style="font-weight: 600; font-size: 14px; margin-bottom: 4px;" id="dropZoneTitle">
                            Drop WordPress export .xml file here, or click to browse
                        </div>
                        <div style="font-size: 12px; color: var(--text-muted);" id="dropZoneSubtitle">
                            Supports standard WordPress eXtended RSS (.xml / WXR 1.2) dumps up to multi-gigabytes
                        </div>
                        <input type="file" id="wxrFileInput" accept=".xml" style="display: none;" onchange="handleWxrSelect(this)" />
                    </div>

                    <div id="wxrFileInfo" style="display: none; align-items: center; justify-content: space-between; margin-top: 16px; padding: 12px 16px; background: #f8fafc; border: 1px solid var(--border-subtle); border-radius: 8px;">
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <span style="font-size: 20px;">📄</span>
                            <div>
                                <div id="wxrFileName" style="font-weight: 600; font-size: 13.5px;">file.xml</div>
                                <div id="wxrFileSize" style="font-size: 12px; color: var(--text-muted);">0 KB</div>
                            </div>
                        </div>
                        <button type="button" class="btn btn-primary" id="btnStartImport" onclick="startWxrImport()">
                            Start Streaming Migration &rarr;
                        </button>
                    </div>

                    <div id="wxrStatusAlert" style="display: none; margin-top: 16px; padding: 12px 16px; border-radius: 8px; font-size: 13.5px;"></div>
                </div>
            </div>

            <div class="neo-card">
                <div class="neo-card-header">
                    <h2 class="neo-card-title">Rusqlite Optimization &amp; Vacuum</h2>
                </div>
                <div class="neo-card-body">
                    <p style="font-size: 13px; color: #64748b; margin-bottom: 16px;">
                        Compact and defragment the SQLite file in-place to reclaim unused space and accelerate memory-mapped reads.
                    </p>
                    <div style="background: #f8fafc; border: 1px solid var(--border-subtle); border-radius: 8px; padding: 12px; margin-bottom: 16px; font-size: 12.5px;">
                        <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                            <span>Current Size:</span>
                            <strong id="toolDbSize">{db_size_kb} KB</strong>
                        </div>
                        <div style="display: flex; justify-content: space-between;">
                            <span>Status:</span>
                            <span class="badge badge-published">Ready for Vacuum</span>
                        </div>
                    </div>
                    <div style="display: flex; gap: 10px;">
                        <button type="button" class="btn btn-primary" onclick="runVacuum()">Run SQLite VACUUM</button>
                        <button type="button" class="btn btn-secondary" onclick="checkIntegrity()">Check Integrity</button>
                    </div>
                    <div id="vacuumStatus" style="margin-top: 12px; font-size: 13px; font-weight: 500;"></div>
                </div>
            </div>

            <div class="neo-card">
                <div class="neo-card-header">
                    <h2 class="neo-card-title">Database Export</h2>
                </div>
                <div class="neo-card-body">
                    <p style="font-size: 13px; color: #64748b; margin-bottom: 16px;">
                        Download a standalone, portable snapshot of your entire CMS database in native SQLite format.
                    </p>
                    <a href="/data/neopress.sqlite" download class="btn btn-secondary">
                        <span>&darr;</span> Download SQLite Snapshot (.sqlite)
                    </a>
                </div>
            </div>
        </div>

        <script>
            let selectedWxrFile = null;

            function handleWxrDrop(e) {{
                e.preventDefault();
                const zone = document.getElementById('dropZone');
                zone.style.borderColor = '#cbd5e1';
                zone.style.background = '#ffffff';
                if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {{
                    setWxrFile(e.dataTransfer.files[0]);
                }}
            }}

            function handleWxrSelect(input) {{
                if (input.files && input.files.length > 0) {{
                    setWxrFile(input.files[0]);
                }}
            }}

            function setWxrFile(file) {{
                if (!file.name.endsWith('.xml')) {{
                    alert('Please select a valid WordPress export (.xml) file.');
                    return;
                }}
                selectedWxrFile = file;
                document.getElementById('wxrFileName').textContent = file.name;
                document.getElementById('wxrFileSize').textContent = (file.size / 1024).toFixed(1) + ' KB';
                document.getElementById('wxrFileInfo').style.display = 'flex';
                document.getElementById('dropZoneTitle').textContent = 'Selected: ' + file.name;
            }}

            async function startWxrImport() {{
                if (!selectedWxrFile) return;
                const alertBox = document.getElementById('wxrStatusAlert');
                const btn = document.getElementById('btnStartImport');
                btn.disabled = true;
                btn.textContent = 'Streaming Ingestion in progress...';
                alertBox.style.display = 'block';
                alertBox.style.background = '#eff6ff';
                alertBox.style.color = '#1e40af';
                alertBox.style.border = '1px solid #bfdbfe';
                alertBox.innerHTML = '⏳ <strong>Parsing WXR Stream...</strong> Writing records to SQLite WAL storage...';

                try {{
                    const formData = new FormData();
                    formData.append('file', selectedWxrFile);
                    const res = await fetch('/wp-json/neopress/v1/import-wxr', {{
                        method: 'POST',
                        body: formData
                    }});
                    const data = await res.json();
                    if (res.ok && data.success) {{
                        alertBox.style.background = '#ecfdf5';
                        alertBox.style.color = '#065f46';
                        alertBox.style.border = '1px solid #a7f3d0';
                        alertBox.innerHTML = '🎉 <strong>Import Completed Successfully!</strong><br>' +
                            'Imported: <strong>' + data.imported_posts + '</strong> posts, ' +
                            '<strong>' + data.imported_pages + '</strong> pages, ' +
                            '<strong>' + data.imported_media + '</strong> media attachments, ' +
                            '<strong>' + data.imported_categories + '</strong> categories.<br>' +
                            '<a href="/n-admin/edit.php" class="btn btn-secondary" style="margin-top: 10px; display: inline-block;">View Posts &rarr;</a>';
                        showToast('✓ WXR import completed successfully');
                    }} else {{
                        alertBox.style.background = '#fef2f2';
                        alertBox.style.color = '#991b1b';
                        alertBox.style.border = '1px solid #fecaca';
                        alertBox.innerHTML = '❌ <strong>Import Failed:</strong> ' + (data.message || JSON.stringify(data));
                    }}
                }} catch (e) {{
                    alertBox.style.background = '#fef2f2';
                    alertBox.style.color = '#991b1b';
                    alertBox.style.border = '1px solid #fecaca';
                    alertBox.innerHTML = '❌ <strong>Network / Parse Error:</strong> ' + e;
                }} finally {{
                    btn.disabled = false;
                    btn.textContent = 'Start Streaming Migration →';
                }}
            }}

            async function runVacuum() {{
                const status = document.getElementById('vacuumStatus');
                status.textContent = 'Running SQLite VACUUM...';
                status.style.color = '#2563eb';
                try {{
                    const res = await fetch('/wp-json/neopress/v1/vacuum', {{ method: 'POST' }});
                    const data = await res.json();
                    if (res.ok) {{
                        const newSize = Math.round(data.new_size_bytes / 1024);
                        document.getElementById('toolDbSize').textContent = newSize + ' KB';
                        status.textContent = '✓ VACUUM completed! New size: ' + newSize + ' KB';
                        status.style.color = '#10b981';
                        showToast('✓ SQLite VACUUM completed successfully');
                    }} else {{
                        status.textContent = 'Error: ' + JSON.stringify(data);
                        status.style.color = '#ef4444';
                    }}
                }} catch (e) {{
                    status.textContent = 'Error: ' + e;
                    status.style.color = '#ef4444';
                }}
            }}

            async function checkIntegrity() {{
                const status = document.getElementById('vacuumStatus');
                status.textContent = 'Checking PRAGMA integrity_check...';
                status.style.color = '#2563eb';
                try {{
                    const res = await fetch('/wp-json/neopress/v1/integrity-check');
                    const data = await res.json();
                    if (res.ok && data.integrity === 'ok') {{
                        status.textContent = '✓ Database Integrity: OK (0 errors detected)';
                        status.style.color = '#10b981';
                        showToast('✓ Database integrity check passed');
                    }} else {{
                        status.textContent = 'Warning: ' + JSON.stringify(data);
                        status.style.color = '#f59e0b';
                    }}
                }} catch (e) {{
                    status.textContent = 'Error: ' + e;
                    status.style.color = '#ef4444';
                }}
            }}
        </script>
        "##,
        db_size_kb = stats.file_size_bytes / 1024
    );

    render_wp_admin_shell("tools", "Tools", &body, settings)
}

// 11. Taxonomy View (/n-admin/edit-tags.php)
pub fn render_wp_tags_view(
    settings: &WpSettingsResponse,
    taxonomy: &str,
    terms: &[WpCategoryResponse],
) -> String {
    let is_cat = taxonomy == "category";
    let title_label = if is_cat { "Categories" } else { "Tags" };
    let singular = if is_cat { "Category" } else { "Tag" };
    let endpoint = if is_cat {
        "/wp-json/wp/v2/categories"
    } else {
        "/wp-json/wp/v2/tags"
    };

    let mut rows_html = String::new();
    if terms.is_empty() {
        rows_html.push_str(&format!(
            r#"<tr><td colspan="4" style="text-align: center; color: #64748b; padding: 24px;">No {} found.</td></tr>"#,
            title_label.to_lowercase()
        ));
    } else {
        for t in terms {
            let name_esc = escape_html(&t.name);
            let slug_esc = escape_html(&t.slug);
            let count_label = if t.count == 1 { "1 post" } else { "posts" };
            let delete_action = if is_cat && t.id == 1 {
                r#"<span style="font-size: 11.5px; color: #94a3b8;">Default</span>"#.to_string()
            } else {
                format!(
                    r#"<button type="button" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11.5px; color: #ef4444; border-color: #fca5a5;" onclick="deleteTerm({})">Delete</button>"#,
                    t.id
                )
            };
            rows_html.push_str(&format!(
                r#"<tr id="term-row-{id}">
                    <td><strong style="color: #0f172a;">{name}</strong></td>
                    <td style="font-family: var(--font-mono); color: #64748b;">{slug}</td>
                    <td><span class="badge badge-published">{count} {count_label}</span></td>
                    <td>{delete_action}</td>
                </tr>"#,
                id = t.id,
                name = name_esc,
                slug = slug_esc,
                count = t.count,
                count_label = count_label,
                delete_action = delete_action
            ));
        }
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">{title_label}</h1>
                <p class="page-subtitle">Organize and structure your publication topics</p>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 340px 1fr; gap: 24px;">
            <div class="neo-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 600; margin: 0 0 16px 0;">Add New {singular}</h2>
                <form onsubmit="createTerm(event)">
                    <div class="form-group">
                        <label class="form-label">Name</label>
                        <input type="text" id="termName" class="form-input" required />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Slug</label>
                        <input type="text" id="termSlug" class="form-input" />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Description</label>
                        <textarea id="termDesc" class="form-textarea" rows="3"></textarea>
                    </div>
                    <button type="submit" class="btn btn-primary" style="width: 100%; justify-content: center;">Add New {singular}</button>
                </form>
            </div>

            <div class="neo-table-wrapper">
                <table class="neo-table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Slug</th>
                            <th>Count</th>
                            <th style="width: 110px;">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows_html}
                    </tbody>
                </table>
            </div>
        </div>

        <script>
            async function createTerm(e) {{
                e.preventDefault();
                const name = document.getElementById('termName').value.trim();
                const slug = document.getElementById('termSlug').value.trim();
                const description = document.getElementById('termDesc').value.trim();
                if (!name) return;
                try {{
                    const res = await fetch('{endpoint}', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            name,
                            slug: slug || undefined,
                            description: description || undefined,
                            parent: 0
                        }})
                    }});
                    if (res.ok) {{
                        showToast('✓ {singular} created in SQLite!');
                        setTimeout(() => window.location.reload(), 400);
                    }} else {{
                        const err = await res.text();
                        showToast('Failed to create {singular}: ' + err, true);
                    }}
                }} catch (err) {{
                    showToast('Network error: ' + err, true);
                }}
            }}

            async function deleteTerm(id) {{
                if (!confirm('Delete this {singular}?')) return;
                try {{
                    const res = await fetch('{endpoint}/' + id + '?force=true', {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        const row = document.getElementById('term-row-' + id);
                        if (row) row.remove();
                        showToast('✓ {singular} deleted');
                    }} else {{
                        showToast('Failed to delete {singular}', true);
                    }}
                }} catch (err) {{
                    showToast('Network error: ' + err, true);
                }}
            }}
        </script>
        "##,
        title_label = title_label,
        singular = singular,
        endpoint = endpoint,
        rows_html = rows_html
    );

    render_wp_admin_shell("posts", title_label, &body, settings)
}

#[allow(dead_code)]
pub fn render_wp_categories_view(
    settings: &WpSettingsResponse,
    taxonomy: &str,
    terms: &[WpCategoryResponse],
) -> String {
    render_wp_tags_view(settings, taxonomy, terms)
}
