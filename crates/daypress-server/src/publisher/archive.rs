//! Pure-Rust ZIP archive writer/reader and cryptographic digest helpers (SHA-256 & SHA-1)
//! for Static Edge bundles and 1-Click Deployment Rollbacks.

use flate2::write::DeflateEncoder;
use flate2::{Compression, Crc};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Read, Write};

/// Computes lowercase hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Computes lowercase hex FIPS 180-4 SHA-1 of `bytes` (used by Netlify's File Digest API).
pub fn sha1_hex(bytes: &[u8]) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(bytes.len() + 72);
    padded.extend_from_slice(bytes);
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.as_chunks::<64>().0 {
        let mut w = [0u32; 80];
        for (i, word) in chunk.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
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
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
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
    hex::encode(out)
}

/// Builds a standard PKZIP archive from an ordered map of `relative_path -> bytes`.
pub fn build_zip_archive(files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    struct CentralRecord {
        name_bytes: Vec<u8>,
        method: u16,
        crc32: u32,
        compressed_size: u32,
        uncompressed_size: u32,
        local_header_offset: u32,
    }

    let mut out = Vec::new();
    let mut central = Vec::with_capacity(files.len());

    for (raw_path, data) in files {
        let clean_path = raw_path.trim_start_matches('/').replace('\\', "/");
        if clean_path.is_empty() {
            continue;
        }
        let name_bytes = clean_path.into_bytes();
        let offset = out.len() as u32;

        let mut crc = Crc::new();
        crc.update(data);
        let crc_val = crc.sum();
        let uncompressed_size = data.len() as u32;

        // Deflate text/HTML/CSS/JS/SVG/JSON; store already-compressed or tiny files
        let (method, payload) = if data.len() > 64 {
            let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
            if enc.write_all(data).is_ok() {
                if let Ok(compressed) = enc.finish() {
                    if compressed.len() < data.len() {
                        (8u16, compressed)
                    } else {
                        (0u16, data.clone())
                    }
                } else {
                    (0u16, data.clone())
                }
            } else {
                (0u16, data.clone())
            }
        } else {
            (0u16, data.clone())
        };

        let compressed_size = payload.len() as u32;

        // Local file header (PK\x03\x04)
        out.extend_from_slice(&0x04034b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed (2.0)
        out.extend_from_slice(&0x0800u16.to_le_bytes()); // UTF-8 flag
        out.extend_from_slice(&method.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // mod time
        out.extend_from_slice(&0x5421u16.to_le_bytes()); // mod date
        out.extend_from_slice(&crc_val.to_le_bytes());
        out.extend_from_slice(&compressed_size.to_le_bytes());
        out.extend_from_slice(&uncompressed_size.to_le_bytes());
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra len
        out.extend_from_slice(&name_bytes);
        out.extend_from_slice(&payload);

        central.push(CentralRecord {
            name_bytes,
            method,
            crc32: crc_val,
            compressed_size,
            uncompressed_size,
            local_header_offset: offset,
        });
    }

    let cd_offset = out.len() as u32;
    for rec in &central {
        // Central directory file header (PK\x01\x02)
        out.extend_from_slice(&0x02014b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes()); // version made by
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0x0800u16.to_le_bytes()); // UTF-8 flag
        out.extend_from_slice(&rec.method.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // mod time
        out.extend_from_slice(&0x5421u16.to_le_bytes()); // mod date
        out.extend_from_slice(&rec.crc32.to_le_bytes());
        out.extend_from_slice(&rec.compressed_size.to_le_bytes());
        out.extend_from_slice(&rec.uncompressed_size.to_le_bytes());
        out.extend_from_slice(&(rec.name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra len
        out.extend_from_slice(&0u16.to_le_bytes()); // comment len
        out.extend_from_slice(&0u16.to_le_bytes()); // disk number
        out.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        out.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        out.extend_from_slice(&rec.local_header_offset.to_le_bytes());
        out.extend_from_slice(&rec.name_bytes);
    }

    let cd_size = (out.len() as u32).saturating_sub(cd_offset);
    let count = central.len() as u16;

    // End of central directory record (PK\x05\x06)
    out.extend_from_slice(&0x06054b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // disk number
    out.extend_from_slice(&0u16.to_le_bytes()); // disk with CD
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment len

    out
}

/// Extracts a ZIP archive built by `build_zip_archive` back into a `BTreeMap<String, Vec<u8>>`
/// (used when rolling back to a previous deployment snapshot).
pub fn read_zip_archive(zip_bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    let mut pos = 0usize;

    while pos + 4 <= zip_bytes.len() {
        let sig = u32::from_le_bytes([
            zip_bytes[pos],
            zip_bytes[pos + 1],
            zip_bytes[pos + 2],
            zip_bytes[pos + 3],
        ]);
        if sig != 0x04034b50 {
            break;
        }
        if pos + 30 > zip_bytes.len() {
            return Err("Truncated local file header in ZIP".to_string());
        }
        let method = u16::from_le_bytes([zip_bytes[pos + 8], zip_bytes[pos + 9]]);
        let comp_size = u32::from_le_bytes([
            zip_bytes[pos + 18],
            zip_bytes[pos + 19],
            zip_bytes[pos + 20],
            zip_bytes[pos + 21],
        ]) as usize;
        let name_len = u16::from_le_bytes([zip_bytes[pos + 26], zip_bytes[pos + 27]]) as usize;
        let extra_len = u16::from_le_bytes([zip_bytes[pos + 28], zip_bytes[pos + 29]]) as usize;

        let name_start = pos + 30;
        let data_start = name_start + name_len + extra_len;
        let data_end = data_start + comp_size;
        if data_end > zip_bytes.len() {
            return Err("Truncated file entry payload in ZIP".to_string());
        }

        let name =
            String::from_utf8_lossy(&zip_bytes[name_start..name_start + name_len]).to_string();
        let raw_payload = &zip_bytes[data_start..data_end];

        let decoded = match method {
            0 => raw_payload.to_vec(),
            8 => {
                let mut decoder = flate2::read::DeflateDecoder::new(raw_payload);
                let mut buf = Vec::new();
                decoder
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Deflate decode error for {name}: {e}"))?;
                buf
            }
            other => {
                return Err(format!(
                    "Unsupported ZIP compression method {other} for {name}"
                ))
            }
        };

        files.insert(name, decoded);
        pos = data_end;
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_and_zip_roundtrip() {
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        let mut map = BTreeMap::new();
        map.insert(
            "index.html".to_string(),
            b"<!DOCTYPE html><html><body><h1>Hello Daypress Static Edge!</h1></body></html>"
                .repeat(4),
        );
        map.insert(
            "robots.txt".to_string(),
            b"User-agent: *\nAllow: /\n".to_vec(),
        );

        let zip = build_zip_archive(&map);
        let unpacked = read_zip_archive(&zip).expect("valid zip roundtrip");
        assert_eq!(unpacked, map);
    }
}
