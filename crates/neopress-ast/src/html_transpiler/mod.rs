//! Universal HTML5 + CSS-to-AST Transpiler (`crates/neopress-ast/src/html_transpiler`).
//!
//! Converts any complete HTML/CSS document or fragment into 100% native `NeopressLayout`
//! AST blocks (`Section`, `Container`, `Grid`, `Flex`, `Column`, `Heading`, `Text`,
//! `Button`, `Image`, `Icon`, `Divider`, `Spacer`) and `ElementorKit` Global Design Tokens
//! with **zero `CustomHtml` fallbacks**.

pub mod css_parser;
pub mod dom_parser;
pub mod mapper;

use crate::kit::ElementorKit;
use crate::schema::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};
use css_parser::{build_elementor_kit, extract_and_parse_styles, resolve_element_styles};
use dom_parser::{parse_html_dom, DomElement, DomNode};
use mapper::{map_dom_children, MapperContext};
use serde::{Deserialize, Serialize};

/// Options controlling how an HTML/CSS document is transpiled into a `NeopressLayout`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlDesignImportOptions {
    /// When `true`, top-level `<header>` and `<footer>` elements are extracted into
    /// `header_html` / `footer_html` so they can be persisted to site-wide chrome options.
    pub extract_header_footer: bool,
}

impl Default for HtmlDesignImportOptions {
    fn default() -> Self {
        Self {
            extract_header_footer: true,
        }
    }
}

/// Complete output of transpiling an HTML/CSS document into native NeoStudio AST + tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlDesignImportResult {
    pub layout: NeopressLayout,
    pub kit: Option<ElementorKit>,
    pub header_html: Option<String>,
    pub footer_html: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub node_count: usize,
    pub token_count: usize,
}

/// Transpiles a full HTML/CSS document (or fragment) into a [`HtmlDesignImportResult`]
/// with 100% native NeoStudio blocks and extracted `ElementorKit` global styles.
pub fn transpile_html_document(
    html: &str,
    options: &HtmlDesignImportOptions,
) -> HtmlDesignImportResult {
    let doc = parse_html_dom(html);
    let sheet = extract_and_parse_styles(&doc);

    let title = find_tag(&doc, "title")
        .map(|el| el.text_content().trim().to_string())
        .filter(|s| !s.is_empty());

    let description = find_meta_description(&doc);

    let body_el = find_tag(&doc, "body").unwrap_or(&doc);

    let mut header_html: Option<String> = None;
    let mut footer_html: Option<String> = None;

    if options.extract_header_footer {
        for child in &body_el.children {
            if let DomNode::Element(el) = child {
                if el.tag == "header" && header_html.is_none() {
                    header_html = Some(el.outer_html());
                } else if el.tag == "footer" && footer_html.is_none() {
                    footer_html = Some(el.outer_html());
                }
            }
        }
    }

    let main_el = find_direct_child_tag(body_el, "main");
    let content_root = main_el.unwrap_or(body_el);

    if options.extract_header_footer && footer_html.is_none() {
        if let Some(last_el) = content_root.children.iter().rev().find_map(|n| match n {
            DomNode::Element(e) if !matches!(e.tag.as_str(), "script" | "style") => Some(e),
            _ => None,
        }) {
            if last_el.tag == "footer" {
                footer_html = Some(last_el.outer_html());
            }
        }
    }

    let filtered_children: Vec<DomNode> = content_root
        .children
        .iter()
        .filter(|node| match node {
            DomNode::Element(el) if options.extract_header_footer => {
                if el.tag == "header" && header_html.as_deref() == Some(&el.outer_html()) {
                    return false;
                }
                if el.tag == "footer" && footer_html.as_deref() == Some(&el.outer_html()) {
                    return false;
                }
                true
            }
            _ => true,
        })
        .cloned()
        .collect();

    let mut ctx = MapperContext::new(&sheet);
    let mut ancestors = Vec::new();
    if main_el.is_some() {
        ancestors.push(body_el);
        ancestors.push(content_root);
    } else if body_el.tag == "body" {
        ancestors.push(body_el);
    }

    let mut root_children = map_dom_children(&mut ctx, &filtered_children, &ancestors);

    let mut root_styles = StyleProperties::default();
    if body_el.tag == "body" {
        let body_styles = resolve_element_styles(&sheet, body_el, &[]);
        root_styles.background_color = body_styles.background_color;
        root_styles.text_color = body_styles.text_color;
        root_styles.font_family = body_styles.font_family;
        root_styles.sync_flat_and_nested();
    }

    if root_children.is_empty() && !html.trim().is_empty() {
        let id = ctx.next_id(None, "sec");
        ctx.node_count += 1;
        root_children.push(NeopressNode {
            id,
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: Vec::new(),
        });
    }

    let has_tokens = !sheet.root_vars.is_empty()
        || !sheet.raw_css.trim().is_empty()
        || !sheet.google_font_urls.is_empty();

    let (kit, global_styles, token_count) = if has_tokens {
        let k = build_elementor_kit(&sheet, title.clone(), description.clone());
        let count = k.system_colors.len()
            + k.custom_colors.len()
            + k.system_typography.len()
            + k.custom_typography.len();
        let val = serde_json::to_value(&k).ok();
        (Some(k), val, count)
    } else {
        (None, None, 0)
    };

    let layout = NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "root".to_string(),
            kind: NodeKind::Root,
            styles: root_styles,
            children: root_children,
        },
        global_styles,
    };

    HtmlDesignImportResult {
        layout,
        kit,
        header_html,
        footer_html,
        title,
        description,
        node_count: ctx.node_count,
        token_count,
    }
}

/// Convenience helper that transpiles an HTML fragment or document directly into a [`NeopressLayout`]
/// without stripping `<header>` / `<footer>` elements.
pub fn transpile_html_to_layout(html: &str) -> NeopressLayout {
    transpile_html_document(
        html,
        &HtmlDesignImportOptions {
            extract_header_footer: false,
        },
    )
    .layout
}

fn find_tag<'a>(el: &'a DomElement, target_tag: &str) -> Option<&'a DomElement> {
    if el.tag == target_tag {
        return Some(el);
    }
    for child in &el.children {
        if let DomNode::Element(c) = child {
            if let Some(found) = find_tag(c, target_tag) {
                return Some(found);
            }
        }
    }
    None
}

fn find_direct_child_tag<'a>(el: &'a DomElement, target_tag: &str) -> Option<&'a DomElement> {
    for child in &el.children {
        if let DomNode::Element(c) = child {
            if c.tag == target_tag {
                return Some(c);
            }
        }
    }
    None
}

fn find_meta_description(el: &DomElement) -> Option<String> {
    if el.tag == "meta" {
        if let Some(name) = el.attr("name") {
            if name.eq_ignore_ascii_case("description") {
                return el
                    .attr("content")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
            }
        }
    }
    for child in &el.children {
        if let DomNode::Element(c) = child {
            if let Some(desc) = find_meta_description(c) {
                return Some(desc);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_custom_html_nodes(node: &NeopressNode) -> usize {
        let self_count = usize::from(matches!(node.kind, NodeKind::CustomHtml { .. }));
        self_count
            + node
                .children
                .iter()
                .map(count_custom_html_nodes)
                .sum::<usize>()
    }

    #[test]
    fn transpiles_editorial_html_design_into_100_percent_native_nodes() {
        let html = r###"<!doctype html>
<html lang="en">
<head>
  <title>Neopress — The Rust Replacement</title>
  <meta name="description" content="Drop-in WordPress replacement in Rust.">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Newsreader:ital,opsz,wght@0,6..72,400..700&family=Schibsted+Grotesk:wght@400..700&family=IBM+Plex+Mono:wght@400;500;600&display=swap">
  <style>
    :root {
      --paper: #F5F1E8;
      --ink: #13120F;
      --oxide: #C8371A;
      --forest: #1F5E3B;
      --serif: 'Newsreader', Georgia, serif;
      --sans: 'Schibsted Grotesk', sans-serif;
      --mono: 'IBM Plex Mono', monospace;
      --wide: 1380px;
    }
    :root[data-theme="dark"] {
      --paper: #12110E;
      --ink: #EFECE2;
    }
    .hero { display: grid; grid-template-columns: repeat(12, 1fr); gap: 32px; padding: 48px 24px; }
    .hero-left { grid-column: 1 / 8; }
    .btn { display: inline-flex; background: var(--ink); color: var(--paper); padding: 14px 22px; }
    .btn:hover { background: var(--oxide); }
    @media (max-width: 860px) {
      .hero-left { grid-column: 1 / -1; }
    }
  </style>
</head>
<body>
  <header class="mast"><div class="mast-inner">Neopress Masthead</div></header>
  <main id="top">
    <section class="hero">
      <div class="hero-left">
        <h1>WordPress, rebuilt in <em>Rust</em>.</h1>
        <p class="lede">Every vulnerability in <code>wp-login.php</code> ends here.</p>
        <div class="actions" style="display:flex;gap:14px;">
          <a href="#join" class="btn">Claim your migration <span>→</span></a>
        </div>
        <figure class="diagram">
          <svg viewBox="0 0 100 20"><rect width="100" height="20" fill="#C8371A"/></svg>
          <figcaption>FIG 01 — Latency distribution</figcaption>
        </figure>
        <dl class="spec" style="display:grid;grid-template-columns:120px 1fr;">
          <dt>Runtime</dt>
          <dd>Tokio + Axum</dd>
        </dl>
      </div>
    </section>
    <footer><div class="foot-inner">Colophon</div></footer>
  </main>
</body>
</html>"###;

        let res = transpile_html_document(html, &HtmlDesignImportOptions::default());
        assert_eq!(count_custom_html_nodes(&res.layout.root), 0);
        assert!(res.node_count >= 10);
        assert!(res.header_html.as_deref().unwrap_or("").contains("mast"));
        assert!(res
            .footer_html
            .as_deref()
            .unwrap_or("")
            .contains("Colophon"));

        let kit = res.kit.as_ref().expect("should extract ElementorKit");
        assert_eq!(kit.content_width, 1380);
        assert!(kit.system_colors.iter().any(|c| c.color == "#C8371A"));
        assert!(kit
            .custom_colors
            .iter()
            .any(|c| c.id == "paper" && c.color == "#F5F1E8"));

        let ssr = crate::render_layout_to_html(&res.layout);
        assert!(ssr.contains("WordPress, rebuilt in <em>Rust</em>."));
        assert!(ssr.contains("<figure"));
        assert!(ssr.contains("<figcaption"));
        assert!(ssr.contains("<dl"));
        assert!(ssr.contains("<dt"));
        assert!(ssr.contains("<dd"));
    }
}
