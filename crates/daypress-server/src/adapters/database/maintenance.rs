//! Offline maintenance for the `daypress` CLI (`daypress reset-password`): open an existing
//! database without seeding it, and get back into an account after losing its password.

use super::password_resets::{lock, revoke_tokens_with_conn};
use super::Database;
use crate::errors::AppError;
use chrono::Utc;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What [`Database::reset_user_password`] changed.
#[derive(Debug, Clone)]
pub struct PasswordResetReport {
    pub user_id: i64,
    pub login: String,
    /// The generated password when the caller supplied none. Show it once; it is not stored.
    pub generated_password: Option<String>,
    /// Unused "Lost your password?" links that were cancelled.
    pub cancelled_reset_links: usize,
}

impl Database {
    /// Opens an existing Daypress database for maintenance. Unlike [`Database::new`] it never
    /// creates a file, seeds the starter site or generates admin passwords; the only schema
    /// change it makes is adding the password reset table when it is missing.
    pub fn open_existing(path: &Path) -> Result<Self, AppError> {
        if !path.is_file() {
            return Err(AppError::NotFound(format!(
                "no database file at {} (pass --db <path> or set DATABASE_PATH)",
                path.display()
            )));
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        // The server may be running against the same file; wait for its writes to finish.
        conn.busy_timeout(Duration::from_secs(10))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let has_users = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'users'",
                [],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !has_users {
            return Err(AppError::BadRequest(format!(
                "{} is not a Daypress database (it has no users table)",
                path.display()
            )));
        }
        crate::legacy::upgrade_schema(&conn)?;
        conn.execute_batch(super::ddl::PASSWORD_RESETS_SCHEMA_SQL)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: path.to_path_buf(),
            login_throttle: Default::default(),
            password_reset_throttle: Default::default(),
        })
    }

    /// Sets a new argon2id password for the account with this login or email address and
    /// cancels its unused reset links. With `new_password: None` a random password is
    /// generated and returned in the report.
    ///
    /// Login sessions are signed tokens without server-side state, so sessions that already
    /// exist stay valid until they expire; [`Self::rotate_auth_secret`] ends all of them.
    pub fn reset_user_password(
        &self,
        identifier: &str,
        new_password: Option<&str>,
    ) -> Result<PasswordResetReport, AppError> {
        let identifier = identifier.trim();
        if let Some(password) = new_password {
            crate::password_reset::check_password_policy(password).map_err(AppError::BadRequest)?;
        }
        let account = self
            .find_account_for_password_reset(identifier)?
            .ok_or_else(|| {
                AppError::NotFound(format!("no user with login or email '{identifier}'"))
            })?;

        let (password, generated_password) = match new_password {
            Some(password) => (password.to_string(), None),
            None => {
                let generated = crate::auth::generate_random_password();
                (generated.clone(), Some(generated))
            }
        };
        // argon2 is deliberately slow: hash before taking the connection lock.
        let pass_hash = crate::auth::hash_user_password(&password);

        let mut conn = lock(self)?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "UPDATE users SET user_pass = ?1 WHERE ID = ?2",
            params![pass_hash, account.user_id],
        )?;
        let cancelled_reset_links =
            revoke_tokens_with_conn(&tx, account.user_id, Utc::now().timestamp())?;
        tx.commit()?;

        Ok(PasswordResetReport {
            user_id: account.user_id,
            login: account.login,
            generated_password,
            cancelled_reset_links,
        })
    }

    /// Replaces the key that signs login sessions, REST nonces and JWTs, which signs out
    /// every user at once (application passwords keep working).
    pub fn rotate_auth_secret(&self) -> Result<(), AppError> {
        let conn = lock(self)?;
        conn.execute(
            r#"INSERT INTO options (option_name, option_value) VALUES ('daypress_auth_secret', ?1)
               ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value"#,
            params![crate::auth::random_hex_64()],
        )?;
        Ok(())
    }
}
