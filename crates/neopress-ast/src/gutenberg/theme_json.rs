use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJson {
    #[serde(default)]
    pub version: u8,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub settings: Option<ThemeJsonSettings>,
    #[serde(default)]
    pub styles: Option<ThemeJsonStyles>,
    #[serde(default, alias = "customTemplates")]
    pub custom_templates: Option<Vec<ThemeJsonCustomTemplate>>,
    #[serde(default, alias = "templateParts")]
    pub template_parts: Option<Vec<ThemeJsonTemplatePart>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonSettings {
    #[serde(default)]
    pub color: Option<ThemeJsonColorSettings>,
    #[serde(default)]
    pub typography: Option<ThemeJsonTypographySettings>,
    #[serde(default)]
    pub spacing: Option<ThemeJsonSpacingSettings>,
    #[serde(default)]
    pub layout: Option<ThemeJsonLayoutSettings>,
    #[serde(default)]
    pub shadow: Option<ThemeJsonShadowSettings>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonColorSettings {
    #[serde(default)]
    pub palette: Option<Vec<ThemeJsonPaletteItem>>,
    #[serde(default)]
    pub gradients: Option<Vec<ThemeJsonGradientItem>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonPaletteItem {
    pub slug: String,
    pub color: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonGradientItem {
    pub slug: String,
    pub gradient: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonTypographySettings {
    #[serde(default, alias = "fontSizes")]
    pub font_sizes: Option<Vec<ThemeJsonFontSizeItem>>,
    #[serde(default, alias = "fontFamilies")]
    pub font_families: Option<Vec<ThemeJsonFontFamilyItem>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonFontSizeItem {
    pub slug: String,
    pub size: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonFontFamilyItem {
    pub slug: String,
    #[serde(alias = "fontFamily")]
    pub font_family: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonSpacingSettings {
    #[serde(default, alias = "spacingSizes")]
    pub spacing_sizes: Option<Vec<ThemeJsonSpacingSizeItem>>,
    #[serde(default)]
    pub units: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonSpacingSizeItem {
    pub slug: String,
    pub size: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonLayoutSettings {
    #[serde(default, alias = "contentSize")]
    pub content_size: Option<String>,
    #[serde(default, alias = "wideSize")]
    pub wide_size: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonShadowSettings {
    #[serde(default)]
    pub presets: Option<Vec<ThemeJsonShadowPreset>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonShadowPreset {
    pub slug: String,
    pub shadow: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonStyles {
    #[serde(default)]
    pub color: Option<ThemeJsonStyleColor>,
    #[serde(default)]
    pub typography: Option<ThemeJsonStyleTypography>,
    #[serde(default)]
    pub spacing: Option<ThemeJsonStyleSpacing>,
    #[serde(default)]
    pub css: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonStyleColor {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonStyleTypography {
    #[serde(default, alias = "fontSize")]
    pub font_size: Option<String>,
    #[serde(default, alias = "fontFamily")]
    pub font_family: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonStyleSpacing {
    #[serde(default)]
    pub padding: Option<serde_json::Value>,
    #[serde(default)]
    pub margin: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonCustomTemplate {
    pub name: String,
    pub title: String,
    #[serde(default, alias = "postTypes")]
    pub post_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ThemeJsonTemplatePart {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub area: Option<String>,
}

impl ThemeJson {
    pub fn compile_to_css(&self) -> String {
        let mut root_vars = Vec::new();
        let mut utility_classes = Vec::new();

        if let Some(settings) = &self.settings {
            if let Some(color) = &settings.color {
                if let Some(palette) = &color.palette {
                    for item in palette {
                        root_vars.push(format!("  --wp--preset--color--{}: {};", item.slug, item.color));
                        utility_classes.push(format!(
                            ".has-{slug}-color {{ color: var(--wp--preset--color--{slug}) !important; }}\n\
                             .has-{slug}-background-color {{ background-color: var(--wp--preset--color--{slug}) !important; }}\n\
                             .has-{slug}-border-color {{ border-color: var(--wp--preset--color--{slug}) !important; }}",
                            slug = item.slug
                        ));
                    }
                }
                if let Some(gradients) = &color.gradients {
                    for item in gradients {
                        root_vars.push(format!("  --wp--preset--gradient--{}: {};", item.slug, item.gradient));
                        utility_classes.push(format!(
                            ".has-{slug}-gradient-background {{ background: var(--wp--preset--gradient--{slug}) !important; }}",
                            slug = item.slug
                        ));
                    }
                }
            }

            if let Some(typo) = &settings.typography {
                if let Some(sizes) = &typo.font_sizes {
                    for item in sizes {
                        root_vars.push(format!("  --wp--preset--font-size--{}: {};", item.slug, item.size));
                        utility_classes.push(format!(
                            ".has-{slug}-font-size {{ font-size: var(--wp--preset--font-size--{slug}) !important; }}",
                            slug = item.slug
                        ));
                    }
                }
                if let Some(families) = &typo.font_families {
                    for item in families {
                        root_vars.push(format!("  --wp--preset--font-family--{}: {};", item.slug, item.font_family));
                        utility_classes.push(format!(
                            ".has-{slug}-font-family {{ font-family: var(--wp--preset--font-family--{slug}) !important; }}",
                            slug = item.slug
                        ));
                    }
                }
            }

            if let Some(spacing) = &settings.spacing {
                if let Some(sizes) = &spacing.spacing_sizes {
                    for item in sizes {
                        root_vars.push(format!("  --wp--preset--spacing--{}: {};", item.slug, item.size));
                    }
                }
            }

            if let Some(shadow) = &settings.shadow {
                if let Some(presets) = &shadow.presets {
                    for item in presets {
                        root_vars.push(format!("  --wp--preset--shadow--{}: {};", item.slug, item.shadow));
                    }
                }
            }

            if let Some(layout) = &settings.layout {
                if let Some(content) = &layout.content_size {
                    root_vars.push(format!("  --wp--style--global--content-size: {};", content));
                }
                if let Some(wide) = &layout.wide_size {
                    root_vars.push(format!("  --wp--style--global--wide-size: {};", wide));
                }
            }
        }

        let mut out = String::new();
        if !root_vars.is_empty() {
            out.push_str(":root {\n");
            out.push_str(&root_vars.join("\n"));
            out.push_str("\n}\n\n");
        }

        out.push_str(
            "body .is-layout-constrained > :where(:not(.alignleft):not(.alignright):not(.alignfull)) {\n\
             \x20 max-width: var(--wp--style--global--content-size, 800px);\n\
             \x20 margin-left: auto !important;\n\
             \x20 margin-right: auto !important;\n\
             }\n\
             body .is-layout-constrained > .alignwide {\n\
             \x20 max-width: var(--wp--style--global--wide-size, 1200px);\n\
             \x20 margin-left: auto !important;\n\
             \x20 margin-right: auto !important;\n\
             }\n\
             body .is-layout-constrained > .alignfull {\n\
             \x20 max-width: none;\n\
             }\n\n"
        );

        if !utility_classes.is_empty() {
            out.push_str(&utility_classes.join("\n"));
            out.push_str("\n\n");
        }

        if let Some(styles) = &self.styles {
            let mut body_rules = Vec::new();
            if let Some(color) = &styles.color {
                if let Some(bg) = &color.background {
                    body_rules.push(format!("background-color: {};", bg));
                }
                if let Some(txt) = &color.text {
                    body_rules.push(format!("color: {};", txt));
                }
            }
            if let Some(typo) = &styles.typography {
                if let Some(fs) = &typo.font_size {
                    body_rules.push(format!("font-size: {};", fs));
                }
                if let Some(ff) = &typo.font_family {
                    body_rules.push(format!("font-family: {};", ff));
                }
            }
            if !body_rules.is_empty() {
                out.push_str("body {\n  ");
                out.push_str(&body_rules.join("\n  "));
                out.push_str("\n}\n\n");
            }
            if let Some(custom_css) = &styles.css {
                out.push_str(custom_css);
                out.push('\n');
            }
        }

        out
    }
}

pub fn parse_theme_json(json_str: &str) -> Result<ThemeJson, serde_json::Error> {
    serde_json::from_str(json_str)
}

pub fn compile_theme_json_to_css(json_str: &str) -> Result<String, serde_json::Error> {
    let theme_json = parse_theme_json(json_str)?;
    Ok(theme_json.compile_to_css())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_json_schema_v2_compilation() {
        let json_data = r##"{
            "version": 2,
            "settings": {
                "color": {
                    "palette": [
                        { "slug": "primary", "color": "#2563eb", "name": "Primary" },
                        { "slug": "secondary", "color": "#f59e0b", "name": "Secondary" }
                    ]
                },
                "typography": {
                    "fontSizes": [
                        { "slug": "base", "size": "16px", "name": "Base" }
                    ]
                },
                "layout": {
                    "contentSize": "840px",
                    "wideSize": "1280px"
                }
            },
            "styles": {
                "color": {
                    "background": "#ffffff",
                    "text": "#111827"
                }
            }
        }"##;

        let css = compile_theme_json_to_css(json_data).expect("valid theme.json");
        assert!(css.contains("--wp--preset--color--primary: #2563eb;"));
        assert!(css.contains(".has-primary-color { color: var(--wp--preset--color--primary) !important; }"));
        assert!(css.contains(".has-primary-background-color { background-color: var(--wp--preset--color--primary) !important; }"));
        assert!(css.contains("--wp--preset--font-size--base: 16px;"));
        assert!(css.contains("--wp--style--global--content-size: 840px;"));
        assert!(css.contains("--wp--style--global--wide-size: 1280px;"));
        assert!(css.contains(".is-layout-constrained"));
        assert!(css.contains("background-color: #ffffff;"));
    }
}
