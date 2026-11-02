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

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Home Page
    xml.push_str(&format!(
        "  <url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>\n",
        xml_escape(base_url)
    ));

    for post in posts {
        if post.seo_noindex {
            continue;
        }
        if post.post_type == "post" || post.post_type == "page" {
            let priority = if post.post_type == "page" {
                "0.8"
            } else {
                "0.6"
            };
            let loc = xml_escape(&format!("{}/{}/", base_url, post.slug));
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
