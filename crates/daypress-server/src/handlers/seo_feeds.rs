//! Native SEO & Streaming Sitemap.xml / Robots.txt HTTP Handlers.

use crate::db::Database;
use crate::models::DaypressPostResponse;
use crate::templates::public::rss::xml_escape;
use axum::{extract::State, http::header, response::IntoResponse};
use chrono::{DateTime, NaiveDate, NaiveDateTime};

/// Formats a post's modification/publish timestamp into W3C Datetime (`YYYY-MM-DDThh:mm:ss+00:00`)
/// as required by the Sitemaps 0.9 specification and Google Search Console.
fn format_sitemap_lastmod(post: &DaypressPostResponse) -> Option<String> {
    for candidate in [
        post.modified_gmt.as_str(),
        post.modified.as_str(),
        post.date_gmt.as_str(),
        post.date.as_str(),
    ] {
        let s = candidate.trim();
        if s.is_empty() || s.starts_with("0000") {
            continue;
        }
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Some(dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string());
        }
        let normalized = s.replace('T', " ");
        if let Ok(ndt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S") {
            return Some(format!("{}+00:00", ndt.format("%Y-%m-%dT%H:%M:%S")));
        }
        if let Ok(nd) = NaiveDate::parse_from_str(&normalized, "%Y-%m-%d") {
            return Some(nd.format("%Y-%m-%d").to_string());
        }
    }
    None
}

pub async fn handle_sitemap_xml(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let base_url = settings.url.trim_end_matches('/');
    let posts = db
        .get_posts("any", Some("publish"), None, 50000)
        .unwrap_or_default();
    let custom_post_types = db.list_post_type_definitions().unwrap_or_default();
    let public_cpt_slugs: std::collections::HashSet<String> = custom_post_types
        .iter()
        .filter(|cpt| cpt.public)
        .map(|cpt| cpt.slug.clone())
        .collect();

    let front_page_id = if settings.show_on_front == "page" && settings.page_on_front > 0 {
        Some(settings.page_on_front)
    } else {
        None
    };

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Home Page (include <lastmod> from static front page if configured)
    let home_lastmod = front_page_id
        .and_then(|fid| posts.iter().find(|p| p.id == fid))
        .and_then(format_sitemap_lastmod);
    match home_lastmod {
        Some(lastmod) => {
            xml.push_str(&format!(
                "  <url><loc>{}/</loc><lastmod>{}</lastmod><changefreq>daily</changefreq><priority>1.0</priority></url>\n",
                xml_escape(base_url),
                lastmod
            ));
        }
        None => {
            xml.push_str(&format!(
                "  <url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>\n",
                xml_escape(base_url)
            ));
        }
    }

    // Public Custom Post Type Archives (when has_archive is true and no page shares that slug)
    for cpt in &custom_post_types {
        if !cpt.public || !cpt.has_archive {
            continue;
        }
        let archive_slug = if !cpt.rewrite_slug.trim().is_empty() {
            cpt.rewrite_slug.trim()
        } else {
            cpt.slug.trim()
        };
        if archive_slug.is_empty()
            || posts
                .iter()
                .any(|p| p.post_type == "page" && p.slug == archive_slug)
        {
            continue;
        }
        let loc = xml_escape(&format!("{}/{}/", base_url, archive_slug));
        xml.push_str(&format!(
            "  <url><loc>{}</loc><priority>0.7</priority></url>\n",
            loc
        ));
    }

    let mut seen_locs = std::collections::HashSet::new();
    seen_locs.insert(format!("{}/", base_url));

    for post in posts {
        if post.seo_noindex {
            continue;
        }
        if Some(post.id) == front_page_id {
            continue;
        }
        let is_public_type = post.post_type == "post"
            || post.post_type == "page"
            || public_cpt_slugs.contains(&post.post_type)
            || (!matches!(
                post.post_type.as_str(),
                "attachment" | "nav_menu_item" | "revision" | "elementor_library" | "elementor_kit"
            ) && !post.post_type.starts_with("acf-")
                && !post.post_type.starts_with("wp_"));
        if is_public_type {
            let priority = if post.post_type == "page" {
                "0.8"
            } else if post.post_type == "post" {
                "0.6"
            } else {
                "0.7"
            };
            let raw_loc = if !post.link.trim().is_empty() {
                post.link.clone()
            } else {
                format!("{}/{}/", base_url, post.slug)
            };
            if !seen_locs.insert(raw_loc.clone()) {
                continue;
            }
            let loc = xml_escape(&raw_loc);
            match format_sitemap_lastmod(&post) {
                Some(lastmod) => {
                    xml.push_str(&format!(
                        "  <url><loc>{}</loc><lastmod>{}</lastmod><priority>{}</priority></url>\n",
                        loc, lastmod, priority
                    ));
                }
                None => {
                    xml.push_str(&format!(
                        "  <url><loc>{}</loc><priority>{}</priority></url>\n",
                        loc, priority
                    ));
                }
            }
        }
    }

    xml.push_str("</urlset>");

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
}

pub async fn handle_robots_txt(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let seo_global = db.get_global_seo_settings().unwrap_or_default();
    let base_url = settings.url.trim_end_matches('/');
    let robots = if !settings.blog_public {
        // Settings → Reading → "Discourage search engines from indexing this site".
        "User-agent: *\nDisallow: /\n".to_string()
    } else if !seo_global.custom_robots_txt.trim().is_empty() {
        seo_global.custom_robots_txt
    } else {
        format!(
            "User-agent: *\nAllow: /\nDisallow: /d-admin/\nDisallow: /wp-json/\n\nSitemap: {}/sitemap.xml\n",
            base_url
        )
    };
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots,
    )
}
