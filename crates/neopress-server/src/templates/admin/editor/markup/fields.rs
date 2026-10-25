//! Field widgets for the post editor: SEO defaults, parent page options, categories, featured image.

use crate::models::{WpCategoryResponse, WpMediaResponse, WpPostResponse};
use crate::templates::utils::escape_html;

/// Initial SEO studio values (title, description, keyphrase, canonical, OG image, noindex, schema).
pub(super) fn initial_seo_fields(
    existing: Option<&WpPostResponse>,
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
    all_pages: &[WpPostResponse],
    existing: Option<&WpPostResponse>,
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
    all_categories: &[WpCategoryResponse],
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
    all_media: &[WpMediaResponse],
    initial_featured_media: i64,
) -> String {
    if initial_featured_media > 0 {
        if let Some(m) = all_media.iter().find(|m| m.id == initial_featured_media) {
            let url_esc = escape_html(&m.source_url);
            format!(
                r#"<div id="featuredImgContainer" style="position: relative; border-radius: 8px; overflow: hidden; height: 140px; background: #0f172a; margin-bottom: 10px;">
                <img id="featuredImgElem" src="{}" style="width: 100%; height: 100%; object-fit: cover;" />
            </div>
            <div style="display: flex; gap: 8px;">
                <button type="button" class="btn btn-secondary" style="flex: 1; font-size: 11.5px; padding: 4px;" onclick="openMediaPicker('featured')">Replace</button>
                <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 4px; color: #ef4444;" onclick="removeFeaturedImage()">Remove</button>
            </div>"#,
                url_esc
            )
        } else {
            r#"<div id="featuredImgContainer" style="display: none; height: 140px; border-radius: 8px; overflow: hidden; background: #0f172a; margin-bottom: 10px;">
            <img id="featuredImgElem" src="" style="width: 100%; height: 100%; object-fit: cover;" />
        </div>
        <button type="button" class="btn btn-secondary" style="width: 100%; justify-content: center; font-size: 12px;" onclick="openMediaPicker('featured')">+ Set Featured Image</button>"#.to_string()
        }
    } else {
        r#"<div id="featuredImgContainer" style="display: none; height: 140px; border-radius: 8px; overflow: hidden; background: #0f172a; margin-bottom: 10px;">
        <img id="featuredImgElem" src="" style="width: 100%; height: 100%; object-fit: cover;" />
    </div>
    <button type="button" class="btn btn-secondary" style="width: 100%; justify-content: center; font-size: 12px;" onclick="openMediaPicker('featured')">+ Set Featured Image</button>"#.to_string()
    }
}
