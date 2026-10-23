//! Theme Assets & Divi Styles Ingestion.

use crate::importer::urls::normalize_menu_url;
use crate::theme::ThemeAssetsManifest;
use rusqlite::{params, Result, Transaction};
use std::collections::BTreeSet;

/// Extracts Google Fonts families from raw Divi shortcode contents.
pub fn extract_divi_enqueued_fonts(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let re = regex::Regex::new(r#"(?:header_font|body_font|title_font|font)\s*=\s*["']([^"':|]+)"#)
        .expect("valid font regex");
    for caps in re.captures_iter(raw) {
        if let Some(fam) = caps.get(1) {
            let f = fam.as_str().trim().to_string();
            if !f.is_empty() && !out.contains(&f) {
                out.push(f);
            }
        }
    }
    let url_re = regex::Regex::new(r"https://fonts\.googleapis\.com/css2?\?family=([^&\\'\x22\s:]+)")
        .expect("valid gf url regex");
    for caps in url_re.captures_iter(raw) {
        if let Some(fam) = caps.get(1) {
            let f = fam.as_str().trim().to_string();
            if !f.is_empty() && !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

/// Builds the global CSS stylesheet for Divi imports.
pub fn build_divi_global_css(
    google_fonts: &BTreeSet<String>,
    custom_css: &str,
    has_spb_calculator: bool,
    has_divi_content: bool,
) -> String {
    let mut css = String::new();
    for fam in google_fonts {
        css.push_str(&format!(
            "@import url('https://fonts.googleapis.com/css?family={}:300,400,500,600,700,800&display=swap');\n",
            fam
        ));
    }
    if has_divi_content {
        css.push_str(
            r#"
/* Divi Global Layout & Utility Styles */
html.elementor-theme-active body {
    background-color: #ffffff;
    color: #333333;
    font-family: 'Montserrat', 'Open Sans', sans-serif;
}
.np-elementor-header, .np-elementor-footer {
    width: 100%;
}
.et-vert-menu .et-menu {
    display: flex !important;
    flex-direction: column !important;
    align-items: center !important;
    gap: 16px !important;
    list-style: none !important;
    margin: 0 !important;
    padding: 0 !important;
}
.et-vert-menu .et-menu li a {
    font-size: 28px !important;
    font-weight: 700 !important;
    text-decoration: none !important;
    color: #000000;
    transition: color 0.2s ease;
}
.et-vert-menu .et-menu li a:hover {
    color: #7c7458;
}
.et_pb_section.popup:not(.is-open) {
    display: none !important;
}
#elementor-canvas .et_pb_section.popup:not(.is-open) {
    display: flex !important;
    border: 2px dashed #7c7458 !important;
    background: #fafaf9 !important;
    margin-bottom: 16px !important;
}
.et_pb_section.popup.is-open {
    display: flex !important;
    flex-direction: column !important;
    justify-content: center !important;
    align-items: center !important;
    position: fixed !important;
    inset: 0 !important;
    z-index: 99999 !important;
    background: rgba(255, 255, 255, 0.98) !important;
    overflow-y: auto !important;
    padding: 40px 20px !important;
}
"#,
        );
    }
    if has_spb_calculator {
        css.push_str(
            r#"
/* Service Price Calculator Rules */
.spb-kw-calculator{padding:1rem 0;color:#ffffff}
.spb-kw-calculator .spb-kw-box{border:1px solid rgba(226,50,18,0.3);border-radius:12px;padding:24px;background:#101523;margin-bottom:1.5rem}
"#,
        );
    }
    if !custom_css.trim().is_empty() {
        css.push_str("\n/* Imported Custom CSS (wp_custom_css) */\n");
        css.push_str(custom_css);
        css.push('\n');
    }
    css
}

/// Stores dynamic ThemeAssetsManifest in `wp_options` under `'neopress_theme_assets'`.
pub fn persist_theme_assets_manifest(
    tx: &Transaction,
    manifest: &ThemeAssetsManifest,
) -> Result<()> {
    let json = serde_json::to_string(manifest).unwrap_or_else(|_| "{}".to_string());
    tx.execute(
        "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_theme_assets', ?1) \
         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
        params![json],
    )?;
    Ok(())
}

/// Resolves the static themes base directory checking both workspace and crate relative paths.
pub fn resolve_themes_base_dir() -> std::path::PathBuf {
    let p1 = std::path::Path::new("crates/neopress-server/static/themes");
    if p1.exists() {
        return p1.to_path_buf();
    }
    let p2 = std::path::Path::new("static/themes");
    if p2.exists() {
        return p2.to_path_buf();
    }
    p1.to_path_buf()
}

/// Discovers the active theme on disk, registers options, and persists the ThemeAssetsManifest.
pub fn resolve_and_persist_theme_assets(
    tx: &Transaction,
    wxr_site_title: &str,
    site_favicon_url: Option<String>,
    detected_logo_url: Option<String>,
    accumulated_custom_css: &str,
) -> Result<String> {
    let themes_base = resolve_themes_base_dir();
    let mut available_themes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&themes_base) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name != "neopress" && !name.starts_with('.') {
                        available_themes.push(name);
                    }
                }
            }
        }
    }

    let title_slug = generated_slugify::slugify(wxr_site_title);
    let explicit_theme: Option<String> = tx
        .query_row(
            "SELECT option_value FROM wp_options WHERE option_name = 'stylesheet' AND option_value != '' AND option_value != 'neopress' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .or_else(|_| {
            tx.query_row(
                "SELECT option_value FROM wp_options WHERE option_name = 'neopress_imported_theme' AND option_value != '' AND option_value != 'neopress' LIMIT 1",
                [],
                |r| r.get(0),
            )
        })
        .ok();

    let active_theme = if let Some(t) = explicit_theme {
        t
    } else if !title_slug.is_empty() && available_themes.contains(&title_slug) {
        title_slug
    } else if available_themes.len() == 1 {
        available_themes[0].clone()
    } else if !title_slug.is_empty() {
        title_slug
    } else {
        "twentytwentyfour".to_string()
    };

    let _ = tx.execute(
        "INSERT INTO wp_options (option_name, option_value) VALUES ('stylesheet', ?1) \
         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
        params![&active_theme],
    );
    let _ = tx.execute(
        "INSERT INTO wp_options (option_name, option_value) VALUES ('template', ?1) \
         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
        params![&active_theme],
    );
    let _ = tx.execute(
        "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_imported_theme', ?1) \
         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
        params![&active_theme],
    );

    let mut stylesheets = Vec::new();
    let mut scripts = Vec::new();
    let theme_dir = themes_base.join(&active_theme);
    if theme_dir.exists() {
        if theme_dir.join("style.css").exists() {
            stylesheets.push(format!("/wp-content/themes/{}/style.css", active_theme));
        }
        if theme_dir.join("css/styles.css").exists() || theme_dir.join("css").join("styles.css").exists() {
            stylesheets.push(format!("/wp-content/themes/{}/css/styles.css", active_theme));
        }
        if theme_dir.join("js/scripts.js").exists() || theme_dir.join("js").join("scripts.js").exists() {
            scripts.push(format!("/wp-content/themes/{}/js/scripts.js", active_theme));
        } else if theme_dir.join("scripts.js").exists() {
            scripts.push(format!("/wp-content/themes/{}/scripts.js", active_theme));
        }
    } else if active_theme != "neopress" {
        stylesheets.push(format!("/wp-content/themes/{}/css/styles.css", active_theme));
    }

    let manifest = ThemeAssetsManifest {
        theme_slug: active_theme.clone(),
        theme_name: active_theme.clone(),
        stylesheets,
        scripts,
        favicon: site_favicon_url.map(|u| normalize_menu_url(&u)),
        logo: detected_logo_url,
        custom_css: if accumulated_custom_css.trim().is_empty() {
            None
        } else {
            Some(accumulated_custom_css.to_string())
        },
        head_snippet: None,
        foot_snippet: None,
    };
    persist_theme_assets_manifest(tx, &manifest)?;

    Ok(active_theme)
}

/// Compiles and persists Elementor Kit styles and JSON into `wp_options`.
pub fn persist_elementor_kit_styles(
    tx: &Transaction,
    raw_settings: &str,
) -> Result<Option<String>> {
    if let Some(kit) = neopress_ast::ElementorKit::from_raw_settings(raw_settings) {
        let compiled_css = kit.compile_to_css();
        let kit_json = serde_json::to_string(&kit).unwrap_or_else(|_| "{}".to_string());

        for opt in ["neopress_elementor_kit_css", "elementor_active_kit_css"] {
            tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES (?1, ?2) \
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![opt, &compiled_css],
            )?;
        }
        for opt in ["neopress_elementor_kit_json", "elementor_active_kit_settings"] {
            tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES (?1, ?2) \
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![opt, &kit_json],
            )?;
        }
        if let Some(prim) = kit.system_colors.iter().find(|c| c.id == "primary") {
            if !prim.color.trim().is_empty() {
                let _ = tx.execute(
                    "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_brand_color', ?1) \
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                    params![prim.color.trim()],
                );
            }
        }
        return Ok(Some(compiled_css));
    }
    Ok(None)
}

