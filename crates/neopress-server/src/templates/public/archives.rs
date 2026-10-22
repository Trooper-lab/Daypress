//! Public Archive, Search, and 404 Templates (`templates/public/archives.rs`).

use super::shell::{plain_text_excerpt, render_base_layout};
use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

pub fn render_archive_page(
    archive_title: &str,
    archive_subtitle: &str,
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
) -> String {
    let mut cards_html = String::new();
    if posts.is_empty() {
        cards_html.push_str(
            r#"<div style="grid-column: 1 / -1; text-align: center; padding: 60px 24px; background: var(--bg-card); border: 1px solid var(--border); border-radius: 12px; color: var(--text-muted);">No matching articles or pages found.</div>"#,
        );
    } else {
        for post in posts {
            let excerpt = plain_text_excerpt(&post.content.rendered, 140);
            let date_short: String = post.date.chars().take(10).collect();
            let author_display = if post.author_name.is_empty() {
                "admin".to_string()
            } else {
                escape_html(&post.author_name)
            };

            cards_html.push_str(&format!(
                r##"
                <a href="/{slug}/" class="np-card">
                    <div class="np-card-body">
                        <div class="np-card-meta">
                            <span class="np-badge">{post_type}</span>
                            <span>&bull;</span>
                            <span>{date}</span>
                            <span>&bull;</span>
                            <span>{author}</span>
                        </div>
                        <h3 class="np-card-title">{title}</h3>
                        <p class="np-card-excerpt">{excerpt}</p>
                        <div class="np-card-link">Read more &rarr;</div>
                    </div>
                </a>
                "##,
                slug = escape_html(&post.slug),
                post_type = escape_html(&post.post_type),
                date = escape_html(&date_short),
                author = author_display,
                title = post.title.rendered,
                excerpt = excerpt,
            ));
        }
    }

    let body = format!(
        r##"
        <section class="np-section">
            <div class="np-container">
                <div style="max-width: 760px; margin-bottom: 40px;">
                    <span class="np-badge" style="margin-bottom: 12px; display: inline-block;">{subtitle}</span>
                    <h1 class="np-heading">{title}</h1>
                </div>
                <div class="np-grid" style="grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));">
                    {cards_html}
                </div>
            </div>
        </section>
        "##,
        subtitle = escape_html(archive_subtitle),
        title = escape_html(archive_title),
        cards_html = cards_html,
    );

    render_base_layout(archive_title, settings, &body, current_user, None)
}

pub fn render_search_page(
    query: &str,
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
) -> String {
    render_archive_page(
        &format!("Search Results for: {}", query),
        "Search Archive",
        posts,
        settings,
        current_user,
    )
}

pub fn render_not_found_page(
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
) -> String {
    let body = r##"
        <section class="np-section">
            <div class="np-container" style="max-width: 720px; text-align: center;">
                <span class="np-badge" style="margin-bottom: 14px; display: inline-block;">HTTP 404</span>
                <h1 class="np-heading">404 — Page Not Found</h1>
                <p class="np-text" style="margin-bottom: 24px;">The requested resource could not be found.</p>
                <a href="/" class="np-button">Return Home</a>
            </div>
        </section>
    "##;

    render_base_layout("404 Not Found", settings, body, current_user, None)
}
