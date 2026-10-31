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
		.setup-box {{
			width: 100%;
			max-width: 640px;
			margin: 32px auto;
		}}
		.setup-header {{
			text-align: center;
			margin-bottom: 28px;
		}}
		.setup-header h1 {{
			font: 460 34px/1.15 var(--font-serif);
			letter-spacing: -0.02em;
			color: var(--ink);
			margin-top: 16px;
			margin-bottom: 8px;
		}}
		.setup-header p {{
			font-size: 15px;
			color: var(--ink-muted);
			max-width: 480px;
			margin: 0 auto;
		}}
		.setup-card {{
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-md);
			padding: 28px 32px;
			box-shadow: 0 12px 32px rgba(22, 23, 26, 0.06);
		}}
		.setup-section {{
			padding-bottom: 22px;
			margin-bottom: 22px;
			border-bottom: 1px solid var(--rule);
		}}
		.setup-section:last-of-type {{
			border-bottom: none;
			margin-bottom: 12px;
			padding-bottom: 8px;
		}}
		.setup-section-title {{
			font: 600 12px/16px var(--font-mono);
			text-transform: uppercase;
			letter-spacing: 0.08em;
			color: var(--oxide);
			margin-bottom: 14px;
			display: flex;
			align-items: center;
			gap: 8px;
		}}
		.setup-grid-2 {{
			display: grid;
			grid-template-columns: 1fr 1fr;
			gap: 16px;
		}}
		@media (max-width: 560px) {{
			.setup-grid-2 {{ grid-template-columns: 1fr; gap: 0; }}
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
			padding: 5px 10px;
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
			gap: 10px;
			margin-top: 6px;
		}}
		.starter-option {{
			display: flex;
			align-items: flex-start;
			gap: 12px;
			padding: 12px 14px;
			border: 1px solid var(--rule);
			border-radius: var(--radius-sm);
			background: var(--paper);
			cursor: pointer;
			font-weight: 400;
			margin-bottom: 0;
		}}
		.starter-option:hover {{
			border-color: var(--rule-strong);
		}}
		.starter-option input[type="radio"] {{
			margin-top: 3px;
			accent-color: var(--oxide);
		}}
		.starter-option strong {{
			display: block;
			font-size: 14px;
			color: var(--ink);
		}}
		.starter-option span {{
			display: block;
			font-size: 13px;
			color: var(--ink-muted);
			margin-top: 2px;
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
	</style>
</head>
<body>
<div class="setup-box">
	<div class="setup-header">
		<a class="login-logo" href="/" aria-label="Daypress" style="margin-bottom: 12px;">
			<img src="/static/daypress-logo.svg" alt="daypress." />
		</a>
		<h1>Welcome to Daypress</h1>
		<p>Create your administrator account, choose your default Daypress theme, and launch your site in under a minute.</p>
	</div>

	<div class="setup-card">
		{notice_html}
		<form method="post" action="/setup" id="daypress-setup-form">
			<!-- 1. Site Identity -->
			<div class="setup-section">
				<div class="setup-section-title">01 &bull; Site Identity</div>
				<div class="setup-grid-2">
					<div>
						<label for="site_title">Site Title</label>
						<input type="text" name="site_title" id="site_title" value="{site_title}" required />
					</div>
					<div>
						<label for="tagline">Tagline</label>
						<input type="text" name="tagline" id="tagline" value="{tagline}" />
					</div>
				</div>
			</div>

			<!-- 2. Administrator Account -->
			<div class="setup-section">
				<div class="setup-section-title">02 &bull; Administrator Account</div>
				<div class="setup-grid-2">
					<div>
						<label for="username">Username</label>
						<input type="text" name="username" id="username" value="{username}" required autocomplete="username" />
						<div class="field-hint">Letters, numbers, underscores, hyphens, or dots.</div>
					</div>
					<div>
						<label for="display_name">Display Name</label>
						<input type="text" name="display_name" id="display_name" value="{display_name}" autocomplete="name" />
						<div class="field-hint">Shown publicly as the author name on posts.</div>
					</div>
				</div>

				<label for="email">Administrator Email</label>
				<input type="email" name="email" id="email" value="{email}" placeholder="you@example.com" required autocomplete="email" />
				<div class="field-hint">Used for password reset links and site notifications.</div>

				<div class="setup-grid-2">
					<div>
						<label for="password">Password (min. {MIN_PASSWORD_LENGTH} characters)</label>
						<input type="password" name="password" id="password" required minlength="{MIN_PASSWORD_LENGTH}" autocomplete="new-password" oninput="updatePasswordStrength()" />
					</div>
					<div>
						<label for="confirm_password">Confirm Password</label>
						<input type="password" name="confirm_password" id="confirm_password" required minlength="{MIN_PASSWORD_LENGTH}" autocomplete="new-password" oninput="updatePasswordStrength()" />
					</div>
				</div>
				<div class="pwd-toolbar">
					<div style="display: flex; gap: 8px;">
						<button type="button" class="btn-inline" onclick="generateStrongPassword()">⚡ Generate strong password</button>
						<button type="button" class="btn-inline" id="toggle-pwd-btn" onclick="togglePasswordVisibility()">Show</button>
					</div>
					<span class="pwd-strength" id="pwd-strength-badge">Enter at least {MIN_PASSWORD_LENGTH} characters</span>
				</div>
			</div>

			<!-- 3. Default Daypress Theme & Content -->
			<div class="setup-section">
				<div class="setup-section-title">03 &bull; Default Theme &amp; Starter Pack</div>
				<div class="setup-grid-2">
					<div>
						<label for="theme">Default Daypress Theme</label>
						<select name="theme" id="theme">
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
					<button type="button" class="swatch-btn" style="background: #2a3bd0;" title="Press Blue (#2a3bd0)" onclick="setBrandColor('#2a3bd0')"></button>
					<button type="button" class="swatch-btn" style="background: #3d5a39;" title="Moss (#3d5a39)" onclick="setBrandColor('#3d5a39')"></button>
					<button type="button" class="swatch-btn" style="background: #16171a;" title="Ink (#16171a)" onclick="setBrandColor('#16171a')"></button>
					<span id="brand_color_label" style="font-family: var(--font-mono); font-size: 12.5px; color: var(--ink-muted);">{brand_color}</span>
				</div>

				<label>Initial Site Content</label>
				<div class="starter-options">
					<label class="starter-option">
						<input type="radio" name="starter_mode" value="starter" {chk_starter} />
						<div>
							<strong>Daypress Starter Pack (Recommended)</strong>
							<span>Includes the updated &ldquo;Hello world!&rdquo; post, a Sample Page, a DayStudio visual builder page, and a ready-to-use navigation menu.</span>
						</div>
					</label>
					<label class="starter-option">
						<input type="radio" name="starter_mode" value="blank" {chk_blank} />
						<div>
							<strong>Clean Blank Slate</strong>
							<span>Start with zero sample posts or pages so you can build from scratch.</span>
						</div>
					</label>
					<label class="starter-option">
						<input type="radio" name="starter_mode" value="migrate" {chk_migrate} />
						<div>
							<strong>Migrate an Existing WordPress Site</strong>
							<span>Skip sample content and open the 1-Click WordPress &amp; WXR Importer right after setup.</span>
						</div>
					</label>
				</div>
			</div>

			<div class="login-actions" style="justify-content: space-between; align-items: center; margin-top: 16px;">
				<a href="/" style="font-size: 13.5px; color: var(--ink-muted); text-decoration: none;">Preview public site &rarr;</a>
				<button type="submit" class="btn-login" style="padding: 12px 24px; font-size: 15px;">Install Daypress &amp; Sign In &rarr;</button>
			</div>
		</form>
	</div>
</div>

<script>
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
	const p2 = document.getElementById('confirm_password');
	const btn = document.getElementById('toggle-pwd-btn');
	if (!p1 || !p2) return;
	const show = p1.type === 'password';
	p1.type = show ? 'text' : 'password';
	p2.type = show ? 'text' : 'password';
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
	if (p1 && p2) {{
		p1.value = out;
		p2.value = out;
		p1.type = 'text';
		p2.type = 'text';
		if (btn) btn.textContent = 'Hide';
		updatePasswordStrength();
	}}
}}
function updatePasswordStrength() {{
	const p1 = document.getElementById('password')?.value || '';
	const p2 = document.getElementById('confirm_password')?.value || '';
	const badge = document.getElementById('pwd-strength-badge');
	if (!badge) return;
	if (p1.length < {MIN_PASSWORD_LENGTH}) {{
		badge.textContent = 'Too short (min. {MIN_PASSWORD_LENGTH} chars)';
		badge.style.background = 'var(--oxide-wash)';
		badge.style.color = 'var(--oxide)';
		return;
	}}
	if (p2.length > 0 && p1 !== p2) {{
		badge.textContent = 'Passwords do not match';
		badge.style.background = 'var(--oxide-wash)';
		badge.style.color = 'var(--oxide)';
		return;
	}}
	const strong = p1.length >= 12 && /[A-Z]/.test(p1) && /[0-9]/.test(p1);
	badge.textContent = strong ? '✓ Strong password' : '✓ Good password';
	badge.style.background = 'var(--moss-wash)';
	badge.style.color = 'var(--moss)';
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
