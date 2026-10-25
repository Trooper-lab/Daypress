pub mod autop;
pub mod fonts;
pub mod shortcode;
pub mod slug;

pub use autop::*;
pub use fonts::*;
pub use shortcode::*;
pub use slug::*;

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
        assert_eq!(
            remove_accents("Œuvre & Æther & Straße"),
            "OEuvre & AEther & Strasse"
        );

        assert_eq!(
            sanitize_title_with_dashes("<h1>Café & Crème Brûlée!</h1>"),
            "cafe-creme-brulee"
        );
        assert_eq!(sanitize_title_with_dashes("   ---   "), "");
        assert_eq!(slugify("   ---   "), "n-a");
    }

    #[test]
    fn test_unicode_alphanumeric_retention_no_na_collapse() {
        // Cyrillic
        assert_eq!(slugify("Привет мир"), "привет-мир");
        assert_eq!(sanitize_title_with_dashes("Привет мир"), "привет-мир");

        // Greek
        assert_eq!(slugify("Ελληνικά"), "ελληνικά");

        // Japanese
        assert_eq!(slugify("テスト 記事"), "テスト-記事");

        // Arabic
        assert_eq!(slugify("مرحبا بالعالم"), "مرحبا-بالعالم");

        // Hebrew
        assert_eq!(slugify("שלום עולם"), "שלום-עולם");

        // Symbols only collapse to "n-a"
        assert_eq!(slugify("$#@ %^&"), "n-a");
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
            &[
                ("id", "1"),
                ("layout", "list"),
                ("columns", "3"),
                ("gap", "16px"),
            ],
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

        assert_eq!(
            sanitize_font_family("Inter, sans-serif"),
            "Inter, sans-serif"
        );
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

        let mut camel_map = std::collections::BTreeMap::new();
        camel_map.insert("fontFamily".to_string(), "Inter");
        camel_map.insert("fontStyle".to_string(), "italic");
        camel_map.insert("unicodeRange".to_string(), "U+0-FF");
        let kebab_map = to_kebab_case(camel_map);
        assert_eq!(kebab_map.get("font-family"), Some(&"Inter"));
        assert_eq!(kebab_map.get("font-style"), Some(&"italic"));
        assert_eq!(kebab_map.get("unicode-range"), Some(&"U+0-FF"));
    }
}
