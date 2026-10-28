//! Login screens sharing one card layout: `/login`, `/login/lost-password` and
//! `/login/reset-password` (see [`crate::password_reset`]).

use crate::models::WpSettingsResponse;
use crate::password_reset::{LOST_PASSWORD_PATH, MIN_PASSWORD_LENGTH, RESET_PASSWORD_PATH};
use crate::templates::utils::escape_html;

const LOGIN_STYLES: &str = r#"
		body {
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
		}
		.login-box {
			width: 100%;
			max-width: 360px;
		}
		.login-logo {
			text-align: center;
			margin-bottom: 24px;
		}
		.login-logo h1 {
			font-size: 26px;
			font-weight: 800;
			color: #1e1e1e;
			letter-spacing: -0.5px;
			display: flex;
			align-items: center;
			justify-content: center;
			gap: 8px;
		}
		.login-card {
			background: #ffffff;
			border: 1px solid #c3c4c7;
			border-radius: 8px;
			padding: 28px;
			box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
		}
		.login-intro {
			font-size: 13.5px;
			line-height: 1.5;
			margin: 0 0 18px;
		}
		.login-notice {
			padding: 12px;
			margin-bottom: 20px;
			font-size: 13.5px;
			line-height: 1.5;
			border-radius: 4px;
		}
		.login-error {
			background: #fef2f2;
			border-left: 4px solid #ef4444;
			color: #991b1b;
		}
		.notice-info {
			background: #ecfdf5;
			border-left: 4px solid #10b981;
			color: #065f46;
		}
		label {
			display: block;
			font-size: 13px;
			font-weight: 600;
			margin-bottom: 6px;
			color: #2c3338;
		}
		input[type="text"], input[type="password"] {
			width: 100%;
			padding: 10px 12px;
			border: 1px solid #8c8f94;
			border-radius: 4px;
			font-size: 15px;
			box-sizing: border-box;
			margin-bottom: 16px;
		}
		input[type="text"]:focus, input[type="password"]:focus {
			border-color: #2271b1;
			box-shadow: 0 0 0 1px #2271b1;
			outline: none;
		}
		.login-actions {
			display: flex;
			justify-content: flex-end;
			margin-top: 8px;
		}
		.btn-login {
			display: inline-block;
			background: #2271b1;
			color: #ffffff;
			border: none;
			border-radius: 4px;
			padding: 10px 18px;
			font-size: 14px;
			font-weight: 600;
			text-decoration: none;
			cursor: pointer;
			transition: background 0.15s;
		}
		.btn-login:hover {
			background: #135e96;
		}
		.login-links {
			margin-top: 20px;
			text-align: center;
			font-size: 13px;
		}
		.login-links a {
			color: #2271b1;
			text-decoration: none;
		}
		.login-links a:hover {
			text-decoration: underline;
		}
"#;

/// A message above a login form. Text is escaped when rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginNotice<'a> {
    None,
    Error(&'a str),
    Info(&'a str),
}

impl LoginNotice<'_> {
    fn to_html(self) -> String {
        match self {
            LoginNotice::None => String::new(),
            LoginNotice::Error(text) => format!(
                r#"<div class="login-notice login-error" role="alert"><strong>Error:</strong> {}</div>"#,
                escape_html(text)
            ),
            LoginNotice::Info(text) => format!(
                r#"<div class="login-notice notice-info" role="status">{}</div>"#,
                escape_html(text)
            ),
        }
    }
}

/// Page skeleton shared by every login screen. `card_html` and `links_html` are trusted markup.
fn render_login_shell(
    settings: &WpSettingsResponse,
    page_title: &str,
    card_html: &str,
    links_html: &str,
) -> String {
    let site_name = escape_html(if settings.title.trim().is_empty() {
        "Neopress"
    } else {
        &settings.title
    });
    let page_title = escape_html(page_title);
    format!(
        r##"<!DOCTYPE html>
<html lang="en-US">
<head>
	<meta charset="UTF-8" />
	<title>{page_title} &bull; {site_name}</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<meta name="referrer" content="no-referrer">
	<style>{LOGIN_STYLES}</style>
</head>
<body>
<div class="login-box">
	<div class="login-logo">
		<h1><span>🦀</span> Neopress</h1>
	</div>
	<div class="login-card">
		{card_html}
	</div>
	<div class="login-links">
		{links_html}
		<p><a href="/">← Go to {site_name}</a></p>
	</div>
</div>
</body>
</html>"##
    )
}

// 8d. Login View (/login)
pub fn render_wp_login_view(
    settings: &WpSettingsResponse,
    error_message: Option<&str>,
    loggedout: bool,
    redirect_to: Option<&str>,
) -> String {
    let notice = match (error_message, loggedout) {
        (Some(err), _) => LoginNotice::Error(err),
        (None, true) => LoginNotice::Info("You are now logged out."),
        (None, false) => LoginNotice::None,
    };
    render_wp_login_view_with_notice(settings, notice, redirect_to)
}

/// The login form with any notice (for example "Your password has been reset").
pub fn render_wp_login_view_with_notice(
    settings: &WpSettingsResponse,
    notice: LoginNotice<'_>,
    redirect_to: Option<&str>,
) -> String {
    let target = escape_html(&crate::auth::sanitize_redirect_target(redirect_to));
    let card = format!(
        r#"{notice}
		<form method="post" action="/login">
			<input type="hidden" name="redirect_to" value="{target}" />
			<label for="user_login">Username or Email Address</label>
			<input type="text" name="log" id="user_login" required autofocus autocomplete="username" />
			<label for="user_pass">Password</label>
			<input type="password" name="pwd" id="user_pass" required autocomplete="current-password" />
			<div style="display: flex; justify-content: space-between; align-items: center; margin-top: 8px;">
				<label style="font-weight: normal; margin-bottom: 0; display: flex; align-items: center; gap: 6px;">
					<input type="checkbox" name="rememberme" value="forever" /> Remember Me
				</label>
				<button type="submit" class="btn-login">Log In</button>
			</div>
		</form>"#,
        notice = notice.to_html(),
    );
    let links = format!(r#"<p><a href="{LOST_PASSWORD_PATH}">Lost your password?</a></p>"#);
    render_login_shell(settings, "Log In", &card, &links)
}

/// `/login/lost-password`: asks for a username or email address.
pub fn render_lost_password_view(settings: &WpSettingsResponse, notice: LoginNotice<'_>) -> String {
    let card = format!(
        r#"{notice}
		<p class="login-intro">Enter your username or email address. If it matches an account, we will email that account a link to choose a new password.</p>
		<form method="post" action="{LOST_PASSWORD_PATH}">
			<label for="user_login">Username or Email Address</label>
			<input type="text" name="user_login" id="user_login" required autofocus autocomplete="username" />
			<div class="login-actions">
				<button type="submit" class="btn-login">Get New Password</button>
			</div>
		</form>"#,
        notice = notice.to_html(),
    );
    render_login_shell(
        settings,
        "Lost Password",
        &card,
        r#"<p><a href="/login">Log in</a></p>"#,
    )
}

/// `/login/reset-password`: the new-password form behind a valid reset link.
pub fn render_reset_password_view(
    settings: &WpSettingsResponse,
    token: &str,
    notice: LoginNotice<'_>,
) -> String {
    let card = format!(
        r#"{notice}
		<p class="login-intro">Choose a new password of at least {MIN_PASSWORD_LENGTH} characters.</p>
		<form method="post" action="{RESET_PASSWORD_PATH}">
			<input type="hidden" name="token" value="{token}" />
			<label for="pass1">New Password</label>
			<input type="password" name="pass1" id="pass1" required minlength="{MIN_PASSWORD_LENGTH}" autofocus autocomplete="new-password" />
			<label for="pass2">Confirm New Password</label>
			<input type="password" name="pass2" id="pass2" required minlength="{MIN_PASSWORD_LENGTH}" autocomplete="new-password" />
			<div class="login-actions">
				<button type="submit" class="btn-login">Save Password</button>
			</div>
		</form>"#,
        notice = notice.to_html(),
        token = escape_html(token),
    );
    render_login_shell(
        settings,
        "Reset Password",
        &card,
        r#"<p><a href="/login">Log in</a></p>"#,
    )
}

/// `/login/reset-password` when the link is unknown, used or expired.
pub fn render_reset_link_invalid_view(settings: &WpSettingsResponse) -> String {
    let card = format!(
        r#"{notice}
		<p class="login-intro">A reset link works once and expires one hour after it was sent. You can ask for a new one.</p>
		<div class="login-actions">
			<a class="btn-login" href="{LOST_PASSWORD_PATH}">Request a New Link</a>
		</div>"#,
        notice =
            LoginNotice::Error("This password reset link is invalid or has expired.").to_html(),
    );
    render_login_shell(
        settings,
        "Reset Password",
        &card,
        r#"<p><a href="/login">Log in</a></p>"#,
    )
}
