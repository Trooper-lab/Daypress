//! Integration and Adversarial Test Suite for Milestone 4:
//! Backend Post Editor & DayStudio Visual Builder Parity & Containment.

use daypress_server::domain::models::*;
use daypress_server::templates::admin::builder::scoping::scope_theme_css;
use daypress_server::{templates, Database};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[test]
fn test_feature17_custom_fields_crud_and_null_deletion() {
    let db = Database::new_in_memory().expect("Failed to initialize test DB");

    // 1. Create post with custom meta
    let create_req = CreatePostRequest {
        title: "Milestone 4 Custom Fields Test".to_string(),
        content: "<p>Verifying postmeta CRUD</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("m4-custom-fields".to_string()),
        post_type: "post".to_string(),
        meta: Some(json!({
            "hero_heading": "Native Rust CMS",
            "view_count": 42,
            "is_featured": true,
            "layout_config": {"sidebar": false, "theme": "cad_dark"}
        })),
        ..Default::default()
    };

    let post = db
        .create_post(&create_req)
        .expect("Failed to create post with meta");
    assert_eq!(post.slug, "m4-custom-fields");

    // Verify all keys persisted and retrieved
    let fetched = db
        .get_post_by_id(post.id)
        .expect("Query failed")
        .expect("Post not found");
    assert_eq!(
        fetched.meta.get("hero_heading"),
        Some(&json!("Native Rust CMS"))
    );
    assert_eq!(fetched.meta.get("view_count"), Some(&json!(42)));
    assert_eq!(fetched.meta.get("is_featured"), Some(&json!(true)));
    assert_eq!(
        fetched.meta.get("layout_config"),
        Some(&json!({"sidebar": false, "theme": "cad_dark"}))
    );

    // 2. Update post: update hero_heading, add cta_url, and DELETE view_count via Null
    let update_req = UpdatePostRequest {
        meta: Some(json!({
            "hero_heading": "Daypress Hyper-Scale",
            "cta_url": "/benchmarks/",
            "view_count": serde_json::Value::Null
        })),
        ..Default::default()
    };

    let updated = db
        .update_post(post.id, &update_req)
        .expect("Failed to update post meta")
        .expect("Post not found");

    // 3. Verify in SQLite that view_count was deleted, hero_heading was updated, and cta_url was added
    assert_eq!(
        updated.meta.get("hero_heading"),
        Some(&json!("Daypress Hyper-Scale"))
    );
    assert_eq!(updated.meta.get("cta_url"), Some(&json!("/benchmarks/")));
    assert_eq!(
        updated.meta.get("view_count"),
        None,
        "Setting meta to Null must delete row from postmeta"
    );
    assert_eq!(
        updated.meta.get("is_featured"),
        Some(&json!(true)),
        "Existing keys not specified in update must remain intact"
    );
}

#[test]
fn test_feature17_custom_fields_editor_markup() {
    let db = Database::new_in_memory().expect("Failed to init DB");
    let settings = db.get_settings().unwrap_or_default();

    let create_req = CreatePostRequest {
        title: "Custom Fields UI Test".to_string(),
        content: "<p>UI verification</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("custom-fields-ui-test".to_string()),
        post_type: "post".to_string(),
        meta: Some(json!({
            "client_name": "Acme Aerospace",
            "rating": 5,
            "active": true
        })),
        ..Default::default()
    };
    let post = db.create_post(&create_req).expect("Create post");

    let html = templates::render_post_new_view(
        &settings,
        "post",
        Some(&post),
        &[],
        &[],
        &[],
        &[],
        false,
        &[],
    );

    // Assert Custom Fields card markup
    assert!(
        html.contains("id=\"custom_fields_card\""),
        "HTML must contain #custom_fields_card"
    );
    assert!(
        html.contains("Dynamic Custom Fields (<code>postmeta</code>)"),
        "HTML must render Dynamic Custom Fields title"
    );
    assert!(
        html.contains("value=\"Acme Aerospace\""),
        "HTML must render client_name meta value"
    );
    assert!(
        html.contains("META: NUMBER"),
        "HTML must render META: NUMBER badge for rating"
    );
    assert!(
        html.contains("META: BOOLEAN"),
        "HTML must render META: BOOLEAN badge for active"
    );
    assert!(
        html.contains("addCustomMetaRow()"),
        "HTML must include addCustomMetaRow handler"
    );
    assert!(
        html.contains("deleteCustomMetaRow("),
        "HTML must include deleteCustomMetaRow handler"
    );
}

#[test]
fn test_feature18_dynamic_taxonomies_discovery_and_persistence() {
    let db = Database::new_in_memory().expect("Failed to init DB");

    // Seed custom taxonomy in term_taxonomy via create_term
    let term1 = db
        .create_term(
            "portfolio_dept",
            &CreateTermRequest {
                name: "Design Systems".to_string(),
                slug: Some("design-systems".to_string()),
                description: Some("Design Systems Department".to_string()),
                parent: None,
            },
        )
        .expect("Create term 1");

    let term2 = db
        .create_term(
            "portfolio_dept",
            &CreateTermRequest {
                name: "Rust Core".to_string(),
                slug: Some("rust-core".to_string()),
                description: Some("Rust Core Engineering".to_string()),
                parent: None,
            },
        )
        .expect("Create term 2");

    // Test discovery for custom taxonomy
    let taxes = db
        .get_taxonomies_for_post_type("portfolio", None)
        .expect("Discovery failed");
    let dept_tax = taxes.iter().find(|t| t.slug == "portfolio_dept");
    assert!(
        dept_tax.is_some(),
        "Custom taxonomy portfolio_dept must be discovered"
    );
    assert_eq!(dept_tax.unwrap().terms.len(), 2);

    // Test assignment via create_post
    let mut tax_map = HashMap::new();
    tax_map.insert("portfolio_dept".to_string(), vec![term1.id]);

    let create_req = CreatePostRequest {
        title: "Portfolio Item Alpha".to_string(),
        content: "<p>Alpha item</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("portfolio-alpha".to_string()),
        post_type: "portfolio".to_string(),
        taxonomies: Some(tax_map),
        ..Default::default()
    };

    let post = db
        .create_post(&create_req)
        .expect("Create post with taxonomy failed");

    // Verify assigned_term_ids on post
    let post_taxes = db
        .get_taxonomies_for_post_type("portfolio", Some(post.id))
        .expect("Get post taxonomies");
    let post_dept = post_taxes
        .iter()
        .find(|t| t.slug == "portfolio_dept")
        .expect("Find portfolio_dept");
    assert_eq!(
        post_dept.assigned_term_ids,
        vec![term1.id],
        "Term assignment must be reflected in assigned_term_ids"
    );

    // Update assignment via update_post
    let mut update_tax_map = HashMap::new();
    update_tax_map.insert("portfolio_dept".to_string(), vec![term2.id]);

    let update_req = UpdatePostRequest {
        taxonomies: Some(update_tax_map),
        ..Default::default()
    };

    db.update_post(post.id, &update_req)
        .expect("Update post taxonomies failed");

    // Verify old term was cleared and new term was inserted
    let updated_taxes = db
        .get_taxonomies_for_post_type("portfolio", Some(post.id))
        .expect("Get updated post taxonomies");
    let updated_dept = updated_taxes
        .iter()
        .find(|t| t.slug == "portfolio_dept")
        .expect("Find portfolio_dept");
    assert_eq!(
        updated_dept.assigned_term_ids,
        vec![term2.id],
        "Updated taxonomy term assignment must be reflected"
    );
}

#[test]
fn test_feature19_lossless_gutenberg_block_comments_preservation() {
    let db = Database::new_in_memory().expect("Failed to init DB");

    let original_content = r#"<!-- wp:heading {"level":2} -->
<h2>Next-Gen High Throughput CMS</h2>
<!-- /wp:heading -->

<!-- wp:spacer {"height":"32px"} /-->

<!-- wp:paragraph -->
<p>Delivering sub-millisecond SSR performance with native Rust execution.</p>
<!-- /wp:paragraph -->"#;

    let create_req = CreatePostRequest {
        title: "Gutenberg Preservation Test".to_string(),
        content: original_content.to_string(),
        status: "publish".to_string(),
        slug: Some("gutenberg-preservation".to_string()),
        post_type: "post".to_string(),
        ..Default::default()
    };

    let post = db
        .create_post(&create_req)
        .expect("Failed to create post with Gutenberg content");

    let fetched = db
        .get_post_by_id(post.id)
        .expect("Query failed")
        .expect("Post not found");

    // Assert that Gutenberg block comments were NOT stripped during persistence
    assert!(
        fetched
            .content
            .rendered
            .contains("<!-- wp:heading {\"level\":2} -->"),
        "Gutenberg block comment open tag must be preserved"
    );
    assert!(
        fetched.content.rendered.contains("<!-- /wp:heading -->"),
        "Gutenberg block comment close tag must be preserved"
    );
    assert!(
        fetched
            .content
            .rendered
            .contains("<!-- wp:spacer {\"height\":\"32px\"} /-->"),
        "Self-closing Gutenberg spacer block must be preserved"
    );

    // Verify AST was synchronized in SQLite
    let layout_str = db
        .get_post_layout(post.id)
        .expect("Query failed")
        .expect("post layout must be automatically synchronized in SQLite");
    assert!(
        layout_str.contains("root"),
        "AST layout root must be synthesized from content"
    );

    // Assert cleanWpContent logic protects wp comments
    let scripts = templates::render_editor_scripts(
        "Daypress",
        &post.id.to_string(),
        true,
        "blocks",
        "Post",
        "post",
        "[]",
    );
    assert!(
        scripts.contains("function cleanWpContent(raw)"),
        "Editor scripts must contain cleanWpContent"
    );
    assert!(
        scripts.contains("<!--(?!\\s*\\/?wp:)"),
        "cleanWpContent must specifically protect wp: comments"
    );
}

#[test]
fn test_feature20_21_daystudio_builder_containment_and_cad_invariants() {
    let db = Database::new_in_memory().expect("Failed to init DB");

    // 1. Test CSS scoping engine
    let test_theme_css = r#"
        :root { --primary: #0066ff; }
        body { margin: 0; background-color: #f8fafc; font-family: sans-serif; }
        .hero-banner { min-height: 400px; display: flex; }
    "#;
    let scoped = scope_theme_css(test_theme_css);
    assert!(
        scoped.contains("@scope (#daystudio-artboard) {"),
        "CSS scoping must wrap in @scope (#daystudio-artboard)"
    );
    assert!(
        scoped.contains(":scope {"),
        ":root and body must be converted to :scope"
    );
    assert!(
        !scoped.contains("body {"),
        "Standalone body selector must not leak outside artboard"
    );

    // 2. Test DayStudio builder view markup and CSS containment
    let mut settings = db.get_settings().unwrap_or_default();
    settings.custom_css = "body { color: #111; }".to_string();

    let create_req = CreatePostRequest {
        title: "DayStudio Containment Test".to_string(),
        content: "<section class=\"section white\"><h2>Hero</h2></section>".to_string(),
        status: "publish".to_string(),
        slug: Some("daystudio-containment".to_string()),
        post_type: "page".to_string(),
        ..Default::default()
    };
    let post = db.create_post(&create_req).expect("Create test page");

    let builder_html = templates::render_builder_view(&settings, &post, None);

    // Topbar CAD invariant: height 40px, max-height 44px
    assert!(
        builder_html.contains("#daystudio-topbar {"),
        "Must contain #daystudio-topbar style definition"
    );
    assert!(
        builder_html.contains("max-height: 44px !important;"),
        "Topbar must enforce max-height: 44px !important"
    );
    assert!(
        builder_html.contains("height: 40px !important;"),
        "Topbar must enforce height: 40px !important"
    );

    // Bottom-bar CAD invariant: height 28px, max-height 32px
    assert!(
        builder_html.contains("#daystudio-bottom-bar {"),
        "Must contain #daystudio-bottom-bar style definition"
    );
    assert!(
        builder_html.contains("max-height: 32px !important;"),
        "Bottom-bar must enforce max-height: 32px !important"
    );
    assert!(
        builder_html.contains("height: 28px !important;"),
        "Bottom-bar must enforce height: 28px !important"
    );

    // Artboard Stacking Context & Containment isolation
    assert!(
        builder_html.contains("contain: layout paint !important;"),
        "Artboard must enforce contain: layout paint !important"
    );
    assert!(
        builder_html.contains("transform: translateZ(0) !important;"),
        "Artboard must enforce transform: translateZ(0) !important"
    );
    assert!(
        builder_html.contains("isolation: isolate !important;"),
        "Artboard must enforce isolation: isolate !important"
    );
    assert!(
        builder_html.contains("overflow: clip !important;"),
        "Artboard must enforce overflow: clip !important"
    );

    // Chrome isolation reset
    assert!(
        builder_html
            .contains("#daystudio-shell *:not(#daystudio-artboard):not(#daystudio-artboard *) {"),
        "Shell must enforce chrome isolation reset"
    );

    // Flanking resize handles positioned as siblings to avoid clip clipping
    assert!(
        builder_html.contains("id=\"handle-artboard-left\""),
        "Left handle must be present"
    );
    assert!(
        builder_html.contains("id=\"handle-artboard-right\""),
        "Right handle must be present"
    );
    let left_handle_idx = builder_html
        .find("id=\"handle-artboard-left\"")
        .expect("Left handle");
    let artboard_idx = builder_html
        .find("id=\"daystudio-artboard\"")
        .expect("Artboard");
    let right_handle_idx = builder_html
        .find("id=\"handle-artboard-right\"")
        .expect("Right handle");

    assert!(
        left_handle_idx < artboard_idx,
        "Left handle must precede artboard as flanking sibling"
    );
    assert!(
        artboard_idx < right_handle_idx,
        "Right handle must follow artboard as flanking sibling"
    );
}

#[test]
fn test_codebase_line_count_invariants() {
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
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", full_path.display(), e));
        let line_count = content.lines().count();
        assert!(
            line_count <= 400,
            "File {} has {} lines, which violates the <= 400 lines invariant!",
            rel_path,
            line_count
        );
    }
}
