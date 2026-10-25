//! URL normalization, permalink generation, and relational thumbnail/parent remapping.

use rusqlite::{params, Result, Transaction};
use std::collections::HashMap;

/// Extracts the host and optional port from a URL (e.g. "websitedemos.net" or "localhost:8085").
pub fn extract_url_host(url: &str) -> Option<&str> {
    let after_scheme = url.split("://").nth(1)?;
    let host_end = after_scheme
        .find(['/', '?', '#'])
        .unwrap_or(after_scheme.len());
    let host = &after_scheme[..host_end];
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

/// Remaps a single URL against the source site URL or relative uploads paths,
/// preserving external links (e.g. GitHub, Twitter, Google Fonts).
pub fn remap_single_url(raw: &str, source_site_url: Option<&str>) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('#') || trimmed.starts_with("mailto:") || trimmed.starts_with("tel:") {
        return trimmed.to_string();
    }

    if let Some(src) = source_site_url.map(|s| s.trim_end_matches('/')) {
        let https_base = src.replacen("http://", "https://", 1);
        let http_base = src.replacen("https://", "http://", 1);

        for base in [&https_base, &http_base] {
            if trimmed == base.as_str() {
                return "/".to_string();
            }
            let with_slash = format!("{}/", base);
            if let Some(rest) = trimmed.strip_prefix(&with_slash) {
                if let Some(media_rest) = rest.strip_prefix("wp-content/uploads/") {
                    return format!("/uploads/{}", media_rest);
                }
                return format!("/{}", rest);
            }
        }

        // Check if on same host domain with /wp-content/uploads/
        if let Some(src_host) = extract_url_host(src) {
            if let Some(raw_host) = extract_url_host(trimmed) {
                if raw_host.eq_ignore_ascii_case(src_host) {
                    if let Some(idx) = trimmed.find("/wp-content/uploads/") {
                        return format!(
                            "/uploads/{}",
                            &trimmed[idx + "/wp-content/uploads/".len()..]
                        );
                    }
                }
            }
        }
    } else if trimmed.starts_with("http://localhost") || trimmed.starts_with("http://127.0.0.1") {
        if let Some(idx) = trimmed
            .find("://")
            .and_then(|i| trimmed[i + 3..].find('/').map(|j| i + 3 + j))
        {
            let rest = &trimmed[idx..];
            if rest.is_empty() {
                return "/".to_string();
            } else if let Some(media_rest) = rest.strip_prefix("/wp-content/uploads/") {
                return format!("/uploads/{}", media_rest);
            } else {
                return rest.to_string();
            }
        }
    }

    // Convert root-relative uploads path to /uploads/
    if let Some(idx) = trimmed.find("/wp-content/uploads/") {
        if trimmed.starts_with('/') && !trimmed.starts_with("//") {
            return format!(
                "/uploads/{}",
                &trimmed[idx + "/wp-content/uploads/".len()..]
            );
        }
    }
    if let Some(media_rest) = trimmed.strip_prefix("wp-content/uploads/") {
        return format!("/uploads/{}", media_rest);
    }

    // Preserve external links unchanged
    trimmed.to_string()
}

/// Specialized remapping wrapper for navigation menu URLs.
pub fn remap_menu_url(raw: &str, source_site_url: Option<&str>) -> String {
    let remapped = remap_single_url(raw, source_site_url);
    if remapped.is_empty() {
        "/".to_string()
    } else if remapped.starts_with('#')
        || remapped.starts_with("http://")
        || remapped.starts_with("https://")
        || remapped.starts_with("mailto:")
        || remapped.starts_with("tel:")
        || remapped.starts_with('/')
    {
        remapped
    } else {
        format!("/{}", remapped)
    }
}

/// Normalizes menu and media URLs (backwards-compatible wrapper).
pub fn normalize_menu_url(raw: &str) -> String {
    remap_menu_url(raw, None)
}

/// Remaps all occurrences of internal site URLs and media upload URLs in HTML,
/// Gutenberg block comments, or Elementor JSON payloads.
pub fn remap_content_urls(content: &str, source_site_url: Option<&str>) -> String {
    if content.is_empty() {
        return String::new();
    }
    let mut out = content.to_string();

    if let Some(src) = source_site_url.map(|s| s.trim_end_matches('/')) {
        let https_base = src.replacen("http://", "https://", 1);
        let http_base = src.replacen("https://", "http://", 1);
        let https_esc = https_base.replace('/', "\\/");
        let http_esc = http_base.replace('/', "\\/");

        // 1. Base URL media paths: base + /wp-content/uploads/ -> /uploads/
        for base in [&https_base, &http_base] {
            out = out.replace(&format!("{}/wp-content/uploads/", base), "/uploads/");
        }
        for base_esc in [&https_esc, &http_esc] {
            out = out.replace(
                &format!("{}\\/wp-content\\/uploads\\/", base_esc),
                "\\/uploads\\/",
            );
        }

        // 2. Domain-level media paths: host + /wp-content/uploads/ -> /uploads/
        if let Some(host) = extract_url_host(src) {
            out = out.replace(
                &format!("https://{}/wp-content/uploads/", host),
                "/uploads/",
            );
            out = out.replace(&format!("http://{}/wp-content/uploads/", host), "/uploads/");
            out = out.replace(
                &format!("https:\\/\\/{}\\/wp-content\\/uploads\\/", host),
                "\\/uploads\\/",
            );
            out = out.replace(
                &format!("http:\\/\\/{}\\/wp-content\\/uploads\\/", host),
                "\\/uploads\\/",
            );
        }

        // 3. Internal page/post links: base + / -> /
        for base in [&https_base, &http_base] {
            out = out.replace(&format!("{}/", base), "/");
        }
        for base_esc in [&https_esc, &http_esc] {
            out = out.replace(&format!("{}\\/", base_esc), "\\/");
        }

        // 4. Exact base URL without trailing slash (in quotes or boundaries)
        for base in [&https_base, &http_base] {
            out = out.replace(&format!("\"{}\"", base), "\"/\"");
            out = out.replace(&format!("'{}'", base), "'/'");
        }
        for base_esc in [&https_esc, &http_esc] {
            out = out.replace(&format!("\\\"{}\\\"", base_esc), "\\\"\\/\\\"");
        }
    }

    // 5. Generic root-relative uploads normalization with boundary protection
    out = neopress_ast::url_utils::remap_root_relative_uploads(&out);

    out
}

/// Convenience helper to remap post_content and postmeta in-place during Pass 1 posts ingestion.
pub fn remap_post_content_and_meta(
    content: &mut String,
    postmeta: &mut HashMap<String, String>,
    source_site_url: Option<&str>,
) {
    if !content.is_empty() {
        *content = remap_content_urls(content, source_site_url);
    }
    if let Some(elem) = postmeta.get_mut("_elementor_data") {
        *elem = remap_content_urls(elem, source_site_url);
    }
    if let Some(page_set) = postmeta.get_mut("_elementor_page_settings") {
        *page_set = remap_content_urls(page_set, source_site_url);
    }
    if let Some(m_url) = postmeta.get_mut("_menu_item_url") {
        *m_url = remap_menu_url(m_url, source_site_url);
    }
    if let Some(c_html) = postmeta.get_mut("custom_html") {
        *c_html = remap_content_urls(c_html, source_site_url);
    }
}

/// Computes standard WordPress permalinks for posts, pages, and custom post types.
pub fn compute_post_permalink(slug: &str, post_type: &str, is_front_page: bool) -> String {
    if is_front_page {
        "/".to_string()
    } else if post_type != "page"
        && post_type != "post"
        && post_type != "attachment"
        && post_type != "nav_menu_item"
        && post_type != "wp_block"
        && post_type != "revision"
    {
        format!("/{}/{}/", post_type, slug)
    } else {
        format!("/{}/", slug)
    }
}

/// Resolves the canonical GUID or public upload URL for attachment media posts.
pub fn resolve_attachment_guid(
    attachment_url: &str,
    raw_guid: &str,
    postmeta: &HashMap<String, String>,
) -> String {
    if !attachment_url.is_empty() {
        normalize_menu_url(attachment_url)
    } else if let Some(af) = postmeta.get("_wp_attached_file") {
        format!("/wp-content/uploads/{}", af.trim_start_matches('/'))
    } else {
        normalize_menu_url(raw_guid)
    }
}

/// Infers the MIME type of an attachment from its declared MIME type, URL, attached file, or slug.
pub fn infer_attachment_mime(
    post_mime_type: &str,
    attachment_url: &str,
    guid: &str,
    slug: &str,
    postmeta: &HashMap<String, String>,
) -> String {
    if !post_mime_type.is_empty() {
        return post_mime_type.to_string();
    }
    let mut m = crate::importer::models::infer_mime_type_from_filename(attachment_url);
    if m.is_empty() {
        if let Some(af) = postmeta.get("_wp_attached_file") {
            m = crate::importer::models::infer_mime_type_from_filename(af);
        }
    }
    if m.is_empty() {
        m = crate::importer::models::infer_mime_type_from_filename(guid);
    }
    if m.is_empty() {
        m = crate::importer::models::infer_mime_type_from_filename(slug);
    }
    m.to_string()
}

/// Remaps post thumbnail and profile image IDs stored in wp_postmeta to newly created attachment IDs.
pub fn remap_post_thumbnails(
    tx: &Transaction,
    old_to_new_post_id: &HashMap<i64, i64>,
) -> Result<()> {
    let thumb_rows: Vec<(i64, String)> = {
        let mut stmt = tx.prepare(
            "SELECT meta_id, COALESCE(meta_value, '') FROM wp_postmeta WHERE meta_key IN ('_thumbnail_id', 'profile_img', 'banner_image')",
        )?;
        let mapped = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        mapped.filter_map(|r| r.ok()).collect()
    };
    for (meta_id, old_val) in thumb_rows {
        if let Ok(old_thumb_id) = old_val.trim().parse::<i64>() {
            if let Some(&new_thumb_id) = old_to_new_post_id.get(&old_thumb_id) {
                let _ = tx.execute(
                    "UPDATE wp_postmeta SET meta_value = ?1 WHERE meta_id = ?2",
                    params![new_thumb_id.to_string(), meta_id],
                );
            }
        }
    }
    Ok(())
}

/// Remaps post_parent IDs for hierarchical posts and pages.
pub fn remap_post_parents(
    tx: &Transaction,
    pending_parent_remaps: &[(i64, i64)],
    old_to_new_post_id: &HashMap<i64, i64>,
) -> Result<()> {
    for &(child_id, old_parent_id) in pending_parent_remaps {
        if let Some(&new_parent_id) = old_to_new_post_id.get(&old_parent_id) {
            let _ = tx.execute(
                "UPDATE wp_posts SET post_parent = ?1 WHERE ID = ?2",
                params![new_parent_id, child_id],
            );
        }
    }
    Ok(())
}

/// Detects site logo attachments preferring SVG over PNG/JPEG.
pub fn detect_logo_attachment(
    post_type: &str,
    slug: &str,
    title: &str,
    guid: &str,
    current_logo: &mut Option<String>,
) {
    if post_type == "attachment" {
        let slug_lower = slug.to_lowercase();
        let title_lower = title.to_lowercase();
        if (slug_lower.contains("logo") || title_lower.contains("logo")) && !guid.is_empty() {
            let is_better = current_logo
                .as_ref()
                .map(|prev| !prev.ends_with(".svg") && guid.ends_with(".svg"))
                .unwrap_or(true);
            if is_better {
                *current_logo = Some(guid.to_string());
            }
        }
    }
}
