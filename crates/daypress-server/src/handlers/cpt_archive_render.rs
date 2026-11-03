//! Universal CPT Archive page rendering and media/taxonomy resolution.

use super::PagedQuery;
use crate::db::Database;
use crate::models::{self, DaypressPostResponse};
use crate::templates;
use crate::templates::admin::layout::format_cpt_title;
use crate::theme_builder::models::RequestContext;
use crate::theme_builder::ssr_injector::{HydrationContext, SsrInjector};
use std::collections::HashMap;

pub fn render_cpt_archive(
    db: &Database,
    cpt_slug: &str,
    settings: &models::DaypressSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
) -> Option<String> {
    render_cpt_archive_with_query(db, cpt_slug, settings, user, &PagedQuery::default())
}

pub fn render_cpt_archive_with_query(
    db: &Database,
    cpt_slug: &str,
    settings: &models::DaypressSettingsResponse,
    user: Option<&crate::auth::AuthUser>,
    query: &PagedQuery,
) -> Option<String> {
    let cpt_def = db.list_post_type_definitions().ok().and_then(|defs| {
        defs.into_iter().find(|d| {
            d.slug.eq_ignore_ascii_case(cpt_slug)
                || (!d.rewrite_slug.is_empty() && d.rewrite_slug.eq_ignore_ascii_case(cpt_slug))
        })
    });
    if let Some(ref def) = cpt_def {
        if !def.has_archive || !def.public {
            return None;
        }
    }

    let posts = db
        .get_posts(cpt_slug, Some("publish"), None, 100)
        .unwrap_or_default();

    if posts.is_empty() && cpt_def.is_none() {
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

    let title_cpt = cpt_def
        .as_ref()
        .filter(|d| !d.label_plural.trim().is_empty())
        .map(|d| d.label_plural.clone())
        .unwrap_or_else(|| format_cpt_title(cpt_slug));
    let subtitle_cpt = cpt_def
        .as_ref()
        .filter(|d| !d.description.trim().is_empty())
        .map(|d| d.description.clone())
        .unwrap_or_else(|| format!("{} Collection Archive", title_cpt));
    let page_title = format!("{} &bull; {}", title_cpt, settings.title);

    let per_page = settings.posts_per_page.clamp(1, 100) as u32;
    let active_page = query.dp_page.or(query.paged).unwrap_or(1).max(1);

    let req_ctx = RequestContext::for_archive(cpt_slug, 0, cpt_slug);
    let hyd_ctx = HydrationContext {
        db: Some(db),
        post: None,
        settings,
        breadcrumbs: req_ctx.build_breadcrumbs(),
        nav_menus: HashMap::new(),
        archive_title: Some(title_cpt.clone()),
        archive_subtitle: Some(subtitle_cpt.clone()),
        archive_posts: Vec::new(),
        pagination_html: None,
        symbol_depth: 0,
        collection_tax: query.dp_tax.clone(),
        collection_term: query.dp_term.clone(),
        collection_sort: query.dp_sort.clone(),
        collection_search: query.dp_q.clone(),
        collection_page: Some(active_page),
    };

    let collection_grid_html = SsrInjector::render_collection_grid_ssr(
        db,
        &format!("cpt-archive-{}", cpt_slug),
        cpt_slug,
        per_page,
        3,
        true,
        true,
        true,
        true,
        query.dp_tax.as_deref(),
        true,
        true,
        true,
        "date_desc",
        "editorial",
        &[],
        &hyd_ctx,
    );

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

    let body_html = format!(
        r#"<section class="dp-section cpt-archive-wrapper container py-5">
            <div class="dp-container" style="max-width: 1180px; margin: 0 auto;">
                <header class="archive-header mb-4" style="margin-bottom: 32px;">
                    <span class="dp-badge" style="margin-bottom: 8px; display: inline-block; text-transform: uppercase;">{cpt_slug}</span>
                    <h1 class="archive-title">{title_cpt}</h1>
                    <p style="color: var(--text-muted, var(--ink-muted)); margin: 0; font-size: 16px;">{subtitle_cpt}</p>
                </header>
                {filters_html}
                {collection_grid_html}
            </div>
        </section>"#,
        cpt_slug = templates::utils::escape_html(cpt_slug),
        title_cpt = templates::utils::escape_html(&title_cpt),
        subtitle_cpt = templates::utils::escape_html(&subtitle_cpt),
        filters_html = filters_html,
        collection_grid_html = collection_grid_html,
    );

    let partials = SsrInjector::resolve_and_hydrate_with_context(db, &req_ctx, None, &hyd_ctx);

    // Ensure resolve_cpt_card_media remains linked for legacy media meta fallback
    let _ = resolve_cpt_card_media;

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

pub fn resolve_cpt_card_media(db: &Database, post: &DaypressPostResponse) -> (String, String) {
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
