//! Adversarial Stress and Verification Tests for DayStudio Box-Model, Overlays & CAD Tokens.
//!
//! Milestone 3 Challenger verification for:
//! 1. Trapezoidal Box-Model widget (`scripts/inspector_box_model.rs`, `styles/box_model.rs`)
//! 2. Live canvas guide overlays (`scripts/canvas_overlays.rs`, `styles/overlays.rs`)
//! 3. CAD Tokens & Typography (`styles/tokens.rs`)
//! 4. Responsive Breakpoints & Zoom Engine (`styles/canvas.rs`, `scripts/topbar.rs`)

use daypress_server::adapters::database::Database;
use daypress_server::templates::admin::builder::scripts;
use daypress_server::templates::admin::builder::styles;
use daypress_server::templates::render_builder_view;

#[test]
fn test_cad_tokens_rigorous_verification() {
    let css = styles::tokens::CSS_TOKENS;

    // 1. Verify the Night edition surface and ink tokens
    assert!(
        css.contains("--day-chassis:         #111214;"),
        "Chassis token must be the Night edition paper #111214"
    );
    assert!(
        css.contains("--day-surface:         #1A1C1F;"),
        "Surface token must be the Night edition paper-raised #1A1C1F"
    );
    assert!(
        css.contains("--day-rule:            #393836;"),
        "Hairline rule token must match #393836"
    );
    assert!(
        css.contains("--day-ink:             #EFE9DE;"),
        "Ink token must be the Night edition ink #EFE9DE"
    );
    assert!(
        css.contains("--day-muted:           #A7A297;"),
        "Muted token must be the Night edition ink-muted #A7A297"
    );

    // 2. Verify Signal and Token Accents
    assert!(
        css.contains("--day-signal:          #93A1FF;"),
        "Signal must be the Night edition press #93A1FF"
    );
    assert!(
        css.contains("--day-token:           #E6B84A;"),
        "Token accent must be the Night edition ochre #E6B84A"
    );

    // 3. Verify Typography Font Stacks
    assert!(
        css.contains("\"Schibsted Grotesk\""),
        "UI font family must be Schibsted Grotesk"
    );
    assert!(
        css.contains("\"IBM Plex Mono\""),
        "Mono font family must be IBM Plex Mono"
    );

    // 4. Verify Control Heights and Metrics
    assert!(
        css.contains("--day-topbar-h:        40px;"),
        "Topbar height must be 40px"
    );
    assert!(
        css.contains("--day-rail-w:          48px;"),
        "Rail width must be 48px"
    );
    assert!(
        css.contains("--day-drawer-w:        280px;"),
        "Drawer width must be 280px"
    );
    assert!(
        css.contains("--day-inspector-w:     300px;"),
        "Inspector width must be 300px"
    );
    assert!(
        css.contains("--day-control-h:       26px;"),
        "CAD control row height must be 26px"
    );
}

#[test]
fn test_responsive_breakpoints_and_artboard_dimensions() {
    let canvas_css = styles::canvas::CSS_CANVAS;
    let topbar_js = scripts::topbar::JS_TOPBAR;

    // 1. Verify Artboard Breakpoint Widths in CSS
    assert!(canvas_css.contains("#daystudio-artboard.artboard-desktop {\n    width: 1200px;\n}"));
    assert!(canvas_css.contains("#daystudio-artboard.artboard-tablet {\n    width: 768px;\n}"));
    assert!(canvas_css.contains("#daystudio-artboard.artboard-mobile {\n    width: 375px;\n}"));

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
    assert!(bm_css.contains(".day-box-model-widget"));
    // Amber for Margin: rgba(245, 158, 11, ...)
    assert!(bm_css.contains(".day-box-margin-outer"));
    assert!(bm_css.contains("rgba(245, 158, 11, 0.06)"));
    assert!(bm_css.contains("rgba(245, 158, 11, 0.35)"));

    // Cobalt for Padding: rgba(37, 99, 235, ...)
    assert!(bm_css.contains(".day-box-padding-inner"));
    assert!(bm_css.contains("rgba(37, 99, 235, 0.07)"));
    assert!(bm_css.contains("rgba(37, 99, 235, 0.35)"));

    // Core content dimensions
    assert!(bm_css.contains(".day-box-content-core"));
    assert!(bm_css.contains(".day-box-facet"));

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
    assert!(ov_css.contains("#daystudio-box-overlays"));
    assert!(ov_css.contains(".overlay-margin"));
    assert!(ov_css.contains("var(--day-token-tint)"));
    assert!(ov_css.contains("var(--day-token-border)"));

    assert!(ov_css.contains(".overlay-padding"));
    assert!(ov_css.contains("var(--day-signal-tint)"));
    assert!(ov_css.contains("var(--day-signal-border)"));

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

    // 4. A click without dragging opens the inline text input
    assert!(bm_js.contains("day-box-model-inline-input"));
    assert!(bm_js.contains("openFacetInput(container, facet, node, prop, isPadding);"));
}

#[test]
fn test_builder_rendered_view_contains_all_components_and_zero_leaks() {
    let db = Database::new_in_memory().unwrap();
    let settings = db.get_settings().unwrap_or_default();
    let posts = db.get_posts("post", None, None, 1).unwrap();
    let post = &posts[0];

    let html = render_builder_view(&settings, post, None);

    // Verify template interpolation without unresolved template tags
    assert!(!html.contains("__STYLES__"));
    assert!(!html.contains("__EXTRA_HEAD_CSS__"));
    assert!(!html.contains("__PREVIEW_HEADER_HTML__"));
    assert!(!html.contains("__PREVIEW_FOOTER_HTML__"));
    assert!(!html.contains("__LAYOUT_JSON_RAW__"));

    // Verify Box Model & Overlays elements in rendered markup
    assert!(html.contains("id=\"daystudio-box-overlays\""));
    assert!(html.contains("id=\"overlay-margin-top\""));
    assert!(html.contains("id=\"overlay-padding-top\""));
    assert!(html.contains("class=\"day-box-margin-outer\""));
    assert!(html.contains("class=\"day-box-padding-inner\""));
}
