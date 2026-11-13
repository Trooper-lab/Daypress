//! SSR Markup for WordPress Post/Page Editor (`templates/admin/editor/markup/`).

mod custom_fields;
mod fields;
mod meta;
mod schedule;
mod shell;
mod sidebar;

use crate::models::{
    DaypressCategoryResponse, DaypressMediaResponse, DaypressPostResponse,
    DaypressSettingsResponse, FieldGroupDefinition, TaxonomyInfo,
};
use crate::templates::admin::layout::render_admin_shell;
use crate::templates::utils::escape_html;
use custom_fields::render_custom_fields_card;
use meta::{render_acf_fields_card, render_structured_field_groups};
use shell::{
    render_editor_card, render_media_picker_modal, render_seo_card, EditorCardParams, SeoCardParams,
};
use sidebar::{render_editor_sidebar, SidebarParams};

#[allow(clippy::too_many_arguments)]
pub fn render_post_new_view(
    settings: &DaypressSettingsResponse,
    post_type: &str,
    existing: Option<&DaypressPostResponse>,
    all_pages: &[DaypressPostResponse],
    all_categories: &[DaypressCategoryResponse],
    all_tags: &[DaypressCategoryResponse],
    all_media: &[DaypressMediaResponse],
    has_visual_layout: bool,
    taxonomies: &[TaxonomyInfo],
) -> String {
    render_post_new_view_with_fields(
        settings,
        post_type,
        existing,
        all_pages,
        all_categories,
        all_tags,
        all_media,
        has_visual_layout,
        taxonomies,
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn render_post_new_view_with_fields(
    settings: &DaypressSettingsResponse,
    post_type: &str,
    existing: Option<&DaypressPostResponse>,
    all_pages: &[DaypressPostResponse],
    all_categories: &[DaypressCategoryResponse],
    all_tags: &[DaypressCategoryResponse],
    all_media: &[DaypressMediaResponse],
    has_visual_layout: bool,
    taxonomies: &[TaxonomyInfo],
    field_groups: &[FieldGroupDefinition],
) -> String {
    let is_page = post_type == "page";
    let formatted_cpt = settings
        .cpt_definitions
        .iter()
        .find(|d| d.slug == post_type && !d.label_singular.trim().is_empty())
        .map(|d| d.label_singular.clone())
        .unwrap_or_else(|| crate::templates::admin::layout::format_cpt_title(post_type));
    let type_singular = match post_type {
        "page" => "Page",
        "post" => "Post",
        _ => formatted_cpt.as_str(),
    };
    // ACF tooling only appears for posts that actually carry ACF data (migrated ACF sites).
    let uses_acf = meta::post_uses_acf(existing);

    let (
        initial_seo_title,
        initial_seo_desc,
        initial_seo_kw,
        initial_seo_canonical,
        initial_seo_og_img,
        initial_seo_noindex,
        initial_seo_schema,
    ) = fields::initial_seo_fields(existing, is_page);

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

    let parent_options = fields::render_parent_options(all_pages, existing, initial_parent);
    let cat_checkboxes = fields::render_category_checkboxes(all_categories, &initial_categories);
    let featured_img_preview =
        fields::render_featured_image_preview(all_media, initial_featured_media);
    let media_picker_items = fields::render_media_picker_items(all_media);
    let (popular_tag_chips, initial_tag_str, known_tags_json) =
        fields::render_tag_chips_and_json(all_tags, &initial_tags);
    let initial_template = existing.map(|p| p.template.as_str()).unwrap_or("default");

    let theme_preview_css = if !settings.custom_css.is_empty() {
        format!(
            r#"<style id="daypress-editor-theme-preview-css">{}</style>"#,
            settings.custom_css
        )
    } else {
        String::new()
    };

    let _add_new_action_btn = if existing.is_some() {
        format!(
            r#"<a href="/d-admin/post-new?post_type={}" class="page-title-action">Add New {}</a>"#,
            escape_html(post_type),
            type_singular
        )
    } else {
        String::new()
    };

    let top_builder_btn = if !initial_id.is_empty() {
        format!(
            r#"<a href="/d-admin/builder?post={}" class="page-title-action">Edit with DayStudio</a>"#,
            initial_id
        )
    } else {
        r#"<button type="button" class="page-title-action" id="btnEditVisualBuilder" onclick="switchToVisualBuilder()">Edit with DayStudio</button>"#.to_string()
    };

    let duplicate_btn_html = if existing.is_some() {
        format!(
            r#"<button type="button" class="page-title-action" onclick="duplicateCurrentPost()" title="Duplicate this {}">Duplicate</button>"#,
            type_singular
        )
    } else {
        String::new()
    };

    let permalink_prefix = if post_type != "post" && post_type != "page" {
        format!("{}/", escape_html(post_type))
    } else {
        String::new()
    };

    let editor_card = render_editor_card(&EditorCardParams {
        type_singular,
        permalink_prefix: &permalink_prefix,
        initial_title: &initial_title,
        initial_slug: &initial_slug,
        initial_content: &initial_content,
        locked_banner_html: &theme_preview_css,
        standard_editor_display: "display: block;",
        acf_inserter_display: "flex",
    });

    let acf_card = if uses_acf {
        render_acf_fields_card(existing, post_type)
    } else {
        String::new()
    };
    let structured_fields_card = render_structured_field_groups(existing, field_groups, all_pages);
    // Gutenberg/ACF content opens in the block inspector; everything else in the visual editor.
    let rich_editing = crate::auth::current_admin()
        .map(|a| a.rich_editing)
        .unwrap_or(true);
    let initial_mode = if uses_acf || initial_content.contains("&lt;!-- wp:") {
        "blocks"
    } else if !rich_editing {
        "html"
    } else {
        "visual"
    };

    let sel = |a: &str, b: &str| if a == b { "selected" } else { "" };
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
        schema_art_sel: sel(&initial_seo_schema, "Article"),
        schema_web_sel: sel(&initial_seo_schema, "WebPage"),
        schema_faq_sel: sel(&initial_seo_schema, "FAQPage"),
        schema_prod_sel: sel(&initial_seo_schema, "Product"),
        schema_loc_sel: sel(&initial_seo_schema, "LocalBusiness"),
    });

    let custom_fields_card = render_custom_fields_card(existing);
    let dynamic_taxonomies_html = sidebar::render_taxonomy_cards(taxonomies, is_page);
    let schedule_html = schedule::render_schedule_controls(existing);
    let initial_lang_code = existing
        .map(|p| p.lang_code.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("en");
    let initial_coauthor_ids = existing
        .map(|p| {
            if p.coauthors.is_empty() {
                p.author.to_string()
            } else {
                p.coauthors
                    .iter()
                    .map(|a| a.id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        })
        .unwrap_or_else(|| "1".to_string());
    let initial_author_display = existing
        .map(|p| p.author_name.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("admin (Administrator)");

    let sidebar_html = render_editor_sidebar(&SidebarParams {
        btn_text: &btn_text,
        is_pub_selected: sel(&initial_status, "publish"),
        is_draft_selected: sel(&initial_status, "draft"),
        is_pending_selected: sel(&initial_status, "pending"),
        is_private_selected: schedule::selected_if(&initial_status, "private"),
        is_future_selected: schedule::selected_if(&initial_status, "future"),
        schedule_html: &schedule_html,
        initial_date: &initial_date,
        initial_modified: &initial_modified,
        page_attr_style: if is_page { "" } else { "display: none;" },
        post_tax_style: if !is_page { "" } else { "display: none;" },
        parent_options: &parent_options,
        tpl_default_sel: if initial_template.is_empty() || initial_template == "default" {
            "selected"
        } else {
            ""
        },
        tpl_full_sel: sel(initial_template, "full-width"),
        tpl_canvas_sel: sel(initial_template, "canvas"),
        tpl_landing_sel: sel(initial_template, "landing"),
        tpl_editorial_sel: sel(initial_template, "editorial"),
        initial_menu_order,
        cat_checkboxes: &cat_checkboxes,
        initial_tag_str: &initial_tag_str,
        popular_tag_chips: &popular_tag_chips,
        featured_img_preview: &featured_img_preview,
        dynamic_taxonomies_html: &dynamic_taxonomies_html,
        initial_lang_code,
        initial_coauthor_ids: &initial_coauthor_ids,
        initial_author_display,
        initial_sticky_checked: if existing.map(|p| p.sticky).unwrap_or(false) {
            "checked"
        } else {
            ""
        },
        initial_password: existing.map(|p| p.password.as_str()).unwrap_or(""),
        comment_open_checked: if existing
            .map(|p| p.comment_status != "closed")
            .unwrap_or(true)
        {
            "checked"
        } else {
            ""
        },
        ping_open_checked: if existing.map(|p| p.ping_status != "closed").unwrap_or(true) {
            "checked"
        } else {
            ""
        },
        gate_public_sel: sel(
            existing.map(|p| p.member_gate.as_str()).unwrap_or("public"),
            "public",
        ),
        gate_subscribers_sel: sel(
            existing.map(|p| p.member_gate.as_str()).unwrap_or("public"),
            "subscribers_only",
        ),
        gate_members_sel: sel(
            existing.map(|p| p.member_gate.as_str()).unwrap_or("public"),
            "members_only",
        ),
    });

    let media_modal = render_media_picker_modal(&media_picker_items);
    let editor_scripts = super::scripts::render_editor_scripts(
        &settings.title,
        &initial_id,
        has_visual_layout,
        initial_mode,
        type_singular,
        post_type,
        &known_tags_json,
    );

    let body = format!(
        r#"<div id="editorScreenOptionsPanel" class="day-card" style="display: none; padding: 14px 18px; margin-bottom: 16px; border: 1px solid var(--rule-strong); background: var(--paper-raised);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; padding-bottom: 8px; border-bottom: 1px solid var(--rule);"><strong style="font-size: 13px; color: var(--ink);">Screen Elements</strong><div style="display: flex; gap: 6px; align-items: center;"><button type="button" class="btn btn-secondary" style="min-height: 26px; padding: 2px 10px; font-size: 12px;" onclick="toggleAllEditorMetaBoxes(true)">Collapse All</button><button type="button" class="btn btn-secondary" style="min-height: 26px; padding: 2px 10px; font-size: 12px;" onclick="toggleAllEditorMetaBoxes(false)">Expand All</button><button type="button" class="btn btn-secondary" style="min-height: 26px; padding: 2px 10px; font-size: 12px;" onclick="resetEditorScreenOptions()">Reset Defaults</button></div></div>
            <div style="margin-bottom: 12px;">
                <div id="editorScreenOptionsCheckboxes" style="display: flex; flex-wrap: wrap; gap: 14px; font-size: 13px;">
                    <label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="status_visibility" checked onchange="onEditorScreenOptionToggle('status_visibility', this.checked)" /> Publish</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="i18n" checked onchange="onEditorScreenOptionToggle('i18n', this.checked)" /> Language &amp; Translations</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="editorial_workflow" checked onchange="onEditorScreenOptionToggle('editorial_workflow', this.checked)" /> Editorial Workflow</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="page_attributes" checked onchange="onEditorScreenOptionToggle('page_attributes', this.checked)" /> Page Attributes</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="header_footer" checked onchange="onEditorScreenOptionToggle('header_footer', this.checked)" /> Header &amp; Footer</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="taxonomies" checked onchange="onEditorScreenOptionToggle('taxonomies', this.checked)" /> Categories &amp; Tags</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="discussion" checked onchange="onEditorScreenOptionToggle('discussion', this.checked)" /> Discussion</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="featured_image" checked onchange="onEditorScreenOptionToggle('featured_image', this.checked)" /> Featured Image</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="revisions" checked onchange="onEditorScreenOptionToggle('revisions', this.checked)" /> Revisions</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="excerpt" checked onchange="onEditorScreenOptionToggle('excerpt', this.checked)" /> Excerpt</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="structured_fields" checked onchange="onEditorScreenOptionToggle('structured_fields', this.checked)" /> Structured Fields</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="acf_settings" checked onchange="onEditorScreenOptionToggle('acf_settings', this.checked)" /> ACF Page Settings</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="custom_fields" checked onchange="onEditorScreenOptionToggle('custom_fields', this.checked)" /> Custom Fields</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-screen-opt-cb" data-box-id="seo_studio" checked onchange="onEditorScreenOptionToggle('seo_studio', this.checked)" /> SEO &amp; Readability</label>
                </div>
            </div>
            <div style="display: flex; align-items: center; gap: 18px; padding-top: 10px; border-top: 1px solid var(--rule); font-size: 13px;"><strong style="font-size: 13px; color: var(--ink);">Layout:</strong><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="radio" name="editorScreenLayoutCols" id="editorCols1" value="1" onchange="setEditorColumnLayout(1)" /> 1 Column</label><label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="radio" name="editorScreenLayoutCols" id="editorCols2" value="2" checked onchange="setEditorColumnLayout(2)" /> 2 Columns</label></div>
        </div>
        <div class="page-header">
            <div><h1 class="page-title">{title_label}</h1>{add_new_action_btn}{top_builder_btn}{duplicate_btn_html}</div>
        </div>
        <form id="postForm" onsubmit="savePost(event)">
            <button type="submit" id="postFormSubmit" style="display: none;"></button>
            <input type="hidden" id="featured_media" value="{initial_featured_media}" />
            <div id="editorColumnsGrid" style="display: grid; grid-template-columns: minmax(0, 1fr) 280px; gap: 20px; align-items: start;"><div id="editorMainColumn" style="display: flex; flex-direction: column; min-width: 0;">{editor_card}{structured_fields_card}{acf_card}{custom_fields_card}{seo_card}</div>{sidebar_html}</div>
        </form>
        {media_modal}<script>{editor_scripts}</script>"#
    );

    let tab = match post_type {
        "page" => "pages",
        "post" => "posts",
        other => other,
    };
    render_admin_shell(tab, &title_label, &body, settings)
}
