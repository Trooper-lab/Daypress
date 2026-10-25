//! Adversarial Stress and Verification Tests for NeoStudio Box-Model, Overlays & CAD Tokens.
//!
//! Milestone 3 Challenger verification for:
//! 1. Trapezoidal Box-Model widget (`scripts/inspector_box_model.rs`, `styles/box_model.rs`)
//! 2. Live canvas guide overlays (`scripts/canvas_overlays.rs`, `styles/overlays.rs`)
//! 3. CAD Tokens & Typography (`styles/tokens.rs`)
//! 4. Responsive Breakpoints & Zoom Engine (`styles/canvas.rs`, `scripts/topbar.rs`)

use neopress_server::adapters::database::Database;
use neopress_server::templates::admin::builder::scripts;
use neopress_server::templates::admin::builder::styles;
use neopress_server::templates::render_wp_builder_view;

#[test]
fn test_cad_tokens_rigorous_verification() {
    let css = styles::tokens::CSS_TOKENS;

    // 1. Verify 5 Named Hex Tokens
    assert!(
        css.contains("--neo-chassis:         #0D0F12;"),
        "Obsidian Chassis token must match #0D0F12"
    );
    assert!(
        css.contains("--neo-surface:         #15181E;"),
        "Graphite Surface token must match #15181E"
    );
    assert!(
        css.contains("--neo-rule:            #242933;"),
        "Hairline Rule token must match #242933"
    );
    assert!(
        css.contains("--neo-ink:             #F2F4F8;"),
        "Alabaster Ink token must match #F2F4F8"
    );
    assert!(
        css.contains("--neo-muted:           #8A93A6;"),
        "Slate Muted token must match #8A93A6"
    );

    // 2. Verify Signal and Token Accents
    assert!(
        css.contains("--neo-signal:          #2563EB;"),
        "Cobalt Signal must match #2563EB"
    );
    assert!(
        css.contains("--neo-token:           #F59E0B;"),
        "Amber Token must match #F59E0B"
    );

    // 3. Verify Typography Font Stacks
    assert!(
        css.contains("\"Inter\", \"Plus Jakarta Sans\""),
        "UI font family must include Inter and Plus Jakarta Sans"
    );
    assert!(
        css.contains("\"JetBrains Mono\""),
        "Mono font family must include JetBrains Mono"
    );

    // 4. Verify Control Heights and Metrics
    assert!(
        css.contains("--neo-topbar-h:        40px;"),
        "Topbar height must be 40px"
    );
    assert!(
        css.contains("--neo-rail-w:          48px;"),
        "Rail width must be 48px"
    );
    assert!(
        css.contains("--neo-drawer-w:        280px;"),
        "Drawer width must be 280px"
    );
    assert!(
        css.contains("--neo-inspector-w:     300px;"),
        "Inspector width must be 300px"
    );
    assert!(
        css.contains("--neo-control-h:       26px;"),
        "CAD control row height must be 26px"
    );
}

#[test]
fn test_responsive_breakpoints_and_artboard_dimensions() {
    let canvas_css = styles::canvas::CSS_CANVAS;
    let topbar_js = scripts::topbar::JS_TOPBAR;

    // 1. Verify Artboard Breakpoint Widths in CSS
    assert!(canvas_css.contains("#neostudio-artboard.artboard-desktop {\n    width: 1200px;\n}"));
    assert!(canvas_css.contains("#neostudio-artboard.artboard-tablet {\n    width: 768px;\n}"));
    assert!(canvas_css.contains("#neostudio-artboard.artboard-mobile {\n    width: 375px;\n}"));

    // 2. Verify Viewport Drag Handles
    assert!(canvas_css.contains(".artboard-handle"));
    assert!(canvas_css.contains(".artboard-handle-left"));
    assert!(canvas_css.contains(".artboard-handle-right"));
    assert!(canvas_css.contains("cursor: ew-resize;"));

    // 3. Verify Topbar Breakpoint Switching Script
    assert!(topbar_js.contains("setBreakpoint(targetBp)"));
    assert!(topbar_js.contains("artboard.classList.add(\"artboard-\" + bp)"));
    assert!(topbar_js.contains("768 <span class=\"dim-unit\">px</span> &times; 100%"));
    assert!(topbar_js.contains("375 <span class=\"dim-unit\">px</span> &times; 100%"));
    assert!(topbar_js.contains("1200 <span class=\"dim-unit\">px</span> &times; 100%"));

    // 4. Verify Zoom Engine Bounds (50% - 150%)
    assert!(topbar_js.contains("Math.max(0.5, Math.min(1.5, Math.round(lvl * 100) / 100))"));
    assert!(topbar_js.contains("scale(${lvl})"));
}

#[test]
fn test_box_model_trapezoid_widget_and_facets() {
    let bm_css = styles::box_model::CSS_BOX_MODEL;
    let bm_js = scripts::inspector_box_model::JS_INSPECTOR_BOX_MODEL;

    // 1. Verify Trapezoidal Box-Model CSS Structure & Palettes
    assert!(bm_css.contains(".neo-box-model-widget"));
    // Amber for Margin: rgba(245, 158, 11, ...)
    assert!(bm_css.contains(".neo-box-margin-outer"));
    assert!(bm_css.contains("rgba(245, 158, 11, 0.06)"));
    assert!(bm_css.contains("rgba(245, 158, 11, 0.35)"));

    // Cobalt for Padding: rgba(37, 99, 235, ...)
    assert!(bm_css.contains(".neo-box-padding-inner"));
    assert!(bm_css.contains("rgba(37, 99, 235, 0.07)"));
    assert!(bm_css.contains("rgba(37, 99, 235, 0.35)"));

    // Core content dimensions
    assert!(bm_css.contains(".neo-box-content-core"));
    assert!(bm_css.contains(".neo-box-facet"));

    // 2. Verify Box-Model Script Facets & Interactive Controls (facets generated per group/side)
    assert!(bm_js.contains("const SIDES = [\"top\", \"right\", \"bottom\", \"left\"];"));
    assert!(bm_js.contains("data-prop=\"${group}_${side}\""));
    assert!(bm_js.contains("SIDES.map(s => facet(\"margin\", s))"));
    assert!(bm_js.contains("SIDES.map(s => facet(\"padding\", s))"));

    // Tools: Unit cycling & Link quads
    assert!(bm_js.contains("id=\"btn-cycle-box-unit\""));
    assert!(bm_js.contains("id=\"btn-toggle-box-link\""));
}

#[test]
fn test_canvas_guide_overlays_amber_and_cobalt() {
    let ov_css = styles::overlays::CSS_OVERLAYS;
    let ov_js = scripts::canvas_overlays::JS_CANVAS_OVERLAYS;

    // 1. Verify Live Guide CSS Overlay Classes & Tokens
    assert!(ov_css.contains("#neostudio-box-overlays"));
    assert!(ov_css.contains(".overlay-margin"));
    assert!(ov_css.contains("var(--neo-token-tint)"));
    assert!(ov_css.contains("var(--neo-token-border)"));

    assert!(ov_css.contains(".overlay-padding"));
    assert!(ov_css.contains("var(--neo-signal-tint)"));
    assert!(ov_css.contains("var(--neo-signal-border)"));

    assert!(ov_css.contains(".overlay-chip"));

    // 2. Verify Live Guide Overlays Geometry Script
    assert!(ov_js.contains("setOverlayFacet(\"overlay-margin-top\""));
    assert!(ov_js.contains("setOverlayFacet(\"overlay-margin-bottom\""));
    assert!(ov_js.contains("setOverlayFacet(\"overlay-margin-left\""));
    assert!(ov_js.contains("setOverlayFacet(\"overlay-margin-right\""));

    assert!(ov_js.contains("setOverlayFacet(\"overlay-padding-top\""));
    assert!(ov_js.contains("setOverlayFacet(\"overlay-padding-bottom\""));
    assert!(ov_js.contains("setOverlayFacet(\"overlay-padding-left\""));
    assert!(ov_js.contains("setOverlayFacet(\"overlay-padding-right\""));
}

#[test]
fn test_scrubber_modifiers_and_clamping_script_source() {
    let bm_js = scripts::inspector_box_model::JS_INSPECTOR_BOX_MODEL;

    // 1. Scrubber Drag Math: Shift 10x, Alt 0.1x
    assert!(bm_js.contains("me.shiftKey ? 10 : (me.altKey ? 0.1 : 1)"));

    // 2. Padding Clamping: Padding clamped to 0 (scrub + typed), Margin negative allowed
    assert!(bm_js.contains("if (isPadding && n < 0) n = 0;"));
    assert!(bm_js.matches("if (isPadding && n < 0) n = 0;").count() >= 2);

    // 3. Link Quads Toggle Logic (linked edits fan out to all four sides)
    assert!(bm_js.contains("if (!isLinked) return [prop];"));
    assert!(bm_js.contains("SIDES.map(s => group + \"_\" + s)"));

    // 4. Double click inline text input
    assert!(bm_js.contains("neo-box-model-inline-input"));
    assert!(bm_js.contains("facet.addEventListener(\"dblclick\""));
}

#[test]
fn test_builder_rendered_view_contains_all_components_and_zero_leaks() {
    let db = Database::new_in_memory().unwrap();
    let settings = db.get_settings().unwrap_or_default();
    let posts = db.get_posts("post", None, None, 1).unwrap();
    let post = &posts[0];

    let html = render_wp_builder_view(&settings, post, None);

    // Verify template interpolation without unresolved template tags
    assert!(!html.contains("__STYLES__"));
    assert!(!html.contains("__EXTRA_HEAD_CSS__"));
    assert!(!html.contains("__PREVIEW_HEADER_HTML__"));
    assert!(!html.contains("__PREVIEW_FOOTER_HTML__"));
    assert!(!html.contains("__LAYOUT_JSON_RAW__"));

    // Verify Box Model & Overlays elements in rendered markup
    assert!(html.contains("id=\"neostudio-box-overlays\""));
    assert!(html.contains("id=\"overlay-margin-top\""));
    assert!(html.contains("id=\"overlay-padding-top\""));
    assert!(html.contains("class=\"neo-box-margin-outer\""));
    assert!(html.contains("class=\"neo-box-padding-inner\""));
}
