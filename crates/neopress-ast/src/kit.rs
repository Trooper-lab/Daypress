use serde::{Deserialize, Serialize};

/// Represents a single global color token translated from Elementor's Global_Colors kit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalColor {
    pub id: String,
    pub title: String,
    pub color: String,
}

/// Represents a typography token translated from Elementor's Global_Typography kit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalTypography {
    pub id: String,
    pub title: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default)]
    pub font_size: Option<String>,
    #[serde(default)]
    pub font_weight: Option<String>,
    #[serde(default)]
    pub line_height: Option<String>,
    #[serde(default)]
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
    #[serde(default = "default_content_width")]
    pub content_width: u32,
    #[serde(default = "default_widgets_space")]
    pub widgets_space: u32,
    #[serde(default)]
    pub site_name: Option<String>,
    #[serde(default)]
    pub site_description: Option<String>,
    #[serde(default)]
    pub site_logo: Option<String>,
    #[serde(default)]
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
}
