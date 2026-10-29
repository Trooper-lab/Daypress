//! Single-use password reset tokens for the "Lost your password?" flow
//! (`daypress_password_resets`, see [`super::ddl::PASSWORD_RESETS_SCHEMA_SQL`]).
//!
//! The plaintext token only exists in the emailed link; the table keeps its SHA-256, so the
//! lookup by hash needs no constant-time comparison of secrets.

use super::Database;
use crate::errors::AppError;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::sync::MutexGuard;

/// How long a reset link stays valid.
pub const PASSWORD_RESET_TTL_SECS: i64 = 60 * 60;

/// Most reset emails one account receives per hour, however many clients ask. This bounds
/// email bombing from many addresses; the per-client limit is the reset request throttle.
pub const MAX_RESET_EMAILS_PER_ACCOUNT_PER_HOUR: i64 = 5;

/// The account a reset request or `daypress reset-password` refers to.
#[derive(Debug, Clone)]
pub struct PasswordResetAccount {
    pub user_id: i64,
    pub login: String,
    pub email: String,
    pub display_name: String,
}

/// One stored reset token. Only the hash is kept.
#[derive(Debug, Clone)]
pub struct PasswordResetTokenRow {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub used_at: Option<i64>,
}

/// Locks the shared connection without panicking on a poisoned mutex.
pub(super) fn lock(db: &Database) -> Result<MutexGuard<'_, Connection>, AppError> {
    db.conn
        .lock()
        .map_err(|_| AppError::Internal("database connection lock poisoned".to_string()))
}

/// Lowercase form of a token taken from a reset link, or `None` when it cannot be one of
/// ours (tokens are 64 hex characters).
fn normalize_token(raw: &str) -> Option<String> {
    let token = raw.trim();
    (token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit()))
        .then(|| token.to_ascii_lowercase())
}

fn hash_token(token: &str) -> String {
    crate::auth::sha256_hex(token.as_bytes())
}

/// Login (exact match first, then case-insensitive) or email address (case-insensitive).
fn find_account_with_conn(
    conn: &Connection,
    identifier: &str,
) -> rusqlite::Result<Option<PasswordResetAccount>> {
    conn.query_row(
        r#"SELECT ID, user_login, COALESCE(user_email, ''), COALESCE(display_name, '')
           FROM users
           WHERE user_login = ?1 COLLATE NOCASE
              OR (TRIM(COALESCE(user_email, '')) <> '' AND TRIM(user_email) = ?1 COLLATE NOCASE)
           ORDER BY (user_login = ?1) DESC, (user_login = ?1 COLLATE NOCASE) DESC, ID ASC
           LIMIT 1"#,
        params![identifier],
        |r| {
            Ok(PasswordResetAccount {
                user_id: r.get(0)?,
                login: r.get(1)?,
                email: r.get::<_, String>(2)?.trim().to_string(),
                display_name: r.get(3)?,
            })
        },
    )
    .optional()
}

/// Cancels every unused, unexpired reset token of `user_id`; returns how many were cancelled.
pub(super) fn revoke_tokens_with_conn(
    conn: &Connection,
    user_id: i64,
    now: i64,
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE daypress_password_resets SET used_at = ?1
         WHERE user_id = ?2 AND used_at IS NULL AND expires_at > ?1",
        params![now, user_id],
    )
}

fn valid_token_user_with_conn(
    conn: &Connection,
    token_hash: &str,
    now: i64,
) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT user_id FROM daypress_password_resets
         WHERE token_hash = ?1 AND used_at IS NULL AND expires_at > ?2",
        params![token_hash, now],
        |r| r.get(0),
    )
    .optional()
}

impl Database {
    /// Finds the account a "Lost your password?" request names, by login or email address.
    pub fn find_account_for_password_reset(
        &self,
        identifier: &str,
    ) -> Result<Option<PasswordResetAccount>, AppError> {
        let identifier = identifier.trim();
        if identifier.is_empty() {
            return Ok(None);
        }
        let conn = lock(self)?;
        Ok(find_account_with_conn(&conn, identifier)?)
    }

    /// Issues a reset token valid for [`PASSWORD_RESET_TTL_SECS`] and returns its plaintext,
    /// which belongs in the emailed link only. The database stores its SHA-256.
    pub fn create_password_reset_token(&self, user_id: i64) -> Result<String, AppError> {
        self.create_password_reset_token_with_ttl(user_id, PASSWORD_RESET_TTL_SECS)
    }

    /// [`Self::create_password_reset_token`] with a custom lifetime in seconds.
    pub fn create_password_reset_token_with_ttl(
        &self,
        user_id: i64,
        ttl_secs: i64,
    ) -> Result<String, AppError> {
        let token = crate::auth::random_hex_64();
        let now = Utc::now().timestamp();
        let conn = lock(self)?;
        // Keep the table small: forget links that expired more than a day ago.
        conn.execute(
            "DELETE FROM daypress_password_resets WHERE expires_at < ?1",
            params![now - 86_400],
        )?;
        conn.execute(
            "INSERT INTO daypress_password_resets (user_id, token_hash, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                user_id,
                hash_token(&token),
                now,
                now.saturating_add(ttl_secs)
            ],
        )?;
        Ok(token)
    }

    /// Reset tokens issued for `user_id` during the last hour.
    pub fn recent_password_reset_count(&self, user_id: i64) -> Result<i64, AppError> {
        let since = Utc::now().timestamp() - 3600;
        let conn = lock(self)?;
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM daypress_password_resets WHERE user_id = ?1 AND created_at > ?2",
            params![user_id, since],
            |r| r.get(0),
        )?)
    }

    /// The user a reset token belongs to, when it exists, is unused and has not expired.
    pub fn password_reset_token_user(&self, token: &str) -> Result<Option<i64>, AppError> {
        let Some(token) = normalize_token(token) else {
            return Ok(None);
        };
        let conn = lock(self)?;
        Ok(valid_token_user_with_conn(
            &conn,
            &hash_token(&token),
            Utc::now().timestamp(),
        )?)
    }

    /// Sets a new password through a reset token. The token is claimed inside one write
    /// transaction (so it works exactly once), the user gets an argon2id hash of
    /// `new_password`, and the user's other unused tokens are cancelled.
    ///
    /// Returns the user id, or `None` when the token is unknown, used or expired.
    pub fn consume_password_reset_token(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<Option<i64>, AppError> {
        crate::password_reset::check_password_policy(new_password).map_err(AppError::BadRequest)?;
        let Some(token) = normalize_token(token) else {
            return Ok(None);
        };
        if self.password_reset_token_user(&token)?.is_none() {
            return Ok(None);
        }
        // argon2 is deliberately slow: hash before taking the connection lock.
        let pass_hash = crate::auth::hash_user_password(new_password);
        let token_hash = hash_token(&token);
        let now = Utc::now().timestamp();

        let mut conn = lock(self)?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Re-check inside the transaction: a concurrent request may have used it meanwhile.
        let Some(user_id) = valid_token_user_with_conn(&tx, &token_hash, now)? else {
            return Ok(None);
        };
        tx.execute(
            "UPDATE daypress_password_resets SET used_at = ?1 WHERE token_hash = ?2",
            params![now, token_hash],
        )?;
        let updated = tx.execute(
            "UPDATE users SET user_pass = ?1 WHERE ID = ?2",
            params![pass_hash, user_id],
        )?;
        if updated == 0 {
            return Ok(None);
        }
        revoke_tokens_with_conn(&tx, user_id, now)?;
        tx.commit()?;
        Ok(Some(user_id))
    }

    /// Cancels every outstanding reset link of `user_id`; returns how many were cancelled.
    pub fn revoke_password_reset_tokens(&self, user_id: i64) -> Result<usize, AppError> {
        let conn = lock(self)?;
        Ok(revoke_tokens_with_conn(
            &conn,
            user_id,
            Utc::now().timestamp(),
        )?)
    }

    /// Stored reset tokens of `user_id` (hashes only), newest first.
    pub fn list_password_reset_tokens(
        &self,
        user_id: i64,
    ) -> Result<Vec<PasswordResetTokenRow>, AppError> {
        let conn = lock(self)?;
        let mut stmt = conn.prepare(
            "SELECT id, user_id, token_hash, created_at, expires_at, used_at
             FROM daypress_password_resets WHERE user_id = ?1 ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![user_id], |r| {
            Ok(PasswordResetTokenRow {
                id: r.get(0)?,
                user_id: r.get(1)?,
                token_hash: r.get(2)?,
                created_at: r.get(3)?,
                expires_at: r.get(4)?,
                used_at: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
