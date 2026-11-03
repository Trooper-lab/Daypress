//! Admin Portal — Content Model Hub (`/d-admin/content-types`)
//!
//! Unified registry for Custom Post Types, Custom Taxonomies, and Structured Field Groups (ACF Parity),
//! plus 1-Click Starter Blueprints for Case Studies, Team, Docs, Jobs, Events, Portfolio, and Testimonials.

use crate::models::{
    CustomPostTypeDefinition, CustomTaxonomyDefinition, DaypressSettingsResponse,
    FieldGroupDefinition,
};
use crate::templates::admin::layout::render_admin_shell;
use crate::templates::utils::escape_html;

pub fn render_content_model_page(
    post_types: &[CustomPostTypeDefinition],
    taxonomies: &[CustomTaxonomyDefinition],
    field_groups: &[FieldGroupDefinition],
    active_tab: &str,
    settings: &DaypressSettingsResponse,
) -> String {
    let tab = match active_tab {
        "taxonomies" => "taxonomies",
        "fields" => "fields",
        _ => "post-types",
    };

    let presets = [
        (
            "case_studies",
            "case_study",
            "Case Studies",
            "Client stories, headline metrics & challenge/solution breakdown.",
        ),
        (
            "team",
            "team_member",
            "Team",
            "Team member profiles, role titles, department taxonomy & portraits.",
        ),
        (
            "docs",
            "doc",
            "Docs",
            "Hierarchical technical documentation with versions & difficulty tags.",
        ),
        (
            "jobs",
            "job_listing",
            "Jobs",
            "Career openings with location, employment type, salary & apply URL.",
        ),
        (
            "events",
            "event",
            "Events",
            "Conferences, workshops & webinars with date, venue & ticket fields.",
        ),
        (
            "portfolio",
            "portfolio",
            "Portfolio",
            "Creative & engineering showcases with client, year & deliverables.",
        ),
        (
            "testimonials",
            "testimonial",
            "Testimonials",
            "Verified client quotes, star ratings & relational case study links.",
        ),
    ];

    let mut presets_html = String::new();
    for (pkey, pslug, plabel, pdesc) in presets {
        let is_provisioned = post_types
            .iter()
            .any(|pt| pt.slug == pslug || pt.preset_key.as_deref() == Some(pkey));
        let action_btn = if is_provisioned {
            format!(
                r#"<div style="display:flex;align-items:center;justify-content:space-between;gap:8px;margin-top:auto;padding-top:10px;border-top:1px solid var(--rule);">
                    <span class="badge badge-published" style="font-family:var(--font-mono);font-size:11px;">&#10003; Active (/{pslug}/)</span>
                    <a href="/d-admin/edit?post_type={pslug}" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;">Open &rarr;</a>
                </div>"#
            )
        } else {
            format!(
                r#"<div style="display:flex;align-items:center;justify-content:space-between;gap:8px;margin-top:auto;padding-top:10px;border-top:1px solid var(--rule);">
                    <span style="font-family:var(--font-mono);font-size:11px;color:var(--ink-muted);">/{pslug}/</span>
                    <button type="button" class="btn btn-primary" style="padding:4px 10px;font-size:12px;" onclick="provisionPreset('{pkey}', this)">+ 1-Click Provision</button>
                </div>"#
            )
        };
        presets_html.push_str(&format!(
            r#"<div class="cm-preset-card">
                <div style="font-family:var(--font-serif);font-weight:500;font-size:16px;color:var(--ink);margin-bottom:4px;">{plabel}</div>
                <div style="font-size:12.5px;color:var(--ink-muted);line-height:1.45;margin-bottom:12px;">{pdesc}</div>
                {action_btn}
            </div>"#
        ));
    }

    // 1. Post Types Table Rows
    let mut pt_rows = String::new();
    if post_types.is_empty() {
        pt_rows.push_str(
            r#"<tr><td colspan="6" style="text-align:center;padding:32px;color:var(--ink-muted);">No Custom Post Types registered yet. Click any 1-Click Starter Blueprint above or click <strong>+ New Post Type</strong>.</td></tr>"#,
        );
    } else {
        for pt in post_types {
            let slug_esc = escape_html(&pt.slug);
            let sing_esc = escape_html(&pt.label_singular);
            let plur_esc = escape_html(&pt.label_plural);
            let rewrite_esc = escape_html(if pt.rewrite_slug.is_empty() {
                &pt.slug
            } else {
                &pt.rewrite_slug
            });
            let taxes_badges = if pt.taxonomies.is_empty() {
                r#"<span style="color:var(--ink-muted);font-size:12px;">—</span>"#.to_string()
            } else {
                pt.taxonomies
                    .iter()
                    .map(|t| {
                        format!(
                            r#"<a href="/d-admin/edit-tags?taxonomy={0}" class="cm-mono-pill">{0}</a>"#,
                            escape_html(t)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let pt_json = escape_html(&serde_json::to_string(pt).unwrap_or_else(|_| "{}".into()));
            pt_rows.push_str(&format!(
                r#"<tr>
                    <td class="table-title-cell">
                        <strong><a href="/d-admin/edit?post_type={slug_esc}">{plur_esc}</a></strong>
                        <div class="table-row-slug">Singular: {sing_esc}</div>
                    </td>
                    <td><code class="cm-code">{slug_esc}</code></td>
                    <td><a href="/{rewrite_esc}/" target="_blank" style="font-family:var(--font-mono);font-size:12px;color:var(--press);">/{rewrite_esc}/ &nearr;</a></td>
                    <td>{taxes_badges}</td>
                    <td><span class="cm-count-badge">{count}</span></td>
                    <td style="text-align:right;white-space:nowrap;">
                        <a href="/d-admin/edit?post_type={slug_esc}" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;">Entries</a>
                        <a href="/d-admin/post-new?post_type={slug_esc}" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;">+ Add</a>
                        <button type="button" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;" data-cpt="{pt_json}" onclick="editCptFromBtn(this)">Edit</button>
                        <button type="button" class="btn btn-danger" style="padding:4px 10px;font-size:12px;" onclick="deleteCpt('{slug_esc}')">Delete</button>
                    </td>
                </tr>"#,
                count = pt.post_count
            ));
        }
    }

    // 2. Custom Taxonomies Table Rows
    let mut tax_rows = String::new();
    if taxonomies.is_empty() {
        tax_rows.push_str(
            r#"<tr><td colspan="6" style="text-align:center;padding:32px;color:var(--ink-muted);">No Custom Taxonomies registered yet. Click <strong>+ New Taxonomy</strong> to create one.</td></tr>"#,
        );
    } else {
        for tx in taxonomies {
            let slug_esc = escape_html(&tx.slug);
            let sing_esc = escape_html(&tx.label_singular);
            let plur_esc = escape_html(&tx.label_plural);
            let mode_badge = if tx.hierarchical {
                r#"<span class="cm-mono-pill">Hierarchical (Categories)</span>"#
            } else {
                r#"<span class="cm-mono-pill">Flat (Tags)</span>"#
            };
            let pts_badges = if tx.post_types.is_empty() {
                r#"<span style="color:var(--ink-muted);font-size:12px;">All</span>"#.to_string()
            } else {
                tx.post_types
                    .iter()
                    .map(|p| format!(r#"<span class="cm-mono-pill">{}</span>"#, escape_html(p)))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            let tx_json = escape_html(&serde_json::to_string(tx).unwrap_or_else(|_| "{}".into()));
            tax_rows.push_str(&format!(
                r#"<tr>
                    <td class="table-title-cell">
                        <strong><a href="/d-admin/edit-tags?taxonomy={slug_esc}">{plur_esc}</a></strong>
                        <div class="table-row-slug">Singular: {sing_esc}</div>
                    </td>
                    <td><code class="cm-code">{slug_esc}</code></td>
                    <td>{mode_badge}</td>
                    <td>{pts_badges}</td>
                    <td><span class="cm-count-badge">{count}</span></td>
                    <td style="text-align:right;white-space:nowrap;">
                        <a href="/d-admin/edit-tags?taxonomy={slug_esc}" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;">Manage Terms</a>
                        <button type="button" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;" data-tax="{tx_json}" onclick="editTaxFromBtn(this)">Edit</button>
                        <button type="button" class="btn btn-danger" style="padding:4px 10px;font-size:12px;" onclick="deleteTax('{slug_esc}')">Delete</button>
                    </td>
                </tr>"#,
                count = tx.term_count
            ));
        }
    }

    // 3. Field Groups Cards
    let mut fg_cards = String::new();
    if field_groups.is_empty() {
        fg_cards.push_str(
            r#"<div class="cm-surface" style="text-align:center;padding:36px;color:var(--ink-muted);">No Structured Field Groups defined yet. Provision a starter blueprint above or click <strong>+ New Field Group</strong> to create custom fields.</div>"#,
        );
    } else {
        for fg in field_groups {
            let title_esc = escape_html(&fg.title);
            let gkey_esc = escape_html(&fg.group_key);
            let desc_esc = escape_html(&fg.description);
            let loc_summary = if fg.location_rules.is_empty() {
                "All Post Types".to_string()
            } else {
                fg.location_rules
                    .iter()
                    .map(|r| format!("{} {} {}", r.param, r.operator, r.value))
                    .collect::<Vec<_>>()
                    .join(" OR ")
            };
            let loc_esc = escape_html(&loc_summary);
            let mut field_rows = String::new();
            for f in &fg.fields {
                let fname = escape_html(&f.name);
                let flabel = escape_html(&f.label);
                let ftype = escape_html(&f.field_type);
                let card_badge = if f.show_in_collection_card {
                    r#"<span class="badge badge-published" style="font-size:10.5px;">Grid Card Badge</span>"#
                } else {
                    ""
                };
                let sub_info = if f.field_type == "repeater" && !f.sub_fields.is_empty() {
                    let snames: Vec<String> = f
                        .sub_fields
                        .iter()
                        .map(|sf| escape_html(&sf.name))
                        .collect();
                    format!(
                        r#" <span style="font-size:11px;color:var(--ink-muted);font-family:var(--font-mono);">[sub-fields: {}]</span>"#,
                        snames.join(", ")
                    )
                } else if f.field_type == "post_object" {
                    format!(
                        r#" <span style="font-size:11px;color:var(--ink-muted);font-family:var(--font-mono);">[target: {}]</span>"#,
                        escape_html(f.target_post_type.as_deref().unwrap_or("any"))
                    )
                } else {
                    String::new()
                };
                field_rows.push_str(&format!(
                    r#"<tr>
                        <td style="font-weight:500;color:var(--ink);">{flabel}{sub_info}</td>
                        <td><code class="cm-code">{fname}</code></td>
                        <td><span class="cm-mono-pill">{ftype}</span> {card_badge}</td>
                        <td>
                            <button type="button" class="cm-token-copy" onclick="copyAcfToken('{{{{acf.{fname}}}}}')" title="Click to copy dynamic token">{{{{acf.{fname}}}}}</button>
                        </td>
                    </tr>"#
                ));
            }
            let fg_json = escape_html(&serde_json::to_string(fg).unwrap_or_else(|_| "{}".into()));
            fg_cards.push_str(&format!(
                r#"<div class="day-card" style="margin-bottom:28px;">
                    <div class="day-card-header">
                        <div>
                            <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap;">
                                <h3 class="day-card-title">{title_esc}</h3>
                                <span class="cm-mono-pill">{loc_esc}</span>
                                <code style="font-family:var(--font-mono);font-size:11px;color:var(--ink-muted);">{gkey_esc}</code>
                            </div>
                            <div style="font-size:13px;color:var(--ink-muted);margin-top:4px;">{desc_esc}</div>
                        </div>
                        <div style="display:flex;gap:8px;align-items:center;">
                            <button type="button" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;" data-fg="{fg_json}" onclick="editFieldGroupFromBtn(this)">Edit Schema</button>
                            <button type="button" class="btn btn-danger" style="padding:4px 10px;font-size:12px;" onclick="deleteFieldGroup('{gkey_esc}')">Delete</button>
                        </div>
                    </div>
                    <div class="day-table-wrapper">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Field Label</th>
                                    <th>Field Name (Key)</th>
                                    <th>Type</th>
                                    <th>DayStudio / Theme Template Token</th>
                                </tr>
                            </thead>
                            <tbody>
                                {field_rows}
                            </tbody>
                        </table>
                    </div>
                </div>"#
            ));
        }
    }

    // Post Type options for Location Rule selector
    let mut pt_options_html =
        r#"<option value="post">Post (post)</option><option value="page">Page (page)</option>"#
            .to_string();
    for pt in post_types {
        pt_options_html.push_str(&format!(
            r#"<option value="{0}">{1} ({0})</option>"#,
            escape_html(&pt.slug),
            escape_html(&pt.label_plural)
        ));
    }

    let body = format!(
        r##"
<style>
    .cm-surface {{
        background: var(--paper-raised);
        border: 1px solid var(--rule);
        border-radius: var(--radius-sm);
        padding: 18px;
    }}
    .cm-presets-grid {{
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
        gap: 12px;
        margin-bottom: 28px;
    }}
    .cm-preset-card {{
        background: var(--paper-raised);
        border: 1px solid var(--rule);
        border-top: 2px solid var(--rule-strong);
        border-radius: var(--radius-sm);
        padding: 14px;
        display: flex;
        flex-direction: column;
    }}
    .cm-tabs {{
        display: flex;
        gap: 6px;
        border-bottom: 1px solid var(--rule-strong);
        margin-bottom: 20px;
    }}
    .cm-tab-btn {{
        background: transparent;
        border: none;
        border-bottom: 2px solid transparent;
        padding: 9px 14px;
        font-family: var(--font-sans);
        font-size: 13.5px;
        font-weight: 600;
        color: var(--ink-muted);
        cursor: pointer;
    }}
    .cm-tab-btn.is-active {{
        color: var(--ink);
        border-bottom-color: var(--oxide);
    }}
    .cm-code {{
        font-family: var(--font-mono);
        font-size: 12px;
        background: var(--paper-sunk);
        padding: 2px 6px;
        border-radius: var(--radius-sm);
        color: var(--ink);
    }}
    .cm-mono-pill {{
        display: inline-block;
        font-family: var(--font-mono);
        font-size: 11px;
        background: var(--paper-sunk);
        border: 1px solid var(--rule);
        color: var(--ink);
        padding: 2px 7px;
        border-radius: var(--radius-sm);
        text-decoration: none;
    }}
    .cm-count-badge {{
        font-family: var(--font-mono);
        font-size: 12px;
        font-weight: 600;
        color: var(--ink);
    }}
    .cm-token-copy {{
        font-family: var(--font-mono);
        font-size: 11.5px;
        background: var(--paper-sunk);
        border: 1px solid var(--rule);
        color: var(--press);
        padding: 2px 8px;
        border-radius: var(--radius-sm);
        cursor: pointer;
    }}
    .cm-modal-backdrop {{
        display: none;
        position: fixed;
        inset: 0;
        background: #16171a73;
        z-index: 9999;
        align-items: center;
        justify-content: center;
        padding: 24px;
    }}
    .cm-modal-dialog {{
        background: var(--paper-raised);
        border: 1px solid var(--rule-strong);
        border-radius: var(--radius-md);
        width: 100%;
        max-width: 780px;
        max-height: 90vh;
        overflow-y: auto;
        padding: 24px;
    }}
    .cm-form-grid {{
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 12px;
        margin-bottom: 12px;
    }}
    .cm-field-builder-row {{
        background: var(--paper);
        border: 1px solid var(--rule);
        border-radius: var(--radius-sm);
        padding: 12px;
        margin-bottom: 10px;
    }}
</style>

<div class="page-header">
    <div>
        <h1 class="page-title">Content Model &amp; Structured Fields</h1>
        <p class="page-subtitle">Register Custom Post Types, Custom Taxonomies &amp; ACF-compatible Structured Field Schemas in SQLite</p>
    </div>
    <div style="display:flex;gap:8px;flex-wrap:wrap;">
        <button type="button" class="btn btn-secondary" onclick="openCptModal()">+ New Post Type</button>
        <button type="button" class="btn btn-secondary" onclick="openTaxModal()">+ New Taxonomy</button>
        <button type="button" class="btn btn-primary" onclick="openFieldGroupModal()">+ New Field Group</button>
    </div>
</div>

<!-- 1-Click Starter Blueprints -->
<div style="margin-bottom:10px;display:flex;align-items:center;justify-content:space-between;">
    <span class="day-kicker">1-Click Starter Blueprints (CPT + Taxonomy + Structured Field Group)</span>
</div>
<div class="cm-presets-grid">
    {presets_html}
</div>

<!-- Navigation Tabs -->
<div class="cm-tabs" role="tablist">
    <button type="button" class="cm-tab-btn {tab_pt_cls}" data-cm-tab="post-types" onclick="switchCmTab('post-types')">Post Types ({pt_len})</button>
    <button type="button" class="cm-tab-btn {tab_tx_cls}" data-cm-tab="taxonomies" onclick="switchCmTab('taxonomies')">Custom Taxonomies ({tx_len})</button>
    <button type="button" class="cm-tab-btn {tab_fg_cls}" data-cm-tab="fields" onclick="switchCmTab('fields')">Structured Field Groups ({fg_len})</button>
</div>

<!-- TAB 1: Post Types -->
<div id="cm-panel-post-types" style="display:{panel_pt_disp};">
    <div class="day-table-wrapper">
        <table class="day-table">
            <thead>
                <tr>
                    <th>Post Type</th>
                    <th>Slug</th>
                    <th>Archive Permalink</th>
                    <th>Taxonomies</th>
                    <th style="width:90px;">Entries</th>
                    <th style="text-align:right;width:280px;">Actions</th>
                </tr>
            </thead>
            <tbody>
                {pt_rows}
            </tbody>
        </table>
    </div>
</div>

<!-- TAB 2: Custom Taxonomies -->
<div id="cm-panel-taxonomies" style="display:{panel_tx_disp};">
    <div class="day-table-wrapper">
        <table class="day-table">
            <thead>
                <tr>
                    <th>Taxonomy</th>
                    <th>Slug</th>
                    <th>Hierarchy</th>
                    <th>Post Types</th>
                    <th style="width:90px;">Terms</th>
                    <th style="text-align:right;width:260px;">Actions</th>
                </tr>
            </thead>
            <tbody>
                {tax_rows}
            </tbody>
        </table>
    </div>
</div>

<!-- TAB 3: Structured Field Groups -->
<div id="cm-panel-fields" style="display:{panel_fg_disp};">
    {fg_cards}
</div>

<!-- MODAL: Create / Edit Post Type -->
<div id="cm-cpt-modal" class="cm-modal-backdrop" onclick="if(event.target===this)closeCptModal()">
    <div class="cm-modal-dialog">
        <h2 id="cm-cpt-modal-title" class="day-card-title" style="margin:0 0 16px;">Register Custom Post Type</h2>
        <div class="cm-form-grid">
            <div>
                <label class="form-label">Slug (lowercase, e.g. <code>case_study</code>)</label>
                <input type="text" id="cpt-slug" class="form-input" placeholder="case_study" />
            </div>
            <div>
                <label class="form-label">Archive Route Slug</label>
                <input type="text" id="cpt-rewrite" class="form-input" placeholder="case_study" />
            </div>
            <div>
                <label class="form-label">Plural Label (e.g. <em>Case Studies</em>)</label>
                <input type="text" id="cpt-plural" class="form-input" placeholder="Case Studies" />
            </div>
            <div>
                <label class="form-label">Singular Label (e.g. <em>Case Study</em>)</label>
                <input type="text" id="cpt-singular" class="form-input" placeholder="Case Study" />
            </div>
            <div>
                <label class="form-label">Attached Taxonomies (comma-separated slugs)</label>
                <input type="text" id="cpt-taxonomies" class="form-input" placeholder="case_industry, post_tag" />
            </div>
            <div>
                <label class="form-label">Icon Class</label>
                <input type="text" id="cpt-icon" class="form-input" value="fa-solid fa-folder-open" />
            </div>
        </div>
        <div style="margin-bottom:14px;">
            <label class="form-label">Description</label>
            <input type="text" id="cpt-desc" class="form-input" placeholder="Short description of this post type..." />
        </div>
        <div style="display:flex;gap:18px;margin-bottom:18px;flex-wrap:wrap;font-size:13.5px;">
            <label style="display:inline-flex;align-items:center;gap:6px;"><input type="checkbox" id="cpt-has-archive" checked /> Enable Public Archive (<code>/{{slug}}/</code>)</label>
            <label style="display:inline-flex;align-items:center;gap:6px;"><input type="checkbox" id="cpt-hierarchical" /> Hierarchical (Parent/Child)</label>
            <label style="display:inline-flex;align-items:center;gap:6px;"><input type="checkbox" id="cpt-rest" checked /> Expose in REST API &amp; MCP</label>
        </div>
        <div style="display:flex;justify-content:flex-end;gap:8px;padding-top:12px;border-top:1px solid var(--rule);">
            <button type="button" class="btn btn-secondary" onclick="closeCptModal()">Cancel</button>
            <button type="button" class="btn btn-primary" onclick="saveCptModal()">Save Post Type</button>
        </div>
    </div>
</div>

<!-- MODAL: Create / Edit Custom Taxonomy -->
<div id="cm-tax-modal" class="cm-modal-backdrop" onclick="if(event.target===this)closeTaxModal()">
    <div class="cm-modal-dialog">
        <h2 id="cm-tax-modal-title" class="day-card-title" style="margin:0 0 16px;">Register Custom Taxonomy</h2>
        <div class="cm-form-grid">
            <div>
                <label class="form-label">Taxonomy Slug (e.g. <code>case_industry</code>)</label>
                <input type="text" id="tax-slug" class="form-input" placeholder="case_industry" />
            </div>
            <div>
                <label class="form-label">Attached Post Types (comma-separated slugs)</label>
                <input type="text" id="tax-post-types" class="form-input" placeholder="case_study" />
            </div>
            <div>
                <label class="form-label">Plural Label (e.g. <em>Industries</em>)</label>
                <input type="text" id="tax-plural" class="form-input" placeholder="Industries" />
            </div>
            <div>
                <label class="form-label">Singular Label (e.g. <em>Industry</em>)</label>
                <input type="text" id="tax-singular" class="form-input" placeholder="Industry" />
            </div>
        </div>
        <div style="margin-bottom:14px;">
            <label class="form-label">Description</label>
            <input type="text" id="tax-desc" class="form-input" />
        </div>
        <div style="display:flex;gap:18px;margin-bottom:18px;font-size:13.5px;">
            <label style="display:inline-flex;align-items:center;gap:6px;"><input type="checkbox" id="tax-hierarchical" checked /> Hierarchical (Category-style checkboxes)</label>
        </div>
        <div style="display:flex;justify-content:flex-end;gap:8px;padding-top:12px;border-top:1px solid var(--rule);">
            <button type="button" class="btn btn-secondary" onclick="closeTaxModal()">Cancel</button>
            <button type="button" class="btn btn-primary" onclick="saveTaxModal()">Save Taxonomy</button>
        </div>
    </div>
</div>

<!-- MODAL: Create / Edit Structured Field Group -->
<div id="cm-fg-modal" class="cm-modal-backdrop" onclick="if(event.target===this)closeFieldGroupModal()">
    <div class="cm-modal-dialog" style="max-width:860px;">
        <h2 id="cm-fg-modal-title" class="day-card-title" style="margin:0 0 16px;">Structured Field Group Schema</h2>
        <input type="hidden" id="fg-id" value="0" />
        <input type="hidden" id="fg-key" value="" />
        <div class="cm-form-grid">
            <div>
                <label class="form-label">Field Group Title</label>
                <input type="text" id="fg-title" class="form-input" placeholder="Case Study Specifications" />
            </div>
            <div>
                <label class="form-label">Assign to Post Type (Location Rule)</label>
                <select id="fg-post-type" class="form-select">
                    {pt_options_html}
                </select>
            </div>
        </div>
        <div style="margin-bottom:14px;">
            <label class="form-label">Description</label>
            <input type="text" id="fg-desc" class="form-input" placeholder="Displayed above fields in the Post Editor..." />
        </div>

        <div style="display:flex;align-items:center;justify-content:space-between;margin:16px 0 10px;">
            <span class="day-kicker">Structured Fields</span>
            <button type="button" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;" onclick="addFgFieldRow()">+ Add Field</button>
        </div>
        <div id="fg-fields-container"></div>

        <div style="display:flex;justify-content:flex-end;gap:8px;margin-top:18px;padding-top:12px;border-top:1px solid var(--rule);">
            <button type="button" class="btn btn-secondary" onclick="closeFieldGroupModal()">Cancel</button>
            <button type="button" class="btn btn-primary" onclick="saveFieldGroupModal()">Save Field Group</button>
        </div>
    </div>
</div>

<script>
    function switchCmTab(tab) {{
        document.querySelectorAll('.cm-tab-btn').forEach(b => b.classList.toggle('is-active', b.dataset.cmTab === tab));
        document.getElementById('cm-panel-post-types').style.display = tab === 'post-types' ? 'block' : 'none';
        document.getElementById('cm-panel-taxonomies').style.display = tab === 'taxonomies' ? 'block' : 'none';
        document.getElementById('cm-panel-fields').style.display = tab === 'fields' ? 'block' : 'none';
        const url = new URL(window.location.href);
        url.searchParams.set('tab', tab);
        window.history.replaceState({{}}, '', url);
    }}

    function copyAcfToken(token) {{
        if (navigator.clipboard) navigator.clipboard.writeText(token);
        showToast('Copied token ' + token);
    }}

    async function provisionPreset(presetKey, btn) {{
        const origText = btn.textContent;
        btn.disabled = true;
        btn.textContent = 'Provisioning...';
        try {{
            const res = await fetch('/wp-json/daypress/v1/content-types/preset', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ preset_key: presetKey }})
            }});
            const data = await res.json();
            if (!res.ok) throw new Error(data.message || 'Failed to provision preset');
            showToast('Provisioned ' + (data.post_type?.label_plural || presetKey) + '!');
            setTimeout(() => window.location.reload(), 400);
        }} catch (err) {{
            showToast(err.message, true);
            btn.disabled = false;
            btn.textContent = origText;
        }}
    }}

    function openCptModal(cpt) {{
        document.getElementById('cpt-slug').value = cpt ? cpt.slug : '';
        document.getElementById('cpt-rewrite').value = cpt ? (cpt.rewrite_slug || cpt.slug) : '';
        document.getElementById('cpt-plural').value = cpt ? cpt.label_plural : '';
        document.getElementById('cpt-singular').value = cpt ? cpt.label_singular : '';
        document.getElementById('cpt-taxonomies').value = cpt && Array.isArray(cpt.taxonomies) ? cpt.taxonomies.join(', ') : '';
        document.getElementById('cpt-icon').value = cpt ? (cpt.icon || 'fa-solid fa-folder-open') : 'fa-solid fa-folder-open';
        document.getElementById('cpt-desc').value = cpt ? (cpt.description || '') : '';
        document.getElementById('cpt-has-archive').checked = cpt ? !!cpt.has_archive : true;
        document.getElementById('cpt-hierarchical').checked = cpt ? !!cpt.hierarchical : false;
        document.getElementById('cpt-rest').checked = cpt ? !!cpt.show_in_rest : true;
        document.getElementById('cm-cpt-modal').style.display = 'flex';
    }}
    function closeCptModal() {{ document.getElementById('cm-cpt-modal').style.display = 'none'; }}
    function editCptFromBtn(btn) {{ openCptModal(JSON.parse(btn.dataset.cpt)); }}

    async function saveCptModal() {{
        const slug = document.getElementById('cpt-slug').value.trim();
        const label_plural = document.getElementById('cpt-plural').value.trim();
        const label_singular = document.getElementById('cpt-singular').value.trim();
        if (!slug) {{ showToast('Post type slug is required', true); return; }}
        const taxonomies = document.getElementById('cpt-taxonomies').value
            .split(',').map(s => s.trim()).filter(Boolean);
        const payload = {{
            slug,
            rewrite_slug: document.getElementById('cpt-rewrite').value.trim() || slug,
            label_plural: label_plural || slug,
            label_singular: label_singular || label_plural || slug,
            description: document.getElementById('cpt-desc').value.trim(),
            icon: document.getElementById('cpt-icon').value.trim() || 'fa-solid fa-folder-open',
            has_archive: document.getElementById('cpt-has-archive').checked,
            hierarchical: document.getElementById('cpt-hierarchical').checked,
            show_in_rest: document.getElementById('cpt-rest').checked,
            taxonomies
        }};
        try {{
            const res = await fetch('/wp-json/daypress/v1/content-types', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify(payload)
            }});
            const data = await res.json();
            if (!res.ok) throw new Error(data.message || 'Failed to save post type');
            showToast('Saved post type ' + data.label_plural);
            window.location.reload();
        }} catch (err) {{
            showToast(err.message, true);
        }}
    }}

    async function deleteCpt(slug) {{
        if (!confirm('Remove custom post type "' + slug + '" from the registry?')) return;
        const res = await fetch('/wp-json/daypress/v1/content-types/' + encodeURIComponent(slug), {{ method: 'DELETE' }});
        if (res.ok) {{ showToast('Deleted ' + slug); window.location.reload(); }}
        else {{ showToast('Failed to delete ' + slug, true); }}
    }}

    function openTaxModal(tx) {{
        document.getElementById('tax-slug').value = tx ? tx.slug : '';
        document.getElementById('tax-post-types').value = tx && Array.isArray(tx.post_types) ? tx.post_types.join(', ') : '';
        document.getElementById('tax-plural').value = tx ? tx.label_plural : '';
        document.getElementById('tax-singular').value = tx ? tx.label_singular : '';
        document.getElementById('tax-desc').value = tx ? (tx.description || '') : '';
        document.getElementById('tax-hierarchical').checked = tx ? !!tx.hierarchical : true;
        document.getElementById('cm-tax-modal').style.display = 'flex';
    }}
    function closeTaxModal() {{ document.getElementById('cm-tax-modal').style.display = 'none'; }}
    function editTaxFromBtn(btn) {{ openTaxModal(JSON.parse(btn.dataset.tax)); }}

    async function saveTaxModal() {{
        const slug = document.getElementById('tax-slug').value.trim();
        if (!slug) {{ showToast('Taxonomy slug is required', true); return; }}
        const post_types = document.getElementById('tax-post-types').value
            .split(',').map(s => s.trim()).filter(Boolean);
        const payload = {{
            slug,
            rewrite_slug: slug,
            label_plural: document.getElementById('tax-plural').value.trim() || slug,
            label_singular: document.getElementById('tax-singular').value.trim() || slug,
            description: document.getElementById('tax-desc').value.trim(),
            hierarchical: document.getElementById('tax-hierarchical').checked,
            post_types
        }};
        try {{
            const res = await fetch('/wp-json/daypress/v1/custom-taxonomies', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify(payload)
            }});
            const data = await res.json();
            if (!res.ok) throw new Error(data.message || 'Failed to save taxonomy');
            showToast('Saved taxonomy ' + data.label_plural);
            window.location.href = '/d-admin/content-types?tab=taxonomies';
        }} catch (err) {{
            showToast(err.message, true);
        }}
    }}

    async function deleteTax(slug) {{
        if (!confirm('Remove custom taxonomy "' + slug + '"?')) return;
        const res = await fetch('/wp-json/daypress/v1/custom-taxonomies/' + encodeURIComponent(slug), {{ method: 'DELETE' }});
        if (res.ok) {{ showToast('Deleted ' + slug); window.location.reload(); }}
        else {{ showToast('Failed to delete ' + slug, true); }}
    }}

    function addFgFieldRow(field = {{}}) {{
        const container = document.getElementById('fg-fields-container');
        const row = document.createElement('div');
        row.className = 'cm-field-builder-row';
        const ftype = field.field_type || 'text';
        const choicesStr = Array.isArray(field.choices) ? field.choices.join(', ') : '';
        const subsStr = Array.isArray(field.sub_fields)
            ? field.sub_fields.map(sf => sf.name + ':' + (sf.field_type || 'text')).join(', ')
            : '';
        row.innerHTML = `
            <div style="display:grid;grid-template-columns:1.3fr 1.1fr 1fr auto;gap:8px;align-items:end;">
                <div>
                    <label class="form-label" style="font-size:11.5px;">Label</label>
                    <input type="text" class="form-input fg-f-label" placeholder="Client Name" value="${{(field.label || '').replace(/"/g, '&quot;')}}" />
                </div>
                <div>
                    <label class="form-label" style="font-size:11.5px;">Field Name (Key)</label>
                    <input type="text" class="form-input fg-f-name" style="font-family:var(--font-mono);" placeholder="client_name" value="${{(field.name || '').replace(/"/g, '&quot;')}}" />
                </div>
                <div>
                    <label class="form-label" style="font-size:11.5px;">Field Type</label>
                    <select class="form-select fg-f-type">
                        <option value="text" ${{ftype==='text'?'selected':''}}>Text</option>
                        <option value="textarea" ${{ftype==='textarea'?'selected':''}}>Textarea</option>
                        <option value="wysiwyg" ${{ftype==='wysiwyg'?'selected':''}}>Rich Text (WYSIWYG)</option>
                        <option value="number" ${{ftype==='number'?'selected':''}}>Number</option>
                        <option value="date" ${{ftype==='date'?'selected':''}}>Date</option>
                        <option value="url" ${{ftype==='url'?'selected':''}}>URL</option>
                        <option value="media" ${{ftype==='media'?'selected':''}}>Media / Image</option>
                        <option value="select" ${{ftype==='select'?'selected':''}}>Select Dropdown</option>
                        <option value="checkbox" ${{ftype==='checkbox'?'selected':''}}>Checkboxes</option>
                        <option value="true_false" ${{ftype==='true_false'?'selected':''}}>True / False Toggle</option>
                        <option value="repeater" ${{ftype==='repeater'?'selected':''}}>Repeater (Nested Rows)</option>
                        <option value="post_object" ${{ftype==='post_object'?'selected':''}}>Post Relation (Post Object)</option>
                    </select>
                </div>
                <div>
                    <button type="button" class="btn btn-danger" style="padding:6px 10px;font-size:12px;" onclick="this.closest('.cm-field-builder-row').remove()">✕</button>
                </div>
            </div>
            <div style="display:grid;grid-template-columns:1fr 1fr auto;gap:8px;margin-top:8px;align-items:center;">
                <input type="text" class="form-input fg-f-choices" style="font-size:12px;" placeholder="Choices (comma-separated for select/checkbox) or target CPT for post_object" value="${{(choicesStr || field.target_post_type || '').replace(/"/g, '&quot;')}}" />
                <input type="text" class="form-input fg-f-subs" style="font-size:12px;" placeholder="Repeater sub-fields (e.g. metric_value:text, metric_label:text)" value="${{subsStr.replace(/"/g, '&quot;')}}" />
                <label style="font-size:12px;white-space:nowrap;display:inline-flex;align-items:center;gap:6px;"><input type="checkbox" class="fg-f-card" ${{field.show_in_collection_card ? 'checked' : ''}} /> Show on Grid Cards</label>
            </div>
        `;
        container.appendChild(row);
    }}

    function openFieldGroupModal(fg) {{
        document.getElementById('fg-id').value = fg ? (fg.id || 0) : 0;
        document.getElementById('fg-key').value = fg ? (fg.group_key || '') : '';
        document.getElementById('fg-title').value = fg ? (fg.title || '') : '';
        document.getElementById('fg-desc').value = fg ? (fg.description || '') : '';
        const ptSelect = document.getElementById('fg-post-type');
        const locPt = fg && fg.location_rules && fg.location_rules[0] ? fg.location_rules[0].value : 'post';
        if (locPt && ![...ptSelect.options].some(o => o.value === locPt)) {{
            const opt = document.createElement('option');
            opt.value = locPt;
            opt.textContent = locPt;
            ptSelect.appendChild(opt);
        }}
        ptSelect.value = locPt;
        const container = document.getElementById('fg-fields-container');
        container.innerHTML = '';
        if (fg && Array.isArray(fg.fields) && fg.fields.length) {{
            fg.fields.forEach(f => addFgFieldRow(f));
        }} else {{
            addFgFieldRow({{ label: 'Subtitle', name: 'subtitle', field_type: 'text', show_in_collection_card: true }});
        }}
        document.getElementById('cm-fg-modal').style.display = 'flex';
    }}
    function closeFieldGroupModal() {{ document.getElementById('cm-fg-modal').style.display = 'none'; }}
    function editFieldGroupFromBtn(btn) {{ openFieldGroupModal(JSON.parse(btn.dataset.fg)); }}

    async function saveFieldGroupModal() {{
        const title = document.getElementById('fg-title').value.trim();
        if (!title) {{ showToast('Field group title is required', true); return; }}
        const targetPt = document.getElementById('fg-post-type').value;
        const fields = [];
        document.querySelectorAll('#fg-fields-container .cm-field-builder-row').forEach(row => {{
            const label = row.querySelector('.fg-f-label').value.trim();
            const name = row.querySelector('.fg-f-name').value.trim() || label.toLowerCase().replace(/[^a-z0-9]+/g, '_');
            if (!name) return;
            const field_type = row.querySelector('.fg-f-type').value;
            const rawExtra = row.querySelector('.fg-f-choices').value.trim();
            const rawSubs = row.querySelector('.fg-f-subs').value.trim();
            const show_in_collection_card = row.querySelector('.fg-f-card').checked;
            const choices = (field_type === 'select' || field_type === 'checkbox') && rawExtra
                ? rawExtra.split(',').map(s => s.trim()).filter(Boolean)
                : [];
            const target_post_type = field_type === 'post_object' && rawExtra ? rawExtra : null;
            const sub_fields = field_type === 'repeater' && rawSubs
                ? rawSubs.split(',').map(pair => {{
                    const [sname, stype] = pair.split(':').map(s => s.trim());
                    return {{ name: sname, label: sname, field_type: stype || 'text' }};
                }}).filter(sf => sf.name)
                : [];
            fields.push({{
                label: label || name,
                name,
                field_type,
                choices,
                target_post_type,
                sub_fields,
                show_in_collection_card
            }});
        }});

        const payload = {{
            id: parseInt(document.getElementById('fg-id').value || '0', 10),
            group_key: document.getElementById('fg-key').value.trim(),
            title,
            description: document.getElementById('fg-desc').value.trim(),
            location_rules: [{{ param: 'post_type', operator: '==', value: targetPt }}],
            fields,
            is_active: true
        }};
        try {{
            const res = await fetch('/wp-json/daypress/v1/field-groups', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify(payload)
            }});
            const data = await res.json();
            if (!res.ok) throw new Error(data.message || 'Failed to save field group');
            showToast('Saved field group ' + data.title);
            window.location.href = '/d-admin/content-types?tab=fields';
        }} catch (err) {{
            showToast(err.message, true);
        }}
    }}

    async function deleteFieldGroup(key) {{
        if (!confirm('Delete field group "' + key + '"?')) return;
        const res = await fetch('/wp-json/daypress/v1/field-groups/' + encodeURIComponent(key), {{ method: 'DELETE' }});
        if (res.ok) {{ showToast('Deleted field group'); window.location.reload(); }}
        else {{ showToast('Failed to delete field group', true); }}
    }}
</script>
"##,
        presets_html = presets_html,
        pt_len = post_types.len(),
        tx_len = taxonomies.len(),
        fg_len = field_groups.len(),
        tab_pt_cls = if tab == "post-types" { "is-active" } else { "" },
        tab_tx_cls = if tab == "taxonomies" { "is-active" } else { "" },
        tab_fg_cls = if tab == "fields" { "is-active" } else { "" },
        panel_pt_disp = if tab == "post-types" { "block" } else { "none" },
        panel_tx_disp = if tab == "taxonomies" { "block" } else { "none" },
        panel_fg_disp = if tab == "fields" { "block" } else { "none" },
        pt_rows = pt_rows,
        tax_rows = tax_rows,
        fg_cards = fg_cards,
        pt_options_html = pt_options_html,
    );

    render_admin_shell("content-types", "Content Model", &body, settings)
}
