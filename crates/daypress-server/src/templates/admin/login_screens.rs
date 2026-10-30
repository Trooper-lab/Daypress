//! Login screens sharing one card layout: `/login`, `/login/lost-password` and
//! `/login/reset-password` (see [`crate::password_reset`]).

use crate::models::DaypressSettingsResponse;
use crate::password_reset::{LOST_PASSWORD_PATH, MIN_PASSWORD_LENGTH, RESET_PASSWORD_PATH};
use crate::templates::admin::layout::{DESK_FONT_FACES, DESK_FOUNDATION};
use crate::templates::utils::escape_html;

const LOGIN_STYLES: &str = r#"
		body {
			min-height: 100vh;
			display: flex;
			flex-direction: column;
			align-items: center;
			justify-content: center;
			padding: 24px;
		}
		.login-box {
			width: 100%;
			max-width: 380px;
		}
		.login-logo {
			display: flex;
			justify-content: center;
			margin-bottom: 28px;
		}
		.login-logo img {
			height: 40px;
			width: auto;
			display: block;
		}
		.login-card {
			border-top: 1px solid var(--rule-strong);
			border-bottom: 1px solid var(--rule);
			padding: 24px 0 20px;
		}
		.login-intro {
			font-size: 15px;
			line-height: 1.55;
			color: var(--ink-muted);
			margin: 0 0 18px;
		}
		.login-notice {
			padding: 10px 12px;
			margin-bottom: 18px;
			font-size: 14px;
			line-height: 1.5;
			border-radius: var(--radius-sm);
			border: 1px solid;
		}
		.login-error {
			background: var(--oxide-wash);
			border-color: var(--oxide);
			color: var(--ink);
		}
		.login-error strong {
			color: var(--oxide);
		}
		.notice-info {
			background: var(--moss-wash);
			border-color: var(--moss);
			color: var(--ink);
		}
		label {
			display: block;
			font: 600 14px/20px var(--font-sans);
			margin-bottom: 6px;
		}
		input[type="text"], input[type="password"] {
			width: 100%;
			padding: 10px 12px;
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-sm);
			background: var(--paper-raised);
			color: var(--ink);
			font: 400 15px/1.4 var(--font-sans);
			margin-bottom: 16px;
		}
		input[type="checkbox"] {
			accent-color: var(--ink);
		}
		.login-remember {
			font-weight: 400;
			margin-bottom: 0;
			display: flex;
			align-items: center;
			gap: 8px;
		}
		.login-actions {
			display: flex;
			justify-content: flex-end;
			margin-top: 8px;
		}
		.btn-login {
			display: inline-block;
			background: var(--oxide);
			color: var(--on-oxide);
			border: 1px solid var(--oxide);
			border-radius: var(--radius-md);
			padding: 10px 20px;
			font: 600 14px/20px var(--font-sans);
			text-decoration: none;
			cursor: pointer;
			transition: background-color 0.12s, border-color 0.12s, color 0.12s;
		}
		.btn-login:hover {
			background: var(--ink);
			border-color: var(--ink);
			color: var(--paper);
		}
		.login-links {
			margin-top: 20px;
			text-align: center;
			font-size: 14px;
		}
		.login-links p + p {
			margin-top: 6px;
		}
		.login-links a {
			color: var(--ink);
			text-decoration: underline;
			text-decoration-color: var(--oxide);
			text-decoration-thickness: 2px;
			text-underline-offset: 4px;
		}
		.login-links a:hover {
			color: var(--oxide);
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
    settings: &DaypressSettingsResponse,
    page_title: &str,
    card_html: &str,
    links_html: &str,
) -> String {
    let site_name = escape_html(if settings.title.trim().is_empty() {
        "Daypress"
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
	<link rel="icon" type="image/svg+xml" href="/static/favicon.svg" />
	<style>{DESK_FONT_FACES}{DESK_FOUNDATION}{LOGIN_STYLES}</style>
</head>
<body>
<div class="login-box">
	<a class="login-logo" href="/" aria-label="Daypress">
		<img src="/static/daypress-logo.svg" alt="daypress." />
	</a>
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
pub fn render_login_view(
    settings: &DaypressSettingsResponse,
    error_message: Option<&str>,
    loggedout: bool,
    redirect_to: Option<&str>,
) -> String {
    let notice = match (error_message, loggedout) {
        (Some(err), _) => LoginNotice::Error(err),
        (None, true) => LoginNotice::Info("You are now logged out."),
        (None, false) => LoginNotice::None,
    };
    render_login_view_with_notice(settings, notice, redirect_to)
}

/// The login form with any notice (for example "Your password has been reset").
pub fn render_login_view_with_notice(
    settings: &DaypressSettingsResponse,
    notice: LoginNotice<'_>,
    redirect_to: Option<&str>,
) -> String {
    let target = escape_html(&crate::auth::sanitize_redirect_target(redirect_to));
    let card = format!(
        r#"{notice}
		<form method="post" action="/login">
			<input type="hidden" name="redirect_to" value="{target}" />
			<label for="user_login">Username or email address</label>
			<input type="text" name="log" id="user_login" required autofocus autocomplete="username" />
			<label for="user_pass">Password</label>
			<input type="password" name="pwd" id="user_pass" required autocomplete="current-password" />
			<div style="display: flex; justify-content: space-between; align-items: center; margin-top: 8px;">
				<label class="login-remember">
					<input type="checkbox" name="rememberme" value="forever" /> Remember me
				</label>
				<button type="submit" class="btn-login">Log in</button>
			</div>
		</form>"#,
        notice = notice.to_html(),
    );
    let links = format!(r#"<p><a href="{LOST_PASSWORD_PATH}">Lost your password?</a></p>"#);
    render_login_shell(settings, "Log In", &card, &links)
}

/// `/login/lost-password`: asks for a username or email address.
pub fn render_lost_password_view(
    settings: &DaypressSettingsResponse,
    notice: LoginNotice<'_>,
) -> String {
    let card = format!(
        r#"{notice}
		<p class="login-intro">Enter your username or email address. If it matches an account, we will email that account a link to choose a new password.</p>
		<form method="post" action="{LOST_PASSWORD_PATH}">
			<label for="user_login">Username or email address</label>
			<input type="text" name="user_login" id="user_login" required autofocus autocomplete="username" />
			<div class="login-actions">
				<button type="submit" class="btn-login">Get new password</button>
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
    settings: &DaypressSettingsResponse,
    token: &str,
    notice: LoginNotice<'_>,
) -> String {
    let card = format!(
        r#"{notice}
		<p class="login-intro">Choose a new password of at least {MIN_PASSWORD_LENGTH} characters.</p>
		<form method="post" action="{RESET_PASSWORD_PATH}">
			<input type="hidden" name="token" value="{token}" />
			<label for="pass1">New password</label>
			<input type="password" name="pass1" id="pass1" required minlength="{MIN_PASSWORD_LENGTH}" autofocus autocomplete="new-password" />
			<label for="pass2">Confirm new password</label>
			<input type="password" name="pass2" id="pass2" required minlength="{MIN_PASSWORD_LENGTH}" autocomplete="new-password" />
			<div class="login-actions">
				<button type="submit" class="btn-login">Save password</button>
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
pub fn render_reset_link_invalid_view(settings: &DaypressSettingsResponse) -> String {
    let card = format!(
        r#"{notice}
		<p class="login-intro">A reset link works once and expires one hour after it was sent. You can ask for a new one.</p>
		<div class="login-actions">
			<a class="btn-login" href="{LOST_PASSWORD_PATH}">Request a new link</a>
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
