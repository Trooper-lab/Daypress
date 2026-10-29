use serde::{Deserialize, Serialize};

/// Dynamic asset manifest stored in `options` under `'daypress_theme_assets'`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ThemeAssetsManifest {
    pub theme_slug: String,
    pub theme_name: String,
    pub stylesheets: Vec<String>,
    pub scripts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_css: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_snippet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foot_snippet: Option<String>,
}

impl ThemeAssetsManifest {
    /// Compiles stylesheets, scripts, and snippets into standard HTML `<link>` and `<script>` tags.
    pub fn compile_to_html(&self) -> String {
        let mut html = String::new();
        for css in &self.stylesheets {
            html.push_str(&format!(
                r#"<link rel="stylesheet" href="{}" media="all" />"#,
                css
            ));
            html.push('\n');
        }
        for js in &self.scripts {
            html.push_str(&format!(r#"<script src="{}"></script>"#, js));
            html.push('\n');
        }
        if let Some(ref custom_css) = self.custom_css {
            if !custom_css.trim().is_empty() {
                html.push_str(&format!(
                    "<style id=\"daypress-theme-manifest-css\">{}</style>\n",
                    custom_css
                ));
            }
        }
        if let Some(ref snippet) = self.head_snippet {
            if !snippet.trim().is_empty() {
                html.push_str(snippet);
                html.push('\n');
            }
        }
        html
    }
}
