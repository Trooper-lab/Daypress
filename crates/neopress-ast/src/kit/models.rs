use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub(crate) fn deserialize_opt_string_or_size<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<Value>::deserialize(deserializer)?;
    match val {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => {
            if s.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(s))
            }
        }
        Some(Value::Number(n)) => Ok(Some(format!("{}px", n))),
        Some(Value::Object(obj)) => {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
            if let Some(sz) = obj.get("size") {
                match sz {
                    Value::String(s) if !s.trim().is_empty() => Ok(Some(format!("{}{}", s, unit))),
                    Value::Number(n) => Ok(Some(format!("{}{}", n, unit))),
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

pub(crate) fn deserialize_u32_or_size<'de, D>(
    deserializer: D,
    default_val: u32,
) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<Value>::deserialize(deserializer)?;
    match val {
        Some(Value::Number(n)) => Ok(n.as_u64().unwrap_or(default_val as u64) as u32),
        Some(Value::String(s)) => Ok(s.trim().parse::<u32>().unwrap_or(default_val)),
        Some(Value::Object(obj)) => {
            if let Some(sz) = obj.get("size") {
                match sz {
                    Value::Number(n) => Ok(n.as_u64().unwrap_or(default_val as u64) as u32),
                    Value::String(s) => Ok(s.trim().parse::<u32>().unwrap_or(default_val)),
                    _ => Ok(default_val),
                }
            } else {
                Ok(default_val)
            }
        }
        _ => Ok(default_val),
    }
}

pub(crate) fn deserialize_content_width<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_u32_or_size(deserializer, default_content_width())
}

pub(crate) fn deserialize_widgets_space<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_u32_or_size(deserializer, default_widgets_space())
}

/// Represents a single global color token translated from Elementor's Global_Colors kit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalColor {
    #[serde(alias = "_id")]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub color: String,
}

/// Represents a typography token translated from Elementor's Global_Typography kit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalTypography {
    #[serde(alias = "_id")]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_font_family", alias = "typography_font_family")]
    pub font_family: String,
    #[serde(
        default,
        alias = "typography_font_size",
        deserialize_with = "deserialize_opt_string_or_size"
    )]
    pub font_size: Option<String>,
    #[serde(
        default,
        alias = "typography_font_weight",
        deserialize_with = "deserialize_opt_string_or_size"
    )]
    pub font_weight: Option<String>,
    #[serde(
        default,
        alias = "typography_line_height",
        deserialize_with = "deserialize_opt_string_or_size"
    )]
    pub line_height: Option<String>,
    #[serde(
        default,
        alias = "typography_letter_spacing",
        deserialize_with = "deserialize_opt_string_or_size"
    )]
    pub letter_spacing: Option<String>,
}

pub(crate) fn default_font_family() -> String {
    "Roboto, sans-serif".to_string()
}

/// Elementor Kit Site Settings translated from Elementor Core PHP (elementor/core/kits/).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementorKit {
    #[serde(default = "default_system_colors")]
    pub system_colors: Vec<GlobalColor>,
    #[serde(default)]
    pub custom_colors: Vec<GlobalColor>,
    #[serde(default = "default_system_typography")]
    pub system_typography: Vec<GlobalTypography>,
    #[serde(default)]
    pub custom_typography: Vec<GlobalTypography>,
    #[serde(
        default = "default_content_width",
        alias = "container_width",
        deserialize_with = "deserialize_content_width"
    )]
    pub content_width: u32,
    #[serde(
        default = "default_widgets_space",
        alias = "space_between_widgets",
        deserialize_with = "deserialize_widgets_space"
    )]
    pub widgets_space: u32,
    #[serde(default)]
    pub site_name: Option<String>,
    #[serde(default)]
    pub site_description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_size")]
    pub site_logo: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_size")]
    pub site_favicon: Option<String>,
    #[serde(default)]
    pub button_style: Option<ButtonStyle>,
    #[serde(default)]
    pub custom_css: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ButtonStyle {
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub text_color: Option<String>,
    #[serde(default)]
    pub border_radius: Option<String>,
}

pub(crate) fn default_content_width() -> u32 {
    1140
}

pub(crate) fn default_widgets_space() -> u32 {
    20
}

pub fn default_system_colors() -> Vec<GlobalColor> {
    vec![
        GlobalColor {
            id: "primary".to_string(),
            title: "Primary".to_string(),
            color: "#6EC1E4".to_string(),
        },
        GlobalColor {
            id: "secondary".to_string(),
            title: "Secondary".to_string(),
            color: "#54595F".to_string(),
        },
        GlobalColor {
            id: "text".to_string(),
            title: "Text".to_string(),
            color: "#7A7A7A".to_string(),
        },
        GlobalColor {
            id: "accent".to_string(),
            title: "Accent".to_string(),
            color: "#61CE70".to_string(),
        },
    ]
}

pub fn default_system_typography() -> Vec<GlobalTypography> {
    vec![
        GlobalTypography {
            id: "primary".to_string(),
            title: "Primary".to_string(),
            font_family: "Roboto, sans-serif".to_string(),
            font_size: Some("32px".to_string()),
            font_weight: Some("600".to_string()),
            line_height: Some("1.2em".to_string()),
            letter_spacing: None,
        },
        GlobalTypography {
            id: "secondary".to_string(),
            title: "Secondary".to_string(),
            font_family: "Roboto Slab, serif".to_string(),
            font_size: Some("24px".to_string()),
            font_weight: Some("400".to_string()),
            line_height: Some("1.3em".to_string()),
            letter_spacing: None,
        },
        GlobalTypography {
            id: "text".to_string(),
            title: "Text".to_string(),
            font_family: "Roboto, sans-serif".to_string(),
            font_size: Some("16px".to_string()),
            font_weight: Some("400".to_string()),
            line_height: Some("1.6em".to_string()),
            letter_spacing: None,
        },
        GlobalTypography {
            id: "accent".to_string(),
            title: "Accent".to_string(),
            font_family: "Roboto, sans-serif".to_string(),
            font_size: Some("14px".to_string()),
            font_weight: Some("500".to_string()),
            line_height: Some("1.4em".to_string()),
            letter_spacing: None,
        },
    ]
}

impl Default for ElementorKit {
    fn default() -> Self {
        Self {
            system_colors: default_system_colors(),
            custom_colors: Vec::new(),
            system_typography: default_system_typography(),
            custom_typography: Vec::new(),
            content_width: default_content_width(),
            widgets_space: default_widgets_space(),
            site_name: None,
            site_description: None,
            site_logo: None,
            site_favicon: None,
            button_style: None,
            custom_css: None,
        }
    }
}
