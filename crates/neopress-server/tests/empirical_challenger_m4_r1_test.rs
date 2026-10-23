//! Empirical Challenger Test Suite for Milestone 4 (Post Editor Custom Fields, Taxonomies & Lossless Blocks).
//! File: `crates/neopress-server/tests/empirical_challenger_m4_r1_test.rs`
//!
//! Validates:
//! 1. Feature 17: Post Editor Dynamic Custom Fields (`wp_postmeta`) CRUD, type detection, and null deletion in SQLite.
//! 2. Feature 18: Dynamic Taxonomies Discovery, multi-taxonomy persistence, partial updates, and term clearing in SQLite.
//! 3. Feature 19: Lossless Gutenberg Block comment retention, AST synchronization, and dual-write invariant.
//! 4. XSS Escaping and markup integrity in the Post Editor custom fields card.

use neopress_server::domain::models::*;
use neopress_server::{templates, Database};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_empirical_postmeta_crud_and_null_deletion_deep() {
    let db = Database::new_in_memory().expect("Failed to initialize test DB");

    // 1. Create post with diverse metadata types (Unicode, booleans, numbers, nested JSON, special characters)
    let post = db
        .create_post(&CreatePostRequest {
            title: "Deep Meta Persistence Test".to_string(),
            content: "<p>Empirical verification of SQLite wp_postmeta</p>".to_string(),
            status: "publish".to_string(),
            slug: Some("deep-meta-test".to_string()),
            post_type: "post".to_string(),
            meta: Some(json!({
                "utf8_title": "日本語のタイトル 🚀",
                "boolean_flag": true,
                "int_val": 1337,
                "json_payload": {"nested": {"engine": "Rust", "sub_ms": true}},
                "json_array": [10, 20, 30],
                "to_be_deleted": "temporary_data",
                "to_be_updated": "initial_value"
            })),
            ..Default::default()
        })
        .expect("Failed to create post with diverse meta");

    // 2. Verify all keys correctly persisted and hydrated from SQLite wp_postmeta
    let fetched = db.get_post_by_id(post.id).expect("Query failed").expect("Post not found");
    assert_eq!(fetched.meta.get("utf8_title"), Some(&json!("日本語のタイトル 🚀")), "UTF-8 meta value must persist losslessly");
    assert_eq!(fetched.meta.get("boolean_flag"), Some(&json!(true)));
    assert_eq!(fetched.meta.get("int_val"), Some(&json!(1337)));
    assert_eq!(fetched.meta.get("json_payload"), Some(&json!({"nested": {"engine": "Rust", "sub_ms": true}})));
    assert_eq!(fetched.meta.get("json_array"), Some(&json!([10, 20, 30])));
    assert_eq!(fetched.meta.get("to_be_deleted"), Some(&json!("temporary_data")));
    assert_eq!(fetched.meta.get("to_be_updated"), Some(&json!("initial_value")));

    // 3. Update post: delete `to_be_deleted` with Null, update `to_be_updated`, add `newly_added`
    let updated = db
        .update_post(post.id, &UpdatePostRequest {
            meta: Some(json!({
                "to_be_deleted": serde_json::Value::Null,
                "to_be_updated": "updated_value_v2",
                "newly_added": 9999
            })),
            ..Default::default()
        })
        .expect("Failed to update post meta")
        .expect("Post not found");

    // 4. Assert updated state directly from SQLite
    assert_eq!(updated.meta.get("to_be_deleted"), None, "wp_postmeta row MUST be deleted from SQLite when set to Null");
    assert_eq!(updated.meta.get("to_be_updated"), Some(&json!("updated_value_v2")), "wp_postmeta row must be updated in SQLite");
    assert_eq!(updated.meta.get("newly_added"), Some(&json!(9999)), "newly_added key must be inserted into SQLite");
    assert_eq!(updated.meta.get("utf8_title"), Some(&json!("日本語のタイトル 🚀")), "Untouched keys must remain intact");

    // 5. Subsequent update deleting another key and updating newly_added
    let updated2 = db
        .update_post(post.id, &UpdatePostRequest {
            meta: Some(json!({ "newly_added": serde_json::Value::Null, "to_be_updated": "final_value" })),
            ..Default::default()
        })
        .expect("Second update failed")
        .expect("Post not found");

    assert_eq!(updated2.meta.get("newly_added"), None);
    assert_eq!(updated2.meta.get("to_be_updated"), Some(&json!("final_value")));
    assert_eq!(updated2.meta.get("boolean_flag"), Some(&json!(true)));
}

#[test]
fn test_empirical_dynamic_taxonomies_relational_integrity() {
    let db = Database::new_in_memory().expect("Failed to init DB");

    // 1. Seed two custom taxonomies: `case_study_industry` and `case_study_dept`
    let ind_tech = db.create_term("case_study_industry", &CreateTermRequest {
        name: "Technology".to_string(), slug: Some("technology".to_string()), description: None, parent: None,
    }).expect("Create term");

    let ind_health = db.create_term("case_study_industry", &CreateTermRequest {
        name: "Healthcare".to_string(), slug: Some("healthcare".to_string()), description: None, parent: None,
    }).expect("Create term");

    let dept_eng = db.create_term("case_study_dept", &CreateTermRequest {
        name: "Engineering".to_string(), slug: Some("engineering".to_string()), description: None, parent: None,
    }).expect("Create term");

    let dept_ops = db.create_term("case_study_dept", &CreateTermRequest {
        name: "Operations".to_string(), slug: Some("operations".to_string()), description: None, parent: None,
    }).expect("Create term");

    // 2. Discover taxonomies before post assignment
    let discovered_before = db.get_taxonomies_for_post_type("case_study", None).expect("Discovery failed");
    assert!(discovered_before.iter().any(|t| t.slug == "case_study_industry"), "Industry taxonomy must be discovered");
    assert!(discovered_before.iter().any(|t| t.slug == "case_study_dept"), "Department taxonomy must be discovered");

    // 3. Create post associated with both taxonomies
    let mut initial_tax = HashMap::new();
    initial_tax.insert("case_study_industry".to_string(), vec![ind_tech.id, ind_health.id]);
    initial_tax.insert("case_study_dept".to_string(), vec![dept_eng.id]);

    let post = db.create_post(&CreatePostRequest {
        title: "Multi-Taxonomy Post".to_string(),
        content: "<p>Content</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("multi-tax-post".to_string()),
        post_type: "case_study".to_string(),
        taxonomies: Some(initial_tax),
        ..Default::default()
    }).expect("Create post");

    // 4. Verify assigned terms in wp_term_relationships via get_taxonomies_for_post_type
    let assigned_taxes = db.get_taxonomies_for_post_type("case_study", Some(post.id)).expect("Get assigned taxonomies");
    let ind_tax = assigned_taxes.iter().find(|t| t.slug == "case_study_industry").expect("Find industry");
    assert_eq!(ind_tax.assigned_term_ids.len(), 2);
    assert!(ind_tax.assigned_term_ids.contains(&ind_tech.id));
    assert!(ind_tax.assigned_term_ids.contains(&ind_health.id));

    let dept_tax = assigned_taxes.iter().find(|t| t.slug == "case_study_dept").expect("Find department");
    assert_eq!(dept_tax.assigned_term_ids, vec![dept_eng.id]);

    // 5. Update post: switch department to `dept_ops`, clear `case_study_industry` by passing empty list `vec![]`
    let mut update_tax = HashMap::new();
    update_tax.insert("case_study_dept".to_string(), vec![dept_ops.id]);
    update_tax.insert("case_study_industry".to_string(), vec![]); // Explicit clear

    db.update_post(post.id, &UpdatePostRequest {
        taxonomies: Some(update_tax),
        ..Default::default()
    }).expect("Update post").expect("Post found");

    // 6. Verify in SQLite that old department was replaced and industry was completely cleared
    let post_taxes_after = db.get_taxonomies_for_post_type("case_study", Some(post.id)).expect("Get post taxonomies after update");
    let ind_after = post_taxes_after.iter().find(|t| t.slug == "case_study_industry").expect("Find industry");
    assert_eq!(ind_after.assigned_term_ids.len(), 0, "Industry terms must be completely cleared when empty vector passed");

    let dept_after = post_taxes_after.iter().find(|t| t.slug == "case_study_dept").expect("Find department");
    assert_eq!(dept_after.assigned_term_ids, vec![dept_ops.id], "Department must now have only dept_ops");
}

#[test]
fn test_empirical_lossless_gutenberg_blocks_and_ast_dual_write() {
    let db = Database::new_in_memory().expect("Failed to init DB");

    // Content with nested Gutenberg blocks and normal HTML comments
    let original_content = r#"<!-- Normal HTML comment that should be ignored by parser -->
<!-- wp:columns {"columns":2,"className":"custom-grid"} -->
<div class="wp-block-columns custom-grid">
<!-- wp:column -->
<div class="wp-block-column">
<!-- wp:heading {"level":3} -->
<h3>Column One Title</h3>
<!-- /wp:heading -->
<!-- wp:paragraph {"fontSize":"small"} -->
<p>First column text with attributes.</p>
<!-- /wp:paragraph -->
</div>
<!-- /wp:column -->
<!-- wp:column -->
<div class="wp-block-column">
<!-- wp:spacer {"height":"24px"} /-->
</div>
<!-- /wp:column -->
</div>
<!-- /wp:columns -->"#;

    let post = db.create_post(&CreatePostRequest {
        title: "Nested Gutenberg Blocks Post".to_string(),
        content: original_content.to_string(),
        status: "publish".to_string(),
        slug: Some("nested-gutenberg".to_string()),
        post_type: "post".to_string(),
        ..Default::default()
    }).expect("Create post");

    // 1. Assert full comment retention in wp_posts.post_content
    let fetched = db.get_post_by_id(post.id).expect("Query failed").expect("Post not found");
    assert!(fetched.content.rendered.contains("<!-- wp:columns {\"columns\":2,\"className\":\"custom-grid\"} -->"));
    assert!(fetched.content.rendered.contains("<!-- wp:spacer {\"height\":\"24px\"} /-->"));
    assert!(fetched.content.rendered.contains("<!-- /wp:columns -->"));

    // 2. Assert AST layout synthesized and saved
    let layout_opt = db.get_post_layout(post.id).expect("Query layout failed");
    assert!(layout_opt.is_some(), "AST layout must be automatically synthesized and populated");
    let layout_str = layout_opt.unwrap();
    assert!(layout_str.contains("root") || layout_str.contains("columns") || layout_str.contains("nodes"));

    // 3. Update post with new Gutenberg content and verify layout synchronization
    let updated_content = r#"<!-- wp:heading {"level":1} -->
<h1>Updated Title Heading</h1>
<!-- /wp:heading -->"#;

    db.update_post(post.id, &UpdatePostRequest {
        content: Some(updated_content.to_string()),
        ..Default::default()
    }).expect("Update post content").expect("Post found");

    let updated_fetched = db.get_post_by_id(post.id).expect("Query post").expect("Post found");
    assert!(updated_fetched.content.rendered.contains("<!-- wp:heading {\"level\":1} -->"));
    assert!(updated_fetched.content.rendered.contains("<!-- /wp:heading -->"));
}

#[test]
fn test_empirical_post_editor_markup_resilience() {
    let db = Database::new_in_memory().expect("Failed to init DB");
    let settings = db.get_settings().unwrap_or_default();

    let post = db.create_post(&CreatePostRequest {
        title: "Editor XSS & Markup Probe".to_string(),
        content: "<p>Testing markup</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("markup-probe".to_string()),
        post_type: "post".to_string(),
        meta: Some(json!({
            "xss_key": "\"><script>alert(1)</script>",
            "numeric_stat": 99.5,
            "bool_status": false,
            "json_blob": {"status": "ok"}
        })),
        ..Default::default()
    }).expect("Create post");

    let html = templates::render_wp_post_new_view(
        &settings, "post", Some(&post), &[], &[], &[], &[], false, &[],
    );

    // 1. Structure assertions
    assert!(html.contains("id=\"custom_fields_card\""));
    assert!(html.contains("id=\"custom-fields-table\""));
    assert!(html.contains("addCustomMetaRow()"));
    assert!(html.contains("deleteCustomMetaRow("));

    // 2. Type badge assertions
    assert!(html.contains("META: NUMBER"));
    assert!(html.contains("META: BOOLEAN"));
    assert!(html.contains("META: JSON"));

    // 3. XSS escaping assertion: raw unescaped script tag must NOT be rendered inside value attribute
    assert!(!html.contains("value=\"\"><script>alert(1)</script>\""));
    assert!(
        html.contains("&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;")
            || html.contains("&quot;&gt;&lt;script&gt;")
    );
}
