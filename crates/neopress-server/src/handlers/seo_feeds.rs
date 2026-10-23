//! Native SEO & Streaming Sitemap.xml / Robots.txt HTTP Handlers.

use crate::db::Database;
use axum::{
    extract::State,
    http::header,
    response::IntoResponse,
};

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
        base_url
    ));

    for post in posts {
        if post.seo_noindex {
            continue;
        }
        if post.post_type == "post" || post.post_type == "page" {
            let lastmod = if !post.modified.is_empty() {
                &post.modified
            } else {
                &post.date
            };
            let priority = if post.post_type == "page" {
                "0.8"
            } else {
                "0.6"
            };
            xml.push_str(&format!(
                "  <url><loc>{}/{}/</loc><lastmod>{}</lastmod><priority>{}</priority></url>\n",
                base_url, post.slug, lastmod, priority
            ));
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
    let robots = if !seo_global.custom_robots_txt.trim().is_empty() {
        seo_global.custom_robots_txt
    } else {
        format!(
            "User-agent: *\nAllow: /\nDisallow: /n-admin/\nDisallow: /wp-json/\n\nSitemap: {}/sitemap.xml\n",
            base_url
        )
    };
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots,
    )
}
