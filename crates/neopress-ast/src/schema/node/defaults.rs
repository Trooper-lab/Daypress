use serde::de::Deserializer;
use serde::Deserialize;
use serde_json::Value;

pub(crate) fn default_grid_columns() -> u8 {
    3
}
pub(crate) fn default_direction() -> String {
    "row".to_string()
}
pub(crate) fn default_justify() -> String {
    "flex-start".to_string()
}
pub(crate) fn default_align() -> String {
    "stretch".to_string()
}
pub(crate) fn default_heading_level() -> u8 {
    2
}
pub(crate) fn default_button_text() -> String {
    "Click Here".to_string()
}
pub(crate) fn default_button_url() -> String {
    "#".to_string()
}
pub(crate) fn default_target() -> String {
    "_self".to_string()
}
pub(crate) fn default_form_id() -> i64 {
    1
}
pub(crate) fn default_spacer_height() -> u32 {
    40
}
pub(crate) fn default_divider_style() -> String {
    "solid".to_string()
}
pub(crate) fn default_counter_number() -> i64 {
    100
}
pub(crate) fn default_aspect() -> String {
    "16/9".to_string()
}
pub(crate) fn default_alert_type() -> String {
    "info".to_string()
}
pub(crate) fn default_true() -> bool {
    true
}
pub(crate) fn default_rating() -> f32 {
    5.0
}
pub(crate) fn default_max_rating() -> u8 {
    5
}
pub(crate) fn default_countdown_mode() -> String {
    "due_date".to_string()
}
pub(crate) fn default_flip_direction() -> String {
    "flip_right".to_string()
}
pub(crate) fn default_animation_style() -> String {
    "typing".to_string()
}
pub(crate) fn default_nav_layout() -> String {
    "horizontal".to_string()
}
pub(crate) fn default_nav_align() -> String {
    "left".to_string()
}
pub(crate) fn default_post_type() -> String {
    "post".to_string()
}
pub(crate) fn default_posts_per_page() -> u32 {
    6
}
pub(crate) fn default_toc_title() -> String {
    "Table of Contents".to_string()
}
pub(crate) fn default_toc_min_depth() -> u8 {
    2
}
pub(crate) fn default_toc_max_depth() -> u8 {
    4
}
pub(crate) fn default_breadcrumb_separator() -> String {
    "/".to_string()
}
pub(crate) fn default_breadcrumb_home() -> String {
    "Home".to_string()
}
pub(crate) fn default_search_placeholder() -> String {
    "Search...".to_string()
}
pub(crate) fn default_search_button_text() -> String {
    "Search".to_string()
}
pub(crate) fn default_search_endpoint() -> String {
    "/".to_string()
}
pub(crate) fn default_login_user_label() -> String {
    "Username or Email Address".to_string()
}
pub(crate) fn default_login_pass_label() -> String {
    "Password".to_string()
}
pub(crate) fn default_login_redirect() -> String {
    "/".to_string()
}

pub(crate) fn deserialize_counter_number<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Value::deserialize(deserializer)?;
    match val {
        Value::Number(n) => Ok(n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f.round() as i64))
            .unwrap_or(100)),
        Value::String(s) => {
            let trimmed = s.trim();
            let mut numeric = String::new();
            for (idx, ch) in trimmed.chars().enumerate() {
                if (idx == 0 && ch == '-') || ch.is_ascii_digit() {
                    numeric.push(ch);
                } else if ch == ',' {
                    continue;
                } else {
                    break;
                }
            }
            Ok(numeric.parse::<i64>().unwrap_or(100))
        }
        _ => Ok(100),
    }
}

pub(crate) fn deserialize_aspect_ratio<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<String>::deserialize(deserializer)?;
    match val {
        Some(s) if !s.trim().is_empty() => Ok(s.replace(':', "/")),
        _ => Ok(default_aspect()),
    }
}
