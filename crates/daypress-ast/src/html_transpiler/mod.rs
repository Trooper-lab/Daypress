//! Universal HTML5 + CSS-to-AST Transpiler (`crates/daypress-ast/src/html_transpiler`).
//!
//! Converts any complete HTML/CSS document or fragment into 100% native `DaypressLayout`
//! AST blocks (`Section`, `Container`, `Grid`, `Flex`, `Column`, `Heading`, `Text`,
//! `Button`, `Image`, `Icon`, `Divider`, `Spacer`) and `DaystudioKit` Global Design Tokens
//! with **zero `CustomHtml` fallbacks**.

pub mod css_parser;
pub mod dom_parser;
pub mod forms;
pub mod mapper;

use crate::kit::DaystudioKit;
use crate::schema::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
use css_parser::{build_daystudio_kit, extract_and_parse_styles, resolve_element_styles};
use dom_parser::{parse_html_dom, DomElement, DomNode};
pub use forms::{ImportedForm, ImportedFormField};
use mapper::{map_dom_children, MapperContext};
use serde::{Deserialize, Serialize};

/// Options controlling how an HTML/CSS document is transpiled into a `DaypressLayout`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlDesignImportOptions {
    /// When `true`, the site chrome is split off the page: the body-level elements before
    /// `<main>` (or the leading `<header>`) and after it (or the closing `<footer>` plus anything
    /// that follows, such as a mobile call bar) become `header_layout` / `footer_layout`.
    pub extract_header_footer: bool,
    /// When `true`, each `<form>` becomes a `Form` node (with a negative placeholder `form_id`)
    /// and an [`ImportedForm`] in `forms`, for the caller to save as FormStudio forms.
    pub extract_forms: bool,
}

impl Default for HtmlDesignImportOptions {
    fn default() -> Self {
        Self {
            extract_header_footer: true,
            extract_forms: true,
        }
    }
}

/// Complete output of transpiling an HTML/CSS document into native DayStudio AST + tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlDesignImportResult {
    pub layout: DaypressLayout,
    pub kit: Option<DaystudioKit>,
    /// Raw HTML of the extracted site header / footer (kept for reference).
    pub header_html: Option<String>,
    pub footer_html: Option<String>,
    /// Native layouts of the extracted site header / footer, for header and footer theme templates.
    pub header_layout: Option<DaypressLayout>,
    pub footer_layout: Option<DaypressLayout>,
    /// Forms found in the page, header and footer. `Form` nodes reference them by
    /// `ImportedForm::placeholder_id` until the caller swaps in the saved form ids
    /// (see [`replace_form_placeholders`]).
    pub forms: Vec<ImportedForm>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub node_count: usize,
    pub token_count: usize,
}

/// Replaces placeholder `form_id`s (negative values from [`ImportedForm::placeholder_id`]) in
/// `node` and its descendants using `ids` (`placeholder -> saved form id`). Returns how many
/// `Form` nodes were rewritten.
pub fn replace_form_placeholders(
    node: &mut DaypressNode,
    ids: &std::collections::HashMap<i64, i64>,
) -> usize {
    let mut count = 0;
    if let NodeKind::Form { form_id, .. } = &mut node.kind {
        if let Some(real) = ids.get(form_id) {
            *form_id = *real;
            count += 1;
        }
    }
    for child in &mut node.children {
        count += replace_form_placeholders(child, ids);
    }
    count
}

/// Transpiles a full HTML/CSS document (or fragment) into a [`HtmlDesignImportResult`]
/// with 100% native DayStudio blocks and extracted `DaystudioKit` global styles.
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
    let main_el = find_direct_child_tag(body_el, "main");
    let parts = split_site_chrome(body_el, main_el, options.extract_header_footer);

    let html_of = |nodes: &[DomNode]| -> Option<String> {
        let mut out = String::new();
        for n in nodes {
            n.write_html(&mut out);
            out.push('\n');
        }
        let trimmed = out.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    };
    let header_html = html_of(&parts.header);
    let footer_html = html_of(&parts.footer);

    let mut ctx = MapperContext::new(&sheet);
    ctx.extract_forms = options.extract_forms;

    let mut chrome_ancestors = Vec::new();
    if body_el.tag == "body" {
        chrome_ancestors.push(body_el);
    }
    let mut ancestors = chrome_ancestors.clone();
    if let Some(main) = main_el {
        ancestors.push(main);
    }

    let header_children =
        as_top_level_bands(map_dom_children(&mut ctx, &parts.header, &chrome_ancestors));
    let mut root_children =
        as_top_level_bands(map_dom_children(&mut ctx, &parts.content, &ancestors));
    let footer_children =
        as_top_level_bands(map_dom_children(&mut ctx, &parts.footer, &chrome_ancestors));

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
        root_children.push(DaypressNode {
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
        let k = build_daystudio_kit(&sheet, title.clone(), description.clone());
        let count = k.system_colors.len()
            + k.custom_colors.len()
            + k.system_typography.len()
            + k.custom_typography.len();
        let val = serde_json::to_value(&k).ok();
        (Some(k), val, count)
    } else {
        (None, None, 0)
    };

    // The site chrome is rendered on every page, so it carries the design's stylesheet with it
    // (the header does; the footer only when there is no header).
    let chrome_layout = |children: Vec<DaypressNode>, with_kit: bool| -> Option<DaypressLayout> {
        (!children.is_empty()).then(|| DaypressLayout {
            version: 1,
            root: DaypressNode {
                id: "root".to_string(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children,
            },
            global_styles: if with_kit {
                global_styles.clone()
            } else {
                None
            },
        })
    };
    let header_layout = chrome_layout(header_children, true);
    let footer_layout = chrome_layout(footer_children, header_layout.is_none());

    let layout = DaypressLayout {
        version: 1,
        root: DaypressNode {
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
        header_layout,
        footer_layout,
        forms: ctx.forms,
        title,
        description,
        node_count: ctx.node_count,
        token_count,
    }
}

/// Class on every top-level block of an imported design. The public theme leaves `.dp-design`
/// blocks and their descendants out of its layout defaults (boxed container width, centring
/// margins, container gaps, the one-column grid fallback on small screens), so the design's own
/// stylesheet lays them out.
pub const DESIGN_MARKER_CLASS: &str = "dp-design";

/// Top-level boxes of a page or template (`<header>`, `<footer>`, `<nav>`, wrapper `<div>`s) are
/// full-width bands, so they become `Section` blocks: a `Container` carries the theme's
/// boxed-content width (`max-width: 1200px; margin: 0 auto`), which would shrink a design's
/// full-bleed header or footer. Every top-level block also gets [`DESIGN_MARKER_CLASS`].
fn as_top_level_bands(mut nodes: Vec<DaypressNode>) -> Vec<DaypressNode> {
    for node in &mut nodes {
        let classes = node.styles.css_classes.take().unwrap_or_default();
        let has_marker = classes.split_whitespace().any(|c| c == DESIGN_MARKER_CLASS);
        node.styles.css_classes = Some(if has_marker {
            classes
        } else {
            format!("{classes} {DESIGN_MARKER_CLASS}")
                .trim()
                .to_string()
        });
        let is_plain_box = matches!(node.kind, NodeKind::Container)
            && node.styles.link_url.as_deref().map_or(true, str::is_empty);
        let band_tag = matches!(
            node.styles.html_tag.as_deref(),
            None | Some("div" | "header" | "footer" | "nav" | "aside" | "article")
        );
        if is_plain_box && band_tag {
            node.kind = NodeKind::Section;
            if node.styles.html_tag.is_none() {
                node.styles.html_tag = Some("div".to_string());
            }
        }
    }
    nodes
}

/// The document body split into site header, page content and site footer.
struct SiteChromeParts {
    header: Vec<DomNode>,
    content: Vec<DomNode>,
    footer: Vec<DomNode>,
}

fn is_chrome_candidate(node: &DomNode) -> bool {
    matches!(node, DomNode::Element(e) if !matches!(e.tag.as_str(), "script" | "style" | "noscript" | "template" | "link" | "meta"))
}

fn has_direct_tag(nodes: &[DomNode], tags: &[&str]) -> bool {
    nodes
        .iter()
        .any(|n| matches!(n, DomNode::Element(e) if tags.contains(&e.tag.as_str())))
}

/// Splits `<body>` into header / content / footer.
///
/// With a `<main>`: everything before it is the header (when it holds a `<header>` or `<nav>`),
/// everything after it is the footer (when it holds a `<footer>`; a `<footer>` closing `<main>`
/// counts too). Without a `<main>`: the leading `<header>` (plus a skip link or similar before
/// it) and the last `<footer>` with whatever follows it. With `extract == false`, everything is
/// content.
fn split_site_chrome(
    body: &DomElement,
    main: Option<&DomElement>,
    extract: bool,
) -> SiteChromeParts {
    let mut parts = SiteChromeParts {
        header: Vec::new(),
        content: Vec::new(),
        footer: Vec::new(),
    };

    if let Some(main) = main {
        let mut before = Vec::new();
        let mut after = Vec::new();
        let mut seen_main = false;
        for child in &body.children {
            match child {
                DomNode::Element(e) if std::ptr::eq(e, main) => seen_main = true,
                n if is_chrome_candidate(n) => {
                    if seen_main {
                        after.push(n.clone());
                    } else {
                        before.push(n.clone());
                    }
                }
                _ => {}
            }
        }
        let mut content: Vec<DomNode> = main.children.clone();
        if extract {
            // A `<footer>` that closes `<main>` is still the site footer.
            if let Some(last_idx) = content.iter().rposition(is_chrome_candidate) {
                if matches!(&content[last_idx], DomNode::Element(e) if e.tag == "footer") {
                    let footer = content.remove(last_idx);
                    after.insert(0, footer);
                }
            }
            if has_direct_tag(&before, &["header", "nav"]) {
                parts.header = before;
            } else {
                content.splice(0..0, before);
            }
            if has_direct_tag(&after, &["footer"]) {
                parts.footer = after;
            } else {
                content.extend(after);
            }
        } else {
            content.splice(0..0, before);
            content.extend(after);
        }
        parts.content = content;
        return parts;
    }

    let children: Vec<DomNode> = body.children.clone();
    if !extract {
        parts.content = children;
        return parts;
    }
    let element_positions: Vec<usize> = children
        .iter()
        .enumerate()
        .filter(|(_, n)| is_chrome_candidate(n))
        .map(|(i, _)| i)
        .collect();
    let header_end = element_positions
        .iter()
        .take(3)
        .copied()
        .find(|&i| matches!(&children[i], DomNode::Element(e) if e.tag == "header"));
    let footer_start = element_positions
        .iter()
        .rev()
        .copied()
        .find(|&i| matches!(&children[i], DomNode::Element(e) if e.tag == "footer"))
        .filter(|&i| header_end.map_or(true, |h| i > h));

    for (i, node) in children.into_iter().enumerate() {
        let in_header = header_end.is_some_and(|h| i <= h);
        let in_footer = footer_start.is_some_and(|f| i >= f);
        if in_header {
            if is_chrome_candidate(&node) {
                parts.header.push(node);
            }
        } else if in_footer {
            if is_chrome_candidate(&node) {
                parts.footer.push(node);
            }
        } else {
            parts.content.push(node);
        }
    }
    parts
}

/// Convenience helper that transpiles an HTML fragment or document directly into a [`DaypressLayout`]
/// without stripping `<header>` / `<footer>` elements or turning forms into FormStudio forms.
pub fn transpile_html_to_layout(html: &str) -> DaypressLayout {
    transpile_html_document(
        html,
        &HtmlDesignImportOptions {
            extract_header_footer: false,
            extract_forms: false,
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

    fn count_custom_html_nodes(node: &DaypressNode) -> usize {
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
  <title>Daypress — The Rust Replacement</title>
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
  <header class="mast"><div class="mast-inner">Daypress Masthead</div></header>
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

        let kit = res.kit.as_ref().expect("should extract DaystudioKit");
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

    /// A trimmed-down agency one-pager (structure of the Grezzo VeroMetal design).
    const AGENCY_PAGE: &str = r###"<!doctype html>
<html lang="nl"><head>
<title>VeroMetal verwerker | Echt metaal | Grezzo</title>
<style>
:root { --color-paper: oklch(0% 0 0); --color-ink: oklch(100% 0 0); --color-ink-2: oklch(78.9% 0 0); --font-body: "Avenir Next", sans-serif; --font-display: "Times New Roman", serif; --container: 1440px; }
body { background: var(--color-paper); color: var(--color-ink-2); }
.container { width: 100%; max-width: var(--container); margin-inline: auto; padding-inline: 64px; }
.site-header { position: absolute; inset: 0 0 auto 0; }
.masthead { display: grid; grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr); gap: 24px; }
.main-nav { border-top: 1px solid #333; }
.main-nav ul { display: flex; justify-content: center; gap: 40px; list-style: none; }
.menu-toggle { display: none; background: transparent; }
.facts { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); }
.facts > div { padding-inline: 24px; border-left: 1px solid #333; }
.facts > div:first-child { padding-left: 0; border-left: 0; }
.split { display: grid; grid-template-columns: repeat(12, minmax(0, 1fr)); column-gap: 32px; }
.fgrid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px 24px; }
.field { display: flex; flex-direction: column; gap: 8px; }
.form-success { padding: 16px; border-left: 2px solid copper; }
.mobile-cta { position: fixed; left: 0; right: 0; bottom: 0; display: none; }
@media (max-width: 60rem) {
  .masthead { grid-template-columns: auto minmax(0, 1fr) auto; }
  .menu-toggle { display: inline-flex; }
  .main-nav { display: none; border-top: 0; }
  .nav-open .main-nav { display: block; position: fixed; inset: 0; }
  .nav-open .main-nav ul { flex-direction: column; align-items: flex-start; }
  .mobile-cta { display: grid; }
  .facts { grid-template-columns: repeat(2, minmax(0, 1fr)); row-gap: 40px; }
}
</style></head>
<body>
<a class="skip" href="#main">Naar de inhoud</a>
<header class="site-header" id="top">
  <div class="container">
    <div class="masthead">
      <p class="masthead__fact">Gecertificeerd verwerker</p>
      <a class="brand" href="/">Grezzo</a>
      <button class="menu-toggle" type="button" aria-controls="menu">Menu</button>
    </div>
    <nav class="main-nav" id="menu"><ul><li><a href="/">Home</a></li><li><a href="/contact/">Contact</a></li></ul></nav>
  </div>
</header>
<main id="main">
  <section class="stage" id="werkwijze">
    <div class="container split">
      <h2>Werkwijze</h2>
      <dl class="facts"><div><dt>2 mm</dt><dd>dik</dd></div><div><dt>100%</dt><dd>metaal</dd></div></dl>
    </div>
  </section>
  <section class="contact" id="aanvragen">
    <div class="container">
      <form action="https://formspree.io/f/JOUW_ID" method="POST" class="js-form">
        <input type="hidden" name="_subject" value="Stalenset aanvraag">
        <input type="hidden" name="aanvraag" value="Stalenset">
        <input type="text" name="_gotcha" style="display:none" tabindex="-1">
        <div class="fgrid">
          <div class="field"><label for="s-naam">Naam <span class="req">*</span></label><input type="text" id="s-naam" name="naam" required></div>
          <div class="field"><label for="s-email">E-mail <span class="req">*</span></label><input type="email" id="s-email" name="email" required></div>
          <div class="field full"><label for="s-type">Ik ben</label><select id="s-type" name="type" required><option value="">Maak een keuze</option><option>Architect</option><option>Anders</option></select></div>
          <fieldset class="field full"><legend class="legend">Toepassing</legend>
            <div class="chips">
              <label class="chip"><input type="checkbox" name="toepassing[]" value="Keuken"><span>Keuken</span></label>
              <label class="chip"><input type="checkbox" name="toepassing[]" value="Deur"><span>Deur</span></label>
            </div>
          </fieldset>
          <div class="field full"><label for="o-file">Foto</label><input type="file" id="o-file" name="bijlage" accept="image/*,.pdf"></div>
          <div class="field full"><label for="s-msg">Project</label><textarea id="s-msg" name="toelichting" placeholder="Bijvoorbeeld: 14 fronten"></textarea></div>
        </div>
        <div class="form-foot"><button type="submit" class="btn">Stalenset aanvragen</button></div>
        <div class="form-success" role="status" hidden>Dank je wel. We nemen contact op.</div>
      </form>
    </div>
  </section>
</main>
<footer class="site-footer"><div class="container"><p class="colophon">Grezzo maakt metalen afwerkingen.</p></div></footer>
<nav class="mobile-cta" aria-label="Snel contact"><a class="btn" href="#aanvragen">Stalenset</a></nav>
<script>document.body.classList.add('x');</script>
</body></html>"###;

    fn find_node<'a>(
        node: &'a DaypressNode,
        pred: &dyn Fn(&DaypressNode) -> bool,
    ) -> Option<&'a DaypressNode> {
        if pred(node) {
            return Some(node);
        }
        node.children.iter().find_map(|c| find_node(c, pred))
    }

    fn has_class(node: &DaypressNode, class: &str) -> bool {
        node.styles
            .css_classes
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .any(|c| c == class)
    }

    #[test]
    fn site_header_and_footer_become_native_template_layouts() {
        let res = transpile_html_document(AGENCY_PAGE, &HtmlDesignImportOptions::default());

        let header = res.header_layout.as_ref().expect("header layout");
        let footer = res.footer_layout.as_ref().expect("footer layout");
        assert_eq!(count_custom_html_nodes(&header.root), 0);
        assert_eq!(count_custom_html_nodes(&footer.root), 0);

        // The skip link and the <header> make up the header; the footer keeps the mobile call bar.
        let header_band = header
            .root
            .children
            .iter()
            .find(|n| has_class(n, "site-header"))
            .expect("site header band");
        assert!(matches!(header_band.kind, NodeKind::Section));
        assert_eq!(header_band.styles.html_tag.as_deref(), Some("header"));
        assert!(has_class(header_band, DESIGN_MARKER_CLASS));
        assert!(header.root.children.iter().any(|n| has_class(n, "skip")));
        assert!(find_node(&header.root, &|n| n.id == "menu").is_some());

        assert!(footer
            .root
            .children
            .iter()
            .any(|n| has_class(n, "site-footer")));
        assert!(footer
            .root
            .children
            .iter()
            .any(|n| has_class(n, "mobile-cta")));

        // The chrome carries the design's stylesheet (header only, so it is not loaded twice).
        assert!(header.global_styles.is_some());
        assert!(footer.global_styles.is_none());

        // ... and is no longer part of the page.
        let page_html = crate::render_node_to_html(&res.layout.root);
        assert!(!page_html.contains("site-header"));
        assert!(!page_html.contains("site-footer"));
        assert!(!page_html.contains("mobile-cta"));
        assert!(page_html.contains("Werkwijze"));

        let header_html = crate::render_layout_to_html(header);
        assert!(
            header_html.contains("<header id=\"top\" class=\"dp-section site-header dp-design\"")
        );
        assert!(header_html.contains("Gecertificeerd verwerker"));
        assert!(res
            .header_html
            .as_deref()
            .unwrap_or("")
            .contains("site-header"));
    }

    #[test]
    fn chrome_without_main_uses_leading_header_and_trailing_footer() {
        let html = r##"<body><header class="top"><a href="/">Brand</a></header><section class="hero"><h1>Hi</h1></section><footer class="bottom"><p>(c) Brand</p></footer><nav class="bar"><a href="#x">Call</a></nav></body>"##;
        let res = transpile_html_document(html, &HtmlDesignImportOptions::default());
        let header = res.header_layout.expect("header");
        let footer = res.footer_layout.expect("footer");
        assert!(header.root.children.iter().any(|n| has_class(n, "top")));
        assert!(footer.root.children.iter().any(|n| has_class(n, "bottom")));
        assert!(footer.root.children.iter().any(|n| has_class(n, "bar")));
        assert_eq!(res.layout.root.children.len(), 1);

        // Without extraction everything stays in the page.
        let kept = transpile_html_document(
            html,
            &HtmlDesignImportOptions {
                extract_header_footer: false,
                extract_forms: false,
            },
        );
        assert!(kept.header_layout.is_none() && kept.footer_layout.is_none());
        assert_eq!(kept.layout.root.children.len(), 4);
    }

    #[test]
    fn forms_become_formstudio_specs_and_form_nodes_with_the_design_markup() {
        let mut res = transpile_html_document(AGENCY_PAGE, &HtmlDesignImportOptions::default());
        assert_eq!(res.forms.len(), 1);
        let spec = &res.forms[0];
        assert_eq!(spec.placeholder_id, -1);
        assert_eq!(spec.label, "Stalenset aanvragen");
        assert_eq!(
            spec.success_message.as_deref(),
            Some("Dank je wel. We nemen contact op.")
        );
        assert_eq!(spec.email_subject.as_deref(), Some("Stalenset aanvraag"));
        let types: Vec<(&str, &str)> = spec
            .fields
            .iter()
            .map(|f| (f.name.as_str(), f.field_type.as_str()))
            .collect();
        assert_eq!(
            types,
            vec![
                ("aanvraag", "hidden"),
                ("naam", "text"),
                ("email", "email"),
                ("type", "select"),
                ("toepassing", "checkbox_group"),
                ("bijlage", "file"),
                ("toelichting", "textarea"),
            ]
        );

        let form = find_node(&res.layout.root, &|n| {
            matches!(n.kind, NodeKind::Form { .. })
        })
        .expect("Form node");
        assert!(matches!(form.kind, NodeKind::Form { form_id: -1, .. }));
        assert!(has_class(form, "js-form"));

        let html = crate::render_layout_to_html(&res.layout);
        assert!(html.contains("class=\"dp-form dp-form-markup js-form\" data-form-id=\"-1\""));
        // Controls keep their attributes; names lose the PHP-style [] suffix.
        assert!(html.contains(r#"<label for="s-naam">Naam <span class="req">*</span></label><input id="s-naam" name="naam" required="" type="text">"#));
        assert!(html.contains(r#"name="toepassing" type="checkbox" value="Keuken""#));
        assert!(html.contains(r#"<option value="">Maak een keuze</option>"#));
        assert!(html.contains(r#"accept="image/*,.pdf""#));
        assert!(html.contains(r#"<input name="aanvraag" type="hidden" value="Stalenset">"#));
        assert!(!html.contains("_gotcha") && !html.contains("_subject"));
        assert!(html.contains(r#"<button type="submit" class="dp-button btn""#));
        // The design's hidden success message is the status line the submit handler reveals.
        assert!(html.contains("dp-text form-success dp-form-status"));

        let ids = std::collections::HashMap::from([(-1, 42)]);
        assert_eq!(replace_form_placeholders(&mut res.layout.root, &ids), 1);
        let html = crate::render_layout_to_html(&res.layout);
        assert!(html.contains("action=\"/wp-json/daypress/v1/forms/42/submit\""));
    }

    #[test]
    fn transpile_html_to_layout_does_not_extract_forms() {
        let layout = transpile_html_to_layout(
            r#"<form><div class="f"><label for="n">Naam</label><input id="n" name="naam"></div></form>"#,
        );
        assert!(find_node(&layout.root, &|n| matches!(n.kind, NodeKind::Form { .. })).is_none());
        let html = crate::render_layout_to_html(&layout);
        assert!(html.contains(r#"<input id="n" name="naam">"#), "{html}");
    }

    #[test]
    fn imported_kit_neutralizes_theme_text_colour_and_section_padding() {
        let res = transpile_html_document(AGENCY_PAGE, &HtmlDesignImportOptions::default());
        let kit = res.kit.expect("kit");
        let custom = kit.custom_css.clone().unwrap_or_default();
        let prelude_at = custom
            .find("--dp-color-text: currentColor;")
            .expect("text colour reset");
        assert!(custom.contains("--dp-section-padding: 0;"));
        assert!(prelude_at < custom.find(".masthead").expect("design css"));

        // Compiled, the reset comes after the kit's own `:root` tokens, so it wins.
        let css = kit.compile_to_css();
        let token = css.find("--dp-color-text: ").expect("kit token");
        assert!(css.rfind("--dp-color-text: currentColor").unwrap() > token);

        // oklch tokens and `--color-*` / `--font-*` names are recognised.
        assert!(kit
            .system_colors
            .iter()
            .any(|c| c.id == "text" && c.color == "oklch(78.9% 0 0)"));
        assert!(kit
            .system_typography
            .iter()
            .any(|t| t.id == "primary" && t.font_family.contains("Times New Roman")));
    }

    #[test]
    fn breakpoint_and_state_values_stay_with_the_design_css() {
        let res = transpile_html_document(AGENCY_PAGE, &HtmlDesignImportOptions::default());
        let header = res.header_layout.as_ref().expect("header");

        // `.main-nav` is hidden by a breakpoint and opened by `.nav-open`: neither `display` nor
        // `border-top` is baked, and nothing becomes a per-node `!important` rule.
        let nav = find_node(&header.root, &|n| n.id == "menu").expect("menu");
        assert!(nav.styles.display.is_none());
        assert!(nav.styles.border_top.is_none());
        assert!(nav.styles.tablet.is_none() && nav.styles.mobile.is_none());
        let css = crate::compile_layout_css(header);
        assert!(!css.contains("[id=\"menu\"]"), "{css}");
        assert!(!css.contains("display: none !important"), "{css}");

        // The nav list's direction changes under `.nav-open`, so it is not a Flex block that
        // would pin `flex-direction: row` inline.
        let list = &nav.children[0];
        assert!(matches!(list.kind, NodeKind::Container));
        assert_eq!(list.styles.justify_content.as_deref(), Some("center"));

        // Values the design sets unconditionally are still baked.
        let toggle = find_node(&header.root, &|n| has_class(n, "menu-toggle")).expect("toggle");
        assert_eq!(
            toggle.styles.background_color.as_deref(),
            Some("transparent")
        );
        assert!(
            toggle.styles.display.is_none(),
            "display varies by breakpoint"
        );

        // `:first-child` overrides keep working: the shared padding/border is not baked.
        let fact = find_node(&res.layout.root, &|n| {
            n.children
                .iter()
                .any(|c| c.styles.html_tag.as_deref() == Some("dt"))
        })
        .expect("fact");
        assert!(fact.styles.padding_left.is_none() && fact.styles.border_left.is_none());

        // A grid whose gap changes at a breakpoint stays a Container (no inline `gap: 0px`).
        let facts = find_node(&res.layout.root, &|n| has_class(n, "facts")).expect("facts");
        assert!(matches!(facts.kind, NodeKind::Container));

        // `column-gap` alone keeps the row gap at `normal`.
        let split = find_node(&res.layout.root, &|n| has_class(n, "split")).expect("split");
        assert_eq!(split.styles.gap.as_deref(), Some("normal 32px"));
        assert!(matches!(split.kind, NodeKind::Grid { .. }));
    }
}
