//! "Lost your password?" screens: `/login/lost-password` and `/login/reset-password`.
//! Both are public (outside `/n-admin`, so the admin session gate does not apply).

use crate::auth::ClientIp;
use crate::db::Database;
use crate::password_reset::{self, ResetCompletion, ResetRequestOutcome};
use crate::templates::{self, LoginNotice};
use axum::{
    extract::{Form, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

/// Answer to every accepted request, whether or not an account matched.
pub const RESET_LINK_SENT_NOTICE: &str = "If an account matches what you entered, we have emailed it a link to choose a new password. The link works once and expires in 1 hour. If nothing arrives, check your spam folder.";

/// Answer to every accepted request while the site has no SMTP server configured. It depends
/// only on site configuration, so it reveals nothing about which accounts exist.
pub const RESET_EMAIL_NOT_SENT_NOTICE: &str = "If an account matches what you entered, we tried to email it a reset link, but this site cannot send email yet (no SMTP server is set up), so nothing was sent. If you run this site, set a new password on the server with: neopress reset-password <username>";

/// Shown above the form while the site has no SMTP server configured.
pub const RESET_EMAIL_UNAVAILABLE_NOTICE: &str = "This site cannot send email yet (no SMTP server is set up), so reset links cannot be delivered. If you run this site, set a new password on the server with: neopress reset-password <username>";

/// Where the reset form sends the user once the password is saved.
pub const PASSWORD_RESET_DONE_LOCATION: &str = "/login?action=resetpass";

#[derive(Debug, Deserialize, Default)]
pub struct LostPasswordForm {
    pub user_login: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ResetPasswordQuery {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ResetPasswordForm {
    pub token: Option<String>,
    pub pass1: Option<String>,
    pub pass2: Option<String>,
}

fn smtp_ready(db: &Database) -> bool {
    crate::email::smtp_config(db).is_some()
}

/// Pages that carry a reset token: never cached, never sent on as a referrer.
fn token_page(status: StatusCode, html: String) -> Response {
    (
        status,
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::REFERRER_POLICY, "no-referrer"),
        ],
        Html(html),
    )
        .into_response()
}

fn lost_password_page(db: &Database, status: StatusCode, notice: LoginNotice<'_>) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    (
        status,
        Html(templates::render_lost_password_view(&settings, notice)),
    )
        .into_response()
}

fn server_error_page(db: &Database, err: &crate::errors::AppError) -> Response {
    tracing::error!("password reset failed: {err}");
    lost_password_page(
        db,
        StatusCode::INTERNAL_SERVER_ERROR,
        LoginNotice::Error("Something went wrong on our side. Please try again later."),
    )
}

/// `GET /login/lost-password`
pub async fn handle_lost_password_page(State(db): State<Database>) -> Response {
    let notice = if smtp_ready(&db) {
        LoginNotice::None
    } else {
        LoginNotice::Info(RESET_EMAIL_UNAVAILABLE_NOTICE)
    };
    lost_password_page(&db, StatusCode::OK, notice)
}

/// `POST /login/lost-password` (`user_login=<username or email>`)
pub async fn handle_lost_password_submit(
    State(db): State<Database>,
    ClientIp(client): ClientIp,
    Form(form): Form<LostPasswordForm>,
) -> Response {
    let identifier = form.user_login.unwrap_or_default();
    let identifier = identifier.trim();
    if identifier.is_empty() {
        return lost_password_page(
            &db,
            StatusCode::OK,
            LoginNotice::Error("Enter a username or email address."),
        );
    }
    let accepted_notice = if smtp_ready(&db) {
        LoginNotice::Info(RESET_LINK_SENT_NOTICE)
    } else {
        LoginNotice::Info(RESET_EMAIL_NOT_SENT_NOTICE)
    };
    // No login or email address is this long; answer like any other unknown name.
    if identifier.chars().count() > 254 {
        return lost_password_page(&db, StatusCode::OK, accepted_notice);
    }

    match password_reset::request_password_reset(&db, identifier, client.as_deref()) {
        Ok(ResetRequestOutcome::Throttled { retry_after_secs }) => {
            let message = format!(
                "Too many password reset requests. Please try again in {} minutes.",
                (retry_after_secs + 59) / 60
            );
            lost_password_page(
                &db,
                StatusCode::TOO_MANY_REQUESTS,
                LoginNotice::Error(&message),
            )
        }
        Ok(ResetRequestOutcome::Accepted(email)) => {
            if let Some(email) = email {
                // Deliver in the background: waiting for SMTP would make the response for an
                // existing account measurably slower than for an unknown name.
                let mail_db = db.clone();
                tokio::spawn(async move {
                    password_reset::deliver_reset_email(&mail_db, &email).await;
                });
            }
            lost_password_page(&db, StatusCode::OK, accepted_notice)
        }
        Err(err) => server_error_page(&db, &err),
    }
}

/// `GET /login/reset-password?token=...`
pub async fn handle_reset_password_page(
    State(db): State<Database>,
    Query(query): Query<ResetPasswordQuery>,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    let token = query.token.unwrap_or_default();
    match db.password_reset_token_user(&token) {
        Ok(Some(_)) => token_page(
            StatusCode::OK,
            templates::render_reset_password_view(&settings, token.trim(), LoginNotice::None),
        ),
        Ok(None) => token_page(
            StatusCode::BAD_REQUEST,
            templates::render_reset_link_invalid_view(&settings),
        ),
        Err(err) => server_error_page(&db, &err),
    }
}

/// `POST /login/reset-password` (`token`, `pass1`, `pass2`)
pub async fn handle_reset_password_submit(
    State(db): State<Database>,
    ClientIp(client): ClientIp,
    Form(form): Form<ResetPasswordForm>,
) -> Response {
    let settings = db.get_settings().unwrap_or_default();
    let token = form.token.unwrap_or_default();
    let password = form.pass1.unwrap_or_default();
    let confirmation = form.pass2.unwrap_or_default();
    match password_reset::complete_password_reset(
        &db,
        &token,
        &password,
        &confirmation,
        client.as_deref(),
    ) {
        Ok(ResetCompletion::Done { .. }) => {
            Redirect::to(PASSWORD_RESET_DONE_LOCATION).into_response()
        }
        Ok(ResetCompletion::Rejected(message)) => token_page(
            StatusCode::OK,
            templates::render_reset_password_view(
                &settings,
                token.trim(),
                LoginNotice::Error(&message),
            ),
        ),
        Ok(ResetCompletion::InvalidLink) => token_page(
            StatusCode::BAD_REQUEST,
            templates::render_reset_link_invalid_view(&settings),
        ),
        Err(err) => server_error_page(&db, &err),
    }
}
