//! Adversarial Challenger Stress Tests for Milestone 3 (NeoStudio Visual Builder UI & Persistence).
//!
//! Evaluates:
//! 1. SQLite dual-write persistence & nonce auth (`X-WP-Nonce: wp_rest_admin`) to `wp_posts` and `wp_postmeta`.
//! 2. Zero-mock verification in client API scripts (ensuring no toast-only persistence stubs).
//! 3. DOM leak firewall stripping 100% of builder chrome attributes.
//! 4. Navigator hierarchy, drag-and-drop reordering, and cycle prevention.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use neopress_ast::sanitize_ssr_html;
use neopress_server::adapters::database::Database;
use neopress_server::create_app;
use neopress_server::templates::admin::builder::markup::modals::render_modals;
use neopress_server::templates::admin::builder::scripts::api_client::JS_API_CLIENT;
use neopress_server::templates::admin::builder::scripts::canvas_pro_commercial::JS_CANVAS_PRO_COMMERCIAL;
use neopress_server::templates::admin::builder::scripts::canvas_pro_dynamic::JS_CANVAS_PRO_DYNAMIC;
use neopress_server::templates::admin::builder::scripts::canvas_pro_interactive::JS_CANVAS_PRO_INTERACTIVE;
use neopress_server::templates::admin::builder::scripts::canvas_pro_widgets::JS_CANVAS_PRO_WIDGETS;
use neopress_server::templates::admin::builder::scripts::canvas_selection::JS_CANVAS_SELECTION;
use neopress_server::templates::admin::builder::scripts::core_state::JS_CORE_STATE;
use neopress_server::templates::admin::builder::scripts::inspector_content::JS_INSPECTOR_CONTENT;
use neopress_server::templates::admin::builder::scripts::inspector_core::JS_INSPECTOR_CORE;
use neopress_server::templates::admin::builder::scripts::navigator::JS_NAVIGATOR;
use neopress_server::templates::admin::builder::scripts::theme_drawer::JS_THEME_DRAWER;
use neopress_server::theme_builder::models::*;
use neopress_server::theme_builder::repository::ThemeTemplateRepository;
use tower::ServiceExt;

#[tokio::test]
async fn test_adversarial_sqlite_persistence_dual_write_and_nonce_auth() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db.clone());

    // 1. Post layout save payload using exact X-WP-Nonce header sent by `scripts/api_client.rs`
    let post_id = 1i64;
    let complex_ast = serde_json::json!({
        "layout": {
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": [
                    {
                        "id": "sec_hero",
                        "type": "Section",
                        "styles": { "background_color": "#0D0F12", "padding_top": "64px" },
                        "children": [
                            {
                                "id": "card_pricing",
                                "type": "PriceCard",
                                "data": {
                                    "title": "Cloud Scale",
                                    "price": "299",
                                    "currency": "$",
                                    "period": "/yr"
                                }
                            },
                            {
                                "id": "timer_launch",
                                "type": "Countdown",
                                "data": {
                                    "target_date": "2027-01-01T00:00:00Z"
                                }
                            }
                        ]
                    }
                ]
            }
        },
        "layout_json": {
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": []
            }
        }
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/neopress/v1/layout/{}", post_id))
                .header("Content-Type", "application/json")
                .header("X-WP-Nonce", "wp_rest_admin")
                .body(Body::from(complex_ast.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        StatusCode::OK,
        "POST /layout/:id with X-WP-Nonce: wp_rest_admin must authenticate and return 200 OK"
    );

    // 2. Direct SQLite verification on `wp_posts`:
    //    Must store canonical JSON in `layout_json` AND pre-compiled SSR HTML in `post_content`.
    let db_layout = db
        .get_post_layout(post_id)
        .expect("Query failed")
        .expect("Post layout must exist");

    assert!(
        db_layout.contains("PriceCard"),
        "Stored layout must contain PriceCard node"
    );
    assert!(
        db_layout.contains("Countdown"),
        "Stored layout must contain Countdown node"
    );

    let post = db
        .get_post_by_id(post_id)
        .expect("Query failed")
        .expect("Post 1 must exist");

    let content_str = &post.content.rendered;
    assert!(
        content_str.contains("<section") || content_str.contains("np-section") || content_str.contains("np-price-card"),
        "wp_posts.post_content must contain compiled semantic HTML from SSR pipeline"
    );

    // 3. Test Theme Template persistence via PUT /wp-json/neopress/v1/templates/:id with X-WP-Nonce
    let repo = ThemeTemplateRepository::from_database(&db);
    let created_tpl = repo
        .create(&CreateTemplateRequest {
            title: "Original Header".to_string(),
            template_type: TemplateType::Header,
            status: Some("publish".to_string()),
            layout: None,
            layout_json: None,
            settings: None,
            priority: Some(10),
            is_active: Some(true),
            conditions: None,
        })
        .expect("Failed to create starter template");

    let update_payload = serde_json::json!({
        "title": "Updated Modern Header",
        "status": "publish",
        "layout": {
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": [
                    {
                        "id": "hdr_nav",
                        "type": "NavMenu",
                        "data": { "menu_name": "Main Navigation" }
                    }
                ]
            }
        }
    });

    let put_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/wp-json/neopress/v1/templates/{}", created_tpl.id))
                .header("Content-Type", "application/json")
                .header("X-WP-Nonce", "wp_rest_admin")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        put_res.status(),
        StatusCode::OK,
        "PUT /templates/:id with X-WP-Nonce must succeed"
    );

    // Verify in SQLite `wp_neopress_templates`
    let fetched = repo.get_by_id(created_tpl.id).unwrap().unwrap();
    assert_eq!(fetched.title, "Updated Modern Header");
    assert!(
        serde_json::to_string(&fetched.layout)
            .unwrap()
            .contains("NavMenu"),
        "Template layout in SQLite must contain NavMenu"
    );
}

#[test]
fn test_adversarial_zero_mock_persistence_in_scripts() {
    // Verify that StudioApi in api_client.rs contains real fetch calls for all operations
    assert!(
        JS_API_CLIENT.contains("async saveLayout()"),
        "StudioApi must implement saveLayout"
    );
    assert!(
        JS_API_CLIENT.contains("/wp-json/neopress/v1/layout/"),
        "saveLayout must target /wp-json/neopress/v1/layout/:id"
    );
    assert!(
        JS_API_CLIENT.contains("/wp-json/neopress/v1/templates/"),
        "saveLayout must target /wp-json/neopress/v1/templates/:id"
    );
    assert!(
        JS_API_CLIENT.contains("\"X-WP-Nonce\": \"wp_rest_admin\""),
        "saveLayout must include authenticating X-WP-Nonce"
    );
    assert!(
        JS_API_CLIENT.contains("async createTemplate("),
        "StudioApi must implement createTemplate"
    );
    assert!(
        JS_API_CLIENT.contains("async updateTemplate("),
        "StudioApi must implement updateTemplate"
    );
    assert!(
        JS_API_CLIENT.contains("async deleteTemplate("),
        "StudioApi must implement deleteTemplate"
    );
    assert!(
        JS_API_CLIENT.contains("async setConditions("),
        "StudioApi must implement setConditions"
    );
    assert!(
        JS_API_CLIENT.contains("async installStarter("),
        "StudioApi must implement installStarter"
    );
}

#[test]
fn test_adversarial_dom_leak_firewall_all_chrome_variants() {
    let contaminated_html = r#"
        <section class="np-section elementor-grid-handle neostudio-grid-handle neostudio-selection-outline neostudio-node neostudio-handle custom-preserved-class"
                 contenteditable="true"
                 draggable="true"
                 data-neostudio="section"
                 data-neostudio-node-id="sec_1"
                 data-neostudio-type="PriceCard"
                 data-drag-handle="true"
                 data-model-cid="cid_1"
                 data-elementor-type="header">
            <h2 CONTENTEDITABLE="TRUE" DRAGGABLE="TRUE" class="np-heading neostudio-element">Pristine Title</h2>
            <div contenteditable class="np-body">Body Text</div>
        </section>
    "#;

    let clean = sanitize_ssr_html(contaminated_html);

    // Verify legitimate content and classes are kept
    assert!(clean.contains("custom-preserved-class"));
    assert!(clean.contains("Pristine Title"));
    assert!(clean.contains("Body Text"));
    assert!(clean.contains("np-section"));
    assert!(clean.contains("np-heading"));

    // Verify 100% of builder chrome is stripped
    assert!(!clean.contains("contenteditable"), "contenteditable must be stripped");
    assert!(!clean.contains("CONTENTEDITABLE"), "CONTENTEDITABLE must be stripped");
    assert!(!clean.contains("draggable"), "draggable must be stripped");
    assert!(!clean.contains("DRAGGABLE"), "DRAGGABLE must be stripped");
    assert!(!clean.contains("elementor-grid-handle"), "elementor-grid-handle must be stripped");
    assert!(!clean.contains("neostudio-grid-handle"), "neostudio-grid-handle must be stripped");
    assert!(!clean.contains("neostudio-selection-outline"), "neostudio-selection-outline must be stripped");
    assert!(!clean.contains("neostudio-node"), "neostudio-node must be stripped");
    assert!(!clean.contains("neostudio-handle"), "neostudio-handle must be stripped");
    assert!(!clean.contains("data-neostudio"), "data-neostudio must be stripped");
    assert!(!clean.contains("data-drag-handle"), "data-drag-handle must be stripped");
    assert!(!clean.contains("data-model-cid"), "data-model-cid must be stripped");
    assert!(!clean.contains("data-elementor"), "data-elementor must be stripped");
    assert!(!clean.contains("class=\"\""), "Empty class attribute must be cleaned");
}

#[test]
fn test_adversarial_navigator_tree_reorder_and_cycle_prevention() {
    // 1. Verify if Navigator tree rows have drag-and-drop attributes or handlers
    let has_draggable = JS_NAVIGATOR.contains("draggable") || JS_NAVIGATOR.contains("dragstart");
    let has_reorder_fn = JS_NAVIGATOR.contains("moveNode") || JS_NAVIGATOR.contains("reorder");

    // 2. Check if cycle prevention is hooked up to tree reordering
    let has_cycle_prevention = JS_CORE_STATE.contains("isDescendant") && (
        JS_NAVIGATOR.contains("isDescendant") || JS_NAVIGATOR.contains("preventCycle")
    );

    println!(
        "Navigator Audit: has_draggable={}, has_reorder_fn={}, has_cycle_prevention={}",
        has_draggable, has_reorder_fn, has_cycle_prevention
    );

    // We empirically assert the requirement from Milestone 3 DISPATCH:
    // "2. Test Navigator tree reordering and cycle prevention.
    //  Verify Navigator drag-and-drop hierarchy handling (cycle prevention, parent/child relationships)."
    assert!(
        has_draggable,
        "Navigator tree items must support drag-and-drop interaction (draggable attribute or dragstart listener missing in JS_NAVIGATOR)"
    );
    assert!(
        has_reorder_fn,
        "Navigator must provide tree reordering logic (moveNode / reorder missing in JS_NAVIGATOR)"
    );
    assert!(
        has_cycle_prevention,
        "Navigator tree reordering must prevent cycles using isDescendant check"
    );
}

#[test]
fn test_adversarial_pro_blocks_renderers_zero_cube_placeholders() {
    // 1. Verify coordinator registers all 26 widget types and delegates to modular submodules
    let pro_types = [
        "PriceCard", "PriceTable", "CallToAction", "Testimonial", "TestimonialCarousel",
        "Countdown", "FlipBox", "AnimatedHeadline", "StarRating", "ProgressBar",
        "Alert", "Counter", "Benchmark", "SocialIcons", "ShareButtons",
        "NavMenu", "MegaMenuContainer", "PopupModal", "LoopGrid", "TableOfContents",
        "Breadcrumbs", "SearchForm", "LoginForm", "NativeForm", "Accordion", "Tabs"
    ];

    for pt in &pro_types {
        assert!(
            JS_CANVAS_PRO_WIDGETS.contains(pt),
            "JS_CANVAS_PRO_WIDGETS must register '{}'", pt
        );
    }

    assert!(
        JS_CANVAS_PRO_WIDGETS.contains("renderProCommercialWidget")
            && JS_CANVAS_PRO_WIDGETS.contains("renderProInteractiveWidget")
            && JS_CANVAS_PRO_WIDGETS.contains("renderProDynamicWidget"),
        "JS_CANVAS_PRO_WIDGETS must delegate to all 3 modular submodules"
    );

    // 2. Commercial blocks in JS_CANVAS_PRO_COMMERCIAL
    let commercial = [
        "PriceCard", "PriceTable", "CallToAction", "Countdown", "FlipBox",
        "AnimatedHeadline", "StarRating", "ProgressBar", "Alert", "Counter", "Benchmark"
    ];
    for b in &commercial {
        assert!(
            JS_CANVAS_PRO_COMMERCIAL.contains(&format!("case \"{}\":", b)),
            "JS_CANVAS_PRO_COMMERCIAL must implement case '{}'", b
        );
    }

    // 3. Interactive blocks in JS_CANVAS_PRO_INTERACTIVE
    let interactive = [
        "Testimonial", "TestimonialCarousel", "SocialIcons", "ShareButtons", "Accordion", "Tabs"
    ];
    for b in &interactive {
        assert!(
            JS_CANVAS_PRO_INTERACTIVE.contains(&format!("case \"{}\":", b)),
            "JS_CANVAS_PRO_INTERACTIVE must implement case '{}'", b
        );
    }

    // 4. Dynamic blocks in JS_CANVAS_PRO_DYNAMIC
    let dynamic = [
        "NavMenu", "LoopGrid", "NativeForm", "LoginForm", "SearchForm",
        "Breadcrumbs", "TableOfContents", "MegaMenuContainer", "PopupModal"
    ];
    for b in &dynamic {
        assert!(
            JS_CANVAS_PRO_DYNAMIC.contains(&format!("case \"{}\":", b)),
            "JS_CANVAS_PRO_DYNAMIC must implement case '{}'", b
        );
    }
}

#[test]
fn test_adversarial_theme_drawer_zero_blocking_dialogs() {
    // 1. Verify zero blocking browser dialogs in JS_THEME_DRAWER
    assert!(
        !JS_THEME_DRAWER.contains("prompt("),
        "JS_THEME_DRAWER must NOT contain browser-blocking prompt() calls"
    );
    assert!(
        !JS_THEME_DRAWER.contains("confirm("),
        "JS_THEME_DRAWER must NOT contain browser-blocking confirm() calls"
    );
    assert!(
        !JS_THEME_DRAWER.contains("alert("),
        "JS_THEME_DRAWER must NOT contain browser-blocking alert() calls"
    );

    // 2. Verify non-blocking CAD dialog handlers
    assert!(
        JS_THEME_DRAWER.contains("openCreateTemplateModal"),
        "JS_THEME_DRAWER must implement openCreateTemplateModal"
    );
    assert!(
        JS_THEME_DRAWER.contains("openDeleteConfirmModal"),
        "JS_THEME_DRAWER must implement openDeleteConfirmModal"
    );

    // 3. Verify markup contains the matching 2026 CAD modal overlays
    let modals_html = render_modals();
    assert!(
        modals_html.contains("id=\"modal-create-template\""),
        "render_modals must contain #modal-create-template"
    );
    assert!(
        modals_html.contains("id=\"modal-confirm-delete\""),
        "render_modals must contain #modal-confirm-delete"
    );
    assert!(
        modals_html.contains("id=\"btn-submit-create-template\""),
        "render_modals must contain #btn-submit-create-template"
    );
    assert!(
        modals_html.contains("id=\"btn-confirm-delete-action\""),
        "render_modals must contain #btn-confirm-delete-action"
    );
}

#[test]
fn test_adversarial_node_lock_enforcement_matrix() {
    // 1. Navigator drag start and target drop guards
    assert!(
        JS_NAVIGATOR.contains("node.data && node.data._locked"),
        "JS_NAVIGATOR must check node.data._locked"
    );
    assert!(
        JS_NAVIGATOR.contains("row.draggable = isDraggable"),
        "JS_NAVIGATOR must set row.draggable only when not locked"
    );

    // 2. Selection chrome guards (delete and duplicate)
    assert!(
        JS_CANVAS_SELECTION.contains("Cannot duplicate locked element"),
        "JS_CANVAS_SELECTION must guard duplicateNode against locked nodes"
    );
    assert!(
        JS_CANVAS_SELECTION.contains("Cannot delete locked element"),
        "JS_CANVAS_SELECTION must guard deleteNode against locked nodes"
    );

    // 3. Inspector style editing guards
    assert!(
        JS_INSPECTOR_CORE.contains("Cannot edit styles of locked element"),
        "JS_INSPECTOR_CORE must guard setLocalStyle against locked nodes"
    );

    // 4. Content settings lock notice and control disabling
    assert!(
        JS_INSPECTOR_CONTENT.contains("Locked element. Unlock in Navigator to edit."),
        "JS_INSPECTOR_CONTENT must render locked banner"
    );
    assert!(
        JS_INSPECTOR_CONTENT.contains("ctrl.disabled = true"),
        "JS_INSPECTOR_CONTENT must disable controls for locked nodes"
    );
}
