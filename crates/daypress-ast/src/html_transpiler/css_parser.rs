//! CSS Cascade & Design Token Extractor (`html_transpiler/css_parser.rs`).
//!
//! Turns a design's stylesheet into native styling:
//!
//! - `:root` colours and fonts become the kit's Global Tokens.
//! - Rules that always apply are baked into node styles (which render inline), `:hover` / `:focus`
//!   / `:active` rules into the node's state layers.
//! - Rules in `@media (max-width: ...)` become the node's `tablet` / `mobile` layers, which keep the
//!   design's exact breakpoint.
//! - What a node style cannot express stays in the kit's `custom_css`, rewritten without the
//!   declarations the nodes already carry.
//!
//! A property that depends on something the static resolver cannot see (`:target`, `:checked`,
//! `:not()`, structural pseudo-classes, attribute selectors, sibling combinators, classes that
//! scripts add such as `.nav-open`, `@supports`, `@container`, `@layer`, `min-width`, ...) is
//! recorded through a [`DynamicRule`]: it is left out of the node's styles and layers entirely, so
//! the design's own cascade decides it without any `!important` per-node override competing.

use super::dom_parser::{DomElement, DomNode};
use crate::kit::{ButtonStyle, DaystudioKit, GlobalColor, GlobalTypography};
use crate::render::css::breakpoint_px;
use crate::schema::StyleProperties;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoTarget {
    None,
    Hover,
    Focus,
    Active,
}

/// A `@media (max-width: ...)` condition.
#[derive(Debug, Clone, PartialEq)]
pub struct Breakpoint {
    /// The design's own width, as written (`60rem`, `900px`).
    pub width: String,
    /// The same width in CSS pixels.
    pub px: f32,
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub pseudo: PseudoTarget,
    pub specificity: (u32, u32, u32),
    /// Node-expressible declarations in physical form (see [`native_declarations`]).
    pub declarations: Vec<(String, String)>,
    pub order: usize,
    /// Set when the rule sits in a `@media (max-width: ...)` block.
    pub breakpoint: Option<Breakpoint>,
}

/// A design rule whose effect depends on a condition or runtime state, or that sets something a
/// node style cannot express. `selector` is a static approximation (only tags, classes, ids,
/// descendant and child combinators) used to find the elements it may style.
#[derive(Debug, Clone)]
pub struct DynamicRule {
    pub selector: String,
    pub declarations: Vec<(String, String)>,
}

/// One `property: value` of a design rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
    pub important: bool,
}

impl Declaration {
    fn to_css(&self) -> String {
        format!(
            "{}: {}{};",
            self.property,
            self.value,
            if self.important { " !important" } else { "" }
        )
    }
}

/// The design's stylesheet as parsed, in source order.
#[derive(Debug, Clone, PartialEq)]
pub enum CssStatement {
    /// Kept as written: `@font-face`, `@keyframes`, `@import`, nested rules.
    Verbatim(String),
    /// `@media`, `@supports`, `@container`, `@layer`, ...
    Group {
        prelude: String,
        items: Vec<CssStatement>,
    },
    Style {
        selectors: Vec<String>,
        declarations: Vec<Declaration>,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum BlockContext {
    /// Top-level rules: statically evaluable ones are baked into node styles.
    TopLevel,
    /// Inside `@media (max-width: ...)`: statically evaluable ones become responsive layers.
    MaxWidth(Breakpoint),
    /// Inside any other condition: never baked, always left to the design CSS.
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
    pub statements: Vec<CssStatement>,
    /// Inline scripts' text and whether external scripts exist: together they tell whether a
    /// class missing from the markup can ever appear.
    pub script_text: String,
    pub has_external_script: bool,
    /// `dir="rtl"` documents keep inline-axis logical properties in the design CSS.
    pub right_to_left: bool,
}

/// Walks the DOM tree to collect all `<style>` contents and Google Fonts `<link>` tags,
/// then parses them into a [`ParsedStylesheet`].
pub fn extract_and_parse_styles(root: &DomElement) -> ParsedStylesheet {
    let mut sheet = ParsedStylesheet::default();
    collect_present_names(root, &mut sheet.present_classes, &mut sheet.present_ids);
    collect_scripts(root, &mut sheet);
    sheet.right_to_left = root.children.iter().any(|c| {
        matches!(c, DomNode::Element(e) if e.tag == "html"
            && e.attr("dir").is_some_and(|d| d.eq_ignore_ascii_case("rtl")))
    });
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
    let statements = parse_css_block(&combined_css);
    collect_root_variables(&statements, false, &mut sheet);
    let mut order_counter = 0usize;
    analyze_statements(
        &statements,
        &BlockContext::TopLevel,
        &mut sheet,
        &mut order_counter,
    );
    sheet.statements = statements;
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

fn collect_scripts(el: &DomElement, sheet: &mut ParsedStylesheet) {
    if el.tag == "script" {
        if el.attr("src").is_some() {
            sheet.has_external_script = true;
        }
        sheet.script_text.push_str(&el.text_content());
        sheet.script_text.push('\n');
    }
    for child in &el.children {
        if let DomNode::Element(child_el) = child {
            collect_scripts(child_el, sheet);
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

/// Parses a stylesheet (or a group rule's body) into statements. Purely syntactic.
fn parse_css_block(css: &str) -> Vec<CssStatement> {
    let cleaned = strip_css_comments(css);
    let chars: Vec<(usize, char)> = cleaned.char_indices().collect();
    let byte_at = |idx: usize| chars.get(idx).map(|c| c.0).unwrap_or(cleaned.len());
    let slice = |a: usize, b: usize| cleaned.get(byte_at(a)..byte_at(b)).unwrap_or("");

    let mut statements = Vec::new();
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
            statements.extend(statement_from_block(prelude, body));
            i = j + 1;
            start = i;
            continue;
        } else if c == ';' {
            let stmt = slice(start, i + 1).trim();
            if stmt.starts_with('@') {
                statements.push(CssStatement::Verbatim(stmt.to_string()));
            }
            i += 1;
            start = i;
            continue;
        }
        i += 1;
    }
    statements
}

fn statement_from_block(prelude: &str, body: &str) -> Option<CssStatement> {
    let lower = prelude.to_ascii_lowercase();
    let is_group = [
        "@media",
        "@supports",
        "@container",
        "@layer",
        "@scope",
        "@document",
    ]
    .iter()
    .any(|at| lower.starts_with(at));
    if is_group {
        return Some(CssStatement::Group {
            prelude: prelude.to_string(),
            items: parse_css_block(body),
        });
    }
    if lower.starts_with('@') || has_nested_block(body) {
        return Some(CssStatement::Verbatim(format!("{prelude} {{ {body} }}")));
    }
    let declarations = parse_declaration_list(body);
    if declarations.is_empty() {
        return None;
    }
    Some(CssStatement::Style {
        selectors: split_selectors(prelude)
            .into_iter()
            .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|s| !s.is_empty())
            .collect(),
        declarations,
    })
}

/// CSS nesting: a rule body that holds further rules is kept as written.
fn has_nested_block(body: &str) -> bool {
    let mut quote: Option<char> = None;
    for c in body.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '{' => return true,
            None => {}
        }
    }
    false
}

/// Fills `root_vars` / `dark_vars` first, so rules can be analysed against every token no matter
/// where in the stylesheet it is defined.
fn collect_root_variables(items: &[CssStatement], in_dark: bool, sheet: &mut ParsedStylesheet) {
    for item in items {
        match item {
            CssStatement::Group { prelude, items } if !in_dark => {
                let lower = prelude.to_ascii_lowercase();
                if lower.starts_with("@media")
                    && lower.contains("prefers-color-scheme")
                    && lower.contains("dark")
                {
                    collect_root_variables(items, true, sheet);
                }
            }
            CssStatement::Style {
                selectors,
                declarations,
            } => {
                for sel in selectors {
                    let is_dark_root = sel.contains("data-theme") && sel.contains("dark");
                    let is_root = sel == ":root" || sel == "html";
                    if !is_dark_root && !is_root {
                        continue;
                    }
                    let vars = declarations.iter().filter(|d| d.property.starts_with("--"));
                    for d in vars {
                        if in_dark || is_dark_root {
                            if in_dark && sheet.dark_vars.iter().any(|(k, _)| k == &d.property) {
                                continue;
                            }
                            upsert_var(&mut sheet.dark_vars, &d.property, &d.value);
                        } else {
                            upsert_var(&mut sheet.root_vars, &d.property, &d.value);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn upsert_var(vars: &mut Vec<(String, String)>, name: &str, value: &str) {
    if let Some(existing) = vars.iter_mut().find(|(k, _)| k == name) {
        existing.1 = value.to_string();
    } else {
        vars.push((name.to_string(), value.to_string()));
    }
}

fn analyze_statements(
    items: &[CssStatement],
    context: &BlockContext,
    sheet: &mut ParsedStylesheet,
    order: &mut usize,
) {
    for item in items {
        match item {
            CssStatement::Verbatim(text) => {
                if *context == BlockContext::TopLevel {
                    register_font_import(text, sheet);
                }
            }
            CssStatement::Group { prelude, items } => {
                let child = group_context(prelude, context);
                analyze_statements(items, &child, sheet, order);
            }
            CssStatement::Style {
                selectors,
                declarations,
            } => analyze_style_rule(selectors, declarations, context, sheet, order),
        }
    }
}

fn register_font_import(stmt: &str, sheet: &mut ParsedStylesheet) {
    if !stmt.to_ascii_lowercase().starts_with("@import") {
        return;
    }
    let Some(url) = extract_url_from_import(stmt) else {
        return;
    };
    if url.contains("fonts.googleapis.com") && !sheet.google_font_urls.contains(&url) {
        for fam in extract_families_from_google_url(&url) {
            if !sheet.google_font_families.contains(&fam) {
                sheet.google_font_families.push(fam);
            }
        }
        sheet.google_font_urls.push(url);
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

fn group_context(prelude: &str, context: &BlockContext) -> BlockContext {
    if *context != BlockContext::TopLevel {
        return BlockContext::Conditional;
    }
    parse_max_width_query(prelude).map_or(BlockContext::Conditional, BlockContext::MaxWidth)
}

/// Recognises `@media (max-width: X)`, optionally prefixed by `screen and` / `only screen and` /
/// `all and`, and the range form `(width <= X)`. Any other query returns `None`.
fn parse_max_width_query(prelude: &str) -> Option<Breakpoint> {
    let lower = prelude.to_ascii_lowercase();
    let query = lower.strip_prefix("@media")?.trim();
    if query.contains(',') {
        return None;
    }
    let (media_type, rest) = query.split_once('(')?;
    let feature = rest.strip_suffix(')')?;
    if feature.contains('(') || feature.contains(')') {
        return None;
    }
    let media_type = media_type.split_whitespace().collect::<Vec<_>>().join(" ");
    if !matches!(
        media_type.as_str(),
        "" | "screen and" | "only screen and" | "all and"
    ) {
        return None;
    }
    let feature = feature.trim();
    let width = if let Some(rest) = feature.strip_prefix("max-width") {
        rest.trim_start().strip_prefix(':')?.trim()
    } else {
        feature
            .strip_prefix("width")?
            .trim_start()
            .strip_prefix("<=")?
            .trim()
    };
    Some(Breakpoint {
        px: breakpoint_px(width)?,
        width: width.to_string(),
    })
}

fn analyze_style_rule(
    selectors: &[String],
    declarations: &[Declaration],
    context: &BlockContext,
    sheet: &mut ParsedStylesheet,
    order: &mut usize,
) {
    for sel in selectors {
        // Pseudo-element rules style generated boxes, not the element; `[data-theme]` overrides
        // are dark-mode token sets. Both only ever live in the design CSS.
        if selector_is_dead(sel, sheet)
            || is_pseudo_element_selector(sel)
            || sel.contains("[data-theme")
        {
            continue;
        }

        let (clean_sel, pseudo) = extract_pseudo_target(sel);
        if selector_is_baked(&clean_sel, pseudo, context, sheet) {
            let mut native = Vec::new();
            let mut left_to_css = Vec::new();
            for decl in declarations {
                let expressed = native_declarations(decl, sheet);
                if expressed.is_empty() {
                    if masks_any_field(&decl.property, &decl.value) {
                        left_to_css.push((decl.property.clone(), decl.value.clone()));
                    }
                } else {
                    native.extend(expressed);
                }
            }
            if !native.is_empty() {
                *order += 1;
                sheet.rules.push(CssRule {
                    selector: clean_sel.clone(),
                    pseudo,
                    specificity: compute_specificity(&clean_sel),
                    declarations: native,
                    order: *order,
                    breakpoint: match context {
                        BlockContext::MaxWidth(bp) => Some(bp.clone()),
                        _ => None,
                    },
                });
            }
            if !left_to_css.is_empty() {
                sheet.dynamic_rules.push(DynamicRule {
                    selector: clean_sel,
                    declarations: left_to_css,
                });
            }
        } else if let Some(reduced) = reduce_to_static_selector(sel, sheet) {
            sheet.dynamic_rules.push(DynamicRule {
                selector: reduced,
                declarations: declarations
                    .iter()
                    .map(|d| (d.property.clone(), d.value.clone()))
                    .collect(),
            });
        }
    }
}

/// Whether a rule with this (pseudo-free) selector is expressed in node styles. A bare `*` is not:
/// it would stamp the design's reset on every node.
fn selector_is_baked(
    clean_sel: &str,
    pseudo: PseudoTarget,
    context: &BlockContext,
    sheet: &ParsedStylesheet,
) -> bool {
    let in_baked_context = match context {
        BlockContext::TopLevel => true,
        BlockContext::MaxWidth(_) => pseudo == PseudoTarget::None,
        BlockContext::Conditional => false,
    };
    in_baked_context && clean_sel != "*" && is_static_selector(clean_sel, sheet)
}

fn is_pseudo_element_selector(sel: &str) -> bool {
    let lower = sel.to_ascii_lowercase();
    lower.contains("::")
        || [":before", ":after", ":first-line", ":first-letter"]
            .iter()
            .any(|p| lower.contains(p))
}

/// True when the selector needs a class or id that neither the markup nor any script can provide,
/// so the rule can never apply. Names inside `:not()` and other functional pseudo-classes or
/// attribute selectors do not count: their absence does not stop the selector from matching.
fn selector_is_dead(sel: &str, sheet: &ParsedStylesheet) -> bool {
    if sheet.has_external_script {
        return false;
    }
    let chars: Vec<char> = sel.chars().collect();
    let mut i = 0usize;
    let mut depth = 0i32;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '.' | '#' if depth == 0 => {
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
                if !name.is_empty() && !present && !sheet.script_text.contains(&name) {
                    return true;
                }
                i = end;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// A value's literal behind `var(--x)` references to `:root` tokens (fallbacks are ignored).
fn resolve_var_chain(val: &str, sheet: &ParsedStylesheet) -> String {
    let mut current = val.trim().to_string();
    for _ in 0..8 {
        let Some(name) = single_var_name(&current) else {
            break;
        };
        match sheet.root_vars.iter().find(|(k, _)| *k == name) {
            Some((_, v)) => current = v.trim().to_string(),
            None => break,
        }
    }
    current
}

/// `var(--x)` or `var(--x, fallback)` -> `--x`.
fn single_var_name(val: &str) -> Option<String> {
    let inner = val.trim().strip_prefix("var(")?.strip_suffix(')')?;
    let name = split_top_level_char(inner, ',').first()?.trim().to_string();
    (name.starts_with("--") && !inner.contains('(')).then_some(name)
}

/// A comma-separated list of nothing but gradient functions.
fn is_gradient_list(val: &str) -> bool {
    !val.is_empty()
        && split_top_level_char(val, ',').iter().all(|layer| {
            let parts = split_shorthand_values(layer.trim());
            let lower = layer.trim().to_ascii_lowercase();
            parts.len() == 1 && lower.ends_with(')') && lower.contains("gradient(")
        })
}

/// A single colour-like token (no image, position, size or repeat parts).
fn is_plain_background_colour(val: &str) -> bool {
    let lower = val.trim().to_ascii_lowercase();
    !lower.is_empty()
        && split_shorthand_values(&lower).len() == 1
        && !lower.contains("url(")
        && !lower.contains("gradient(")
        && !lower.contains("image-set(")
}

/// The physical longhand declarations for one design declaration, or nothing when a node style
/// cannot express it exactly. Those stay in the design CSS (and mask the fields they overlap).
fn native_declarations(decl: &Declaration, sheet: &ParsedStylesheet) -> Vec<(String, String)> {
    if decl.important || decl.property.starts_with("--") {
        return Vec::new();
    }
    let prop = decl.property.as_str();
    let val = decl.value.as_str();
    if let Some(expanded) = expand_logical_property(prop, val, sheet.right_to_left) {
        return expanded;
    }
    if prop == "background" {
        let resolved = resolve_var_chain(val, sheet);
        if is_gradient_list(&resolved) {
            return vec![("background-image".to_string(), val.to_string())];
        }
        if is_plain_background_colour(&resolved) && !resolved.starts_with("var(") {
            return vec![("background".to_string(), val.to_string())];
        }
        return Vec::new();
    }
    let mut probe = StyleProperties::default();
    apply_css_declaration(&mut probe, prop, val);
    if probe == StyleProperties::default() {
        Vec::new()
    } else {
        vec![(prop.to_string(), val.to_string())]
    }
}

/// Rewrites a logical property to physical ones (left-to-right, horizontal writing mode), or
/// returns `None` when `prop` is not logical or the document is right-to-left.
fn expand_logical_property(
    prop: &str,
    val: &str,
    right_to_left: bool,
) -> Option<Vec<(String, String)>> {
    let size_name = match prop {
        "inline-size" => Some("width"),
        "min-inline-size" => Some("min-width"),
        "max-inline-size" => Some("max-width"),
        "block-size" => Some("height"),
        "min-block-size" => Some("min-height"),
        "max-block-size" => Some("max-height"),
        _ => None,
    };
    if let Some(name) = size_name {
        return (!right_to_left || !prop.contains("inline"))
            .then(|| vec![(name.to_string(), val.to_string())]);
    }
    if prop == "inset" {
        let parts = split_shorthand_values(val);
        let (t, r, b, l) = match parts.as_slice() {
            [all] => (*all, *all, *all, *all),
            [v, h] => (*v, *h, *v, *h),
            [t, h, b] => (*t, *h, *b, *h),
            [t, r, b, l] => (*t, *r, *b, *l),
            _ => return None,
        };
        return Some(vec![
            ("top".to_string(), t.to_string()),
            ("right".to_string(), r.to_string()),
            ("bottom".to_string(), b.to_string()),
            ("left".to_string(), l.to_string()),
        ]);
    }
    for base in ["padding", "margin", "inset", "border"] {
        let Some(rest) = prop.strip_prefix(base).and_then(|r| r.strip_prefix('-')) else {
            continue;
        };
        let (sides, is_inline): (&[&str], bool) = match rest {
            "block" => (&["top", "bottom"], false),
            "block-start" => (&["top"], false),
            "block-end" => (&["bottom"], false),
            "inline" => (&["left", "right"], true),
            "inline-start" => (&["left"], true),
            "inline-end" => (&["right"], true),
            _ => return None,
        };
        if is_inline && right_to_left {
            return None;
        }
        let parts = if base == "border" {
            vec![val]
        } else {
            split_shorthand_values(val)
        };
        let values: Vec<&str> = match (sides.len(), parts.as_slice()) {
            (1, [v]) => vec![*v],
            (2, [v]) => vec![*v, *v],
            (2, [a, b]) => vec![*a, *b],
            _ => return None,
        };
        return Some(
            sides
                .iter()
                .zip(values)
                .map(|(side, v)| {
                    let name = if base == "inset" {
                        (*side).to_string()
                    } else {
                        format!("{base}-{side}")
                    };
                    (name, v.to_string())
                })
                .collect(),
        );
    }
    None
}

fn masks_any_field(prop: &str, val: &str) -> bool {
    let mut probe = StyleProperties::default();
    mask_declaration(&mut probe, prop, val);
    probe != StyleProperties::default()
}

/// Matches an element without a `class` attribute. Native blocks always carry `dp-*` classes, so
/// after import only the static resolver can still honour this selector.
const NO_CLASS_PSEUDO: &str = ":not([class])";

/// True when the selector only uses tags, classes and ids that exist in the markup, joined by
/// descendant or child combinators, so it can be evaluated against the static DOM.
fn is_static_selector(sel: &str, sheet: &ParsedStylesheet) -> bool {
    let sel = sel.replace(NO_CLASS_PSEUDO, "");
    if sel.contains([':', '[', '+', '~', '(', '|', '\\']) {
        return false;
    }
    names_present(&sel, sheet)
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

    let sel = sel.replace(NO_CLASS_PSEUDO, ".");
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
    parse_declaration_list(body)
        .into_iter()
        .map(|d| (d.property, d.value))
        .collect()
}

fn parse_declaration_list(body: &str) -> Vec<Declaration> {
    let mut out = Vec::new();
    for raw_decl in split_top_level_char(body, ';') {
        let decl = raw_decl.trim();
        if decl.is_empty() {
            continue;
        }
        if let Some((prop, val)) = decl.split_once(':') {
            let name = prop.trim();
            // Custom property names are case-sensitive.
            let property = if name.starts_with("--") {
                name.to_string()
            } else {
                name.to_ascii_lowercase()
            };
            let (value, important) = split_important(val.trim());
            if !property.is_empty() && !value.is_empty() {
                out.push(Declaration {
                    property,
                    value,
                    important,
                });
            }
        }
    }
    out
}

/// Separates a trailing `!important` (in any case, with or without spaces) from a value.
fn split_important(val: &str) -> (String, bool) {
    if let Some(pos) = val.rfind('!') {
        let tail = val.get(pos + 1..).unwrap_or("").trim();
        if tail.eq_ignore_ascii_case("important") {
            return (val.get(..pos).unwrap_or("").trim().to_string(), true);
        }
    }
    (val.to_string(), false)
}

fn split_selectors(prelude: &str) -> Vec<&str> {
    split_top_level_char(prelude, ',')
}

pub fn split_top_level_char(s: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut start = 0usize;
    let mut escaped = false;

    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
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
/// The public theme's element defaults skip imported blocks (`.dp-design`), so all that is left
/// to undo is the 15px root font size, which would shrink every `rem` in the design.
pub const IMPORTED_DESIGN_PRELUDE_CSS: &str = "/* Daypress HTML design import: the design's stylesheet below owns colour, spacing and type. */
html { font-size: 100%; }
";

/// The public theme's selector for the page background and text colour.
const THEME_BODY_SELECTOR: &str = "html.daystudio-theme-active body";

/// What the node mapper learned about each element, used to drop from `custom_css` whatever the
/// node styles already express.
#[derive(Debug, Default)]
pub struct DesignCoverage<'a> {
    /// Every element the design's selectors can match, with its ancestors (root to parent).
    pub elements: Vec<(&'a DomElement, Vec<&'a DomElement>)>,
    /// Elements whose styles were written onto a node, with the fields left to the design CSS.
    pub nodes: HashMap<*const DomElement, HashSet<String>>,
}

impl<'a> DesignCoverage<'a> {
    /// Records `nodes` and their descendants as elements below `ancestors`.
    pub fn add_elements(&mut self, nodes: &'a [DomNode], ancestors: &[&'a DomElement]) {
        for node in nodes {
            if let DomNode::Element(el) = node {
                self.elements.push((el, ancestors.to_vec()));
                let mut below = ancestors.to_vec();
                below.push(el);
                self.add_elements(&el.children, &below);
            }
        }
    }
}

/// True when a node style carries this declaration on the element whose left-to-CSS fields are
/// `masked`.
fn declaration_is_covered(
    decl: &Declaration,
    masked: &HashSet<String>,
    sheet: &ParsedStylesheet,
) -> bool {
    let native = native_declarations(decl, sheet);
    if native.is_empty() {
        return false;
    }
    let mut fields = StyleProperties::default();
    for (p, v) in &native {
        mask_declaration(&mut fields, p, v);
    }
    !field_names(&fields).iter().any(|f| masked.contains(f))
}

/// The design's stylesheet for `custom_css`: everything the node styles cannot express.
///
/// Rules that cannot match are dropped, as is each declaration every matched element already
/// carries as a node style. `:root` colours and fonts that became Global Tokens are written as
/// references to those tokens (`aliases`), so editing a token restyles the page.
fn serialize_design_css(
    sheet: &ParsedStylesheet,
    coverage: &DesignCoverage<'_>,
    aliases: &HashMap<String, String>,
) -> String {
    let mut out = String::new();
    write_statements(
        &sheet.statements,
        &BlockContext::TopLevel,
        sheet,
        coverage,
        aliases,
        &mut out,
    );
    out
}

fn write_statements(
    items: &[CssStatement],
    context: &BlockContext,
    sheet: &ParsedStylesheet,
    coverage: &DesignCoverage<'_>,
    aliases: &HashMap<String, String>,
    out: &mut String,
) {
    for item in items {
        match item {
            CssStatement::Verbatim(text) => {
                // `@import` must come first: see `build_daystudio_kit`.
                if !text.to_ascii_lowercase().starts_with("@import") {
                    out.push_str(text);
                    out.push('\n');
                }
            }
            CssStatement::Group { prelude, items } => {
                let child = group_context(prelude, context);
                let mut inner = String::new();
                write_statements(items, &child, sheet, coverage, aliases, &mut inner);
                if !inner.trim().is_empty() {
                    out.push_str(&format!("{prelude} {{\n{inner}}}\n"));
                }
            }
            CssStatement::Style {
                selectors,
                declarations,
            } => write_style_rule(
                selectors,
                declarations,
                context,
                sheet,
                coverage,
                aliases,
                out,
            ),
        }
    }
}

fn write_style_rule(
    selectors: &[String],
    declarations: &[Declaration],
    context: &BlockContext,
    sheet: &ParsedStylesheet,
    coverage: &DesignCoverage<'_>,
    aliases: &HashMap<String, String>,
    out: &mut String,
) {
    // Selectors keeping the same declarations share one rule.
    let mut groups: Vec<(Vec<&str>, Vec<String>)> = Vec::new();
    for sel in selectors {
        let kept = kept_declarations(sel, declarations, context, sheet, coverage, aliases);
        if kept.is_empty() {
            continue;
        }
        match groups.last_mut() {
            Some((sels, decls)) if *decls == kept => sels.push(sel),
            _ => groups.push((vec![sel], kept)),
        }
    }
    for (sels, decls) in groups {
        // The theme's `body` rule outranks a plain `body`, and comes first: match it and win.
        let list: Vec<&str> = sels
            .iter()
            .flat_map(|s| {
                if *s == "body" {
                    vec![*s, THEME_BODY_SELECTOR]
                } else {
                    vec![*s]
                }
            })
            .collect();
        out.push_str(&format!("{} {{ {} }}\n", list.join(", "), decls.join(" ")));
    }
}

/// The declarations of `sel` that must stay in `custom_css`, as CSS text.
fn kept_declarations(
    sel: &str,
    declarations: &[Declaration],
    context: &BlockContext,
    sheet: &ParsedStylesheet,
    coverage: &DesignCoverage<'_>,
    aliases: &HashMap<String, String>,
) -> Vec<String> {
    if selector_is_dead(sel, sheet) {
        return Vec::new();
    }
    let is_root = (sel == ":root" || sel == "html") && *context == BlockContext::TopLevel;
    let write = |d: &Declaration| -> String {
        match aliases.get(&d.property).filter(|_| is_root) {
            Some(alias) => format!("{}: {};", d.property, alias),
            None => d.to_css(),
        }
    };
    let (clean_sel, pseudo) = extract_pseudo_target(sel);
    if is_pseudo_element_selector(sel)
        || sel.contains("[data-theme")
        || !selector_is_baked(&clean_sel, pseudo, context, sheet)
    {
        return declarations.iter().map(write).collect();
    }
    let matched: Vec<_> = coverage
        .elements
        .iter()
        .filter(|(el, ancestors)| selector_matches(&clean_sel, el, ancestors))
        .collect();
    declarations
        .iter()
        .filter(|d| {
            !matched.iter().all(|(el, _)| {
                coverage
                    .nodes
                    .get(&(*el as *const DomElement))
                    .is_some_and(|masked| declaration_is_covered(d, masked, sheet))
            })
        })
        .map(write)
        .collect()
}

/// One of the four system token slots, and the design variable it came from.
struct TokenSlot {
    value: String,
    source: Option<String>,
}

/// The last value `prop` gets in a plain top-level rule for exactly one of `selectors`.
fn rule_value(sheet: &ParsedStylesheet, selectors: &[&str], prop: &str) -> Option<String> {
    sheet
        .rules
        .iter()
        .filter(|r| {
            r.pseudo == PseudoTarget::None
                && r.breakpoint.is_none()
                && selectors.contains(&r.selector.as_str())
        })
        .flat_map(|r| r.declarations.iter())
        .rfind(|(p, _)| p == prop)
        .map(|(_, v)| v.clone())
}

/// Turns a design value into a token slot: the literal it resolves to, and the variable behind it.
fn slot_from_value(raw: &str, sheet: &ParsedStylesheet) -> TokenSlot {
    TokenSlot {
        value: resolve_var_chain(raw, sheet),
        source: single_var_name(raw).map(|n| n.trim_start_matches('-').to_ascii_lowercase()),
    }
}

fn slot_from_keys(vars: &[(String, String)], keys: &[&str]) -> Option<TokenSlot> {
    find_var_by_keys(vars, keys).map(|(key, value)| TokenSlot {
        value,
        source: Some(key),
    })
}

/// Rough chroma check for picking a brand colour from an unnamed palette.
fn is_saturated_colour(val: &str) -> bool {
    let lower = val.trim().to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix('#') {
        let long: String = if hex.len() == 3 || hex.len() == 4 {
            hex.chars().take(3).flat_map(|c| [c, c]).collect()
        } else {
            hex.chars().take(6).collect()
        };
        let channel =
            |i: usize| u8::from_str_radix(long.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
        let (r, g, b) = (channel(0), channel(2), channel(4));
        return r.max(g).max(b) - r.min(g).min(b) > 28;
    }
    let numbers: Vec<f32> = lower
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter_map(|n| n.parse().ok())
        .collect();
    if lower.starts_with("oklch(") || lower.starts_with("lch(") {
        return numbers.get(1).is_some_and(|c| *c > 0.03);
    }
    if lower.starts_with("hsl") {
        return numbers.get(1).is_some_and(|s| *s > 12.0);
    }
    if lower.starts_with("rgb") {
        if let [r, g, b, ..] = numbers.as_slice() {
            return r.max(*g).max(*b) - r.min(*g).min(*b) > 28.0;
        }
    }
    false
}

/// A named CSS colour used as a token value (`--ink: black`).
fn is_named_colour(val: &str) -> bool {
    const NAMES: &[&str] = &[
        "black",
        "white",
        "gray",
        "grey",
        "silver",
        "red",
        "maroon",
        "orange",
        "gold",
        "yellow",
        "olive",
        "lime",
        "green",
        "teal",
        "aqua",
        "cyan",
        "blue",
        "navy",
        "fuchsia",
        "magenta",
        "purple",
        "pink",
        "brown",
        "tan",
        "beige",
        "ivory",
        "coral",
        "salmon",
        "crimson",
        "indigo",
        "violet",
        "turquoise",
        "khaki",
        "lavender",
        "plum",
        "orchid",
        "sienna",
        "chocolate",
        "copper",
        "peru",
        "tomato",
        "wheat",
        "snow",
        "linen",
        "azure",
        "transparent",
    ];
    NAMES.contains(&val.trim().to_ascii_lowercase().as_str())
}

/// Font-related values of a heading or body rule, for the design's type scale.
fn typography_from_rule(
    sheet: &ParsedStylesheet,
    selectors: &[&str],
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    (
        rule_value(sheet, selectors, "font-size"),
        rule_value(sheet, selectors, "font-weight"),
        rule_value(sheet, selectors, "line-height"),
        rule_value(sheet, selectors, "letter-spacing"),
    )
}

/// Synthesizes an [`DaystudioKit`] (`DaypressLayout.global_styles`) from the parsed stylesheet.
///
/// The system colours and fonts come from what the design actually uses (`body`, headings, its
/// buttons) and its `:root` tokens; Daypress defaults only remain when the design says nothing.
pub fn build_daystudio_kit(
    sheet: &ParsedStylesheet,
    coverage: &DesignCoverage<'_>,
    site_name: Option<String>,
    site_description: Option<String>,
) -> DaystudioKit {
    let mut color_vars: Vec<(String, String)> = Vec::new();
    let mut font_vars: Vec<(String, String)> = Vec::new();
    // Variables holding a literal value, which the kit can own. `--a: var(--b)` follows `--b`.
    let mut token_vars: HashMap<String, String> = HashMap::new();
    let mut content_width = 1320u32;

    for (name, val) in &sheet.root_vars {
        let key = name.trim_start_matches('-').to_ascii_lowercase();
        let resolved = resolve_var_chain(val, sheet);
        let is_literal = !val.trim().starts_with("var(");

        if is_color_value(&resolved) || (is_named_colour(&resolved) && is_colour_token_name(&key)) {
            color_vars.push((key.clone(), resolved));
            if is_literal {
                token_vars.insert(key, name.clone());
            }
        } else if is_font_family_var(&key, &resolved) {
            font_vars.push((key.clone(), resolved));
            if is_literal {
                token_vars.insert(key, name.clone());
            }
        } else if matches!(
            key.as_str(),
            "wide" | "max-width" | "container" | "container-width" | "content-width"
        ) {
            if let Some(px) = parse_px_u32(&resolved) {
                if (600..=2400).contains(&px) {
                    content_width = px;
                }
            }
        }
    }

    let has_design_colours = !color_vars.is_empty()
        || sheet
            .rules
            .iter()
            .any(|r| r.declarations.iter().any(|(p, _)| p == "color"));
    let design_literal = |raw: Option<String>| raw.map(|v| slot_from_value(&v, sheet));

    // 1. System Colors (primary, secondary, text, accent) + Custom Colors
    let text_slot =
        design_literal(rule_value(sheet, &["body", "html", ":root"], "color")).or_else(|| {
            slot_from_keys(
                &color_vars,
                &["ink-2", "text", "body", "fg", "foreground", "ink"],
            )
        });

    let secondary_slot = design_literal(rule_value(sheet, &["h1", "h2"], "color"))
        .or_else(|| {
            slot_from_keys(
                &color_vars,
                &["ink", "secondary", "heading", "dark", "foreground", "fg"],
            )
        })
        .or_else(|| {
            text_slot.as_ref().map(|t| TokenSlot {
                value: t.value.clone(),
                source: t.source.clone(),
            })
        });

    let button_background = |sheet: &ParsedStylesheet| {
        rule_value(sheet, &[".btn-primary", ".btn", ".button"], "background")
            .or_else(|| {
                rule_value(
                    sheet,
                    &[".btn-primary", ".btn", ".button"],
                    "background-color",
                )
            })
            .filter(|v| !v.contains("gradient("))
    };
    let first_saturated = |skip: &[Option<String>]| {
        color_vars
            .iter()
            .find(|(k, v)| is_saturated_colour(v) && !skip.contains(&Some(k.clone())))
            .map(|(k, v)| TokenSlot {
                value: v.clone(),
                source: Some(k.clone()),
            })
    };
    let primary_slot = slot_from_keys(
        &color_vars,
        &["oxide", "primary", "brand", "accent-primary", "accent"],
    )
    .or_else(|| design_literal(button_background(sheet)))
    .or_else(|| first_saturated(&[]))
    .or_else(|| {
        color_vars.first().map(|(k, v)| TokenSlot {
            value: v.clone(),
            source: Some(k.clone()),
        })
    });

    let used_for_type_or_brand: Vec<Option<String>> = [&primary_slot, &secondary_slot, &text_slot]
        .iter()
        .map(|s| s.as_ref().and_then(|t| t.source.clone()))
        .collect();
    let accent_slot = slot_from_keys(
        &color_vars,
        &[
            "forest",
            "accent-2",
            "amber",
            "oxide-deep",
            "accent",
            "highlight",
        ],
    )
    .or_else(|| first_saturated(&used_for_type_or_brand));

    let fallback = |slot: &Option<TokenSlot>, other: &Option<TokenSlot>, default: &str| {
        slot.as_ref()
            .or(other.as_ref())
            .map(|s| s.value.clone())
            .unwrap_or_else(|| default.to_string())
    };
    let default_of = |daypress: &'static str| {
        if has_design_colours {
            "currentColor"
        } else {
            daypress
        }
    };
    let primary_color = fallback(&primary_slot, &text_slot, default_of("#C8371A"));
    let secondary_color = fallback(&secondary_slot, &text_slot, default_of("#13120F"));
    let text_color = fallback(&text_slot, &secondary_slot, default_of("#13120F"));
    let accent_color = fallback(&accent_slot, &primary_slot, default_of("#C8371A"));

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

    // Each variable a system slot came from is owned by that slot, and is not a custom token too.
    let mut aliases: HashMap<String, String> = HashMap::new();
    let mut alias_to = |key: &Option<String>, target: String| {
        if let Some(name) = key.as_ref().and_then(|k| token_vars.get(k)) {
            aliases.entry(name.clone()).or_insert(target);
        }
    };
    for (slot, id) in [
        (&primary_slot, "primary"),
        (&secondary_slot, "secondary"),
        (&text_slot, "text"),
        (&accent_slot, "accent"),
    ] {
        alias_to(
            &slot.as_ref().and_then(|s| s.source.clone()),
            format!("var(--e-global-color-{id})"),
        );
    }
    let color_owned: HashSet<Option<String>> =
        [&primary_slot, &secondary_slot, &text_slot, &accent_slot]
            .iter()
            .map(|s| s.as_ref().and_then(|t| t.source.clone()))
            .collect();

    let mut custom_colors = Vec::new();
    for (k, v) in &color_vars {
        let id = sanitize_token_id(k);
        if matches!(id.as_str(), "primary" | "secondary" | "text" | "accent")
            || color_owned.contains(&Some(k.clone()))
        {
            continue;
        }
        let dark_color = sheet
            .dark_vars
            .iter()
            .find(|(dk, _)| dk.trim_start_matches('-').eq_ignore_ascii_case(k))
            .map(|(_, dv)| dv.trim().to_string())
            .filter(|dv| is_color_value(dv));
        alias_to(&Some(k.clone()), format!("var(--e-global-color-{id})"));
        custom_colors.push(GlobalColor {
            id: id.clone(),
            title: humanize_token_title(k),
            color: v.clone(),
            dark_color,
        });
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
    let body_slot = design_literal(rule_value(sheet, &["body", "html", ":root"], "font-family"))
        .or_else(|| {
            slot_from_keys(
                &font_vars,
                &["sans", "body", "text", "font-sans", "font-body", "base"],
            )
        })
        .or_else(|| {
            sheet.google_font_families.get(1).map(|f| TokenSlot {
                value: f.clone(),
                source: None,
            })
        });

    let heading_slot = design_literal(rule_value(sheet, &["h1", "h1, h2"], "font-family"))
        .or_else(|| {
            slot_from_keys(
                &font_vars,
                &["serif", "display", "heading", "font-heading", "headline"],
            )
        })
        .or_else(|| {
            sheet.google_font_families.first().map(|f| TokenSlot {
                value: f.clone(),
                source: None,
            })
        });

    let subheading_slot = design_literal(rule_value(sheet, &["h2", "h3"], "font-family"));

    let mono_slot = slot_from_keys(&font_vars, &["mono", "code", "font-mono"]).or_else(|| {
        sheet.google_font_families.get(2).map(|f| TokenSlot {
            value: f.clone(),
            source: None,
        })
    });

    let font_of = |slot: &Option<TokenSlot>, fallback: &Option<TokenSlot>| -> String {
        slot.as_ref()
            .or(fallback.as_ref())
            .map(|s| s.value.clone())
            .unwrap_or_else(|| "inherit".to_string())
    };
    let heading_font = font_of(&heading_slot, &body_slot);
    let body_font = font_of(&body_slot, &heading_slot);
    let subheading_font = subheading_slot
        .as_ref()
        .map(|s| s.value.clone())
        .unwrap_or_else(|| body_font.clone());
    let mono_font = font_of(&mono_slot, &body_slot);

    for (slot, id) in [
        (&heading_slot, "primary"),
        (&subheading_slot, "secondary"),
        (&body_slot, "text"),
        (&mono_slot, "accent"),
    ] {
        alias_to(
            &slot.as_ref().and_then(|s| s.source.clone()),
            format!("var(--e-global-typography-{id}-font-family)"),
        );
    }
    let font_owned: HashSet<Option<String>> =
        [&heading_slot, &subheading_slot, &body_slot, &mono_slot]
            .iter()
            .map(|s| s.as_ref().and_then(|t| t.source.clone()))
            .collect();

    let scale = |id: &str, title: &str, font: String, selectors: &[&str]| {
        let (font_size, font_weight, line_height, letter_spacing) =
            typography_from_rule(sheet, selectors);
        GlobalTypography {
            id: id.to_string(),
            title: title.to_string(),
            font_family: font,
            font_size,
            font_weight,
            line_height,
            letter_spacing,
        }
    };
    let system_typography = vec![
        scale(
            "primary",
            "Display / Editorial Heading",
            heading_font,
            &["h1"],
        ),
        scale(
            "secondary",
            "Section / Deck Heading",
            subheading_font,
            &["h2"],
        ),
        scale("text", "Body Copy", body_font, &["body"]),
        scale("accent", "Monospace / Spec Labels", mono_font, &[]),
    ];

    let mut custom_typography = Vec::new();
    for (k, v) in &font_vars {
        let id = sanitize_token_id(k);
        if matches!(id.as_str(), "primary" | "secondary" | "text" | "accent")
            || font_owned.contains(&Some(k.clone()))
        {
            continue;
        }
        alias_to(
            &Some(k.clone()),
            format!("var(--e-global-typography-{id}-font-family)"),
        );
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

    // 3. Button Style (the design's own, for buttons added later in the builder)
    let mut btn_bg = Some(primary_color);
    let mut btn_color = None;
    let mut btn_radius = None;
    for rule in &sheet.rules {
        if matches!(rule.selector.as_str(), ".btn-primary" | ".btn" | ".button")
            && rule.pseudo == PseudoTarget::None
            && rule.breakpoint.is_none()
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

    // 4. Custom CSS: font imports first (`@import` is ignored after any other rule), the theme
    // reset, then what remains of the design's stylesheet.
    let mut custom_css = String::new();
    for url in &sheet.google_font_urls {
        custom_css.push_str(&format!("@import url('{}');\n", url));
    }
    for statement in &sheet.statements {
        if let CssStatement::Verbatim(text) = statement {
            let is_import = text.to_ascii_lowercase().starts_with("@import");
            if is_import && !text.contains("fonts.googleapis.com") {
                custom_css.push_str(text);
                custom_css.push('\n');
            }
        }
    }
    if !sheet.raw_css.trim().is_empty() {
        if !custom_css.is_empty() {
            custom_css.push('\n');
        }
        custom_css.push_str(IMPORTED_DESIGN_PRELUDE_CSS);
        custom_css.push('\n');
        custom_css.push_str(serialize_design_css(sheet, coverage, &aliases).trim());
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

/// The result of running the design's cascade for one element.
#[derive(Debug, Clone, Default)]
pub struct ResolvedElement {
    /// Base values, responsive layers and state layers for the node.
    pub styles: StyleProperties,
    /// Fields (set in this struct) left to the design CSS, not written to the node.
    pub mask: StyleProperties,
    /// The plain cascade of the design's unconditional rules, masking aside.
    pub unmasked: StyleProperties,
}

/// Resolves all matching CSS rules + inline `style="..."` for `el` into a [`StyleProperties`].
pub fn resolve_element_styles(
    sheet: &ParsedStylesheet,
    el: &DomElement,
    ancestors: &[&DomElement],
) -> StyleProperties {
    resolve_element(sheet, el, ancestors).styles
}

/// Runs the cascade for `el`: unconditional rules give the base values, `max-width` rules the
/// responsive layers, and inline `style` wins over both, as it does in the browser.
pub fn resolve_element(
    sheet: &ParsedStylesheet,
    el: &DomElement,
    ancestors: &[&DomElement],
) -> ResolvedElement {
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

    let unmasked = cascade(&matched, None);

    // Properties a state rule can change stay with the design CSS: baking them inline (or as
    // `!important` overrides) would beat `:target`, `.nav-open`, `@supports` etc.
    let mut mask = dynamic_property_mask(sheet, el, ancestors);
    let (tablet, mobile) = responsive_layers(&matched, &unmasked, &mut mask);

    let mut styles = unmasked.clone();
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
    styles.tablet = tablet;
    styles.mobile = mobile;

    if let Some(cls) = el.class_str() {
        styles.css_classes = Some(cls.to_string());
    }

    // Inline `style="..."` beats stylesheet declarations at every width. What a node style cannot
    // express stays inline through `custom_css` declarations.
    if let Some(inline_style) = el.attr("style") {
        let mut inline_fields = StyleProperties::default();
        let mut inline_custom = Vec::new();
        for decl in parse_declaration_list(inline_style) {
            let native = native_declarations(&decl, sheet);
            if native.is_empty() {
                inline_custom.push(decl.to_css());
            }
            for (p, v) in native {
                apply_css_declaration(&mut styles, &p, &v);
                apply_css_declaration(&mut inline_fields, &p, &v);
            }
        }
        for layer in [&mut styles.tablet, &mut styles.mobile] {
            if let Some(inner) = layer.take() {
                let cleaned = without_masked_properties(*inner, &inline_fields);
                if field_names(&cleaned).iter().any(|f| f != "breakpoint") {
                    *layer = Some(Box::new(cleaned));
                }
            }
        }
        if !inline_custom.is_empty() {
            styles.custom_css = Some(inline_custom.join(" "));
        }
    }

    styles.sync_flat_and_nested();
    ResolvedElement {
        styles,
        mask,
        unmasked,
    }
}

/// Applies matched rules in cascade order: unconditional ones plus, when `viewport` is given, the
/// `max-width` rules active at that width.
fn cascade(matched: &[&CssRule], viewport: Option<f32>) -> StyleProperties {
    let mut styles = StyleProperties::default();
    for rule in matched {
        if let Some(bp) = &rule.breakpoint {
            if viewport.is_none_or(|w| bp.px < w) {
                continue;
            }
        }
        let target: &mut StyleProperties = match rule.pseudo {
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
            apply_css_declaration(target, prop, val);
        }
    }
    styles
}

/// The flat fields set on a style, without its state and responsive layers.
fn layer_fields(styles: &StyleProperties) -> Map<String, Value> {
    let mut map = match serde_json::to_value(styles) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    };
    for layer in ["tablet", "mobile", "hover", "focus", "active"] {
        map.remove(layer);
    }
    map
}

/// Names of the fields set on a style.
fn field_names(styles: &StyleProperties) -> Vec<String> {
    layer_fields(styles).keys().cloned().collect()
}

/// Builds the `tablet` / `mobile` layers by replaying the design's cascade at each breakpoint,
/// so specificity and source order decide exactly as they do in the browser.
///
/// A node has two layers, each with one breakpoint. A field whose value changes at any other
/// width (above 1024px, or a third breakpoint) cannot be expressed on the node: it is added to
/// `mask` and stays in the design CSS at every width, which keeps it exact.
fn responsive_layers(
    matched: &[&CssRule],
    base: &StyleProperties,
    mask: &mut StyleProperties,
) -> (Option<Box<StyleProperties>>, Option<Box<StyleProperties>>) {
    let mut widths: Vec<&Breakpoint> = Vec::new();
    for bp in matched.iter().filter_map(|r| r.breakpoint.as_ref()) {
        if !widths.iter().any(|w| w.px == bp.px) {
            widths.push(bp);
        }
    }
    if widths.is_empty() {
        return (None, None);
    }
    widths.sort_by(|a, b| b.px.total_cmp(&a.px));

    let masked = layer_fields(mask);
    let mut states: Vec<Map<String, Value>> = Vec::new();
    let mut changes: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut previous = layer_fields(base);
    for (i, bp) in widths.iter().enumerate() {
        let mut current = layer_fields(&cascade(matched, Some(bp.px)));
        // A `background` shorthand dropped an image set at a wider width.
        let has_image = |m: &Map<String, Value>| {
            m.contains_key("background_image") || m.contains_key("background_gradient")
        };
        if has_image(&previous) && !has_image(&current) {
            current.insert("background_image".to_string(), Value::from("none"));
        }
        for (field, value) in &current {
            if previous.get(field) != Some(value) && !masked.contains_key(field) {
                changes.entry(field.clone()).or_default().push(i);
            }
        }
        previous = current.clone();
        states.push(current);
    }

    let usable: Vec<usize> = (0..widths.len())
        .filter(|&i| widths[i].px <= TABLET_MAX_PX && changes.values().any(|p| p.contains(&i)))
        .collect();
    let (tablet_slot, mobile_slot) = assign_layer_slots(&widths, &usable);

    let mut tablet = Map::new();
    let mut mobile = Map::new();
    let mut left_to_css = Vec::new();
    for (field, points) in &changes {
        let fits = points
            .iter()
            .all(|p| Some(*p) == tablet_slot || Some(*p) == mobile_slot);
        if !fits {
            left_to_css.push(field.clone());
            continue;
        }
        for p in points {
            let target = if Some(*p) == tablet_slot {
                &mut tablet
            } else {
                &mut mobile
            };
            if let Some(value) = states[*p].get(field) {
                target.insert(field.clone(), value.clone());
            }
        }
    }
    if !left_to_css.is_empty() {
        add_masked_fields(mask, &left_to_css);
    }

    let build = |fields: Map<String, Value>, slot: Option<usize>, standard_px: f32| {
        let bp = widths[slot?];
        if fields.is_empty() {
            return None;
        }
        let mut layer: StyleProperties = serde_json::from_value(Value::Object(fields)).ok()?;
        if bp.px != standard_px {
            layer.breakpoint = Some(bp.width.clone());
        }
        Some(Box::new(layer))
    };
    (
        build(tablet, tablet_slot, TABLET_MAX_PX),
        build(mobile, mobile_slot, MOBILE_MAX_PX),
    )
}

const TABLET_MAX_PX: f32 = 1024.0;
const MOBILE_MAX_PX: f32 = 767.0;

/// Decides which of a node's breakpoints (indexes into `widths`, widest first) fill its two layers.
fn assign_layer_slots(widths: &[&Breakpoint], usable: &[usize]) -> (Option<usize>, Option<usize>) {
    match usable {
        [] => (None, None),
        [only] if widths[*only].px <= MOBILE_MAX_PX => (None, Some(*only)),
        [only] => (Some(*only), None),
        [wide, narrow] => (Some(*wide), Some(*narrow)),
        many => {
            let tablet = many
                .iter()
                .copied()
                .find(|&i| widths[i].px > MOBILE_MAX_PX)
                .unwrap_or(many[0]);
            let mobile = many
                .iter()
                .copied()
                .find(|&i| i != tablet && widths[i].px <= MOBILE_MAX_PX)
                .or_else(|| many.iter().copied().find(|&i| i != tablet));
            (Some(tablet), mobile)
        }
    }
}

/// Marks `fields` as left to the design CSS.
fn add_masked_fields(mask: &mut StyleProperties, fields: &[String]) {
    let mut map = layer_fields(mask);
    for field in fields {
        map.insert(field.clone(), Value::from("1"));
    }
    if let Ok(updated) = serde_json::from_value::<StyleProperties>(Value::Object(map)) {
        *mask = updated;
    }
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
    if let Some(expanded) = expand_logical_property(prop, val, false) {
        for (p, v) in expanded {
            mask_declaration(mask, &p, &v);
        }
        return;
    }
    match prop {
        "overflow-x" | "overflow-y" => mask.overflow = any(),
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
        "overflow" => styles.overflow = Some(val.to_string()),
        "width" => styles.width = Some(val.to_string()),
        "min-width" => styles.min_width = Some(val.to_string()),
        "max-width" => styles.max_width = Some(val.to_string()),
        "height" => styles.height = Some(val.to_string()),
        "min-height" => styles.min_height = Some(val.to_string()),
        "max-height" => styles.max_height = Some(val.to_string()),
        "aspect-ratio" => styles.aspect_ratio = Some(val.to_string()),
        "object-fit" => styles.object_fit = Some(val.to_string()),
        "background" => {
            // The shorthand replaces whatever came before; layered backgrounds have no field.
            let v = val.trim();
            let is_gradient = is_gradient_list(v);
            let is_none = v.eq_ignore_ascii_case("none");
            if is_gradient || is_none || is_plain_background_colour(v) {
                styles.background_color = None;
                styles.background_image = None;
                styles.background_gradient = None;
            }
            if is_gradient {
                styles.background_gradient = Some(v.to_string());
            } else if is_none {
                styles.background_color = Some("transparent".to_string());
            } else if is_plain_background_colour(v) {
                styles.background_color = Some(v.to_string());
            }
        }
        "background-color" => styles.background_color = Some(val.to_string()),
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
            // The shorthand replaces what individual sides set before it.
            styles.border_top = None;
            styles.border_right = None;
            styles.border_bottom = None;
            styles.border_left = None;
            if val == "none" || val == "0" {
                styles.border_style = Some("none".to_string());
            } else if let Some((width, style, color)) = split_border(val) {
                styles.border_width = Some(width);
                styles.border_style = Some(style);
                styles.border_color = Some(color);
            } else {
                styles.border_top = Some(val.to_string());
                styles.border_right = Some(val.to_string());
                styles.border_bottom = Some(val.to_string());
                styles.border_left = Some(val.to_string());
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

/// Splits a `border` shorthand with all three parts, in any order, into `(width, style, colour)`.
fn split_border(val: &str) -> Option<(String, String, String)> {
    let parts = split_shorthand_values(val);
    let [_, _, _] = parts.as_slice() else {
        return None;
    };
    let is_style = |p: &str| {
        matches!(
            p.to_ascii_lowercase().as_str(),
            "none"
                | "hidden"
                | "dotted"
                | "dashed"
                | "solid"
                | "double"
                | "groove"
                | "ridge"
                | "inset"
                | "outset"
        )
    };
    let is_width = |p: &str| {
        let lower = p.to_ascii_lowercase();
        lower.starts_with(|c: char| c.is_ascii_digit() || c == '.')
            || matches!(lower.as_str(), "thin" | "medium" | "thick")
    };
    let style = parts.iter().find(|p| is_style(p))?.to_string();
    let others: Vec<&&str> = parts.iter().filter(|p| !is_style(p)).collect();
    let [first, second] = others.as_slice() else {
        return None;
    };
    // A `var()` width has no tell of its own: it conventionally comes before the colour.
    if is_width(second) && !is_width(first) {
        Some((second.to_string(), style, first.to_string()))
    } else {
        Some((first.to_string(), style, second.to_string()))
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
    let without_no_class = selector.replace(NO_CLASS_PSEUDO, "");
    if without_no_class.contains('+')
        || without_no_class.contains('~')
        || without_no_class.contains('[')
    {
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
    if let Some(rest) = compound.strip_suffix(NO_CLASS_PSEUDO) {
        return el.class_str().is_none() && (rest.is_empty() || compound_matches(rest, el));
    }
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
    // Numbers (`--font-size-lg: 1.25rem`, `--font-weight: 600`) are not font families.
    if val.starts_with(|c: char| c.is_ascii_digit() || c == '.') || val.contains("clamp(") {
        return false;
    }
    let scale_word = [
        "size", "weight", "height", "spacing", "leading", "tracking", "style",
    ]
    .iter()
    .any(|w| key.contains(w));
    !scale_word
        && (matches!(
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
            || val.contains("monospace"))
}

/// True when a variable's name says it holds a colour (for values like `white`).
fn is_colour_token_name(key: &str) -> bool {
    [
        "color",
        "colour",
        "clr",
        "bg",
        "fg",
        "ink",
        "paper",
        "accent",
        "brand",
        "primary",
        "secondary",
        "text",
    ]
    .iter()
    .any(|w| key.contains(w))
}

fn parse_px_u32(val: &str) -> Option<u32> {
    let digits: String = val.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<u32>().ok()
}

/// Finds the first variable whose name matches one of `keys`, also accepting the common
/// `color-` / `colour-` / `clr-` / `font-` name prefixes (`--color-ink-2`, `--font-display`).
/// Returns the variable's name and value.
fn find_var_by_keys(vars: &[(String, String)], keys: &[&str]) -> Option<(String, String)> {
    let bare = |k: &str| -> String {
        ["color-", "colour-", "clr-", "font-"]
            .iter()
            .find_map(|p| k.strip_prefix(p))
            .unwrap_or(k)
            .to_string()
    };
    for target in keys {
        if let Some(found) = vars.iter().find(|(k, _)| k == target || bare(k) == *target) {
            return Some(found.clone());
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

#[cfg(test)]
pub(super) use test_hooks::*;

#[cfg(test)]
mod test_hooks {
    use super::*;

    pub(in crate::html_transpiler) fn native_declarations_for_test(
        decl: &Declaration,
        sheet: &ParsedStylesheet,
    ) -> Vec<(String, String)> {
        native_declarations(decl, sheet)
    }

    pub(in crate::html_transpiler) fn parse_declarations_for_test(body: &str) -> Vec<Declaration> {
        parse_declaration_list(body)
    }

    pub(in crate::html_transpiler) fn parse_statements_for_test(css: &str) -> Vec<CssStatement> {
        parse_css_block(css)
    }

    pub(in crate::html_transpiler) fn specificity_of(selector: &str) -> (u32, u32, u32) {
        compute_specificity(selector)
    }

    pub(in crate::html_transpiler) fn parse_max_width_query_for_test(
        prelude: &str,
    ) -> Option<Breakpoint> {
        parse_max_width_query(prelude)
    }
}
