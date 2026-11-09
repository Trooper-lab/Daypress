---
name: daystudio-website-builder
description: >-
  Design and build pages, front pages, headers, footers, dynamic templates, and
  reusable components in Daypress using DayStudio. Use this skill whenever
  creating or updating page layouts via daypress_import_html_design,
  daypress_update_post_layout, daypress_save_template, or daypress_update_global_styles,
  and when auditing pages with daypress_audit_post.
---

# DayStudio Website Builder & AST Layout Skill

Use this skill whenever a user asks you to **design, build, restyle, or edit web pages, landing pages, headers, footers, single post/page templates, or synced components** in Daypress.

> **Important Safety Rule for Live Sites:**
> Before overwriting an existing page's layout, ALWAYS call `daypress_get_post_layout` (and `daypress_list_templates` for headers/footers) first so you preserve any manual edits the user made in the DayStudio Visual Builder (`/d-admin/builder`).

---

## 1. Choosing the Right Building Workflow

Daypress gives AI agents two complementary ways to build pages that remain **100% editable in the visual drag-and-drop builder (`/d-admin/builder`)**:

### Workflow A: Full HTML5/CSS Import (`daypress_import_html_design`) — *Fastest for New Pages*
Pass a complete, semantic HTML5 document (with a `<style>` block, `<header>`, `<main>` sections, and `<footer>`) to `daypress_import_html_design`.
- Daypress's Rust HTML-to-AST transpiler converts `<section>`, `<div>`, CSS Grid/Flexbox layouts, `<h1>`–`<h6>`, `<p>`, `<a>`, `<img>`, `<form>`, `<details>`, and pricing/feature cards into **100% native `DaypressNode` blocks** (`custom_html_nodes: 0`).
- When `extract_header_footer: true` (default), the site chrome becomes native theme templates, active on the entire site: everything in `<body>` before `<main>` (skip link + `<header>`) is saved as the `"{site_name} header"` template, everything after it (`<footer>` plus trailing bars such as a mobile call bar) as `"{site_name} footer"`. `site_name` defaults to the last segment of the design's `<title>` ("Studio | Acme" → "Acme"); pass `site_name` to override. The header template carries the design's stylesheet, so the chrome is styled on every page. Re-importing updates the same-titled templates (their display conditions are kept) instead of adding new ones. The response lists `header_template` / `footer_template` ids.
- Every `<form>` becomes a FormStudio form plus a `Form` node (see the forms skill); the response lists them under `forms`.
- The design's stylesheet becomes native styling: unconditional rules are written to the blocks, `:hover` / `:focus` / `:active` to their state layers, and `@media (max-width: ...)` rules to each block's `tablet` / `mobile` layer at the design's own breakpoint. `:root` colours and fonts fill the kit's Global Tokens, and the design's variables point at those tokens, so editing a token restyles the page.
- Only what has no native equivalent stays in the kit's `custom_css`: state rules (`:target`, `.nav-open`, `:checked`), `min-width` and other conditions, breakpoints above 1024px, a third breakpoint on one block, pseudo-elements, and properties without a block field. A block property such a rule can change stays off the block, so the design's cascade decides it. Rules that can never match are dropped.
- The theme's own element defaults (text colour, section padding, button look, boxed container width, browser-default resets) are switched off for imported blocks (`dp-design` class on top-level blocks and on imported buttons). A link without button styling stays a plain link (a `Button` node with `html_tag: "a"`), and `data-*` attributes are kept.
- Scripts are not imported: tab panels, carousels' arrow buttons and script-driven menus need native equivalents (or CSS such as a `:target` menu) after the import.
- When `set_as_front_page: true`, Daypress automatically sets `show_on_front = "page"` and `page_on_front = <post_id>`.

```json
{
  "name": "daypress_import_html_design",
  "arguments": {
    "title": "Home",
    "slug": "home",
    "set_as_front_page": true,
    "extract_header_footer": true,
    "html": "<!DOCTYPE html><html><head><style>:root { --accent: #a8401b; }</style></head><body><header>...</header><main><section style=\"padding: 96px 24px; background: #f3eee4;\">...</section></main><footer>...</footer></body></html>"
  }
}
```

### Workflow B: Direct `DaypressLayout` AST Authoring (`daypress_update_post_layout`) — *Best for Precision Edits & Pro Widgets*
`daypress_get_post_layout` returns `{"post_id": 4, "layout_json": "<string>"}`: `layout_json` is the layout serialized as a JSON **string**. Parse it before editing; send it back as a string to `daypress_update_post_layout`, or as the parsed object (with its `root`) in `layout` to `daypress_save_template`.

Use `daypress_get_post_layout` and `daypress_update_post_layout` when:
- Making targeted updates to an existing page without regenerating it from scratch.
- Using interactive Pro widgets (`LoopGrid`, `TestimonialCarousel`, `Accordion`, `Tabs`, `Countdown`, `FlipBox`, `AnimatedHeadline`, `Form`, `LanguageSwitcher`, `SymbolInstance`).
- Authoring responsive overrides (`styles.tablet`, `styles.mobile`) and hover states (`styles.hover`).

---

## 2. Canonical `DaypressLayout` AST JSON Schema

Every page or template layout is a `DaypressLayout` JSON object:

```json
{
  "version": 1,
  "root": {
    "id": "sec_root",
    "type": "Root",
    "styles": {},
    "children": [
      {
        "id": "hero_section",
        "type": "Section",
        "styles": {
          "background_color": "#f3eee4",
          "padding_top": "96px",
          "padding_bottom": "96px",
          "padding_left": "24px",
          "padding_right": "24px",
          "mobile": {
            "padding_top": "56px",
            "padding_bottom": "56px"
          }
        },
        "children": [
          {
            "id": "hero_container",
            "type": "Container",
            "styles": {
              "max_width": "1160px",
              "margin_left": "auto",
              "margin_right": "auto",
              "display": "flex",
              "flex_direction": "column",
              "gap": "24px"
            },
            "children": [
              {
                "id": "hero_h1",
                "type": "Heading",
                "data": {
                  "text": "Crafted in Native Rust. Published in Microseconds.",
                  "level": 1
                },
                "styles": {
                  "font_family": "'Newsreader', Georgia, serif",
                  "font_size": "56px",
                  "font_weight": "500",
                  "line_height": "1.08",
                  "text_color": "#16171a",
                  "mobile": {
                    "font_size": "36px"
                  }
                }
              },
              {
                "id": "hero_lead",
                "type": "Text",
                "data": {
                  "content_html": "<p>Zero PHP. Zero MySQL. One self-contained binary with a visual builder and built-in MCP server.</p>"
                },
                "styles": {
                  "font_size": "19px",
                  "line_height": "1.6",
                  "text_color": "#4b4f56",
                  "max_width": "640px"
                }
              },
              {
                "id": "hero_cta",
                "type": "Button",
                "data": {
                  "text": "Explore Our Work →",
                  "url": "/portfolio/",
                  "target": "_self"
                },
                "styles": {
                  "background_color": "#a8401b",
                  "text_color": "#ffffff",
                  "padding_top": "14px",
                  "padding_bottom": "14px",
                  "padding_left": "28px",
                  "padding_right": "28px",
                  "border_radius": "6px",
                  "font_weight": "600",
                  "hover": {
                    "background_color": "#16171a"
                  }
                }
              }
            ]
          }
        ]
      }
    ]
  }
}
```

> **Note:** When calling `daypress_update_post_layout`, pass `layout_json` as a serialized JSON string (`JSON.stringify(layout)`).

---

## 3. Complete `NodeKind` Widget Reference

Every `DaypressNode` has `"id"`, `"type"`, optional `"data"`, `"styles"`, and optional `"children"`:

### Structural & Layout Containers (Accept `children`)
| `type` | `data` Fields | Notes |
| :--- | :--- | :--- |
| `"Root"` | *(none)* | Top-level tree root. |
| `"Section"` | *(none)* | Full-bleed `<section>` wrapper (`html_tag` override supported). |
| `"Container"` | *(none)* | Centered max-width container (`div`). |
| `"Column"` | *(none)* | Flex/grid column wrapper. |
| `"Grid"` | `{"columns": 3, "gap": 24}` | Responsive CSS Grid container. |
| `"Flex"` | `{"direction": "row", "justify": "space-between", "align": "center", "wrap": true, "basis": null}` | Flexbox layout container. |

### Basic Content & Media Widgets
| `type` | `data` Fields |
| :--- | :--- |
| `"Heading"` | `{"text": "Section Title", "level": 2}` (`level`: `1`–`6`) |
| `"Text"` | `{"content_html": "<p>Rich HTML paragraph...</p>"}` |
| `"Image"` | `{"src": "/uploads/photo.jpg", "alt": "Descriptive alt text", "width": 1200, "height": 800, "link": null}` |
| `"Button"` | `{"text": "Get Started", "url": "/contact/", "target": "_self"}` |
| `"Spacer"` | `{"height": 48}` |
| `"Divider"` | `{"style": "solid"}` (`"solid"`, `"dashed"`, `"dotted"`) |
| `"Icon"` | `{"name": "fa-solid fa-bolt", "size": 24, "color": "#a8401b"}` |
| `"IconBox"` | `{"icon": "fa-solid fa-shield", "title": "Feature", "description": "Details...", "link": null}` |
| `"ImageBox"` | `{"image_url": "/uploads/card.jpg", "title": "Card Title", "description": "Summary...", "link": "/read/"}` |
| `"IconList"` | `{"items": [{"text": "Sub-ms SSR", "icon": "✓", "link": null}]}` |
| `"Video"` | `{"url": "https://www.youtube.com/watch?v=...", "aspect_ratio": "16:9", "autoplay": false, "muted": false, "loop_video": false}` |
| `"Audio"` | `{"url": "/uploads/episode1.mp3"}` |
| `"Map"` | `{"address": "Amsterdam, Netherlands", "zoom": 13}` |

### Commercial, Marketing & Interactive Widgets
| `type` | `data` Fields |
| :--- | :--- |
| `"PriceCard"` | `{"title": "Pro", "subtitle": "For growing teams", "currency": "$", "price": "49", "original_price": "69", "period": "/month", "badge": "Most Popular", "featured": true, "features": [{"text": "Unlimited pages", "included": true}], "button_text": "Start Trial", "button_url": "/signup"}` |
| `"CallToAction"` | `{"title": "Ready to launch?", "description": "Migrate in one click.", "button_text": "Get Started", "button_url": "/setup", "secondary_button_text": "Read Docs", "secondary_button_url": "/docs"}` |
| `"Testimonial"` | `{"content": "Quote text...", "name": "Jane Doe", "job": "CTO, Acme", "image_url": null}` |
| `"TestimonialCarousel"` | `{"slides": [{"content": "Quote...", "name": "Jane Doe", "title": "CTO", "image_url": null, "rating": 5}], "autoplay": true, "loop_slides": true}` |
| `"Accordion"` | `{"items": [{"title": "Question 1?", "content": "<p>Answer 1.</p>"}]}` (Also automatically emits `FAQPage` Schema.org JSON-LD!) |
| `"Tabs"` | `{"items": [{"title": "Tab 1", "content": "<p>Tab 1 body.</p>"}]}` |
| `"Counter"` | `{"number": 99, "prefix": "", "suffix": "%", "title": "Uptime"}` |
| `"Benchmark"` | `{"label": "SSR Latency", "value": "0.04ms", "comparison": "vs 420ms PHP"}` |
| `"ProgressBar"` | `{"title": "Migration Fidelity", "percent": 96, "display_text": "96%"}` |
| `"Countdown"` | `{"target_date": "2027-01-01T00:00:00Z", "mode": "due_date", "show_days": true, "show_hours": true, "show_minutes": true, "show_seconds": true}` |
| `"AnimatedHeadline"` | `{"before_text": "Built for ", "words": ["Speed", "Creators", "AI Agents"], "after_text": ".", "animation_style": "rotate"}` |

### Dynamic, Navigation & Form Widgets
| `type` | `data` Fields |
| :--- | :--- |
| `"NavMenu"` | `{"menu_slug": "primary", "layout": "horizontal", "align": "right", "mobile_hamburger": true, "items": [{"label": "Work", "url": "/portfolio/", "children": []}]}` |
| `"LoopGrid"` | `{"post_type": "post", "posts_per_page": 6, "columns": 3, "show_thumbnail": true, "show_excerpt": true, "show_meta": true, "show_pagination": true, "taxonomy": "category", "enable_filter_bar": true, "enable_sort": true, "enable_search": true, "default_sort": "newest", "card_skin": "editorial", "meta_fields": []}` (`card_skin`: `"editorial"`, `"boxed"`, `"minimal"`, `"horizontal"`) |
| `"Form"` | `{"form_id": 1, "title": "Contact Us", "button_text": "Send Message"}` (Renders FormStudio form `#1`) |
| `"LanguageSwitcher"` | `{"display_style": "pills", "show_flags": true, "show_names": true, "show_codes": true, "current_lang": "en", "links": []}` |
| `"TableOfContents"` | `{"title": "On this page", "min_depth": 2, "max_depth": 3, "collapsible": true}` |
| `"Breadcrumbs"` | `{"separator": "/", "home_text": "Home"}` |
| `"SearchForm"` | `{"placeholder": "Search articles...", "button_text": "Search", "endpoint": "/"}` |
| `"SymbolInstance"` | `{"symbol_id": 5, "overrides": {}}` (Renders a synced global component created via `daypress_save_template`) |

---

## 4. `StyleProperties` Cheat Sheet

Every node's `"styles"` object supports flat CSS-aligned properties plus responsive/pseudo-state nested objects:
- **Colors & Backgrounds**: `background_color`, `background_image`, `background_gradient`, `background_size`, `background_position`, `text_color`
- **Typography**: `font_family`, `font_size`, `font_weight`, `line_height`, `letter_spacing`, `text_align`, `text_transform`, `font_style`
- **Spacing & Sizing**: `padding_top`, `padding_right`, `padding_bottom`, `padding_left`, `margin_top`, `margin_right`, `margin_bottom`, `margin_left`, `width`, `max_width`, `min_height`, `aspect_ratio`, `object_fit`
- **Flexbox & Grid**: `display` (`"flex"`, `"grid"`, `"block"`), `flex_direction`, `justify_content`, `align_items`, `flex_wrap`, `gap`, `grid_template_columns`, `grid_gap`
- **Borders & Effects**: `border_radius`, `border_width`, `border_style`, `border_color`, `border_top`, `border_bottom`, `box_shadow`, `opacity`, `backdrop_filter`, `transition`
- **Semantic HTML & Custom CSS**: `html_tag` (`"header"`, `"nav"`, `"section"`, `"article"`, `"aside"`, `"footer"`), `css_classes`, `custom_css`
  - `custom_css` as plain declarations (`"backdrop-filter: blur(8px); cursor: pointer;"`) is added to the node's inline style.
  - `custom_css` with rule blocks is scoped to the node: `selector`, `&` and `:scope` mean the node itself, any other selector targets its descendants, and `@media` blocks are scoped too. Example: `"selector:hover { opacity: .9 } .title { color: var(--ink) } @media (max-width: 600px) { .title { font-size: 14px } }"`.
- **Responsive & State Overrides**:
  - `"tablet": { ... }` (`<= 1024px`)
  - `"mobile": { ... }` (`<= 767px`)
  - Inside `tablet` / `mobile`, `"breakpoint": "900px"` (px, em or rem) starts that layer at another width. Imported designs use it to keep their own breakpoints.
  - `"hover": { ... }`, `"focus": { ... }`, `"active": { ... }`
  - `"hide_desktop": false`, `"hide_tablet": false`, `"hide_mobile": false`

---

## 5. Dynamic Theme Templates, Headers, Footers & Synced Symbols

Use `daypress_list_templates` and `daypress_save_template` to manage site-wide chrome and dynamic layouts. `layout` must be a full `DaypressLayout` with a `root` node (`{"version": 1, "root": {"id": "root", "type": "Root", "children": [...]}}`); a bare node or the whole `daypress_get_post_layout` response is rejected with an error. Headers and footers from `daypress_import_html_design` are already saved this way; edit them with `daypress_list_templates` → change the `layout` → `daypress_save_template` with the same `id`.
- **`template_type` values**:
  - `"header"`: Global or conditional site header
  - `"footer"`: Global or conditional site footer
  - `"single_post"`: Layout wrapper for blog posts (supports dynamic tokens: `{{post.title}}`, `{{post.date}}`, `{{post.content}}`, `{{post.excerpt}}`, `{{post.featured_image}}`, `{{author.name}}`, `{{acf.field_name}}`)
  - `"single_page"`: Layout wrapper for pages or specific Custom Post Types
  - `"archive"`: Layout wrapper for blog/category/CPT archives
  - `"error_404"`: Custom 404 page template
  - `"symbol"`: Reusable Figma-style synced component (embed anywhere via `SymbolInstance`)
  - `"popup"`: Site-wide announcement bar or modal (`settings: {"popup_variant": "banner", "trigger": "load", "dismiss_days": 7}`)

### Example: Creating a Conditional Single Template for a Custom Post Type
```json
{
  "name": "daypress_save_template",
  "arguments": {
    "title": "Case Study Single Layout",
    "template_type": "single_post",
    "status": "publish",
    "is_active": true,
    "priority": 20,
    "conditions": [
      { "condition_type": "include", "scope": "post_type", "target_value": "case_study" }
    ],
    "layout": {
      "version": 1,
      "root": {
        "id": "cs_root",
        "type": "Section",
        "styles": { "padding_top": "72px", "padding_bottom": "72px" },
        "children": [
          {
            "id": "cs_title",
            "type": "Heading",
            "data": { "text": "{{post.title}}", "level": 1 },
            "styles": { "font_size": "48px" }
          },
          {
            "id": "cs_client",
            "type": "Text",
            "data": { "content_html": "<p><strong>Client:</strong> {{acf.client_name}} &bull; <strong>Impact:</strong> {{acf.key_metric}}</p>" },
            "styles": {}
          }
        ]
      }
    }
  }
}
```

---

## 6. Always Run the WCAG 2.1 Contrast, Link & SEO Linter (`daypress_audit_post`)

After creating or updating any page layout, run `daypress_audit_post` to verify accessibility and SEO health:

```json
{
  "name": "daypress_audit_post",
  "arguments": {
    "post_id": 3,
    "theme": "light"
  }
}
```
Fix any reported issues before finishing:
- Ensure every page has exactly one `Heading` with `level: 1` (`H1`) and no skipped heading levels (`H1` → `H2` → `H3`).
- Ensure text/background color pairs pass WCAG 2.1 AA (`>= 4.5:1` for body text, `>= 3.0:1` for large headings).
- Ensure every `Image` node has a descriptive `alt` attribute and every internal link resolves.
- Set `seo_title` and `seo_description` via `daypress_update_post`.
