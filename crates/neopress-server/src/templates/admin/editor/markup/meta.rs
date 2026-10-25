//! Custom ACF and CPT metadata card renderer (`markup/meta.rs`).

use crate::models::WpPostResponse;
use crate::templates::utils::escape_html;

pub fn get_meta_value(existing: Option<&WpPostResponse>, key: &str) -> String {
    existing
        .and_then(|p| p.meta.as_object())
        .and_then(|m| m.get(key))
        .and_then(|v| v.as_str())
        .map(escape_html)
        .unwrap_or_default()
}

/// Meta keys edited by the ACF fields card below.
const ACF_CARD_META_KEYS: [&str; 7] = [
    "header_height",
    "header_hoogte",
    "header_title",
    "show_title",
    "header_shape",
    "show_icon",
    "header_left",
];

/// Whether a post carries ACF data: ACF blocks in its content, ACF field references
/// (`_field` meta values like `field_6123abc`), or the meta keys this card edits.
/// New posts and plain sites never show the ACF tooling.
pub fn post_uses_acf(existing: Option<&WpPostResponse>) -> bool {
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

pub fn render_acf_fields_card(existing: Option<&WpPostResponse>, post_type: &str) -> String {
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

    let cpt_specific_fields = match post_type {
        "trainingen" => {
            let datum = get_meta("datum");
            let start_time = get_meta("start_time");
            let training_tijd = {
                let v = get_meta("training_tijd");
                if v.is_empty() {
                    get_meta("time_investment")
                } else {
                    v
                }
            };
            let deelnemers = {
                let v = get_meta("training_deelnemers");
                if v.is_empty() {
                    get_meta("deelnemers")
                } else {
                    v
                }
            };
            let score = get_meta("score_training");
            let sterren = get_meta("sterren_training");
            format!(
                r#"
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 14px; padding-top: 14px; border-top: 1px solid #f1f5f9;">
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">📅 Datum</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="datum" value="{datum}" placeholder="Bijv. 14 november 2026" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">⏰ Starttijd</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="start_time" value="{start_time}" placeholder="09:30" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">⏱️ Tijdsduur / Dagen</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="training_tijd" value="{training_tijd}" placeholder="1 dag / 4 dagdelen" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">👥 Deelnemers</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="training_deelnemers" value="{deelnemers}" placeholder="Max. 8 personen" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">⭐ Beoordeling Cijfer</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="score_training" value="{score}" placeholder="9.2" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">🌟 Aantal Sterren</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="sterren_training" value="{sterren}" placeholder="5" oninput="onAcfFieldChange()" />
                    </div>
                </div>"#
            )
        }
        "vacatures" => {
            let salaris = get_meta("salaris");
            let uren = get_meta("uren");
            let locatie = get_meta("locatie");
            let niveau = get_meta("opleidingsniveau");
            format!(
                r#"
                <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 14px; margin-top: 14px; padding-top: 14px; border-top: 1px solid #f1f5f9;">
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">💶 Salaris Indicatie</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="salaris" value="{salaris}" placeholder="€ 3.500 - € 5.500" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">⏱️ Uren per week</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="uren" value="{uren}" placeholder="32 - 40 uur" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">📍 Locatie / Standplaats</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="locatie" value="{locatie}" placeholder="Utrecht / Hybride" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">🎓 Opleidingsniveau</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="opleidingsniveau" value="{niveau}" placeholder="HBO / WO Werk- en denkniveau" oninput="onAcfFieldChange()" />
                    </div>
                </div>"#
            )
        }
        "cases" => {
            let klant = get_meta("klant");
            let resultaat = get_meta("resultaat");
            let doorlooptijd = get_meta("doorlooptijd");
            format!(
                r#"
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 14px; padding-top: 14px; border-top: 1px solid #f1f5f9;">
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">🏢 Opdrachtgever / Klant</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="klant" value="{klant}" placeholder="Naam organisatie" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">📈 Resultaat Highlight</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="resultaat" value="{resultaat}" placeholder="Bijv. 40% efficiënter" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">⏳ Doorlooptijd</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="doorlooptijd" value="{doorlooptijd}" placeholder="6 maanden" oninput="onAcfFieldChange()" />
                    </div>
                </div>"#
            )
        }
        "kandidaten" => {
            let beschikbaar = get_meta("beschikbaarheid");
            let rol = get_meta("functietitel");
            let ervaring = get_meta("ervaring");
            format!(
                r#"
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 14px; padding-top: 14px; border-top: 1px solid #f1f5f9;">
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">💼 Functietitel / Specialisme</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="functietitel" value="{rol}" placeholder="Data Engineer / Scrum Master" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">📅 Beschikbaarheid</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="beschikbaarheid" value="{beschikbaar}" placeholder="Per direct (36u)" oninput="onAcfFieldChange()" />
                    </div>
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">⭐ Ervaring</label>
                        <input type="text" class="form-input acf-meta-input" data-acf-key="ervaring" value="{ervaring}" placeholder="Senior (8+ jaar)" oninput="onAcfFieldChange()" />
                    </div>
                </div>"#
            )
        }
        _ => String::new(),
    };

    format!(
        r#"<div class="neo-card" style="padding: 22px;">
            <div class="neo-card-header" style="padding: 0 0 14px 0; margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h2 class="neo-card-title">Theme Header &amp; ACF Custom Fields</h2>
                    <span class="badge badge-published" style="font-size: 11px;">ACF / Gutenberg Hybrid</span>
                </div>
                <span style="font-size: 11.5px; color: #64748b; font-family: var(--font-mono);">wp_postmeta synced</span>
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
            {cpt_specific_fields}
        </div>"#
    )
}
