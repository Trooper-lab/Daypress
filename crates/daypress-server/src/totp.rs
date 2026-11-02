//! RFC 6238 Time-Based One-Time Password (TOTP) and single-use backup recovery codes.
//!
//! Uses a pure safe Rust implementation of RFC 3174 SHA-1, RFC 2104 HMAC-SHA1, and RFC 4648
//! Base32 so authenticator apps (Google Authenticator, 1Password, Authy, Bitwarden, Apple
//! Passwords) work out of the box with zero extra external dependencies.

use sha2::{Digest, Sha256};

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
pub const TOTP_STEP_SECS: i64 = 30;
const TOTP_DIGITS_MOD: u32 = 1_000_000;

/// Encodes raw bytes as uppercase RFC 4648 Base32 without `=` padding.
pub fn base32_encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity((data.len() * 8).div_ceil(5));
    let mut buffer: u32 = 0;
    let mut bits_left: u32 = 0;
    for &byte in data {
        buffer = (buffer << 8) | u32::from(byte);
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            let idx = ((buffer >> bits_left) & 0x1f) as usize;
            out.push(BASE32_ALPHABET[idx] as char);
        }
    }
    if bits_left > 0 {
        let idx = ((buffer << (5 - bits_left)) & 0x1f) as usize;
        out.push(BASE32_ALPHABET[idx] as char);
    }
    out
}

/// Decodes an RFC 4648 Base32 string (case-insensitive, ignoring spaces, hyphens, and `=`).
pub fn base32_decode(encoded: &str) -> Option<Vec<u8>> {
    let mut buffer: u32 = 0;
    let mut bits_left: u32 = 0;
    let mut out = Vec::with_capacity(encoded.len() * 5 / 8);

    for ch in encoded.chars() {
        if ch == '=' || ch == ' ' || ch == '-' {
            continue;
        }
        let val: u32 = match ch.to_ascii_uppercase() {
            'A'..='Z' => (ch.to_ascii_uppercase() as u8 - b'A') as u32,
            '2'..='7' => (ch as u8 - b'2' + 26) as u32,
            _ => return None,
        };
        buffer = (buffer << 5) | val;
        bits_left += 5;
        if bits_left >= 8 {
            bits_left -= 8;
            out.push(((buffer >> bits_left) & 0xff) as u8);
        }
    }
    Some(out)
}

/// Pure safe Rust RFC 3174 SHA-1 digest (20 bytes), used exclusively for RFC 4226 / RFC 6238
/// HMAC-SHA1 TOTP compatibility with standard authenticator apps.
pub fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x6745_2301;
    let mut h1: u32 = 0xEFCD_AB89;
    let mut h2: u32 = 0x98BA_DCFE;
    let mut h3: u32 = 0x1032_5476;
    let mut h4: u32 = 0xC3D2_E1F0;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity((data.len() + 9).div_ceil(64) * 64);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 80];
        for (i, word_bytes) in chunk.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word_bytes);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDCu32),
                _ => (b ^ c ^ d, 0xCA62_C1D6u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

/// RFC 2104 HMAC-SHA1 over `message` with `key`.
pub fn hmac_sha1(key: &[u8], message: &[u8]) -> [u8; 20] {
    let mut k_block = [0u8; 64];
    if key.len() > 64 {
        let hashed = sha1_digest(key);
        k_block[..20].copy_from_slice(&hashed);
    } else {
        k_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k_block[i];
        opad[i] ^= k_block[i];
    }

    let mut inner = Vec::with_capacity(64 + message.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(message);
    let inner_hash = sha1_digest(&inner);

    let mut outer = [0u8; 84];
    outer[..64].copy_from_slice(&opad);
    outer[64..].copy_from_slice(&inner_hash);
    sha1_digest(&outer)
}

/// Generates a 160-bit (20-byte) CSPRNG secret encoded as 32 Base32 characters.
pub fn generate_totp_secret() -> String {
    let mut buf = [0u8; 20];
    getrandom::getrandom(&mut buf).expect("OS CSPRNG unavailable");
    base32_encode(&buf)
}

/// Percent-encodes a URI label or query component for an `otpauth://` URI.
fn percent_encode_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'@' => {
                out.push(*b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

/// Builds a standard `otpauth://totp/...` provisioning URI for authenticator apps.
pub fn build_otpauth_uri(secret_b32: &str, account_name: &str, issuer: &str) -> String {
    let clean_issuer = if issuer.trim().is_empty() {
        "Daypress"
    } else {
        issuer.trim()
    };
    let enc_issuer = percent_encode_component(clean_issuer);
    let enc_account = percent_encode_component(account_name.trim());
    let clean_secret: String = secret_b32
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect::<String>()
        .to_ascii_uppercase();
    format!(
        "otpauth://totp/{enc_issuer}:{enc_account}?secret={clean_secret}&issuer={enc_issuer}&algorithm=SHA1&digits=6&period=30"
    )
}

/// Computes the 6-digit RFC 4226 / RFC 6238 code for a given 64-bit time step counter.
pub fn totp_code_at_step(secret_bytes: &[u8], step: u64) -> String {
    let counter_bytes = step.to_be_bytes();
    let mac = hmac_sha1(secret_bytes, &counter_bytes);
    let offset = (mac[19] & 0x0f) as usize;
    let binary = ((u32::from(mac[offset] & 0x7f)) << 24)
        | ((u32::from(mac[offset + 1])) << 16)
        | ((u32::from(mac[offset + 2])) << 8)
        | u32::from(mac[offset + 3]);
    let otp = binary % TOTP_DIGITS_MOD;
    format!("{otp:06}")
}

/// Computes the current 6-digit TOTP code for `secret_b32` at `unix_secs`.
pub fn current_totp_code(secret_b32: &str, unix_secs: i64) -> Option<String> {
    let secret_bytes = base32_decode(secret_b32)?;
    if secret_bytes.is_empty() || unix_secs < 0 {
        return None;
    }
    let step = (unix_secs / TOTP_STEP_SECS) as u64;
    Some(totp_code_at_step(&secret_bytes, step))
}

/// Verifies a 6-digit TOTP `code` against `secret_b32` with a ±1 step window (±30s clock drift)
/// and replay protection (`last_accepted_step`).
///
/// Returns `Some(accepted_step)` when valid so the caller can persist `accepted_step` and
/// reject any replay of the same or earlier time step.
pub fn verify_totp_code(
    secret_b32: &str,
    code: &str,
    unix_secs: i64,
    last_accepted_step: Option<i64>,
) -> Option<i64> {
    let normalized: String = code
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    if normalized.len() != 6 || !normalized.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let secret_bytes = base32_decode(secret_b32)?;
    if secret_bytes.is_empty() || unix_secs < 0 {
        return None;
    }
    let current_step = unix_secs / TOTP_STEP_SECS;
    for offset in [0i64, -1, 1] {
        let candidate_step = current_step + offset;
        if candidate_step < 0 {
            continue;
        }
        if let Some(last) = last_accepted_step {
            if candidate_step <= last {
                continue;
            }
        }
        let expected = totp_code_at_step(&secret_bytes, candidate_step as u64);
        if crate::auth::constant_time_eq(normalized.as_bytes(), expected.as_bytes()) {
            return Some(candidate_step);
        }
    }
    None
}

/// Generates 8 single-use backup recovery codes formatted as `xxxx-xxxx` (lowercase hex/alnum).
pub fn generate_recovery_codes() -> Vec<String> {
    const CHARSET: &[u8; 32] = b"abcdefghjkmnpqrstuvwxyz234567892";
    let mut codes = Vec::with_capacity(8);
    for _ in 0..8 {
        let mut rnd = [0u8; 8];
        getrandom::getrandom(&mut rnd).expect("OS CSPRNG unavailable");
        let mut s = String::with_capacity(9);
        for (idx, &b) in rnd.iter().enumerate() {
            if idx == 4 {
                s.push('-');
            }
            s.push(CHARSET[(b as usize) & 0x1f] as char);
        }
        codes.push(s);
    }
    codes
}

/// Normalizes and hashes a recovery code with SHA-256 so raw recovery codes are never stored.
pub fn hash_recovery_code(raw: &str) -> String {
    let normalized: String = raw
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(b"daypress-2fa-recovery:v1:");
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_and_hmac_sha1_match_rfc_vectors() {
        // FIPS 180-1 / RFC 3174 test vector: SHA1("abc")
        let digest = sha1_digest(b"abc");
        assert_eq!(
            hex::encode(digest),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );

        // RFC 2202 test vector: HMAC-SHA1("Jefe", "what do ya want for nothing?")
        let mac = hmac_sha1(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(hex::encode(mac), "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79");
    }

    #[test]
    fn base32_roundtrip_and_rfc4648_vectors() {
        let secret = b"12345678901234567890";
        let encoded = base32_encode(secret);
        assert_eq!(encoded, "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
        let decoded = base32_decode(&encoded).expect("valid base32");
        assert_eq!(decoded, secret);
    }

    #[test]
    fn totp_matches_rfc6238_sha1_test_vectors() {
        // RFC 6238 Appendix B SHA1 seed: ASCII "12345678901234567890"
        // Note: RFC 6238 Appendix B uses 8 digits; our 6-digit code is the last 6 digits.
        let secret_b32 = base32_encode(b"12345678901234567890");
        // t = 59 -> 8-digit 94287082 -> 6-digit 287082
        assert_eq!(
            current_totp_code(&secret_b32, 59).as_deref(),
            Some("287082")
        );
        // t = 1111111109 -> 8-digit 07081804 -> 6-digit 081804
        assert_eq!(
            current_totp_code(&secret_b32, 1_111_111_109).as_deref(),
            Some("081804")
        );
        // t = 1234567890 -> 8-digit 89005924 -> 6-digit 005924
        assert_eq!(
            current_totp_code(&secret_b32, 1_234_567_890).as_deref(),
            Some("005924")
        );
    }

    #[test]
    fn totp_verification_enforces_drift_window_and_replay_protection() {
        let secret = generate_totp_secret();
        let now = 1_700_000_000i64;
        let code = current_totp_code(&secret, now).expect("code");

        // Valid within current step and ±30s drift
        let step = verify_totp_code(&secret, &code, now + 25, None).expect("accepted");
        assert_eq!(step, now / 30);

        // Replaying the same step or an earlier step is rejected
        assert!(verify_totp_code(&secret, &code, now, Some(step)).is_none());

        // Outside ±1 step window (+90s) is rejected
        assert!(verify_totp_code(&secret, &code, now + 90, None).is_none());
    }

    #[test]
    fn recovery_codes_normalize_hyphens_and_case() {
        let codes = generate_recovery_codes();
        assert_eq!(codes.len(), 8);
        for c in &codes {
            assert_eq!(c.len(), 9);
            let h1 = hash_recovery_code(c);
            let h2 = hash_recovery_code(&c.replace('-', "").to_ascii_uppercase());
            assert_eq!(h1, h2);
        }
    }
}
