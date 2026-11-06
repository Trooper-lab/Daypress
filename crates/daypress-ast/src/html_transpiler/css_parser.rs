//! CSS Cascade & Design Token Extractor (`html_transpiler/css_parser.rs`).
//!
//! Extracts `:root` / `[data-theme="dark"]` variables, Google Fonts `<link>` / `@import`s,
//! static top-level CSS rules and their `:hover` / `:focus` / `:active` pseudo-states into
//! `DaystudioKit` and `StyleProperties`.
//!
//! The design's full stylesheet is kept in the kit (`custom_css`), so it keeps applying on the
//! public page. Only values the design applies unconditionally are baked into node styles (which
//! render inline). Values that depend on a breakpoint (`@media`), on `@supports` / `@container` /
//! `@layer`, or on state the static resolver cannot see (`:target`, `:checked`, `:not()`,
//! structural pseudo-classes, attribute selectors, sibling combinators, classes that scripts add
//! such as `.nav-open`) are recorded as [`DynamicRule`]s. Their properties are left out of the
//! baked styles entirely, so the design's own cascade (including its state classes) decides them,
//! without any `!important` per-node override competing with it.

use super::dom_parser::{DomElement, DomNode};
use crate::kit::{ButtonStyle, DaystudioKit, GlobalColor, GlobalTypography};
use crate::schema::StyleProperties;
use std::collections::HashSet;

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
    pub pseudo: PseudoTarget,
    pub specificity: (u32, u32, u32),
    pub declarations: Vec<(String, String)>,
    pub order: usize,
}

/// A design rule whose effect depends on a breakpoint, a conditional group or runtime state.
/// `selector` is a static approximation (only tags, classes, ids, descendant and child
/// combinators) used to find the elements it may style.
#[derive(Debug, Clone)]
pub struct DynamicRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockContext {
    /// Top-level rules: statically evaluable ones are baked into node styles.
    TopLevel,
    /// Inside `@media`, `@supports`, `@container`, `@layer`: never baked, always left to the design CSS.
    Conditional,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedStylesheet {
    pub google_font_urls: Vec<String>,
    pub google_font_families: Vec<String>,
    pub root_vars: Vec<(String, String)>,
    pub dark_vars: Vec<(String, String)>,
    pub rules: Vec<CssRule>,
    pub dynamic_rules: Vec<DynamicRule>,
    /// Classes and ids that occur in the document's markup. A selector that needs a class or id
    /// missing from the markup (e.g. `.nav-open`, `.js`) only matches once a script adds it.
    pub present_classes: HashSet<String>,
    pub present_ids: HashSet<String>,
    pub raw_css: String,
}

/// Walks the DOM tree to collect all `<style>` contents and Google Fonts `<link>` tags,
/// then parses them into a [`ParsedStylesheet`].
pub fn extract_and_parse_styles(root: &DomElement) -> ParsedStylesheet {
    let mut sheet = ParsedStylesheet::default();
    collect_present_names(root, &mut sheet.present_classes, &mut sheet.present_ids);
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
        BlockContext::TopLevel,
        &mut sheet,
        &mut order_counter,
    );

    sheet
}

fn collect_present_names(
    el: &DomElement,
    classes: &mut HashSet<String>,
    ids: &mut HashSet<String>,
) {
    for c in el.classes() {
        classes.insert(c.to_string());
    }
    if let Some(id) = el.id() {
        ids.insert(id.to_string());
    }
    for child in &el.children {
        if let DomNode::Element(child_el) = child {
            collect_present_names(child_el, classes, ids);
        }
    }
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
    context: BlockContext,
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
            handle_rule_or_at_block(prelude, body, context, sheet, order);
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
    context: BlockContext,
    sheet: &mut ParsedStylesheet,
    order: &mut usize,
) {
    let lower_prelude = prelude.to_ascii_lowercase();
    if lower_prelude.starts_with("@media") {
        if lower_prelude.contains("prefers-color-scheme") && lower_prelude.contains("dark") {
            // Parse dark mode variables inside @media (prefers-color-scheme: dark)
            let mut temp_sheet = ParsedStylesheet::default();
            let mut tmp_order = 0;
            parse_css_block(
                body,
                BlockContext::TopLevel,
                &mut temp_sheet,
                &mut tmp_order,
            );
            for (k, v) in temp_sheet.root_vars.into_iter().chain(temp_sheet.dark_vars) {
                if !sheet.dark_vars.iter().any(|(ek, _)| ek == &k) {
                    sheet.dark_vars.push((k, v));
                }
            }
            return;
        }
        parse_css_block(body, BlockContext::Conditional, sheet, order);
        return;
    }
    if ["@supports", "@container", "@layer", "@scope", "@document"]
        .iter()
        .any(|at| lower_prelude.starts_with(at))
    {
        parse_css_block(body, BlockContext::Conditional, sheet, order);
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

        // Capture :root and [data-theme="dark"] custom properties (unconditional ones only:
        // a breakpoint's `:root { --gutter: 24px }` must not replace the design token).
        let is_dark_root = sel.contains("data-theme") && sel.contains("dark");
        let is_root = sel == ":root" || sel == "html";

        if (is_dark_root || is_root) && context == BlockContext::TopLevel {
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

        // Pseudo-element rules style generated boxes, not the element; `[data-theme]` overrides are
        // dark-mode token sets. Both stay active through `global_styles.custom_css` only.
        if is_pseudo_element_selector(sel) || sel.contains("[data-theme") {
            continue;
        }

        let (clean_sel, pseudo) = extract_pseudo_target(sel);
        if context == BlockContext::TopLevel && is_static_selector(&clean_sel, sheet) {
            let specificity = compute_specificity(&clean_sel);
            *order += 1;
            sheet.rules.push(CssRule {
                selector: clean_sel,
                pseudo,
                specificity,
                declarations: decls.clone(),
                order: *order,
            });
        } else if let Some(reduced) = reduce_to_static_selector(sel, sheet) {
            sheet.dynamic_rules.push(DynamicRule {
                selector: reduced,
                declarations: decls.clone(),
            });
        }
    }
}

fn is_pseudo_element_selector(sel: &str) -> bool {
    let lower = sel.to_ascii_lowercase();
    lower.contains("::")
        || [":before", ":after", ":first-line", ":first-letter"]
            .iter()
            .any(|p| lower.contains(p))
}

/// True when the selector only uses tags, classes and ids that exist in the markup, joined by
/// descendant or child combinators, so it can be evaluated against the static DOM.
fn is_static_selector(sel: &str, sheet: &ParsedStylesheet) -> bool {
    if sel.contains([':', '[', '+', '~', '(', '|', '\\']) {
        return false;
    }
    names_present(sel, sheet)
}

/// Checks every `.class` and `#id` token of a combinator-free-of-siblings selector against the markup.
fn names_present(sel: &str, sheet: &ParsedStylesheet) -> bool {
    let mut chars = sel.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '.' || c == '#' {
            let start = i + c.len_utf8();
            let mut end = start;
            while let Some(&(j, nc)) = chars.peek() {
                if nc.is_alphanumeric() || nc == '-' || nc == '_' || !nc.is_ascii() {
                    end = j + nc.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            let name = sel.get(start..end).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let present = if c == '.' {
                sheet.present_classes.contains(name)
            } else {
                sheet.present_ids.contains(name)
            };
            if !present {
                return false;
            }
        }
    }
    true
}

/// Reduces a stateful selector to a static approximation of the elements it may style:
/// pseudo-classes, attribute selectors and names missing from the markup are dropped, sibling
/// compounds (`a + b`, `a ~ b`) collapse to their subject, and trivially satisfied `html`, `body`,
/// `:root` and `*` ancestors are removed. Returns `None` when nothing identifies the subject (for
/// example `[hidden]` or `:focus-visible`), so such rules never mask properties document-wide.
fn reduce_to_static_selector(sel: &str, sheet: &ParsedStylesheet) -> Option<String> {
    let tokens = split_selector_tokens(sel);
    let mut chain: Vec<(char, String)> = Vec::new();
    for (comb, compound) in tokens {
        let reduced = reduce_compound(&compound, sheet);
        if comb == '+' || comb == '~' {
            // The subject keeps the relation its sibling had to the ancestors.
            let prev_comb = chain.pop().map(|(c, _)| c).unwrap_or(' ');
            chain.push((prev_comb, reduced));
        } else {
            chain.push((comb, reduced));
        }
    }
    let (_, subject) = chain.last()?;
    if subject.is_empty() || subject == "*" {
        return None;
    }
    let last = chain.len() - 1;
    let mut out = String::new();
    for (idx, (comb, compound)) in chain.iter().enumerate() {
        let trivial =
            compound.is_empty() || matches!(compound.as_str(), "*" | "html" | "body" | ":root");
        if idx != last && trivial {
            continue;
        }
        if !out.is_empty() {
            out.push_str(if *comb == '>' { " > " } else { " " });
        }
        out.push_str(compound);
    }
    Some(out)
}

/// Splits a selector into `(combinator, compound)` pairs; the first combinator is `' '`.
fn split_selector_tokens(sel: &str) -> Vec<(char, String)> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut pending: char = ' ';
    let mut depth = 0i32;
    for c in sel.chars() {
        match c {
            '(' | '[' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' => {
                depth -= 1;
                current.push(c);
            }
            '>' | '+' | '~' if depth == 0 => {
                if !current.trim().is_empty() {
                    out.push((pending, current.trim().to_string()));
                    current.clear();
                }
                pending = c;
            }
            c if c.is_whitespace() && depth == 0 => {
                if !current.trim().is_empty() {
                    out.push((pending, current.trim().to_string()));
                    current.clear();
                    pending = ' ';
                }
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        out.push((pending, current.trim().to_string()));
    }
    out
}

/// Keeps the tag, present classes and present ids of one compound selector.
fn reduce_compound(compound: &str, sheet: &ParsedStylesheet) -> String {
    let mut out = String::new();
    let chars: Vec<char> = compound.chars().collect();
    let mut i = 0usize;
    let mut first = true;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ':' => {
                // Skip the pseudo-class (and its parenthesised argument).
                i += 1;
                while i < chars.len() && chars[i] == ':' {
                    i += 1;
                }
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-') {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '(' {
                    let mut depth = 0i32;
                    while i < chars.len() {
                        if chars[i] == '(' {
                            depth += 1;
                        } else if chars[i] == ')' {
                            depth -= 1;
                            if depth == 0 {
                                i += 1;
                                break;
                            }
                        }
                        i += 1;
                    }
                }
            }
            '[' => {
                while i < chars.len() && chars[i] != ']' {
                    i += 1;
                }
                i += 1;
            }
            '.' | '#' => {
                let start = i + 1;
                let mut end = start;
                while end < chars.len()
                    && (chars[end].is_alphanumeric() || chars[end] == '-' || chars[end] == '_')
                {
                    end += 1;
                }
                let name: String = chars[start..end].iter().collect();
                let present = if c == '.' {
                    sheet.present_classes.contains(&name)
                } else {
                    sheet.present_ids.contains(&name)
                };
                if present && !name.is_empty() {
                    out.push(c);
                    out.push_str(&name);
                }
                i = end;
            }
            _ if first && (c.is_alphanumeric() || c == '*') => {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '*')
                {
                    i += 1;
                }
                out.push_str(&chars[start..i].iter().collect::<String>());
            }
            _ => {
                i += 1;
            }
        }
        first = false;
    }
    if out.is_empty() {
        "*".to_string()
    } else {
        out
    }
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

/// First block of an imported design's `custom_css` (before the design's own stylesheet).
///
/// Daypress themes style the `.dp-*` classes every native block carries (text colour, section
/// padding, button look, heading scale) and set a 15px root font size. On an imported design those
/// defaults would show wherever the design does not set the property itself (and shrink every
/// `rem`), so the import restores the browser's root size and resets the theme variables:
/// `currentColor` / `0` where a neutral value exists, and `none` elsewhere. `none` is invalid for
/// those properties, which makes the theme declaration compute as `unset` (inherit or initial),
/// exactly as if the theme rule were not there.
pub const IMPORTED_DESIGN_PRELUDE_CSS: &str = "/* Daypress HTML design import: the design's stylesheet below owns colour, spacing and type; theme element defaults step aside. */
html { font-size: 100%; }
:root, body {
  --dp-color-text: currentColor;
  --dp-section-padding: 0;
  --dp-section-border: none;
  --dp-color-heading: none;
  --dp-heading-weight: bold;
  --dp-heading-tracking: normal;
  --dp-heading-line-height: normal;
  --dp-h1-size: 2em; --dp-h2-size: 1.5em; --dp-h3-size: 1.17em; --dp-h4-size: 1em;
  --dp-text-size: none;
  --dp-text-line-height: none;
  --dp-paragraph-gap: none;
  --dp-image-radius: none;
  --dp-flex-gap: normal;
  --dp-grid-gap: normal;
  --dp-column-gap: normal;
  --dp-container-max-width: none;
  --dp-button-bg: none;
  --dp-button-hover-bg: none;
  --dp-button-color: none;
  --dp-button-font-size: none;
  --dp-button-font-weight: none;
  --dp-button-padding: none;
  --dp-button-radius: none;
  --dp-button-border: none;
  --dp-font-accent: 0;
}
";

/// Synthesizes an [`DaystudioKit`] (`DaypressLayout.global_styles`) from the parsed stylesheet.
pub fn build_daystudio_kit(
    sheet: &ParsedStylesheet,
    site_name: Option<String>,
    site_description: Option<String>,
) -> DaystudioKit {
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
        &["oxide", "primary", "brand", "accent-primary", "accent"],
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

    let accent_color = find_var_by_keys(
        &color_vars,
        &["forest", "accent-2", "amber", "oxide-deep", "accent"],
    )
    .unwrap_or_else(|| primary_color.clone());

    let system_colors = vec![
        GlobalColor {
            id: "primary".to_string(),
            title: "Primary Brand".to_string(),
            color: primary_color.clone(),
            dark_color: None,
        },
        GlobalColor {
            id: "secondary".to_string(),
            title: "Secondary / Ink".to_string(),
            color: secondary_color,
            dark_color: None,
        },
        GlobalColor {
            id: "text".to_string(),
            title: "Body Text".to_string(),
            color: text_color,
            dark_color: None,
        },
        GlobalColor {
            id: "accent".to_string(),
            title: "Accent".to_string(),
            color: accent_color,
            dark_color: None,
        },
    ];

    let mut custom_colors = Vec::new();
    for (k, v) in &color_vars {
        let id = sanitize_token_id(k);
        if !matches!(id.as_str(), "primary" | "secondary" | "text" | "accent") {
            let dark_color = sheet
                .dark_vars
                .iter()
                .find(|(dk, _)| dk == k)
                .map(|(_, dv)| dv.trim().to_string())
                .filter(|dv| is_color_value(dv));
            custom_colors.push(GlobalColor {
                id: id.clone(),
                title: humanize_token_title(k),
                color: v.clone(),
                dark_color,
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
                    dark_color: None,
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

    // Without a monospace token the accent style uses the design's body face (instead of loading
    // a font the design never asked for).
    let mono_font = find_var_by_keys(&font_vars, &["mono", "code", "font-mono"])
        .or_else(|| sheet.google_font_families.get(2).cloned())
        .unwrap_or_else(|| {
            if font_vars.is_empty() && sheet.google_font_families.is_empty() {
                "'IBM Plex Mono', monospace".to_string()
            } else {
                body_font.clone()
            }
        });

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
        custom_css.push_str(IMPORTED_DESIGN_PRELUDE_CSS);
        custom_css.push('\n');
        custom_css.push_str(sheet.raw_css.trim());
    }

    DaystudioKit {
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
    resolve_element_styles_with_mask(sheet, el, ancestors).0
}

/// Like [`resolve_element_styles`], also returning the dynamic property mask (see
/// [`dynamic_property_mask`]) so callers can avoid node kinds that inject layout inline.
pub fn resolve_element_styles_with_mask(
    sheet: &ParsedStylesheet,
    el: &DomElement,
    ancestors: &[&DomElement],
) -> (StyleProperties, StyleProperties) {
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
        let target_styles: &mut StyleProperties = match rule.pseudo {
            PseudoTarget::None => &mut styles,
            PseudoTarget::Hover => styles
                .hover
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
            PseudoTarget::Focus => styles
                .focus
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
            PseudoTarget::Active => styles
                .active
                .get_or_insert_with(|| Box::new(StyleProperties::default())),
        };
        for (prop, val) in &rule.declarations {
            apply_css_declaration(target_styles, prop, val);
        }
    }

    // Properties a breakpoint or a state rule can change stay with the design CSS: baking them
    // inline (or as `!important` overrides) would beat `:target`, `.nav-open`, `@media` etc.
    let mask = dynamic_property_mask(sheet, el, ancestors);
    if mask != StyleProperties::default() {
        styles = without_masked_properties(styles, &mask);
        for layer in [&mut styles.hover, &mut styles.focus, &mut styles.active] {
            if let Some(inner) = layer.take() {
                let cleaned = without_masked_properties(*inner, &mask);
                if cleaned != StyleProperties::default() {
                    *layer = Some(Box::new(cleaned));
                }
            }
        }
    }

    // Inline `style="..."` overrides stylesheet declarations
    if let Some(inline_style) = el.attr("style") {
        for (prop, val) in parse_declarations(inline_style) {
            apply_css_declaration(&mut styles, &prop, &val);
        }
    }

    styles.sync_flat_and_nested();
    (styles, mask)
}

/// Collects (as a `StyleProperties` whose set fields name the properties) everything that
/// breakpoint, conditional or state-dependent design rules may set on `el`.
pub fn dynamic_property_mask(
    sheet: &ParsedStylesheet,
    el: &DomElement,
    ancestors: &[&DomElement],
) -> StyleProperties {
    let mut mask = StyleProperties::default();
    for rule in &sheet.dynamic_rules {
        if selector_matches(&rule.selector, el, ancestors) {
            for (prop, val) in &rule.declarations {
                mask_declaration(&mut mask, prop, val);
            }
        }
    }
    mask
}

/// Records the longhand fields a declaration (including shorthands and logical properties the
/// baker ignores) would set.
fn mask_declaration(mask: &mut StyleProperties, prop: &str, val: &str) {
    let any = || Some(String::from("1"));
    match prop {
        "inset" => {
            mask.top = any();
            mask.right = any();
            mask.bottom = any();
            mask.left = any();
        }
        "inset-block" | "inset-block-start" | "inset-block-end" => {
            mask.top = any();
            mask.bottom = any();
        }
        "inset-inline" | "inset-inline-start" | "inset-inline-end" => {
            mask.left = any();
            mask.right = any();
        }
        p if p.starts_with("padding-block") => {
            mask.padding_top = any();
            mask.padding_bottom = any();
        }
        p if p.starts_with("padding-inline") => {
            mask.padding_left = any();
            mask.padding_right = any();
        }
        p if p.starts_with("margin-block") => {
            mask.margin_top = any();
            mask.margin_bottom = any();
        }
        p if p.starts_with("margin-inline") => {
            mask.margin_left = any();
            mask.margin_right = any();
        }
        p if p.starts_with("border-block") => {
            mask.border_top = any();
            mask.border_bottom = any();
            mask.border_width = any();
            mask.border_style = any();
            mask.border_color = any();
        }
        p if p.starts_with("border-inline") => {
            mask.border_left = any();
            mask.border_right = any();
            mask.border_width = any();
            mask.border_style = any();
            mask.border_color = any();
        }
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left"
        | "border-width" | "border-style" | "border-color" => {
            apply_css_declaration(mask, prop, val);
            // A side (or the whole border) changing makes the shorthand parts dynamic too.
            mask.border_width = any();
            mask.border_style = any();
            mask.border_color = any();
            if prop == "border" {
                mask.border_top = any();
                mask.border_right = any();
                mask.border_bottom = any();
                mask.border_left = any();
            }
        }
        p if p.starts_with("border-") && p.ends_with("-color") => {
            mask.border_color = any();
            let side = p.trim_start_matches("border-").trim_end_matches("-color");
            match side {
                "top" => mask.border_top = any(),
                "right" => mask.border_right = any(),
                "bottom" => mask.border_bottom = any(),
                "left" => mask.border_left = any(),
                _ => {}
            }
        }
        "background" | "background-color" | "background-image" => {
            mask.background_color = any();
            mask.background_gradient = any();
            mask.background_image = any();
        }
        "font" => {
            mask.font_family = any();
            mask.font_size = any();
            mask.font_weight = any();
            mask.font_style = any();
            mask.line_height = any();
        }
        "flex-flow" => {
            mask.flex_direction = any();
            mask.flex_wrap = any();
        }
        "place-items" => {
            mask.align_items = any();
            mask.justify_items = any();
        }
        "place-content" => mask.justify_content = any(),
        "place-self" | "justify-self" => mask.align_self = any(),
        "grid-template" | "grid" => {
            mask.grid_template_columns = any();
            mask.grid_template_rows = any();
        }
        "grid-area" => {
            mask.grid_column = any();
            mask.grid_row = any();
        }
        "gap" | "grid-gap" | "column-gap" | "row-gap" | "grid-column-gap" | "grid-row-gap" => {
            mask.gap = any();
            mask.grid_gap = any();
        }
        "text-decoration-line" | "text-decoration-color" | "text-decoration-thickness" => {
            mask.text_decoration = any()
        }
        _ => apply_css_declaration(mask, prop, val),
    }
}

/// Returns `styles` without the flat properties set in `mask`.
fn without_masked_properties(styles: StyleProperties, mask: &StyleProperties) -> StyleProperties {
    let (Ok(serde_json::Value::Object(mut map)), Ok(serde_json::Value::Object(mask_map))) =
        (serde_json::to_value(&styles), serde_json::to_value(mask))
    else {
        return styles;
    };
    for key in mask_map.keys() {
        if !matches!(
            key.as_str(),
            "tablet" | "mobile" | "hover" | "focus" | "active" | "css_classes" | "html_tag"
        ) {
            map.remove(key);
        }
    }
    serde_json::from_value(serde_json::Value::Object(map)).unwrap_or(styles)
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
        "gap" | "grid-gap" => {
            styles.gap = Some(val.to_string());
            styles.grid_gap = Some(val.to_string());
        }
        "column-gap" | "grid-column-gap" | "row-gap" | "grid-row-gap" => {
            // Keep the other axis: `column-gap: 32px` alone is `gap: normal 32px`, not `gap: 32px`.
            let (row, col) = split_gap(styles.gap.as_deref());
            let combined = if prop.contains("column") {
                format!("{row} {val}")
            } else {
                format!("{val} {col}")
            };
            styles.gap = Some(combined.clone());
            styles.grid_gap = Some(combined);
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

/// Splits a `gap` value into its row and column parts (`normal` when unset).
fn split_gap(gap: Option<&str>) -> (String, String) {
    let Some(g) = gap.map(str::trim).filter(|g| !g.is_empty()) else {
        return ("normal".to_string(), "normal".to_string());
    };
    let parts = split_shorthand_values(g);
    match parts.as_slice() {
        [both] => (both.to_string(), both.to_string()),
        [row, col, ..] => (row.to_string(), col.to_string()),
        [] => ("normal".to_string(), "normal".to_string()),
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
        || [
            "rgb(",
            "rgba(",
            "hsl(",
            "hsla(",
            "hwb(",
            "lab(",
            "lch(",
            "oklab(",
            "oklch(",
            "color(",
            "color-mix(",
        ]
        .iter()
        .any(|f| lower.starts_with(f))
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

/// Finds the first variable whose name matches one of `keys`, also accepting the common
/// `color-` / `colour-` / `clr-` / `font-` name prefixes (`--color-ink-2`, `--font-display`).
fn find_var_by_keys(vars: &[(String, String)], keys: &[&str]) -> Option<String> {
    let bare = |k: &str| -> String {
        ["color-", "colour-", "clr-", "font-"]
            .iter()
            .find_map(|p| k.strip_prefix(p))
            .unwrap_or(k)
            .to_string()
    };
    for target in keys {
        if let Some((_, v)) = vars.iter().find(|(k, _)| k == target || bare(k) == *target) {
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
