//! Checks `tests/fixtures/html_import/responsive_one_page.html` end to end: for every element
//! tagged `data-check`, the cascade of the design's stylesheet must give the same values at
//! 1440, 1024, 800 and 390px as the cascade of the imported page (inline styles, responsive
//! layers and the kit's remaining `custom_css`).

use super::css_parser::*;
use super::dom_parser::{parse_html_dom, DomElement, DomNode};
use super::*;
use crate::render_layout_to_html;
use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../../tests/fixtures/html_import/responsive_one_page.html");
const WIDTHS: [f32; 4] = [1440.0, 1024.0, 800.0, 390.0];

#[derive(Clone, Copy, PartialEq)]
enum Condition {
    Always,
    MaxWidth(f32),
    MinWidth(f32),
    Never,
}

impl Condition {
    fn active_at(self, width: f32) -> bool {
        match self {
            Condition::Always => true,
            Condition::MaxWidth(max) => width <= max,
            Condition::MinWidth(min) => width >= min,
            Condition::Never => false,
        }
    }
}

struct Rule {
    selector: String,
    condition: Condition,
    declarations: Vec<Declaration>,
}

fn collect_rules(items: &[CssStatement], condition: Condition, out: &mut Vec<Rule>) {
    for item in items {
        match item {
            CssStatement::Style {
                selectors,
                declarations,
            } => {
                for selector in selectors {
                    out.push(Rule {
                        selector: selector.clone(),
                        condition,
                        declarations: declarations.clone(),
                    });
                }
            }
            CssStatement::Group { prelude, items } => {
                collect_rules(items, media_condition(prelude), out);
            }
            CssStatement::Verbatim(_) => {}
        }
    }
}

fn media_condition(prelude: &str) -> Condition {
    let lower = prelude.to_ascii_lowercase();
    let px_of = |marker: &str| -> Option<f32> {
        let rest = lower.split(marker).nth(1)?.trim_start_matches([':', ' ']);
        let number: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let value: f32 = number.parse().ok()?;
        let unit = rest.get(number.len()..).unwrap_or("");
        Some(if unit.starts_with("rem") || unit.starts_with("em") {
            value * 16.0
        } else {
            value
        })
    };
    if lower.starts_with("@media") {
        if let Some(px) = px_of("max-width") {
            return Condition::MaxWidth(px);
        }
        if let Some(px) = px_of("min-width") {
            return Condition::MinWidth(px);
        }
    }
    Condition::Never
}

fn rule_applies(selector: &str, el: &DomElement, ancestors: &[&DomElement]) -> bool {
    if let Some(id) = selector
        .strip_prefix("[id=\"")
        .and_then(|s| s.strip_suffix("\"]"))
    {
        return el.id() == Some(id);
    }
    selector_matches(selector, el, ancestors)
}

/// The values the cascade gives `el` at `width`, on the same field names as a node style.
fn effective_styles(
    rules: &[Rule],
    sheet: &ParsedStylesheet,
    el: &DomElement,
    ancestors: &[&DomElement],
    width: f32,
) -> BTreeMap<String, String> {
    let mut matched: Vec<(&Rule, (u32, u32, u32), usize)> = rules
        .iter()
        .enumerate()
        .filter(|(_, r)| r.condition.active_at(width) && rule_applies(&r.selector, el, ancestors))
        .map(|(i, r)| (r, specificity_of(&r.selector), i))
        .collect();
    matched.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));

    let mut styles = StyleProperties::default();
    let mut apply = |decl: &Declaration| {
        let plain = Declaration {
            important: false,
            ..decl.clone()
        };
        for (prop, value) in native_declarations_for_test(&plain, sheet) {
            apply_css_declaration(&mut styles, &prop, &value);
        }
    };
    for (rule, _, _) in &matched {
        rule.declarations
            .iter()
            .filter(|d| !d.important)
            .for_each(&mut apply);
    }
    if let Some(inline) = el.attr("style") {
        parse_declarations_for_test(inline)
            .iter()
            .filter(|d| !d.important)
            .for_each(&mut apply);
    }
    for (rule, _, _) in &matched {
        rule.declarations
            .iter()
            .filter(|d| d.important)
            .for_each(&mut apply);
    }

    let mut out = BTreeMap::new();
    let flat = match serde_json::to_value(&styles) {
        Ok(serde_json::Value::Object(map)) => map,
        _ => panic!("style map"),
    };
    for (field, value) in flat {
        let text = value.as_str().unwrap_or_default().to_string();
        // Defaults that `Flex` / `Grid` nodes spell out inline.
        let is_default = matches!(
            (field.as_str(), text.as_str()),
            ("flex_direction", "row")
                | ("flex_wrap", "nowrap")
                | ("align_items", "stretch")
                | ("justify_content", "flex-start")
                | ("gap", "0px")
                | ("grid_gap", "0px")
        );
        if !is_default {
            out.insert(field, text);
        }
    }
    out
}

/// Every `data-check` element in a parsed document, with its ancestors.
fn checked_elements<'a>(
    nodes: &'a [DomNode],
    ancestors: &[&'a DomElement],
    out: &mut BTreeMap<String, (&'a DomElement, Vec<&'a DomElement>)>,
) {
    for node in nodes {
        if let DomNode::Element(el) = node {
            if let Some(name) = el.attr("data-check") {
                out.insert(name.to_string(), (el, ancestors.to_vec()));
            }
            let mut below = ancestors.to_vec();
            below.push(el);
            checked_elements(&el.children, &below, out);
        }
    }
}

fn import() -> HtmlDesignImportResult {
    transpile_html_document(FIXTURE, &HtmlDesignImportOptions::default())
}

fn node_by_check<'a>(layout: &'a DaypressLayout, name: &str) -> &'a DaypressNode {
    fn find<'a>(node: &'a DaypressNode, name: &str) -> Option<&'a DaypressNode> {
        let matches = node
            .styles
            .data_attributes
            .as_ref()
            .and_then(|a| a.get("data-check"))
            .is_some_and(|v| v == name);
        if matches {
            return Some(node);
        }
        node.children.iter().find_map(|c| find(c, name))
    }
    find(&layout.root, name).unwrap_or_else(|| panic!("no node tagged {name}"))
}

#[test]
fn import_matches_the_source_cascade_at_every_test_width() {
    let res = import();
    let source_doc = parse_html_dom(FIXTURE);
    let source_sheet = extract_and_parse_styles(&source_doc);
    let mut source_rules = Vec::new();
    collect_rules(
        &source_sheet.statements,
        Condition::Always,
        &mut source_rules,
    );

    let kit = res.kit.as_ref().expect("kit");
    let header = res.header_layout.as_ref().expect("header");
    let footer = res.footer_layout.as_ref().expect("footer");
    let imported_html = format!(
        "{}{}{}",
        render_layout_to_html(header),
        render_layout_to_html(&res.layout),
        render_layout_to_html(footer)
    );
    let imported_css = format!(
        "{}\n{}\n{}\n{}",
        kit.custom_css.as_deref().unwrap_or(""),
        crate::compile_layout_css(header),
        crate::compile_layout_css(&res.layout),
        crate::compile_layout_css(footer)
    );
    let imported_doc = parse_html_dom(&imported_html);
    let mut imported_rules = Vec::new();
    collect_rules(
        &parse_statements_for_test(&imported_css),
        Condition::Always,
        &mut imported_rules,
    );

    let mut source_elements = BTreeMap::new();
    checked_elements(&source_doc.children, &[], &mut source_elements);
    let mut imported_elements = BTreeMap::new();
    checked_elements(&imported_doc.children, &[], &mut imported_elements);
    assert_eq!(
        source_elements.keys().collect::<Vec<_>>(),
        imported_elements.keys().collect::<Vec<_>>(),
        "every checked element survives the import"
    );
    assert!(source_elements.len() >= 20);

    let mut differences = Vec::new();
    for width in WIDTHS {
        for (name, (source_el, source_anc)) in &source_elements {
            let (imported_el, imported_anc) = &imported_elements[name];
            let expected =
                effective_styles(&source_rules, &source_sheet, source_el, source_anc, width);
            let actual = effective_styles(
                &imported_rules,
                &source_sheet,
                imported_el,
                imported_anc,
                width,
            );
            if expected != actual {
                differences.push(format!(
                    "{width}px {name}\n  source: {expected:?}\n  import: {actual:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn breakpoints_fill_the_native_tablet_and_mobile_layers() {
    let res = import();

    // The design's in-between breakpoint: a tablet layer starting at 900px, not at 1024px.
    let hero = node_by_check(&res.layout, "hero");
    let tablet = hero.styles.tablet.as_ref().expect("tablet");
    assert_eq!(tablet.grid_template_columns.as_deref(), Some("1fr"));
    assert_eq!(tablet.breakpoint.as_deref(), Some("900px"));

    // A design rule exactly at 1024px needs no breakpoint of its own.
    let band = node_by_check(&res.layout, "band");
    let tablet = band.styles.tablet.as_ref().expect("tablet");
    assert_eq!(tablet.padding_top.as_deref(), Some("48px"));
    assert!(tablet.breakpoint.is_none());

    let title = node_by_check(&res.layout, "hero-title");
    assert_eq!(title.styles.font_size.as_deref(), Some("64px"));
    let tablet = title.styles.tablet.as_ref().expect("tablet");
    assert_eq!(tablet.font_size.as_deref(), Some("48px"));
    let mobile = title.styles.mobile.as_ref().expect("mobile");
    assert_eq!(mobile.font_size.as_deref(), Some("36px"));
    assert_eq!(mobile.breakpoint.as_deref(), Some("640px"));

    // Widest query first, so the phone layer wins over the tablet layer.
    let css = crate::compile_layout_css(&res.layout);
    let at = |needle: &str| {
        css.find(needle)
            .unwrap_or_else(|| panic!("{needle} missing from {css}"))
    };
    assert!(at("(max-width: 1024px)") < at("(max-width: 900px)"));
    assert!(at("(max-width: 900px)") < at("(max-width: 640px)"));

    // A lower-specificity breakpoint rule loses to the base rule, as in the browser.
    let card_title = node_by_check(&res.layout, "card-title");
    assert_eq!(card_title.styles.font_size.as_deref(), Some("22px"));
    assert!(card_title.styles.mobile.is_none());

    // `style=""` beats the breakpoint rule at every width, and non-native inline survives.
    let note = node_by_check(&res.layout, "note");
    assert_eq!(note.styles.max_width.as_deref(), Some("300px"));
    assert!(note.styles.tablet.is_none());
    assert_eq!(
        note.styles.custom_css.as_deref(),
        Some("list-style: square;")
    );

    // A width above the tablet range has no layer: the value stays with the design CSS.
    assert!(hero.styles.gap.is_none());
    assert!(matches!(hero.kind, NodeKind::Container | NodeKind::Section));
}

#[test]
fn text_links_stay_links_and_buttons_stay_buttons() {
    let res = import();
    let page = render_layout_to_html(&res.layout);

    assert!(page.contains(r#"class="dp-link link""#), "{page}");
    assert!(
        page.contains(r#"class="dp-button btn dp-design""#),
        "{page}"
    );
    let header = render_layout_to_html(res.header_layout.as_ref().expect("header"));
    assert!(
        !header.contains(r#"class="dp-button brand"#),
        "a logo link is not a button: {header}"
    );

    let cta = node_by_check(&res.layout, "hero-cta");
    assert_eq!(
        cta.styles
            .hover
            .as_ref()
            .and_then(|h| h.background_color.as_deref()),
        Some("var(--moss)")
    );
}

/// Every piece of markup a Text or Heading node stores verbatim, with the node kind beside it.
fn stored_markup(node: &DaypressNode, out: &mut Vec<String>) {
    match &node.kind {
        NodeKind::Text { content_html } => out.push(content_html.clone()),
        NodeKind::Heading { text, .. } => out.push(text.clone()),
        _ => {}
    }
    for child in &node.children {
        stored_markup(child, out);
    }
}

#[test]
fn every_button_is_a_native_button_node_never_raw_markup() {
    let res = import();
    let layouts = [
        res.header_layout.as_ref().expect("header"),
        &res.layout,
        res.footer_layout.as_ref().expect("footer"),
    ];
    for layout in layouts {
        let mut markup = Vec::new();
        stored_markup(&layout.root, &mut markup);
        for html in markup {
            assert!(!html.contains("<button"), "raw button in: {html}");
            assert!(!html.contains("class=\"btn"), "raw button link in: {html}");
            assert!(!html.contains("type=\"submit\""), "raw submit in: {html}");
            assert!(!html.contains("type=\"reset\""), "raw reset in: {html}");
        }
    }

    for name in [
        "hero-cta",
        "more-btn",
        "more-btn-2",
        "heading-btn",
        "cell-btn",
        "signup-btn",
        "signup-reset",
    ] {
        let node = node_by_check(&res.layout, name);
        assert!(
            matches!(node.kind, NodeKind::Button { .. }),
            "{name} is not a Button node"
        );
    }

    // The text around a button stays editable, in the element it came from, with its spaces.
    let more = node_by_check(&res.layout, "more");
    assert_eq!(more.styles.html_tag.as_deref(), Some("p"));
    let runs: Vec<&str> = more
        .children
        .iter()
        .filter_map(|c| match &c.kind {
            NodeKind::Text { content_html } => Some(content_html.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(runs, ["Thirsty? ", " ", " today"]);

    // A heading that holds a button keeps its tag.
    let page = render_layout_to_html(&res.layout);
    assert!(
        page.contains("<h3 id=\"box-") && page.contains("Pre-order "),
        "{page}"
    );

    // A form row holding a button keeps its controls and labels, without a layout box of their own.
    assert!(
        page.contains(r#"<label data-check="signup-label" for="signup-email">E-mail</label>"#),
        "{page}"
    );
    assert!(page.contains(r#"type="email""#), "{page}");
    assert!(page.contains("display: contents;"), "{page}");
    assert!(page.contains(r#"type="reset""#), "{page}");

    // With every button on a node, the shared button rule leaves `custom_css`.
    let custom = res.kit.expect("kit").custom_css.unwrap_or_default();
    assert!(!custom.contains(".btn"), "{custom}");
}

#[test]
fn inherited_colour_and_descendant_rules_land_on_the_right_nodes() {
    let res = import();

    // No rule targets this paragraph: it inherits the band's colour instead of hard-coding one.
    let copy = node_by_check(&res.layout, "band-copy");
    assert!(copy.styles.text_color.is_none());
    assert_eq!(
        node_by_check(&res.layout, "band")
            .styles
            .text_color
            .as_deref(),
        Some("var(--chalk)")
    );

    // Descendant rules reach their nodes, base and responsive.
    let fine = node_by_check(&res.layout, "band-fine");
    assert_eq!(fine.styles.text_color.as_deref(), Some("var(--sand)"));
    assert_eq!(fine.styles.font_size.as_deref(), Some("13px"));
    let item = node_by_check(&res.layout, "menu-item");
    assert_eq!(item.styles.font_size.as_deref(), Some("15px"));
    assert_eq!(item.styles.text_color.as_deref(), Some("var(--moss)"));
    assert_eq!(
        item.styles
            .mobile
            .as_ref()
            .and_then(|m| m.font_size.as_deref()),
        Some("14px")
    );

    // Selectors that depend on the absence of a class survive native `dp-*` classes.
    let card_copy = node_by_check(&res.layout, "card-copy");
    assert_eq!(card_copy.styles.max_width.as_deref(), Some("60ch"));
}

#[test]
fn data_attributes_reach_the_rendered_elements() {
    let res = import();
    let page = render_layout_to_html(&res.layout);
    assert!(page.contains(r#"data-label="Loaf""#), "{page}");
    assert!(page.contains(r#"data-label="Price""#), "{page}");

    let kit = res.kit.expect("kit");
    assert!(kit
        .custom_css
        .unwrap_or_default()
        .contains("attr(data-label)"));
}

#[test]
fn root_tokens_fill_the_native_global_tokens() {
    let kit = import().kit.expect("kit");
    let colour = |id: &str| -> String {
        kit.system_colors
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.color.clone())
            .unwrap_or_default()
    };
    assert_eq!(colour("text"), "#5c4a3a"); // body
    assert_eq!(colour("secondary"), "#2b2118"); // headings
    assert_eq!(colour("primary"), "#b5532a"); // the design's button background
    assert_eq!(colour("accent"), "#4d6b3c"); // the remaining brand colour

    let font = |id: &str| {
        kit.system_typography
            .iter()
            .find(|t| t.id == id)
            .expect("system typography")
    };
    assert!(font("primary").font_family.contains("Playfair Display"));
    assert_eq!(font("primary").font_size.as_deref(), Some("64px"));
    assert!(font("text").font_family.contains("Inter"));
    assert_eq!(font("text").font_size.as_deref(), Some("17px"));
    assert_eq!(font("text").line_height.as_deref(), Some("1.6"));

    // Nothing is left over from the Daypress defaults, and each variable lives in one token.
    let everything = serde_json::to_string(&kit).unwrap_or_default();
    for default in ["Newsreader", "Schibsted", "IBM Plex", "#13120F", "#C8371A"] {
        assert!(!everything.contains(default), "{default} left in kit");
    }
    assert!(kit.custom_colors.iter().any(|c| c.id == "sand"));
    assert!(!kit.custom_colors.iter().any(|c| c.id == "clay"));

    // Editing a Global Token restyles the page: the design's variables point at the tokens.
    let custom = kit.custom_css.clone().unwrap_or_default();
    assert!(
        custom.contains("--clay: var(--e-global-color-primary);"),
        "{custom}"
    );
    assert!(
        custom.contains("--sand: var(--e-global-color-sand);"),
        "{custom}"
    );
    assert!(
        custom.contains("--display: var(--e-global-typography-primary-font-family);"),
        "{custom}"
    );
    assert!(custom.contains("--gutter: 48px;"), "{custom}");
}

#[test]
fn custom_css_only_holds_what_has_no_native_equivalent() {
    let res = import();
    let custom = res.kit.expect("kit").custom_css.unwrap_or_default();

    for kept in [
        "box-sizing: border-box",                               // no node field
        "list-style: none",                                     // no node field
        "border-collapse: collapse",                            // no node field
        "content: attr(data-label)",                            // pseudo-element
        ".nav-open .nav",                                       // state class a script adds
        "@media (min-width: 1600px)",                           // no native bucket
        "@media (max-width: 1100px)",                           // above the tablet range
        ".hero { gap: 64px; }",                                 // changes at that width
        "--gutter: 20px",                                       // token override at a breakpoint
        "background: var(--sand)",                              // `body` is not a node
        "@media (max-width: 900px) {\n.nav { display: none; }", // `.nav-open` can change it
    ] {
        assert!(custom.contains(kept), "missing {kept:?} in:\n{custom}");
    }

    for gone in [
        ".legacy-card",    // cannot match
        ".btn",            // on nodes (with hover)
        ".card {",         // on nodes
        ".card h3",        // on nodes
        ".site-header",    // on nodes
        ".site-footer",    // on nodes
        ".band",           // on nodes
        "h1 {",            // on nodes
        "font-size: 64px", // on nodes
        "grid-template",   // on nodes and layers
        ".hero h1",        // on nodes
        "object-fit",      // on nodes
        "text-decoration", // on nodes
    ] {
        assert!(
            !custom.contains(gone),
            "{gone:?} should be gone from:\n{custom}"
        );
    }
}

#[test]
fn breakpoint_queries_are_recognised_exactly() {
    let width = |q: &str| parse_max_width_query_for_test(q).map(|b| (b.width, b.px));
    assert_eq!(
        width("@media (max-width: 900px)"),
        Some(("900px".into(), 900.0))
    );
    assert_eq!(
        width("@media only screen and (max-width: 60rem)"),
        Some(("60rem".into(), 960.0))
    );
    assert_eq!(
        width("@media screen and (width <= 40em)"),
        Some(("40em".into(), 640.0))
    );
    assert_eq!(width("@media (min-width: 900px)"), None);
    assert_eq!(
        width("@media (max-width: 900px) and (min-width: 600px)"),
        None
    );
    assert_eq!(width("@media print, (max-width: 900px)"), None);
    assert_eq!(width("@media (max-height: 900px)"), None);
}
