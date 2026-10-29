//! Empirical Challenger Test Suite for Milestone 4 R2
//! (`crates/daypress-server/tests/empirical_challenger_m4_r2_test.rs`)
//!
//! Validates:
//! 1. Adversarial CSS scoping resilience against dock chrome hijacking.
//! 2. CAD Dock topbar and bottom-bar invariants in rendered builder output.
//! 3. Artboard containment and Stacking Context isolation invariants.
//! 4. Viewport resize handles DOM placement outside clipped artboard.
//! 5. Modularity line count compliance (all touched files <= 400 lines).

use daypress_server::domain::models::*;
use daypress_server::templates::admin::builder::scoping::scope_theme_css;
use daypress_server::{templates, Database};
use std::fs;
use std::path::Path;

#[test]
fn test_adversarial_theme_css_scoping_isolation() {
    // Adversarial theme stylesheet attempting to break out of artboard and hijack CAD dock
    let adversarial_css = r#"
        :root {
            --theme-primary: #ff0055;
            --wp--preset--color--primary: #123456;
        }
        html, body {
            margin: 0;
            padding: 0;
            background-color: #000000;
        }
        body {
            font-family: 'Comic Sans MS', cursive;
            color: #333333;
        }
        #daystudio-topbar {
            height: 500px !important;
            background: red !important;
            display: none !important;
        }
        #daystudio-bottom-bar {
            height: 200px !important;
        }
        @media screen and (max-width: 768px) {
            body { font-size: 14px; }
            .hero-card { display: block; }
        }
    "#;

    let scoped = scope_theme_css(adversarial_css);

    // 1. Must wrap entire stylesheet in @scope (#daystudio-artboard)
    assert!(
        scoped.starts_with("@scope (#daystudio-artboard) {"),
        "Scoped CSS must begin with @scope (#daystudio-artboard)"
    );
    assert!(
        scoped.ends_with('}'),
        "Scoped CSS must close the @scope block"
    );

    // 2. Global root and body references must be rewritten to :scope
    assert!(
        scoped.contains(":scope {\n            --theme-primary: #ff0055;"),
        ":root must be rewritten to :scope"
    );
    assert!(
        scoped.contains(":scope {\n            margin: 0;"),
        "html, body must be rewritten to :scope"
    );
    assert!(
        scoped.contains(":scope {\n            font-family: 'Comic Sans MS', cursive;"),
        "body {{ must be rewritten to :scope {{"
    );
    assert!(
        !scoped.contains("body {"),
        "Standalone 'body {{' selector must not exist in output"
    );

    // 3. Hostile rules targeting outer CAD dock are strictly contained inside @scope
    // Because #daystudio-topbar is outside #daystudio-artboard in DOM, @scope ensures it never matches
    assert!(
        scoped.contains("#daystudio-topbar {"),
        "Rules remain present but confined to descendants of #daystudio-artboard"
    );

    // 4. Empty and whitespace inputs return empty string
    assert_eq!(scope_theme_css(""), "");
    assert_eq!(scope_theme_css("   \n\t  "), "");
}

#[test]
fn test_builder_view_rendered_cad_dock_and_containment_invariants() {
    let db = Database::new_in_memory().expect("Init in-memory DB");
    let mut settings = db.get_settings().unwrap_or_default();
    settings.custom_css = r#"
        :root { --accent: #2563eb; }
        body { background: #fafafa; }
    "#
    .to_string();

    let post = db
        .create_post(&CreatePostRequest {
            title: "CAD Dock & Containment Probe".to_string(),
            content: "<section><h2>Probing Artboard</h2></section>".to_string(),
            status: "publish".to_string(),
            slug: Some("cad-dock-probe".to_string()),
            post_type: "page".to_string(),
            ..Default::default()
        })
        .expect("Create probe page");

    let builder_html = templates::render_wp_builder_view(&settings, &post, None);

    // 1. Topbar CAD Invariants
    assert!(
        builder_html.contains("height: 40px !important;"),
        "Topbar must enforce height: 40px !important;"
    );
    assert!(
        builder_html.contains("max-height: 44px !important;"),
        "Topbar must enforce max-height: 44px !important;"
    );
    assert!(
        builder_html.contains("overflow: hidden !important;"),
        "Topbar and bottom-bar must enforce overflow: hidden !important;"
    );
    assert!(
        builder_html.contains("flex-shrink: 0 !important;"),
        "Topbar and bottom-bar must enforce flex-shrink: 0 !important;"
    );

    // 2. Bottom-Bar CAD Invariants
    assert!(
        builder_html.contains("height: 28px !important;"),
        "Bottom-bar must enforce height: 28px !important;"
    );
    assert!(
        builder_html.contains("max-height: 32px !important;"),
        "Bottom-bar must enforce max-height: 32px !important;"
    );

    // 3. Studio Chrome Isolation Reset
    assert!(
        builder_html
            .contains("#daystudio-shell *:not(#daystudio-artboard):not(#daystudio-artboard *) {"),
        "Shell must have chrome isolation reset"
    );

    // 4. Artboard Stacking Context & Paint Containment Invariants
    assert!(
        builder_html.contains("contain: layout paint !important;"),
        "Artboard must enforce contain: layout paint !important;"
    );
    assert!(
        builder_html.contains("transform: translateZ(0) !important;"),
        "Artboard must enforce transform: translateZ(0) !important;"
    );
    assert!(
        builder_html.contains("isolation: isolate !important;"),
        "Artboard must enforce isolation: isolate !important;"
    );
    assert!(
        builder_html.contains("overflow: clip !important;"),
        "Artboard must enforce overflow: clip !important;"
    );

    // 5. Flanking Resize Handles Placement Outside Artboard
    let left_h = builder_html
        .find("id=\"handle-artboard-left\"")
        .expect("handle-artboard-left");
    let artboard = builder_html
        .find("id=\"daystudio-artboard\"")
        .expect("daystudio-artboard");
    let right_h = builder_html
        .find("id=\"handle-artboard-right\"")
        .expect("handle-artboard-right");
    assert!(
        left_h < artboard,
        "Left resize handle must precede artboard"
    );
    assert!(
        artboard < right_h,
        "Right resize handle must follow artboard"
    );

    // 6. Theme CSS Scoping in Builder <head>
    assert!(
        builder_html.contains("@scope (#daystudio-artboard) {"),
        "Theme custom_css must be injected wrapped in @scope (#daystudio-artboard)"
    );
    assert!(
        builder_html.contains(":scope { background: #fafafa; }"),
        "Theme custom_css body selector must be rewritten to :scope"
    );
}

#[test]
fn test_milestone4_all_touched_files_line_counts_strictly_under_400() {
    let touched_files = [
        "src/adapters/database/post_ast.rs",
        "src/adapters/database/taxonomies_discovery.rs",
        "src/handlers/admin_extra.rs",
        "src/handlers/admin_pages.rs",
        "src/templates/admin/editor/markup/custom_fields.rs",
        "src/templates/admin/editor/markup/sidebar.rs",
        "src/templates/admin/editor/markup/mod.rs",
        "src/templates/admin/editor/scripts/core.rs",
        "src/templates/admin/editor/scripts/acf_blocks.rs",
        "src/templates/admin/editor/scripts/acf_render.rs",
        "src/templates/admin/editor/scripts/actions.rs",
        "src/templates/admin/editor/scripts/mod.rs",
        "src/templates/admin/builder/scoping.rs",
        "src/templates/admin/builder/styles/canvas.rs",
        "src/templates/admin/builder/styles/topbar.rs",
        "src/templates/admin/builder/styles/shell.rs",
        "src/templates/admin/builder/markup/shell.rs",
        "src/templates/admin/builder/mod.rs",
    ];

    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel_path in &touched_files {
        let full_path = base_dir.join(rel_path);
        assert!(
            full_path.exists(),
            "Expected file to exist: {}",
            full_path.display()
        );
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Could not read {}: {}", full_path.display(), e));
        let lines = content.lines().count();
        assert!(
            lines <= 400,
            "File {} exceeds 400 lines: {} lines",
            rel_path,
            lines
        );
    }
}
