use daypress_ast::*;
use std::collections::HashMap;

#[test]
fn test_gutenberg_transpilation_and_rendering() {
    let gb_content = r##"
<!-- wp:cover {"url":"https://example.com/hero.jpg","customOverlayColor":"#0f172a"} -->
<div class="wp-block-cover">
    <!-- wp:heading {"level":1,"textAlign":"center"} -->
    <h1 class="wp-block-heading has-text-align-center">Gutenberg to Native Rust</h1>
    <!-- /wp:heading -->
    <!-- wp:paragraph -->
    <p>Zero-overhead block transpilation.</p>
    <!-- /wp:paragraph -->
    <!-- wp:buttons -->
    <div class="wp-block-buttons">
        <!-- wp:button -->
        <div class="wp-block-button"><a class="wp-block-button__link" href="/start">Start Now</a></div>
        <!-- /wp:button -->
    </div>
    <!-- /wp:buttons -->
</div>
<!-- /wp:cover -->
<!-- wp:spacer {"height":"60px"} /-->
    "##;

    let meta = HashMap::new();
    let layout = transpile_content(gb_content, &meta).expect("Gutenberg transpilation failed");
    assert_eq!(layout.root.children.len(), 2);

    let cover = &layout.root.children[0];
    assert_eq!(cover.kind, NodeKind::Section);
    assert_eq!(cover.children.len(), 3);

    let html = render_layout_to_html(&layout);
    assert!(html.contains("Gutenberg to Native Rust</h1>"));
    assert!(html.contains("Zero-overhead block transpilation."));
    assert!(html.contains("<a href=\"/start\" target=\"_self\" class=\"dp-button\""));
    assert!(html.contains("height: 60px;"));
}

#[test]
fn test_visual_builder_and_seed_schema_unification() {
    let vb_json = r##"{
        "version": 1,
        "root": {
            "id": "root",
            "type": "Root",
            "children": [
                {
                    "id": "sec_1",
                    "type": "Section",
                    "data": {},
                    "styles": {
                        "background_color": "#0f172a",
                        "padding_top": "80px",
                        "border": "1px solid #e2e8f0",
                        "margin": "0 auto"
                    },
                    "children": [
                        {
                            "id": "col_1",
                            "type": "Column",
                            "styles": { "flex": "1" },
                            "children": [
                                {
                                    "id": "cnt_1",
                                    "type": "Counter",
                                    "data": { "number": "100%", "label": "Lighthouse Performance" },
                                    "styles": {}
                                },
                                {
                                    "id": "sp_1",
                                    "type": "Spacer",
                                    "styles": { "height": "40px" }
                                },
                                {
                                    "id": "frm_1",
                                    "type": "NativeForm",
                                    "data": { "title": "Contact Us", "button_text": "Submit" }
                                },
                                {
                                    "id": "bm_1",
                                    "type": "Benchmark",
                                    "data": { "text": "95,000 req/sec", "subtitle": "Tokio Engine" }
                                },
                                {
                                    "id": "ic_1",
                                    "type": "Icon",
                                    "data": { "icon": "fa-solid fa-rocket" }
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    }"##;

    let layout: DaypressLayout =
        serde_json::from_str(vb_json).expect("Visual Builder JSON should deserialize cleanly");
    assert_eq!(layout.root.kind, NodeKind::Root);
    let sec = &layout.root.children[0];
    assert_eq!(sec.kind, NodeKind::Section);
    let col = &sec.children[0];
    assert_eq!(col.kind, NodeKind::Column);

    match &col.children[0].kind {
        NodeKind::Counter {
            number,
            suffix,
            title,
            ..
        } => {
            assert_eq!(*number, 100);
            assert_eq!(suffix.as_deref(), Some("%"));
            assert_eq!(title.as_deref(), Some("Lighthouse Performance"));
        }
        _ => panic!("Expected Counter"),
    }

    match &col.children[1].kind {
        NodeKind::Spacer { height } => assert_eq!(*height, 40),
        _ => panic!("Expected Spacer with 40px height"),
    }

    let html = render_layout_to_html(&layout);
    assert!(html.contains("class=\"dp-column\""));
    assert!(html.contains("100%"));
    assert!(html.contains("Lighthouse Performance"));
    assert!(html.contains("class=\"dp-form\""));
    assert!(html.contains("95,000 req/sec"));
    assert!(html.contains("<i class=\"fa-solid fa-rocket\""));
}
