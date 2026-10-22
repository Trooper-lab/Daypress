use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

fn deserialize_opt_string_or_size<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
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

fn deserialize_u32_or_size<'de, D>(deserializer: D, default_val: u32) -> Result<u32, D::Error>
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

fn deserialize_content_width<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_u32_or_size(deserializer, default_content_width())
}

fn deserialize_widgets_space<'de, D>(deserializer: D) -> Result<u32, D::Error>
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

fn default_font_family() -> String {
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

fn default_content_width() -> u32 {
    1140
}

fn default_widgets_space() -> u32 {
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

impl ElementorKit {
    /// Parses an `ElementorKit` from either a JSON string or a PHP-serialized `_elementor_page_settings` string.
    pub fn from_raw_settings(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(kit) = serde_json::from_str::<ElementorKit>(trimmed) {
            return Some(kit);
        }
        if let Some(json_val) = parse_php_serialized_to_json(trimmed) {
            if let Ok(kit) = serde_json::from_value::<ElementorKit>(json_val.clone()) {
                return Some(kit);
            }
            if let Some(settings) = json_val.get("settings") {
                if let Ok(kit) = serde_json::from_value::<ElementorKit>(settings.clone()) {
                    return Some(kit);
                }
            }
        }
        None
    }

    /// Compiles the global colors and typography tokens into valid CSS variables.
    pub fn compile_to_css(&self) -> String {
        let mut css = String::new();
        css.push_str(":root {\n");
        css.push_str(&format!("  --e-content-width: {}px;\n", self.content_width));
        css.push_str(&format!("  --e-widgets-space: {}px;\n", self.widgets_space));

        // System Colors
        for c in &self.system_colors {
            css.push_str(&format!("  --e-global-color-{}: {};\n", c.id, c.color));
        }
        // Custom Colors
        for c in &self.custom_colors {
            css.push_str(&format!("  --e-global-color-{}: {};\n", c.id, c.color));
        }

        // System Typography
        for t in &self.system_typography {
            css.push_str(&format!("  --e-global-typography-{}-font-family: {};\n", t.id, t.font_family));
            if let Some(ref sz) = t.font_size {
                css.push_str(&format!("  --e-global-typography-{}-font-size: {};\n", t.id, sz));
            }
            if let Some(ref fw) = t.font_weight {
                css.push_str(&format!("  --e-global-typography-{}-font-weight: {};\n", t.id, fw));
            }
            if let Some(ref lh) = t.line_height {
                css.push_str(&format!("  --e-global-typography-{}-line-height: {};\n", t.id, lh));
            }
            if let Some(ref ls) = t.letter_spacing {
                css.push_str(&format!("  --e-global-typography-{}-letter-spacing: {};\n", t.id, ls));
            }
        }

        // Custom Typography
        for t in &self.custom_typography {
            css.push_str(&format!("  --e-global-typography-{}-font-family: {};\n", t.id, t.font_family));
            if let Some(ref sz) = t.font_size {
                css.push_str(&format!("  --e-global-typography-{}-font-size: {};\n", t.id, sz));
            }
            if let Some(ref fw) = t.font_weight {
                css.push_str(&format!("  --e-global-typography-{}-font-weight: {};\n", t.id, fw));
            }
            if let Some(ref lh) = t.line_height {
                css.push_str(&format!("  --e-global-typography-{}-line-height: {};\n", t.id, lh));
            }
            if let Some(ref ls) = t.letter_spacing {
                css.push_str(&format!("  --e-global-typography-{}-letter-spacing: {};\n", t.id, ls));
            }
        }

        // Buttons Style
        if let Some(ref btn) = self.button_style {
            if let Some(ref bg) = btn.background_color {
                css.push_str(&format!("  --e-button-bg: {};\n", bg));
            }
            if let Some(ref tc) = btn.text_color {
                css.push_str(&format!("  --e-button-color: {};\n", tc));
            }
            if let Some(ref br) = btn.border_radius {
                css.push_str(&format!("  --e-button-border-radius: {};\n", br));
            }
        }

        css.push_str("}\n");

        if let Some(ref custom_css) = self.custom_css {
            if !custom_css.trim().is_empty() {
                css.push_str("\n/* Elementor Custom CSS */\n");
                css.push_str(custom_css);
                css.push('\n');
            }
        }

        css
    }
}

/// Zero-panic recursive parser for PHP `serialize()` payloads (`a:...`, `s:...`, `i:...`, `d:...`, `b:...`, `N;`),
/// converting them into `serde_json::Value`.
pub fn parse_php_serialized_to_json(raw: &str) -> Option<Value> {
    let bytes = raw.trim().as_bytes();
    let mut pos = 0;
    parse_php_val(bytes, &mut pos)
}

fn parse_php_val(bytes: &[u8], pos: &mut usize) -> Option<Value> {
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
    if *pos >= bytes.len() {
        return None;
    }
    let tag = bytes[*pos];
    *pos += 1;
    match tag {
        b'N' => {
            if *pos < bytes.len() && bytes[*pos] == b';' {
                *pos += 1;
                Some(Value::Null)
            } else {
                None
            }
        }
        b'b' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b';' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let val_str = std::str::from_utf8(&bytes[start..*pos]).ok()?;
            *pos += 1; // skip ';'
            Some(Value::Bool(val_str.trim() == "1"))
        }
        b'i' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b';' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let val_str = std::str::from_utf8(&bytes[start..*pos]).ok()?;
            *pos += 1; // skip ';'
            let n = val_str.trim().parse::<i64>().ok()?;
            Some(Value::Number(n.into()))
        }
        b'd' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b';' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let val_str = std::str::from_utf8(&bytes[start..*pos]).ok()?;
            *pos += 1; // skip ';'
            let f = val_str.trim().parse::<f64>().ok()?;
            serde_json::Number::from_f64(f).map(Value::Number)
        }
        b's' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let len_start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b':' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let len_str = std::str::from_utf8(&bytes[len_start..*pos]).ok()?;
            let str_len = len_str.trim().parse::<usize>().ok()?;
            *pos += 1; // skip ':'
            if *pos >= bytes.len() || bytes[*pos] != b'"' {
                return None;
            }
            *pos += 1; // skip opening '"'
            if *pos + str_len > bytes.len() {
                return None;
            }
            let s = String::from_utf8_lossy(&bytes[*pos..*pos + str_len]).to_string();
            *pos += str_len;
            if *pos < bytes.len() && bytes[*pos] == b'"' {
                *pos += 1;
            }
            if *pos < bytes.len() && bytes[*pos] == b';' {
                *pos += 1;
            }
            Some(Value::String(s))
        }
        b'a' => {
            if *pos >= bytes.len() || bytes[*pos] != b':' {
                return None;
            }
            *pos += 1;
            let count_start = *pos;
            while *pos < bytes.len() && bytes[*pos] != b':' {
                *pos += 1;
            }
            if *pos >= bytes.len() {
                return None;
            }
            let count_str = std::str::from_utf8(&bytes[count_start..*pos]).ok()?;
            let count = count_str.trim().parse::<usize>().ok()?;
            *pos += 1; // skip ':'
            if *pos >= bytes.len() || bytes[*pos] != b'{' {
                return None;
            }
            *pos += 1; // skip '{'

            let mut entries: Vec<(Value, Value)> = Vec::with_capacity(count);
            for _ in 0..count {
                let k = parse_php_val(bytes, pos)?;
                let v = parse_php_val(bytes, pos)?;
                entries.push((k, v));
            }
            while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
                *pos += 1;
            }
            if *pos < bytes.len() && bytes[*pos] == b'}' {
                *pos += 1;
            }

            let is_sequential_list = entries.iter().enumerate().all(|(idx, (k, _))| {
                k.as_i64() == Some(idx as i64)
            });

            if is_sequential_list {
                Some(Value::Array(entries.into_iter().map(|(_, v)| v).collect()))
            } else {
                let mut map = Map::new();
                for (k, v) in entries {
                    let key_str = match k {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        _ => continue,
                    };
                    map.insert(key_str, v);
                }
                Some(Value::Object(map))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elementor_kit_defaults() {
        let kit = ElementorKit::default();
        assert_eq!(kit.system_colors.len(), 4);
        assert_eq!(kit.system_colors[0].color, "#6EC1E4");
        assert_eq!(kit.system_typography.len(), 4);
        assert_eq!(kit.content_width, 1140);
        assert_eq!(kit.widgets_space, 20);

        let css = kit.compile_to_css();
        assert!(css.contains("--e-global-color-primary: #6EC1E4;"));
        assert!(css.contains("--e-global-color-accent: #61CE70;"));
        assert!(css.contains("--e-content-width: 1140px;"));
        assert!(css.contains("--e-global-typography-primary-font-family: Roboto, sans-serif;"));
    }

    #[test]
    fn test_elementor_kit_custom_colors_and_css() {
        let mut kit = ElementorKit::default();
        kit.custom_colors.push(GlobalColor {
            id: "brand-purple".to_string(),
            title: "Brand Purple".to_string(),
            color: "#8b5cf6".to_string(),
        });
        kit.custom_css = Some(".custom-banner { padding: 40px; }".to_string());

        let css = kit.compile_to_css();
        assert!(css.contains("--e-global-color-brand-purple: #8b5cf6;"));
        assert!(css.contains(".custom-banner { padding: 40px; }"));
    }

    #[test]
    fn test_elementor_kit_button_style() {
        let kit = ElementorKit {
            button_style: Some(ButtonStyle {
                background_color: Some("#ff4f00".to_string()),
                text_color: Some("#ffffff".to_string()),
                border_radius: Some("8px".to_string()),
            }),
            ..Default::default()
        };

        let css = kit.compile_to_css();
        assert!(css.contains("--e-button-bg: #ff4f00;"));
        assert!(css.contains("--e-button-color: #ffffff;"));
        assert!(css.contains("--e-button-border-radius: 8px;"));
    }

    #[test]
    fn test_php_serialized_elementor_kit_parsing() {
        let php_serialized = r##"a:19:{s:21:"space_between_widgets";a:3:{s:4:"unit";s:2:"px";s:4:"size";s:0:"";s:5:"sizes";a:0:{}}s:13:"system_colors";a:4:{i:0;a:3:{s:3:"_id";s:7:"primary";s:5:"title";s:7:"Primair";s:5:"color";s:7:"#6A6351";}i:1;a:3:{s:3:"_id";s:9:"secondary";s:5:"title";s:9:"Secundair";s:5:"color";s:7:"#3A3633";}i:2;a:3:{s:3:"_id";s:4:"text";s:5:"title";s:5:"Tekst";s:5:"color";s:7:"#1F1C17";}i:3;a:3:{s:3:"_id";s:6:"accent";s:5:"title";s:6:"Accent";s:5:"color";s:7:"#B8A98A";}}s:13:"custom_colors";a:4:{i:0;a:3:{s:3:"_id";s:7:"2171565";s:5:"title";s:14:"Saved Color #5";s:5:"color";s:7:"#4054B2";}i:1;a:3:{s:3:"_id";s:8:"581fd4a7";s:5:"title";s:14:"Saved Color #6";s:5:"color";s:7:"#23A455";}i:2;a:3:{s:3:"_id";s:8:"4b94e36e";s:5:"title";s:14:"Saved Color #7";s:5:"color";s:4:"#000";}i:3;a:3:{s:3:"_id";s:8:"498af94a";s:5:"title";s:14:"Saved Color #8";s:5:"color";s:4:"#FFF";}}s:17:"system_typography";a:4:{i:0;a:5:{s:3:"_id";s:7:"primary";s:5:"title";s:17:"Primaire kopregel";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:9:"Fira Sans";s:22:"typography_font_weight";s:3:"600";}i:1;a:5:{s:3:"_id";s:9:"secondary";s:5:"title";s:19:"Secundaire kopregel";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:9:"Fira Sans";s:22:"typography_font_weight";s:3:"400";}i:2;a:5:{s:3:"_id";s:4:"text";s:5:"title";s:10:"Body-tekst";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:6:"Roboto";s:22:"typography_font_weight";s:3:"400";}i:3;a:5:{s:3:"_id";s:6:"accent";s:5:"title";s:12:"Accent-tekst";s:21:"typography_typography";s:6:"custom";s:22:"typography_font_family";s:6:"Roboto";s:22:"typography_font_weight";s:3:"500";}}s:9:"site_name";s:18:"Acme Flooring Demo";s:16:"site_description";s:36:"Thuiskomen op de vloer van je dromen";s:17:"custom_typography";a:0:{}s:21:"default_generic_fonts";s:10:"Sans-serif";s:19:"page_title_selector";s:14:"h1.entry-title";s:27:"hello_footer_copyright_text";s:25:"Alle rechten voorbehouden";s:15:"activeItemIndex";i:1;s:11:"viewport_md";i:768;s:11:"viewport_lg";i:1025;s:25:"hello_header_logo_display";s:0:"";s:28:"hello_header_tagline_display";s:0:"";s:44:"settings_page_transitions_entrance_animation";s:8:"fade-out";s:40:"settings_page_transitions_preloader_type";s:9:"animation";s:50:"settings_page_transitions_preloader_animation_type";s:13:"circle-dashed";s:11:"__globals__";a:1:{s:42:"settings_page_transitions_background_color";s:27:"globals/colors?id=secondary";}}"##;

        let kit = ElementorKit::from_raw_settings(php_serialized).expect("Should parse PHP serialized ElementorKit");
        assert_eq!(kit.system_colors.len(), 4);
        assert_eq!(kit.system_colors[0].id, "primary");
        assert_eq!(kit.system_colors[0].color, "#6A6351");
        assert_eq!(kit.system_colors[1].color, "#3A3633");
        assert_eq!(kit.system_colors[2].color, "#1F1C17");
        assert_eq!(kit.system_colors[3].color, "#B8A98A");
        assert_eq!(kit.custom_colors.len(), 4);
        assert_eq!(kit.custom_colors[3].id, "498af94a");
        assert_eq!(kit.custom_colors[3].color, "#FFF");
        assert_eq!(kit.system_typography[0].font_family, "Fira Sans");
        assert_eq!(kit.system_typography[2].font_family, "Roboto");

        let css = kit.compile_to_css();
        assert!(css.contains("--e-global-color-primary: #6A6351;"));
        assert!(css.contains("--e-global-color-498af94a: #FFF;"));
        assert!(css.contains("--e-global-typography-primary-font-family: Fira Sans;"));
    }
}
