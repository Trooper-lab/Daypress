//! Library-level tests for getting back into an account: `Database::reset_user_password`
//! (behind `neopress reset-password`), the CLI entry point on a real database file, and the
//! token side of the "Lost your password?" flow.

use neopress_server::auth::{sha256_hex, verify_user_password};
use neopress_server::password_reset::{
    self, check_password_policy, request_password_reset, reset_link, reset_password_from_cli,
    validate_new_password, ResetRequestOutcome, MAX_RESET_EMAILS_PER_ACCOUNT_PER_HOUR,
};
use neopress_server::{AppError, CreateUserRequest, Database};
use std::path::{Path, PathBuf};

const NEW_PASSWORD: &str = "Correct-Horse-42";

fn create_user(db: &Database, username: &str, email: &str) -> i64 {
    let req: CreateUserRequest = serde_json::from_value(serde_json::json!({
        "username": username,
        "email": email,
        "password": NEW_PASSWORD,
    }))
    .expect("user request");
    db.create_user(&req).expect("create user").id
}

fn logs_in(db: &Database, login: &str, password: &str) -> bool {
    db.authenticate_credentials(login, password)
        .expect("credential check")
        .is_some()
}

#[test]
fn reset_with_given_password_replaces_the_old_one() {
    let db = Database::new_in_memory().expect("in-memory db");
    assert!(logs_in(&db, "admin", "admin"));

    let report = db
        .reset_user_password("admin", Some(NEW_PASSWORD))
        .expect("reset");
    assert_eq!(report.user_id, 1);
    assert_eq!(report.login, "admin");
    assert!(report.generated_password.is_none());

    assert!(logs_in(&db, "admin", NEW_PASSWORD));
    assert!(!logs_in(&db, "admin", "admin"));
}

#[test]
fn reset_without_password_generates_one_and_accepts_the_email_address() {
    let db = Database::new_in_memory().expect("in-memory db");
    let report = db
        .reset_user_password("ADMIN@example.com", None)
        .expect("reset by email");
    assert_eq!(report.login, "admin");
    let generated = report.generated_password.expect("generated password");
    assert_eq!(generated.chars().count(), 20);
    assert!(logs_in(&db, "admin", &generated));
    assert!(!logs_in(&db, "admin", "admin"));
}

#[test]
fn reset_rejects_unknown_users_and_short_passwords() {
    let db = Database::new_in_memory().expect("in-memory db");
    assert!(matches!(
        db.reset_user_password("nobody", Some(NEW_PASSWORD)),
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        db.reset_user_password("", None),
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        db.reset_user_password("admin", Some("short")),
        Err(AppError::BadRequest(_))
    ));
    // Nothing changed.
    assert!(logs_in(&db, "admin", "admin"));
}

#[test]
fn reset_cancels_outstanding_reset_links() {
    let db = Database::new_in_memory().expect("in-memory db");
    let token = db.create_password_reset_token(1).expect("token");
    assert_eq!(
        db.password_reset_token_user(&token).expect("lookup"),
        Some(1)
    );

    let report = db
        .reset_user_password("admin", Some(NEW_PASSWORD))
        .expect("reset");
    assert_eq!(report.cancelled_reset_links, 1);
    assert_eq!(db.password_reset_token_user(&token).expect("lookup"), None);
}

#[test]
fn password_policy() {
    assert!(check_password_policy("12345678").is_ok());
    assert!(check_password_policy("ünïcødé!").is_ok());
    assert!(check_password_policy("1234567").is_err());
    assert!(check_password_policy(" 12345678").is_err());
    assert!(check_password_policy("12345678 ").is_err());
    assert!(validate_new_password(NEW_PASSWORD, NEW_PASSWORD).is_ok());
    assert!(validate_new_password(NEW_PASSWORD, "Correct-Horse-43").is_err());
}

#[test]
fn reset_request_stores_only_the_token_hash_and_emails_the_link() {
    let db = Database::new_in_memory().expect("in-memory db");
    db.set_option("siteurl", "https://cms.example.com/")
        .expect("set siteurl");
    let outcome = request_password_reset(&db, "admin", Some("192.0.2.10")).expect("request");
    let ResetRequestOutcome::Accepted(Some(email)) = outcome else {
        panic!("expected an email for the admin account, got {outcome:?}");
    };
    assert_eq!(email.user_id, 1);
    assert_eq!(email.message.to, "admin@example.com");
    let link = format!(
        "https://cms.example.com/login/reset-password?token={}",
        email.token
    );
    assert_eq!(reset_link(&db, &email.token), link);
    assert!(email.message.body_text.contains(&link));
    assert!(email.message.body_html.contains(&link));

    let rows = db.list_password_reset_tokens(1).expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].token_hash, sha256_hex(email.token.as_bytes()));
    assert_ne!(rows[0].token_hash, email.token);
    assert_eq!(
        rows[0].expires_at - rows[0].created_at,
        password_reset::PASSWORD_RESET_TTL_SECS
    );
    assert!(rows[0].used_at.is_none());
}

#[test]
fn reset_request_for_unknown_or_emailless_accounts_sends_nothing() {
    let db = Database::new_in_memory().expect("in-memory db");
    let outcome = request_password_reset(&db, "nobody@example.com", None).expect("request");
    assert!(matches!(outcome, ResetRequestOutcome::Accepted(None)));

    let user_id = create_user(&db, "no-mail", "placeholder@example.com");
    db.update_user_detail(
        user_id,
        &neopress_server::UpdateUserRequest {
            email: Some(String::new()),
            ..Default::default()
        },
    )
    .expect("clear email");
    let outcome = request_password_reset(&db, "no-mail", None).expect("request");
    assert!(matches!(outcome, ResetRequestOutcome::Accepted(None)));
    assert!(db
        .list_password_reset_tokens(user_id)
        .expect("rows")
        .is_empty());
}

#[test]
fn each_account_gets_a_limited_number_of_reset_emails_per_hour() {
    let db = Database::new_in_memory().expect("in-memory db");
    // Different clients, so the per-client throttle never applies.
    for i in 0..MAX_RESET_EMAILS_PER_ACCOUNT_PER_HOUR {
        let client = format!("198.51.100.{i}");
        let outcome = request_password_reset(&db, "admin", Some(&client)).expect("request");
        assert!(matches!(outcome, ResetRequestOutcome::Accepted(Some(_))));
    }
    let outcome = request_password_reset(&db, "admin", Some("203.0.113.99")).expect("request");
    assert!(matches!(outcome, ResetRequestOutcome::Accepted(None)));
    assert_eq!(
        db.list_password_reset_tokens(1).expect("rows").len() as i64,
        MAX_RESET_EMAILS_PER_ACCOUNT_PER_HOUR
    );
}

#[test]
fn deleting_a_user_removes_its_reset_links() {
    let db = Database::new_in_memory().expect("in-memory db");
    let user_id = create_user(&db, "leaving", "leaving@example.com");
    let token = db.create_password_reset_token(user_id).expect("token");
    assert!(db.delete_user(user_id).expect("delete"));
    assert_eq!(db.password_reset_token_user(&token).expect("lookup"), None);
    assert!(db
        .list_password_reset_tokens(user_id)
        .expect("rows")
        .is_empty());
}

/// A scratch directory under the system temp dir, removed again on drop (even on panic).
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let dir =
            std::env::temp_dir().join(format!("neopress-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        TempDir(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn stored_hash(path: &Path, login: &str) -> String {
    let conn = rusqlite::Connection::open(path).expect("open db file");
    conn.query_row(
        "SELECT user_pass FROM wp_users WHERE user_login = ?1",
        rusqlite::params![login],
        |r| r.get(0),
    )
    .expect("user row")
}

#[test]
fn cli_reset_works_on_an_existing_database_file_and_never_creates_one() {
    let dir = TempDir::new("reset-cli");
    let db_path = dir.path("neopress.sqlite");

    let missing = dir.path("missing.sqlite");
    assert!(matches!(
        reset_password_from_cli(&missing, "admin", Some(NEW_PASSWORD), false),
        Err(AppError::NotFound(_))
    ));
    assert!(!missing.exists(), "the CLI must not create a database");

    let empty = dir.path("empty.sqlite");
    std::fs::write(&empty, b"").expect("write empty file");
    assert!(matches!(
        reset_password_from_cli(&empty, "admin", Some(NEW_PASSWORD), false),
        Err(AppError::BadRequest(_))
    ));

    // A regular install (as `neopress` creates it on first start).
    let secret_before = {
        let db = Database::new(&db_path).expect("create file db");
        db.get_auth_secret().expect("secret")
    };

    let report =
        reset_password_from_cli(&db_path, "admin", Some(NEW_PASSWORD), false).expect("reset");
    assert_eq!(report.login, "admin");
    assert!(report.generated_password.is_none());
    let hash = stored_hash(&db_path, "admin");
    assert!(hash.starts_with("$argon2id$"));
    assert!(verify_user_password(NEW_PASSWORD, &hash));
    let secret_after_reset = Database::open_existing(&db_path)
        .expect("reopen")
        .get_auth_secret()
        .expect("secret");
    assert_eq!(secret_before, secret_after_reset);

    assert!(matches!(
        reset_password_from_cli(&db_path, "nobody", Some(NEW_PASSWORD), false),
        Err(AppError::NotFound(_))
    ));

    // Generated password + signing everyone out.
    let report = reset_password_from_cli(&db_path, "admin", None, true).expect("reset");
    let generated = report.generated_password.expect("generated password");
    let hash = stored_hash(&db_path, "admin");
    assert!(verify_user_password(&generated, &hash));
    assert!(!verify_user_password(NEW_PASSWORD, &hash));
    let secret_after_sign_out = Database::open_existing(&db_path)
        .expect("reopen")
        .get_auth_secret()
        .expect("secret");
    assert_ne!(secret_before, secret_after_sign_out);
}
