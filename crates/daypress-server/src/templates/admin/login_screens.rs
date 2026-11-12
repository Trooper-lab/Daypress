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

/// `/login/2fa`: Two-Factor Authentication verification step (6-digit TOTP or recovery code).
pub fn render_login_2fa_view(
    settings: &DaypressSettingsResponse,
    error_message: Option<&str>,
    redirect_to: Option<&str>,
) -> String {
    let target = escape_html(&crate::auth::sanitize_redirect_target(redirect_to));
    let notice = match error_message {
        Some(err) => LoginNotice::Error(err).to_html(),
        None => String::new(),
    };
    let card = format!(
        r#"{notice}
		<p class="login-intro">Enter the 6-digit code from your authenticator app, or one of your single-use backup recovery codes.</p>
		<form method="post" action="/login/2fa">
			<input type="hidden" name="redirect_to" value="{target}" />
			<label for="authcode">Authentication code</label>
			<input type="text" name="authcode" id="authcode" required autofocus autocomplete="one-time-code" inputmode="text" placeholder="123456 or xxxx-xxxx" style="font-family: var(--font-mono); letter-spacing: 0.08em;" />
			<div class="login-actions">
				<button type="submit" class="btn-login">Verify &amp; Log in</button>
			</div>
		</form>"#,
    );
    let links = r#"<p><a href="/login">← Use a different account</a></p>"#;
    render_login_shell(settings, "Two-Factor Authentication", &card, links)
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

/// `/setup` (and `/d-admin/install` / `/wp-admin/install.php`): First-Time Onboarding & Admin Account Creation Wizard.
pub fn render_onboarding_view(
    settings: &DaypressSettingsResponse,
    prefill: Option<&crate::adapters::database::schema::OnboardingSetupRequest>,
    error_message: Option<&str>,
) -> String {
    let site_title = prefill
        .map(|p| p.site_title.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if settings.title.trim().is_empty() {
                "Daypress"
            } else {
                settings.title.trim()
            }
        });
    let tagline = prefill
        .map(|p| p.tagline.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if settings.description.trim().is_empty() {
                "A self-hosted CMS written in Rust."
            } else {
                settings.description.trim()
            }
        });
    let username = prefill
        .map(|p| p.username.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("admin");
    let display_name = prefill
        .map(|p| p.display_name.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Admin");
    let email = prefill
        .map(|p| p.email.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("");
    let theme = prefill
        .map(|p| p.theme.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Daypress Daylight");
    let brand_color = prefill
        .map(|p| p.brand_color.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("#a8401b");
    let typography = prefill
        .map(|p| p.typography_scale.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("serif");
    let starter_mode = prefill
        .map(|p| p.starter_mode.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("starter");

    let notice_html = match error_message {
        Some(err) => LoginNotice::Error(err).to_html(),
        None => String::new(),
    };

    let sel_daylight = if theme == "Daypress Daylight" {
        "selected"
    } else {
        ""
    };
    let sel_modern = if theme == "Daypress Modern" {
        "selected"
    } else {
        ""
    };
    let sel_night = if theme == "Daypress Night" {
        "selected"
    } else {
        ""
    };

    let sel_serif = if typography == "serif" {
        "selected"
    } else {
        ""
    };
    let sel_system = if typography == "system" {
        "selected"
    } else {
        ""
    };
    let sel_mono = if typography == "mono" { "selected" } else { "" };

    let chk_starter = if starter_mode == "starter" {
        "checked"
    } else {
        ""
    };
    let chk_blank = if starter_mode == "blank" {
        "checked"
    } else {
        ""
    };
    let chk_migrate = if starter_mode == "migrate" {
        "checked"
    } else {
        ""
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en-US">
<head>
	<meta charset="UTF-8" />
	<title>Welcome to Daypress &bull; First-Time Setup</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<meta name="referrer" content="no-referrer">
	<link rel="icon" type="image/svg+xml" href="/static/favicon.svg" />
	<style>{DESK_FONT_FACES}{DESK_FOUNDATION}{LOGIN_STYLES}
		:root {{
			--orb-a: rgba(168, 64, 27, 0.14);
			--orb-b: rgba(42, 59, 208, 0.09);
			--grid-line: rgba(22, 23, 26, 0.055);
		}}
		body[data-preview-theme="Daypress Modern"] {{
			--paper: #f8fafc;
			--paper-raised: #ffffff;
			--paper-sunk: #f1f5f9;
			--ink: #0f172a;
			--ink-muted: #475569;
			--rule: #e2e8f0;
			--rule-strong: #cbd5e1;
			--oxide: #2563eb;
			--orb-a: rgba(37, 99, 235, 0.14);
			--orb-b: rgba(14, 165, 233, 0.10);
			--grid-line: rgba(15, 23, 42, 0.05);
		}}
		body[data-preview-theme="Daypress Night"] {{
			--paper: #090d16;
			--paper-raised: #111827;
			--paper-sunk: #060911;
			--ink: #f8fafc;
			--ink-muted: #94a3b8;
			--rule: #1e293b;
			--rule-strong: #334155;
			--oxide: #38bdf8;
			--on-oxide: #090d16;
			--orb-a: rgba(56, 189, 248, 0.16);
			--orb-b: rgba(99, 102, 241, 0.14);
			--grid-line: rgba(148, 163, 184, 0.065);
		}}
		body {{
			position: relative;
			overflow-x: hidden;
			transition: background-color 0.45s ease, color 0.45s ease;
		}}
		/* Subtle Claude / Codex Desktop Installer Ambient Background */
		.setup-ambient {{
			position: fixed;
			inset: 0;
			pointer-events: none;
			z-index: 0;
			overflow: hidden;
		}}
		.setup-ambient-grid {{
			position: absolute;
			inset: -10%;
			background-image:
				linear-gradient(to right, var(--grid-line) 1px, transparent 1px),
				linear-gradient(to bottom, var(--grid-line) 1px, transparent 1px);
			background-size: 56px 56px;
			mask-image: radial-gradient(ellipse 75% 65% at 50% 45%, #000 35%, transparent 100%);
			-webkit-mask-image: radial-gradient(ellipse 75% 65% at 50% 45%, #000 35%, transparent 100%);
			animation: gridFloat 26s ease-in-out infinite alternate;
		}}
		.setup-orb {{
			position: absolute;
			border-radius: 999px;
			filter: blur(68px);
			transition: background 0.6s ease;
		}}
		.setup-orb-a {{
			width: 480px;
			height: 480px;
			top: -120px;
			left: calc(50% - 420px);
			background: radial-gradient(circle, var(--orb-a) 0%, transparent 70%);
			animation: orbDriftA 18s ease-in-out infinite alternate;
		}}
		.setup-orb-b {{
			width: 520px;
			height: 520px;
			bottom: -160px;
			right: calc(50% - 440px);
			background: radial-gradient(circle, var(--orb-b) 0%, transparent 70%);
			animation: orbDriftB 22s ease-in-out infinite alternate;
		}}
		.setup-topo-svg {{
			position: absolute;
			inset: 0;
			width: 100%;
			height: 100%;
			opacity: 0.36;
		}}
		.setup-topo-path {{
			fill: none;
			stroke: var(--oxide);
			stroke-width: 1;
			stroke-dasharray: 8 14;
			opacity: 0.35;
			animation: topoDash 36s linear infinite;
		}}
		@keyframes orbDriftA {{
			0% {{ transform: translate3d(0, 0, 0) scale(1); }}
			100% {{ transform: translate3d(60px, 40px, 0) scale(1.08); }}
		}}
		@keyframes orbDriftB {{
			0% {{ transform: translate3d(0, 0, 0) scale(1); }}
			100% {{ transform: translate3d(-55px, -35px, 0) scale(1.1); }}
		}}
		@keyframes gridFloat {{
			0% {{ transform: translateY(0px); }}
			100% {{ transform: translateY(18px); }}
		}}
		@keyframes topoDash {{
			to {{ stroke-dashoffset: -440; }}
		}}
		.setup-box {{
			position: relative;
			z-index: 1;
			width: 100%;
			max-width: 660px;
			margin: 24px auto;
		}}
		.setup-header {{
			text-align: center;
			margin-bottom: 22px;
		}}
		.setup-badge {{
			display: inline-flex;
			align-items: center;
			gap: 6px;
			padding: 4px 11px;
			border-radius: 999px;
			border: 1px solid var(--rule-strong);
			background: var(--paper-raised);
			font: 600 11px/16px var(--font-mono);
			letter-spacing: 0.06em;
			text-transform: uppercase;
			color: var(--oxide);
			margin-bottom: 10px;
		}}
		.setup-header h1 {{
			font: 600 28px/1.2 var(--font-sans);
			letter-spacing: -0.02em;
			color: var(--ink);
			margin-top: 6px;
			margin-bottom: 8px;
		}}
		.setup-header p {{
			font-size: 15px;
			color: var(--ink-muted);
			max-width: 490px;
			margin: 0 auto;
		}}
		.setup-stepper {{
			display: grid;
			grid-template-columns: repeat(4, 1fr);
			gap: 8px;
			margin-bottom: 16px;
		}}
		.setup-step-pill {{
			display: flex;
			align-items: center;
			gap: 8px;
			padding: 9px 12px;
			border-radius: var(--radius-sm);
			border: 1px solid var(--rule);
			background: var(--paper-raised);
			color: var(--ink-muted);
			font: 600 12.5px/16px var(--font-sans);
			cursor: pointer;
			transition: all 0.15s ease;
			text-align: left;
		}}
		.setup-step-pill .step-num {{
			font: 600 11px/18px var(--font-mono);
			width: 20px;
			height: 20px;
			border-radius: 999px;
			display: inline-flex;
			align-items: center;
			justify-content: center;
			background: var(--paper-sunk);
			color: var(--ink-muted);
		}}
		.setup-step-pill.active {{
			border-color: var(--oxide);
			color: var(--ink);
			box-shadow: 0 2px 10px rgba(0, 0, 0, 0.04);
		}}
		.setup-step-pill.active .step-num {{
			background: var(--oxide);
			color: var(--on-oxide);
		}}
		.setup-step-pill.done .step-num {{
			background: var(--moss);
			color: #fff;
		}}
		.setup-card {{
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-md);
			padding: 28px 32px;
			box-shadow: 0 18px 48px rgba(22, 23, 26, 0.08);
			transition: background-color 0.35s ease, border-color 0.35s ease;
		}}
		.setup-pane {{
			display: none;
			animation: paneFadeIn 0.22s ease;
		}}
		.setup-pane.active {{
			display: block;
		}}
		@keyframes paneFadeIn {{
			from {{ opacity: 0; transform: translateY(4px); }}
			to {{ opacity: 1; transform: translateY(0); }}
		}}
		.setup-section-title {{
			font: 600 12px/16px var(--font-mono);
			text-transform: uppercase;
			letter-spacing: 0.08em;
			color: var(--oxide);
			margin-bottom: 6px;
		}}
		.setup-section-sub {{
			font-size: 14px;
			color: var(--ink-muted);
			margin-bottom: 18px;
		}}
		.setup-grid-2 {{
			display: grid;
			grid-template-columns: 1fr 1fr;
			gap: 16px;
		}}
		@media (max-width: 560px) {{
			.setup-grid-2, .setup-stepper, .theme-cards-grid {{ grid-template-columns: 1fr !important; }}
			.setup-card {{ padding: 20px; }}
		}}
		input[type="email"], select {{
			width: 100%;
			padding: 10px 12px;
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-sm);
			background: var(--paper);
			color: var(--ink);
			font: 400 15px/1.4 var(--font-sans);
			margin-bottom: 16px;
		}}
		input[type="text"], input[type="password"] {{
			background: var(--paper);
		}}
		.field-hint {{
			font-size: 12.5px;
			color: var(--ink-muted);
			margin-top: -10px;
			margin-bottom: 14px;
		}}
		.pwd-toolbar {{
			display: flex;
			align-items: center;
			justify-content: space-between;
			gap: 10px;
			margin-top: -6px;
			margin-bottom: 14px;
			flex-wrap: wrap;
		}}
		.btn-inline {{
			background: var(--paper);
			border: 1px solid var(--rule-strong);
			color: var(--ink);
			border-radius: var(--radius-sm);
			padding: 6px 11px;
			font: 600 12px/16px var(--font-sans);
			cursor: pointer;
		}}
		.btn-inline:hover {{
			background: var(--ink);
			color: var(--paper);
		}}
		.pwd-strength {{
			font: 500 12px/16px var(--font-mono);
			padding: 3px 8px;
			border-radius: var(--radius-sm);
			background: var(--paper-sunk);
			color: var(--ink-muted);
		}}
		.starter-options {{
			display: grid;
			grid-template-columns: 1fr;
			gap: 12px;
			margin-top: 6px;
		}}
		.starter-option {{
			display: flex;
			align-items: flex-start;
			gap: 14px;
			padding: 16px 18px;
			border: 1.5px solid var(--rule);
			border-radius: var(--radius-md);
			background: var(--paper);
			cursor: pointer;
			font-weight: 400;
			margin-bottom: 0;
			transition: border-color 0.15s ease, transform 0.12s ease, box-shadow 0.15s ease;
		}}
		.starter-option:hover {{
			border-color: var(--oxide);
			transform: translateY(-1px);
		}}
		.starter-option.selected-path {{
			border-color: var(--oxide);
			box-shadow: 0 0 0 1px var(--oxide);
		}}
		.starter-option input[type="radio"] {{
			margin-top: 4px;
			accent-color: var(--oxide);
		}}
		.starter-option strong {{
			display: flex;
			align-items: center;
			gap: 8px;
			font-size: 15px;
			color: var(--ink);
		}}
		.starter-option span {{
			display: block;
			font-size: 13.5px;
			color: var(--ink-muted);
			margin-top: 4px;
			line-height: 1.45;
		}}
		.path-tag {{
			font: 600 10.5px/14px var(--font-mono);
			padding: 2px 7px;
			border-radius: 999px;
			background: var(--paper-sunk);
			color: var(--oxide);
			border: 1px solid var(--rule);
		}}
		.theme-cards-grid {{
			display: grid;
			grid-template-columns: repeat(3, 1fr);
			gap: 12px;
			margin-bottom: 18px;
		}}
		.theme-preview-card {{
			border: 1.5px solid var(--rule);
			border-radius: var(--radius-sm);
			padding: 10px;
			background: var(--paper);
			cursor: pointer;
			transition: all 0.15s ease;
		}}
		.theme-preview-card.active {{
			border-color: var(--oxide);
			box-shadow: 0 0 0 1px var(--oxide);
		}}
		.theme-swatch-art {{
			height: 64px;
			border-radius: 4px;
			margin-bottom: 8px;
			padding: 10px;
			display: flex;
			flex-direction: column;
			justify-content: space-between;
			border: 1px solid rgba(0,0,0,0.08);
		}}
		.swatch-row {{
			display: flex;
			align-items: center;
			gap: 8px;
			margin-bottom: 16px;
		}}
		.swatch-btn {{
			width: 26px;
			height: 26px;
			border-radius: 999px;
			border: 2px solid var(--rule-strong);
			cursor: pointer;
			padding: 0;
		}}
		.wp-dropzone {{
			border: 2px dashed var(--rule-strong);
			border-radius: var(--radius-md);
			padding: 26px 20px;
			text-align: center;
			background: var(--paper);
			cursor: pointer;
			transition: border-color 0.15s ease;
			margin-bottom: 14px;
		}}
		.wp-dropzone:hover {{
			border-color: var(--oxide);
		}}
	</style>
</head>
<body data-preview-theme="{theme}">
<div class="setup-ambient" aria-hidden="true">
	<div class="setup-ambient-grid"></div>
	<div class="setup-orb setup-orb-a"></div>
	<div class="setup-orb setup-orb-b"></div>
	<svg class="setup-topo-svg" viewBox="0 0 1440 900" preserveAspectRatio="none">
		<path class="setup-topo-path" d="M-100,180 C320,90 540,310 920,190 C1180,110 1340,240 1560,170" />
		<path class="setup-topo-path" d="M-100,460 C260,360 610,560 980,430 C1220,340 1380,480 1560,410" />
		<path class="setup-topo-path" d="M-100,720 C340,630 680,790 1040,680 C1260,610 1400,720 1560,660" />
	</svg>
</div>

<div class="setup-box">
	<div class="setup-header">
		<div class="setup-badge">&#9679; Local Native Rust Engine &bull; Ready in 60 Seconds</div>
		<a class="login-logo" href="/" aria-label="Daypress" style="margin-bottom: 8px;">
			<img src="/static/daypress-logo.svg" alt="daypress." />
		</a>
		<h1>Welcome to Daypress</h1>
		<p>Let&rsquo;s set up your administrator account and get your new website live on your homepage in under a minute.</p>
	</div>

	<div class="setup-stepper" role="tablist" aria-label="Setup steps">
		<button type="button" class="setup-step-pill active" id="step-pill-1" onclick="goToSetupStep(1)">
			<span class="step-num">1</span>
			<span>Account &amp; Site</span>
		</button>
		<button type="button" class="setup-step-pill" id="step-pill-2" onclick="goToSetupStep(2)">
			<span class="step-num">2</span>
			<span>Starting Path</span>
		</button>
		<button type="button" class="setup-step-pill" id="step-pill-3" onclick="goToSetupStep(3)">
			<span class="step-num">3</span>
			<span>Atmosphere</span>
		</button>
		<button type="button" class="setup-step-pill" id="step-pill-4" onclick="goToSetupStep(4)">
			<span class="step-num">4</span>
			<span>Permissions</span>
		</button>
	</div>

	<div class="setup-card">
		{notice_html}
		<form method="post" action="/setup" id="daypress-setup-form">
			<input type="hidden" name="import_payload" id="import_payload" value="" />
			<input type="hidden" name="import_format" id="import_format" value="" />

			<!-- STEP 1: Site Identity & Administrator Account -->
			<div class="setup-pane active" id="setup-step-1">
				<div class="setup-section-title">Step 01 of 04 &bull; Your Site &amp; Admin Account</div>
				<div class="setup-section-sub">Name your website and create the primary administrator login stored locally in SQLite.</div>

				<div class="setup-grid-2">
					<div>
						<label for="site_title">Website Name</label>
						<input type="text" name="site_title" id="site_title" value="{site_title}" placeholder="My Personal Site" required />
					</div>
					<div>
						<label for="tagline">Short Tagline</label>
						<input type="text" name="tagline" id="tagline" value="{tagline}" placeholder="Notes, projects, and essays." />
					</div>
				</div>

				<div class="setup-grid-2">
					<div>
						<label for="username">Admin Username</label>
						<input type="text" name="username" id="username" value="{username}" required autocomplete="username" />
					</div>
					<div>
						<label for="email">Admin Email</label>
						<input type="email" name="email" id="email" value="{email}" placeholder="you@example.com" required autocomplete="email" />
					</div>
				</div>
				<input type="hidden" name="display_name" id="display_name" value="{display_name}" />

				<label for="password">Administrator Password (min. {MIN_PASSWORD_LENGTH} characters)</label>
				<input type="password" name="password" id="password" required minlength="{MIN_PASSWORD_LENGTH}" autocomplete="new-password" oninput="syncAndValidatePassword()" placeholder="Create or generate a strong password..." />
				<input type="hidden" name="confirm_password" id="confirm_password" value="" />

				<div class="pwd-toolbar">
					<div style="display: flex; gap: 8px;">
						<button type="button" class="btn-inline" onclick="generateStrongPassword()">⚡ Generate strong password</button>
						<button type="button" class="btn-inline" id="toggle-pwd-btn" onclick="togglePasswordVisibility()">Show</button>
					</div>
					<span class="pwd-strength" id="pwd-strength-badge">Enter at least {MIN_PASSWORD_LENGTH} characters</span>
				</div>

				<div class="login-actions" style="justify-content: space-between; align-items: center; margin-top: 22px;">
					<span style="font-size: 12.5px; color: var(--ink-muted);">100% local &bull; Argon2id encrypted</span>
					<button type="button" class="btn-login" onclick="advanceFromStep1()" style="padding: 11px 22px; font-size: 14.5px;">Continue: Choose Starting Path &rarr;</button>
				</div>
			</div>

			<!-- STEP 2: Starting Path -->
			<div class="setup-pane" id="setup-step-2">
				<div class="setup-section-title">Step 02 of 04 &bull; How Would You Like to Start?</div>
				<div class="setup-section-sub">Pick the workflow that matches your project. You can also connect Claude Code, Codex, or Cursor right after setup.</div>

				<div class="starter-options">
					<label class="starter-option selected-path" id="path-card-starter" onclick="selectStarterPath('starter')">
						<input type="radio" name="starter_mode" value="starter" {chk_starter} onchange="selectStarterPath('starter')" />
						<div>
							<strong>Start with a Daypress Design <span class="path-tag">Recommended</span></strong>
							<span>Launches a designed Front Page in your chosen Daypress theme plus a sample &ldquo;Hello world!&rdquo; article so you can edit visually or with AI immediately.</span>
						</div>
					</label>
					<label class="starter-option" id="path-card-blank" onclick="selectStarterPath('blank')">
						<input type="radio" name="starter_mode" value="blank" {chk_blank} onchange="selectStarterPath('blank')" />
						<div>
							<strong>Start from Scratch (Blank Canvas) <span class="path-tag">AI &amp; Builder Ready</span></strong>
							<span>Creates a clean, minimal front page with zero sample blog posts&mdash;ideal when your AI agent or DayStudio is building the site from the ground up.</span>
						</div>
					</label>
					<label class="starter-option" id="path-card-migrate" onclick="selectStarterPath('migrate')">
						<input type="radio" name="starter_mode" value="migrate" {chk_migrate} onchange="selectStarterPath('migrate')" />
						<div>
							<strong>Import an Existing WordPress Website <span class="path-tag">1-Click / WXR</span></strong>
							<span>Drop a WordPress XML (WXR) or Daypress JSON bundle in the next step, or open the 1-Click WordPress Importer.</span>
						</div>
					</label>
				</div>

				<div class="login-actions" style="justify-content: space-between; align-items: center; margin-top: 22px;">
					<button type="button" class="btn-inline" onclick="goToSetupStep(1)">&larr; Back</button>
					<button type="button" class="btn-login" onclick="goToSetupStep(3)" style="padding: 11px 22px; font-size: 14.5px;">Continue: Final Step &rarr;</button>
				</div>
			</div>

			<!-- STEP 3: Theme Picker OR Inline WordPress Import -->
			<div class="setup-pane" id="setup-step-3">
				<div id="step3-design-panel">
					<div class="setup-section-title">Step 03 of 04 &bull; Choose Your Atmosphere &amp; Theme</div>
					<div class="setup-section-sub">Click a theme to preview its atmosphere live. You can change colors, fonts, and layouts anytime.</div>

					<div class="theme-cards-grid">
						<div class="theme-preview-card active" id="theme-card-daylight" onclick="pickThemeCard('Daypress Daylight', '#a8401b', 'serif')">
							<div class="theme-swatch-art" style="background: #f3eee4; color: #16171a;">
								<div style="font: 700 13px var(--font-serif);">Aa Editorial</div>
								<div style="display:flex;gap:4px;"><span style="width:14px;height:14px;border-radius:99px;background:#a8401b;display:inline-block;"></span><span style="width:14px;height:14px;border-radius:99px;background:#16171a;display:inline-block;"></span></div>
							</div>
							<strong style="font-size:13.5px;display:block;color:var(--ink);">Daypress Daylight</strong>
							<span style="font-size:11.5px;color:var(--ink-muted);">Warm paper &amp; terracotta</span>
						</div>
						<div class="theme-preview-card" id="theme-card-modern" onclick="pickThemeCard('Daypress Modern', '#2563eb', 'system')">
							<div class="theme-swatch-art" style="background: #f8fafc; color: #0f172a;">
								<div style="font: 700 13px var(--font-sans);">Aa Studio</div>
								<div style="display:flex;gap:4px;"><span style="width:14px;height:14px;border-radius:99px;background:#2563eb;display:inline-block;"></span><span style="width:14px;height:14px;border-radius:99px;background:#0f172a;display:inline-block;"></span></div>
							</div>
							<strong style="font-size:13.5px;display:block;color:var(--ink);">Daypress Modern</strong>
							<span style="font-size:11.5px;color:var(--ink-muted);">Crisp alabaster &amp; cobalt</span>
						</div>
						<div class="theme-preview-card" id="theme-card-night" onclick="pickThemeCard('Daypress Night', '#38bdf8', 'system')">
							<div class="theme-swatch-art" style="background: #090d16; color: #f8fafc;">
								<div style="font: 700 13px var(--font-mono);">Aa Night</div>
								<div style="display:flex;gap:4px;"><span style="width:14px;height:14px;border-radius:99px;background:#38bdf8;display:inline-block;"></span><span style="width:14px;height:14px;border-radius:99px;background:#1e293b;display:inline-block;"></span></div>
							</div>
							<strong style="font-size:13.5px;display:block;color:var(--ink);">Daypress Night</strong>
							<span style="font-size:11.5px;color:var(--ink-muted);">Obsidian dark slate</span>
						</div>
					</div>

					<div class="setup-grid-2">
						<div>
							<label for="theme">Active Theme Preset</label>
							<select name="theme" id="theme" onchange="onThemeSelectChange(this.value)">
								<option value="Daypress Daylight" {sel_daylight}>Daypress Daylight (Warm Editorial)</option>
								<option value="Daypress Modern" {sel_modern}>Daypress Modern (Clean Studio)</option>
								<option value="Daypress Night" {sel_night}>Daypress Night (Dark Obsidian)</option>
							</select>
						</div>
						<div>
							<label for="typography_scale">Typography Pairing</label>
							<select name="typography_scale" id="typography_scale">
								<option value="serif" {sel_serif}>Editorial Serif (Newsreader + Grotesk)</option>
								<option value="system" {sel_system}>Modern Sans (Schibsted Grotesk)</option>
								<option value="mono" {sel_mono}>Technical Mono (IBM Plex Mono)</option>
							</select>
						</div>
					</div>

					<label for="brand_color">Brand Accent Color</label>
					<div class="swatch-row">
						<input type="color" name="brand_color" id="brand_color" value="{brand_color}" style="width: 42px; height: 32px; border: 1px solid var(--rule-strong); border-radius: var(--radius-sm); background: var(--paper); cursor: pointer;" />
						<button type="button" class="swatch-btn" style="background: #a8401b;" title="Oxide (#a8401b)" onclick="setBrandColor('#a8401b')"></button>
						<button type="button" class="swatch-btn" style="background: #2563eb;" title="Cobalt (#2563eb)" onclick="setBrandColor('#2563eb')"></button>
						<button type="button" class="swatch-btn" style="background: #3d5a39;" title="Moss (#3d5a39)" onclick="setBrandColor('#3d5a39')"></button>
						<button type="button" class="swatch-btn" style="background: #38bdf8;" title="Sky (#38bdf8)" onclick="setBrandColor('#38bdf8')"></button>
						<span id="brand_color_label" style="font-family: var(--font-mono); font-size: 12.5px; color: var(--ink-muted);">{brand_color}</span>
					</div>
				</div>

				<div id="step3-migrate-panel" style="display: none;">
					<div class="setup-section-title">Step 03 of 03 &bull; Import Your WordPress Site</div>
					<div class="setup-section-sub">Drop a WordPress WXR (<code>.xml</code>) or Daypress (<code>.json</code>) export file below to migrate right now, or launch directly into the full 1-Click Importer.</div>

					<label class="wp-dropzone" for="wp_export_file_input">
						<input type="file" id="wp_export_file_input" accept=".xml,.json" style="display:none;" onchange="handleWpSetupFile(this)" />
						<div style="font-size: 24px; margin-bottom: 6px;">📦</div>
						<strong id="wp-dropzone-title" style="display:block;font-size:14.5px;color:var(--ink);">Click to select a WordPress <code>.xml</code> or <code>.json</code> export file</strong>
						<span id="wp-dropzone-sub" style="font-size:12.5px;color:var(--ink-muted);">Optional &mdash; leave empty if you want to download the 1-Click WordPress Exporter plugin on the next screen.</span>
					</label>
				</div>

				<div class="login-actions" style="justify-content: space-between; align-items: center; margin-top: 20px;">
					<button type="button" class="btn-inline" onclick="goToSetupStep(2)">&larr; Back</button>
					<button type="button" class="btn-login" onclick="goToSetupStep(4)" style="padding: 11px 22px; font-size: 14.5px;">Continue: Network Permissions &rarr;</button>
				</div>
			</div>

			<!-- STEP 4: Network & Firewall Permissions + Final Launch -->
			<div class="setup-pane" id="setup-step-4">
				<div class="setup-section-title">Step 04 of 04 &bull; Network &amp; Firewall Permissions</div>
				<div class="setup-section-sub">
					Why does Windows or macOS ask for Private/Public network permissions? When a local server listens on all network interfaces (<code>0.0.0.0</code>) so phones or other devices on Wi-Fi can reach it, your operating system asks once to allow the executable through the firewall.
				</div>

				<div class="starter-options" style="margin-bottom: 16px;">
					<label class="starter-option selected-path" id="net-card-localhost" onclick="selectNetworkMode('localhost')">
						<input type="radio" name="network_bind_mode" value="localhost" checked onchange="selectNetworkMode('localhost')" />
						<div>
							<strong>Localhost Only (<code>127.0.0.1</code>) <span class="path-tag">Default &bull; Zero Firewall Popups</span></strong>
							<span>Runs privately on this computer only (<code>http://localhost:8090</code>). Your browser and local AI agents (Claude Code, Codex, Cursor, Antigravity) connect without triggering any OS firewall dialogs.</span>
						</div>
					</label>
					<label class="starter-option" id="net-card-lan" onclick="selectNetworkMode('lan')">
						<input type="radio" name="network_bind_mode" value="lan" onchange="selectNetworkMode('lan')" />
						<div>
							<strong>Local Network &amp; Public Access (<code>0.0.0.0</code>) <span class="path-tag">Wi-Fi Preview / Server</span></strong>
							<span>Allows phones, tablets, and other devices on your network to open your site. Clicking &ldquo;Allow &amp; Verify Permissions&rdquo; below triggers the 1-time OS Firewall prompt right now so it never interrupts you later.</span>
						</div>
					</label>
				</div>

				<div style="padding: 14px 16px; border: 1px solid var(--rule); border-radius: var(--radius-sm); background: var(--paper); display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap;">
					<div>
						<strong id="net-perm-status-title" style="display: block; font-size: 13.5px; color: var(--ink);">🛡️ Pre-authorize Network Access</strong>
						<span id="net-perm-status-sub" style="font-size: 12.5px; color: var(--ink-muted);">Click once to test and approve the network listener for your chosen mode.</span>
					</div>
					<button type="button" class="btn-inline" id="verify-net-btn" onclick="verifyNetworkPermission()">Allow &amp; Verify Permissions</button>
				</div>

				<div class="login-actions" style="justify-content: space-between; align-items: center; margin-top: 22px;">
					<button type="button" class="btn-inline" onclick="goToSetupStep(3)">&larr; Back</button>
					<button type="submit" class="btn-login" id="final-submit-btn" style="padding: 12px 24px; font-size: 15px;">Install Daypress &amp; Sign In &rarr;</button>
				</div>
			</div>
		</form>
	</div>
</div>

<script>
let currentSetupStep = 1;
function goToSetupStep(step) {{
	currentSetupStep = step;
	for (let i = 1; i <= 4; i++) {{
		const pane = document.getElementById('setup-step-' + i);
		const pill = document.getElementById('step-pill-' + i);
		if (pane) pane.classList.toggle('active', i === step);
		if (pill) {{
			pill.classList.toggle('active', i === step);
			pill.classList.toggle('done', i < step);
		}}
	}}
}}
function selectNetworkMode(mode) {{
	['localhost', 'lan'].forEach(m => {{
		const card = document.getElementById('net-card-' + m);
		if (card) card.classList.toggle('selected-path', m === mode);
	}});
	document.querySelectorAll('input[name="network_bind_mode"]').forEach(r => {{
		if (r.value === mode) r.checked = true;
	}});
}}
async function verifyNetworkPermission() {{
	const btn = document.getElementById('verify-net-btn');
	const title = document.getElementById('net-perm-status-title');
	const sub = document.getElementById('net-perm-status-sub');
	const selected = document.querySelector('input[name="network_bind_mode"]:checked')?.value || 'localhost';
	if (btn) {{
		btn.disabled = true;
		btn.textContent = 'Checking...';
	}}
	try {{
		const res = await fetch('/setup/network-permission', {{
			method: 'POST',
			headers: {{ 'Content-Type': 'application/json' }},
			body: JSON.stringify({{ mode: selected }})
		}});
		const data = await res.json();
		if (data.success) {{
			if (title) {{
				title.textContent = '✓ Network & Firewall Permissions Ready (' + (selected === 'lan' ? '0.0.0.0 LAN/Public' : '127.0.0.1 Localhost') + ')';
				title.style.color = 'var(--moss)';
			}}
			if (sub) sub.textContent = 'Verified listener on ' + (data.bound_test_addr || selected) + ' — you are ready to launch!';
			if (btn) {{
				btn.textContent = '✓ Verified';
				btn.disabled = false;
			}}
		}} else {{
			if (title) title.textContent = 'Could not bind network socket';
			if (sub) sub.textContent = data.error || 'Try Localhost mode.';
			if (btn) {{
				btn.textContent = 'Retry';
				btn.disabled = false;
			}}
		}}
	}} catch (e) {{
		if (btn) {{
			btn.textContent = 'Retry';
			btn.disabled = false;
		}}
	}}
}}
function advanceFromStep1() {{
	const title = document.getElementById('site_title');
	const user = document.getElementById('username');
	const email = document.getElementById('email');
	const pwd = document.getElementById('password');
	if (!title.value.trim()) {{ title.focus(); return; }}
	if (!user.value.trim()) {{ user.focus(); return; }}
	if (!email.value.trim() || !email.value.includes('@')) {{ email.focus(); return; }}
	if (!pwd.value || pwd.value.length < {MIN_PASSWORD_LENGTH}) {{
		pwd.focus();
		syncAndValidatePassword();
		return;
	}}
	syncAndValidatePassword();
	goToSetupStep(2);
}}
function selectStarterPath(mode) {{
	['starter', 'blank', 'migrate'].forEach(m => {{
		const card = document.getElementById('path-card-' + m);
		if (card) card.classList.toggle('selected-path', m === mode);
	}});
	const radios = document.querySelectorAll('input[name="starter_mode"]');
	radios.forEach(r => {{ if (r.value === mode) r.checked = true; }});
	const designPanel = document.getElementById('step3-design-panel');
	const migratePanel = document.getElementById('step3-migrate-panel');
	if (designPanel && migratePanel) {{
		designPanel.style.display = mode === 'migrate' ? 'none' : 'block';
		migratePanel.style.display = mode === 'migrate' ? 'block' : 'none';
	}}
}}
function pickThemeCard(themeName, accentHex, typo) {{
	const sel = document.getElementById('theme');
	if (sel) sel.value = themeName;
	const typoSel = document.getElementById('typography_scale');
	if (typoSel && typo) typoSel.value = typo;
	setBrandColor(accentHex);
	onThemeSelectChange(themeName);
}}
function onThemeSelectChange(themeName) {{
	document.body.setAttribute('data-preview-theme', themeName);
	const map = {{
		'Daypress Daylight': 'theme-card-daylight',
		'Daypress Modern': 'theme-card-modern',
		'Daypress Night': 'theme-card-night'
	}};
	Object.entries(map).forEach(([k, id]) => {{
		const el = document.getElementById(id);
		if (el) el.classList.toggle('active', k === themeName);
	}});
}}
function setBrandColor(hex) {{
	const input = document.getElementById('brand_color');
	const label = document.getElementById('brand_color_label');
	if (input) input.value = hex;
	if (label) label.textContent = hex;
}}
document.getElementById('brand_color')?.addEventListener('input', function(e) {{
	const label = document.getElementById('brand_color_label');
	if (label) label.textContent = e.target.value;
}});
function togglePasswordVisibility() {{
	const p1 = document.getElementById('password');
	const btn = document.getElementById('toggle-pwd-btn');
	if (!p1) return;
	const show = p1.type === 'password';
	p1.type = show ? 'text' : 'password';
	if (btn) btn.textContent = show ? 'Hide' : 'Show';
}}
function generateStrongPassword() {{
	const alphabet = 'abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@#$%&*-_';
	const bytes = new Uint8Array(18);
	if (window.crypto && window.crypto.getRandomValues) {{
		window.crypto.getRandomValues(bytes);
	}} else {{
		for (let i = 0; i < bytes.length; i++) bytes[i] = Math.floor(Math.random() * 256);
	}}
	let out = '';
	for (let i = 0; i < bytes.length; i++) {{
		out += alphabet[bytes[i] % alphabet.length];
	}}
	const p1 = document.getElementById('password');
	const p2 = document.getElementById('confirm_password');
	const btn = document.getElementById('toggle-pwd-btn');
	if (p1) {{
		p1.value = out;
		if (p2) p2.value = out;
		p1.type = 'text';
		if (btn) btn.textContent = 'Hide';
		syncAndValidatePassword();
	}}
}}
function syncAndValidatePassword() {{
	const p1 = document.getElementById('password')?.value || '';
	const p2 = document.getElementById('confirm_password');
	if (p2) p2.value = p1;
	const badge = document.getElementById('pwd-strength-badge');
	if (!badge) return;
	if (p1.length < {MIN_PASSWORD_LENGTH}) {{
		badge.textContent = 'Too short (min. {MIN_PASSWORD_LENGTH} chars)';
		badge.style.background = 'var(--oxide-wash)';
		badge.style.color = 'var(--oxide)';
		return;
	}}
	const strong = p1.length >= 12 && /[A-Z]/.test(p1) && /[0-9]/.test(p1);
	badge.textContent = strong ? '✓ Strong password' : '✓ Good password';
	badge.style.background = 'var(--moss-wash)';
	badge.style.color = 'var(--moss)';
}}
function handleWpSetupFile(input) {{
	const file = input.files && input.files[0];
	if (!file) return;
	const reader = new FileReader();
	reader.onload = function(evt) {{
		const content = evt.target.result || '';
		document.getElementById('import_payload').value = content;
		const isJson = file.name.toLowerCase().endsWith('.json') || content.trim().startsWith('{{');
		document.getElementById('import_format').value = isJson ? 'json' : 'wxr';
		document.getElementById('wp-dropzone-title').textContent = '✓ Ready to import: ' + file.name;
		document.getElementById('wp-dropzone-sub').textContent = (file.size / 1024).toFixed(1) + ' KB loaded — will be imported automatically on launch.';
	}};
	reader.readAsText(file);
}}
</script>
</body>
</html>"##,
        DESK_FONT_FACES = DESK_FONT_FACES,
        DESK_FOUNDATION = DESK_FOUNDATION,
        LOGIN_STYLES = LOGIN_STYLES,
        MIN_PASSWORD_LENGTH = MIN_PASSWORD_LENGTH,
        notice_html = notice_html,
        site_title = escape_html(site_title),
        tagline = escape_html(tagline),
        username = escape_html(username),
        display_name = escape_html(display_name),
        email = escape_html(email),
        theme = escape_html(theme),
        brand_color = escape_html(brand_color),
        sel_daylight = sel_daylight,
        sel_modern = sel_modern,
        sel_night = sel_night,
        sel_serif = sel_serif,
        sel_system = sel_system,
        sel_mono = sel_mono,
        chk_starter = chk_starter,
        chk_blank = chk_blank,
        chk_migrate = chk_migrate,
    )
}
