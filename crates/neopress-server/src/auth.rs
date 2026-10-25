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

/// Hashes a user password with salted SHA-256 in `$neopress$sha256$<salt>$<hash>` format.
pub fn hash_user_password(password: &str) -> String {
    let raw = uuid_like_random_string();
    let salt: String = raw.chars().take(16).collect();
    let salted = format!("{}:{}", salt, password);
    let hash = sha256_hex(salted.as_bytes());
    format!("$neopress$sha256${}${}", salt, hash)
}

/// Placeholder hash written by older seeds for the initial `admin` account.
/// It matches no password; [`crate::db::Database::ensure_admin_credentials`] replaces it on startup.
pub const LEGACY_SEED_PASSWORD_HASH: &str = "$2y$10$abcdefghijklmnopqrstuvwxyz1234567890";

/// Verifies a user password against a stored password hash.
/// Only Neopress hashes are accepted: imported WordPress (`$P$` / `$wp$`) hashes, seed
/// placeholders, and plaintext values never authenticate.
pub fn verify_user_password(password: &str, stored_pass: &str) -> bool {
    if password.is_empty() {
        return false;
    }
    if let Some(rest) = stored_pass.strip_prefix("$neopress$sha256$") {
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

/// Generates pseudo-random alphanumeric string from time & entropy.
///
/// Mixes the wall clock with the OS-seeded SipHash keys of `RandomState`
/// (128 bits of process randomness) so values are not predictable from a timestamp alone.
fn uuid_like_random_string() -> String {
    random_hex_64().chars().take(24).collect()
}

/// Returns 64 hex characters of OS-seeded randomness (no extra crate dependency).
pub fn random_hex_64() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut material = String::with_capacity(96);
    for i in 0..4u64 {
        let mut h = RandomState::new().build_hasher();
        h.write_u128(nanos);
        h.write_u64(i);
        h.write_usize(&material as *const String as usize);
        material.push_str(&format!("{:016x}", h.finish()));
    }
    sha256_hex(material.as_bytes())
}

/// Name of the signed admin session cookie.
pub const SESSION_COOKIE: &str = "neopress_admin";

/// Session lifetime when "Remember Me" is ticked (14 days, WordPress parity).
pub const SESSION_TTL_REMEMBER_SECS: i64 = 14 * 86400;
/// Session lifetime for a regular login (2 days, WordPress parity).
pub const SESSION_TTL_DEFAULT_SECS: i64 = 2 * 86400;

/// Creates a tamper-proof session token: `<user_id>.<expires_unix>.<hmac_hex>`.
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

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Builds the `Set-Cookie` header value for a freshly authenticated user.
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
    let token = create_session_token(user_id, &secret, SESSION_TTL_DEFAULT_SECS);
    format!("{}={}", SESSION_COOKIE, token)
}

/// Resolves the logged-in user from the signed `neopress_admin` session cookie.
/// Unsigned, forged, or expired values are rejected.
pub fn session_user_from_cookie_header(cookie_header: &str, db: &Database) -> Option<AuthUser> {
    let secret = db.get_auth_secret().ok()?;
    for item in cookie_header.split(';') {
        let trimmed = item.trim();
        if let Some(val) = trimmed.strip_prefix("neopress_admin=") {
            if let Some(uid) = verify_session_token(val, &secret) {
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
    /// Full site management: settings, users, plugins, imports, theme templates.
    pub fn is_admin(&self) -> bool {
        self.role.trim().eq_ignore_ascii_case("administrator")
    }

    /// Content management: posts, pages, media, terms, comments, and layouts.
    pub fn can_edit_content(&self) -> bool {
        matches!(
            self.role.trim().to_ascii_lowercase().as_str(),
            "administrator" | "editor" | "author" | "contributor"
        )
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
pub fn create_wp_nonce(action: &str, user_id: i64, secret: &str) -> String {
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
pub fn verify_wp_nonce(nonce: &str, action: &str, user_id: i64, secret: &str) -> bool {
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
                            if let Some(auth_user) =
                                db.authenticate_credentials(user.trim(), pass.trim())?
                            {
                                return Ok(auth_user);
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
                return Ok(user);
            }
        }
    }

    Err(AppError::Unauthorized(
        "Sorry, you are not allowed to create posts as this user.".to_string(),
    ))
}

// =========================================================================
// ADMIN SESSION GATE (/n-admin/*)
// =========================================================================

/// Per-request context of the logged-in admin, available to SSR templates
/// (top bar "Howdy, …", REST nonce) without threading it through every renderer.
#[derive(Debug, Clone)]
pub struct AdminContext {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub nonce: String,
}

tokio::task_local! {
    pub static ADMIN_CONTEXT: AdminContext;
}

/// Returns the admin context of the current request (only set inside `/n-admin/*`).
pub fn current_admin() -> Option<AdminContext> {
    ADMIN_CONTEXT.try_with(|c| c.clone()).ok()
}

fn path_requires_admin_session(path: &str) -> bool {
    if path == "/studio" || path.starts_with("/studio/") {
        return true;
    }
    if path != "/n-admin" && !path.starts_with("/n-admin/") {
        return false;
    }
    !matches!(path, "/n-admin/login" | "/n-admin/logout")
}

fn is_admin_machine_endpoint(path: &str) -> bool {
    path.starts_with("/n-admin/api/")
}

/// Admin screens a role may open. Administrators reach everything; content roles reach the
/// content screens; everyone else (subscribers, unknown roles) only their own profile.
fn admin_path_allowed(path: &str, user: &AuthUser) -> bool {
    if user.is_admin() {
        return true;
    }
    let path = path.trim_end_matches('/');
    if path == "/n-admin/profile" {
        return true;
    }
    user.can_edit_content()
        && matches!(
            path,
            "/n-admin"
                | "/n-admin/index"
                | "/n-admin/edit"
                | "/n-admin/post-new"
                | "/n-admin/post"
                | "/n-admin/builder"
                | "/n-admin/studio"
                | "/studio"
                | "/n-admin/upload"
                | "/n-admin/media-new"
                | "/n-admin/edit-tags"
                | "/n-admin/edit-comments"
        )
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

/// Axum middleware: every admin screen and `/n-admin/api/*` endpoint requires a valid,
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
        return next.run(req).await;
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
        return axum::response::Redirect::to("/n-admin/profile").into_response();
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
        nonce: create_wp_nonce("wp_rest", user.user_id, &secret),
    };

    let mut req = req;
    req.extensions_mut().insert(user);
    ADMIN_CONTEXT.scope(ctx, next.run(req)).await
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
        "/n-admin/".to_string()
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
        let nonce = create_wp_nonce("wp_rest", 1, secret);
        assert_eq!(nonce.len(), 10);
        assert!(verify_wp_nonce(&nonce, "wp_rest", 1, secret));
        assert!(!verify_wp_nonce("invalid-nonce", "wp_rest", 1, secret));
        assert!(!verify_wp_nonce(&nonce, "other_action", 1, secret));
        assert!(!verify_wp_nonce(&nonce, "wp_rest", 2, secret));
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
}
