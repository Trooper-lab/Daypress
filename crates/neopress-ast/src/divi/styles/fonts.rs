use crate::schema::StyleProperties;

pub(crate) fn parse_divi_font_composite(font_raw: &str, styles: &mut StyleProperties) {
    let parts: Vec<&str> = font_raw.split('|').collect();
    if let Some(fam) = parts.first().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let resolved_fam = match fam {
            "--et_global_heading_font" => "'Squada One', sans-serif".to_string(),
            "--et_global_body_font" => "'Barlow', sans-serif".to_string(),
            other => format!("'{}', sans-serif", other),
        };
        styles.font_family = Some(resolved_fam);
    }
    if let Some(w) = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty()) {
        styles.font_weight = Some(w.to_string());
    }
    if let Some(style) = parts.get(2).map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if style.contains("italic") || style == "on" {
            styles.font_style = Some("italic".to_string());
        }
    }
    if let Some(uppercase) = parts.get(3).map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if uppercase == "on" || uppercase.contains("uppercase") {
            styles.text_transform = Some("uppercase".to_string());
        }
    }
    if let Some(underline) = parts.get(4).map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if underline == "on" || underline.contains("underline") {
            styles.text_decoration = Some("underline".to_string());
        }
    }
}
