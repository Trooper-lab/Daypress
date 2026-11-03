//! Field widgets for the post editor: SEO defaults, parent page options, categories, featured image.

use crate::models::{DaypressCategoryResponse, DaypressMediaResponse, DaypressPostResponse};
use crate::templates::utils::escape_html;

/// Initial SEO studio values (title, description, keyphrase, canonical, OG image, noindex, schema).
pub(super) fn initial_seo_fields(
    existing: Option<&DaypressPostResponse>,
    is_page: bool,
) -> (String, String, String, String, String, bool, String) {
    if let Some(post) = existing {
        (
            escape_html(&post.seo_title),
            escape_html(&post.seo_description),
            escape_html(&post.seo_keyword),
            escape_html(&post.seo_canonical),
            escape_html(&post.seo_og_image),
            post.seo_noindex,
            if post.seo_schema_type.is_empty() {
                if is_page {
                    "WebPage".to_string()
                } else {
                    "Article".to_string()
                }
            } else {
                post.seo_schema_type.clone()
            },
        )
    } else {
        (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            false,
            if is_page {
                "WebPage".to_string()
            } else {
                "Article".to_string()
            },
        )
    }
}

pub(super) fn render_parent_options(
    all_pages: &[DaypressPostResponse],
    existing: Option<&DaypressPostResponse>,
    initial_parent: i64,
) -> String {
    let mut parent_options = String::from(r#"<option value="0">(no parent - top level)</option>"#);
    for p in all_pages {
        if existing.map(|e| e.id) != Some(p.id) {
            let sel = if p.id == initial_parent {
                "selected"
            } else {
                ""
            };
            let t_esc = escape_html(&p.title.rendered);
            parent_options.push_str(&format!(
                r#"<option value="{}" {}>{}</option>"#,
                p.id, sel, t_esc
            ));
        }
    }
    parent_options
}

pub(super) fn render_category_checkboxes(
    all_categories: &[DaypressCategoryResponse],
    initial_categories: &[i64],
) -> String {
    let mut cat_checkboxes = String::new();
    for c in all_categories {
        let checked = if initial_categories.contains(&c.id) {
            "checked"
        } else {
            ""
        };
        let c_name = escape_html(&c.name);
        cat_checkboxes.push_str(&format!(
        r#"<label style="display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer;">
            <input type="checkbox" name="post_category" value="{}" {} />
            <span>{}</span>
        </label>"#,
        c.id, checked, c_name
    ));
    }
    cat_checkboxes
}

pub(super) fn render_featured_image_preview(
    all_media: &[DaypressMediaResponse],
    initial_featured_media: i64,
) -> String {
    if initial_featured_media > 0 {
        if let Some(m) = all_media.iter().find(|m| m.id == initial_featured_media) {
            let url_esc = escape_html(&m.source_url);
            format!(
                r#"<div id="featuredImgContainer" style="position: relative; border-radius: var(--radius-sm); overflow: hidden; height: 140px; background: var(--ink); margin-bottom: 10px;">
                <img id="featuredImgElem" src="{}" style="width: 100%; height: 100%; object-fit: cover;" />
            </div>
            <div style="display: flex; gap: 8px;">
                <button type="button" class="btn btn-secondary" style="flex: 1; font-size: 11.5px; padding: 4px;" onclick="openMediaPicker('featured')">Replace</button>
                <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 4px; color: var(--oxide);" onclick="removeFeaturedImage()">Remove</button>
            </div>"#,
                url_esc
            )
        } else {
            r#"<div id="featuredImgContainer" style="display: none; height: 140px; border-radius: var(--radius-sm); overflow: hidden; background: var(--ink); margin-bottom: 10px;">
            <img id="featuredImgElem" src="" style="width: 100%; height: 100%; object-fit: cover;" />
        </div>
        <button type="button" class="btn btn-secondary" style="width: 100%; justify-content: center; font-size: 12px;" onclick="openMediaPicker('featured')">+ Set Featured Image</button>"#.to_string()
        }
    } else {
        r#"<div id="featuredImgContainer" style="display: none; height: 140px; border-radius: var(--radius-sm); overflow: hidden; background: var(--ink); margin-bottom: 10px;">
        <img id="featuredImgElem" src="" style="width: 100%; height: 100%; object-fit: cover;" />
    </div>
    <button type="button" class="btn btn-secondary" style="width: 100%; justify-content: center; font-size: 12px;" onclick="openMediaPicker('featured')">+ Set Featured Image</button>"#.to_string()
    }
}

pub(super) fn render_media_picker_items(all_media: &[DaypressMediaResponse]) -> String {
    let mut media_picker_items = String::new();
    for m in all_media {
        let url_esc = escape_html(&m.source_url);
        let title_esc = escape_html(&m.title.rendered);
        if m.mime_type.starts_with("image/") {
            media_picker_items.push_str(&format!(
                r#"<div class="media-pick-item" onclick="selectMediaItem({}, '{}', '{}')" style="cursor: pointer; border: 2px solid transparent; border-radius: var(--radius-sm); overflow: hidden; height: 110px; background: var(--ink); position: relative; transition: all 0.15s ease;" onmouseover="this.style.borderColor='var(--press)'" onmouseout="this.style.borderColor='transparent'">
                    <img src="{}" alt="{}" style="width: 100%; height: 100%; object-fit: cover;" loading="lazy" />
                    <div style="position: absolute; bottom: 0; left: 0; right: 0; background: rgba(15,23,42,0.85); color: var(--paper-raised); font-size: 10px; padding: 2px 6px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{}</div>
                </div>"#,
                m.id, url_esc, title_esc, url_esc, title_esc, title_esc
            ));
        }
    }
    media_picker_items
}

pub(super) fn render_tag_chips_and_json(
    all_tags: &[DaypressCategoryResponse],
    initial_tags: &[i64],
) -> (String, String, String) {
    let mut popular_tag_chips = String::new();
    for t in all_tags.iter().take(6) {
        let t_name = escape_html(&t.name);
        popular_tag_chips.push_str(&format!(
            r#"<span class="tag-chip" onclick="addTagChip('{}')" style="background: var(--paper-sunk); color: var(--ink-muted); font-size: 11px; padding: 2px 8px; border-radius: 4px; cursor: pointer; border: 1px solid var(--rule-strong);">+ {}</span>"#,
            t_name, t_name
        ));
    }
    let initial_tag_names: Vec<String> = all_tags
        .iter()
        .filter(|t| initial_tags.contains(&t.id))
        .map(|t| t.name.clone())
        .collect();
    let initial_tag_str = initial_tag_names.join(", ");
    let mut known_tags_entries = Vec::new();
    for t in all_tags {
        known_tags_entries.push(format!(
            r#""{}": {}"#,
            escape_html(&t.name.to_lowercase()),
            t.id
        ));
    }
    let known_tags_json = format!("{{{}}}", known_tags_entries.join(", "));
    (popular_tag_chips, initial_tag_str, known_tags_json)
}
