use crate::db::Database;
use crate::errors::AppError;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
    pub data: JwtUserData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtUserData {
    pub user: JwtUserRecord,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtUserRecord {
    pub id: i64,
    pub username: String,
}

/// Computes HMAC-SHA256 for the given secret and payload.
pub fn hmac_sha256(secret: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size in SHA256");
    mac.update(payload);
    mac.finalize().into_bytes().to_vec()
}

/// Computes SHA256 hex digest.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Hashes a user password with argon2id (PHC string `$argon2id$v=19$...`, OWASP-recommended
/// default parameters) and a random 16-byte salt.
pub fn hash_user_password(password: &str) -> String {
    use argon2::password_hash::{PasswordHasher, SaltString};

    let mut salt_bytes = [0u8; 16];
    fill_random(&mut salt_bytes);
    SaltString::encode_b64(&salt_bytes)
        .and_then(|salt| {
            argon2::Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
        })
        // Only reachable with invalid argon2 parameters; keep a salted hash rather than panic.
        .unwrap_or_else(|_| legacy_sha256_password_hash(password))
}

/// Pre-argon2 Daypress format (`$daypress$sha256$<salt>$<hash>`); still verified so existing
/// accounts keep working, and upgraded to argon2id on their next successful login.
fn legacy_sha256_password_hash(password: &str) -> String {
    let salt: String = random_hex_64().chars().take(16).collect();
    let hash = sha256_hex(format!("{}:{}", salt, password).as_bytes());
    format!("$daypress$sha256${}${}", salt, hash)
}

/// Whether a stored hash predates argon2id and should be re-hashed after a successful login.
pub fn password_needs_rehash(stored_pass: &str) -> bool {
    !stored_pass.starts_with("$argon2")
}

/// Placeholder hash written by older seeds for the initial `admin` account.
/// It matches no password; [`crate::db::Database::ensure_admin_credentials`] replaces it on startup.
pub const LEGACY_SEED_PASSWORD_HASH: &str = "$2y$10$abcdefghijklmnopqrstuvwxyz1234567890";

/// Verifies a user password against a stored password hash.
/// Only Daypress hashes are accepted: imported WordPress (`$P$` / `$wp$`) hashes, seed
/// placeholders, and plaintext values never authenticate.
pub fn verify_user_password(password: &str, stored_pass: &str) -> bool {
    if password.is_empty() {
        return false;
    }
    if stored_pass.starts_with("$argon2") {
        use argon2::password_hash::{PasswordHash, PasswordVerifier};
        return PasswordHash::new(stored_pass).is_ok_and(|hash| {
            argon2::Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_ok()
        });
    }
    if let Some(rest) = stored_pass.strip_prefix("$daypress$sha256$") {
        if let Some((salt, expected_hash)) = rest.split_once('$') {
            let computed = sha256_hex(format!("{}:{}", salt, password).as_bytes());
            return constant_time_eq(
                computed.to_ascii_lowercase().as_bytes(),
                expected_hash.to_ascii_lowercase().as_bytes(),
            );
        }
        return false;
    }
    if stored_pass.len() == 64 && stored_pass.chars().all(|c| c.is_ascii_hexdigit()) {
        return constant_time_eq(
            sha256_hex(password.as_bytes()).as_bytes(),
            stored_pass.to_ascii_lowercase().as_bytes(),
        );
    }
    false
}

/// Generates a random 20-character password for first-run and imported accounts.
pub fn generate_random_password() -> String {
    random_hex_64().chars().take(20).collect()
}

/// Generates a standard 24-character WordPress Application Password in 4 chunks of 4 characters:
/// e.g. "abcd efgh ijkl mnop".
pub fn generate_application_password() -> String {
    let raw = uuid_like_random_string();
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(16)
        .collect();
    let chunks: Vec<&str> = cleaned
        .as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap_or("xxxx"))
        .collect();
    chunks.join(" ")
}

fn uuid_like_random_string() -> String {
    random_hex_64().chars().take(24).collect()
}

/// Returns 64 hex characters (32 bytes) from the operating system CSPRNG.
pub fn random_hex_64() -> String {
    let mut bytes = [0u8; 32];
    fill_random(&mut bytes);
    hex::encode(bytes)
}

/// Fills `buf` from the OS CSPRNG. If the OS generator is unavailable (practically never),
/// falls back to process-seeded SipHash keys mixed with the clock instead of panicking.
fn fill_random(buf: &mut [u8]) {
    if getrandom::getrandom(buf).is_ok() {
        return;
    }
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    for (i, chunk) in buf.chunks_mut(8).enumerate() {
        let mut h = RandomState::new().build_hasher();
        h.write_u128(nanos);
        h.write_usize(i);
        let word = h.finish().to_le_bytes();
        chunk.copy_from_slice(&word[..chunk.len()]);
    }
}

/// Failed logins allowed per client + username within [`LOGIN_WINDOW_SECS`].
pub const MAX_FAILED_LOGINS: u32 = 5;
/// Length of the failed-login window, and of the lockout once it is exceeded.
pub const LOGIN_WINDOW_SECS: i64 = 15 * 60;

/// Outcome of a throttled credential check (login form, Basic auth, JWT token endpoint).
#[derive(Debug, Clone)]
pub enum LoginAttempt {
    Success(AuthUser),
    Invalid,
    /// Too many recent failures for this client + username; retry after the given seconds.
    Throttled {
        retry_after_secs: i64,
    },
}

/// In-memory failed-login counter keyed by client address + username, so a brute-force run
/// against one account from one address is slowed to [`MAX_FAILED_LOGINS`] per window
/// without letting a third party lock that account out for everyone else.
#[derive(Debug, Clone, Default)]
pub struct LoginThrottle(
    std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, (u32, i64)>>>,
);

impl LoginThrottle {
    pub fn key(client: Option<&str>, username: &str) -> String {
        format!(
            "{}|{}",
            client.unwrap_or("-"),
            username.trim().to_lowercase()
        )
    }

    /// Seconds until `key` may try again, or `None` when it is not throttled.
    pub fn retry_after(&self, key: &str) -> Option<i64> {
        let now = Utc::now().timestamp();
        let map = self.0.lock().ok()?;
        let (failures, window_start) = *map.get(key)?;
        let remaining = window_start + LOGIN_WINDOW_SECS - now;
        (failures >= MAX_FAILED_LOGINS && remaining > 0).then_some(remaining)
    }

    pub fn record_failure(&self, key: &str) {
        let now = Utc::now().timestamp();
        if let Ok(mut map) = self.0.lock() {
            if map.len() > 10_000 {
                map.retain(|_, (_, start)| now - *start < LOGIN_WINDOW_SECS);
            }
            let entry = map.entry(key.to_string()).or_insert((0, now));
            if now - entry.1 >= LOGIN_WINDOW_SECS {
                *entry = (0, now);
            }
            entry.0 += 1;
        }
    }

    pub fn record_success(&self, key: &str) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(key);
        }
    }
}

/// The connecting socket's IP, when the server runs with connect info (see `main.rs`).
pub fn client_ip(parts: &Parts) -> Option<String> {
    parts
        .extensions
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|info| info.0.ip().to_string())
}

/// Extractor for [`client_ip`]; never rejects.
#[derive(Debug, Clone)]
pub struct ClientIp(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(ClientIp(client_ip(parts)))
    }
}

/// Name of the signed admin session cookie.
pub const SESSION_COOKIE: &str = "daypress_admin";

/// Name of the short-lived (5-minute) signed 2FA challenge cookie set after step-1 password verification.
pub const CHALLENGE_2FA_COOKIE: &str = "daypress_2fa_challenge";
/// Lifetime of the 2FA challenge cookie (5 minutes).
pub const CHALLENGE_2FA_TTL_SECS: i64 = 5 * 60;

/// Session lifetime when "Remember Me" is ticked (14 days, WordPress parity).
pub const SESSION_TTL_REMEMBER_SECS: i64 = 14 * 86400;
/// Session lifetime for a regular login (2 days, WordPress parity).
pub const SESSION_TTL_DEFAULT_SECS: i64 = 2 * 86400;

/// Creates a signed 5-minute 2FA challenge token `<user_id>.<exp>.<1_or_0>.<hmac_hex>`.
pub fn create_2fa_challenge_token(user_id: i64, remember: bool, secret: &str) -> String {
    let exp = Utc::now().timestamp() + CHALLENGE_2FA_TTL_SECS;
    let rem_flag = if remember { 1 } else { 0 };
    let payload = format!("{}.{}.{}", user_id, exp, rem_flag);
    let sig = hex::encode(hmac_sha256(
        secret.as_bytes(),
        format!("2fa_challenge|{}", payload).as_bytes(),
    ));
    format!("{}.{}", payload, sig)
}

/// Verifies a 2FA challenge token and returns `(user_id, remember)`.
pub fn verify_2fa_challenge_token(token: &str, secret: &str) -> Option<(i64, bool)> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let uid: i64 = parts[0].parse().ok()?;
    let exp: i64 = parts[1].parse().ok()?;
    let rem_flag: i64 = parts[2].parse().ok()?;
    if exp < Utc::now().timestamp() {
        return None;
    }
    let expected = hex::encode(hmac_sha256(
        secret.as_bytes(),
        format!("2fa_challenge|{}.{}.{}", parts[0], parts[1], parts[2]).as_bytes(),
    ));
    if constant_time_eq(expected.as_bytes(), parts[3].as_bytes()) {
        Some((uid, rem_flag != 0))
    } else {
        None
    }
}

pub fn challenge_2fa_cookie_header(user_id: i64, remember: bool, secret: &str) -> String {
    let token = create_2fa_challenge_token(user_id, remember, secret);
    format!(
        "{}={}; Path=/; Max-Age={}; SameSite=Lax; HttpOnly",
        CHALLENGE_2FA_COOKIE, token, CHALLENGE_2FA_TTL_SECS
    )
}

pub fn clear_2fa_challenge_cookie_header() -> &'static str {
    "daypress_2fa_challenge=; Path=/; Max-Age=0; SameSite=Lax; HttpOnly"
}

pub fn extract_2fa_challenge_from_cookie_header(
    cookie_header: &str,
    secret: &str,
) -> Option<(i64, bool)> {
    let prefix = format!("{}=", CHALLENGE_2FA_COOKIE);
    for item in cookie_header.split(';') {
        let trimmed = item.trim();
        if let Some(val) = trimmed.strip_prefix(&prefix) {
            if let Some(res) = verify_2fa_challenge_token(val, secret) {
                return Some(res);
            }
        }
    }
    None
}

/// Creates a tamper-proof session token: `<user_id>.<expires_unix>.<hmac_hex>`.
pub fn create_versioned_session_token(
    user_id: i64,
    secret: &str,
    ttl_secs: i64,
    session_ver: i64,
) -> String {
    let exp = Utc::now().timestamp() + ttl_secs;
    let payload = if session_ver > 0 {
        format!("{}.{}.{}", user_id, exp, session_ver)
    } else {
        format!("{}.{}", user_id, exp)
    };
    let sig = hex::encode(hmac_sha256(
        secret.as_bytes(),
        format!("session|{}", payload).as_bytes(),
    ));
    format!("{}.{}", payload, sig)
}

pub fn verify_versioned_session_token(token: &str, secret: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() == 3 {
        let uid: i64 = parts[0].parse().ok()?;
        let exp: i64 = parts[1].parse().ok()?;
        if exp < Utc::now().timestamp() {
            return None;
        }
        let expected = hex::encode(hmac_sha256(
            secret.as_bytes(),
            format!("session|{}.{}", parts[0], parts[1]).as_bytes(),
        ));
        if constant_time_eq(expected.as_bytes(), parts[2].as_bytes()) {
            return Some((uid, 0));
        }
        return None;
    }
    if parts.len() == 4 {
        let uid: i64 = parts[0].parse().ok()?;
        let exp: i64 = parts[1].parse().ok()?;
        let ver: i64 = parts[2].parse().ok()?;
        if exp < Utc::now().timestamp() {
            return None;
        }
        let expected = hex::encode(hmac_sha256(
            secret.as_bytes(),
            format!("session|{}.{}.{}", parts[0], parts[1], parts[2]).as_bytes(),
        ));
        if constant_time_eq(expected.as_bytes(), parts[3].as_bytes()) {
            return Some((uid, ver));
        }
    }
    None
}

pub fn create_session_token(user_id: i64, secret: &str, ttl_secs: i64) -> String {
    let exp = Utc::now().timestamp() + ttl_secs;
    let payload = format!("{}.{}", user_id, exp);
    let sig = hex::encode(hmac_sha256(
        secret.as_bytes(),
        format!("session|{}", payload).as_bytes(),
    ));
    format!("{}.{}", payload, sig)
}

/// Verifies a session token created by [`create_session_token`] and returns the user id.
pub fn verify_session_token(token: &str, secret: &str) -> Option<i64> {
    let mut parts = token.trim().splitn(3, '.');
    let uid_raw = parts.next()?;
    let exp_raw = parts.next()?;
    let sig = parts.next()?;
    let uid: i64 = uid_raw.parse().ok()?;
    let exp: i64 = exp_raw.parse().ok()?;
    if exp < Utc::now().timestamp() {
        return None;
    }
    let expected = hex::encode(hmac_sha256(
        secret.as_bytes(),
        format!("session|{}.{}", uid_raw, exp_raw).as_bytes(),
    ));
    if constant_time_eq(expected.as_bytes(), sig.as_bytes()) {
        Some(uid)
    } else {
        None
    }
}

pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Builds the `Set-Cookie` header value for a freshly authenticated user.
pub fn session_cookie_header_with_version(
    user_id: i64,
    secret: &str,
    remember: bool,
    session_ver: i64,
) -> String {
    let ttl = if remember {
        SESSION_TTL_REMEMBER_SECS
    } else {
        SESSION_TTL_DEFAULT_SECS
    };
    let token = create_versioned_session_token(user_id, secret, ttl, session_ver);
    if remember {
        format!(
            "{}={}; Path=/; Max-Age={}; SameSite=Lax; HttpOnly",
            SESSION_COOKIE, token, ttl
        )
    } else {
        format!(
            "{}={}; Path=/; SameSite=Lax; HttpOnly",
            SESSION_COOKIE, token
        )
    }
}

pub fn session_cookie_header(user_id: i64, secret: &str, remember: bool) -> String {
    let ttl = if remember {
        SESSION_TTL_REMEMBER_SECS
    } else {
        SESSION_TTL_DEFAULT_SECS
    };
    let token = create_session_token(user_id, secret, ttl);
    if remember {
        format!(
            "{}={}; Path=/; Max-Age={}; SameSite=Lax; HttpOnly",
            SESSION_COOKIE, token, ttl
        )
    } else {
        format!(
            "{}={}; Path=/; SameSite=Lax; HttpOnly",
            SESSION_COOKIE, token
        )
    }
}

/// `Cookie` header value carrying a fresh signed session for `user_id`
/// (what `/login` would issue); used by integration tests and tooling.
pub fn session_cookie_for(user_id: i64, db: &Database) -> String {
    let secret = db.get_auth_secret().unwrap_or_default();
    let ver = db
        .get_user_detail(user_id)
        .ok()
        .flatten()
        .map(|u| u.session_ver)
        .unwrap_or(1);
    let token = create_versioned_session_token(user_id, &secret, SESSION_TTL_DEFAULT_SECS, ver);
    format!("{}={}", SESSION_COOKIE, token)
}

/// Resolves the logged-in user from the signed `daypress_admin` session cookie.
/// Unsigned, forged, or expired values are rejected.
pub fn session_user_from_cookie_header(cookie_header: &str, db: &Database) -> Option<AuthUser> {
    let secret = db.get_auth_secret().ok()?;
    for item in cookie_header.split(';') {
        let trimmed = item.trim();
        if let Some(val) = trimmed.strip_prefix("daypress_admin=") {
            if let Some((uid, token_ver)) = verify_versioned_session_token(val, &secret) {
                if let Ok(Some(u)) = db.get_user_detail(uid) {
                    if token_ver >= u.session_ver {
                        return Some(AuthUser {
                            user_id: u.id,
                            username: u.username,
                            role: u.role,
                        });
                    }
                }
            }
            if let Some(uid) = None::<i64> {
                if let Ok(Some(u)) = db.get_user_detail(uid) {
                    return Some(AuthUser {
                        user_id: u.id,
                        username: u.username,
                        role: u.role,
                    });
                }
            }
        }
    }
    None
}

impl AuthUser {
    fn normalized_role(&self) -> String {
        self.role.trim().to_ascii_lowercase()
    }

    /// Full site management: settings, users, plugins, imports, theme templates.
    pub fn is_admin(&self) -> bool {
        self.normalized_role() == "administrator"
    }

    /// Editors and Administrators (`edit_others_posts`, `edit_pages`, `manage_categories`, `moderate_comments`).
    pub fn is_editor_or_above(&self) -> bool {
        matches!(self.normalized_role().as_str(), "administrator" | "editor")
    }

    /// Authors, Editors, and Administrators (`publish_posts`, `upload_files`, `edit_published_posts`).
    pub fn is_author_or_above(&self) -> bool {
        matches!(
            self.normalized_role().as_str(),
            "administrator" | "editor" | "author"
        )
    }

    /// Content management (`edit_posts`): Administrator, Editor, Author, and Contributor.
    pub fn can_edit_content(&self) -> bool {
        matches!(
            self.normalized_role().as_str(),
            "administrator" | "editor" | "author" | "contributor"
        )
    }

    /// WordPress `edit_pages` / `publish_pages` capability (Administrator & Editor).
    pub fn can_edit_pages(&self) -> bool {
        self.is_editor_or_above()
    }

    /// WordPress `publish_posts` capability (Administrator, Editor, Author).
    pub fn can_publish_posts(&self) -> bool {
        self.is_author_or_above()
    }

    /// WordPress `upload_files` capability (Administrator, Editor, Author).
    pub fn can_upload_files(&self) -> bool {
        self.is_author_or_above()
    }

    /// WordPress `manage_categories` capability (Administrator, Editor).
    pub fn can_manage_categories(&self) -> bool {
        self.is_editor_or_above()
    }

    /// WordPress `moderate_comments` capability (Administrator, Editor).
    pub fn can_moderate_comments(&self) -> bool {
        self.is_editor_or_above()
    }

    /// Checks whether this user may edit a specific post/page record given its author, status, and post_type.
    pub fn can_edit_post(&self, post_author: i64, post_status: &str, post_type: &str) -> bool {
        if self.is_editor_or_above() {
            return true;
        }
        if post_type.eq_ignore_ascii_case("page") {
            return false;
        }
        if post_author != self.user_id {
            return false;
        }
        match self.normalized_role().as_str() {
            "author" => true,
            "contributor" => !matches!(
                post_status.trim().to_ascii_lowercase().as_str(),
                "publish" | "future" | "private"
            ),
            _ => false,
        }
    }

    /// Checks whether this user may delete a specific post/page record given its author, status, and post_type.
    pub fn can_delete_post(&self, post_author: i64, post_status: &str, post_type: &str) -> bool {
        self.can_edit_post(post_author, post_status, post_type)
    }
}

/// Whether content with `status` may be shown to this visitor. Published content (and media,
/// stored as `inherit`) is public; drafts, pending, private, and scheduled posts are visible
/// only to users who can edit content, which doubles as preview. Trash is never shown.
pub fn can_view_status(status: &str, user: Option<&AuthUser>) -> bool {
    match status {
        "publish" | "inherit" => true,
        "trash" => false,
        _ => user.is_some_and(|u| u.can_edit_content()),
    }
}

/// Extractor for endpoints that require a content-editing role (not a subscriber).
#[derive(Debug, Clone)]
pub struct EditorUser(pub AuthUser);

/// Extractor for endpoints that require the administrator role.
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

/// Strips whitespace and dashes from application passwords for flexible comparison.
pub fn normalize_app_password(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect()
}

/// Creates a standard WordPress REST Nonce.
/// Matches WordPress core: tick = timestamp / 43200 (12-hour validity window per tick).
pub fn create_rest_nonce(action: &str, user_id: i64, secret: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let tick = now / 43200;
    let data = format!("{}|{}|{}", action, user_id, tick);
    let mac = hmac_sha256(secret.as_bytes(), data.as_bytes());
    let hex_str = hex::encode(mac);
    hex_str[hex_str.len().saturating_sub(10)..].to_string()
}

/// Verifies a WordPress REST Nonce against current tick and previous tick (valid up to 24h).
pub fn verify_rest_nonce(nonce: &str, action: &str, user_id: i64, secret: &str) -> bool {
    if nonce.is_empty() {
        return false;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let tick = now / 43200;

    for t in [tick, tick.saturating_sub(1)] {
        let data = format!("{}|{}|{}", action, user_id, t);
        let mac = hmac_sha256(secret.as_bytes(), data.as_bytes());
        let hex_str = hex::encode(mac);
        let expected = &hex_str[hex_str.len().saturating_sub(10)..];
        if expected.eq_ignore_ascii_case(nonce) {
            return true;
        }
    }
    false
}

/// Creates a signed HS256 JWT token compatible with WordPress `jwt-auth` plugins.
pub fn create_jwt_token(
    user_id: i64,
    username: &str,
    site_url: &str,
    secret: &str,
    duration_days: i64,
) -> String {
    let now = Utc::now().timestamp();
    let exp = now + (duration_days * 86400);

    let header = JwtHeader {
        alg: "HS256".to_string(),
        typ: "JWT".to_string(),
    };

    let claims = JwtClaims {
        iss: site_url.to_string(),
        iat: now,
        exp,
        data: JwtUserData {
            user: JwtUserRecord {
                id: user_id,
                username: username.to_string(),
            },
        },
    };

    let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
    let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());
    let signing_input = format!("{}.{}", header_b64, claims_b64);
    let sig = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);

    format!("{}.{}", signing_input, sig_b64)
}

/// Verifies and decodes an HS256 JWT token.
pub fn verify_jwt_token(token: &str, secret: &str) -> Result<JwtClaims, AppError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AppError::Unauthorized("Invalid token format".to_string()));
    }

    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let expected_sig = hmac_sha256(secret.as_bytes(), signing_input.as_bytes());
    let provided_sig = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| AppError::Unauthorized("Invalid token signature encoding".to_string()))?;

    if !constant_time_eq(&expected_sig, &provided_sig) {
        return Err(AppError::Unauthorized(
            "Invalid token signature".to_string(),
        ));
    }

    let claims_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| AppError::Unauthorized("Invalid token payload encoding".to_string()))?;

    let claims: JwtClaims = serde_json::from_slice(&claims_bytes)
        .map_err(|_| AppError::Unauthorized("Invalid token payload JSON".to_string()))?;

    let now = Utc::now().timestamp();
    if claims.exp < now {
        return Err(AppError::Unauthorized("Token has expired".to_string()));
    }

    Ok(claims)
}

/// Authenticates an incoming request using either:
/// 1. Bearer JWT Token
/// 2. Basic Auth (Username + User Password or Application Password)
/// 3. Signed session cookie
pub async fn authenticate_request(parts: &Parts, db: &Database) -> Result<AuthUser, AppError> {
    let secret = db.get_auth_secret()?;

    // 1. Check Authorization header
    if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            let auth_str = auth_str.trim();

            // Bearer Token: the role always comes from the current user record, never the token.
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let claims = verify_jwt_token(token.trim(), &secret)?;
                let user = db.get_user_detail(claims.data.user.id)?.ok_or_else(|| {
                    AppError::Unauthorized("Token user no longer exists".to_string())
                })?;
                return Ok(AuthUser {
                    user_id: user.id,
                    username: user.username,
                    role: user.role,
                });
            }

            // Basic Auth (e.g. Application Passwords or admin user credentials)
            if let Some(b64_cred) = auth_str.strip_prefix("Basic ") {
                if let Ok(decoded) =
                    base64::engine::general_purpose::STANDARD.decode(b64_cred.trim())
                {
                    if let Ok(cred_str) = String::from_utf8(decoded) {
                        if let Some((user, pass)) = cred_str.split_once(':') {
                            let client = client_ip(parts);
                            match db.attempt_login_with_method(
                                user.trim(),
                                pass.trim(),
                                client.as_deref(),
                            )? {
                                (LoginAttempt::Success(auth_user), used_app_password) => {
                                    if !used_app_password
                                        && db
                                            .is_user_2fa_enabled(auth_user.user_id)
                                            .unwrap_or(false)
                                    {
                                        let otp = parts
                                            .headers
                                            .get("x-daypress-otp")
                                            .and_then(|v| v.to_str().ok())
                                            .unwrap_or("");
                                        if !db
                                            .verify_user_2fa_code(auth_user.user_id, otp)
                                            .unwrap_or(false)
                                        {
                                            return Err(AppError::Unauthorized(
                                                "Two-factor authentication code required (provide X-Daypress-OTP header or use an Application Password).".to_string(),
                                            ));
                                        }
                                    }
                                    return Ok(auth_user);
                                }
                                (LoginAttempt::Throttled { retry_after_secs }, _) => {
                                    return Err(AppError::TooManyRequests(format!(
                                        "Too many failed login attempts. Try again in {} minutes.",
                                        (retry_after_secs + 59) / 60
                                    )))
                                }
                                (LoginAttempt::Invalid, _) => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check signed session cookie (issued by /login). Unsigned / forged values are rejected.
    // As in WordPress, an `X-WP-Nonce` on its own never authenticates a request.
    if let Some(cookie_header) = parts.headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            if let Some(user) = session_user_from_cookie_header(cookie_str, db) {
                let is_mutating = !matches!(
                    parts.method,
                    axum::http::Method::GET
                        | axum::http::Method::HEAD
                        | axum::http::Method::OPTIONS
                );
                if is_mutating {
                    // If an explicit X-WP-Nonce header is supplied with a session cookie, verify it
                    if let Some(nonce_val) = parts
                        .headers
                        .get("x-wp-nonce")
                        .and_then(|v| v.to_str().ok())
                    {
                        if !verify_rest_nonce(nonce_val.trim(), "wp_rest", user.user_id, &secret) {
                            return Err(AppError::Forbidden(
                                "Cookie check failed (invalid X-WP-Nonce).".to_string(),
                            ));
                        }
                    } else {
                        // Enforce Same-Origin / Sec-Fetch-Site CSRF protection on cookie-authenticated mutations
                        if let Some(sec_site) = parts
                            .headers
                            .get("sec-fetch-site")
                            .and_then(|v| v.to_str().ok())
                        {
                            let s = sec_site.trim().to_ascii_lowercase();
                            if s == "cross-site" || s == "same-site" {
                                return Err(AppError::Forbidden(
                                    "Cross-site cookie mutation blocked (CSRF protection)."
                                        .to_string(),
                                ));
                            }
                        }
                        if let Some(origin) = parts
                            .headers
                            .get(header::ORIGIN)
                            .and_then(|v| v.to_str().ok())
                        {
                            let host = parts
                                .headers
                                .get(header::HOST)
                                .and_then(|v| v.to_str().ok());
                            let forwarded_host = parts
                                .headers
                                .get("x-forwarded-host")
                                .and_then(|v| v.to_str().ok());
                            if !origin_matches_request_host(origin, host, forwarded_host) {
                                return Err(AppError::Forbidden(
                                    "Cross-origin cookie mutation blocked (CSRF protection)."
                                        .to_string(),
                                ));
                            }
                        }
                    }
                }
                return Ok(user);
            }
        }
    }

    Err(AppError::Unauthorized(
        "Sorry, you are not allowed to create posts as this user.".to_string(),
    ))
}

fn strip_default_port(authority: &str, is_https: Option<bool>) -> &str {
    let trimmed = authority.trim();
    match is_https {
        Some(true) => trimmed.strip_suffix(":443").unwrap_or(trimmed),
        Some(false) => trimmed.strip_suffix(":80").unwrap_or(trimmed),
        None => trimmed
            .strip_suffix(":443")
            .or_else(|| trimmed.strip_suffix(":80"))
            .unwrap_or(trimmed),
    }
}

fn origin_matches_request_host(
    origin: &str,
    host: Option<&str>,
    forwarded_host: Option<&str>,
) -> bool {
    let trimmed_origin = origin.trim();
    let (is_https, rest) = if let Some(r) = trimmed_origin.strip_prefix("https://") {
        (Some(true), r)
    } else if let Some(r) = trimmed_origin.strip_prefix("http://") {
        (Some(false), r)
    } else {
        (None, trimmed_origin)
    };
    let origin_authority = strip_default_port(rest.split('/').next().unwrap_or(""), is_https);
    if origin_authority.is_empty() || origin_authority.eq_ignore_ascii_case("null") {
        return true;
    }

    let primary_forwarded = forwarded_host
        .and_then(|raw| raw.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let direct_host = host.map(str::trim).filter(|s| !s.is_empty());

    if primary_forwarded.is_none() && direct_host.is_none() {
        return true;
    }

    [primary_forwarded, direct_host]
        .into_iter()
        .flatten()
        .any(|candidate| {
            let norm_candidate = strip_default_port(candidate, is_https);
            origin_authority.eq_ignore_ascii_case(candidate)
                || origin_authority.eq_ignore_ascii_case(norm_candidate)
        })
}

// =========================================================================
// ADMIN SESSION GATE (/d-admin/*)
// =========================================================================

/// Per-request context of the logged-in admin, available to SSR templates
/// (top bar "Howdy, …", REST nonce) without threading it through every renderer.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct _UnusedDeriveTarget;
static USER_UI_PREFS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<i64, (bool, String)>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

pub fn sync_user_ui_prefs(user_id: i64, show_admin_bar_front: bool, avatar_url: &str) {
    if let Ok(mut map) = USER_UI_PREFS.lock() {
        map.insert(user_id, (show_admin_bar_front, avatar_url.to_string()));
    }
}

pub fn should_show_admin_bar_front(user_id: i64) -> bool {
    USER_UI_PREFS
        .lock()
        .ok()
        .and_then(|m| m.get(&user_id).map(|(show, _)| *show))
        .unwrap_or(true)
}

pub fn cached_user_avatar_url(user_id: i64) -> Option<String> {
    USER_UI_PREFS
        .lock()
        .ok()
        .and_then(|m| m.get(&user_id).map(|(_, av)| av.clone()))
        .filter(|s| !s.trim().is_empty())
}

#[derive(Debug, Clone)]
pub struct AdminContext {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub nonce: String,
    pub admin_color: String,
    pub locale: String,
    pub rich_editing: bool,
    pub avatar_url: String,
}
#[allow(dead_code)]
struct UnusedAdminCtx {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub nonce: String,
}

tokio::task_local! {
    pub static ADMIN_CONTEXT: AdminContext;
}

/// Returns the admin context of the current request (only set inside `/d-admin/*`).
pub fn current_admin() -> Option<AdminContext> {
    ADMIN_CONTEXT.try_with(|c| c.clone()).ok()
}

fn path_requires_admin_session(path: &str) -> bool {
    if path == "/studio" || path.starts_with("/studio/") {
        return true;
    }
    if path != "/d-admin" && !path.starts_with("/d-admin/") {
        return false;
    }
    !matches!(
        path,
        "/d-admin/login"
            | "/d-admin/login/2fa"
            | "/d-admin/logout"
            | "/d-admin/install"
            | "/d-admin/setup"
    )
}

fn is_admin_machine_endpoint(path: &str) -> bool {
    path.starts_with("/d-admin/api/")
}

/// Admin screens a role may open. Administrators reach everything; content roles reach the
/// content screens permitted for their role; everyone else (subscribers, unknown roles) only
/// their own profile.
fn admin_path_allowed(path: &str, user: &AuthUser) -> bool {
    if user.is_admin() {
        return true;
    }
    let path = path.trim_end_matches('/');
    if path == "/d-admin/profile" || path == "/d-admin/user-edit" {
        return true;
    }
    match path {
        "/d-admin/upload" | "/d-admin/media-new" => user.can_upload_files(),
        "/d-admin/edit-tags" => user.can_manage_categories(),
        "/d-admin/edit-comments" => user.can_moderate_comments(),
        "/d-admin" | "/d-admin/index" | "/d-admin/edit" | "/d-admin/post-new" | "/d-admin/post"
        | "/d-admin/builder" | "/d-admin/studio" | "/studio" => user.can_edit_content(),
        _ => false,
    }
}

fn url_encode_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() * 3);
    for b in raw.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn apply_baseline_security_headers(resp: &mut axum::response::Response) {
    let headers = resp.headers_mut();
    if !headers.contains_key(axum::http::header::X_CONTENT_TYPE_OPTIONS) {
        if let Ok(v) = "nosniff".parse() {
            headers.insert(axum::http::header::X_CONTENT_TYPE_OPTIONS, v);
        }
    }
    if !headers.contains_key(axum::http::header::X_FRAME_OPTIONS) {
        if let Ok(v) = "SAMEORIGIN".parse() {
            headers.insert(axum::http::header::X_FRAME_OPTIONS, v);
        }
    }
    if !headers.contains_key(axum::http::header::REFERRER_POLICY) {
        if let Ok(v) = "strict-origin-when-cross-origin".parse() {
            headers.insert(axum::http::header::REFERRER_POLICY, v);
        }
    }
    if !headers.contains_key("permissions-policy") {
        if let Ok(v) = "camera=(), microphone=(), geolocation=(), payment=()".parse() {
            headers.insert("permissions-policy", v);
        }
    }
    if !headers.contains_key("cross-origin-opener-policy") {
        if let Ok(v) = "same-origin".parse() {
            headers.insert("cross-origin-opener-policy", v);
        }
    }
    if !headers.contains_key(axum::http::header::CONTENT_SECURITY_POLICY) {
        if let Ok(v) = "frame-ancestors 'self'; base-uri 'self'; object-src 'none'".parse() {
            headers.insert(axum::http::header::CONTENT_SECURITY_POLICY, v);
        }
    }
}

/// Axum middleware: every admin screen and `/d-admin/api/*` endpoint requires a valid,
/// signed login session. Unauthenticated page loads are redirected to `/login`
/// (with `redirect_to`), API calls receive `401`. Screens outside the user's role
/// (see [`admin_path_allowed`]) redirect to the profile, or return `403` for API calls.
pub async fn admin_session_gate(
    axum::extract::State(db): axum::extract::State<Database>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let path = req.uri().path().to_string();
    if !path_requires_admin_session(&path) {
        let mut resp = next.run(req).await;
        apply_baseline_security_headers(&mut resp);
        return resp;
    }

    // Same credentials as the REST API: signed session cookie, Basic (user or application
    // password), or Bearer JWT.
    let (parts, body) = req.into_parts();
    let user = authenticate_request(&parts, &db).await.ok();
    let req = axum::extract::Request::from_parts(parts, body);

    let user = match user {
        Some(u) => u,
        None => {
            if is_admin_machine_endpoint(&path) {
                return (
                    axum::http::StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({
                        "code": "rest_not_logged_in",
                        "message": "Your session has expired. Please log in again."
                    })),
                )
                    .into_response();
            }
            if db.needs_onboarding() {
                return axum::response::Redirect::to("/setup").into_response();
            }
            let target = match req.uri().query() {
                Some(q) => format!("{}?{}", path, q),
                None => path.clone(),
            };
            let location = format!("/login?redirect_to={}", url_encode_component(&target));
            return axum::response::Redirect::to(&location).into_response();
        }
    };

    if !admin_path_allowed(&path, &user) {
        if is_admin_machine_endpoint(&path) {
            return (
                axum::http::StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "code": "rest_forbidden",
                    "message": "Sorry, you are not allowed to do that."
                })),
            )
                .into_response();
        }
        return axum::response::Redirect::to("/d-admin/profile").into_response();
    }

    let detail = db.get_user_detail(user.user_id).ok().flatten();
    let display_name = detail
        .as_ref()
        .map(|d| d.display_name.clone())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| user.username.clone());
    let secret = db.get_auth_secret().unwrap_or_default();
    let ctx = AdminContext {
        user_id: user.user_id,
        username: user.username.clone(),
        display_name,
        role: user.role.clone(),
        nonce: create_rest_nonce("wp_rest", user.user_id, &secret),
        admin_color: detail
            .as_ref()
            .map(|d| d.admin_color.clone())
            .unwrap_or_else(|| "default".to_string()),
        locale: detail
            .as_ref()
            .map(|d| d.locale.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                db.get_option("WPLANG")
                    .ok()
                    .flatten()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| db.get_default_language_code()),
        rich_editing: detail.as_ref().map(|d| d.rich_editing).unwrap_or(true),
        avatar_url: detail
            .as_ref()
            .map(|d| d.avatar_url.clone())
            .unwrap_or_default(),
    };

    let mut req = req;
    req.extensions_mut().insert(user);
    let mut resp = ADMIN_CONTEXT.scope(ctx, next.run(req)).await;
    apply_baseline_security_headers(&mut resp);
    resp
}

/// Only allow local, same-site redirect targets after login (prevents open redirects).
pub fn sanitize_redirect_target(raw: Option<&str>) -> String {
    let candidate = raw.unwrap_or("").trim();
    let is_local = candidate.starts_with('/')
        && !candidate.starts_with("//")
        && !candidate.starts_with("/\\")
        && !candidate.contains(['\r', '\n']);
    if is_local {
        candidate.to_string()
    } else {
        "/d-admin/".to_string()
    }
}

impl FromRequestParts<Database> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Database,
    ) -> Result<Self, Self::Rejection> {
        authenticate_request(parts, state).await
    }
}

impl FromRequestParts<Database> for EditorUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Database,
    ) -> Result<Self, Self::Rejection> {
        let user = authenticate_request(parts, state).await?;
        if !user.can_edit_content() {
            return Err(AppError::Forbidden(
                "Sorry, you are not allowed to edit content.".to_string(),
            ));
        }
        Ok(EditorUser(user))
    }
}

impl FromRequestParts<Database> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Database,
    ) -> Result<Self, Self::Rejection> {
        let user = authenticate_request(parts, state).await?;
        if !user.is_admin() {
            return Err(AppError::Forbidden(
                "Sorry, you are not allowed to manage this site.".to_string(),
            ));
        }
        Ok(AdminUser(user))
    }
}

impl FromRequestParts<Database> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Database,
    ) -> Result<Self, Self::Rejection> {
        match authenticate_request(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_generation_and_verification() {
        let secret = "test-secret-key-123456";
        let nonce = create_rest_nonce("wp_rest", 1, secret);
        assert_eq!(nonce.len(), 10);
        assert!(verify_rest_nonce(&nonce, "wp_rest", 1, secret));
        assert!(!verify_rest_nonce("invalid-nonce", "wp_rest", 1, secret));
        assert!(!verify_rest_nonce(&nonce, "other_action", 1, secret));
        assert!(!verify_rest_nonce(&nonce, "wp_rest", 2, secret));
    }

    #[test]
    fn test_argon2_hashing_and_legacy_hash_compatibility() {
        let hash = hash_user_password("CorrectHorseBatteryStaple!");
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_user_password("CorrectHorseBatteryStaple!", &hash));
        assert!(!verify_user_password("wrong", &hash));
        assert!(!password_needs_rehash(&hash));

        let legacy = legacy_sha256_password_hash("old-password");
        assert!(verify_user_password("old-password", &legacy));
        assert!(password_needs_rehash(&legacy));
        assert!(!verify_user_password("admin", LEGACY_SEED_PASSWORD_HASH));
    }

    #[test]
    fn test_login_throttle_locks_after_repeated_failures() {
        let throttle = LoginThrottle::default();
        let key = LoginThrottle::key(Some("203.0.113.7"), "Admin");
        for _ in 0..MAX_FAILED_LOGINS {
            assert!(throttle.retry_after(&key).is_none());
            throttle.record_failure(&key);
        }
        assert!(throttle.retry_after(&key).is_some_and(|s| s > 0));
        // Other clients are unaffected, and a success clears the counter.
        assert!(throttle
            .retry_after(&LoginThrottle::key(Some("198.51.100.1"), "admin"))
            .is_none());
        throttle.record_success(&key);
        assert!(throttle.retry_after(&key).is_none());
    }

    #[test]
    fn test_application_password_generation_and_normalization() {
        let app_pass = generate_application_password();
        assert_eq!(app_pass.split(' ').count(), 4);
        let normalized = normalize_app_password(&app_pass);
        assert_eq!(normalized.len(), 16);
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let secret = "super-secret-jwt-key";
        let token = create_jwt_token(1, "admin", "http://localhost:8090", secret, 7);
        let claims = verify_jwt_token(&token, secret).expect("Token verification failed");
        assert_eq!(claims.data.user.id, 1);
        assert_eq!(claims.data.user.username, "admin");

        // Fails with wrong secret
        assert!(verify_jwt_token(&token, "wrong-secret").is_err());
    }

    #[test]
    fn test_origin_matches_request_host_behind_reverse_proxy() {
        // Daypress Pro Edge -> Supervisor -> child process (`Host: 127.0.0.1:19001` + `X-Forwarded-Host: grezzo.onlyforward.daypress.dev`)
        assert!(origin_matches_request_host(
            "https://grezzo.onlyforward.daypress.dev",
            Some("127.0.0.1:19001"),
            Some("grezzo.onlyforward.daypress.dev"),
        ));
        // Chained X-Forwarded-Host header
        assert!(origin_matches_request_host(
            "https://grezzo.onlyforward.daypress.dev",
            Some("127.0.0.1:19001"),
            Some("grezzo.onlyforward.daypress.dev:443, 10.0.0.4"),
        ));
        // Local dev port preservation
        assert!(origin_matches_request_host(
            "http://grezzo.localhost:8080",
            Some("127.0.0.1:19001"),
            Some("grezzo.localhost:8080"),
        ));
        // Cross-origin attacker is rejected even behind reverse proxy
        assert!(!origin_matches_request_host(
            "https://attacker.example",
            Some("127.0.0.1:19001"),
            Some("grezzo.onlyforward.daypress.dev"),
        ));
    }
}
