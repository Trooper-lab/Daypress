use regex::Regex;
use std::collections::{BTreeMap, HashMap};

/// Transliterates Latin-1 Supplement and Latin Extended-A/B accented characters
/// into their ASCII equivalents, matching WordPress `remove_accents()` and
/// PHP `iconv('utf-8', 'us-ascii//TRANSLIT')`.
pub fn remove_accents(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            // Latin-1 Supplement Uppercase
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => out.push('A'),
            'Æ' => out.push_str("AE"),
            'Ç' => out.push('C'),
            'È' | 'É' | 'Ê' | 'Ë' => out.push('E'),
            'Ì' | 'Í' | 'Î' | 'Ï' => out.push('I'),
            'Ð' => out.push('D'),
            'Ñ' => out.push('N'),
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => out.push('O'),
            'Ù' | 'Ú' | 'Û' | 'Ü' => out.push('U'),
            'Ý' => out.push('Y'),
            'Þ' => out.push_str("TH"),
            'ß' => out.push_str("ss"),

            // Latin-1 Supplement Lowercase
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => out.push('a'),
            'æ' => out.push_str("ae"),
            'ç' => out.push('c'),
            'è' | 'é' | 'ê' | 'ë' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' => out.push('i'),
            'ð' => out.push('d'),
            'ñ' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => out.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            'þ' => out.push_str("th"),

            // Latin Extended-A & Extended-B
            'Ā' | 'Ă' | 'Ą' => out.push('A'),
            'ā' | 'ă' | 'ą' => out.push('a'),
            'Ć' | 'Ĉ' | 'Ċ' | 'Č' => out.push('C'),
            'ć' | 'ĉ' | 'ċ' | 'č' => out.push('c'),
            'Ď' | 'Đ' => out.push('D'),
            'ď' | 'đ' => out.push('d'),
            'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => out.push('E'),
            'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => out.push('e'),
            'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' => out.push('G'),
            'ĝ' | 'ğ' | 'ġ' | 'ģ' => out.push('g'),
            'Ĥ' | 'Ħ' => out.push('H'),
            'ĥ' | 'ħ' => out.push('h'),
            'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' => out.push('I'),
            'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => out.push('i'),
            'Ĳ' => out.push_str("IJ"),
            'ĳ' => out.push_str("ij"),
            'Ĵ' => out.push('J'),
            'ĵ' => out.push('j'),
            'Ķ' => out.push('K'),
            'ķ' | 'ĸ' => out.push('k'),
            'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' => out.push('L'),
            'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => out.push('l'),
            'Ń' | 'Ņ' | 'Ň' | 'Ŋ' => out.push('N'),
            'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => out.push('n'),
            'Ō' | 'Ŏ' | 'Ő' => out.push('O'),
            'ō' | 'ŏ' | 'ő' => out.push('o'),
            'Œ' => out.push_str("OE"),
            'œ' => out.push_str("oe"),
            'Ŕ' | 'Ŗ' | 'Ř' => out.push('R'),
            'ŕ' | 'ŗ' | 'ř' => out.push('r'),
            'Ś' | 'Ŝ' | 'Ş' | 'Š' | 'Ș' => out.push('S'),
            'ś' | 'ŝ' | 'ş' | 'š' | 'ș' | 'ſ' => out.push('s'),
            'Ţ' | 'Ť' | 'Ŧ' | 'Ț' => out.push('T'),
            'ţ' | 'ť' | 'ŧ' | 'ț' => out.push('t'),
            'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' => out.push('U'),
            'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => out.push('u'),
            'Ŵ' => out.push('W'),
            'ŵ' => out.push('w'),
            'Ŷ' | 'Ÿ' => out.push('Y'),
            'ŷ' => out.push('y'),
            'Ź' | 'Ż' | 'Ž' => out.push('Z'),
            'ź' | 'ż' | 'ž' => out.push('z'),

            // Strip combining diacritical marks
            '\u{0300}'..='\u{036F}' => {}

            other => out.push(other),
        }
    }
    out
}

/// WordPress `sanitize_title_with_dashes()` port:
/// Strips HTML tags, transliterates accents via `remove_accents()`, replaces
/// non-alphanumeric sequences with `-`, trims leading/trailing `-`, and lowercases.
pub fn sanitize_title_with_dashes(title: &str) -> String {
    let re_tags = Regex::new(r"<[^>]*>").expect("valid html tag regex");
    let stripped = re_tags.replace_all(title, "");
    let unaccented = remove_accents(&stripped);
    let re_non_alnum = Regex::new(r"[^a-zA-Z0-9]+").expect("valid alnum regex");
    let replaced = re_non_alnum.replace_all(&unaccented, "-");
    replaced.trim_matches('-').to_lowercase()
}

/// Transpiled & Golden-Master verified `slugify()` matching `fixtures/slugify.php`.
pub fn slugify(text: &str) -> String {
    let unaccented = remove_accents(text);
    let re_non_alnum = Regex::new(r"[^a-zA-Z0-9]+").expect("valid alnum regex");
    let replaced = re_non_alnum.replace_all(&unaccented, "-");
    let trimmed = replaced.trim_matches('-');
    let lowered = trimmed.to_lowercase();
    if lowered.is_empty() {
        "n-a".to_string()
    } else {
        lowered
    }
}

/// WordPress `wpautop()` port:
/// Converts double line-breaks into `<p>...</p>` paragraphs and optional single line-breaks
/// into `<br />\n`, while preserving block-level HTML elements (`<div>`, `<section>`, `<pre>`,
/// `<script>`, `<style>`, `<table>`, `<ul>`, `<ol>`, `<blockquote>`, `<h1>`..`<h6>`).
pub fn wpautop(pee: &str, br: bool) -> String {
    if pee.trim().is_empty() {
        return String::new();
    }

    let normalized = pee.replace("\r\n", "\n").replace('\r', "\n");
    let re_double_nl = Regex::new(r"\n\s*\n+").expect("valid paragraph split regex");

    let block_prefixes = [
        "<div",
        "</div",
        "<section",
        "</section",
        "<pre",
        "</pre",
        "<script",
        "</script",
        "<style",
        "</style",
        "<table",
        "</table",
        "<ul",
        "</ul",
        "<ol",
        "</ol",
        "<blockquote",
        "</blockquote",
        "<h1",
        "<h2",
        "<h3",
        "<h4",
        "<h5",
        "<h6",
        "<p>",
        "<p ",
        "<header",
        "<footer",
        "<nav",
        "<article",
        "<aside",
        "<figure",
        "<form",
        "<hr",
        "<!--",
    ];

    let mut paragraphs = Vec::new();
    for chunk in re_double_nl.split(normalized.trim()) {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();
        let is_block = block_prefixes.iter().any(|p| lower.starts_with(p));

        if is_block {
            paragraphs.push(trimmed.to_string());
        } else if br {
            let with_br = trimmed.replace('\n', "<br />\n");
            paragraphs.push(format!("<p>{}</p>", with_br));
        } else {
            paragraphs.push(format!("<p>{}</p>", trimmed));
        }
    }

    paragraphs.join("\n")
}

/// WordPress `shortcode_parse_atts()` port:
/// Parses shortcode attribute strings (`foo="bar" baz='qux' id=42`) into a key-value map.
pub fn shortcode_parse_atts(text: &str) -> HashMap<String, String> {
    let mut atts = HashMap::new();
    let re = Regex::new(
        r#"([a-zA-Z0-9_-]+)\s*=\s*"([^"]*)"|([a-zA-Z0-9_-]+)\s*=\s*'([^']*)'|([a-zA-Z0-9_-]+)\s*=\s*([^\s"']+)"#,
    )
    .expect("valid shortcode_parse_atts regex");

    for cap in re.captures_iter(text) {
        if let (Some(k), Some(v)) = (cap.get(1), cap.get(2)) {
            atts.insert(k.as_str().to_lowercase(), v.as_str().to_string());
        } else if let (Some(k), Some(v)) = (cap.get(3), cap.get(4)) {
            atts.insert(k.as_str().to_lowercase(), v.as_str().to_string());
        } else if let (Some(k), Some(v)) = (cap.get(5), cap.get(6)) {
            atts.insert(k.as_str().to_lowercase(), v.as_str().to_string());
        }
    }

    atts
}

/// WordPress `shortcode_atts()` port:
/// Combines user shortcode attributes with known default key-value pairs, ignoring unknown keys.
pub fn shortcode_atts(
    defaults: &[(&str, &str)],
    atts: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut out = HashMap::with_capacity(defaults.len());
    for &(key, default_val) in defaults {
        if let Some(val) = atts
            .get(key)
            .or_else(|| atts.get(&key.to_lowercase()))
        {
            out.insert(key.to_string(), val.clone());
        } else {
            out.insert(key.to_string(), default_val.to_string());
        }
    }
    out
}

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
        let is_exclusively_alpha_or_dash = trimmed
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '-');
        let is_generic_wrapper = trimmed.starts_with("generic(")
            && trimmed.ends_with(')')
            && trimmed.len() > 9
            && trimmed[8..trimmed.len() - 1]
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c == '-');

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
                let cleaned = elem
                    .replace(['"', '\'', ';'], "")
                    .trim()
                    .to_string();
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

            // Order `src` items for browser support matching `WP_Font_Face::order_src`
            let mut ordered_src: Vec<(String, &'static str)> = Vec::new();
            let mut by_ext: HashMap<String, String> = HashMap::new();

            for url in &face.src {
                let trimmed_url = url.trim();
                if trimmed_url.starts_with("data:") {
                    ordered_src.push((url.clone(), "data"));
                    continue;
                }
                let ext = trimmed_url
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fs;

    #[derive(Debug, Deserialize)]
    struct GoldenRecord {
        input: String,
        expected_output: String,
    }

    #[test]
    fn test_slugify_against_golden_master() {
        let paths = [
            "golden_master.json",
            concat!(env!("CARGO_MANIFEST_DIR"), "/golden_master.json"),
        ];

        let mut content = None;
        for p in paths {
            if let Ok(c) = fs::read_to_string(p) {
                content = Some(c);
                break;
            }
        }

        let raw = content.expect("Could not find golden_master.json");
        let records: Vec<GoldenRecord> =
            serde_json::from_str(&raw).expect("Failed to deserialize golden_master.json");

        assert!(
            !records.is_empty(),
            "Golden master records must not be empty"
        );

        for record in records {
            let actual = slugify(&record.input);
            assert_eq!(
                actual, record.expected_output,
                "Golden Master mismatch for input: '{}'",
                record.input
            );
        }
    }

    #[test]
    fn test_remove_accents_and_sanitize_title() {
        assert_eq!(remove_accents("Café & Crème Brûlée"), "Cafe & Creme Brulee");
        assert_eq!(remove_accents("Übermensch"), "Ubermensch");
        assert_eq!(remove_accents("Español & Niño"), "Espanol & Nino");
        assert_eq!(remove_accents("Łódź & Gdańsk"), "Lodz & Gdansk");
        assert_eq!(remove_accents("Œuvre & Æther & Straße"), "OEuvre & AEther & Strasse");

        assert_eq!(
            sanitize_title_with_dashes("<h1>Café & Crème Brûlée!</h1>"),
            "cafe-creme-brulee"
        );
        assert_eq!(sanitize_title_with_dashes("   ---   "), "");
        assert_eq!(slugify("   ---   "), "n-a");
    }

    #[test]
    fn test_wpautop_paragraphs_and_blocks() {
        let input = "First paragraph\nwith single break.\n\nSecond paragraph.\n\n<div class=\"hero\">Inside div</div>";
        let output = wpautop(input, true);
        assert_eq!(
            output,
            "<p>First paragraph<br />\nwith single break.</p>\n<p>Second paragraph.</p>\n<div class=\"hero\">Inside div</div>"
        );

        let no_br = wpautop("Line 1\nLine 2", false);
        assert_eq!(no_br, "<p>Line 1\nLine 2</p>");
        assert_eq!(wpautop("   ", true), "");
    }

    #[test]
    fn test_shortcode_parse_and_atts() {
        let raw = r#"id="123" layout='grid' columns=4 extra="ignored""#;
        let parsed = shortcode_parse_atts(raw);
        assert_eq!(parsed.get("id").map(String::as_str), Some("123"));
        assert_eq!(parsed.get("layout").map(String::as_str), Some("grid"));
        assert_eq!(parsed.get("columns").map(String::as_str), Some("4"));

        let merged = shortcode_atts(
            &[("id", "1"), ("layout", "list"), ("columns", "3"), ("gap", "16px")],
            &parsed,
        );
        assert_eq!(merged.get("id").map(String::as_str), Some("123"));
        assert_eq!(merged.get("layout").map(String::as_str), Some("grid"));
        assert_eq!(merged.get("columns").map(String::as_str), Some("4"));
        assert_eq!(merged.get("gap").map(String::as_str), Some("16px"));
        assert!(!merged.contains_key("extra"));
    }

    #[test]
    fn test_wp_font_library_parity() {
        use wp_font_library::*;

        assert_eq!(sanitize_font_family("Inter, sans-serif"), "Inter, sans-serif");
        assert_eq!(
            sanitize_font_family("Open Sans, generic(sans-serif)"),
            "\"Open Sans\", generic(sans-serif)"
        );
        assert_eq!(sanitize_font_family("Roboto Mono"), "\"Roboto Mono\"");
        assert_eq!(
            sanitize_font_family("\"Playfair Display\", serif"),
            "\"Playfair Display\", serif"
        );
        assert_eq!(sanitize_font_family(""), "");

        let mimes = get_allowed_font_mime_types();
        assert_eq!(mimes.get("woff2").map(String::as_str), Some("font/woff2"));
        assert_eq!(mimes.get("ttf").map(String::as_str), Some("font/sfnt"));

        let slug = get_font_face_slug(
            "\"Open Sans\"",
            Some("Normal"),
            Some("bold"),
            Some("condensed"),
            None,
        );
        assert_eq!(slug, "open sans;normal;700;75%;U+0-10FFFF");

        let mut face = FontFaceDeclaration::new(
            "Open Sans",
            vec![
                "https://example.org/fonts/open-sans.ttf".to_string(),
                "https://example.org/fonts/open-sans.woff2".to_string(),
                "https://example.org/fonts/open-sans.woff".to_string(),
            ],
        );
        face.font_weight = "700".to_string();
        face.font_display = "swap".to_string();

        let css = generate_css(&[face]);
        assert_eq!(
            css,
            "@font-face{font-family:\"Open Sans\";font-style:normal;font-weight:700;font-display:swap;src:url('https://example.org/fonts/open-sans.woff2') format('woff2'), url('https://example.org/fonts/open-sans.woff') format('woff'), url('https://example.org/fonts/open-sans.ttf') format('truetype');}"
        );

        let mut camel_map = BTreeMap::new();
        camel_map.insert("fontFamily".to_string(), "Inter");
        camel_map.insert("fontStyle".to_string(), "italic");
        camel_map.insert("unicodeRange".to_string(), "U+0-FF");
        let kebab_map = to_kebab_case(camel_map);
        assert_eq!(kebab_map.get("font-family"), Some(&"Inter"));
        assert_eq!(kebab_map.get("font-style"), Some(&"italic"));
        assert_eq!(kebab_map.get("unicode-range"), Some(&"U+0-FF"));
    }
}