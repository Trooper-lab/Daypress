//! Dynamic Custom Fields (`wp_postmeta`) card renderer (`markup/custom_fields.rs`).

use crate::models::WpPostResponse;
use crate::templates::utils::escape_html;

pub fn render_custom_fields_card(existing: Option<&WpPostResponse>) -> String {
    let mut rows_html = String::new();

    if let Some(post) = existing {
        if let Some(meta_map) = post.meta.as_object() {
            let mut keys: Vec<&String> = meta_map.keys().collect();
            keys.sort();

            for k in keys {
                // Filter internal private WordPress / builder metadata
                if k == "_elementor_data"
                    || k == "_neopress_layout"
                    || k == "_wp_page_template"
                    || k == "_thumbnail_id"
                    || k.starts_with("_neopress_seo_")
                    || k == "_edit_lock"
                    || k == "_edit_last"
                {
                    continue;
                }

                let val_str = match meta_map.get(k) {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(serde_json::Value::Null) => String::new(),
                    Some(other) => other.to_string(),
                    None => String::new(),
                };

                let detected_type = detect_meta_type(&val_str);
                let type_badge = format!("META: {}", detected_type.to_uppercase());
                let k_esc = escape_html(k);
                let v_esc = escape_html(&val_str);

                rows_html.push_str(&format!(
                    r#"<tr class="custom-meta-row" data-meta-key="{k_esc}">
                        <td style="padding: 8px 10px; width: 32%;">
                            <input type="text" class="form-input custom-meta-key" value="{k_esc}" style="font-family: var(--font-mono); font-size: 12px; height: 30px; background: #f8fafc;" oninput="markDirty()" />
                        </td>
                        <td style="padding: 8px 10px; width: 44%;">
                            <input type="text" class="form-input custom-meta-val" value="{v_esc}" style="font-size: 12px; height: 30px;" oninput="updateMetaTypeBadge(this); markDirty()" />
                        </td>
                        <td style="padding: 8px 10px; width: 14%; text-align: center;">
                            <span class="badge custom-meta-type-badge" style="font-size: 10px; background: #f1f5f9; color: #475569; text-transform: uppercase;">{type_badge}</span>
                        </td>
                        <td style="padding: 8px 10px; width: 10%; text-align: center;">
                            <button type="button" class="btn btn-secondary btn-delete-meta" onclick="deleteCustomMetaRow(this)" title="Delete Custom Field" style="height: 26px; padding: 0 6px; font-size: 12px; color: #ef4444; border-color: #fecaca; cursor: pointer;">🗑️</button>
                        </td>
                    </tr>"#
                ));
            }
        }
    }

    let empty_hint = if rows_html.is_empty() {
        r#"<tr id="custom-fields-empty-hint"><td colspan="4" style="padding: 16px; text-align: center; color: #94a3b8; font-size: 12px;">No custom fields yet. Click <strong>+ Add Custom Field</strong> below to attach arbitrary <code>wp_postmeta</code> attributes.</td></tr>"#
    } else {
        ""
    };

    format!(
        r#"<div class="neo-card custom-fields-card" style="padding: 22px;" id="custom_fields_card">
            <div class="neo-card-header" style="padding: 0 0 14px 0; margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h2 class="neo-card-title">Dynamic Custom Fields (<code>wp_postmeta</code>)</h2>
                    <span class="badge badge-published" style="font-size: 11px;">Dynamic Schema</span>
                </div>
                <span style="font-size: 11.5px; color: #64748b; font-family: var(--font-mono);">Direct SQLite Persistence</span>
            </div>

            <table class="neo-table" id="custom-fields-table" style="width: 100%; border-collapse: collapse; margin-bottom: 12px;">
                <thead>
                    <tr style="border-bottom: 1px solid #e2e8f0; font-size: 11px; text-transform: uppercase; color: #64748b; text-align: left;">
                        <th style="padding: 6px 10px;">Name (Meta Key)</th>
                        <th style="padding: 6px 10px;">Value</th>
                        <th style="padding: 6px 10px; text-align: center;">Type</th>
                        <th style="padding: 6px 10px; text-align: center;">Action</th>
                    </tr>
                </thead>
                <tbody id="custom-fields-body">
                    {rows_html}
                    {empty_hint}
                </tbody>
            </table>

            <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid #f1f5f9; padding-top: 14px; margin-top: 6px;">
                <button type="button" class="btn btn-secondary" id="btn-add-custom-field" onclick="addCustomMetaRow()" style="font-size: 12px; height: 32px; display: inline-flex; align-items: center; gap: 6px;">
                    <span>+ Add Custom Field</span>
                </button>
                <span style="font-size: 11.5px; color: #94a3b8;">Custom fields sync automatically with <code>wp_postmeta</code> on Save / Publish.</span>
            </div>
        </div>"#
    )
}

fn detect_meta_type(val: &str) -> &'static str {
    let t = val.trim();
    if t == "1" || t == "0" || t == "true" || t == "false" {
        "boolean"
    } else if !t.is_empty() && t.parse::<f64>().is_ok() {
        "number"
    } else if (t.starts_with('{') && t.ends_with('}')) || (t.starts_with('[') && t.ends_with(']')) {
        "json"
    } else {
        "text"
    }
}
