//! End-to-End Integration Test Suite for Daypress v0.6.0:
//! Custom Post Types, Structured Fields (ACF Parity) & Filterable Collection Grids.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
use daypress_server::{create_app, Database};
use serde_json::{json, Value};
use std::io::Cursor;
use tower::ServiceExt;

async fn body_to_string(res: axum::response::Response) -> String {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

async fn body_to_json(res: axum::response::Response) -> Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).expect("valid JSON response")
}

#[tokio::test]
async fn test_v060_all_seven_starter_blueprints_and_content_model_hub() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let app = create_app(db.clone());

    let presets = [
        ("case_studies", "case_study"),
        ("team", "team_member"),
        ("docs", "doc"),
        ("jobs", "job_listing"),
        ("events", "event"),
        ("portfolio", "portfolio"),
        ("testimonials", "testimonial"),
    ];

    for (preset_key, expected_slug) in presets {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/daypress/v1/content-types/preset")
                    .header("Content-Type", "application/json")
                    .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                    .body(Body::from(json!({ "preset": preset_key }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK, "Preset {preset_key} failed");
        let data = body_to_json(res).await;
        assert_eq!(data["post_type"]["slug"], expected_slug);
        assert!(data["field_group"]["fields"].as_array().unwrap().len() >= 3);
    }

    // Verify GET /wp-json/daypress/v1/content-types lists all 7 provisioned CPTs
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/content-types")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let cpts = body_to_json(res).await;
    assert_eq!(cpts.as_array().unwrap().len(), 7);

    // Verify GET /wp-json/wp/v2/types and /wp-json/wp/v2/taxonomies include our custom types & taxonomies
    let types_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/types")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let types_json = body_to_json(types_res).await;
    assert_eq!(types_json["case_study"]["name"], "Case Studies");
    assert_eq!(types_json["team_member"]["name"], "Team");

    let tax_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/taxonomies")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let tax_json = body_to_json(tax_res).await;
    assert_eq!(tax_json["case_industry"]["name"], "Industries");
    assert_eq!(tax_json["department"]["name"], "Departments");

    // Verify /d-admin/content-types SSR renders the Content Model Hub & dynamic sidebar menus
    let admin_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/d-admin/content-types")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_res.status(), StatusCode::OK);
    let admin_html = body_to_string(admin_res).await;
    assert!(admin_html.contains("Content Model"));
    assert!(admin_html.contains("Case Studies"));
    assert!(admin_html.contains("Case Study Specifications"));
}

#[tokio::test]
async fn test_v060_dynamic_cpt_rest_v2_crud_structured_fields_and_ssr_tokens() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let app = create_app(db.clone());

    // 1. Provision Case Studies preset (creates `case_study` CPT, `industry` taxonomy, `FieldGroupDefinition`, and Single/Archive templates)
    let preset_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/content-types/preset")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(json!({ "preset": "case_studies" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(preset_res.status(), StatusCode::OK);

    // 2. Create two terms in the `case_industry` custom taxonomy
    let term_fintech_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/taxonomies/case_industry/terms")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(
                    json!({ "name": "FinTech", "slug": "fintech" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(term_fintech_res.status(), StatusCode::CREATED);
    let fintech_term_id = body_to_json(term_fintech_res).await["id"].as_i64().unwrap();

    let term_saas_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/taxonomies/case_industry/terms")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(
                    json!({ "name": "Enterprise SaaS", "slug": "saas" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(term_saas_res.status(), StatusCode::CREATED);
    let saas_term_id = body_to_json(term_saas_res).await["id"].as_i64().unwrap();

    // 3. Create two `case_study` posts via Dynamic REST v2 route `/wp-json/wp/v2/case_study` with structured `acf` fields (including repeater!)
    let cs1_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/case_study")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(
                    json!({
                        "title": "Stripe Core Ledger Migration",
                        "slug": "stripe-ledger-migration",
                        "content": "<p>Migrated 14M daily transactions to native Rust.</p>",
                        "excerpt": "Sub-millisecond ledger throughput.",
                        "status": "publish",
                        "taxonomies": { "case_industry": [fintech_term_id] },
                        "acf": {
                            "client_name": "Stripe Financial",
                            "headline_metric": "99.999% Uptime / 0.3ms P99",
                            "project_url": "https://stripe.com",
                            "metrics": [
                                { "label": "Latency Reduction", "value": "94%" },
                                { "label": "Throughput", "value": "14M tx/day" }
                            ]
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cs1_res.status(), StatusCode::CREATED);
    let cs1_json = body_to_json(cs1_res).await;
    let cs1_id = cs1_json["id"].as_i64().unwrap();
    assert_eq!(cs1_json["type"], "case_study");
    assert_eq!(cs1_json["acf"]["client_name"], "Stripe Financial");
    assert_eq!(cs1_json["acf"]["metrics"][0]["label"], "Latency Reduction");

    let cs2_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/case_study")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(
                    json!({
                        "title": "Linear Workspace Sync Engine",
                        "slug": "linear-sync-engine",
                        "content": "<p>Zero-latency edge SQLite replication.</p>",
                        "excerpt": "Instant workspace state synchronization.",
                        "status": "publish",
                        "taxonomies": { "case_industry": [saas_term_id] },
                        "acf": {
                            "client_name": "Linear Corp",
                            "headline_metric": "42x Faster Cold Boot"
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cs2_res.status(), StatusCode::CREATED);

    // 4. Verify Single CPT SSR `/case_study/stripe-ledger-migration/` renders structured fields!
    let single_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/case_study/stripe-ledger-migration/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(single_res.status(), StatusCode::OK);
    let single_html = body_to_string(single_res).await;
    assert!(
        single_html.contains("Stripe Financial"),
        "Single CPT SSR must render structured field client_name"
    );
    assert!(
        single_html.contains("99.999% Uptime / 0.3ms P99"),
        "Single CPT SSR must render structured field headline_metric"
    );

    // 5. Verify Post Editor SSR (`/d-admin/post?post={cs1_id}&action=edit`) renders the structured field group values & repeater rows
    let editor_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/d-admin/post?post={cs1_id}&action=edit"))
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(editor_res.status(), StatusCode::OK);
    let editor_html = body_to_string(editor_res).await;
    assert!(editor_html.contains("Case Study Specifications"));
    assert!(editor_html.contains("Stripe Financial"));
    assert!(editor_html.contains("acf-repeater-container"));

    // 6. Verify Collection Grid Archive SSR `/case_study/` and taxonomy filtering `/case_study/?dp_tax=case_industry&dp_term=fintech`
    let archive_all_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/case_study/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(archive_all_res.status(), StatusCode::OK);
    let archive_all_html = body_to_string(archive_all_res).await;
    assert!(archive_all_html.contains("Stripe Core Ledger Migration"));
    assert!(archive_all_html.contains("Linear Workspace Sync Engine"));
    assert!(
        archive_all_html.contains("dp-collection-container"),
        "Must render v0.6 interactive Collection Grid wrapper"
    );
    assert!(
        archive_all_html.contains("dp-filter-pill"),
        "Must render taxonomy filter bar pills"
    );

    // Filtered by `?tax=case_industry&term=fintech`
    let archive_filtered_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/case_study/?tax=case_industry&term=fintech")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(archive_filtered_res.status(), StatusCode::OK);
    let archive_filtered_html = body_to_string(archive_filtered_res).await;
    assert!(archive_filtered_html.contains("Stripe Core Ledger Migration"));
    assert!(
        !archive_filtered_html.contains("Linear Workspace Sync Engine"),
        "Filtered collection grid must exclude non-matching terms"
    );

    // 7. Verify `/wp-json/daypress/v1/collection-query` JSON endpoint (used by instant client-side filter/sort/search)
    let cq_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/collection-query?post_type=case_study&taxonomy=case_industry&sort=title_asc&card_skin=boxed&meta_fields=client_name,headline_metric")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cq_res.status(), StatusCode::OK);
    let cq_json = body_to_json(cq_res).await;
    assert_eq!(cq_json["total"], 2);
    // Sorted by title_asc: Linear comes before Stripe
    assert_eq!(
        cq_json["posts"][0]["title"]["rendered"],
        "Linear Workspace Sync Engine"
    );
    assert!(cq_json["cards_html"]
        .as_str()
        .unwrap()
        .contains("dp-card-skin-boxed"));
    assert!(cq_json["cards_html"]
        .as_str()
        .unwrap()
        .contains("Stripe Financial"));

    if let Ok(dump_dir) = std::env::var("DUMP_V060_HTML_DIR") {
        let _ = std::fs::create_dir_all(&dump_dir);
        let cm_pt_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/d-admin/content-types")
                    .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let cm_pt_html = body_to_string(cm_pt_res).await;
        let _ = std::fs::write(format!("{dump_dir}/content_model_pt.html"), &cm_pt_html);

        let cm_fg_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/d-admin/content-types?tab=fields")
                    .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let cm_fg_html = body_to_string(cm_fg_res).await;
        let _ = std::fs::write(format!("{dump_dir}/content_model_fg.html"), &cm_fg_html);
        let _ = std::fs::write(format!("{dump_dir}/post_editor_cpt.html"), &editor_html);
        let _ = std::fs::write(
            format!("{dump_dir}/collection_archive.html"),
            &archive_all_html,
        );
        let _ = std::fs::write(format!("{dump_dir}/single_cpt.html"), &single_html);
    }
}

#[tokio::test]
async fn test_v060_canvas_page_collection_grid_ssr_hydration() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let app = create_app(db.clone());

    // Provision `team` preset & create a team member
    let _ = db
        .provision_content_type_preset("team")
        .expect("provision team preset");
    let _ = db
        .create_post(&daypress_server::models::CreatePostRequest {
            title: "Ada Lovelace".to_string(),
            slug: Some("ada-lovelace".to_string()),
            content: "<p>Principal Systems Architect.</p>".to_string(),
            status: "publish".to_string(),
            post_type: "team_member".to_string(),
            acf: Some(json!({
                "role_title": "Principal Systems Architect",
                "location": "London, UK"
            })),
            ..Default::default()
        })
        .expect("create team member");

    // Create a standard Page with a `LoopGrid` (CollectionGrid) node in its `layout_json`
    let page = db
        .create_post(&daypress_server::models::CreatePostRequest {
            title: "Our Engineering Team".to_string(),
            slug: Some("engineering-team".to_string()),
            content: "<p>Placeholder</p>".to_string(),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            ..Default::default()
        })
        .expect("create page");

    let layout = DaypressLayout {
        version: 1,
        root: DaypressNode {
            id: "root_team_page".to_string(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![DaypressNode {
                id: "team_collection_grid".to_string(),
                kind: NodeKind::LoopGrid {
                    post_type: "team_member".to_string(),
                    posts_per_page: 6,
                    columns: 3,
                    show_thumbnail: true,
                    show_excerpt: true,
                    show_meta: false,
                    show_pagination: true,
                    taxonomy: Some("department".to_string()),
                    enable_filter_bar: true,
                    enable_sort: true,
                    enable_search: true,
                    default_sort: "title_asc".to_string(),
                    card_skin: "boxed".to_string(),
                    meta_fields: vec!["role_title".to_string(), "location".to_string()],
                },
                styles: StyleProperties::default(),
                children: vec![],
            }],
        },
        global_styles: None,
    };
    let layout_str = serde_json::to_string(&layout).unwrap();
    db.save_post_layout(page.id, &layout_str, None)
        .expect("save page layout");

    // Request `/engineering-team/` and verify the LoopGrid inside the page's layout_json is hydrated on SSR!
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/engineering-team/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_to_string(res).await;
    assert!(html.contains("Ada Lovelace"));
    assert!(html.contains("Principal Systems Architect"));
    assert!(html.contains("London, UK"));
    assert!(html.contains("dp-card-skin-boxed"));
}

#[tokio::test]
async fn test_v060_wxr_and_bundle_acf_field_groups_and_repeater_ingestion() {
    let db = Database::new_in_memory().expect("in-memory DB");

    let wxr_with_acf = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>ACF Agency Migration</title>
	<description>ACF Field Groups &amp; Flattened Repeaters</description>
	<wp:wxr_version>1.2</wp:wxr_version>

	<!-- ACF Field Group post -->
	<item>
		<title>Project Specifications</title>
		<wp:post_id>500</wp:post_id>
		<wp:post_name>group_project_specs</wp:post_name>
		<wp:post_type>acf-field-group</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[{"location":[[{"param":"post_type","operator":"==","value":"project"}]]}]]></content:encoded>
	</item>

	<!-- ACF Child Fields -->
	<item>
		<title>Client Name</title>
		<wp:post_id>501</wp:post_id>
		<wp:post_parent>500</wp:post_parent>
		<wp:post_name>field_client_name</wp:post_name>
		<wp:post_excerpt>client_name</wp:post_excerpt>
		<wp:post_type>acf-field</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[{"type":"text","required":1}]]></content:encoded>
	</item>
	<item>
		<title>Deliverables</title>
		<wp:post_id>502</wp:post_id>
		<wp:post_parent>500</wp:post_parent>
		<wp:post_name>field_deliverables</wp:post_name>
		<wp:post_excerpt>deliverables</wp:post_excerpt>
		<wp:post_type>acf-field</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[{"type":"repeater"}]]></content:encoded>
	</item>
	<item>
		<title>Title</title>
		<wp:post_id>503</wp:post_id>
		<wp:post_parent>502</wp:post_parent>
		<wp:post_name>field_deliv_title</wp:post_name>
		<wp:post_excerpt>title</wp:post_excerpt>
		<wp:post_type>acf-field</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[{"type":"text"}]]></content:encoded>
	</item>

	<!-- Imported CPT Item with flattened ACF repeater rows -->
	<item>
		<title>Next-Gen Rust Compiler</title>
		<wp:post_id>600</wp:post_id>
		<wp:post_name>next-gen-rust-compiler</wp:post_name>
		<wp:post_type>project</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[<p>Compiler engineering case study.</p>]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>client_name</wp:meta_key>
			<wp:meta_value><![CDATA[Foundation Labs]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>deliverables</wp:meta_key>
			<wp:meta_value><![CDATA[2]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>deliverables_0_title</wp:meta_key>
			<wp:meta_value><![CDATA[AST Parser]]></wp:meta_value>
		</wp:postmeta>
		<wp:postmeta>
			<wp:meta_key>deliverables_1_title</wp:meta_key>
			<wp:meta_value><![CDATA[Zero-Copy SSR Engine]]></wp:meta_value>
		</wp:postmeta>
	</item>
</channel>
</rss>"#;

    let res = db
        .import_wxr(Cursor::new(wxr_with_acf))
        .expect("WXR import with ACF should succeed");
    assert_eq!(
        res.imported_posts, 1,
        "Schema posts must not pollute public posts count"
    );

    // 1. Verify FieldGroupDefinition was ingested into `daypress_field_groups` with nested repeater sub_fields
    let groups = db
        .list_field_groups(Some("project"))
        .expect("list field groups");
    assert!(!groups.is_empty(), "Project field group must be registered");
    let grp = groups
        .iter()
        .find(|g| g.title == "Project Specifications")
        .expect("Project Specifications group found");
    assert_eq!(grp.fields.len(), 2);
    let rep_field = grp
        .fields
        .iter()
        .find(|f| f.name == "deliverables")
        .expect("deliverables repeater field");
    assert_eq!(rep_field.field_type, "repeater");
    assert_eq!(rep_field.sub_fields.len(), 1);
    assert_eq!(rep_field.sub_fields[0].name, "title");

    // 2. Verify the imported `project` post automatically unpacked the flattened ACF repeater rows into `post.acf["deliverables"]`
    let project_post = db
        .get_post_by_slug("next-gen-rust-compiler")
        .expect("db query")
        .expect("project post exists");
    assert_eq!(project_post.post_type, "project");
    assert_eq!(project_post.acf["client_name"], "Foundation Labs");
    let delivs = project_post.acf["deliverables"]
        .as_array()
        .expect("unpacked repeater array");
    assert_eq!(delivs.len(), 2);
    assert_eq!(delivs[0]["title"], "AST Parser");
    assert_eq!(delivs[1]["title"], "Zero-Copy SSR Engine");
}

#[tokio::test]
async fn test_v060_mcp_tools_for_cpts_and_field_groups() {
    let db = Database::new_in_memory().expect("in-memory DB");
    let app = create_app(db.clone());

    // Call `daypress_provision_cpt_preset` via MCP JSON-RPC 2.0
    let mcp_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "daypress_provision_cpt_preset",
            "arguments": {
                "preset": "events"
            }
        }
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/mcp")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(mcp_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let rpc_json = body_to_json(res).await;
    assert_eq!(rpc_json["result"]["isError"], false);
    let inner_text = rpc_json["result"]["content"][0]["text"].as_str().unwrap();
    assert!(inner_text.contains("\"slug\": \"event\""));

    // Call `daypress_list_post_types` & `daypress_list_field_groups` via MCP
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "daypress_list_field_groups",
            "arguments": {
                "post_type": "event"
            }
        }
    });
    let list_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/mcp")
                .header("Content-Type", "application/json")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::from(list_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let list_json = body_to_json(list_res).await;
    assert_eq!(list_json["result"]["isError"], false);
    assert!(list_json["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Event Schedule & Venue"));
}
