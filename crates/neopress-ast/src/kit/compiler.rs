use super::models::{ButtonStyle, ElementorKit, GlobalColor, GlobalTypography};
use super::php::parse_php_serialized_to_json;

fn is_generic_font(fam: &str) -> bool {
    let lower = fam.to_lowercase();
    matches!(
        lower.as_str(),
        "sans-serif"
            | "serif"
            | "monospace"
            | "cursive"
            | "fantasy"
            | "system-ui"
            | "inherit"
            | "initial"
            | "unset"
            | "arial"
            | "helvetica"
            | "times new roman"
            | "times"
            | "georgia"
            | "courier new"
            | "courier"
            | "verdana"
            | "tahoma"
            | "trebuchet ms"
    )
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

        // 1. Google Fonts @import
        let mut google_fonts = std::collections::BTreeSet::new();
        for t in self
            .system_typography
            .iter()
            .chain(self.custom_typography.iter())
        {
            if let Some(first_fam) = t.font_family.split(',').next() {
                let clean = first_fam.trim().trim_matches('\'').trim_matches('"').trim();
                if !clean.is_empty() && !is_generic_font(clean) {
                    google_fonts.insert(clean.to_string());
                }
            }
        }
        for fam in &google_fonts {
            let fam_encoded = fam.replace(' ', "+");
            css.push_str(&format!(
                "@import url('https://fonts.googleapis.com/css2?family={}:wght@300;400;500;600;700;800;900&display=swap');\n",
                fam_encoded
            ));
        }

        // 2. CSS Custom Properties (:root)
        css.push_str(":root {\n");
        css.push_str(&format!("  --e-content-width: {}px;\n", self.content_width));
        css.push_str(&format!("  --content-width: {}px;\n", self.content_width));
        css.push_str(&format!(
            "  --container-max-width: {}px;\n",
            self.content_width
        ));
        css.push_str(&format!(
            "  --np-container-max-width: {}px;\n",
            self.content_width
        ));
        css.push_str(&format!("  --e-widgets-space: {}px;\n", self.widgets_space));

        // System Colors + Converged Neo (`--np-*`) Aliases
        for c in &self.system_colors {
            css.push_str(&format!("  --e-global-color-{}: {};\n", c.id, c.color));
            css.push_str(&format!("  --np-color-{}: {};\n", c.id, c.color));
        }
        if let Some(sec) = self.system_colors.iter().find(|c| c.id == "secondary") {
            css.push_str(&format!("  --np-color-heading: {};\n", sec.color));
        }
        // Custom Colors + Divi `gcid-*` Aliases
        for c in &self.custom_colors {
            css.push_str(&format!("  --e-global-color-{}: {};\n", c.id, c.color));
            css.push_str(&format!("  --np-color-{}: {};\n", c.id, c.color));
        }

        // System Typography + Converged Neo (`--np-font-*`) Aliases
        for t in &self.system_typography {
            css.push_str(&format!(
                "  --e-global-typography-{}-font-family: {};\n",
                t.id, t.font_family
            ));
            match t.id.as_str() {
                "primary" => css.push_str(&format!("  --np-font-heading: {};\n", t.font_family)),
                "secondary" => {
                    css.push_str(&format!("  --np-font-subheading: {};\n", t.font_family))
                }
                "text" => css.push_str(&format!("  --np-font-body: {};\n", t.font_family)),
                "accent" => css.push_str(&format!("  --np-font-accent: {};\n", t.font_family)),
                _ => {}
            }
            if let Some(ref sz) = t.font_size {
                css.push_str(&format!(
                    "  --e-global-typography-{}-font-size: {};\n",
                    t.id, sz
                ));
            }
            if let Some(ref fw) = t.font_weight {
                css.push_str(&format!(
                    "  --e-global-typography-{}-font-weight: {};\n",
                    t.id, fw
                ));
            }
            if let Some(ref lh) = t.line_height {
                css.push_str(&format!(
                    "  --e-global-typography-{}-line-height: {};\n",
                    t.id, lh
                ));
            }
            if let Some(ref ls) = t.letter_spacing {
                css.push_str(&format!(
                    "  --e-global-typography-{}-letter-spacing: {};\n",
                    t.id, ls
                ));
            }
        }

        // Custom Typography
        for t in &self.custom_typography {
            css.push_str(&format!(
                "  --e-global-typography-{}-font-family: {};\n",
                t.id, t.font_family
            ));
            if let Some(ref sz) = t.font_size {
                css.push_str(&format!(
                    "  --e-global-typography-{}-font-size: {};\n",
                    t.id, sz
                ));
            }
            if let Some(ref fw) = t.font_weight {
                css.push_str(&format!(
                    "  --e-global-typography-{}-font-weight: {};\n",
                    t.id, fw
                ));
            }
            if let Some(ref lh) = t.line_height {
                css.push_str(&format!(
                    "  --e-global-typography-{}-line-height: {};\n",
                    t.id, lh
                ));
            }
            if let Some(ref ls) = t.letter_spacing {
                css.push_str(&format!(
                    "  --e-global-typography-{}-letter-spacing: {};\n",
                    t.id, ls
                ));
            }
        }

        // Buttons Style
        if let Some(ref btn) = self.button_style {
            if let Some(ref bg) = btn.background_color {
                css.push_str(&format!("  --e-button-bg: {};\n", bg));
                css.push_str(&format!("  --np-button-bg: {};\n", bg));
            }
            if let Some(ref tc) = btn.text_color {
                css.push_str(&format!("  --e-button-color: {};\n", tc));
                css.push_str(&format!("  --np-button-color: {};\n", tc));
            }
            if let Some(ref br) = btn.border_radius {
                css.push_str(&format!("  --e-button-border-radius: {};\n", br));
                css.push_str(&format!("  --np-button-radius: {};\n", br));
            }
        }

        css.push_str("}\n\n");

        // 3. Utility Classes for Typography
        for t in self
            .system_typography
            .iter()
            .chain(self.custom_typography.iter())
        {
            css.push_str(&format!(
                ".elementor-typography-{} {{\n  font-family: var(--e-global-typography-{}-font-family);\n",
                t.id, t.id
            ));
            if t.font_size.is_some() {
                css.push_str(&format!(
                    "  font-size: var(--e-global-typography-{}-font-size);\n",
                    t.id
                ));
            }
            if t.font_weight.is_some() {
                css.push_str(&format!(
                    "  font-weight: var(--e-global-typography-{}-font-weight);\n",
                    t.id
                ));
            }
            if t.line_height.is_some() {
                css.push_str(&format!(
                    "  line-height: var(--e-global-typography-{}-line-height);\n",
                    t.id
                ));
            }
            if t.letter_spacing.is_some() {
                css.push_str(&format!(
                    "  letter-spacing: var(--e-global-typography-{}-letter-spacing);\n",
                    t.id
                ));
            }
            css.push_str("}\n");
        }

        // 4. Utility Classes for Colors
        for c in self.system_colors.iter().chain(self.custom_colors.iter()) {
            css.push_str(&format!(
                ".has-e-global-color-{}-color {{ color: var(--e-global-color-{}) !important; }}\n.has-e-global-color-{}-background-color {{ background-color: var(--e-global-color-{}) !important; }}\n",
                c.id, c.id, c.id, c.id
            ));
        }

        // 5. Zero-Specificity Base Element Styles (:where(...))
        css.push_str("\n:where(body.elementor-theme-active, .elementor-kit) {\n  font-family: var(--e-global-typography-text-font-family, inherit);\n  color: var(--e-global-color-text, inherit);\n}\n");
        css.push_str(":where(h1, h2, h3, h4, h5, h6) {\n  font-family: var(--e-global-typography-primary-font-family, inherit);\n  color: var(--e-global-color-secondary, var(--e-global-color-primary, inherit));\n}\n");
        css.push_str(":where(.elementor-button, .np-button) {\n  background-color: var(--e-button-bg, var(--e-global-color-accent, #3b82f6));\n  color: var(--e-button-color, #ffffff);\n  border-radius: var(--e-button-border-radius, 4px);\n}\n");

        if let Some(ref custom_css) = self.custom_css {
            if !custom_css.trim().is_empty() {
                css.push_str("\n/* Elementor Custom CSS */\n");
                css.push_str(custom_css);
                css.push('\n');
            }
        }

        css
    }

    /// Synthesizes a converged `ElementorKit` (Neo Styling System) from Divi global colors (`gcid-*`),
    /// enqueued Google Fonts (`et_enqueued_post_fonts`), custom CSS, and detected brand palette.
    #[allow(clippy::too_many_arguments)]
    pub fn from_divi_theme(
        primary_color: &str,
        secondary_color: &str,
        text_color: &str,
        accent_color: &str,
        heading_font: &str,
        body_font: &str,
        custom_colors: Vec<GlobalColor>,
        site_name: Option<String>,
        site_description: Option<String>,
        site_logo: Option<String>,
        site_favicon: Option<String>,
    ) -> Self {
        Self {
            system_colors: vec![
                GlobalColor {
                    id: "primary".to_string(),
                    title: "Primary (Divi Brand)".to_string(),
                    color: primary_color.to_string(),
                },
                GlobalColor {
                    id: "secondary".to_string(),
                    title: "Secondary (Divi Dark/Heading)".to_string(),
                    color: secondary_color.to_string(),
                },
                GlobalColor {
                    id: "text".to_string(),
                    title: "Body Text".to_string(),
                    color: text_color.to_string(),
                },
                GlobalColor {
                    id: "accent".to_string(),
                    title: "Accent / CTA".to_string(),
                    color: accent_color.to_string(),
                },
            ],
            custom_colors,
            system_typography: vec![
                GlobalTypography {
                    id: "primary".to_string(),
                    title: "Primary Heading".to_string(),
                    font_family: format!("{}, sans-serif", heading_font),
                    font_size: Some("40px".to_string()),
                    font_weight: Some("700".to_string()),
                    line_height: Some("1.2em".to_string()),
                    letter_spacing: None,
                },
                GlobalTypography {
                    id: "secondary".to_string(),
                    title: "Secondary Heading".to_string(),
                    font_family: format!("{}, sans-serif", heading_font),
                    font_size: Some("28px".to_string()),
                    font_weight: Some("600".to_string()),
                    line_height: Some("1.3em".to_string()),
                    letter_spacing: None,
                },
                GlobalTypography {
                    id: "text".to_string(),
                    title: "Body Text".to_string(),
                    font_family: format!("{}, sans-serif", body_font),
                    font_size: Some("16px".to_string()),
                    font_weight: Some("400".to_string()),
                    line_height: Some("1.7em".to_string()),
                    letter_spacing: None,
                },
                GlobalTypography {
                    id: "accent".to_string(),
                    title: "Button / Accent".to_string(),
                    font_family: format!("{}, sans-serif", body_font),
                    font_size: Some("15px".to_string()),
                    font_weight: Some("600".to_string()),
                    line_height: Some("1.4em".to_string()),
                    letter_spacing: None,
                },
            ],
            custom_typography: Vec::new(),
            content_width: 1080,
            widgets_space: 24,
            site_name,
            site_description,
            site_logo,
            site_favicon,
            button_style: Some(ButtonStyle {
                background_color: Some(primary_color.to_string()),
                text_color: Some("#ffffff".to_string()),
                border_radius: Some("6px".to_string()),
            }),
            custom_css: None,
        }
    }
}
