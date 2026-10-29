use super::globals::extract_global_color;
use crate::schema::{BorderConfig, Spacing};
use serde_json::{Map, Value};

pub(crate) fn extract_spacing_val(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        if !s.is_empty() {
            return Some(s.to_string());
        }
    } else if let Some(n) = v.as_i64() {
        return Some(n.to_string());
    } else if let Some(n) = v.as_u64() {
        return Some(n.to_string());
    } else if let Some(n) = v.as_f64() {
        if (n - n.round()).abs() < f64::EPSILON {
            return Some((n.round() as i64).to_string());
        }
        return Some(n.to_string());
    }
    None
}

pub(crate) fn parse_spacing_obj(obj: &Map<String, Value>) -> Option<Spacing> {
    let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
    let top = obj
        .get("top")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));
    let right = obj
        .get("right")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));
    let bottom = obj
        .get("bottom")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));
    let left = obj
        .get("left")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));

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

pub(crate) fn extract_border_config(
    settings: &Value,
    globals: Option<&Map<String, Value>>,
) -> Option<BorderConfig> {
    let border_radius = settings
        .get("border_radius")
        .or_else(|| settings.get("_border_radius"))
        .and_then(|b| {
            if let Some(obj) = b.as_object() {
                let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
                if let Some(sz) = obj.get("size").and_then(extract_spacing_val) {
                    return Some(format!("{}{}", sz, unit));
                }
                let top = obj.get("top").and_then(extract_spacing_val);
                let right = obj.get("right").and_then(extract_spacing_val);
                let bottom = obj.get("bottom").and_then(extract_spacing_val);
                let left = obj.get("left").and_then(extract_spacing_val);
                match (top, right, bottom, left) {
                    (Some(t), Some(r), Some(btm), Some(l)) => {
                        if t == r && r == btm && btm == l {
                            Some(format!("{}{}", t, unit))
                        } else {
                            Some(format!(
                                "{}{} {}{} {}{} {}{}",
                                t, unit, r, unit, btm, unit, l, unit
                            ))
                        }
                    }
                    (Some(t), _, _, _) => Some(format!("{}{}", t, unit)),
                    _ => None,
                }
            } else if let Some(n) = b.as_u64() {
                Some(format!("{}px", n))
            } else {
                b.as_str().filter(|s| !s.is_empty()).map(|s| s.to_string())
            }
        });

    let border_style = settings
        .get("border_border")
        .or_else(|| settings.get("_border_border"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty() && *s != "none")
        .map(|s| s.to_string());

    let border_width = settings
        .get("border_width")
        .or_else(|| settings.get("_border_width"))
        .and_then(|w| w.as_object())
        .and_then(|obj| {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
            if let Some(sz) = obj.get("size").and_then(extract_spacing_val) {
                return Some(format!("{}{}", sz, unit));
            }
            let t = obj.get("top").and_then(extract_spacing_val);
            let r = obj.get("right").and_then(extract_spacing_val);
            let b = obj.get("bottom").and_then(extract_spacing_val);
            let l = obj.get("left").and_then(extract_spacing_val);
            if t.is_some() || r.is_some() || b.is_some() || l.is_some() {
                let ts = t.unwrap_or_else(|| "0".to_string());
                let rs = r.unwrap_or_else(|| "0".to_string());
                let bs = b.unwrap_or_else(|| "0".to_string());
                let ls = l.unwrap_or_else(|| "0".to_string());
                if ts == rs && rs == bs && bs == ls {
                    Some(format!("{}{}", ts, unit))
                } else {
                    Some(format!(
                        "{}{} {}{} {}{} {}{}",
                        ts, unit, rs, unit, bs, unit, ls, unit
                    ))
                }
            } else {
                None
            }
        });

    let border_color = globals
        .and_then(|g| {
            extract_global_color(g, "border_color")
                .or_else(|| extract_global_color(g, "_border_color"))
        })
        .or_else(|| {
            settings
                .get("border_color")
                .or_else(|| settings.get("_border_color"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });

    if border_radius.is_some()
        || border_style.is_some()
        || border_width.is_some()
        || border_color.is_some()
    {
        Some(BorderConfig {
            radius: border_radius,
            width: border_width,
            color: border_color,
            style: border_style,
        })
    } else {
        None
    }
}
