//! Public Single Post & Page SSR Renderer (`templates/public/single.rs`).

use super::shell::render_base_layout;
use crate::auth::AuthUser;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::utils::{clean_post_content, escape_html, sanitize_comment_html};
use daypress_ast::DaypressLayout;

pub fn render_single_post(
    post: &DaypressPostResponse,
    settings: &DaypressSettingsResponse,
    current_user: Option<&AuthUser>,
    layout_json: Option<&str>,
    partials: Option<&crate::theme_builder::models::ResolvedThemePartials>,
) -> String {
    let parsed_layout = layout_json
        .and_then(|s| serde_json::from_str::<DaypressLayout>(s).ok())
        .filter(|l| !l.root.children.is_empty());

    let ast_rendered = parsed_layout
        .as_ref()
        .map(daypress_ast::render_layout_to_html)
        .filter(|h| !h.trim().is_empty());

    let layout_scoped_css = parsed_layout
        .as_ref()
        .map(daypress_ast::compile_layout_css)
        .unwrap_or_default();

    let has_ast_layout = ast_rendered.is_some();
    let clean_content = ast_rendered.unwrap_or_else(|| clean_post_content(&post.content.rendered));
    let is_page = post.post_type == "page";
    let site_name = if settings.title.trim().is_empty() {
        "Home"
    } else {
        settings.title.trim()
    };

    let global_css = super::site_chrome::page_kit_css(settings, parsed_layout.as_ref());

    let page_custom_css = if layout_scoped_css.is_empty() {
        global_css
    } else if global_css.is_empty() {
        layout_scoped_css
    } else {
        format!("{}\n{}", global_css, layout_scoped_css)
    };

    // Full-bleed Visual Builder page vs standard article/page with header shell
    let is_full_bleed_visual = has_ast_layout
        || (is_page
            && (clean_content.contains("elem_hero_sec")
                || clean_content.contains("divi_section_")
                || clean_content.contains("divi_node_")
                || clean_content.contains("elementor-section")
                || post.template == "canvas"
                || post.template == "full-width"));

    let has_daystudio_theme = !settings.daystudio_kit_css.trim().is_empty()
        || !settings.daystudio_header_html.trim().is_empty()
        || !settings.daystudio_footer_html.trim().is_empty();

    let is_section_layout = clean_content.trim_start().starts_with("<section")
        || clean_content.contains("<section class=\"pageheader")
        || clean_content.contains("<section class=\"section ")
        || clean_content.contains("<section class=\"dp-section");

    let content = if is_section_layout {
        clean_content
    } else if is_full_bleed_visual {
        format!(
            r##"
            <div class="dp-visual-canvas">
                {body}
            </div>
            "##,
            body = clean_content
        )
    } else if is_page && has_daystudio_theme {
        format!(
            r##"
            <section class="dp-section" style="border-bottom: none; padding: 48px 24px;">
                <div class="dp-container" style="max-width: var(--e-content-width, 1140px); margin: 0 auto;">
                    <h1 class="dp-heading entry-title" style="margin-bottom: 24px;">{title}</h1>
                    <div class="dp-text entry-content">
                        {body}
                    </div>
                </div>
            </section>
            "##,
            title = post.title.rendered,
            body = clean_content
        )
    } else {
        let author_display = if post.author_name.is_empty() {
            "admin".to_string()
        } else {
            escape_html(&post.author_name)
        };

        let categories_html = if !post.category_names.is_empty() {
            let badges: Vec<String> = post
                .category_names
                .iter()
                .map(|c| {
                    let slug = generated_slugify::slugify(c);
                    format!(
                        r#"<a href="/category/{}" class="dp-badge">{}</a>"#,
                        escape_html(&slug),
                        escape_html(c)
                    )
                })
                .collect();
            format!("<span>&bull;</span>{}", badges.join(" "))
        } else {
            String::new()
        };

        let featured_img_html = if let Some(ref img_url) = post.featured_image_url {
            if !img_url.is_empty() && !clean_content.contains(img_url.as_str()) {
                format!(
                    r#"<div style="margin-bottom: 28px; border-radius: 12px; overflow: hidden; border: 1px solid var(--border);"><img src="{url}" alt="{alt}" class="dp-image" style="width: 100%; max-height: 460px; object-fit: cover;" /></div>"#,
                    url = escape_html(img_url),
                    alt = escape_html(&post.title.rendered),
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let tags_html = if !post.tag_names.is_empty() {
            let pills: Vec<String> = post
                .tag_names
                .iter()
                .map(|t| {
                    let slug = generated_slugify::slugify(t);
                    format!(
                        r#"<a href="/tag/{}" style="background: var(--bg-card); border: 1px solid var(--border); color: var(--text-muted); padding: 4px 12px; border-radius: 9999px; font-size: 12px;">#{}</a>"#,
                        escape_html(&slug),
                        escape_html(t)
                    )
                })
                .collect();
            format!(
                r#"<div style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: 28px;">{}</div>"#,
                pills.join("")
            )
        } else {
            String::new()
        };

        let comments_section_html = if !is_page {
            let mut comment_items = String::new();
            if post.comments.is_empty() {
                comment_items.push_str(
                    r#"<p id="no-comments-placeholder" style="color: var(--text-dim); font-size: 14px; margin-bottom: 20px;">No comments yet. Be the first to join the discussion!</p>"#,
                );
            } else {
                for c in &post.comments {
                    let c_author = if c.author_name.is_empty() {
                        "Anonymous".to_string()
                    } else {
                        escape_html(&c.author_name)
                    };
                    let c_date: String = c.date.chars().take(10).collect();
                    let c_body = sanitize_comment_html(&c.content.rendered);
                    let indent_style = if c.parent > 0 {
                        "margin-left: 32px; border-left: 3px solid #2563eb;"
                    } else {
                        ""
                    };
                    comment_items.push_str(&format!(
                        r##"<div class="comment-card" style="background: var(--bg-card); border: 1px solid var(--border); border-radius: 8px; padding: 18px 20px; margin-bottom: 14px; {indent}">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                                <strong style="color: #ffffff; font-size: 14px;">{author}</strong>
                                <span style="color: var(--text-dim); font-size: 12px; font-family: var(--font-mono);">{date}</span>
                            </div>
                            <div style="color: #d4d4d8; font-size: 14.5px; line-height: 1.6;">{body}</div>
                        </div>"##,
                        indent = indent_style,
                        author = c_author,
                        date = escape_html(&c_date),
                        body = c_body,
                    ));
                }
            }

            format!(
                r##"<section id="comments" style="margin-top: 56px; padding-top: 36px; border-top: 1px solid var(--border);">
                    <h3 class="dp-heading" style="font-size: 22px; margin-bottom: 20px;">Comments ({count})</h3>
                    <div id="comments-list">
                        {comment_items}
                    </div>
                    <div style="background: var(--bg-card); border: 1px solid var(--border); border-radius: 10px; padding: 24px; margin-top: 24px;">
                        <h4 style="font-size: 16px; font-weight: 700; color: #ffffff; margin-bottom: 16px;">Leave a Comment</h4>
                        <form id="comment-form" onsubmit="submitPublicComment(event)" style="display: flex; flex-direction: column; gap: 14px;">
                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;">
                                <input type="text" id="comment-author" placeholder="Your Name *" required style="background: #09090b; border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: #ffffff; font-size: 14px;" />
                                <input type="email" id="comment-email" placeholder="Your Email *" required style="background: #09090b; border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: #ffffff; font-size: 14px;" />
                            </div>
                            <textarea id="comment-content" rows="4" placeholder="Share your thoughts..." required style="background: #09090b; border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: #ffffff; font-size: 14px; font-family: inherit;"></textarea>
                            <div>
                                <button type="submit" id="comment-submit-btn" class="dp-button">Post Comment</button>
                            </div>
                        </form>
                    </div>
                </section>
                <script>
                async function submitPublicComment(e) {{
                    e.preventDefault();
                    const btn = document.getElementById('comment-submit-btn');
                    const author_name = document.getElementById('comment-author').value.trim();
                    const author_email = document.getElementById('comment-email').value.trim();
                    const content = document.getElementById('comment-content').value.trim();
                    if (!author_name || !content) return;
                    btn.disabled = true;
                    btn.textContent = 'Posting...';
                    try {{
                        const res = await fetch('/wp-json/wp/v2/comments', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ post: {post_id}, author_name, author_email, content }})
                        }});
                        if (res.ok) {{
                            const saved = await res.json().catch(() => ({{}}));
                            if (saved.status === 'approved') {{
                                window.location.reload();
                                return;
                            }}
                            document.getElementById('comment-form').reset();
                            btn.textContent = 'Comment submitted';
                            const note = document.createElement('p');
                            note.style.cssText = 'color: var(--text-dim); font-size: 14px; margin-top: 10px;';
                            note.textContent = 'Thanks! Your comment is awaiting moderation.';
                            document.getElementById('comment-form').appendChild(note);
                        }} else {{
                            btn.disabled = false;
                            btn.textContent = 'Post Comment';
                        }}
                    }} catch (err) {{
                        btn.disabled = false;
                        btn.textContent = 'Post Comment';
                    }}
                }}
                </script>"##,
                count = post.comments.len(),
                comment_items = comment_items,
                post_id = post.id,
            )
        } else {
            String::new()
        };

        let date_short: String = post.date.chars().take(10).collect();

        format!(
            r##"
            <section class="dp-section">
                <div class="dp-container" style="max-width: 880px;">
                    <div style="margin-bottom: 32px; border-bottom: 1px solid var(--border); padding-bottom: 24px;">
                        <a href="/" style="font-size: 13px; font-weight: 600; color: #60a5fa; display: inline-block; margin-bottom: 16px;">&larr; {site_name}</a>
                        <div class="dp-card-meta">
                            <span class="dp-badge">{post_type}</span>
                            <span>&bull;</span>
                            <time datetime="{date}">{date}</time>
                            <span>&bull;</span>
                            <span>{author_display}</span>
                            {categories_html}
                        </div>
                        <h1 class="dp-heading">{title}</h1>
                    </div>

                    {featured_img_html}

                    <div class="dp-text">
                        {body}
                    </div>

                    {tags_html}

                    {comments_section_html}
                </div>
            </section>
            "##,
            site_name = escape_html(site_name),
            title = post.title.rendered,
            date = date_short,
            post_type = escape_html(&post.post_type),
            author_display = author_display,
            categories_html = categories_html,
            featured_img_html = featured_img_html,
            tags_html = tags_html,
            comments_section_html = comments_section_html,
            body = clean_content
        )
    };

    let doc = render_base_layout(
        &post.title.rendered,
        settings,
        &content,
        current_user,
        Some(post),
        partials,
    );

    if !page_custom_css.trim().is_empty() {
        let page_style_tag = format!(
            "\t<style id=\"dp-page-styles\">\n{}\n\t</style>\n</head>",
            page_custom_css.trim()
        );
        doc.replace("</head>", &page_style_tag)
    } else {
        doc
    }
}
