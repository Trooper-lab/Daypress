//! DOM-to-AST Native Block Mapper (`html_transpiler/mapper.rs`).
//!
//! Maps a parsed `DomElement` tree + `ParsedStylesheet` into 100% native `DaypressNode`
//! blocks (`Section`, `Container`, `Grid`, `Flex`, `Column`, `Heading`, `Text`,
//! `Button`, `Image`, `Icon`, `Divider`, `Spacer`) with zero `CustomHtml` fallbacks.

use super::css_parser::{
    resolve_element_styles, resolve_element_styles_with_mask, ParsedStylesheet,
};
use super::dom_parser::{is_inline_tag, DomElement, DomNode};
use super::forms::{
    control_markup, extract_form, field_markup, is_form_control_tag, is_service_hidden_input,
    is_status_element, is_submit_control, ImportedForm,
};
use crate::schema::{DaypressNode, NodeKind, StyleProperties};
use std::collections::HashSet;

pub struct MapperContext<'a> {
    pub sheet: &'a ParsedStylesheet,
    pub used_ids: HashSet<String>,
    pub counter: usize,
    pub node_count: usize,
    /// When `true`, each `<form>` becomes a `Form` node plus an [`ImportedForm`] in `forms`.
    pub extract_forms: bool,
    pub forms: Vec<ImportedForm>,
    /// Depth of `<form>` elements currently being mapped.
    pub form_depth: usize,
}

impl<'a> MapperContext<'a> {
    pub fn new(sheet: &'a ParsedStylesheet) -> Self {
        let mut used_ids = HashSet::new();
        used_ids.insert("root".to_string());
        Self {
            sheet,
            used_ids,
            counter: 0,
            node_count: 0,
            extract_forms: false,
            forms: Vec::new(),
            form_depth: 0,
        }
    }

    pub fn next_id(&mut self, preferred: Option<&str>, prefix: &str) -> String {
        if let Some(raw) = preferred.map(str::trim).filter(|s| !s.is_empty()) {
            if !self.used_ids.contains(raw) {
                self.used_ids.insert(raw.to_string());
                return raw.to_string();
            }
        }
        loop {
            self.counter += 1;
            let candidate = format!("{}-{}", prefix, self.counter);
            if !self.used_ids.contains(&candidate) {
                self.used_ids.insert(candidate.clone());
                return candidate;
            }
        }
    }
}

/// Maps a list of DOM child nodes into native `DaypressNode` blocks.
pub fn map_dom_children(
    ctx: &mut MapperContext<'_>,
    children: &[DomNode],
    ancestors: &[&DomElement],
) -> Vec<DaypressNode> {
    let mut out = Vec::new();
    let mut inline_buf = String::new();

    let flush_inline_buf =
        |buf: &mut String, out: &mut Vec<DaypressNode>, ctx: &mut MapperContext<'_>| {
            let trimmed = buf.trim();
            if !trimmed.is_empty() {
                let id = ctx.next_id(None, "txt");
                ctx.node_count += 1;
                let styles = StyleProperties {
                    html_tag: Some("span".to_string()),
                    ..Default::default()
                };
                out.push(DaypressNode {
                    id,
                    kind: NodeKind::Text {
                        content_html: trimmed.to_string(),
                    },
                    styles,
                    children: Vec::new(),
                });
            }
            buf.clear();
        };

    for child in children {
        match child {
            DomNode::Text(t) => {
                if !t.trim().is_empty() {
                    inline_buf.push_str(t);
                } else if !inline_buf.is_empty() {
                    inline_buf.push(' ');
                }
            }
            DomNode::Element(el) => {
                if should_skip_element(&el.tag) {
                    continue;
                }
                if is_pure_inline_formatting_tag(&el.tag)
                    && !has_layout_display(ctx.sheet, el, ancestors)
                {
                    inline_buf.push_str(&el.outer_html());
                    continue;
                }
                flush_inline_buf(&mut inline_buf, &mut out, ctx);
                if let Some(mapped) = map_element(ctx, el, ancestors) {
                    out.push(mapped);
                }
            }
        }
    }
    flush_inline_buf(&mut inline_buf, &mut out, ctx);
    out
}

fn should_skip_element(tag: &str) -> bool {
    matches!(
        tag,
        "script" | "style" | "meta" | "link" | "title" | "head" | "noscript" | "base"
    )
}

fn is_pure_inline_formatting_tag(tag: &str) -> bool {
    matches!(
        tag,
        "em" | "strong" | "b" | "u" | "s" | "small" | "mark" | "sub" | "sup" | "kbd" | "br" | "wbr"
    )
}

fn has_layout_display(
    sheet: &ParsedStylesheet,
    el: &DomElement,
    ancestors: &[&DomElement],
) -> bool {
    let st = resolve_element_styles(sheet, el, ancestors);
    matches!(
        st.display.as_deref(),
        Some("grid" | "flex" | "inline-flex" | "block" | "inline-block")
    ) || st.grid_template_columns.is_some()
}

pub fn map_element(
    ctx: &mut MapperContext<'_>,
    el: &DomElement,
    ancestors: &[&DomElement],
) -> Option<DaypressNode> {
    if should_skip_element(&el.tag) || is_service_hidden_input(el) {
        return None;
    }

    let (mut styles, mask) = resolve_element_styles_with_mask(ctx.sheet, el, ancestors);
    let tag = el.tag.as_str();

    // 0a. `<form>` -> native `Form` node posting to FormStudio (fields keep the design's markup).
    if tag == "form" && ctx.extract_forms {
        return Some(map_form(ctx, el, ancestors, styles));
    }

    // 0b. Inside a form, the design's hidden success message becomes the form's status line
    // (shown by the `Form` node's submit handler), and other `hidden` parts stay hidden.
    if ctx.form_depth > 0 && el.attr("hidden").is_some() {
        styles.display = Some("none".to_string());
        if is_status_element(el) {
            let classes = styles.css_classes.take().unwrap_or_default();
            styles.css_classes = Some(format!("{} dp-form-status", classes).trim().to_string());
        }
    }

    // 0c. Submit buttons submit (`target: "submit"` renders `<button type="submit">`).
    if is_submit_control(el) && ctx.form_depth > 0 {
        let id = ctx.next_id(el.id(), "btn");
        ctx.node_count += 1;
        styles.html_tag = Some("button".to_string());
        let text = if tag == "input" {
            el.attr("value").unwrap_or("Submit").to_string()
        } else {
            el.inner_html().trim().to_string()
        };
        return Some(DaypressNode {
            id,
            kind: NodeKind::Button {
                text,
                url: String::new(),
                target: "submit".to_string(),
            },
            styles,
            children: Vec::new(),
        });
    }

    // 0d. Form controls keep every attribute (name, type, required, placeholder, options, `for`
    // on labels): the element that directly holds a control (a field wrapper, or a `<label>`
    // around a checkbox) becomes a Text block with the raw markup; a lone control gets one too.
    if is_form_control_tag(tag) {
        let id = ctx.next_id(None, "field");
        ctx.node_count += 1;
        let mut wrapper = StyleProperties {
            html_tag: Some("div".to_string()),
            ..Default::default()
        };
        if el.attr("type").map(str::to_ascii_lowercase).as_deref() == Some("hidden") {
            wrapper.display = Some("none".to_string());
        }
        return Some(DaypressNode {
            id,
            kind: NodeKind::Text {
                content_html: control_markup(el),
            },
            styles: wrapper,
            children: Vec::new(),
        });
    }
    if holds_form_control(el) {
        let id = ctx.next_id(el.id(), "field");
        ctx.node_count += 1;
        styles.html_tag = Some(tag.to_string());
        return Some(DaypressNode {
            id,
            kind: NodeKind::Text {
                content_html: field_markup(el),
            },
            styles,
            children: Vec::new(),
        });
    }

    // 1. Heading (`h1`..`h6`)
    if matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
        let level = tag
            .strip_prefix('h')
            .and_then(|n| n.parse::<u8>().ok())
            .unwrap_or(2);
        let id = ctx.next_id(el.id(), tag);
        ctx.node_count += 1;
        return Some(DaypressNode {
            id,
            kind: NodeKind::Heading {
                text: el.inner_html().trim().to_string(),
                level,
            },
            styles,
            children: Vec::new(),
        });
    }

    // 2. Image (`img`)
    if tag == "img" {
        let id = ctx.next_id(el.id(), "img");
        ctx.node_count += 1;
        let width = el.attr("width").and_then(|w| w.parse::<u32>().ok());
        let height = el.attr("height").and_then(|h| h.parse::<u32>().ok());
        return Some(DaypressNode {
            id,
            kind: NodeKind::Image {
                src: el.attr("src").unwrap_or("").to_string(),
                alt: el.attr("alt").unwrap_or("").to_string(),
                width,
                height,
                link: None,
            },
            styles,
            children: Vec::new(),
        });
    }

    // 3. Divider (`hr`)
    if tag == "hr" {
        let id = ctx.next_id(el.id(), "div");
        ctx.node_count += 1;
        return Some(DaypressNode {
            id,
            kind: NodeKind::Divider {
                style: "solid".to_string(),
            },
            styles,
            children: Vec::new(),
        });
    }

    // 4. Inline SVG graphic (`svg`) -> native Text block wrapping the vector graphic
    if tag == "svg" {
        let id = ctx.next_id(el.id(), "svg");
        ctx.node_count += 1;
        styles.html_tag = Some("div".to_string());
        return Some(DaypressNode {
            id,
            kind: NodeKind::Text {
                content_html: el.outer_html(),
            },
            styles,
            children: Vec::new(),
        });
    }

    // 5. Icon (`i` with icon classes and no text)
    if tag == "i" && el.text_content().trim().is_empty() {
        if let Some(cls) = el.class_str() {
            let id = ctx.next_id(el.id(), "icon");
            ctx.node_count += 1;
            return Some(DaypressNode {
                id,
                kind: NodeKind::Icon {
                    name: cls.to_string(),
                    size: None,
                    color: None,
                },
                styles,
                children: Vec::new(),
            });
        }
    }

    // 6. Anchor (`a`) or `<button>`
    if tag == "a" || tag == "button" {
        if tag == "a" && has_block_descendant(el) {
            // Card/container link wrapping block elements
            let id = ctx.next_id(el.id(), "link-box");
            ctx.node_count += 1;
            styles.link_url = el.attr("href").map(String::from);
            let mut next_anc = ancestors.to_vec();
            next_anc.push(el);
            let children = map_dom_children(ctx, &el.children, &next_anc);
            return Some(DaypressNode {
                id,
                kind: NodeKind::Container,
                styles,
                children,
            });
        }

        let id = ctx.next_id(el.id(), "btn");
        ctx.node_count += 1;
        if tag == "button" {
            styles.html_tag = Some("button".to_string());
        }
        return Some(DaypressNode {
            id,
            kind: NodeKind::Button {
                text: el.inner_html().trim().to_string(),
                url: el.attr("href").unwrap_or("").to_string(),
                target: el.attr("target").unwrap_or("_self").to_string(),
            },
            styles,
            children: Vec::new(),
        });
    }

    // 7. Empty Spacer `div` (no children, explicit height, no background/border)
    if tag == "div"
        && el.children.is_empty()
        && styles.background_color.is_none()
        && styles.background_gradient.is_none()
        && styles.border_width.is_none()
        && styles.border_top.is_none()
        && el.class_str().is_none()
    {
        if let Some(ref h_str) = styles.height {
            if let Some(px) = parse_leading_u32(h_str) {
                let id = ctx.next_id(el.id(), "spacer");
                ctx.node_count += 1;
                return Some(DaypressNode {
                    id,
                    kind: NodeKind::Spacer { height: px },
                    styles,
                    children: Vec::new(),
                });
            }
        }
    }

    // 8. Leaf Text / Rich-Text Elements (`p`, `figcaption`, `cite`, `code`, `pre`, `dt`, `dd`, `th`, `td`, `li`, `span`, etc.)
    if is_leaf_text_element(el, &styles) {
        let id = ctx.next_id(el.id(), "txt");
        ctx.node_count += 1;
        styles.html_tag = Some(tag.to_string());
        return Some(DaypressNode {
            id,
            kind: NodeKind::Text {
                content_html: el.inner_html().trim().to_string(),
            },
            styles,
            children: Vec::new(),
        });
    }

    // 9. Structural Container (`Section`, `Grid`, `Flex`, `Column`, `Container`)
    let mut next_anc = ancestors.to_vec();
    next_anc.push(el);
    let children = map_dom_children(ctx, &el.children, &next_anc);

    // `Grid` / `Flex` render their layout inline (display, gap, direction, alignment). Use them
    // only when the design sets those unconditionally; otherwise a breakpoint or state rule of
    // the design must be able to change them, so the element stays a plain `Container` and the
    // design CSS lays it out.
    let display = styles.display.as_deref();
    let is_grid = matches!(display, Some("grid" | "inline-grid"))
        && mask.display.is_none()
        && mask.gap.is_none();
    let is_flex = matches!(display, Some("flex" | "inline-flex"))
        && mask.display.is_none()
        && mask.flex_direction.is_none()
        && mask.justify_content.is_none()
        && mask.align_items.is_none()
        && mask.flex_wrap.is_none();

    let (kind, prefix, default_tag) = if tag == "section" {
        (NodeKind::Section, "sec", "section")
    } else if is_grid {
        let cols = styles
            .grid_template_columns
            .as_deref()
            .and_then(infer_grid_columns)
            .unwrap_or(0);
        let gap = styles
            .gap
            .as_deref()
            .or(styles.grid_gap.as_deref())
            .and_then(parse_leading_u32)
            .unwrap_or(0);
        (NodeKind::Grid { columns: cols, gap }, "grid", "div")
    } else if is_flex {
        let dir = styles
            .flex_direction
            .clone()
            .unwrap_or_else(|| "row".to_string());
        let justify = styles
            .justify_content
            .clone()
            .unwrap_or_else(|| "flex-start".to_string());
        let align = styles
            .align_items
            .clone()
            .unwrap_or_else(|| "stretch".to_string());
        let wrap = styles
            .flex_wrap
            .as_deref()
            .map(|w| w.contains("wrap") && w != "nowrap")
            .unwrap_or(false);
        (
            NodeKind::Flex {
                direction: dir,
                justify,
                align,
                wrap,
                basis: None,
            },
            "flex",
            "div",
        )
    } else {
        (NodeKind::Container, "box", "div")
    };

    if tag != default_tag {
        styles.html_tag = Some(tag.to_string());
    }

    let id = ctx.next_id(el.id(), prefix);
    ctx.node_count += 1;

    Some(DaypressNode {
        id,
        kind,
        styles,
        children,
    })
}

/// Maps a `<form>` to a `Form` node whose children are the design's own field markup, and records
/// the FormStudio specification (fields, submit label, success message) in `ctx.forms`.
fn map_form(
    ctx: &mut MapperContext<'_>,
    el: &DomElement,
    ancestors: &[&DomElement],
    mut styles: StyleProperties,
) -> DaypressNode {
    let placeholder_id = -(ctx.forms.len() as i64 + 1);
    let spec = extract_form(el, placeholder_id);
    let button_text = spec.submit_label.clone();
    ctx.forms.push(spec);

    let mut next_anc = ancestors.to_vec();
    next_anc.push(el);
    ctx.form_depth += 1;
    let children = map_dom_children(ctx, &el.children, &next_anc);
    ctx.form_depth -= 1;

    styles.html_tag = None;
    let id = ctx.next_id(el.id(), "form");
    ctx.node_count += 1;
    DaypressNode {
        id,
        kind: NodeKind::Form {
            form_id: placeholder_id,
            title: None,
            button_text,
            endpoint: None,
        },
        styles,
        children,
    }
}

/// An element that directly contains a form control (a field wrapper such as
/// `<div class="field"><label for=..>..</label><input ..></div>`), or a `<label>` wrapping one.
fn holds_form_control(el: &DomElement) -> bool {
    if el.tag == "form" {
        return false;
    }
    let direct = el.children.iter().any(|c| {
        matches!(c, DomNode::Element(e) if is_form_control_tag(&e.tag) && !is_service_hidden_input(e))
    });
    direct || (el.tag == "label" && has_form_control_descendant(el))
}

fn has_form_control_descendant(el: &DomElement) -> bool {
    el.children.iter().any(|c| match c {
        DomNode::Element(e) => is_form_control_tag(&e.tag) || has_form_control_descendant(e),
        DomNode::Text(_) => false,
    })
}

/// Determines whether `el` should be represented as a leaf `NodeKind::Text` block
/// (preserving inline HTML markup like `<em>`, `<strong>`, `<code>`, `<span>`, `<a>`, `<br>`)
/// rather than a structural container.
fn is_leaf_text_element(el: &DomElement, styles: &StyleProperties) -> bool {
    let tag = el.tag.as_str();

    // Never treat an empty decorative element (`<span class="dot"></span>`) as Text;
    // mapping it to `Container` with `html_tag` renders a clean empty `<span class="..."></span>`.
    if el.children.is_empty() || el.inner_html().trim().is_empty() {
        return false;
    }

    // Never treat a `section`, `main`, `header`, `footer`, `nav`, `aside`, `article`, `figure`,
    // `dl`, `ul`, `ol`, `table`, `thead`, `tbody`, `tr` as a leaf Text node.
    if matches!(
        tag,
        "section"
            | "main"
            | "header"
            | "footer"
            | "nav"
            | "aside"
            | "article"
            | "figure"
            | "dl"
            | "ul"
            | "ol"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "form"
            | "details"
    ) {
        return false;
    }

    // Paragraphs, code/pre blocks, citations, definition terms/descriptions, table cells
    // without nested block containers are always leaf Text nodes.
    if matches!(
        tag,
        "p" | "cite"
            | "code"
            | "pre"
            | "dt"
            | "dd"
            | "th"
            | "td"
            | "summary"
            | "address"
            | "time"
            | "label"
    ) && !has_block_descendant(el)
    {
        return true;
    }

    // If the element is explicitly a multi-child CSS Grid or Flex container and has no direct
    // prose text (only child elements, like `<div class="masthead-bar"><span>...</span><span>...</span></div>`),
    // map it as a Grid/Flex container so each child element is an independent node.
    let is_layout = matches!(
        styles.display.as_deref(),
        Some("grid" | "flex" | "inline-flex")
    ) || styles.grid_template_columns.is_some();

    let has_direct_non_ws_text = el
        .children
        .iter()
        .any(|c| matches!(c, DomNode::Text(t) if !t.trim().is_empty()));

    let element_children_count = el
        .children
        .iter()
        .filter(|c| matches!(c, DomNode::Element(_)))
        .count();

    if is_layout && !has_direct_non_ws_text && element_children_count > 1 {
        return false;
    }

    // If the element has no block descendants and either:
    // - has direct non-whitespace text (e.g. `<div>Hello <span>world</span></div>`, `<li>Item</li>`, `<figcaption>Fig 1</figcaption>`)
    // - or is a single inline wrapper (`<span><strong>Label</strong></span>`, `<p><code>foo</code></p>`)
    if !has_block_descendant(el)
        && (has_direct_non_ws_text || matches!(tag, "span" | "figcaption" | "li" | "blockquote"))
    {
        // Check if it's a container of multiple distinct child spans/links without direct text
        if !has_direct_non_ws_text && element_children_count > 1 {
            return false;
        }
        return true;
    }

    false
}

fn has_block_descendant(el: &DomElement) -> bool {
    for child in &el.children {
        if let DomNode::Element(c) = child {
            if !is_inline_tag(&c.tag) && c.tag != "a" && c.tag != "svg" {
                return true;
            }
            if has_block_descendant(c) {
                return true;
            }
        }
    }
    false
}

fn infer_grid_columns(tmpl: &str) -> Option<u8> {
    let trimmed = tmpl.trim();
    if let Some(rest) = trimmed.strip_prefix("repeat(") {
        let count_str: String = rest
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(n) = count_str.parse::<u8>() {
            // Only use `repeat(N, minmax(0, 1fr))` shorthand if the template is uniform `1fr` tracks
            if rest.contains("1fr") && !rest.contains("px") && !rest.contains("auto") {
                return Some(n);
            }
        }
        return Some(0);
    }
    Some(0)
}

fn parse_leading_u32(val: &str) -> Option<u32> {
    let digits: String = val
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<u32>().ok()
}
