//! Post editor sidebar with Status, Attributes, Taxonomies, and Featured Image (`markup/sidebar.rs`).

use crate::models::TaxonomyInfo;
use crate::templates::utils::escape_html;

pub struct SidebarParams<'a> {
    pub btn_text: &'a str,
    pub is_pub_selected: &'a str,
    pub is_draft_selected: &'a str,
    pub is_pending_selected: &'a str,
    pub is_private_selected: &'a str,
    pub is_future_selected: &'a str,
    /// Date/time picker for the "Scheduled" status (`markup/schedule.rs`).
    pub schedule_html: &'a str,
    pub initial_date: &'a str,
    pub initial_modified: &'a str,
    pub page_attr_style: &'a str,
    pub post_tax_style: &'a str,
    pub parent_options: &'a str,
    pub tpl_default_sel: &'a str,
    pub tpl_full_sel: &'a str,
    pub tpl_canvas_sel: &'a str,
    pub tpl_landing_sel: &'a str,
    pub tpl_editorial_sel: &'a str,
    pub initial_menu_order: i64,
    pub cat_checkboxes: &'a str,
    pub initial_tag_str: &'a str,
    pub popular_tag_chips: &'a str,
    pub featured_img_preview: &'a str,
    pub dynamic_taxonomies_html: &'a str,
}

pub fn render_taxonomy_cards(taxonomies: &[TaxonomyInfo], is_page: bool) -> String {
    if is_page && taxonomies.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for tax in taxonomies {
        let tax_slug = escape_html(&tax.slug);
        let tax_name = escape_html(&tax.name);

        if tax.hierarchical {
            let mut terms_html = String::new();
            for term in &tax.terms {
                let checked = if tax.assigned_term_ids.contains(&term.id) {
                    "checked"
                } else {
                    ""
                };
                let indent = if term.parent > 0 { 16 } else { 0 };
                let term_name = escape_html(&term.name);

                terms_html.push_str(&format!(
                    r#"<label style="display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer; padding-left: {indent}px;">
                        <input type="checkbox" class="taxonomy-term-checkbox" data-taxonomy="{tax_slug}" name="post_category" value="{term_id}" {checked} />
                        <span>{term_name}</span>
                    </label>"#,
                    indent = indent,
                    tax_slug = tax_slug,
                    term_id = term.id,
                    checked = checked,
                    term_name = term_name,
                ));
            }

            if terms_html.is_empty() {
                terms_html = format!(
                    r#"<p style="font-size: 12px; color: var(--ink-muted); margin: 0 0 8px 0;">No {} found.</p>"#,
                    tax_name
                );
            }

            out.push_str(&format!(
                r#"<div class="day-card" id="taxonomy-card-{tax_slug}">
                    <div class="day-card-header" style="display: flex; justify-content: space-between; align-items: center;">
                        <h2 class="day-card-title">{tax_name}</h2>
                        <a href="/d-admin/edit-tags?taxonomy={tax_slug}" target="_blank" style="font-size: 11.5px; color: var(--press); text-decoration: none;">Manage</a>
                    </div>
                    <div class="day-card-body">
                        <div id="taxonomy-list-{tax_slug}" style="max-height: 160px; overflow-y: auto; padding-right: 4px; margin-bottom: 12px;">
                            {terms_html}
                        </div>
                        <div style="border-top: 1px solid var(--rule); padding-top: 10px;">
                            <div style="display: flex; gap: 6px;">
                                <input type="text" id="new_term_{tax_slug}" placeholder="New {tax_name}..." class="form-input" style="height: 28px; font-size: 12px;" />
                                <button type="button" class="btn btn-secondary" style="height: 28px; font-size: 11.5px; padding: 0 8px;" onclick="inlineAddTaxonomyTerm('{tax_slug}')">Add</button>
                            </div>
                        </div>
                    </div>
                </div>"#
            ));
        } else {
            // Flat taxonomy (like post_tag)
            let assigned_names: Vec<String> = tax
                .terms
                .iter()
                .filter(|t| tax.assigned_term_ids.contains(&t.id))
                .map(|t| t.name.clone())
                .collect();
            let initial_val = escape_html(&assigned_names.join(", "));

            let mut chips_html = String::new();
            for term in tax.terms.iter().take(12) {
                let t_esc = escape_html(&term.name);
                chips_html.push_str(&format!(
                    r#"<span class="badge" onclick="appendFlatTaxonomyTerm('{tax_slug}', '{t_esc}')" style="cursor: pointer; font-size: 11px; background: var(--paper-sunk); color: var(--ink-muted);">+ {t_esc}</span>"#
                ));
            }

            out.push_str(&format!(
                r#"<div class="day-card" id="taxonomy-card-{tax_slug}">
                    <div class="day-card-header">
                        <h2 class="day-card-title">{tax_name}</h2>
                    </div>
                    <div class="day-card-body">
                        <input type="text" id="taxonomy_input_{tax_slug}" class="form-input taxonomy-flat-input" data-taxonomy="{tax_slug}" value="{initial_val}" placeholder="Separate with commas..." style="margin-bottom: 8px;" />
                        <small style="color: var(--ink-muted); font-size: 11.5px; margin-bottom: 10px; display: block;">Separate {tax_name} with commas.</small>
                        <div style="font-size: 11px; color: var(--ink-muted); margin-bottom: 6px;">Popular {tax_name}:</div>
                        <div style="display: flex; flex-wrap: wrap; gap: 6px;">
                            {chips_html}
                        </div>
                    </div>
                </div>"#
            ));
        }
    }

    out
}

pub fn render_editor_sidebar(p: &SidebarParams) -> String {
    let taxonomies_section = if !p.dynamic_taxonomies_html.is_empty() {
        p.dynamic_taxonomies_html.to_string()
    } else {
        format!(
            r#"<!-- Legacy Category & Tag Cards -->
            <div class="day-card" style="{post_tax_style}">
                <div class="day-card-header" style="display: flex; justify-content: space-between; align-items: center;">
                    <h2 class="day-card-title">Categories</h2>
                    <a href="/d-admin/edit-tags?taxonomy=category" target="_blank" style="font-size: 11.5px; color: var(--press); text-decoration: none;">Manage</a>
                </div>
                <div class="day-card-body">
                    <div id="categoryList" style="max-height: 160px; overflow-y: auto; padding-right: 4px; margin-bottom: 12px;">
                        {cat_checkboxes}
                    </div>
                    <div style="border-top: 1px solid var(--rule); padding-top: 10px;">
                        <div style="display: flex; gap: 6px;">
                            <input type="text" id="newCatName" placeholder="New category..." class="form-input" style="height: 28px; font-size: 12px;" />
                            <button type="button" class="btn btn-secondary" style="height: 28px; font-size: 11.5px; padding: 0 8px;" onclick="inlineAddCategory()">Add</button>
                        </div>
                    </div>
                </div>
            </div>
            <div class="day-card" style="{post_tax_style}">
                <div class="day-card-header">
                    <h2 class="day-card-title">Tags</h2>
                </div>
                <div class="day-card-body">
                    <input type="text" id="post_tags" class="form-input" value="{initial_tag_str}" placeholder="rust, axum, sqlite..." style="margin-bottom: 8px;" />
                    <small style="color: var(--ink-muted); font-size: 11.5px; margin-bottom: 10px; display: block;">Separate tags with commas.</small>
                    <div style="font-size: 11px; color: var(--ink-muted); margin-bottom: 6px;">Popular Tags:</div>
                    <div style="display: flex; flex-wrap: wrap; gap: 6px;">
                        {popular_tag_chips}
                    </div>
                </div>
            </div>"#,
            post_tax_style = p.post_tax_style,
            cat_checkboxes = p.cat_checkboxes,
            initial_tag_str = p.initial_tag_str,
            popular_tag_chips = p.popular_tag_chips
        )
    };

    format!(
        r#"<div style="display: flex; flex-direction: column; gap: 20px;">
            <!-- 1. Publish Card -->
            <div class="day-card">
                <div class="day-card-header">
                    <h2 class="day-card-title">Status &amp; Visibility</h2>
                </div>
                <div class="day-card-body">
                    <div class="form-group">
                        <label class="form-label">Status</label>
                        <select id="post_status" class="form-select">
                            <option value="publish" {is_pub_selected}>Published</option>
                            <option value="draft" {is_draft_selected}>Draft</option>
                            <option value="pending" {is_pending_selected}>Pending Review</option>
                            <option value="private" {is_private_selected}>Private</option>
                            <option value="future" {is_future_selected}>Scheduled</option>
                        </select>
                    </div>
                    {schedule_html}

                    <div class="form-group">
                        <label class="form-label">Author</label>
                        <div style="display: flex; align-items: center; gap: 8px; font-size: 13px;">
                            <div class="user-avatar" style="width: 22px; height: 22px; font-size: 10px;">B</div>
                            <span>admin (Administrator)</span>
                        </div>
                    </div>

                    <div style="font-size: 12px; color: var(--ink-muted); margin-top: 10px; line-height: 1.5; border-top: 1px solid var(--rule); padding-top: 10px;">
                        <div>Created: <span style="font-family: var(--font-mono);">{initial_date}</span></div>
                        <div>Modified: <span style="font-family: var(--font-mono);">{initial_modified}</span></div>
                    </div>

                    <div style="border-top: 1px solid var(--rule); padding-top: 14px; margin-top: 14px; display: flex; justify-content: space-between; align-items: center;">
                        <button type="button" onclick="trashPost()" style="color: var(--oxide); font-size: 12px; background: none; border: none; cursor: pointer; padding: 4px;">Move to Trash</button>
                        <button type="submit" class="btn btn-primary dp-publish-btn">{btn_text}</button>
                    </div>
                </div>
            </div>

            <!-- 2. Page Attributes Card (when post_type == page) -->
            <div class="day-card" style="{page_attr_style}">
                <div class="day-card-header">
                    <h2 class="day-card-title">Page Attributes</h2>
                </div>
                <div class="day-card-body">
                    <div class="form-group">
                        <label class="form-label">Parent Page</label>
                        <select id="post_parent" class="form-select">
                            {parent_options}
                        </select>
                        <small style="color: var(--ink-muted); font-size: 11.5px; margin-top: 4px; display: block;">Organize pages into hierarchical sub-pages.</small>
                    </div>

                    <div class="form-group">
                        <label class="form-label">Template</label>
                        <select id="page_template" class="form-select">
                            <option value="default" {tpl_default_sel}>Default Template</option>
                            <option value="full-width" {tpl_full_sel}>Full-Width Clean Canvas</option>
                            <option value="canvas" {tpl_canvas_sel}>Elementor Canvas (No Header/Footer)</option>
                            <option value="landing" {tpl_landing_sel}>Landing Page Hero</option>
                            <option value="editorial" {tpl_editorial_sel}>Editorial Longform</option>
                        </select>
                    </div>

                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">Order</label>
                        <input type="number" id="menu_order" class="form-input" value="{initial_menu_order}" style="width: 100px;" />
                        <small style="color: var(--ink-muted); font-size: 11.5px; margin-top: 4px; display: block;">Numerical ordering for navigation menus.</small>
                    </div>
                </div>
            </div>

            <!-- 2b. Header & Footer Assignment Card -->
            <div class="day-card" id="editorHeaderFooterCard">
                <div class="day-card-header" style="display: flex; justify-content: space-between; align-items: center;">
                    <h2 class="day-card-title">Header &amp; Footer</h2>
                    <a href="/d-admin/headers-footers" target="_blank" style="font-size: 11.5px; color: var(--press); text-decoration: none; font-weight: 600;">Manager &nearr;</a>
                </div>
                <div class="day-card-body">
                    <div class="form-group">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                            <label class="form-label" style="margin-bottom: 0;">Active Header</label>
                            <a id="editorEditHeaderLink" href="/d-admin/headers-footers" style="display: none; font-size: 11px; color: var(--press); font-weight: 600; text-decoration: none;">Edit in DayStudio</a>
                        </div>
                        <div id="editorResolvedHeaderSummary" style="font-size: 11.5px; color: var(--ink-muted); background: var(--paper-raised); border: 1px solid var(--rule); border-radius: 6px; padding: 6px 8px; margin-bottom: 6px;">
                            Resolving active header...
                        </div>
                        <select id="editorHeaderSelect" class="form-select" onchange="onEditorSlotChange()">
                            <option value="inherit">Inherit Global Rules</option>
                            <option value="fallback">Built-in Theme Fallback Header</option>
                            <option value="none">Hide / No Header on this Page</option>
                        </select>
                    </div>

                    <div class="form-group" style="margin-bottom: 0;">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                            <label class="form-label" style="margin-bottom: 0;">Active Footer</label>
                            <a id="editorEditFooterLink" href="/d-admin/headers-footers" style="display: none; font-size: 11px; color: var(--press); font-weight: 600; text-decoration: none;">Edit in DayStudio</a>
                        </div>
                        <div id="editorResolvedFooterSummary" style="font-size: 11.5px; color: var(--ink-muted); background: var(--paper-raised); border: 1px solid var(--rule); border-radius: 6px; padding: 6px 8px; margin-bottom: 6px;">
                            Resolving active footer...
                        </div>
                        <select id="editorFooterSelect" class="form-select" onchange="onEditorSlotChange()">
                            <option value="inherit">Inherit Global Rules</option>
                            <option value="fallback">Built-in Theme Fallback Footer</option>
                            <option value="none">Hide / No Footer on this Page</option>
                        </select>
                    </div>
                </div>
            </div>

            <!-- 3. Dynamic Taxonomies -->
            {taxonomies_section}

            <!-- 4. Featured Image Card -->
            <div class="day-card">
                <div class="day-card-header">
                    <h2 class="day-card-title">Featured Image</h2>
                </div>
                <div class="day-card-body">
                    {featured_img_preview}
                </div>
            </div>
        </div>"#,
        is_pub_selected = p.is_pub_selected,
        is_draft_selected = p.is_draft_selected,
        is_pending_selected = p.is_pending_selected,
        is_private_selected = p.is_private_selected,
        is_future_selected = p.is_future_selected,
        schedule_html = p.schedule_html,
        initial_date = p.initial_date,
        initial_modified = p.initial_modified,
        btn_text = p.btn_text,
        page_attr_style = p.page_attr_style,
        parent_options = p.parent_options,
        tpl_default_sel = p.tpl_default_sel,
        tpl_full_sel = p.tpl_full_sel,
        tpl_canvas_sel = p.tpl_canvas_sel,
        tpl_landing_sel = p.tpl_landing_sel,
        tpl_editorial_sel = p.tpl_editorial_sel,
        initial_menu_order = p.initial_menu_order,
        taxonomies_section = taxonomies_section,
        featured_img_preview = p.featured_img_preview,
    )
}
