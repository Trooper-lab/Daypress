//! SEO Meta, OpenGraph, Canonical, and Structured Data Head Component.

use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::{clean_wp_content, escape_html};

/// Strips HTML tags cleanly and truncates on UTF-8 character boundaries.
pub fn plain_text_excerpt(html: &str, max_chars: usize) -> String {
    let cleaned = clean_wp_content(html);
    let mut out = String::with_capacity(cleaned.len());
    let mut in_tag = false;
    for ch in cleaned.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let normalized = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() > max_chars {
        let prefix: String = normalized.chars().take(max_chars).collect();
        format!("{}...", escape_html(&prefix))
    } else {
        escape_html(&normalized)
    }
}

pub struct HeadMetaOutput {
    pub page_title: String,
    pub meta_desc: String,
    pub robots_meta: &'static str,
    pub canonical_tag: String,
    pub og_image_tags: String,
    pub schema_type: String,
}

pub fn render_head_meta(
    title: &str,
    settings: &WpSettingsResponse,
    current_post: Option<&WpPostResponse>,
    site_name: &str,
    site_desc: &str,
) -> HeadMetaOutput {
    let page_title = if let Some(post) = current_post {
        if !post.seo_title.trim().is_empty() {
            post.seo_title.trim().to_string()
        } else {
            format!("{} – {}", title, site_name)
        }
    } else if title == site_name {
        if !settings.description.trim().is_empty() {
            format!("{} – {}", site_name, site_desc)
        } else {
            site_name.to_string()
        }
    } else {
        format!("{} – {}", title, site_name)
    };

    let meta_desc = if let Some(post) = current_post {
        if !post.seo_description.trim().is_empty() {
            post.seo_description.trim().to_string()
        } else {
            let ex = plain_text_excerpt(&post.content.rendered, 155);
            if !ex.is_empty() {
                ex
            } else {
                site_desc.to_string()
            }
        }
    } else {
        site_desc.to_string()
    };

    let robots_meta = if current_post.map(|p| p.seo_noindex).unwrap_or(false) {
        r#"<meta name="robots" content="noindex, follow" />"#
    } else {
        r#"<meta name="robots" content="index, follow" />"#
    };

    let canonical_tag = if let Some(post) = current_post {
        let can_url = if !post.seo_canonical.trim().is_empty() {
            post.seo_canonical.trim().to_string()
        } else {
            post.link.clone()
        };
        if !can_url.is_empty() {
            format!(r#"<link rel="canonical" href="{}" />"#, escape_html(&can_url))
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let og_image_tags = if let Some(post) = current_post {
        let img = if !post.seo_og_image.trim().is_empty() {
            Some(post.seo_og_image.trim())
        } else {
            post.featured_image_url.as_deref()
        };
        if let Some(url) = img.filter(|u| !u.is_empty()) {
            format!(
                r#"<meta property="og:image" content="{0}" /><meta name="twitter:image" content="{0}" />"#,
                escape_html(url)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let schema_type = if let Some(post) = current_post {
        if !post.seo_schema_type.trim().is_empty() {
            post.seo_schema_type.trim().to_string()
        } else if post.post_type == "post" {
            "BlogPosting".to_string()
        } else {
            "WebPage".to_string()
        }
    } else {
        "WebSite".to_string()
    };

    HeadMetaOutput {
        page_title,
        meta_desc,
        robots_meta,
        canonical_tag,
        og_image_tags,
        schema_type,
    }
}
