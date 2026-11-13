//! Public Single Post & Page SSR Renderer (`templates/public/single.rs`).

use super::shell::render_base_layout;
use crate::adapters::database::search_fts::{
    estimate_reading_time_minutes, extract_ast_plain_text, strip_html_for_fts,
};
use crate::auth::AuthUser;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::utils::{clean_post_content, escape_html, sanitize_comment_html};
use daypress_ast::DaypressLayout;

/// Scans HTML content for `<h2>`..`<h4>` headings (`min_depth..=max_depth`), injects deterministic
/// `id="..."` anchor attributes into headings lacking an `id`, populates any existing
/// `<ul class="dp-toc-list"></ul>` placeholders inside `html`, and returns:
/// `(anchor_injected_html, standalone_toc_nav_html, heading_count)`.
pub fn inject_heading_anchors_and_build_toc(
    html: &str,
    title: &str,
    min_depth: u8,
    max_depth: u8,
    collapsible: bool,
) -> (String, String, usize) {
    let min_d = min_depth.clamp(1, 6);
    let max_d = max_depth.clamp(min_d, 6);
    let Ok(re) = regex::Regex::new(r#"(?is)<h([1-6])(\s[^>]*)?>(.*?)</h[1-6]>"#) else {
        return (html.to_string(), String::new(), 0);
    };
    let id_re = regex::Regex::new(r#"(?i)\bid\s*=\s*["']([^"']+)["']"#).ok();

    let mut headings: Vec<(u8, String, String)> = Vec::new();
    let mut used_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    let updated_html = re
        .replace_all(html, |caps: &regex::Captures| {
            let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let level: u8 = caps
                .get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(2);
            if level < min_d || level > max_d {
                return full.to_string();
            }
            let attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let inner = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let plain = strip_html_for_fts(inner);
            if plain.trim().is_empty() {
                return full.to_string();
            }

            let existing_id = id_re
                .as_ref()
                .and_then(|r| r.captures(attrs))
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty());

            let anchor_id = if let Some(eid) = existing_id.clone() {
                used_ids.insert(eid.clone());
                eid
            } else {
                let base_slug = generated_slugify::slugify(&plain);
                let base = if base_slug.is_empty() {
                    format!("section-{}", headings.len() + 1)
                } else {
                    format!("toc-{}", base_slug)
                };
                let mut candidate = base.clone();
                let mut suffix = 2usize;
                while used_ids.contains(&candidate) {
                    candidate = format!("{}-{}", base, suffix);
                    suffix += 1;
                }
                used_ids.insert(candidate.clone());
                candidate
            };

            headings.push((level, anchor_id.clone(), plain));

            if existing_id.is_some() {
                full.to_string()
            } else {
                static ON_EVENT_ATTR_RE: std::sync::OnceLock<regex::Regex> =
                    std::sync::OnceLock::new();
                let ev_re = ON_EVENT_ATTR_RE.get_or_init(|| {
                    regex::Regex::new(r#"(?i)\s+on[a-z0-9_-]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#)
                        .unwrap()
                });
                let clean_attrs = ev_re.replace_all(attrs, "");
                format!(
                    "<h{level} id=\"{id}\"{attrs}>{inner}</h{level}>",
                    level = level,
                    id = escape_html(&anchor_id),
                    attrs = clean_attrs,
                    inner = inner
                )
            }
        })
        .into_owned();

    if headings.is_empty() {
        return (updated_html, String::new(), 0);
    }

    let mut list_items = String::new();
    for (idx, (level, anchor_id, text)) in headings.iter().enumerate() {
        list_items.push_str(&format!(
            r##"<li class="dp-toc-item" data-depth="{level}"><span class="dp-toc-num">{num:02}</span><a href="#{id}" class="dp-toc-link">{text}</a></li>"##,
            level = level,
            num = idx + 1,
            id = escape_html(anchor_id),
            text = escape_html(text),
        ));
    }

    let final_html = if updated_html.contains(r#"<ul class="dp-toc-list"></ul>"#) {
        updated_html.replace(
            r#"<ul class="dp-toc-list"></ul>"#,
            &format!(r#"<ul class="dp-toc-list">{}</ul>"#, list_items),
        )
    } else {
        updated_html
    };

    let collapse_btn = if collapsible {
        r#"<button type="button" class="dp-toc-toggle" aria-label="Toggle Table of Contents" onclick="this.closest('.dp-toc').classList.toggle('is-collapsed');"><i class="fa-solid fa-chevron-down"></i></button>"#
    } else {
        ""
    };
    let toc_title = if title.trim().is_empty() {
        "Table of Contents"
    } else {
        title.trim()
    };

    let standalone_toc = format!(
        r#"<nav class="dp-toc dp-auto-toc" aria-label="{title_esc}" data-min-depth="{min_d}" data-max-depth="{max_d}">
            <div class="dp-toc-header">
                <h4 class="dp-toc-title">{title_esc}</h4>
                {collapse_btn}
            </div>
            <div class="dp-toc-body">
                <ul class="dp-toc-list">{list_items}</ul>
            </div>
        </nav>"#,
        title_esc = escape_html(toc_title),
        min_d = min_d,
        max_d = max_d,
        collapse_btn = collapse_btn,
        list_items = list_items,
    );

    (final_html, standalone_toc, headings.len())
}

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

    let is_page = post.post_type == "page";
    let has_section_ast = parsed_layout.as_ref().is_some_and(|l| {
        l.root.children.iter().any(|n| {
            matches!(
                n.kind,
                daypress_ast::NodeKind::Section | daypress_ast::NodeKind::Container
            )
        })
    });
    let has_visual_canvas_layout = (is_page && ast_rendered.is_some()) || has_section_ast;
    let theme_single_html = if !has_visual_canvas_layout {
        partials
            .and_then(|p| p.single_html.as_ref())
            .filter(|s| !s.trim().is_empty())
    } else {
        None
    };
    let raw_clean_content =
        ast_rendered.unwrap_or_else(|| clean_post_content(&post.content.rendered));
    let (clean_content, standalone_toc_html, toc_heading_count) =
        inject_heading_anchors_and_build_toc(&raw_clean_content, "Table of Contents", 2, 4, true);
    let plain_for_reading = if let Some(lj) = layout_json {
        let ast_txt = extract_ast_plain_text(lj);
        if ast_txt.is_empty() {
            clean_content.clone()
        } else {
            format!("{} {}", clean_content, ast_txt)
        }
    } else {
        clean_content.clone()
    };
    let (word_count, reading_mins) = estimate_reading_time_minutes(&plain_for_reading);
    let site_name = if settings.title.trim().is_empty() {
        "Home"
    } else {
        settings.title.trim()
    };

    let global_css = super::site_chrome::page_kit_css(settings, parsed_layout.as_ref());

    let mut page_custom_css = if layout_scoped_css.is_empty() {
        global_css
    } else if global_css.is_empty() {
        layout_scoped_css
    } else {
        format!("{}\n{}", global_css, layout_scoped_css)
    };

    if theme_single_html.is_some() {
        let single_tpl_css = partials.and_then(|p| p.single_css.as_deref()).unwrap_or("");
        if !single_tpl_css.trim().is_empty() {
            if page_custom_css.is_empty() {
                page_custom_css = single_tpl_css.to_string();
            } else {
                page_custom_css = format!("{}\n{}", page_custom_css, single_tpl_css);
            }
        }
    }

    // Full-bleed Visual Builder page vs standard article/page with header shell
    let is_full_bleed_visual = has_visual_canvas_layout
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

    // Render structured field summary block for Custom Post Types if `post.acf` has values
    let acf_summary_html = if post.post_type != "post" && post.post_type != "page" {
        if let Some(acf_obj) = post.acf.as_object() {
            let mut items = Vec::new();
            for (k, v) in acf_obj {
                if k.starts_with('_') {
                    continue;
                }
                match v {
                    serde_json::Value::String(s)
                        if !s.trim().is_empty() && s.chars().count() <= 140 =>
                    {
                        let label = k.replace('_', " ");
                        items.push(format!(
                            r#"<div class="dp-acf-meta-item" style="display:flex;flex-direction:column;gap:2px;"><span style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.06em;color:var(--ink-muted, var(--text-muted));font-family:var(--font-mono, monospace);">{}</span><strong style="font-size:14px;color:var(--ink, var(--text-main));">{}</strong></div>"#,
                            escape_html(&label),
                            escape_html(s.trim())
                        ));
                    }
                    serde_json::Value::Number(n) => {
                        let label = k.replace('_', " ");
                        items.push(format!(
                            r#"<div class="dp-acf-meta-item" style="display:flex;flex-direction:column;gap:2px;"><span style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.06em;color:var(--ink-muted, var(--text-muted));font-family:var(--font-mono, monospace);">{}</span><strong style="font-size:14px;color:var(--ink, var(--text-main));">{}</strong></div>"#,
                            escape_html(&label),
                            escape_html(&n.to_string())
                        ));
                    }
                    serde_json::Value::Array(arr) => {
                        for elem in arr {
                            if let Some(obj) = elem.as_object() {
                                let lbl = obj.get("label").and_then(|x| x.as_str()).unwrap_or("");
                                let val = obj.get("value").and_then(|x| x.as_str()).unwrap_or("");
                                if !lbl.is_empty() && !val.is_empty() {
                                    items.push(format!(
                                        r#"<div class="dp-acf-meta-item" style="display:flex;flex-direction:column;gap:2px;"><span style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.06em;color:var(--ink-muted, var(--text-muted));font-family:var(--font-mono, monospace);">{}</span><strong style="font-size:14px;color:var(--ink, var(--text-main));">{}</strong></div>"#,
                                        escape_html(lbl),
                                        escape_html(val)
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            if !items.is_empty() {
                format!(
                    r#"<div class="dp-single-acf-summary" style="display:grid;grid-template-columns:repeat(auto-fit, minmax(160px, 1fr));gap:16px;padding:16px 20px;margin:24px auto;max-width:880px;background:var(--paper-raised, #fbf8f2);border:1px solid var(--rule, #16171a2e);border-top:2px solid var(--rule-strong, #16171a);border-radius:2px;">{}</div>"#,
                    items.join("")
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let content = if let Some(tpl_html) = theme_single_html {
        let (hydrated_tpl, _, _) =
            inject_heading_anchors_and_build_toc(tpl_html, "Table of Contents", 2, 4, true);
        hydrated_tpl
    } else if is_section_layout {
        format!("{acf_summary_html}{clean_content}")
    } else if is_full_bleed_visual {
        format!(
            r##"
            <div class="dp-visual-canvas">
                {acf_summary_html}
                {body}
            </div>
            "##,
            acf_summary_html = acf_summary_html,
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
            title = escape_html(&post.title.rendered),
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
                        "margin-left: 32px; border-left: 3px solid var(--accent, #a8401b);"
                    } else {
                        ""
                    };
                    comment_items.push_str(&format!(
                        r##"<div class="comment-card" style="background: var(--bg-card); border: 1px solid var(--border); border-radius: 8px; padding: 18px 20px; margin-bottom: 14px; {indent}">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                                <strong style="color: var(--text-main); font-size: 14px;">{author}</strong>
                                <span style="color: var(--text-dim); font-size: 12px; font-family: var(--font-mono);">{date}</span>
                            </div>
                            <div style="color: var(--text-muted); font-size: 14.5px; line-height: 1.6;">{body}</div>
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
                        <h4 style="font-size: 16px; font-weight: 700; color: var(--text-main); margin-bottom: 16px;">Leave a Comment</h4>
                        <form id="comment-form" onsubmit="submitPublicComment(event)" style="display: flex; flex-direction: column; gap: 14px;">
                            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;">
                                <input type="text" id="comment-author" placeholder="Your Name *" required style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: var(--text-main); font-size: 14px;" />
                                <input type="email" id="comment-email" placeholder="Your Email *" required style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: var(--text-main); font-size: 14px;" />
                            </div>
                            <textarea id="comment-content" rows="4" placeholder="Share your thoughts..." required style="background: var(--bg-base); border: 1px solid var(--border); border-radius: 6px; padding: 10px 14px; color: var(--text-main); font-size: 14px; font-family: inherit;"></textarea>
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
        let disable_toc = post
            .meta
            .as_object()
            .and_then(|m| m.get("disable_toc"))
            .is_some_and(|v| {
                v.as_bool() == Some(true)
                    || v.as_str()
                        .is_some_and(|s| matches!(s, "1" | "true" | "yes"))
            });
        let auto_toc_block = if !is_page
            && post.post_type == "post"
            && toc_heading_count >= 2
            && !clean_content.contains("class=\"dp-toc")
            && !disable_toc
        {
            standalone_toc_html
        } else {
            String::new()
        };
        let reading_time_badge = if !is_page {
            format!(
                r#"<span>&bull;</span><span class="dp-reading-time" style="font-family: var(--font-mono, monospace); font-size: 12px; color: var(--ink-muted, var(--text-muted));"><i class="fa-regular fa-clock" style="margin-right: 4px;"></i>{mins} min read ({words} words)</span>"#,
                mins = reading_mins,
                words = word_count
            )
        } else {
            String::new()
        };
        let back_href = if !post.lang_code.trim().is_empty()
            && !post.lang_code.eq_ignore_ascii_case("en")
        {
            let clean_lang: String = post
                .lang_code
                .trim()
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if clean_lang.is_empty() {
                "/".to_string()
            } else {
                format!("/{}/", clean_lang)
            }
        } else {
            "/".to_string()
        };

        format!(
            r##"
            <section class="dp-section">
                <div class="dp-container" style="max-width: 880px;">
                    <div style="margin-bottom: 32px; border-bottom: 1px solid var(--border); padding-bottom: 24px;">
                        <a href="{back_href}" style="font-size: 13px; font-weight: 600; color: var(--accent, #a8401b); display: inline-block; margin-bottom: 16px;">&larr; {site_name}</a>
                        <div class="dp-card-meta">
                            <span class="dp-badge">{post_type}</span>
                            <span>&bull;</span>
                            <time datetime="{date}">{date}</time>
                            <span>&bull;</span>
                            <span>{author_display}</span>
                            {reading_time_badge}
                            {categories_html}
                        </div>
                        <h1 class="dp-heading">{title}</h1>
                    </div>

                    {acf_summary_html}

                    {featured_img_html}

                    {auto_toc_block}

                    <div class="dp-text">
                        {body}
                    </div>

                    {tags_html}

                    {comments_section_html}
                </div>
            </section>
            "##,
            site_name = escape_html(site_name),
            title = escape_html(&post.title.rendered),
            date = date_short,
            post_type = escape_html(&post.post_type),
            author_display = author_display,
            reading_time_badge = reading_time_badge,
            categories_html = categories_html,
            acf_summary_html = acf_summary_html,
            featured_img_html = featured_img_html,
            auto_toc_block = auto_toc_block,
            tags_html = tags_html,
            comments_section_html = comments_section_html,
            body = clean_content,
            back_href = escape_html(&back_href)
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
        let safe_css = crate::templates::utils::escape_style_tag_contents(page_custom_css.trim());
        let page_style_tag = format!(
            "\t<style id=\"dp-page-styles\">\n{}\n\t</style>\n</head>",
            safe_css
        );
        doc.replace("</head>", &page_style_tag)
    } else {
        doc
    }
}
