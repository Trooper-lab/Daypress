//! SSR Markup for WordPress Post/Page Editor (`templates/admin/editor/markup/`).

mod custom_fields;
mod meta;
mod shell;
mod sidebar;

use crate::models::{TaxonomyInfo, WpCategoryResponse, WpMediaResponse, WpPostResponse, WpSettingsResponse};
use crate::templates::admin::layout::render_wp_admin_shell;
use crate::templates::utils::escape_html;
use custom_fields::render_custom_fields_card;
use meta::render_acf_fields_card;
use shell::{render_editor_card, render_media_picker_modal, render_seo_card, EditorCardParams, SeoCardParams};
use sidebar::{render_editor_sidebar, SidebarParams};

#[allow(clippy::too_many_arguments)]
pub fn render_wp_post_new_view(
    settings: &WpSettingsResponse,
    post_type: &str,
    existing: Option<&WpPostResponse>,
    all_pages: &[WpPostResponse],
    all_categories: &[WpCategoryResponse],
    all_tags: &[WpCategoryResponse],
    all_media: &[WpMediaResponse],
    has_visual_layout: bool,
    taxonomies: &[TaxonomyInfo],
) -> String {
    let is_page = post_type == "page";
    let type_singular = match post_type {
        "page" => "Page",
        "post" => "Post",
        "trainingen" => "Training",
        "vacatures" => "Vacature",
        "cases" => "Case",
        "kandidaten" => "Kandidaat",
        "faq" => "FAQ",
        "quotes" => "Quote",
        "logo" => "Logo",
        "reviews" => "Review",
        other => other,
    };

    let (
        initial_seo_title,
        initial_seo_desc,
        initial_seo_kw,
        initial_seo_canonical,
        initial_seo_og_img,
        initial_seo_noindex,
        initial_seo_schema,
    ) = if let Some(post) = existing {
        (
            escape_html(&post.seo_title),
            escape_html(&post.seo_description),
            escape_html(&post.seo_keyword),
            escape_html(&post.seo_canonical),
            escape_html(&post.seo_og_image),
            post.seo_noindex,
            if post.seo_schema_type.is_empty() {
                if is_page { "WebPage".to_string() } else { "Article".to_string() }
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
            if is_page { "WebPage".to_string() } else { "Article".to_string() },
        )
    };

    let (
        initial_id,
        initial_title,
        initial_slug,
        initial_content,
        initial_excerpt,
        initial_status,
        initial_parent,
        initial_menu_order,
        initial_featured_media,
        initial_categories,
        initial_tags,
        initial_date,
        initial_modified,
    ) = if let Some(post) = existing {
        (
            post.id.to_string(),
            escape_html(&post.title.rendered),
            escape_html(&post.slug),
            escape_html(&post.content.rendered),
            escape_html(&post.excerpt.rendered),
            post.status.clone(),
            post.parent,
            post.menu_order,
            post.featured_media,
            post.categories.clone(),
            post.tags.clone(),
            post.date.clone(),
            post.modified.clone(),
        )
    } else {
        (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "publish".to_string(),
            0,
            0,
            0,
            vec![1],
            vec![],
            "".to_string(),
            "".to_string(),
        )
    };

    let title_label = if existing.is_some() {
        format!("Edit {}", type_singular)
    } else {
        format!("Add New {}", type_singular)
    };

    let btn_text = if existing.is_some() {
        format!("Update {}", type_singular)
    } else {
        format!("Publish {}", type_singular)
    };

    let mut parent_options = String::from(r#"<option value="0">(no parent - top level)</option>"#);
    for p in all_pages {
        if existing.map(|e| e.id) != Some(p.id) {
            let sel = if p.id == initial_parent { "selected" } else { "" };
            let t_esc = escape_html(&p.title.rendered);
            parent_options.push_str(&format!(r#"<option value="{}" {}>{}</option>"#, p.id, sel, t_esc));
        }
    }

    let mut cat_checkboxes = String::new();
    for c in all_categories {
        let checked = if initial_categories.contains(&c.id) { "checked" } else { "" };
        let c_name = escape_html(&c.name);
        cat_checkboxes.push_str(&format!(
            r#"<label style="display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer;">
                <input type="checkbox" name="post_category" value="{}" {} />
                <span>{}</span>
            </label>"#,
            c.id, checked, c_name
        ));
    }

    let featured_img_preview = if initial_featured_media > 0 {
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
    };

    let mut media_picker_items = String::new();
    for m in all_media {
        let url_esc = escape_html(&m.source_url);
        let title_esc = escape_html(&m.title.rendered);
        if m.mime_type.starts_with("image/") {
            media_picker_items.push_str(&format!(
                r#"<div class="media-pick-item" onclick="selectMediaItem({}, '{}', '{}')" style="cursor: pointer; border: 2px solid transparent; border-radius: 8px; overflow: hidden; height: 110px; background: #0f172a; position: relative; transition: all 0.15s ease;" onmouseover="this.style.borderColor='#2563eb'" onmouseout="this.style.borderColor='transparent'">
                    <img src="{}" alt="{}" style="width: 100%; height: 100%; object-fit: cover;" loading="lazy" />
                    <div style="position: absolute; bottom: 0; left: 0; right: 0; background: rgba(15,23,42,0.85); color: #ffffff; font-size: 10px; padding: 2px 6px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{}</div>
                </div>"#,
                m.id, url_esc, title_esc, url_esc, title_esc, title_esc
            ));
        }
    }

    let mut popular_tag_chips = String::new();
    for t in all_tags.iter().take(6) {
        let t_name = escape_html(&t.name);
        popular_tag_chips.push_str(&format!(
            r#"<span class="tag-chip" onclick="addTagChip('{}')" style="background: #f1f5f9; color: #475569; font-size: 11px; padding: 2px 8px; border-radius: 4px; cursor: pointer; border: 1px solid #cbd5e1;">+ {}</span>"#,
            t_name, t_name
        ));
    }

    let initial_tag_names: Vec<String> = all_tags.iter().filter(|t| initial_tags.contains(&t.id)).map(|t| t.name.clone()).collect();
    let initial_tag_str = initial_tag_names.join(", ");

    let mut known_tags_entries = Vec::new();
    for t in all_tags {
        known_tags_entries.push(format!(r#""{}": {}"#, escape_html(&t.name.to_lowercase()), t.id));
    }
    let known_tags_json = format!("{{{}}}", known_tags_entries.join(", "));
    let initial_template = existing.map(|p| p.template.as_str()).unwrap_or("default");

    let theme_preview_css = if !settings.custom_css.is_empty() {
        format!(r#"<style id="neopress-editor-theme-preview-css">{}</style>"#, settings.custom_css)
    } else {
        String::new()
    };

    let top_builder_btn = if !initial_id.is_empty() {
        format!(
            r#"<a href="/n-admin/builder?post={}" class="btn" style="background: linear-gradient(135deg, #2563eb 0%, #4f46e5 100%); color: #ffffff; border: none; font-weight: 700; display: inline-flex; align-items: center; gap: 8px; padding: 7px 18px; border-radius: 6px; box-shadow: 0 2px 10px rgba(37, 99, 235, 0.35); text-decoration: none; transition: all 0.2s ease;">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
                <span>⚡ Open in NeoStudio (Live Theme Canvas)</span>
            </a>"#,
            initial_id
        )
    } else {
        r#"<button type="button" class="btn" id="btnEditVisualBuilder" onclick="switchToVisualBuilder()" style="background: linear-gradient(135deg, #2563eb 0%, #4f46e5 100%); color: #ffffff; border: none; font-weight: 600; display: inline-flex; align-items: center; gap: 8px; padding: 7px 16px; border-radius: 6px; box-shadow: 0 2px 8px rgba(37, 99, 235, 0.35); cursor: pointer; transition: all 0.2s ease;">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
            <span>⚡ Open in NeoStudio</span>
        </button>"#.to_string()
    };

    let duplicate_btn_html = if existing.is_some() {
        format!(r#"<button type="button" class="btn btn-secondary" onclick="duplicateCurrentPost()" title="Deep-clone this {}">⧉ Duplicate</button>"#, type_singular)
    } else {
        String::new()
    };

    let editor_card = render_editor_card(&EditorCardParams {
        type_singular,
        initial_title: &initial_title,
        initial_slug: &initial_slug,
        initial_content: &initial_content,
        locked_banner_html: &theme_preview_css,
        standard_editor_display: "display: block;",
    });

    let acf_card = render_acf_fields_card(existing, post_type);

    let seo_card = render_seo_card(&SeoCardParams {
        type_singular,
        site_title: &settings.title,
        initial_title: &initial_title,
        initial_slug: &initial_slug,
        initial_excerpt: &initial_excerpt,
        initial_seo_title: &initial_seo_title,
        initial_seo_desc: &initial_seo_desc,
        initial_seo_kw: &initial_seo_kw,
        initial_seo_canonical: &initial_seo_canonical,
        initial_seo_og_img: &initial_seo_og_img,
        seo_noindex_checked: if initial_seo_noindex { "checked" } else { "" },
        schema_art_sel: if initial_seo_schema == "Article" { "selected" } else { "" },
        schema_web_sel: if initial_seo_schema == "WebPage" { "selected" } else { "" },
        schema_faq_sel: if initial_seo_schema == "FAQPage" { "selected" } else { "" },
        schema_prod_sel: if initial_seo_schema == "Product" { "selected" } else { "" },
        schema_loc_sel: if initial_seo_schema == "LocalBusiness" { "selected" } else { "" },
    });

    let custom_fields_card = render_custom_fields_card(existing);
    let dynamic_taxonomies_html = sidebar::render_taxonomy_cards(taxonomies, is_page);

    let sidebar_html = render_editor_sidebar(&SidebarParams {
        btn_text: &btn_text,
        is_pub_selected: if initial_status == "publish" { "selected" } else { "" },
        is_draft_selected: if initial_status == "draft" { "selected" } else { "" },
        is_pending_selected: if initial_status == "pending" { "selected" } else { "" },
        is_private_selected: if initial_status == "private" { "selected" } else { "" },
        initial_date: &initial_date,
        initial_modified: &initial_modified,
        page_attr_style: if is_page { "" } else { "display: none;" },
        post_tax_style: if !is_page { "" } else { "display: none;" },
        parent_options: &parent_options,
        tpl_default_sel: if initial_template.is_empty() || initial_template == "default" { "selected" } else { "" },
        tpl_full_sel: if initial_template == "full-width" { "selected" } else { "" },
        tpl_canvas_sel: if initial_template == "canvas" { "selected" } else { "" },
        tpl_landing_sel: if initial_template == "landing" { "selected" } else { "" },
        tpl_editorial_sel: if initial_template == "editorial" { "selected" } else { "" },
        initial_menu_order,
        cat_checkboxes: &cat_checkboxes,
        initial_tag_str: &initial_tag_str,
        popular_tag_chips: &popular_tag_chips,
        featured_img_preview: &featured_img_preview,
        dynamic_taxonomies_html: &dynamic_taxonomies_html,
    });

    let media_modal = render_media_picker_modal(&media_picker_items);
    let editor_scripts = super::scripts::render_editor_scripts(&settings.title, &initial_id, has_visual_layout, type_singular, post_type, &known_tags_json);

    let body = format!(
        r#"
        <div class="page-header">
            <div>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h1 class="page-title">{title_label}</h1>
                    <span class="badge badge-published" style="font-size: 11px; text-transform: uppercase;">{post_type}</span>
                </div>
                <p class="page-subtitle">Craft and publish high-performance editorial content directly to embedded Rusqlite</p>
            </div>
            <div style="display: flex; gap: 10px; align-items: center;">
                {top_builder_btn}
                {duplicate_btn_html}
                <button type="button" class="btn btn-secondary" onclick="saveDraft()">Save Draft</button>
                <button type="button" class="btn btn-secondary" onclick="previewPage()">View Live ↗</button>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('postFormSubmit').click()">{btn_text}</button>
            </div>
        </div>

        <form id="postForm" onsubmit="savePost(event)">
            <button type="submit" id="postFormSubmit" style="display: none;"></button>
            <input type="hidden" id="featured_media" value="{initial_featured_media}" />
            <div style="display: grid; grid-template-columns: 3fr 1fr; gap: 24px;">
                <div style="display: flex; flex-direction: column; gap: 20px;">
                    {editor_card}
                    {acf_card}
                    {custom_fields_card}
                    {seo_card}
                </div>
                {sidebar_html}
            </div>
        </form>

        {media_modal}

        <script>
            {editor_scripts}
        </script>
        "#,
        title_label = title_label,
        post_type = post_type,
        top_builder_btn = top_builder_btn,
        duplicate_btn_html = duplicate_btn_html,
        btn_text = btn_text,
        initial_featured_media = initial_featured_media,
        editor_card = editor_card,
        acf_card = acf_card,
        custom_fields_card = custom_fields_card,
        seo_card = seo_card,
        sidebar_html = sidebar_html,
        media_modal = media_modal,
        editor_scripts = editor_scripts,
    );

    let tab = match post_type {
        "page" => "pages",
        "post" => "posts",
        other => other,
    };
    render_wp_admin_shell(tab, &title_label, &body, settings)
}
