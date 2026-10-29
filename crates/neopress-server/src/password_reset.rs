//! Getting back in after losing a password.
//!
//! * The "Lost your password?" email flow behind [`LOST_PASSWORD_PATH`] and
//!   [`RESET_PASSWORD_PATH`] (handlers in [`crate::handlers::lost_password`]).
//! * [`reset_password_from_cli`], behind `neopress reset-password <username>`, for when email
//!   is not set up or the account has no address.
//!
//! Token storage lives in [`crate::adapters::database::password_resets`].

pub use crate::adapters::database::maintenance::PasswordResetReport;
pub use crate::adapters::database::password_resets::{
    PasswordResetAccount, PasswordResetTokenRow, MAX_RESET_EMAILS_PER_ACCOUNT_PER_HOUR,
    PASSWORD_RESET_TTL_SECS,
};
use crate::auth::LoginThrottle;
use crate::db::Database;
use crate::email::EmailMessage;
use crate::errors::AppError;
use crate::templates::utils::escape_html;
use std::path::Path;

/// Environment variable `neopress reset-password` takes the new password from.
pub const NEW_PASSWORD_ENV: &str = "NEOPRESS_NEW_PASSWORD";

/// Shortest password the reset flows accept (the "Add New User" form asks for the same).
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// The "Lost your password?" page.
pub const LOST_PASSWORD_PATH: &str = "/login/lost-password";
/// The page a reset link opens (`?token=...`).
pub const RESET_PASSWORD_PATH: &str = "/login/reset-password";

/// Checks a new password against the reset policy; the error is a sentence for the user.
pub fn check_password_policy(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "The password must be at least {MIN_PASSWORD_LENGTH} characters long."
        ));
    }
    if password.trim() != password {
        return Err("The password cannot start or end with a space.".to_string());
    }
    Ok(())
}

/// Validates the two password fields of the reset form.
pub fn validate_new_password(password: &str, confirmation: &str) -> Result<(), String> {
    if password != confirmation {
        return Err("The two passwords do not match.".to_string());
    }
    check_password_policy(password)
}

/// A reset email ready to send.
#[derive(Debug, Clone)]
pub struct ResetEmail {
    pub user_id: i64,
    /// Plaintext token in the link. Only its hash is stored; the HTTP handler discards it.
    pub token: String,
    pub message: EmailMessage,
}

/// Outcome of a "Lost your password?" request.
///
/// Callers answer `Accepted(Some(_))` and `Accepted(None)` with the same response, so it
/// never reveals whether an account exists.
#[derive(Debug, Clone)]
pub enum ResetRequestOutcome {
    /// Too many requests from this client for this name; nothing was created or sent.
    Throttled { retry_after_secs: i64 },
    /// `Some` carries the email to deliver. `None`: no matching account, an account without an
    /// email address, or the account already got [`MAX_RESET_EMAILS_PER_ACCOUNT_PER_HOUR`].
    Accepted(Option<ResetEmail>),
}

/// Handles a "Lost your password?" request for a login name or email address.
///
/// Requests are limited per client address + name like logins (5 per 15 minutes, a counter
/// separate from the login throttle). Every request counts, whether or not an account matches.
pub fn request_password_reset(
    db: &Database,
    identifier: &str,
    client: Option<&str>,
) -> Result<ResetRequestOutcome, AppError> {
    let identifier = identifier.trim();
    let key = LoginThrottle::key(client, identifier);
    if let Some(retry_after_secs) = db.password_reset_throttle.retry_after(&key) {
        return Ok(ResetRequestOutcome::Throttled { retry_after_secs });
    }
    db.password_reset_throttle.record_failure(&key);

    let Some(account) = db.find_account_for_password_reset(identifier)? else {
        return Ok(ResetRequestOutcome::Accepted(None));
    };
    // One plain address only: the mailer would treat commas as extra recipients.
    if !account.email.contains('@') || account.email.contains([',', ';']) {
        return Ok(ResetRequestOutcome::Accepted(None));
    }
    if db.recent_password_reset_count(account.user_id)? >= MAX_RESET_EMAILS_PER_ACCOUNT_PER_HOUR {
        tracing::warn!(
            user_id = account.user_id,
            "password reset email skipped: hourly limit for this account reached"
        );
        return Ok(ResetRequestOutcome::Accepted(None));
    }
    let token = db.create_password_reset_token(account.user_id)?;
    let message = reset_email_message(db, &account, &token);
    Ok(ResetRequestOutcome::Accepted(Some(ResetEmail {
        user_id: account.user_id,
        token,
        message,
    })))
}

/// Sends a reset email. The outcome lands in the email log (`not_sent` without SMTP); it is
/// never shown to the visitor.
pub async fn deliver_reset_email(db: &Database, email: &ResetEmail) {
    if let Err(err) = crate::email::send_transactional_email(db, &email.message).await {
        tracing::warn!(
            user_id = email.user_id,
            "password reset email not delivered: {err}"
        );
    }
}

/// Base URL for links in emails: the `siteurl` option (then `home`). Never the request's
/// `Host` header, which whoever sends the request controls.
pub fn site_base_url(db: &Database) -> String {
    ["siteurl", "home"]
        .iter()
        .find_map(|name| {
            db.get_option(name)
                .ok()
                .flatten()
                .map(|v| v.trim().trim_end_matches('/').to_string())
                .filter(|v| v.starts_with("http://") || v.starts_with("https://"))
        })
        .unwrap_or_else(|| "http://localhost:8090".to_string())
}

/// The link a reset email carries.
pub fn reset_link(db: &Database, token: &str) -> String {
    format!(
        "{}{}?token={}",
        site_base_url(db),
        RESET_PASSWORD_PATH,
        token
    )
}

fn reset_email_message(db: &Database, account: &PasswordResetAccount, token: &str) -> EmailMessage {
    let site = db
        .get_option("blogname")
        .ok()
        .flatten()
        .map(|s| s.replace(['\r', '\n'], " ").trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Neopress".to_string());
    let link = reset_link(db, token);
    let minutes = PASSWORD_RESET_TTL_SECS / 60;
    let login = &account.login;
    let body_text = format!(
        "Someone asked to reset the password of this account on {site}.\n\n\
         Username: {login}\n\n\
         To choose a new password, open this link within {minutes} minutes. It works once:\n\
         {link}\n\n\
         If you did not ask for this, ignore this email. Your password stays the same.\n"
    );
    let body_html = format!(
        "<p>Someone asked to reset the password of this account on {site}.</p>\
         <p>Username: <strong>{login}</strong></p>\
         <p>To choose a new password, open this link within {minutes} minutes. It works once:<br>\
         <a href=\"{link}\">{link}</a></p>\
         <p>If you did not ask for this, ignore this email. Your password stays the same.</p>",
        site = escape_html(&site),
        login = escape_html(login),
        link = escape_html(&link),
    );
    EmailMessage {
        to: account.email.clone(),
        subject: format!("[{site}] Password reset"),
        body_html,
        body_text,
    }
}

/// Outcome of submitting the reset form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResetCompletion {
    /// The token is unknown, already used or expired.
    InvalidLink,
    /// The new password was not accepted; the message is for the user.
    Rejected(String),
    /// The password was changed.
    Done { user_id: i64 },
}

/// Completes a reset from the reset form: checks the token, validates the password, uses the
/// token once and clears this client's failed-login counter for the account, so the user can
/// log in right away.
pub fn complete_password_reset(
    db: &Database,
    token: &str,
    password: &str,
    confirmation: &str,
    client: Option<&str>,
) -> Result<ResetCompletion, AppError> {
    if db.password_reset_token_user(token)?.is_none() {
        return Ok(ResetCompletion::InvalidLink);
    }
    if let Err(message) = validate_new_password(password, confirmation) {
        return Ok(ResetCompletion::Rejected(message));
    }
    let Some(user_id) = db.consume_password_reset_token(token, password)? else {
        return Ok(ResetCompletion::InvalidLink);
    };
    if let Ok(Some(user)) = db.get_user_detail(user_id) {
        for name in [user.username, user.email] {
            db.login_throttle
                .record_success(&LoginThrottle::key(client, &name));
        }
    }
    Ok(ResetCompletion::Done { user_id })
}

/// `neopress reset-password`: opens the existing database at `db_path` (never creates one),
/// sets a new password for the account (generated when `new_password` is `None`) and, with
/// `sign_out_everyone`, rotates the session signing key so every user is logged out.
pub fn reset_password_from_cli(
    db_path: &Path,
    identifier: &str,
    new_password: Option<&str>,
    sign_out_everyone: bool,
) -> Result<PasswordResetReport, AppError> {
    let db = Database::open_existing(db_path)?;
    let report = db.reset_user_password(identifier, new_password)?;
    if sign_out_everyone {
        db.rotate_auth_secret()?;
    }
    Ok(report)
}
