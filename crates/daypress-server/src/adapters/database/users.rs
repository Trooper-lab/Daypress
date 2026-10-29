#![allow(unused_imports)]
use super::Database;
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

impl Database {
    pub fn get_user_detail(&self, id: i64) -> Result<Option<WpUserDetail>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT u.ID, u.user_login, u.display_name, u.user_email, u.user_url, u.user_registered,
                      COALESCE((SELECT meta_value FROM usermeta WHERE user_id = u.ID AND meta_key = 'role'), 'Administrator') as role
               FROM users u WHERE u.ID = ?1"#
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            let id: i64 = row.get(0)?;
            let username: String = row.get(1)?;
            let display_name: String = row.get(2)?;
            let email: String = row.get(3)?;
            let url: String = row.get(4)?;
            let registered: String = row.get(5)?;
            let role: String = row.get(6)?;

            Ok(WpUserDetail {
                id,
                username,
                display_name,
                email,
                url,
                registered: registered.replace(' ', "T"),
                role,
            })
        })?;

        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    pub fn get_all_users(&self) -> Result<Vec<UserListItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT u.ID, u.user_login, u.display_name, u.user_email, u.user_url, u.user_registered,
                      COALESCE((SELECT meta_value FROM usermeta WHERE user_id = u.ID AND meta_key = 'role'), 'Administrator') as role,
                      (SELECT count(*) FROM posts WHERE post_author = u.ID AND post_type = 'post' AND post_status = 'publish') as post_count
               FROM users u
               ORDER BY u.ID ASC"#
        )?;

        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let username: String = row.get(1)?;
            let display_name: String = row.get(2)?;
            let email: String = row.get(3)?;
            let url: String = row.get(4)?;
            let registered: String = row.get(5)?;
            let role: String = row.get(6)?;
            let post_count: i64 = row.get(7)?;

            Ok(UserListItem {
                id,
                username,
                display_name,
                email,
                url,
                registered: registered.replace(' ', "T"),
                role,
                post_count,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn create_user(&self, req: &CreateUserRequest) -> Result<WpUserDetail, AppError> {
        let username = req.username.trim();
        let email = req.email.trim();
        if username.is_empty() {
            return Err(AppError::BadRequest("Username is required".to_string()));
        }
        if email.is_empty() {
            return Err(AppError::BadRequest("Email is required".to_string()));
        }

        let mut conn = self.conn.lock().unwrap();
        let exists: Option<i64> = conn
            .query_row(
                "SELECT ID FROM users WHERE user_login = ?1 OR user_email = ?2",
                params![username, email],
                |r| r.get(0),
            )
            .ok();

        if exists.is_some() {
            return Err(AppError::BadRequest(
                "A user with that username or email already exists".to_string(),
            ));
        }

        let display_name = req
            .display_name
            .as_deref()
            .or(req.name.as_deref())
            .unwrap_or(username);
        let nicename = generated_slugify::slugify(username);
        let url = req.url.as_deref().unwrap_or("");
        let role = req.role.as_deref().unwrap_or("Subscriber");
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let pass_hash = crate::auth::hash_user_password(&req.password);

        let tx = conn.transaction()?;
        tx.execute(
            r#"INSERT INTO users (user_login, user_pass, user_nicename, user_email, user_url, user_registered, display_name)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![username, pass_hash, nicename, email, url, now, display_name],
        )?;

        let id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO usermeta (user_id, meta_key, meta_value) VALUES (?1, 'role', ?2)",
            params![id, role],
        )?;

        if let Some(fn_str) = &req.first_name {
            let _ = tx.execute(
                "INSERT INTO usermeta (user_id, meta_key, meta_value) VALUES (?1, 'first_name', ?2)",
                params![id, fn_str],
            );
        }
        if let Some(ln_str) = &req.last_name {
            let _ = tx.execute(
                "INSERT INTO usermeta (user_id, meta_key, meta_value) VALUES (?1, 'last_name', ?2)",
                params![id, ln_str],
            );
        }

        tx.commit()?;

        Ok(WpUserDetail {
            id,
            username: username.to_string(),
            display_name: display_name.to_string(),
            email: email.to_string(),
            url: url.to_string(),
            registered: now.replace(' ', "T"),
            role: role.to_string(),
        })
    }

    pub fn delete_user(&self, id: i64) -> Result<bool, AppError> {
        if id == 1 {
            return Err(AppError::Forbidden(
                "Cannot delete the primary administrator account".to_string(),
            ));
        }

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // Reassign posts authored by this user to admin (1)
        let _ = tx.execute(
            "UPDATE posts SET post_author = 1 WHERE post_author = ?1",
            params![id],
        );

        let _ = tx.execute(
            "DELETE FROM application_passwords WHERE user_id = ?1",
            params![id],
        );
        let _ = tx.execute("DELETE FROM usermeta WHERE user_id = ?1", params![id]);
        let rows = tx.execute("DELETE FROM users WHERE ID = ?1", params![id])?;

        tx.commit()?;
        Ok(rows > 0)
    }

    pub fn update_user_detail(
        &self,
        id: i64,
        req: &UpdateUserRequest,
    ) -> Result<Option<WpUserDetail>> {
        let conn = self.conn.lock().unwrap();

        if let Some(display_name) = req.display_name.as_ref().or(req.name.as_ref()) {
            conn.execute(
                "UPDATE users SET display_name = ?1, user_nicename = ?2 WHERE ID = ?3",
                params![display_name, generated_slugify::slugify(display_name), id],
            )?;
        }
        if let Some(email) = &req.email {
            conn.execute(
                "UPDATE users SET user_email = ?1 WHERE ID = ?2",
                params![email, id],
            )?;
        }
        if let Some(url) = &req.url {
            conn.execute(
                "UPDATE users SET user_url = ?1 WHERE ID = ?2",
                params![url, id],
            )?;
        }
        if let Some(pass) = &req.password {
            if !pass.trim().is_empty() {
                let pass_hash = crate::auth::hash_user_password(pass.trim());
                conn.execute(
                    "UPDATE users SET user_pass = ?1 WHERE ID = ?2",
                    params![pass_hash, id],
                )?;
            }
        }
        if let Some(role) = &req.role {
            if !role.trim().is_empty() {
                let _ = conn.execute(
                    "DELETE FROM usermeta WHERE user_id = ?1 AND meta_key = 'role'",
                    params![id],
                );
                let _ = conn.execute(
                    "INSERT INTO usermeta (user_id, meta_key, meta_value) VALUES (?1, 'role', ?2)",
                    params![id, role.trim()],
                );
            }
        }

        drop(conn);
        self.get_user_detail(id)
    }

    // --- Authentication & Application Passwords ---

    pub fn get_auth_secret(&self) -> Result<String> {
        // Legacy installs shipped with this public constant; anything signed with it is forgeable,
        // so it is rotated to a per-install random secret the first time it is read.
        const LEGACY_PUBLIC_SECRET: &str = "daypress_secret_key_change_in_production_f94b8e231";
        let conn = self.conn.lock().unwrap();
        let secret_opt: Option<String> = conn
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'daypress_auth_secret'",
                [],
                |r| r.get(0),
            )
            .ok();

        if let Some(s) = secret_opt {
            if s != LEGACY_PUBLIC_SECRET && !s.trim().is_empty() {
                return Ok(s);
            }
        }

        let fresh_secret = crate::auth::random_hex_64();
        let _ = conn.execute(
            "DELETE FROM options WHERE option_name = 'daypress_auth_secret'",
            [],
        );
        let _ = conn.execute(
            "INSERT INTO options (option_name, option_value) VALUES ('daypress_auth_secret', ?1)",
            params![fresh_secret],
        );
        Ok(fresh_secret)
    }

    pub fn verify_user_credentials(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<AuthUser>> {
        self.authenticate_credentials(username, password)
    }

    pub fn authenticate_credentials(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<AuthUser>> {
        // Look the account up, then release the connection: argon2 verification is
        // deliberately slow and must not hold the global SQLite lock.
        let account: Option<(i64, String, String, String)> = {
            let conn = self.conn.lock().unwrap();
            let user_row: Option<(i64, String, String)> = conn
                .query_row(
                    "SELECT ID, user_login, user_pass FROM users WHERE user_login = ?1 OR user_email = ?1",
                    params![username],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();
            user_row.map(|(user_id, login, stored_pass)| {
                let role: String = conn
                    .query_row(
                        "SELECT meta_value FROM usermeta WHERE user_id = ?1 AND meta_key = 'role'",
                        params![user_id],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| "administrator".to_string());
                (user_id, login, stored_pass, role)
            })
        };
        let Some((user_id, login, stored_pass, role)) = account else {
            return Ok(None);
        };

        if crate::auth::verify_user_password(password, &stored_pass) {
            // Transparently upgrade pre-argon2 hashes now that the plaintext is known.
            if crate::auth::password_needs_rehash(&stored_pass) {
                let upgraded = crate::auth::hash_user_password(password);
                let conn = self.conn.lock().unwrap();
                let _ = conn.execute(
                    "UPDATE users SET user_pass = ?1 WHERE ID = ?2",
                    params![upgraded, user_id],
                );
            }
            return Ok(Some(AuthUser {
                user_id,
                username: login,
                role,
            }));
        }

        // Check application passwords for this user (random 16-char tokens, stored as SHA-256)
        let conn = self.conn.lock().unwrap();
        let norm_input = crate::auth::normalize_app_password(password);
        let input_hash = crate::auth::sha256_hex(norm_input.as_bytes());
        let mut stmt = conn
            .prepare("SELECT uuid, password_hash FROM application_passwords WHERE user_id = ?1")?;
        let rows = stmt.query_map(params![user_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for r in rows {
            let (uuid, hash) = r?;
            if input_hash == hash {
                let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = conn.execute(
                    "UPDATE application_passwords SET last_used = ?1 WHERE uuid = ?2",
                    params![now, uuid],
                );
                return Ok(Some(AuthUser {
                    user_id,
                    username: login,
                    role,
                }));
            }
        }

        Ok(None)
    }

    /// [`Self::authenticate_credentials`] behind the failed-login throttle: after
    /// [`crate::auth::MAX_FAILED_LOGINS`] failures for `client` + `username` within the window,
    /// attempts are refused (even with the right password) until the window passes.
    pub fn attempt_login(
        &self,
        username: &str,
        password: &str,
        client: Option<&str>,
    ) -> Result<crate::auth::LoginAttempt> {
        use crate::auth::{LoginAttempt, LoginThrottle};
        let key = LoginThrottle::key(client, username);
        if let Some(retry_after_secs) = self.login_throttle.retry_after(&key) {
            return Ok(LoginAttempt::Throttled { retry_after_secs });
        }
        match self.authenticate_credentials(username, password)? {
            Some(user) => {
                self.login_throttle.record_success(&key);
                Ok(LoginAttempt::Success(user))
            }
            None => {
                self.login_throttle.record_failure(&key);
                Ok(LoginAttempt::Invalid)
            }
        }
    }

    pub fn create_application_password(
        &self,
        user_id: i64,
        name: &str,
        app_id: Option<&str>,
    ) -> Result<AppPasswordResponse> {
        let conn = self.conn.lock().unwrap();
        let plain = crate::auth::generate_application_password();
        let norm = crate::auth::normalize_app_password(&plain);
        let hash = crate::auth::sha256_hex(norm.as_bytes());
        let uuid = format!(
            "{:016x}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let app_id_str = app_id.unwrap_or("daypress-client").to_string();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            r#"INSERT INTO application_passwords (uuid, app_id, user_id, app_name, password_hash, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6);"#,
            params![uuid, app_id_str, user_id, name, hash, now],
        )?;

        Ok(AppPasswordResponse {
            uuid,
            app_id: app_id_str,
            name: name.to_string(),
            password: Some(plain),
            created: now,
            last_used: None,
        })
    }

    pub fn list_application_passwords(&self, user_id: i64) -> Result<Vec<AppPasswordResponse>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT uuid, app_id, app_name, created_at, last_used FROM application_passwords WHERE user_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![user_id], |r| {
            Ok(AppPasswordResponse {
                uuid: r.get(0)?,
                app_id: r.get(1)?,
                name: r.get(2)?,
                password: None,
                created: r.get(3)?,
                last_used: r.get(4)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_application_password(&self, user_id: i64, uuid: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM application_passwords WHERE user_id = ?1 AND uuid = ?2",
            params![user_id, uuid],
        )?;
        Ok(rows > 0)
    }

    pub fn export_personal_data(&self, identifier: &str) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let user: Option<(i64, String, String, String)> = conn
            .query_row(
                "SELECT ID, user_login, user_email, user_registered FROM users WHERE user_login = ?1 OR user_email = ?1",
                params![identifier],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .ok();

        let comments: Vec<WpCommentAdminItem> = conn
            .prepare("SELECT c.comment_ID, c.comment_post_ID, COALESCE(p.post_title, 'Post'), c.comment_author, c.comment_author_email, c.comment_author_url, c.comment_author_IP, c.comment_content, c.comment_date, c.comment_approved FROM comments c LEFT JOIN posts p ON c.comment_post_ID = p.ID WHERE c.comment_author_email = ?1 OR c.comment_author = ?1")
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![identifier], |r| {
                    Ok(WpCommentAdminItem {
                        id: r.get(0)?,
                        post_id: r.get(1)?,
                        post_title: r.get(2)?,
                        author: r.get(3)?,
                        author_email: r.get(4)?,
                        author_url: r.get(5)?,
                        author_ip: r.get(6)?,
                        content: r.get(7)?,
                        date: r.get(8)?,
                        status: r.get(9)?,
                    })
                })?;
                let mut v = Vec::new();
                for row in rows {
                    v.push(row?);
                }
                Ok(v)
            })
            .unwrap_or_default();

        Ok(serde_json::json!({
            "export_type": "personal_data_archive",
            "requested_identifier": identifier,
            "user_record": user.map(|(id, login, email, reg)| {
                serde_json::json!({
                    "id": id,
                    "username": login,
                    "email": email,
                    "registered": reg
                })
            }),
            "comments": comments
        }))
    }

    pub fn erase_personal_data(&self, identifier: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let modified = conn.execute(
            "UPDATE comments SET comment_author = 'Anonymous', comment_author_email = 'deleted@privacy.local', comment_author_IP = '0.0.0.0' WHERE comment_author_email = ?1 OR comment_author = ?1",
            params![identifier],
        )?;
        Ok(modified)
    }
}
