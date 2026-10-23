use super::quad::parse_divi_quad;
use crate::schema::{StyleProperties, TypographyConfig};
use std::collections::HashMap;

pub(crate) fn apply_divi_responsive_styles(
    styles: &mut StyleProperties,
    attrs: &HashMap<String, String>,
) {
    // Responsive Quads & Typography (Tablet)
    let tab_pad = attrs
        .get("custom_padding_tablet")
        .and_then(|p| parse_divi_quad(p));
    let tab_mar = attrs
        .get("custom_margin_tablet")
        .and_then(|m| parse_divi_quad(m));
    let tab_font_sz = attrs
        .get("title_font_size_tablet")
        .or_else(|| attrs.get("header_font_size_tablet"))
        .or_else(|| attrs.get("text_font_size_tablet"))
        .filter(|s| !s.is_empty())
        .cloned();
    let tab_align = attrs
        .get("text_orientation_tablet")
        .or_else(|| attrs.get("header_text_align_tablet"))
        .filter(|s| !s.is_empty())
        .cloned();

    if tab_pad.is_some() || tab_mar.is_some() || tab_font_sz.is_some() || tab_align.is_some() {
        let tab = styles
            .tablet
            .get_or_insert_with(|| Box::new(StyleProperties::default()));
        if let Some(p) = tab_pad {
            tab.padding = Some(p);
        }
        if let Some(m) = tab_mar {
            tab.margin = Some(m);
        }
        if let Some(fs) = tab_font_sz {
            tab.font_size = Some(fs.clone());
            let t = tab.typography.get_or_insert_with(TypographyConfig::default);
            t.font_size = Some(fs);
        }
        if let Some(ta) = tab_align {
            tab.text_align = Some(ta.clone());
            let t = tab.typography.get_or_insert_with(TypographyConfig::default);
            t.text_align = Some(ta);
        }
        tab.sync_flat_and_nested();
    }

    // Responsive Quads & Typography (Phone/Mobile)
    let phone_pad = attrs
        .get("custom_padding_phone")
        .and_then(|p| parse_divi_quad(p));
    let phone_mar = attrs
        .get("custom_margin_phone")
        .and_then(|m| parse_divi_quad(m));
    let phone_font_sz = attrs
        .get("title_font_size_phone")
        .or_else(|| attrs.get("header_font_size_phone"))
        .or_else(|| attrs.get("text_font_size_phone"))
        .filter(|s| !s.is_empty())
        .cloned();
    let phone_align = attrs
        .get("text_orientation_phone")
        .or_else(|| attrs.get("header_text_align_phone"))
        .filter(|s| !s.is_empty())
        .cloned();

    if phone_pad.is_some() || phone_mar.is_some() || phone_font_sz.is_some() || phone_align.is_some() {
        let mob = styles
            .mobile
            .get_or_insert_with(|| Box::new(StyleProperties::default()));
        if let Some(p) = phone_pad {
            mob.padding = Some(p);
        }
        if let Some(m) = phone_mar {
            mob.margin = Some(m);
        }
        if let Some(fs) = phone_font_sz {
            mob.font_size = Some(fs.clone());
            let t = mob.typography.get_or_insert_with(TypographyConfig::default);
            t.font_size = Some(fs);
        }
        if let Some(ta) = phone_align {
            mob.text_align = Some(ta.clone());
            let t = mob.typography.get_or_insert_with(TypographyConfig::default);
            t.text_align = Some(ta);
        }
        mob.sync_flat_and_nested();
    }

    // Disabled On / Visibility flags (disabled_on="on|off|off" -> desktop|tablet|phone)
    if let Some(disabled) = attrs.get("disabled_on") {
        let parts: Vec<&str> = disabled.split('|').collect();
        if let Some(&d) = parts.first() {
            if d == "on" {
                styles.hide_desktop = Some(true);
            }
        }
        if let Some(&t) = parts.get(1) {
            if t == "on" {
                styles.hide_tablet = Some(true);
            }
        }
        if let Some(&m) = parts.get(2) {
            if m == "on" {
                styles.hide_mobile = Some(true);
            }
        }
    }
}
