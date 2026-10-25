use regex::Regex;
use std::collections::{BTreeMap, HashMap};

/// Native Rust port of WordPress 6.5+ Font Library (`fixtures/wp-includes/fonts/class-wp-font-*.php`):
/// - `WP_Font_Utils::sanitize_font_family`
/// - `WP_Font_Utils::get_allowed_font_mime_types`
/// - `WP_Font_Utils::get_font_face_slug`
/// - `WP_Font_Face::generate_css`
/// - `WP_Font_Face_Resolver::to_kebab_case`
pub mod wp_font_library {
    use super::*;

    /// Equivalent to WordPress `sanitize_text_field()`: strips HTML tags, normalizes whitespace, and trims.
    pub fn sanitize_text_field(input: &str) -> String {
        let re_tags = Regex::new(r"<[^>]*>").expect("valid tag regex");
        let without_tags = re_tags.replace_all(input, "");
        let re_ws = Regex::new(r"[\r\n\t ]+").expect("valid ws regex");
        re_ws.replace_all(without_tags.trim(), " ").into_owned()
    }

    fn maybe_add_quotes(item: &str) -> String {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        let is_exclusively_alpha_or_dash =
            trimmed.chars().all(|c| c.is_ascii_alphabetic() || c == '-');

        let is_generic_wrapper = trimmed
            .strip_prefix("generic(")
            .and_then(|s| s.strip_suffix(')'))
            .map(|inner| inner.chars().all(|c| c.is_ascii_alphabetic() || c == '-'))
            .unwrap_or(false);

        if is_exclusively_alpha_or_dash || is_generic_wrapper {
            trimmed.to_string()
        } else {
            let unquoted = trimmed.trim_matches(|c| c == '"' || c == '\'');
            format!("\"{}\"", unquoted)
        }
    }

    /// Native Rust port of `WP_Font_Utils::sanitize_font_family`.
    pub fn sanitize_font_family(font_family: &str) -> String {
        if font_family.is_empty() {
            return String::new();
        }
        let output = sanitize_text_field(font_family);
        if output.is_empty() {
            return String::new();
        }
        if output.contains(',') {
            let formatted_items: Vec<String> = output
                .split(',')
                .map(maybe_add_quotes)
                .filter(|s| !s.is_empty())
                .collect();
            formatted_items.join(", ")
        } else {
            maybe_add_quotes(&output)
        }
    }

    /// Native Rust port of `WP_Font_Utils::get_allowed_font_mime_types` (PHP 8.1.12+ / 8.3).
    pub fn get_allowed_font_mime_types() -> BTreeMap<String, String> {
        let mut mimes = BTreeMap::new();
        mimes.insert("otf".to_string(), "application/vnd.ms-opentype".to_string());
        mimes.insert("ttf".to_string(), "font/sfnt".to_string());
        mimes.insert("woff".to_string(), "font/woff".to_string());
        mimes.insert("woff2".to_string(), "font/woff2".to_string());
        mimes
    }

    /// Native Rust port of `WP_Font_Utils::get_font_face_slug`.
    pub fn get_font_face_slug(
        font_family: &str,
        font_style: Option<&str>,
        font_weight: Option<&str>,
        font_stretch: Option<&str>,
        unicode_range: Option<&str>,
    ) -> String {
        let family = font_family.to_lowercase();
        let style = font_style.unwrap_or("normal").to_lowercase();
        let weight = font_weight
            .unwrap_or("400")
            .to_lowercase()
            .replace("normal", "400")
            .replace("bold", "700");

        let mut stretch = font_stretch.unwrap_or("100%").to_lowercase();
        let stretch_map = [
            ("ultra-condensed", "50%"),
            ("extra-condensed", "62.5%"),
            ("semi-condensed", "87.5%"),
            ("condensed", "75%"),
            ("normal", "100%"),
            ("semi-expanded", "112.5%"),
            ("extra-expanded", "150%"),
            ("ultra-expanded", "200%"),
            ("expanded", "125%"),
        ];
        for (k, v) in stretch_map {
            stretch = stretch.replace(k, v);
        }

        let range = unicode_range.unwrap_or("U+0-10FFFF").to_uppercase();
        let re_comma_ws = Regex::new(r",\s+").expect("valid comma ws regex");

        let elements: Vec<String> = [family, style, weight, stretch, range]
            .into_iter()
            .map(|elem| {
                let cleaned = elem.replace(['"', '\'', ';'], "").trim().to_string();
                re_comma_ws.replace_all(&cleaned, ",").into_owned()
            })
            .collect();

        sanitize_text_field(&elements.join(";"))
    }

    /// Font face declaration for `WP_Font_Face::generate_css`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FontFaceDeclaration {
        pub font_family: String,
        pub src: Vec<String>,
        pub font_style: String,
        pub font_weight: String,
        pub font_display: String,
        pub font_stretch: Option<String>,
        pub unicode_range: Option<String>,
    }

    impl FontFaceDeclaration {
        pub fn new(font_family: impl Into<String>, src: Vec<String>) -> Self {
            Self {
                font_family: font_family.into(),
                src,
                font_style: "normal".to_string(),
                font_weight: "400".to_string(),
                font_display: "fallback".to_string(),
                font_stretch: None,
                unicode_range: None,
            }
        }
    }

    /// Native Rust port of `WP_Font_Face` CSS compilation (`order_src`, `build_font_face_css`, `compile_src`).
    pub fn generate_css(font_faces: &[FontFaceDeclaration]) -> String {
        let valid_displays = ["auto", "block", "fallback", "swap", "optional"];
        let mut blocks = Vec::new();

        for face in font_faces {
            if face.font_family.trim().is_empty() || face.src.is_empty() {
                continue;
            }
            if face.src.iter().any(|s| s.trim().is_empty()) {
                continue;
            }

            let mut ordered_src: Vec<(String, &'static str)> = Vec::new();
            let mut by_ext: HashMap<String, String> = HashMap::new();

            for url in &face.src {
                let trimmed_url = url.trim();
                if trimmed_url.starts_with("data:") {
                    ordered_src.push((url.clone(), "data"));
                    continue;
                }
                let ext = trimmed_url.rsplit('.').next().unwrap_or("").to_lowercase();
                by_ext.insert(ext, url.clone());
            }

            if let Some(u) = by_ext.get("woff2") {
                ordered_src.push((u.clone(), "woff2"));
            }
            if let Some(u) = by_ext.get("woff") {
                ordered_src.push((u.clone(), "woff"));
            }
            if let Some(u) = by_ext.get("ttf") {
                ordered_src.push((u.clone(), "truetype"));
            }
            if let Some(u) = by_ext.get("eot") {
                ordered_src.push((u.clone(), "embedded-opentype"));
            }
            if let Some(u) = by_ext.get("otf") {
                ordered_src.push((u.clone(), "opentype"));
            }

            if ordered_src.is_empty() {
                continue;
            }

            let compiled_src = ordered_src
                .iter()
                .map(|(url, fmt)| {
                    if *fmt == "data" {
                        format!("url({})", url)
                    } else {
                        format!("url('{}') format('{}')", url, fmt)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let family = if face.font_family.contains(' ')
                && !face.font_family.contains('"')
                && !face.font_family.contains('\'')
            {
                format!("\"{}\"", face.font_family)
            } else {
                face.font_family.clone()
            };

            let display = if valid_displays.contains(&face.font_display.as_str()) {
                face.font_display.as_str()
            } else {
                "fallback"
            };

            let mut decls = format!(
                "font-family:{};font-style:{};font-weight:{};font-display:{};src:{};",
                family, face.font_style, face.font_weight, display, compiled_src
            );

            if let Some(ref stretch) = face.font_stretch {
                if !stretch.is_empty() {
                    decls.push_str(&format!("font-stretch:{};", stretch));
                }
            }
            if let Some(ref ur) = face.unicode_range {
                if !ur.is_empty() {
                    decls.push_str(&format!("unicode-range:{};", ur));
                }
            }

            blocks.push(format!("@font-face{{{}}}", decls));
        }

        blocks.join("\n")
    }

    /// Converts a camelCase or PascalCase string into kebab-case (`_wp_to_kebab_case`).
    pub fn to_kebab_case_str(input: &str) -> String {
        let mut out = String::with_capacity(input.len() + 4);
        for (i, ch) in input.chars().enumerate() {
            if ch.is_ascii_uppercase() {
                if i > 0 {
                    out.push('-');
                }
                out.push(ch.to_ascii_lowercase());
            } else if ch == '_' || ch == ' ' {
                out.push('-');
            } else {
                out.push(ch);
            }
        }
        out
    }

    /// Native Rust port of `WP_Font_Face_Resolver::to_kebab_case`:
    /// Converts all first-dimension map keys into kebab-case.
    pub fn to_kebab_case<V>(data: BTreeMap<String, V>) -> BTreeMap<String, V> {
        let mut out = BTreeMap::new();
        for (k, v) in data {
            out.insert(to_kebab_case_str(&k), v);
        }
        out
    }
}
