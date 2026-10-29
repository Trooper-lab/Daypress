//! Universal CPT Archive page rendering and media/taxonomy resolution.

use crate::db::Database;
use crate::models::{self, WpPostResponse};
use crate::templates;
use crate::templates::admin::layout::format_cpt_title;
use crate::theme_builder::models::RequestContext;
use crate::theme_builder::ssr_injector::SsrInjector;

pub fn render_cpt_archive(
    db: &Database,
    cpt_slug: &str,
    settings: &models::WpSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
) -> Option<String> {
    let posts = db
        .get_posts(cpt_slug, Some("publish"), None, 100)
        .unwrap_or_default();

    if posts.is_empty() {
        let is_cpt = if let Ok(conn) = db.conn.lock() {
            conn.query_row(
                "SELECT 1 FROM posts WHERE post_type = ?1 LIMIT 1",
                rusqlite::params![cpt_slug],
                |_| Ok(()),
            )
            .is_ok()
        } else {
            false
        };
        if !is_cpt {
            return None;
        }
    }

    let title_cpt = format_cpt_title(cpt_slug);
    let page_title = format!("{} Archive &bull; {}", title_cpt, settings.title);

    let taxonomies = get_cpt_taxonomies(db, cpt_slug);
    let mut filters_html = String::new();
    if !taxonomies.is_empty() {
        filters_html.push_str(
            "<div class=\"filters_desktop\"><div class=\"facetwp-facet facetwp-type-checkboxes\">",
        );
        for (tax_name, terms) in &taxonomies {
            let tax_label = format_cpt_title(tax_name);
            filters_html.push_str(&format!(
                "<div class=\"facet-group\"><span class=\"facet-title\">{}</span>",
                tax_label
            ));
            for (_term_id, name, slug) in terms {
                filters_html.push_str(&format!(
                    r#"<div class="facetwp-checkbox" data-value="{}"><span class="facetwp-display-value">{}</span></div>"#,
                    slug, name
                ));
            }
            filters_html.push_str("</div>");
        }
        filters_html.push_str("</div></div>");
    }

    let mut cards_html = String::new();
    for p in &posts {
        let (img_url, img_alt) = resolve_cpt_card_media(db, p);
        let excerpt = if !p.excerpt.rendered.trim().is_empty() {
            p.excerpt.rendered.clone()
        } else {
            let meta_sub = get_postmeta_value(db, p.id, "samenvatting");
            if !meta_sub.is_empty() {
                format!("<p>{}</p>", meta_sub)
            } else {
                let plain: String = p.content.rendered.chars().take(140).collect();
                if plain.is_empty() {
                    String::new()
                } else {
                    format!("<p>{}...</p>", plain)
                }
            }
        };

        let card_img = if !img_url.is_empty() {
            format!(
                r#"<div class="card-media"><img src="{}" alt="{}" loading="lazy" class="img-fluid" /></div>"#,
                img_url, img_alt
            )
        } else {
            String::new()
        };

        let post_url = format!("/{}/{}/", cpt_slug, p.slug);
        cards_html.push_str(&format!(
            r#"<article class="col-md-4 col-sm-6 mb-4 card-col facetwp-template-item">
                <div class="archive-card h-100">
                    {card_img}
                    <div class="card-body">
                        <h3 class="card-title"><a href="{url}">{title}</a></h3>
                        <div class="card-excerpt">{excerpt}</div>
                        <a href="{url}" class="readmore btn-link">Read more &rarr;</a>
                    </div>
                </div>
            </article>"#,
            card_img = card_img,
            url = post_url,
            title = p.title.rendered,
            excerpt = excerpt
        ));
    }

    let body_html = format!(
        r#"<div class="cpt-archive-wrapper container py-5">
            <header class="archive-header mb-4">
                <h1 class="archive-title">{title_cpt}</h1>
            </header>
            {filters}
            <div class="row archive-grid facetwp-template">
                {cards}
            </div>
        </div>"#,
        title_cpt = title_cpt,
        filters = filters_html,
        cards = cards_html
    );

    let req_ctx = RequestContext::for_archive(cpt_slug, 0, cpt_slug);
    let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, settings);

    Some(templates::public::shell::render_base_layout(
        &page_title,
        settings,
        &body_html,
        user,
        None,
        Some(&partials),
    ))
}

fn get_postmeta_value(db: &Database, post_id: i64, key: &str) -> String {
    if let Ok(conn) = db.conn.lock() {
        conn.query_row(
            "SELECT COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key = ?2 LIMIT 1",
            rusqlite::params![post_id, key],
            |r| r.get(0),
        )
        .unwrap_or_default()
    } else {
        String::new()
    }
}

fn resolve_cpt_card_media(db: &Database, post: &WpPostResponse) -> (String, String) {
    if post.featured_media > 0 {
        if let Ok(Some(media_post)) = db.get_post_by_id(post.featured_media) {
            return (media_post.guid.rendered, media_post.title.rendered);
        }
    }
    for meta_key in [
        "banner_image",
        "profile_img",
        "header_image",
        "_thumbnail_id",
        "image",
    ] {
        let val = get_postmeta_value(db, post.id, meta_key);
        if val.is_empty() {
            continue;
        }
        if val.starts_with("http://") || val.starts_with("https://") || val.starts_with('/') {
            return (val, post.title.rendered.clone());
        }
        if let Ok(att_id) = val.trim().parse::<i64>() {
            if let Ok(Some(media_post)) = db.get_post_by_id(att_id) {
                return (media_post.guid.rendered, media_post.title.rendered);
            }
        }
    }
    (String::new(), String::new())
}

fn get_cpt_taxonomies(db: &Database, cpt_slug: &str) -> Vec<(String, Vec<(i64, String, String)>)> {
    let mut map: std::collections::BTreeMap<String, Vec<(i64, String, String)>> =
        std::collections::BTreeMap::new();
    if let Ok(conn) = db.conn.lock() {
        let sql = "SELECT DISTINCT tt.taxonomy, t.term_id, t.name, t.slug \
                   FROM term_relationships r \
                   JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id \
                   JOIN terms t ON tt.term_id = t.term_id \
                   JOIN posts p ON r.object_id = p.ID \
                   WHERE p.post_type = ?1 AND p.post_status = 'publish' \
                   ORDER BY t.name ASC";
        if let Ok(mut stmt) = conn.prepare_cached(sql) {
            if let Ok(rows) = stmt.query_map(rusqlite::params![cpt_slug], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            }) {
                for row in rows.flatten() {
                    map.entry(row.0).or_default().push((row.1, row.2, row.3));
                }
            }
        }
    }
    map.into_iter().collect()
}
