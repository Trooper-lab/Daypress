//! Custom ACF and Structured Field Groups card renderer (`markup/meta.rs`).

use crate::models::{DaypressPostResponse, FieldGroupDefinition, StructuredFieldSchema};
use crate::templates::utils::escape_html;

pub fn get_meta_value(existing: Option<&DaypressPostResponse>, key: &str) -> String {
    if let Some(post) = existing {
        if let Some(acf_obj) = post.acf.as_object() {
            if let Some(val) = acf_obj.get(key) {
                match val {
                    serde_json::Value::String(s) => return escape_html(s),
                    serde_json::Value::Number(n) => return escape_html(&n.to_string()),
                    serde_json::Value::Bool(b) => return if *b { "1" } else { "0" }.to_string(),
                    serde_json::Value::Object(obj) => {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_i64()) {
                            return id.to_string();
                        }
                        if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
                            return escape_html(url);
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(m) = post.meta.as_object() {
            if let Some(v) = m.get(key) {
                if let Some(s) = v.as_str() {
                    return escape_html(s);
                }
                if !v.is_null() {
                    return escape_html(&v.to_string());
                }
            }
        }
    }
    String::new()
}

/// Meta keys edited by the ACF header settings card below.
const ACF_CARD_META_KEYS: [&str; 7] = [
    "header_height",
    "header_hoogte",
    "header_title",
    "show_title",
    "header_shape",
    "show_icon",
    "header_left",
];

/// Whether a post carries ACF block/header data: ACF blocks in its content, ACF field references
/// (`_field` meta values like `field_6123abc`), or the header meta keys this card edits.
pub fn post_uses_acf(existing: Option<&DaypressPostResponse>) -> bool {
    let Some(post) = existing else {
        return false;
    };
    if post.content.rendered.contains("<!-- wp:acf/") {
        return true;
    }
    post.meta.as_object().is_some_and(|m| {
        m.iter().any(|(k, v)| {
            ACF_CARD_META_KEYS.contains(&k.as_str())
                || v.as_str().is_some_and(|s| s.starts_with("field_"))
        })
    })
}

pub fn render_acf_fields_card(existing: Option<&DaypressPostResponse>, _post_type: &str) -> String {
    let get_meta = |key: &str| get_meta_value(existing, key);

    let header_height = {
        let v = get_meta("header_height");
        if v.is_empty() {
            get_meta("header_hoogte")
        } else {
            v
        }
    };
    let hh_full_sel = if header_height == "full" {
        "selected"
    } else {
        ""
    };
    let hh_auto_sel = if header_height != "full" {
        "selected"
    } else {
        ""
    };
    let show_title = {
        let v = get_meta("show_title");
        if v.is_empty() {
            get_meta("header_title")
        } else {
            v
        }
    };
    let st_show_sel = if show_title != "hide" { "selected" } else { "" };
    let st_hide_sel = if show_title == "hide" { "selected" } else { "" };
    let show_icon = {
        let v = get_meta("show_icon");
        if v.is_empty() {
            get_meta("header_shape")
        } else {
            v
        }
    };
    let si_show_sel = if show_icon != "hide" { "selected" } else { "" };
    let si_hide_sel = if show_icon == "hide" { "selected" } else { "" };
    let header_left = get_meta("header_left");

    format!(
        r#"<div class="day-card" style="padding: 22px;">
            <div class="day-card-header" style="padding: 0 0 14px 0; margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h2 class="day-card-title">Theme Header &amp; ACF Page Settings</h2>
                    <span class="badge badge-published" style="font-size: 11px;">ACF / Gutenberg Hybrid</span>
                </div>
                <span style="font-size: 11.5px; color: var(--ink-muted); font-family: var(--font-mono);">postmeta synced</span>
            </div>

            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px;">
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">Header banner height (<code>header_height</code>)</label>
                    <select class="form-select acf-meta-input" data-acf-key="header_height" onchange="onAcfFieldChange()">
                        <option value="auto" {hh_auto_sel}>Compact (Auto Height)</option>
                        <option value="full" {hh_full_sel}>Full Height Hero Banner</option>
                    </select>
                </div>
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">Show page title in header (<code>show_title</code>)</label>
                    <select class="form-select acf-meta-input" data-acf-key="show_title" onchange="onAcfFieldChange()">
                        <option value="show" {st_show_sel}>Show title</option>
                        <option value="hide" {st_hide_sel}>Hide title</option>
                    </select>
                </div>
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">Header decoration shape (<code>show_icon</code>)</label>
                    <select class="form-select acf-meta-input" data-acf-key="show_icon" onchange="onAcfFieldChange()">
                        <option value="show" {si_show_sel}>Show splash shape</option>
                        <option value="hide" {si_hide_sel}>Hide splash shape</option>
                    </select>
                </div>
            </div>
            <div class="form-group" style="margin-top: 12px; margin-bottom: 0;">
                <label class="form-label">Hero Banner Intro / Left Copy (<code>header_left</code>)</label>
                <textarea rows="2" class="form-textarea acf-meta-input" data-acf-key="header_left" placeholder="Optional hero subtitle / intro paragraph..." oninput="onAcfFieldChange()">{header_left}</textarea>
            </div>
        </div>"#
    )
}

/// Renders all active `FieldGroupDefinition` cards matching the current post type.
pub fn render_structured_field_groups(
    existing: Option<&DaypressPostResponse>,
    field_groups: &[FieldGroupDefinition],
    all_posts: &[DaypressPostResponse],
) -> String {
    if field_groups.is_empty() {
        return String::new();
    }

    let mut html = String::new();
    for group in field_groups
        .iter()
        .filter(|g| g.is_active && !g.fields.is_empty())
    {
        let group_title = escape_html(&group.title);
        let group_key = escape_html(&group.group_key);

        let mut fields_html = String::new();
        for field in &group.fields {
            fields_html.push_str(&render_single_structured_field(existing, field, all_posts));
        }

        html.push_str(&format!(
            r#"<div class="day-card dp-structured-field-group" data-field-group-key="{group_key}" style="padding: 22px;">
                <div class="day-card-header" style="padding: 0 0 14px 0; margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;">
                    <div style="display: flex; align-items: center; gap: 10px;">
                        <h2 class="day-card-title">{group_title}</h2>
                        <span class="badge badge-published" style="font-size: 11px;">Structured Fields</span>
                    </div>
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <span style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono);">{group_key}</span>
                        <a href="/d-admin/content-types?tab=fields" style="font-size: 11.5px; color: var(--press); text-decoration: none; font-weight: 600;">Schema &nearr;</a>
                    </div>
                </div>
                <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 16px;">
                    {fields_html}
                </div>
            </div>"#
        ));
    }

    html
}

fn render_single_structured_field(
    existing: Option<&DaypressPostResponse>,
    field: &StructuredFieldSchema,
    all_posts: &[DaypressPostResponse],
) -> String {
    let f_name = escape_html(&field.name);
    let f_label = escape_html(&field.label);
    let f_placeholder = escape_html(if field.placeholder.is_empty() {
        &field.default_value
    } else {
        &field.placeholder
    });
    let f_instructions = if !field.instructions.is_empty() {
        format!(
            r#"<small style="display:block;color:var(--ink-muted);font-size:11.5px;margin-top:4px;">{}</small>"#,
            escape_html(&field.instructions)
        )
    } else {
        String::new()
    };
    let req_badge = if field.required {
        r#" <span style="color:var(--oxide);">*</span>"#
    } else {
        ""
    };
    let card_badge = if field.show_in_collection_card {
        r#" <span class="badge" style="font-size:9.5px;background:var(--paper-sunk);color:var(--ink-muted);margin-left:6px;" title="Displayed on Collection Grid cards">Card Badge</span>"#
    } else {
        ""
    };

    let current_val = {
        let v = get_meta_value(existing, &field.name);
        if v.is_empty() && existing.is_none() {
            escape_html(&field.default_value)
        } else {
            v
        }
    };

    let is_full_width = matches!(
        field.field_type.as_str(),
        "textarea" | "wysiwyg" | "repeater" | "checkbox"
    );
    let col_span = if is_full_width {
        "grid-column: 1 / -1;"
    } else {
        ""
    };

    let control_html = match field.field_type.as_str() {
        "textarea" => format!(
            r#"<textarea rows="3" class="form-textarea acf-meta-input" data-acf-key="{f_name}" placeholder="{f_placeholder}" oninput="onAcfFieldChange()">{current_val}</textarea>"#
        ),
        "wysiwyg" => format!(
            r#"<textarea rows="5" class="form-textarea acf-meta-input" data-acf-key="{f_name}" placeholder="{f_placeholder}" style="font-family: var(--font-sans); line-height: 1.5;" oninput="onAcfFieldChange()">{current_val}</textarea>"#
        ),
        "number" => format!(
            r#"<input type="number" step="any" class="form-input acf-meta-input" data-acf-key="{f_name}" value="{current_val}" placeholder="{f_placeholder}" oninput="onAcfFieldChange()" />"#
        ),
        "date" => format!(
            r#"<input type="date" class="form-input acf-meta-input" data-acf-key="{f_name}" value="{current_val}" oninput="onAcfFieldChange()" />"#
        ),
        "url" => format!(
            r#"<input type="url" class="form-input acf-meta-input" data-acf-key="{f_name}" value="{current_val}" placeholder="{f_placeholder}" oninput="onAcfFieldChange()" />"#
        ),
        "true_false" => {
            let checked = if current_val == "1"
                || current_val.eq_ignore_ascii_case("true")
                || current_val.eq_ignore_ascii_case("yes")
            {
                "checked"
            } else {
                ""
            };
            format!(
                r#"<label style="display: inline-flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px; padding: 6px 0;">
                    <input type="checkbox" class="acf-meta-input" data-acf-key="{f_name}" {checked} onchange="onAcfFieldChange()" />
                    <span>Enable <code>{f_name}</code></span>
                </label>"#
            )
        }
        "select" => {
            let mut opts_html = String::from(r#"<option value="">— Select —</option>"#);
            for choice in &field.choices {
                let (c_val, c_label) = if let Some((k, v)) = choice.split_once(':') {
                    (k.trim(), v.trim())
                } else {
                    (choice.trim(), choice.trim())
                };
                let c_val_esc = escape_html(c_val);
                let c_lbl_esc = escape_html(c_label);
                let sel = if current_val == c_val_esc || current_val == c_lbl_esc {
                    "selected"
                } else {
                    ""
                };
                opts_html.push_str(&format!(
                    r#"<option value="{c_val_esc}" {sel}>{c_lbl_esc}</option>"#
                ));
            }
            format!(
                r#"<select class="form-select acf-meta-input" data-acf-key="{f_name}" onchange="onAcfFieldChange()">{opts_html}</select>"#
            )
        }
        "checkbox" => {
            let selected_items: Vec<String> = if current_val.starts_with('[') {
                serde_json::from_str::<Vec<String>>(&current_val).unwrap_or_default()
            } else {
                current_val
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            let mut checks_html = String::new();
            for choice in &field.choices {
                let (c_val, c_label) = if let Some((k, v)) = choice.split_once(':') {
                    (k.trim(), v.trim())
                } else {
                    (choice.trim(), choice.trim())
                };
                let c_val_esc = escape_html(c_val);
                let c_lbl_esc = escape_html(c_label);
                let chk = if selected_items.iter().any(|s| s == c_val || s == c_label) {
                    "checked"
                } else {
                    ""
                };
                checks_html.push_str(&format!(
                    r#"<label style="display:inline-flex;align-items:center;gap:6px;font-size:12.5px;cursor:pointer;background:var(--paper-raised);border:1px solid var(--rule);padding:4px 10px;border-radius:4px;">
                        <input type="checkbox" class="acf-checkbox-option" value="{c_val_esc}" {chk} onchange="onAcfFieldChange()" />
                        <span>{c_lbl_esc}</span>
                    </label>"#
                ));
            }
            format!(
                r#"<div class="acf-checkbox-group" data-acf-checkbox-key="{f_name}" style="display:flex;flex-wrap:wrap;gap:8px;">{checks_html}</div>"#
            )
        }
        "media" => format!(
            r#"<div style="display:flex;gap:8px;align-items:center;">
                <input type="text" id="acf_media_{f_name}" class="form-input acf-meta-input" data-acf-key="{f_name}" value="{current_val}" placeholder="Media URL or Attachment ID..." oninput="onAcfFieldChange()" style="flex:1;" />
                <button type="button" class="btn btn-secondary" onclick="openMediaPicker('acf:{f_name}')" style="height:36px;white-space:nowrap;">Library &nearr;</button>
            </div>"#
        ),
        "post_object" => {
            let mut opts_html = String::from(r#"<option value="">— None —</option>"#);
            for p in all_posts {
                let pid_str = p.id.to_string();
                let sel = if current_val == pid_str {
                    "selected"
                } else {
                    ""
                };
                let p_title = escape_html(&p.title.rendered);
                let p_type = escape_html(&p.post_type);
                opts_html.push_str(&format!(
                    r#"<option value="{pid_str}" {sel}>{p_title} ({p_type} #{pid_str})</option>"#
                ));
            }
            format!(
                r#"<select class="form-select acf-meta-input" data-acf-key="{f_name}" onchange="onAcfFieldChange()">{opts_html}</select>"#
            )
        }
        "repeater" => render_repeater_field(existing, field),
        _ => format!(
            r#"<input type="text" class="form-input acf-meta-input" data-acf-key="{f_name}" value="{current_val}" placeholder="{f_placeholder}" oninput="onAcfFieldChange()" />"#
        ),
    };

    format!(
        r#"<div class="form-group" style="margin-bottom: 0; {col_span}">
            <label class="form-label" style="display:flex;align-items:center;justify-content:space-between;">
                <span>{f_label}{req_badge}{card_badge}</span>
                <code style="font-size:10.5px;color:var(--ink-muted);font-family:var(--font-mono);">{{{{acf.{f_name}}}}}</code>
            </label>
            {control_html}
            {f_instructions}
        </div>"#
    )
}

fn render_repeater_field(
    existing: Option<&DaypressPostResponse>,
    field: &StructuredFieldSchema,
) -> String {
    let f_name = escape_html(&field.name);
    let subfields: Vec<StructuredFieldSchema> = if field.sub_fields.is_empty() {
        vec![
            StructuredFieldSchema {
                key: format!("field_{}_label", field.name),
                name: "label".to_string(),
                label: "Label".to_string(),
                field_type: "text".to_string(),
                required: false,
                default_value: String::new(),
                placeholder: String::new(),
                instructions: String::new(),
                choices: Vec::new(),
                target_post_type: None,
                sub_fields: Vec::new(),
                show_in_collection_card: false,
            },
            StructuredFieldSchema {
                key: format!("field_{}_value", field.name),
                name: "value".to_string(),
                label: "Value".to_string(),
                field_type: "text".to_string(),
                required: false,
                default_value: String::new(),
                placeholder: String::new(),
                instructions: String::new(),
                choices: Vec::new(),
                target_post_type: None,
                sub_fields: Vec::new(),
                show_in_collection_card: false,
            },
        ]
    } else {
        field.sub_fields.clone()
    };

    let subfields_json =
        escape_html(&serde_json::to_string(&subfields).unwrap_or_else(|_| "[]".to_string()));

    // Extract existing repeater rows from post.acf or post.meta
    let rows: Vec<serde_json::Map<String, serde_json::Value>> = existing
        .and_then(|post| {
            post.acf
                .as_object()
                .and_then(|acf| acf.get(&field.name))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_object().cloned())
                        .collect()
                })
        })
        .unwrap_or_default();

    let mut rows_html = String::new();
    for row in &rows {
        let mut cells_html = String::new();
        for sf in &subfields {
            let sf_name = escape_html(&sf.name);
            let sf_label = escape_html(&sf.label);
            let val = row
                .get(&sf.name)
                .and_then(|v| v.as_str())
                .map(escape_html)
                .unwrap_or_default();
            cells_html.push_str(&format!(
                r#"<div style="flex:1;min-width:140px;">
                    <label style="display:block;font-size:10.5px;color:var(--ink-muted);margin-bottom:3px;font-family:var(--font-mono);">{sf_label} ({sf_name})</label>
                    <input type="text" class="form-input acf-repeater-subinput" data-sub-key="{sf_name}" value="{val}" oninput="onAcfFieldChange()" style="height:30px;font-size:12px;" />
                </div>"#
            ));
        }
        rows_html.push_str(&format!(
            r#"<div class="acf-repeater-row" style="display:flex;gap:10px;align-items:flex-end;background:var(--paper-raised);border:1px solid var(--rule);border-radius:4px;padding:10px;margin-bottom:8px;">
                {cells_html}
                <button type="button" class="btn btn-secondary" onclick="this.closest('.acf-repeater-row').remove();onAcfFieldChange();" style="height:30px;padding:0 8px;color:var(--oxide);border-color:var(--oxide);" title="Remove Row">✕</button>
            </div>"#
        ));
    }

    format!(
        r#"<div class="acf-repeater-container" data-repeater-key="{f_name}" data-subfields="{subfields_json}" style="background:var(--paper-sunk);border:1px solid var(--rule);border-radius:4px;padding:12px;">
            <div class="acf-repeater-rows">{rows_html}</div>
            <button type="button" class="btn btn-secondary" onclick="addAcfRepeaterRow(this)" style="height:28px;font-size:11.5px;padding:0 10px;">+ Add {label} Row</button>
        </div>"#,
        f_name = f_name,
        subfields_json = subfields_json,
        rows_html = rows_html,
        label = escape_html(&field.label)
    )
}
