//! CSS Cascade & Design Token Extractor (`html_transpiler/css_parser.rs`).
//!
//! Extracts `:root` / `[data-theme="dark"]` variables, Google Fonts `<link>` / `@import`s,
//! and CSS rules across desktop, `@media` breakpoints (`tablet` / `mobile`), and
//! `:hover` / `:focus` / `:active` pseudo-states into `ElementorKit` and `StyleProperties`.

use super::dom_parser::{DomElement, DomNode};
use crate::kit::{ButtonStyle, ElementorKit, GlobalColor, GlobalTypography};
use crate::schema::StyleProperties;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaTarget {
    Desktop,
    Tablet,
    Mobile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoTarget {
    None,
    Hover,
    Focus,
    Active,
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub media: MediaTarget,
    pub pseudo: PseudoTarget,
    pub specificity: (u32, u32, u32),
    pub declarations: Vec<(String, String)>,
    pub order: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedStylesheet {
    pub google_font_urls: Vec<String>,
    pub google_font_families: Vec<String>,
    pub root_vars: Vec<(String, String)>,
    pub dark_vars: Vec<(String, String)>,
    pub rules: Vec<CssRule>,
    pub raw_css: String,
}

/// Walks the DOM tree to collect all `<style>` contents and Google Fonts `<link>` tags,
/// then parses them into a [`ParsedStylesheet`].
pub fn extract_and_parse_styles(root: &DomElement) -> ParsedStylesheet {
    let mut sheet = ParsedStylesheet::default();
    let mut raw_blocks = Vec::new();
    collect_style_nodes(root, &mut sheet.google_font_urls, &mut raw_blocks);

    for url in &sheet.google_font_urls {
        for fam in extract_families_from_google_url(url) {
            if !sheet.google_font_families.contains(&fam) {
                sheet.google_font_families.push(fam);
            }
        }
    }

    let combined_css = raw_blocks.join("\n\n");
    sheet.raw_css = combined_css.clone();
    let mut order_counter = 0usize;
    parse_css_block(
        &combined_css,
        MediaTarget::Desktop,
        &mut sheet,
        &mut order_counter,
    );

    sheet
}

fn collect_style_nodes(
    el: &DomElement,
    font_urls: &mut Vec<String>,
    style_blocks: &mut Vec<String>,
) {
    if el.tag == "link" {
        if let Some(href) = el.attr("href") {
            if href.contains("fonts.googleapis.com/css") && !font_urls.contains(&href.to_string()) {
                font_urls.push(href.to_string());
            }
        }
    } else if el.tag == "style" {
        let css_text = el.text_content();
        if !css_text.trim().is_empty() {
            style_blocks.push(css_text);
        }
    }

    for child in &el.children {
        if let DomNode::Element(child_el) = child {
            collect_style_nodes(child_el, font_urls, style_blocks);
        }
    }
}

fn extract_families_from_google_url(url: &str) -> Vec<String> {
    let mut families = Vec::new();
    let Some((_, query)) = url.split_once('?') else {
        return families;
    };
    for part in query.split('&') {
        if let Some(rest) = part.strip_prefix("family=") {
            let fam_part = rest.split(':').next().unwrap_or(rest);
            let decoded = fam_part.replace('+', " ").replace("%20", " ");
            let clean = decoded.trim();
            if !clean.is_empty() && !families.contains(&clean.to_string()) {
                families.push(clean.to_string());
            }
        }
    }
    families
}

fn strip_css_comments(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut in_quote: Option<char> = None;

    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = in_quote {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                i += 1;
                out.push(chars[i]);
            } else if c == q {
                in_quote = None;
            }
            i += 1;
            continue;
        }
        if c == '"' || c == '\'' {
            in_quote = Some(c);
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i < chars.len() {
                if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn parse_css_block(
    css: &str,
    current_media: MediaTarget,
    sheet: &mut ParsedStylesheet,
    order: &mut usize,
) {
    let cleaned = strip_css_comments(css);
    let chars: Vec<(usize, char)> = cleaned.char_indices().collect();
    let byte_at = |idx: usize| chars.get(idx).map(|c| c.0).unwrap_or(cleaned.len());
    let slice = |a: usize, b: usize| cleaned.get(byte_at(a)..byte_at(b)).unwrap_or("");

    let mut start = 0usize;
    let mut i = 0usize;
    let mut in_quote: Option<char> = None;

    while i < chars.len() {
        let c = chars[i].1;
        if let Some(q) = in_quote {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_quote = None;
            }
            i += 1;
            continue;
        }
        if c == '"' || c == '\'' {
            in_quote = Some(c);
            i += 1;
            continue;
        }

        if c == '{' {
            let prelude = slice(start, i).trim();
            let mut depth = 1usize;
            let mut j = i + 1;
            let mut inner_quote: Option<char> = None;
            while j < chars.len() {
                let ch = chars[j].1;
                if let Some(q) = inner_quote {
                    if ch == '\\' {
                        j += 2;
                        continue;
                    }
                    if ch == q {
                        inner_quote = None;
                    }
                    j += 1;
                    continue;
                }
                if ch == '"' || ch == '\'' {
                    inner_quote = Some(ch);
                    j += 1;
                    continue;
                }
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            let body = slice(i + 1, j).trim();
            handle_rule_or_at_block(prelude, body, current_media, sheet, order);
            i = j + 1;
            start = i;
            continue;
        } else if c == ';' {
            let stmt = slice(start, i + 1).trim();
            if stmt.to_ascii_lowercase().starts_with("@import") {
                if let Some(url) = extract_url_from_import(stmt) {
                    if url.contains("fonts.googleapis.com")
                        && !sheet.google_font_urls.contains(&url)
                    {
                        for fam in extract_families_from_google_url(&url) {
                            if !sheet.google_font_families.contains(&fam) {
                                sheet.google_font_families.push(fam);
                            }
                        }
                        sheet.google_font_urls.push(url);
                    }
                }
            }
            i += 1;
            start = i;
            continue;
        }
        i += 1;
    }
}

fn extract_url_from_import(stmt: &str) -> Option<String> {
    for quote in ['\'', '"'] {
        if let Some(first) = stmt.find(quote) {
            let rest = stmt.get(first + 1..)?;
            if let Some(second) = rest.find(quote) {
                return Some(rest.get(..second)?.to_string());
            }
        }
    }
    None
}

fn handle_rule_or_at_block(
    prelude: &str,
    body: &str,
    current_media: MediaTarget,
    sheet: &mut ParsedStylesheet,
    order: &mut usize,
) {
    let lower_prelude = prelude.to_ascii_lowercase();
    if lower_prelude.starts_with("@media") {
        if lower_prelude.contains("prefers-color-scheme") && lower_prelude.contains("dark") {
            // Parse dark mode variables inside @media (prefers-color-scheme: dark)
            let mut temp_sheet = ParsedStylesheet::default();
            let mut tmp_order = 0;
            parse_css_block(body, current_media, &mut temp_sheet, &mut tmp_order);
            for (k, v) in temp_sheet.root_vars.into_iter().chain(temp_sheet.dark_vars) {
                if !sheet.dark_vars.iter().any(|(ek, _)| ek == &k) {
                    sheet.dark_vars.push((k, v));
                }
            }
            return;
        }
        let target_media = classify_media_query(&lower_prelude).unwrap_or(current_media);
        parse_css_block(body, target_media, sheet, order);
        return;
    }
    if lower_prelude.starts_with('@') {
        return;
    }

    let decls = parse_declarations(body);
    if decls.is_empty() {
        return;
    }

    for raw_sel in split_selectors(prelude) {
        let sel = raw_sel.trim();
        if sel.is_empty() {
            continue;
        }

        // Capture :root and [data-theme="dark"] custom properties
        let is_dark_root = sel.contains("data-theme") && sel.contains("dark");
        let is_root = sel == ":root" || sel == "html";

        if is_dark_root || is_root {
            for (k, v) in &decls {
                if k.starts_with("--") {
                    let target_vec = if is_dark_root {
                        &mut sheet.dark_vars
                    } else {
                        &mut sheet.root_vars
                    };
                    if let Some(existing) = target_vec.iter_mut().find(|(ek, _)| ek == k) {
                        existing.1 = v.clone();
                    } else {
                        target_vec.push((k.clone(), v.clone()));
                    }
                }
            }
        }

        // Skip rules with pseudo-elements (::before, ::after) or [data-theme] attribute overrides
        // from static AST style baking; they remain active via `global_styles.custom_css`.
        if sel.contains("::")
            || sel.contains(":before")
            || sel.contains(":after")
            || sel.contains("[data-theme")
            || sel.contains(":first-child")
            || sel.contains(":last-child")
            || sel.contains(":nth-")
            || sel.contains(":not(")
        {
            continue;
        }

        let (clean_sel, pseudo) = extract_pseudo_target(sel);
        let specificity = compute_specificity(&clean_sel);
        *order += 1;
        sheet.rules.push(CssRule {
            selector: clean_sel,
            media: current_media,
            pseudo,
            specificity,
            declarations: decls.clone(),
            order: *order,
        });
    }
}

fn classify_media_query(lower_prelude: &str) -> Option<MediaTarget> {
    if let Some(idx) = lower_prelude.find("max-width") {
        let after = lower_prelude.get(idx + "max-width".len()..)?;
        let digits: String = after
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(px) = digits.parse::<u32>() {
            if px <= 767 {
                return Some(MediaTarget::Mobile);
            } else if px <= 1150 {
                return Some(MediaTarget::Tablet);
            }
        }
    }
    None
}

fn extract_pseudo_target(sel: &str) -> (String, PseudoTarget) {
    if let Some(stripped) = sel.strip_suffix(":hover") {
        (stripped.trim().to_string(), PseudoTarget::Hover)
    } else if let Some(stripped) = sel.strip_suffix(":focus") {
        (stripped.trim().to_string(), PseudoTarget::Focus)
    } else if let Some(stripped) = sel.strip_suffix(":active") {
        (stripped.trim().to_string(), PseudoTarget::Active)
    } else {
        (sel.to_string(), PseudoTarget::None)
    }
}

fn compute_specificity(sel: &str) -> (u32, u32, u32) {
    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut tags = 0u32;

    for token in sel.split(|c: char| c.is_whitespace() || matches!(c, '>' | '+' | '~')) {
        let t = token.trim();
        if t.is_empty() || t == "*" {
            continue;
        }
        let mut at_start = true;
        for c in t.chars() {
            if c == '#' {
                ids += 1;
                at_start = false;
            } else if c == '.' || c == '[' || c == ':' {
                classes += 1;
                at_start = false;
            } else if at_start && c.is_ascii_alphabetic() {
                tags += 1;
                at_start = false;
            }
        }
    }
    (ids, classes, tags)
}

pub fn parse_declarations(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for raw_decl in split_top_level_char(body, ';') {
        let decl = raw_decl.trim();
        if decl.is_empty() {
            continue;
        }
        if let Some((prop, val)) = decl.split_once(':') {
            let p = prop.trim().to_ascii_lowercase();
            let v = val.trim().trim_end_matches("!important").trim().to_string();
            if !p.is_empty() && !v.is_empty() {
                out.push((p, v));
            }
        }
    }
    out
}

fn split_selectors(prelude: &str) -> Vec<&str> {
    split_top_level_char(prelude, ',')
}

pub fn split_top_level_char(s: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut start = 0usize;

    for (i, c) in s.char_indices() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(' | '[') => depth += 1,
            (None, ')' | ']') => depth -= 1,
            (None, _) if c == sep && depth == 0 => {
                parts.push(s.get(start..i).unwrap_or(""));
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(s.get(start..).unwrap_or(""));
    parts
}

/// Splits CSS shorthand values on top-level whitespace while keeping `var(...)`, `clamp(...)`, `rgb(...)` intact.
pub fn split_shorthand_values(val: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut start: Option<usize> = None;

    for (i, c) in val.char_indices() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => {
                if start.is_none() {
                    start = Some(i);
                }
                quote = Some(c);
            }
            (None, '(') => {
                if start.is_none() {
                    start = Some(i);
                }
                depth += 1;
            }
            (None, ')') => {
                depth -= 1;
            }
            (None, _) if c.is_whitespace() && depth == 0 => {
                if let Some(s_idx) = start.take() {
                    parts.push(val.get(s_idx..i).unwrap_or(""));
                }
            }
            (None, _) => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
    }
    if let Some(s_idx) = start {
        parts.push(val.get(s_idx..).unwrap_or(""));
    }
    parts
}

/// Synthesizes an [`ElementorKit`] (`NeopressLayout.global_styles`) from the parsed stylesheet.
pub fn build_elementor_kit(
    sheet: &ParsedStylesheet,
    site_name: Option<String>,
    site_description: Option<String>,
) -> ElementorKit {
    let mut color_vars: Vec<(String, String)> = Vec::new();
    let mut font_vars: Vec<(String, String)> = Vec::new();
    let mut content_width = 1320u32;

    for (name, val) in &sheet.root_vars {
        let key = name.trim_start_matches('-').to_ascii_lowercase();
        let trimmed_val = val.trim();

        if is_color_value(trimmed_val) {
            color_vars.push((key, trimmed_val.to_string()));
        } else if is_font_family_var(&key, trimmed_val) {
            font_vars.push((key, trimmed_val.to_string()));
        } else if matches!(
            key.as_str(),
            "wide" | "max-width" | "container" | "container-width" | "content-width"
        ) {
            if let Some(px) = parse_px_u32(trimmed_val) {
                if (600..=2400).contains(&px) {
                    content_width = px;
                }
            }
        }
    }

    // 1. System Colors (primary, secondary, text, accent) + Custom Colors
    let primary_color = find_var_by_keys(
        &color_vars,
        &["oxide", "primary", "brand", "accent-primary"],
    )
    .unwrap_or_else(|| {
        color_vars
            .first()
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "#C8371A".to_string())
    });

    let secondary_color = find_var_by_keys(&color_vars, &["ink", "secondary", "heading", "dark"])
        .unwrap_or_else(|| "#13120F".to_string());

    let text_color = find_var_by_keys(&color_vars, &["ink-2", "text", "body", "fg", "ink"])
        .unwrap_or_else(|| secondary_color.clone());

    let accent_color = find_var_by_keys(&color_vars, &["forest", "accent", "amber", "oxide-deep"])
        .unwrap_or_else(|| primary_color.clone());

    let system_colors = vec![
        GlobalColor {
            id: "primary".to_string(),
            title: "Primary Brand".to_string(),
            color: primary_color.clone(),
        },
        GlobalColor {
            id: "secondary".to_string(),
            title: "Secondary / Ink".to_string(),
            color: secondary_color,
        },
        GlobalColor {
            id: "text".to_string(),
            title: "Body Text".to_string(),
            color: text_color,
        },
        GlobalColor {
            id: "accent".to_string(),
            title: "Accent".to_string(),
            color: accent_color,
        },
    ];

    let mut custom_colors = Vec::new();
    for (k, v) in &color_vars {
        let id = sanitize_token_id(k);
        if !matches!(id.as_str(), "primary" | "secondary" | "text" | "accent") {
            custom_colors.push(GlobalColor {
                id: id.clone(),
                title: humanize_token_title(k),
                color: v.clone(),
            });
        }
    }

    // Also include dark-mode color tokens if present
    for (k, v) in &sheet.dark_vars {
        let clean_k = k.trim_start_matches('-');
        if is_color_value(v.trim()) {
            let dark_id = format!("dark-{}", sanitize_token_id(clean_k));
            if !custom_colors.iter().any(|c| c.id == dark_id) {
                custom_colors.push(GlobalColor {
                    id: dark_id,
                    title: format!("Dark {}", humanize_token_title(clean_k)),
                    color: v.trim().to_string(),
                });
            }
        }
    }

    // 2. System Typography + Custom Typography
    let heading_font =
        find_var_by_keys(&font_vars, &["serif", "display", "heading", "font-heading"])
            .or_else(|| sheet.google_font_families.first().cloned())
            .unwrap_or_else(|| "'Newsreader', Georgia, serif".to_string());

    let body_font = find_var_by_keys(
        &font_vars,
        &["sans", "body", "text", "font-sans", "font-body"],
    )
    .or_else(|| sheet.google_font_families.get(1).cloned())
    .unwrap_or_else(|| "'Schibsted Grotesk', -apple-system, sans-serif".to_string());

    let mono_font = find_var_by_keys(&font_vars, &["mono", "code", "font-mono"])
        .or_else(|| sheet.google_font_families.get(2).cloned())
        .unwrap_or_else(|| "'IBM Plex Mono', monospace".to_string());

    let system_typography = vec![
        GlobalTypography {
            id: "primary".to_string(),
            title: "Display / Editorial Heading".to_string(),
            font_family: heading_font,
            font_size: Some("clamp(36px, 4.4vw, 60px)".to_string()),
            font_weight: Some("400".to_string()),
            line_height: Some("1.02".to_string()),
            letter_spacing: Some("-0.025em".to_string()),
        },
        GlobalTypography {
            id: "secondary".to_string(),
            title: "Section / Deck Heading".to_string(),
            font_family: body_font.clone(),
            font_size: Some("clamp(22px, 2.4vw, 32px)".to_string()),
            font_weight: Some("600".to_string()),
            line_height: Some("1.15".to_string()),
            letter_spacing: Some("-0.015em".to_string()),
        },
        GlobalTypography {
            id: "text".to_string(),
            title: "Body Copy".to_string(),
            font_family: body_font,
            font_size: Some("16px".to_string()),
            font_weight: Some("400".to_string()),
            line_height: Some("1.55".to_string()),
            letter_spacing: None,
        },
        GlobalTypography {
            id: "accent".to_string(),
            title: "Monospace / Spec Labels".to_string(),
            font_family: mono_font,
            font_size: Some("11.5px".to_string()),
            font_weight: Some("500".to_string()),
            line_height: Some("1.4".to_string()),
            letter_spacing: Some("0.12em".to_string()),
        },
    ];

    let mut custom_typography = Vec::new();
    for (k, v) in &font_vars {
        let id = sanitize_token_id(k);
        if !matches!(id.as_str(), "primary" | "secondary" | "text" | "accent") {
            custom_typography.push(GlobalTypography {
                id,
                title: humanize_token_title(k),
                font_family: v.clone(),
                font_size: None,
                font_weight: None,
                line_height: None,
                letter_spacing: None,
            });
        }
    }

    // 3. Button Style
    let mut btn_bg = Some(primary_color);
    let mut btn_color = Some("#F5F1E8".to_string());
    let mut btn_radius = Some("0px".to_string());
    for rule in &sheet.rules {
        if matches!(rule.selector.as_str(), ".btn-primary" | ".btn" | ".button")
            && rule.pseudo == PseudoTarget::None
        {
            for (k, v) in &rule.declarations {
                match k.as_str() {
                    "background" | "background-color" => btn_bg = Some(v.clone()),
                    "color" => btn_color = Some(v.clone()),
                    "border-radius" => btn_radius = Some(v.clone()),
                    _ => {}
                }
            }
        }
    }

    // 4. Preserved Custom CSS (including @import for Google Fonts + full stylesheet)
    let mut custom_css = String::new();
    for url in &sheet.google_font_urls {
        custom_css.push_str(&format!("@import url('{}');\n", url));
    }
    if !sheet.raw_css.trim().is_empty() {
        if !custom_css.is_empty() {
            custom_css.push('\n');
        }
        custom_css.push_str(sheet.raw_css.trim());
    }

    ElementorKit {
        system_colors,
        custom_colors,
        system_typography,
        custom_typography,
        content_width,
        widgets_space: 24,
        site_name,
        site_description,
        site_logo: None,
        site_favicon: None,
        button_style: Some(ButtonStyle {
            background_color: btn_bg,
            text_color: btn_color,
            border_radius: btn_radius,
        }),
        custom_css: if custom_css.is_empty() {
            None
        } else {
            Some(custom_css)
        },
    }
}

/// Resolves all matching CSS rules + inline `style="..."` for `el` into a [`StyleProperties`].
pub fn resolve_element_styles(
    sheet: &ParsedStylesheet,
    el: &DomElement,
    ancestors: &[&DomElement],
) -> StyleProperties {
    let mut styles = StyleProperties::default();
    if let Some(cls) = el.class_str() {
        styles.css_classes = Some(cls.to_string());
    }

    let mut matched: Vec<&CssRule> = sheet
        .rules
        .iter()
        .filter(|r| selector_matches(&r.selector, el, ancestors))
        .collect();

    matched.sort_by(|a, b| {
        a.specificity
            .cmp(&b.specificity)
            .then(a.order.cmp(&b.order))
    });

    for rule in matched {
        let target_styles: &mut StyleProperties = match (rule.media, rule.pseudo) {
            (MediaTarget::Desktop, PseudoTarget::None) => &mut styles,
            (MediaTarget::Tablet, PseudoTarget::None) => styles
                .tablet
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
            (MediaTarget::Mobile, PseudoTarget::None) => styles
                .mobile
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
            (_, PseudoTarget::Hover) => styles
                .hover
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
            (_, PseudoTarget::Focus) => styles
                .focus
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
            (_, PseudoTarget::Active) => styles
                .active
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
        };
        for (prop, val) in &rule.declarations {
            apply_css_declaration(target_styles, prop, val);
        }
    }

    // Inline `style="..."` overrides stylesheet declarations
    if let Some(inline_style) = el.attr("style") {
        for (prop, val) in parse_declarations(inline_style) {
            apply_css_declaration(&mut styles, &prop, &val);
        }
    }

    styles.sync_flat_and_nested();
    styles
}

/// Applies a single CSS declaration (`property: value`) onto a [`StyleProperties`].
pub fn apply_css_declaration(styles: &mut StyleProperties, prop: &str, val: &str) {
    match prop {
        "display" => styles.display = Some(val.to_string()),
        "grid-template-columns" => styles.grid_template_columns = Some(val.to_string()),
        "grid-template-rows" => styles.grid_template_rows = Some(val.to_string()),
        "grid-column" => styles.grid_column = Some(val.to_string()),
        "grid-row" => styles.grid_row = Some(val.to_string()),
        "grid-auto-flow" => styles.grid_auto_flow = Some(val.to_string()),
        "justify-items" => styles.justify_items = Some(val.to_string()),
        "gap" | "grid-gap" | "column-gap" | "row-gap" => {
            if prop == "gap" || styles.gap.is_none() {
                styles.gap = Some(val.to_string());
                styles.grid_gap = Some(val.to_string());
            }
        }
        "flex-direction" => styles.flex_direction = Some(val.to_string()),
        "flex-wrap" => styles.flex_wrap = Some(val.to_string()),
        "justify-content" => styles.justify_content = Some(val.to_string()),
        "align-items" => styles.align_items = Some(val.to_string()),
        "flex" => styles.flex = Some(val.to_string()),
        "align-self" => styles.align_self = Some(val.to_string()),
        "position" => styles.position = Some(val.to_string()),
        "top" => styles.top = Some(val.to_string()),
        "right" => styles.right = Some(val.to_string()),
        "bottom" => styles.bottom = Some(val.to_string()),
        "left" => styles.left = Some(val.to_string()),
        "z-index" => styles.z_index = Some(val.to_string()),
        "overflow" | "overflow-x" | "overflow-y" => {
            if prop == "overflow" || styles.overflow.is_none() {
                styles.overflow = Some(val.to_string());
            }
        }
        "width" => styles.width = Some(val.to_string()),
        "min-width" => styles.min_width = Some(val.to_string()),
        "max-width" => styles.max_width = Some(val.to_string()),
        "height" => styles.height = Some(val.to_string()),
        "min-height" => styles.min_height = Some(val.to_string()),
        "max-height" => styles.max_height = Some(val.to_string()),
        "aspect-ratio" => styles.aspect_ratio = Some(val.to_string()),
        "object-fit" => styles.object_fit = Some(val.to_string()),
        "background" | "background-color" => {
            if val.contains("gradient(") {
                styles.background_gradient = Some(val.to_string());
            } else {
                styles.background_color = Some(val.to_string());
            }
        }
        "background-image" => {
            if val.contains("gradient(") {
                styles.background_gradient = Some(val.to_string());
            } else {
                styles.background_image = Some(val.to_string());
            }
        }
        "background-position" => styles.background_position = Some(val.to_string()),
        "background-size" => styles.background_size = Some(val.to_string()),
        "background-repeat" => styles.background_repeat = Some(val.to_string()),
        "color" => styles.text_color = Some(val.to_string()),
        "font-family" => styles.font_family = Some(val.to_string()),
        "font-size" => styles.font_size = Some(val.to_string()),
        "font-weight" => styles.font_weight = Some(val.to_string()),
        "line-height" => styles.line_height = Some(val.to_string()),
        "letter-spacing" => styles.letter_spacing = Some(val.to_string()),
        "text-align" => styles.text_align = Some(val.to_string()),
        "text-transform" => styles.text_transform = Some(val.to_string()),
        "font-style" => styles.font_style = Some(val.to_string()),
        "text-decoration" => styles.text_decoration = Some(val.to_string()),
        "text-shadow" => styles.text_shadow = Some(val.to_string()),
        "padding" => {
            let parts = split_shorthand_values(val);
            let (t, r, b, l) = match parts.as_slice() {
                [all] => (*all, *all, *all, *all),
                [v, h] => (*v, *h, *v, *h),
                [t, h, b] => (*t, *h, *b, *h),
                [t, r, b, l, ..] => (*t, *r, *b, *l),
                [] => return,
            };
            styles.padding_top = Some(t.to_string());
            styles.padding_right = Some(r.to_string());
            styles.padding_bottom = Some(b.to_string());
            styles.padding_left = Some(l.to_string());
        }
        "padding-top" => styles.padding_top = Some(val.to_string()),
        "padding-right" => styles.padding_right = Some(val.to_string()),
        "padding-bottom" => styles.padding_bottom = Some(val.to_string()),
        "padding-left" => styles.padding_left = Some(val.to_string()),
        "margin" => {
            let parts = split_shorthand_values(val);
            let (t, r, b, l) = match parts.as_slice() {
                [all] => (*all, *all, *all, *all),
                [v, h] => (*v, *h, *v, *h),
                [t, h, b] => (*t, *h, *b, *h),
                [t, r, b, l, ..] => (*t, *r, *b, *l),
                [] => return,
            };
            styles.margin_top = Some(t.to_string());
            styles.margin_right = Some(r.to_string());
            styles.margin_bottom = Some(b.to_string());
            styles.margin_left = Some(l.to_string());
        }
        "margin-top" => styles.margin_top = Some(val.to_string()),
        "margin-right" => styles.margin_right = Some(val.to_string()),
        "margin-bottom" => styles.margin_bottom = Some(val.to_string()),
        "margin-left" => styles.margin_left = Some(val.to_string()),
        "border-radius" => styles.border_radius = Some(val.to_string()),
        "border" => {
            if val == "none" || val == "0" {
                styles.border_style = Some("none".to_string());
            } else {
                let parts = split_shorthand_values(val);
                if parts.len() == 3 {
                    styles.border_width = Some(parts[0].to_string());
                    styles.border_style = Some(parts[1].to_string());
                    styles.border_color = Some(parts[2].to_string());
                } else {
                    styles.border_top = Some(val.to_string());
                    styles.border_right = Some(val.to_string());
                    styles.border_bottom = Some(val.to_string());
                    styles.border_left = Some(val.to_string());
                }
            }
        }
        "border-width" => styles.border_width = Some(val.to_string()),
        "border-style" => styles.border_style = Some(val.to_string()),
        "border-color" => styles.border_color = Some(val.to_string()),
        "border-top" => styles.border_top = Some(val.to_string()),
        "border-right" => styles.border_right = Some(val.to_string()),
        "border-bottom" => styles.border_bottom = Some(val.to_string()),
        "border-left" => styles.border_left = Some(val.to_string()),
        "box-shadow" => styles.box_shadow = Some(val.to_string()),
        "opacity" => styles.opacity = Some(val.to_string()),
        "transform" => styles.transform = Some(val.to_string()),
        "transition" => styles.transition = Some(val.to_string()),
        "backdrop-filter" => styles.backdrop_filter = Some(val.to_string()),
        "filter" => styles.filter = Some(val.to_string()),
        "cursor" => styles.cursor = Some(val.to_string()),
        _ => {}
    }
}

/// Evaluates whether a CSS selector (`tag`, `.class`, `#id`, descendant `A B`, or child `A > B`)
/// matches `el` given its `ancestors` (ordered from root to parent).
pub fn selector_matches(selector: &str, el: &DomElement, ancestors: &[&DomElement]) -> bool {
    // Reject adjacent/general sibling combinators (`+`, `~`) for static baking
    if selector.contains('+') || selector.contains('~') || selector.contains('[') {
        return false;
    }

    let tokens = tokenize_selector_chain(selector);
    if tokens.is_empty() {
        return false;
    }

    let (last_comb, last_compound) = &tokens[tokens.len() - 1];
    if !compound_matches(last_compound, el) {
        return false;
    }
    if tokens.len() == 1 {
        return true;
    }

    let mut anc_idx = ancestors.len();
    let mut required_direct = *last_comb == Some('>');

    for idx in (0..tokens.len() - 1).rev() {
        let (comb_before, compound) = &tokens[idx];
        if required_direct {
            if anc_idx == 0 {
                return false;
            }
            anc_idx -= 1;
            if !compound_matches(compound, ancestors[anc_idx]) {
                return false;
            }
        } else {
            let mut found = false;
            while anc_idx > 0 {
                anc_idx -= 1;
                if compound_matches(compound, ancestors[anc_idx]) {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        required_direct = *comb_before == Some('>');
    }

    true
}

fn tokenize_selector_chain(sel: &str) -> Vec<(Option<char>, String)> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut pending_comb: Option<char> = None;

    for c in sel.chars() {
        if c == '>' {
            if !current.trim().is_empty() {
                out.push((pending_comb, current.trim().to_string()));
                current.clear();
            }
            pending_comb = Some('>');
        } else if c.is_whitespace() {
            if !current.trim().is_empty() {
                out.push((pending_comb, current.trim().to_string()));
                current.clear();
                pending_comb = Some(' ');
            }
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        out.push((pending_comb, current.trim().to_string()));
    }
    out
}

fn compound_matches(compound: &str, el: &DomElement) -> bool {
    if compound.is_empty() || compound.contains(':') {
        return false;
    }
    if compound == "*" {
        return true;
    }

    let mut tag_req: Option<&str> = None;
    let mut id_req: Option<&str> = None;
    let mut class_reqs: Vec<&str> = Vec::new();

    let mut start = 0usize;
    let mut mode = 't'; // 't' = tag, '#' = id, '.' = class

    for (i, c) in compound.char_indices() {
        if c == '#' || c == '.' {
            let token = compound.get(start..i).unwrap_or("");
            if !token.is_empty() {
                match mode {
                    't' => tag_req = Some(token),
                    '#' => id_req = Some(token),
                    '.' => class_reqs.push(token),
                    _ => {}
                }
            }
            mode = c;
            start = i + 1;
        }
    }
    let last_token = compound.get(start..).unwrap_or("");
    if !last_token.is_empty() {
        match mode {
            't' => tag_req = Some(last_token),
            '#' => id_req = Some(last_token),
            '.' => class_reqs.push(last_token),
            _ => {}
        }
    }

    if let Some(t) = tag_req {
        if !el.tag.eq_ignore_ascii_case(t) {
            return false;
        }
    }
    if let Some(id) = id_req {
        if el.id() != Some(id) {
            return false;
        }
    }
    if !class_reqs.is_empty() {
        let el_classes = el.classes();
        for req in class_reqs {
            if !el_classes.contains(&req) {
                return false;
            }
        }
    }

    true
}

fn is_color_value(val: &str) -> bool {
    let lower = val.to_ascii_lowercase();
    (lower.starts_with('#') && matches!(lower.len(), 4 | 5 | 7 | 9))
        || lower.starts_with("rgb(")
        || lower.starts_with("rgba(")
        || lower.starts_with("hsl(")
        || lower.starts_with("hsla(")
}

fn is_font_family_var(key: &str, val: &str) -> bool {
    matches!(
        key,
        "serif"
            | "sans"
            | "mono"
            | "display"
            | "font-sans"
            | "font-serif"
            | "font-mono"
            | "font-heading"
            | "font-body"
    ) || key.contains("font")
        || val.contains("serif")
        || val.contains("monospace")
}

fn parse_px_u32(val: &str) -> Option<u32> {
    let digits: String = val.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<u32>().ok()
}

fn find_var_by_keys(vars: &[(String, String)], keys: &[&str]) -> Option<String> {
    for target in keys {
        if let Some((_, v)) = vars.iter().find(|(k, _)| k == target) {
            return Some(v.clone());
        }
    }
    None
}

fn sanitize_token_id(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn humanize_token_title(raw: &str) -> String {
    raw.split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
