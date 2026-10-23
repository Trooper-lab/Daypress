use crate::schema::Spacing;

pub(crate) fn map_divi_icon(raw: &str) -> String {
    let trimmed = raw.trim();
    match trimmed {
        "&#x49;" | "49" | "I" => "fa-solid fa-gem".to_string(),
        "&#xe06b;" => "fa-solid fa-heart".to_string(),
        "&#xe089;" => "fa-solid fa-star".to_string(),
        "&#xe027;" => "fa-solid fa-check".to_string(),
        "&#xe076;" => "fa-solid fa-shield-halved".to_string(),
        "&#xe099;" => "fa-brands fa-whatsapp".to_string(),
        "&#xe00b;" => "fa-solid fa-envelope".to_string(),
        "&#xe090;" => "fa-solid fa-phone".to_string(),
        "&#xe081;" => "fa-solid fa-location-dot".to_string(),
        "&#xe064;" => "fa-solid fa-circle-info".to_string(),
        "&#xe08b;" => "fa-solid fa-trophy".to_string(),
        "&#xe079;" => "fa-solid fa-wrench".to_string(),
        "&#xe028;" => "fa-solid fa-check-double".to_string(),
        "&#xe059;" => "fa-solid fa-bath".to_string(),
        "&#xe030;" => "fa-solid fa-utensils".to_string(),
        "&#xe031;" => "fa-solid fa-building".to_string(),
        "&#xe06a;" => "fa-solid fa-handshake".to_string(),
        "&#xf4fd;" | "f4fd" => "fa-solid fa-user-tie".to_string(),
        "&#xf5a0;" | "f5a0" => "fa-solid fa-map-location-dot".to_string(),
        "&#xf2a0;" | "f2a0" => "fa-solid fa-phone-volume".to_string(),
        "&#x3b;" | "3b" | ";" => "fa-solid fa-chevron-down".to_string(),
        "&#xe093;" => "fa-brands fa-facebook".to_string(),
        "&#xe09a;" => "fa-brands fa-instagram".to_string(),
        _ => "fa-solid fa-bolt".to_string(),
    }
}

pub(crate) fn parse_divi_quad(raw: &str) -> Option<Spacing> {
    let parts: Vec<&str> = raw.split('|').collect();
    let clean_part = |idx: usize| -> Option<String> {
        parts.get(idx).and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() || matches!(trimmed, "true" | "false" | "on" | "off") {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    };

    let top = clean_part(0);
    let right = clean_part(1);
    let bottom = clean_part(2);
    let left = clean_part(3);

    if top.is_some() || right.is_some() || bottom.is_some() || left.is_some() {
        Some(Spacing {
            top,
            right,
            bottom,
            left,
        })
    } else {
        None
    }
}
