#![allow(unused_imports)]
use super::Database;
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Maps any stored `admin_color` identifier (including legacy imported values) to its canonical
/// Daypress softer backend theme slug for UI selection.
pub fn canonical_daypress_theme(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "linen" | "light" => "linen",
        "studio" | "modern" => "studio",
        "nordic" | "blue" => "nordic",
        "espresso" | "coffee" => "espresso",
        "sage" | "ectoplasm" => "sage",
        "dusk" | "midnight" => "dusk",
        "harbor" | "ocean" => "harbor",
        "terracotta" | "sunrise" => "terracotta",
        _ => "default",
    }
}

/// Computes a standard Gravatar URL (SHA-256 hex of trimmed lowercase email) with `identicon` fallback.
pub fn gravatar_url_for_email(email: &str, size: u32) -> String {
    let clean = email.trim().to_ascii_lowercase();
    let hash = crate::auth::sha256_hex(clean.as_bytes());
    format!(
        "https://www.gravatar.com/avatar/{}?s={}&d=identicon",
        hash, size
    )
}

fn parse_bool_meta(val: Option<&String>, default: bool) -> bool {
    match val.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("false" | "0" | "no" | "off") => false,
        Some("true" | "1" | "yes" | "on") => true,
        _ => default,
    }
}

impl Database {
    pub(crate) fn read_usermeta_map(conn: &Connection, user_id: i64) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(mut stmt) = conn.prepare_cached(
            "SELECT meta_key, COALESCE(meta_value, '') FROM usermeta WHERE user_id = ?1",
        ) {
            if let Ok(rows) = stmt.query_map(params![user_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            }) {
                for (k, v) in rows.flatten() {
                    map.insert(k, v);
                }
            }
        }
        map
    }

    pub(crate) fn upsert_usermeta(
        conn: &Connection,
        user_id: i64,
        key: &str,
        value: &str,
    ) -> Result<()> {
        conn.execute(
            "DELETE FROM usermeta WHERE user_id = ?1 AND meta_key = ?2",
            params![user_id, key],
        )?;
        conn.execute(
            "INSERT INTO usermeta (user_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
            params![user_id, key, value],
        )?;
        Ok(())
    }

    pub fn get_user_detail(&self, id: i64) -> Result<Option<DaypressUserDetail>> {
        let conn = self.conn.lock().unwrap();
        let base_row: Option<(i64, String, String, String, String, String)> = conn
            .query_row(
                "SELECT u.ID, u.user_login, u.display_name, u.user_email, u.user_url, u.user_registered FROM users u WHERE u.ID = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .ok();

        let Some((id, username, display_name, email, url, registered)) = base_row else {
            return Ok(None);
        };

        let meta = Self::read_usermeta_map(&conn, id);
        let role = meta
            .get("role")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Administrator".to_string());
        let first_name = meta.get("first_name").cloned().unwrap_or_default();
        let last_name = meta.get("last_name").cloned().unwrap_or_default();
        let nickname = meta
            .get("nickname")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| username.clone());
        let description = meta.get("description").cloned().unwrap_or_default();
        let locale = meta.get("locale").cloned().unwrap_or_default();
        let rich_editing = parse_bool_meta(meta.get("rich_editing"), true);
        let syntax_highlighting = parse_bool_meta(meta.get("syntax_highlighting"), true);
        let comment_shortcuts = parse_bool_meta(meta.get("comment_shortcuts"), false);
        let admin_color = meta
            .get("admin_color")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "default".to_string());
        let show_admin_bar_front = parse_bool_meta(meta.get("show_admin_bar_front"), true);
        let custom_avatar = meta.get("daypress_avatar_url").cloned().unwrap_or_default();
        let avatar_url = if custom_avatar.trim().is_empty() {
            gravatar_url_for_email(&email, 96)
        } else {
            custom_avatar
        };
        let session_ver = meta
            .get("daypress_session_ver")
            .and_then(|v| v.trim().parse::<i64>().ok())
            .unwrap_or(0);
        let totp_enabled = parse_bool_meta(meta.get("daypress_2fa_enabled"), false)
            && meta
                .get("daypress_2fa_secret")
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);

        crate::auth::sync_user_ui_prefs(id, show_admin_bar_front, &avatar_url, &admin_color);

        Ok(Some(DaypressUserDetail {
            id,
            username,
            display_name,
            email,
            url,
            registered: registered.replace(' ', "T"),
            role,
            first_name,
            last_name,
            nickname,
            description,
            locale,
            rich_editing,
            syntax_highlighting,
            comment_shortcuts,
            admin_color,
            show_admin_bar_front,
            avatar_url,
            session_ver,
            totp_enabled,
        }))
    }

    pub fn get_all_users(&self) -> Result<Vec<UserListItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT u.ID, u.user_login, u.display_name, u.user_email, u.user_url, u.user_registered,
                      COALESCE((SELECT meta_value FROM usermeta WHERE user_id = u.ID AND meta_key = 'role'), 'Administrator') as role,
                      (SELECT count(*) FROM posts WHERE post_author = u.ID AND post_type = 'post' AND post_status = 'publish') as post_count,
                      COALESCE((SELECT meta_value FROM usermeta WHERE user_id = u.ID AND meta_key = 'daypress_avatar_url'), '') as avatar_url
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
            let raw_avatar: String = row.get(8)?;
            let avatar_url = if raw_avatar.trim().is_empty() {
                gravatar_url_for_email(&email, 64)
            } else {
                raw_avatar
            };

            Ok(UserListItem {
                id,
                username,
                display_name,
                email,
                url,
                registered: registered.replace(' ', "T"),
                role,
                post_count,
                avatar_url,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn create_user(&self, req: &CreateUserRequest) -> Result<DaypressUserDetail, AppError> {
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
            .filter(|s| !s.trim().is_empty())
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

        Self::upsert_usermeta(&tx, id, "role", role)?;
        Self::upsert_usermeta(
            &tx,
            id,
            "nickname",
            req.nickname
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(username),
        )?;

        if let Some(fn_str) = &req.first_name {
            let _ = Self::upsert_usermeta(&tx, id, "first_name", fn_str.trim());
        }
        if let Some(ln_str) = &req.last_name {
            let _ = Self::upsert_usermeta(&tx, id, "last_name", ln_str.trim());
        }
        if let Some(desc) = &req.description {
            let _ = Self::upsert_usermeta(&tx, id, "description", desc.trim());
        }
        if let Some(loc) = &req.locale {
            let _ = Self::upsert_usermeta(&tx, id, "locale", loc.trim());
        }

        tx.commit()?;
        drop(conn);

        Ok(self.get_user_detail(id)?.unwrap_or_default())
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
    ) -> Result<Option<DaypressUserDetail>> {
        let conn = self.conn.lock().unwrap();

        if let Some(display_name) = req.display_name.as_ref().or(req.name.as_ref()) {
            let clean_dn = display_name.trim();
            if !clean_dn.is_empty() {
                conn.execute(
                    "UPDATE users SET display_name = ?1 WHERE ID = ?2",
                    params![clean_dn, id],
                )?;
            }
        }
        if let Some(email) = &req.email {
            conn.execute(
                "UPDATE users SET user_email = ?1 WHERE ID = ?2",
                params![email.trim(), id],
            )?;
        }
        if let Some(url) = &req.url {
            conn.execute(
                "UPDATE users SET user_url = ?1 WHERE ID = ?2",
                params![url.trim(), id],
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
                Self::upsert_usermeta(&conn, id, "role", role.trim())?;
            }
        }
        if let Some(first_name) = &req.first_name {
            Self::upsert_usermeta(&conn, id, "first_name", first_name.trim())?;
        }
        if let Some(last_name) = &req.last_name {
            Self::upsert_usermeta(&conn, id, "last_name", last_name.trim())?;
        }
        if let Some(nickname) = &req.nickname {
            let clean_nick = nickname.trim();
            if !clean_nick.is_empty() {
                Self::upsert_usermeta(&conn, id, "nickname", clean_nick)?;
            }
        }
        if let Some(description) = &req.description {
            Self::upsert_usermeta(&conn, id, "description", description.trim())?;
        }
        if let Some(locale) = &req.locale {
            Self::upsert_usermeta(&conn, id, "locale", locale.trim())?;
        }
        if let Some(rich) = req.rich_editing {
            Self::upsert_usermeta(
                &conn,
                id,
                "rich_editing",
                if rich { "true" } else { "false" },
            )?;
        }
        if let Some(syn) = req.syntax_highlighting {
            Self::upsert_usermeta(
                &conn,
                id,
                "syntax_highlighting",
                if syn { "true" } else { "false" },
            )?;
        }
        if let Some(sc) = req.comment_shortcuts {
            Self::upsert_usermeta(
                &conn,
                id,
                "comment_shortcuts",
                if sc { "true" } else { "false" },
            )?;
        }
        if let Some(color) = &req.admin_color {
            let trimmed = color.trim().to_ascii_lowercase();
            let valid = match trimmed.as_str() {
                "default" | "linen" | "studio" | "nordic" | "espresso" | "sage" | "dusk"
                | "harbor" | "terracotta" | "light" | "modern" | "blue" | "coffee"
                | "ectoplasm" | "midnight" | "ocean" | "sunrise" => trimmed.as_str(),
                _ => "default",
            };
            Self::upsert_usermeta(&conn, id, "admin_color", valid)?;
        }
        if let Some(bar) = req.show_admin_bar_front {
            Self::upsert_usermeta(
                &conn,
                id,
                "show_admin_bar_front",
                if bar { "true" } else { "false" },
            )?;
        }
        if let Some(avatar) = &req.avatar_url {
            let clean_av = avatar.trim();
            // Do not lock in the auto-generated Gravatar fallback URL as a custom avatar override
            let store_av = if clean_av.starts_with("https://www.gravatar.com/avatar/")
                && clean_av.contains("d=identicon")
            {
                ""
            } else {
                clean_av
            };
            Self::upsert_usermeta(&conn, id, "daypress_avatar_url", store_av)?;
        }

        drop(conn);
        self.get_user_detail(id)
    }

    /// Increments the user's `daypress_session_ver` in `usermeta`, invalidating all older session cookies
    /// for this user ("Log Out Everywhere Else"). Returns the new session version.
    pub fn bump_user_session_version(&self, user_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let current: i64 = conn
            .query_row(
                "SELECT meta_value FROM usermeta WHERE user_id = ?1 AND meta_key = 'daypress_session_ver'",
                params![user_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.trim().parse::<i64>().ok())
            .unwrap_or(0);
        let next = current + 1;
        Self::upsert_usermeta(&conn, user_id, "daypress_session_ver", &next.to_string())?;
        Ok(next)
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
        Ok(self
            .authenticate_credentials_with_method(username, password)?
            .map(|(user, _used_app_password)| user))
    }

    /// Authenticates `username` + `password` and returns `(AuthUser, used_application_password)`.
    /// When `used_application_password` is `false`, the primary account password was used.
    pub fn authenticate_credentials_with_method(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<(AuthUser, bool)>> {
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
            return Ok(Some((
                AuthUser {
                    user_id,
                    username: login,
                    role,
                },
                false,
            )));
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
                return Ok(Some((
                    AuthUser {
                        user_id,
                        username: login,
                        role,
                    },
                    true,
                )));
            }
        }

        Ok(None)
    }

    // --- Two-Factor Authentication (2FA / RFC 6238 TOTP) ---

    pub fn is_user_2fa_enabled(&self, user_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let meta = Self::read_usermeta_map(&conn, user_id);
        let enabled = parse_bool_meta(meta.get("daypress_2fa_enabled"), false)
            && meta
                .get("daypress_2fa_secret")
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        Ok(enabled)
    }

    pub fn get_user_2fa_status(&self, user_id: i64) -> Result<TwoFactorStatusResponse> {
        let conn = self.conn.lock().unwrap();
        let meta = Self::read_usermeta_map(&conn, user_id);
        let enabled = parse_bool_meta(meta.get("daypress_2fa_enabled"), false)
            && meta
                .get("daypress_2fa_secret")
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        let recovery_codes_remaining = meta
            .get("daypress_2fa_recovery_hashes")
            .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
            .map(|v| v.len())
            .unwrap_or(0);
        Ok(TwoFactorStatusResponse {
            enabled,
            recovery_codes_remaining,
        })
    }

    pub fn begin_user_2fa_setup(&self, user_id: i64) -> Result<TwoFactorSetupResponse, AppError> {
        let conn = self.conn.lock().unwrap();
        let login: String = conn
            .query_row(
                "SELECT user_login FROM users WHERE ID = ?1",
                params![user_id],
                |r| r.get(0),
            )
            .map_err(|_| AppError::NotFound(format!("User {} not found", user_id)))?;
        let issuer: String = conn
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'blogname'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "Daypress".to_string());

        let secret = crate::totp::generate_totp_secret();
        Self::upsert_usermeta(&conn, user_id, "daypress_2fa_pending_secret", &secret)?;
        let otpauth_uri = crate::totp::build_otpauth_uri(&secret, &login, &issuer);
        Ok(TwoFactorSetupResponse {
            secret,
            otpauth_uri,
        })
    }

    pub fn enable_user_2fa(
        &self,
        user_id: i64,
        code: &str,
    ) -> Result<TwoFactorEnableResponse, AppError> {
        let conn = self.conn.lock().unwrap();
        let meta = Self::read_usermeta_map(&conn, user_id);
        let secret = meta
            .get("daypress_2fa_pending_secret")
            .or_else(|| meta.get("daypress_2fa_secret"))
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                AppError::BadRequest("No 2FA setup in progress; start 2FA setup first".to_string())
            })?;

        let now = Utc::now().timestamp();
        let accepted_step =
            crate::totp::verify_totp_code(&secret, code, now, None).ok_or_else(|| {
                AppError::BadRequest("Invalid 6-digit authenticator code".to_string())
            })?;

        let recovery_codes = crate::totp::generate_recovery_codes();
        let hashes: Vec<String> = recovery_codes
            .iter()
            .map(|c| crate::totp::hash_recovery_code(c))
            .collect();
        let hashes_json = serde_json::to_string(&hashes).unwrap_or_else(|_| "[]".to_string());

        Self::upsert_usermeta(&conn, user_id, "daypress_2fa_secret", &secret)?;
        Self::upsert_usermeta(&conn, user_id, "daypress_2fa_enabled", "true")?;
        Self::upsert_usermeta(&conn, user_id, "daypress_2fa_recovery_hashes", &hashes_json)?;
        Self::upsert_usermeta(
            &conn,
            user_id,
            "daypress_2fa_last_step",
            &accepted_step.to_string(),
        )?;
        conn.execute(
            "DELETE FROM usermeta WHERE user_id = ?1 AND meta_key = 'daypress_2fa_pending_secret'",
            params![user_id],
        )?;

        Ok(TwoFactorEnableResponse {
            enabled: true,
            recovery_codes,
        })
    }

    pub fn regenerate_user_2fa_recovery_codes(
        &self,
        user_id: i64,
    ) -> Result<Vec<String>, AppError> {
        let conn = self.conn.lock().unwrap();
        let meta = Self::read_usermeta_map(&conn, user_id);
        let enabled = parse_bool_meta(meta.get("daypress_2fa_enabled"), false)
            && meta
                .get("daypress_2fa_secret")
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        if !enabled {
            return Err(AppError::BadRequest(
                "Two-factor authentication is not enabled on this account".to_string(),
            ));
        }

        let recovery_codes = crate::totp::generate_recovery_codes();
        let hashes: Vec<String> = recovery_codes
            .iter()
            .map(|c| crate::totp::hash_recovery_code(c))
            .collect();
        let hashes_json = serde_json::to_string(&hashes).unwrap_or_else(|_| "[]".to_string());
        Self::upsert_usermeta(&conn, user_id, "daypress_2fa_recovery_hashes", &hashes_json)?;
        Ok(recovery_codes)
    }

    pub fn disable_user_2fa(&self, user_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let was_enabled = {
            let meta = Self::read_usermeta_map(&conn, user_id);
            parse_bool_meta(meta.get("daypress_2fa_enabled"), false)
        };
        conn.execute(
            "DELETE FROM usermeta WHERE user_id = ?1 AND meta_key IN ('daypress_2fa_enabled', 'daypress_2fa_secret', 'daypress_2fa_pending_secret', 'daypress_2fa_recovery_hashes', 'daypress_2fa_last_step')",
            params![user_id],
        )?;
        Ok(was_enabled)
    }

    /// Verifies either a 6-digit TOTP code (with replay protection) or a single-use recovery code.
    pub fn verify_user_2fa_code(&self, user_id: i64, code: &str) -> Result<bool> {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }
        let conn = self.conn.lock().unwrap();
        let meta = Self::read_usermeta_map(&conn, user_id);
        let enabled = parse_bool_meta(meta.get("daypress_2fa_enabled"), false);
        let Some(secret) = meta
            .get("daypress_2fa_secret")
            .filter(|s| !s.trim().is_empty())
        else {
            return Ok(false);
        };
        if !enabled {
            return Ok(false);
        }

        let last_step = meta
            .get("daypress_2fa_last_step")
            .and_then(|v| v.trim().parse::<i64>().ok());
        let now = Utc::now().timestamp();
        if let Some(accepted_step) = crate::totp::verify_totp_code(secret, trimmed, now, last_step)
        {
            Self::upsert_usermeta(
                &conn,
                user_id,
                "daypress_2fa_last_step",
                &accepted_step.to_string(),
            )?;
            return Ok(true);
        }

        // Check single-use recovery codes
        if let Some(raw_hashes) = meta.get("daypress_2fa_recovery_hashes") {
            if let Ok(mut hashes) = serde_json::from_str::<Vec<String>>(raw_hashes) {
                let candidate = crate::totp::hash_recovery_code(trimmed);
                if let Some(pos) = hashes
                    .iter()
                    .position(|h| crate::auth::constant_time_eq(h.as_bytes(), candidate.as_bytes()))
                {
                    hashes.remove(pos);
                    let updated_json =
                        serde_json::to_string(&hashes).unwrap_or_else(|_| "[]".to_string());
                    Self::upsert_usermeta(
                        &conn,
                        user_id,
                        "daypress_2fa_recovery_hashes",
                        &updated_json,
                    )?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
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
        Ok(self
            .attempt_login_with_method(username, password, client)?
            .0)
    }

    pub fn attempt_login_with_method(
        &self,
        username: &str,
        password: &str,
        client: Option<&str>,
    ) -> Result<(crate::auth::LoginAttempt, bool)> {
        use crate::auth::{LoginAttempt, LoginThrottle};
        let key = LoginThrottle::key(client, username);
        if let Some(retry_after_secs) = self.login_throttle.retry_after(&key) {
            return Ok((LoginAttempt::Throttled { retry_after_secs }, false));
        }
        match self.authenticate_credentials_with_method(username, password)? {
            Some((user, used_app_password)) => {
                self.login_throttle.record_success(&key);
                Ok((LoginAttempt::Success(user), used_app_password))
            }
            None => {
                self.login_throttle.record_failure(&key);
                Ok((LoginAttempt::Invalid, false))
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

    pub fn delete_all_application_passwords(&self, user_id: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM application_passwords WHERE user_id = ?1",
            params![user_id],
        )?;
        Ok(rows)
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

        let comments: Vec<DaypressCommentAdminItem> = conn
            .prepare("SELECT c.comment_ID, c.comment_post_ID, COALESCE(p.post_title, 'Post'), c.comment_author, c.comment_author_email, c.comment_author_url, c.comment_author_IP, c.comment_content, c.comment_date, c.comment_approved FROM comments c LEFT JOIN posts p ON c.comment_post_ID = p.ID WHERE c.comment_author_email = ?1 OR c.comment_author = ?1")
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![identifier], |r| {
                    Ok(DaypressCommentAdminItem {
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

    pub fn get_user_screen_preferences(&self, user_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let raw: Option<String> = conn
            .query_row(
                "SELECT meta_value FROM usermeta WHERE user_id = ?1 AND meta_key = '_daypress_screen_prefs'",
                params![user_id],
                |r| r.get(0),
            )
            .ok();
        let parsed = raw
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        Ok(parsed)
    }

    pub fn save_user_screen_preferences(
        &self,
        user_id: i64,
        patch: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut current = self.get_user_screen_preferences(user_id)?;
        if !current.is_object() {
            current = serde_json::json!({});
        }
        let effective_patch = patch
            .get("preferences")
            .filter(|v| v.is_object())
            .unwrap_or(patch);
        if let (Some(cur_obj), Some(patch_obj)) =
            (current.as_object_mut(), effective_patch.as_object())
        {
            for (k, v) in patch_obj {
                if k == "screens" {
                    let entry = cur_obj
                        .entry("screens".to_string())
                        .or_insert_with(|| serde_json::json!({}));
                    if !entry.is_object() {
                        *entry = serde_json::json!({});
                    }
                    if let (Some(cur_screens), Some(patch_screens)) =
                        (entry.as_object_mut(), v.as_object())
                    {
                        for (sk, sv) in patch_screens {
                            cur_screens.insert(sk.clone(), sv.clone());
                        }
                    }
                } else {
                    cur_obj.insert(k.clone(), v.clone());
                }
            }
        }
        let serialized = serde_json::to_string(&current).unwrap_or_else(|_| "{}".to_string());
        let conn = self.conn.lock().unwrap();
        Self::upsert_usermeta(&conn, user_id, "_daypress_screen_prefs", &serialized)?;
        Ok(current)
    }
}
