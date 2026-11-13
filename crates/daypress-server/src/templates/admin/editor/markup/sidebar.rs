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
    pub initial_lang_code: &'a str,
    pub initial_coauthor_ids: &'a str,
    pub initial_author_display: &'a str,
    pub initial_sticky_checked: &'a str,
    pub initial_password: &'a str,
    pub comment_open_checked: &'a str,
    pub ping_open_checked: &'a str,
    pub gate_public_sel: &'a str,
    pub gate_subscribers_sel: &'a str,
    pub gate_members_sel: &'a str,
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
                    r#"<label style="display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer; padding-left: {indent}px;"><input type="checkbox" class="taxonomy-term-checkbox" data-taxonomy="{tax_slug}" name="post_category" value="{term_id}" {checked} /><span>{term_name}</span></label>"#,
                    indent = indent, tax_slug = tax_slug, term_id = term.id, checked = checked, term_name = term_name,
                ));
            }
            if terms_html.is_empty() {
                terms_html = format!(
                    r#"<p style="font-size: 12px; color: var(--ink-muted); margin: 0 0 8px 0;">No {} found.</p>"#,
                    tax_name
                );
            }
            out.push_str(&format!(
                r#"<div class="day-card" id="taxonomy-card-{tax_slug}" data-metabox-id="taxonomies" data-metabox-title="{tax_name}">
                    <div class="day-card-header"><h2 class="day-card-title">{tax_name}</h2></div>
                    <div class="day-card-body">
                        <div id="taxonomy-list-{tax_slug}" style="max-height: 160px; overflow-y: auto; padding: 8px; border: 1px solid var(--rule); background: var(--paper-raised); margin-bottom: 10px;">{terms_html}</div>
                        <div style="display: flex; gap: 6px; align-items: stretch;">
                            <input type="text" id="new_term_{tax_slug}" placeholder="New {tax_name}..." class="form-input" style="padding: 4px 8px; font-size: 12.5px;" onkeydown="if(event.key==='Enter'){{event.preventDefault();inlineAddTaxonomyTerm('{tax_slug}');}}" />
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 0 10px;" onclick="inlineAddTaxonomyTerm('{tax_slug}')">Add</button>
                        </div>
                    </div>
                </div>"#
            ));
        } else {
            let assigned_names: Vec<String> = tax
                .terms
                .iter()
                .filter(|t| tax.assigned_term_ids.contains(&t.id))
                .map(|t| t.name.clone())
                .collect();
            let initial_val = escape_html(&assigned_names.join(", "));
            let mut known_map = serde_json::Map::new();
            for term in &tax.terms {
                known_map.insert(term.name.to_lowercase(), serde_json::json!(term.id));
            }
            let known_terms_esc = escape_html(
                &serde_json::to_string(&known_map).unwrap_or_else(|_| "{}".to_string()),
            );
            let mut chips_html = String::new();
            for term in tax.terms.iter().take(12) {
                let t_esc = escape_html(&term.name);
                chips_html.push_str(&format!(
                    r#"<span class="badge" onclick="appendFlatTaxonomyTerm('{tax_slug}', '{t_esc}')" style="cursor: pointer; font-size: 11px; background: var(--paper-sunk); color: var(--ink-muted);">+ {t_esc}</span>"#
                ));
            }
            out.push_str(&format!(
                r#"<div class="day-card" id="taxonomy-card-{tax_slug}" data-metabox-id="taxonomies" data-metabox-title="{tax_name}">
                    <div class="day-card-header"><h2 class="day-card-title">{tax_name}</h2></div>
                    <div class="day-card-body">
                        <input type="text" id="taxonomy_input_{tax_slug}" class="form-input taxonomy-flat-input" data-taxonomy="{tax_slug}" data-known-terms="{known_terms_esc}" value="{initial_val}" placeholder="Separate with commas..." style="margin-bottom: 8px;" />
                        <small style="color: var(--ink-muted); font-size: 11.5px; margin-bottom: 10px; display: block;">Separate {tax_name} with commas.</small>
                        <div style="font-size: 11px; color: var(--ink-muted); margin-bottom: 6px;">Popular {tax_name}:</div>
                        <div style="display: flex; flex-wrap: wrap; gap: 6px;">{chips_html}</div>
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
            <div class="day-card" data-metabox-id="taxonomies" style="{post_tax_style}">
                <div class="day-card-header"><h2 class="day-card-title">Categories</h2></div>
                <div class="day-card-body">
                    <div id="categoryList" style="max-height: 160px; overflow-y: auto; padding: 8px; border: 1px solid var(--rule); background: var(--paper-raised); margin-bottom: 10px;">
                        {cat_checkboxes}
                    </div>
                    <div style="display: flex; gap: 6px; align-items: stretch;">
                        <input type="text" id="newCatName" placeholder="New category..." class="form-input" style="padding: 4px 8px; font-size: 12.5px;" onkeydown="if(event.key==='Enter'){{event.preventDefault();inlineAddCategory();}}" />
                        <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 0 10px;" onclick="inlineAddCategory()">Add</button>
                    </div>
                </div>
            </div>
            <div class="day-card" data-metabox-id="taxonomies" style="{post_tax_style}">
                <div class="day-card-header"><h2 class="day-card-title">Tags</h2></div>
                <div class="day-card-body">
                    <input type="text" id="post_tags" class="form-input" value="{initial_tag_str}" placeholder="rust, axum, sqlite..." style="margin-bottom: 6px;" />
                    <small style="color: var(--ink-muted); font-size: 11.5px; margin-bottom: 8px; display: block;">Separate tags with commas.</small>
                    <div style="font-size: 11px; color: var(--ink-muted); margin-bottom: 6px;">Choose from the most used tags:</div>
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

    let initial_lang_code = escape_html(p.initial_lang_code);
    let initial_coauthor_ids = escape_html(p.initial_coauthor_ids);
    let initial_author_display = escape_html(p.initial_author_display);
    let initial_password = escape_html(p.initial_password);

    format!(
        r#"<div id="editorSideColumn" style="display: flex; flex-direction: column; gap: 20px;">
            <!-- 1. Publish Card (#submitdiv) -->
            <div class="day-card" id="metabox-status_visibility" data-metabox-id="status_visibility" data-metabox-title="Publish">
                <div class="day-card-header"><h2 class="day-card-title">Publish</h2></div>
                <div class="day-card-body" style="padding: 0;">
                    <div id="minor-publishing" style="padding: 10px 12px 12px 12px;">
                        <div id="minor-publishing-actions" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; height: 28px; min-height: 28px; padding: 0 10px;" onclick="document.getElementById('post_status').value='draft'; document.getElementById('postEditorForm').requestSubmit();">Save Draft</button>
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; height: 28px; min-height: 28px; padding: 0 10px;" onclick="window.open(document.getElementById('viewPageLink').href, '_blank')">Preview</button>
                        </div>
                        <div id="misc-publishing-actions" style="display: flex; flex-direction: column; gap: 8px; font-size: 13px; color: var(--ink);">
                            <div style="display: flex; align-items: center; gap: 6px;">
                                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-muted)" stroke-width="2" style="flex-shrink:0;"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>
                                <label for="post_status" style="color: var(--ink-soft);">Status:</label>
                                <select id="post_status" class="form-select" style="height: 26px; min-height: 26px; padding: 0 22px 0 8px; font-size: 12.5px; font-weight: 600; width: auto; flex: 1;">
                                    <option value="publish" {is_pub_selected}>Published</option><option value="draft" {is_draft_selected}>Draft</option><option value="pending" {is_pending_selected}>Pending Review</option><option value="private" {is_private_selected}>Private</option><option value="future" {is_future_selected}>Scheduled</option>
                                </select>
                            </div>
                            <div style="display: flex; align-items: center; gap: 6px;">
                                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-muted)" stroke-width="2" style="flex-shrink:0;"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                                <label for="post_member_gate" style="color: var(--ink-soft);">Visibility:</label>
                                <select id="post_member_gate" class="form-select" style="height: 26px; min-height: 26px; padding: 0 22px 0 8px; font-size: 12.5px; font-weight: 600; width: auto; flex: 1;" onchange="markDirty()">
                                    <option value="public" {gate_public_sel}>Public</option>
                                    <option value="subscribers_only" {gate_subscribers_sel}>Subscribers Only</option>
                                    <option value="members_only" {gate_members_sel}>Members Only</option>
                                </select>
                            </div>
                            {schedule_html}
                            <div style="display: flex; align-items: center; gap: 6px;">
                                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-muted)" stroke-width="2" style="flex-shrink:0;"><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                                <span style="color: var(--ink-soft);">Published on: <strong style="color: var(--ink); font-size: 12px;">{initial_date}</strong></span>
                            </div>
                            <details style="font-size: 12px; color: var(--ink-soft); border-top: 1px solid var(--rule); padding-top: 8px; margin-top: 2px;">
                                <summary style="cursor: pointer; color: var(--press); font-weight: 500; user-select: none;">More publish options &amp; author ({initial_author_display})</summary>
                                <div style="display: flex; flex-direction: column; gap: 8px; padding-top: 8px;">
                                    <div>
                                        <label class="form-label" style="font-size: 11.5px;">Password Protection</label>
                                        <input type="text" id="post_password" class="form-input" value="{initial_password}" placeholder="Optional password..." style="height: 26px; padding: 2px 8px; font-size: 12px;" oninput="markDirty()" />
                                    </div>
                                    <label style="display: flex; align-items: center; gap: 6px; font-size: 12px; cursor: pointer; color: var(--ink);"><input type="checkbox" id="post_sticky" {initial_sticky_checked} onchange="markDirty()" /><span>Stick this post to the front page</span></label>
                                    <label style="display: flex; align-items: center; gap: 6px; font-size: 12px; cursor: pointer; color: var(--ink);"><input type="checkbox" id="send_newsletter_digest" onchange="markDirty()" /><span>Email digest to subscribers on publish</span></label>
                                    <div>
                                        <label class="form-label" style="font-size: 11.5px;">Author &amp; Co-Authors</label>
                                        <div id="coauthorsPillBox" style="display: flex; flex-direction: column; gap: 4px; margin-bottom: 6px;">
                                            <div style="display: flex; align-items: center; gap: 6px; font-size: 12px; padding: 3px 6px; border: 1px solid var(--rule); border-radius: 3px; background: var(--paper);">
                                                <span style="flex: 1; font-weight: 600;">{initial_author_display}</span>
                                            </div>
                                        </div>
                                        <div style="display: flex; gap: 4px; margin-bottom: 4px;">
                                            <select id="coauthorAddSelect" class="form-select" style="height: 26px; min-height: 26px; padding: 0 22px 0 6px; font-size: 12px; flex: 1;"><option value="">+ Add co-author...</option></select>
                                            <button type="button" class="btn btn-secondary" style="height: 26px; min-height: 26px; font-size: 11.5px; padding: 0 8px;" onclick="addCoauthorFromSelect()">Add</button>
                                        </div>
                                        <input type="text" id="post_coauthor_ids" class="form-input" value="{initial_coauthor_ids}" placeholder="Co-author IDs (e.g. 1, 2)" style="height: 26px; padding: 2px 6px; font-size: 11.5px; font-family: var(--font-mono);" oninput="renderCoauthorPillsFromInput()" />
                                    </div>
                                    <div style="font-size: 11px; color: var(--ink-muted);">Modified: {initial_modified}</div>
                                </div>
                            </details>
                        </div>
                    </div>
                    <div id="major-publishing-actions" style="padding: 10px 12px; background: var(--paper); border-top: 1px solid var(--rule); display: flex; justify-content: space-between; align-items: center;">
                        <button type="button" onclick="trashPost()" style="color: var(--danger); font-size: 13px; background: none; border: none; cursor: pointer; padding: 0; text-decoration: underline;">Move to Trash</button>
                        <button type="submit" class="btn btn-primary dp-publish-btn">{btn_text}</button>
                    </div>
                </div>
            </div>
            <!-- 2. Categories & Tags (Posts) -->
            {taxonomies_section}
            <!-- 2b. Page Attributes (Pages) -->
            <div class="day-card" id="metabox-page_attributes" data-metabox-id="page_attributes" data-metabox-title="Page Attributes" style="{page_attr_style}">
                <div class="day-card-header"><h2 class="day-card-title">Page Attributes</h2></div>
                <div class="day-card-body">
                    <div class="form-group">
                        <label class="form-label">Parent</label>
                        <select id="post_parent" class="form-select">{parent_options}</select>
                    </div>
                    <div class="form-group">
                        <label class="form-label">Template</label>
                        <select id="page_template" class="form-select">
                            <option value="default" {tpl_default_sel}>Default template</option>
                            <option value="full-width" {tpl_full_sel}>Full-Width Clean Canvas</option>
                            <option value="canvas" {tpl_canvas_sel}>DayStudio Canvas (No Header/Footer)</option>
                            <option value="landing" {tpl_landing_sel}>Landing Page Hero</option>
                            <option value="editorial" {tpl_editorial_sel}>Editorial Longform</option>
                        </select>
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">Order</label>
                        <input type="number" id="menu_order" class="form-input" value="{initial_menu_order}" style="width: 80px;" />
                    </div>
                </div>
            </div>
            <!-- 3. Featured Image -->
            <div class="day-card" id="metabox-featured_image" data-metabox-id="featured_image" data-metabox-title="Featured image">
                <div class="day-card-header"><h2 class="day-card-title">Featured image</h2></div>
                <div class="day-card-body">{featured_img_preview}</div>
            </div>
            <!-- 4. Language & Translations -->
            <div class="day-card" id="editorI18nCard" data-metabox-id="i18n" data-metabox-title="Language &amp; Translations">
                <div class="day-card-header"><h2 class="day-card-title">Language &amp; Translations</h2></div>
                <div class="day-card-body">
                    <div class="form-group" style="margin-bottom: 10px;">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                            <label class="form-label" style="margin-bottom: 0;">Content Language</label>
                            <a href="/d-admin/languages" target="_blank" style="font-size: 11.5px; color: var(--press);">Manage Languages</a>
                        </div>
                        <select id="post_lang_code" class="form-select" data-initial-lang="{initial_lang_code}">
                            <option value="{initial_lang_code}" selected>{initial_lang_code}</option>
                        </select>
                    </div>
                    <div id="editorTranslationsBox" style="font-size: 12px; color: var(--ink-muted); border-top: 1px solid var(--rule); padding-top: 10px;">
                        <div id="editorTranslationsLinksList" style="display: flex; flex-direction: column; gap: 5px; margin-bottom: 8px;"></div>
                        <div style="display: flex; gap: 6px; align-items: stretch; margin-bottom: 8px;">
                            <select id="translateTargetLang" class="form-select" style="padding: 4px 24px 4px 8px; font-size: 12px; flex: 1;">
                                <option value="de">Deutsch (de)</option><option value="nl">Nederlands (nl)</option><option value="fr">Fran&ccedil;ais (fr)</option><option value="es">Espa&ntilde;ol (es)</option><option value="en">English (en)</option>
                            </select>
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 0 10px; white-space: nowrap;" onclick="createPostTranslationDraft()">+ Translate</button>
                        </div>
                        <div style="display: flex; gap: 6px; align-items: stretch;">
                            <input type="number" id="linkExistingTranslationPostId" class="form-input" placeholder="Link existing Post ID..." style="padding: 4px 8px; font-size: 12px; flex: 1;" />
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 0 10px; white-space: nowrap;" onclick="linkEditorExistingTranslation()">Link</button>
                        </div>
                    </div>
                </div>
            </div>
            <!-- 5. Header & Footer Assignment -->
            <div class="day-card" id="editorHeaderFooterCard" data-metabox-id="header_footer" data-metabox-title="Header &amp; Footer">
                <div class="day-card-header"><h2 class="day-card-title">Header &amp; Footer</h2></div>
                <div class="day-card-body">
                    <div class="form-group">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                            <label class="form-label" style="margin-bottom: 0;">Active Header</label>
                            <a id="editorEditHeaderLink" href="/d-admin/headers-footers" style="display: none; font-size: 11.5px; color: var(--press);">Edit in DayStudio</a>
                        </div>
                        <div id="editorResolvedHeaderSummary" style="font-size: 11.5px; color: var(--ink-muted); background: var(--paper); border: 1px solid var(--rule); border-radius: 3px; padding: 5px 8px; margin-bottom: 6px;">Resolving active header...</div>
                        <select id="editorHeaderSelect" class="form-select" onchange="onEditorSlotChange()">
                            <option value="inherit">Inherit Global Rules</option><option value="fallback">Built-in Theme Fallback Header</option><option value="none">Hide / No Header on this Page</option>
                        </select>
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                            <label class="form-label" style="margin-bottom: 0;">Active Footer</label>
                            <a id="editorEditFooterLink" href="/d-admin/headers-footers" style="display: none; font-size: 11.5px; color: var(--press);">Edit in DayStudio</a>
                        </div>
                        <div id="editorResolvedFooterSummary" style="font-size: 11.5px; color: var(--ink-muted); background: var(--paper); border: 1px solid var(--rule); border-radius: 3px; padding: 5px 8px; margin-bottom: 6px;">Resolving active footer...</div>
                        <select id="editorFooterSelect" class="form-select" onchange="onEditorSlotChange()">
                            <option value="inherit">Inherit Global Rules</option><option value="fallback">Built-in Theme Fallback Footer</option><option value="none">Hide / No Footer on this Page</option>
                        </select>
                    </div>
                </div>
            </div>
            <!-- 6. Editorial Workflow -->
            <div class="day-card" id="editorEditorialWorkflowCard" data-metabox-id="editorial_workflow" data-metabox-title="Editorial Workflow">
                <div class="day-card-header"><h2 class="day-card-title">Editorial Workflow</h2></div>
                <div class="day-card-body">
                    <div style="margin-bottom: 12px;">
                        <div style="font-size: 12px; font-weight: 600; color: var(--ink); margin-bottom: 4px;">Shareable Draft Preview Links</div>
                        <small style="color: var(--ink-muted); font-size: 11px; display: block; margin-bottom: 8px;">Generate a secret no-login preview link with <code>noindex</code> protection.</small>
                        <div style="display: flex; gap: 6px; align-items: stretch; margin-bottom: 8px;">
                            <select id="previewTtlHours" class="form-select" style="padding: 4px 24px 4px 8px; font-size: 12px; width: 110px;">
                                <option value="24">24 hours</option><option value="72" selected>72 hours</option><option value="168">7 days</option>
                            </select>
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 0 10px; flex: 1;" onclick="generateSecretPreviewLink()">+ Create Link</button>
                        </div>
                        <div id="editorPreviewLinksList" style="display: flex; flex-direction: column; gap: 6px; font-size: 11.5px;"></div>
                    </div>
                    <div style="border-top: 1px solid var(--rule); padding-top: 10px;">
                        <div style="font-size: 12px; font-weight: 600; color: var(--ink); margin-bottom: 4px;">Scheduled Update to Live Page</div>
                        <div id="editorScheduledRevisionBadge" style="display: none; margin-bottom: 8px; padding: 6px 8px; border-radius: 3px; background: var(--ochre-wash); border: 1px solid var(--ochre); font-size: 11.5px;"></div>
                        <div style="display: flex; gap: 6px; align-items: stretch;">
                            <input type="datetime-local" id="scheduleRevisionAtInput" class="form-input" style="padding: 4px 8px; font-size: 12px; flex: 1;" />
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 0 10px; white-space: nowrap;" onclick="scheduleLiveRevisionUpdate()">Schedule</button>
                        </div>
                    </div>
                </div>
            </div>
            <!-- 7. Discussion & Revisions -->
            <div class="day-card" id="metabox-discussion" data-metabox-id="discussion" data-metabox-title="Discussion">
                <div class="day-card-header"><h2 class="day-card-title">Discussion</h2></div>
                <div class="day-card-body">
                    <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 8px; cursor: pointer; color: var(--ink);"><input type="checkbox" id="post_comment_status" {comment_open_checked} onchange="markDirty()" /><span>Allow comments</span></label>
                    <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; color: var(--ink);"><input type="checkbox" id="post_ping_status" {ping_open_checked} onchange="markDirty()" /><span>Allow trackbacks and pingbacks on this page</span></label>
                </div>
            </div>
            <div class="day-card" id="editorRevisionsCard" data-metabox-id="revisions" data-metabox-title="Revisions" style="display: none;">
                <div class="day-card-header"><h2 class="day-card-title">Revisions (<span id="editorRevisionsCount">0</span>)</h2></div>
                <div class="day-card-body"><div id="editorRevisionsList" style="display: flex; flex-direction: column; gap: 8px; max-height: 220px; overflow-y: auto;"><div style="font-size: 12px; color: var(--ink-muted);">No revisions recorded yet.</div></div></div>
            </div>
        </div>"#,
        is_pub_selected = p.is_pub_selected,
        is_draft_selected = p.is_draft_selected,
        is_pending_selected = p.is_pending_selected,
        is_private_selected = p.is_private_selected,
        is_future_selected = p.is_future_selected,
        schedule_html = p.schedule_html,
        initial_password = initial_password,
        initial_sticky_checked = p.initial_sticky_checked,
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
        comment_open_checked = p.comment_open_checked,
        ping_open_checked = p.ping_open_checked,
        featured_img_preview = p.featured_img_preview,
        gate_public_sel = p.gate_public_sel,
        gate_subscribers_sel = p.gate_subscribers_sel,
        gate_members_sel = p.gate_members_sel,
    )
}
