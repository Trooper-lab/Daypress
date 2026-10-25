use crate::elementor::NavMenuItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A service item from the `spb_service` custom post type used by the service price calculator.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SpbServiceItem {
    pub id: i64,
    pub title: String,
    pub price: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub is_extra: bool,
}

/// Site-wide resolution context for Divi dynamic attributes (`@ET-DC@...`),
/// global colors (`gcid-*`), menus (`et_pb_menu`), Contact Form 7 forms (`[contact-form-7]`,
/// `[dvppl_cf7_styler]`), and custom Divi modules (`[spb_kw_calculator_module]`).
#[derive(Debug, Clone, Default)]
pub struct DiviContext {
    pub post_urls: HashMap<String, String>,
    pub menus: HashMap<String, Vec<NavMenuItem>>,
    pub cf7_forms: HashMap<String, String>,
    pub spb_services: Vec<SpbServiceItem>,
    pub source_site_url: Option<String>,
}

impl DiviContext {
    pub fn from_meta(meta: &HashMap<String, String>) -> Self {
        let mut ctx = Self::default();
        if let Some(urls_json) = meta.get("__np_post_urls_json") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(urls_json) {
                ctx.post_urls = map;
            }
        }
        if let Some(menus_json) = meta.get("__np_menus_json") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<NavMenuItem>>>(menus_json) {
                ctx.menus = map;
            }
        }
        if let Some(cf7_json) = meta.get("__np_cf7_forms_json") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(cf7_json) {
                ctx.cf7_forms = map;
            }
        }
        if let Some(spb_json) = meta.get("__np_spb_services_json") {
            if let Ok(list) = serde_json::from_str::<Vec<SpbServiceItem>>(spb_json) {
                ctx.spb_services = list;
            }
        }
        if let Some(src) = meta.get("__np_source_site_url").filter(|s| !s.is_empty()) {
            ctx.source_site_url = Some(src.trim_end_matches('/').to_string());
        }
        ctx
    }

    pub fn normalize_url(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        if let Some(ref base) = self.source_site_url {
            let https_base = base.replacen("http://", "https://", 1);
            let http_base = base.replacen("https://", "http://", 1);
            for candidate in [&https_base, &http_base] {
                if trimmed == candidate.as_str() {
                    return "/".to_string();
                }
                let with_slash = format!("{}/", candidate);
                if let Some(rest) = trimmed.strip_prefix(&with_slash) {
                    if let Some(media_rest) = rest.strip_prefix("wp-content/uploads/") {
                        return format!("/uploads/{}", media_rest);
                    }
                    return format!("/{}", rest);
                }
            }
            if let Some(host) = crate::url_utils::extract_url_host(base) {
                if let Some(raw_host) = crate::url_utils::extract_url_host(trimmed) {
                    if raw_host.eq_ignore_ascii_case(host) {
                        if let Some(idx) = trimmed.find("/wp-content/uploads/") {
                            return format!(
                                "/uploads/{}",
                                &trimmed[idx + "/wp-content/uploads/".len()..]
                            );
                        }
                    }
                }
            }
        }
        // Normalize only root-relative or relative `/wp-content/uploads/` paths to `/uploads/`
        if let Some(idx) = trimmed.find("/wp-content/uploads/") {
            if trimmed.starts_with('/') && !trimmed.starts_with("//") {
                let media_path = &trimmed[idx + "/wp-content/uploads/".len()..];
                return format!("/uploads/{}", media_path);
            }
        }
        if let Some(rest) = trimmed.strip_prefix("wp-content/uploads/") {
            return format!("/uploads/{}", rest);
        }
        trimmed.to_string()
    }

    pub fn normalize_html(&self, raw_html: &str) -> String {
        let mut out = if let Some(ref base) = self.source_site_url {
            let clean_base = base.trim_end_matches('/');
            let https_base = format!("{}/", clean_base.replacen("http://", "https://", 1));
            let http_base = format!("{}/", clean_base.replacen("https://", "http://", 1));
            let mut s = raw_html
                .replace(&format!("{}/wp-content/uploads/", clean_base), "/uploads/")
                .replace(&https_base, "/")
                .replace(&http_base, "/");
            if let Some(host) = crate::url_utils::extract_url_host(clean_base) {
                s = s.replace(
                    &format!("https://{}/wp-content/uploads/", host),
                    "/uploads/",
                );
                s = s.replace(&format!("http://{}/wp-content/uploads/", host), "/uploads/");
            }
            s
        } else {
            raw_html.to_string()
        };
        out = crate::url_utils::remap_root_relative_uploads(&out);
        // Expand any embedded [contact-form-7 ...] shortcodes inside Divi text modules
        if out.contains("[contact-form-7") && !self.cf7_forms.is_empty() {
            out = crate::divi::widgets::expand_cf7_shortcodes_in_html(&out, &self.cf7_forms);
        }
        out
    }

    /// Resolves Divi `@ET-DC@<base64>@` dynamic attributes and normalizes site URLs.
    pub fn resolve_divi_attr(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(inner) = trimmed
            .strip_prefix("@ET-DC@")
            .and_then(|s| s.strip_suffix('@'))
        {
            if let Some(decoded_bytes) = decode_base64_simple(inner) {
                if let Ok(json_str) = String::from_utf8(decoded_bytes) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        let content_type = val
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        match content_type {
                            "home_url" => return "/".to_string(),
                            "post_link_url_page" | "post_link_url_post" | "post_link_url"
                            | "custom_meta_link" => {
                                if let Some(pid) = val
                                    .get("settings")
                                    .and_then(|s| s.get("post_id"))
                                    .and_then(|p| {
                                        p.as_str()
                                            .map(|s| s.to_string())
                                            .or_else(|| p.as_i64().map(|n| n.to_string()))
                                    })
                                {
                                    if let Some(url) = self.post_urls.get(&pid) {
                                        return url.clone();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        self.normalize_url(trimmed)
    }
}

pub(crate) fn decode_base64_simple(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u8 = 0;
    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

/// Resolves Divi Global Color IDs (`gcid-*`) into CSS hex/var colors.
pub fn resolve_divi_color(raw: &str) -> String {
    let trimmed = raw.trim();
    match trimmed {
        "gcid-primary-color" => "#e23212".to_string(),
        "gcid-secondary-color" => "#101523".to_string(),
        "gcid-b0c0bd74-c2d9-4a33-8fc8-1f11eab87211" => "#fcfbfc".to_string(),
        "gcid-f5b5c1a3-263d-4635-a170-86276049a28b" => "#101523".to_string(),
        "gcid-heading-color" | "gcid-body-color" => "#101523".to_string(),
        other if other.starts_with("gcid-") => "#e23212".to_string(),
        other => other.to_string(),
    }
}

/// Decodes Divi's URL-encoded `global_colors_info` attribute (e.g. `{%22gcid-primary-color%22:%91%22icon_color%22%93}`)
/// and populates any missing color attributes on the shortcode map.
pub(crate) fn apply_divi_global_colors_info(attrs: &mut HashMap<String, String>) {
    let Some(raw_info) = attrs.get("global_colors_info").cloned() else {
        return;
    };
    if raw_info.is_empty() || raw_info == "{}" {
        return;
    }
    let decoded = raw_info
        .replace("%22", "\"")
        .replace("%91", "[")
        .replace("%93", "]")
        .replace("%5B", "[")
        .replace("%5D", "]")
        .replace("%7B", "{")
        .replace("%7D", "}");
    if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<String>>>(&decoded) {
        for (gcid, props) in map {
            let resolved = resolve_divi_color(&gcid);
            for prop in props {
                attrs.entry(prop).or_insert_with(|| resolved.clone());
            }
        }
    }
}
