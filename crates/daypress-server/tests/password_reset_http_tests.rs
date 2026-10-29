//! HTTP tests for the "Lost your password?" flow: `/login/lost-password` and
//! `/login/reset-password`.

use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Request, StatusCode},
    Router,
};
use daypress_server::{create_app, Database, EmailLogEntry};
use std::time::Duration;
use tower::ServiceExt;

const NEW_PASSWORD: &str = "Brand-New-Pass-7";

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    Reply {
        status,
        headers,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

async fn get(app: &Router, uri: &str) -> Reply {
    send(
        app,
        Request::builder()
            .uri(uri)
            .body(Body::empty())
            .expect("request"),
    )
    .await
}

async fn post_form(app: &Router, uri: &str, form: &str) -> Reply {
    send(
        app,
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form.to_string()))
            .expect("request"),
    )
    .await
}

fn reset_form(token: &str, pass1: &str, pass2: &str) -> String {
    format!("token={token}&pass1={pass1}&pass2={pass2}")
}

/// Reset emails are delivered in a background task; give it a moment to log its attempt.
async fn email_logs_after_delivery(db: &Database, expected: usize) -> Vec<EmailLogEntry> {
    for _ in 0..200 {
        let logs = db.get_email_logs(50).expect("email logs");
        if logs.len() >= expected {
            return logs;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    db.get_email_logs(50).expect("email logs")
}

fn setup() -> (Database, Router) {
    let db = Database::new_in_memory().expect("in-memory db");
    let app = create_app(db.clone());
    (db, app)
}

#[tokio::test]
async fn login_page_links_to_lost_password() {
    let (_db, app) = setup();
    let reply = get(&app, "/login").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.body.contains(r#"href="/login/lost-password""#));
    assert!(reply.body.contains("Lost your password?"));
}

#[tokio::test]
async fn lost_password_screens_are_public() {
    let (_db, app) = setup();
    let reply = get(&app, "/login/lost-password").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.body.contains(r#"name="user_login""#));
    assert!(reply.body.contains(r#"action="/login/lost-password""#));

    // An unknown token renders the explanation page, not a redirect to /login.
    let reply = get(&app, "/login/reset-password?token=not-a-token").await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.body.contains("invalid or has expired"));
    assert!(reply.body.contains(r#"href="/login/lost-password""#));
}

#[tokio::test]
async fn lost_password_answers_the_same_for_known_and_unknown_accounts() {
    let (db, app) = setup();
    let known = post_form(&app, "/login/lost-password", "user_login=admin").await;
    let unknown = post_form(
        &app,
        "/login/lost-password",
        "user_login=nobody%40example.com",
    )
    .await;
    assert_eq!(known.status, StatusCode::OK);
    assert_eq!(known.status, unknown.status);
    assert_eq!(known.body, unknown.body);
    assert!(known.body.contains("If an account matches"));
    assert!(!known.body.contains("nobody"));

    // Only the real account got a token, stored as a SHA-256 hex digest.
    let rows = db.list_password_reset_tokens(1).expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].token_hash.len(), 64);
    assert!(rows[0].token_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // One delivery attempt, for the admin's address. Without SMTP it is logged as not sent.
    let logs = email_logs_after_delivery(&db, 1).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let logs_later = db.get_email_logs(50).expect("email logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs_later.len(), 1);
    assert_eq!(logs[0].recipient, "admin@example.com");
    assert!(logs[0].subject.contains("Password reset"));
    if std::env::var("DAYPRESS_SMTP_HOST").is_err() {
        assert_eq!(logs[0].status, "not_sent");
        assert!(known.body.contains("cannot send email yet"));
    }
}

#[tokio::test]
async fn reset_link_works_once_and_the_new_password_logs_in() {
    let (db, app) = setup();
    let token = db.create_password_reset_token(1).expect("token");
    let other_token = db.create_password_reset_token(1).expect("second token");

    let reply = get(&app, &format!("/login/reset-password?token={token}")).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply
        .body
        .contains(&format!(r#"name="token" value="{token}""#)));
    assert_eq!(reply.headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(reply.headers[header::REFERRER_POLICY], "no-referrer");

    // Validation errors keep the link usable.
    let reply = post_form(
        &app,
        "/login/reset-password",
        &reset_form(&token, NEW_PASSWORD, "something-else"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.body.contains("do not match"));
    let reply = post_form(
        &app,
        "/login/reset-password",
        &reset_form(&token, "short", "short"),
    )
    .await;
    assert!(reply.body.contains("at least 8 characters"));

    let reply = post_form(
        &app,
        "/login/reset-password",
        &reset_form(&token, NEW_PASSWORD, NEW_PASSWORD),
    )
    .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
    assert_eq!(reply.headers[header::LOCATION], "/login?action=resetpass");
    let reply = get(&app, "/login?action=resetpass").await;
    assert!(reply.body.contains("Your password has been reset"));

    // The new password logs in through /login; the old one no longer does.
    let reply = post_form(
        &app,
        "/login",
        &format!("log=admin&pwd={NEW_PASSWORD}&redirect_to=/d-admin/"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
    assert!(reply.headers.contains_key(header::SET_COOKIE));
    let reply = post_form(&app, "/login", "log=admin&pwd=admin").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply
        .body
        .contains("Unknown username or incorrect password"));

    // Second use fails, and the user's other outstanding link was cancelled too.
    let reply = post_form(
        &app,
        "/login/reset-password",
        &reset_form(&token, "Another-Pass-99", "Another-Pass-99"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.body.contains("invalid or has expired"));
    let reply = get(&app, &format!("/login/reset-password?token={other_token}")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(db
        .list_password_reset_tokens(1)
        .expect("rows")
        .iter()
        .all(|row| row.used_at.is_some()));
    assert!(db
        .authenticate_credentials("admin", NEW_PASSWORD)
        .expect("check")
        .is_some());
}

#[tokio::test]
async fn expired_reset_links_are_rejected() {
    let (db, app) = setup();
    let token = db
        .create_password_reset_token_with_ttl(1, -60)
        .expect("expired token");

    let reply = get(&app, &format!("/login/reset-password?token={token}")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.body.contains("invalid or has expired"));

    let reply = post_form(
        &app,
        "/login/reset-password",
        &reset_form(&token, NEW_PASSWORD, NEW_PASSWORD),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(db
        .authenticate_credentials("admin", "admin")
        .expect("check")
        .is_some());
}

#[tokio::test]
async fn lost_password_requests_are_rate_limited_without_revealing_accounts() {
    let (db, app) = setup();
    for name in ["admin", "nobody"] {
        for _ in 0..5 {
            let reply =
                post_form(&app, "/login/lost-password", &format!("user_login={name}")).await;
            assert_eq!(reply.status, StatusCode::OK);
        }
        let reply = post_form(&app, "/login/lost-password", &format!("user_login={name}")).await;
        assert_eq!(reply.status, StatusCode::TOO_MANY_REQUESTS);
        assert!(reply.body.contains("Too many password reset requests"));
    }
    // The throttled request created no token, and logins have their own counter.
    assert_eq!(db.list_password_reset_tokens(1).expect("rows").len(), 5);
    let reply = post_form(&app, "/login", "log=admin&pwd=admin").await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn user_input_is_escaped_and_never_echoed() {
    let (_db, app) = setup();
    let reply = get(
        &app,
        "/login/reset-password?token=%3Cscript%3Ealert(1)%3C%2Fscript%3E",
    )
    .await;
    assert!(!reply.body.contains("<script>"));

    let reply = post_form(
        &app,
        "/login/lost-password",
        "user_login=%3Cscript%3Ealert(1)%3C%2Fscript%3E",
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(!reply.body.contains("<script>"));
    assert!(!reply.body.contains("&lt;script&gt;"));

    let reply = post_form(&app, "/login/lost-password", "user_login=+").await;
    assert!(reply.body.contains("Enter a username or email address"));
}
