//! Dynamic Taxonomy Archive Handler (`src/handlers/taxonomy_archive_render.rs`).

use crate::auth::AuthUser;
use crate::db::Database;
use crate::models::WpSettingsResponse;
use crate::templates;
use crate::templates::admin::layout::format_cpt_title;
use crate::theme_builder::models::RequestContext;
use crate::theme_builder::ssr_injector::SsrInjector;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// Attempts to render a dynamic taxonomy archive (e.g. `/{taxonomy}/{term_slug}/`).
/// Returns `Some(Response)` with HTTP 200 if the term exists, or `None` if not found.
pub fn try_handle_taxonomy_archive(
    db: &Database,
    taxonomy: &str,
    slug: &str,
    settings: &WpSettingsResponse,
    user: Option<&AuthUser>,
) -> Option<Response> {
    let term = db.get_taxonomy_term(taxonomy, slug).ok()??;
    let posts = db
        .get_posts_by_taxonomy_term(taxonomy, term.id)
        .unwrap_or_default();

    let tax_label = format_cpt_title(taxonomy);
    let title = format!("{}: {}", tax_label, term.name);
    let subtitle = if !term.description.trim().is_empty() {
        term.description.clone()
    } else {
        format!("{} Archive", tax_label)
    };

    let req_ctx = RequestContext::for_archive(taxonomy, term.id, &term.slug);
    let partials = SsrInjector::resolve_and_hydrate_partials(db, &req_ctx, settings);

    let archive_html = templates::render_archive_page(
        &title,
        &subtitle,
        &posts,
        settings,
        user,
        Some(&partials),
    );

    let final_html = crate::plugins::inject_plugin_hooks_into_html(
        db,
        settings,
        None,
        archive_html,
    );

    Some((StatusCode::OK, Html(final_html)).into_response())
}
