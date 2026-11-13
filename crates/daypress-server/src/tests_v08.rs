use super::*;
use crate::adapters::mcp::{handle_mcp_rpc, JsonRpcRequest};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn test_v08_i18n_multilingual_routing_hreflang_sitemap_and_switcher() {
    let db = Database::new_in_memory().expect("in-memory db");

    // 1. Register German (de) and Dutch (nl) languages alongside default English (en)
    db.upsert_language(&DaypressLanguage {
        code: "de".into(),
        locale: "de_DE".into(),
        name: "Deutsch".into(),
        native_name: "Deutsch".into(),
        flag_emoji: "🇩🇪".into(),
        is_default: false,
        is_rtl: false,
        is_active: true,
        sort_order: 1,
    })
    .expect("upsert de");

    let langs = db.list_languages().expect("list langs");
    assert!(langs.iter().any(|l| l.code == "en" && l.is_default));
    assert!(langs.iter().any(|l| l.code == "de"));

    // 2. Create an English post with a LanguageSwitcher AST block and create its German translation draft
    let layout = DaypressLayout {
        version: 1,
        global_styles: None,
        root: DaypressNode {
            id: "root".into(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![
                DaypressNode {
                    id: "hero_h1".into(),
                    kind: NodeKind::Heading {
                        text: "Welcome to Daypress 0.8".into(),
                        level: 1,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "lang_sw".into(),
                    kind: NodeKind::LanguageSwitcher {
                        display_style: "pills".into(),
                        show_flags: true,
                        show_names: true,
                        show_codes: false,
                        current_lang: "en".into(),
                        links: vec![],
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
            ],
        },
    };
    let en_post = db
        .create_post(&CreatePostRequest {
            title: "Welcome Guide".into(),
            slug: Some("welcome-guide".into()),
            content: "<p>English guide content [daypress_language_switcher]</p>".into(),
            status: "publish".into(),
            post_type: "page".into(),
            lang_code: Some("en".into()),
            ..Default::default()
        })
        .expect("create en page");
    db.save_post_layout(
        en_post.id,
        &serde_json::to_string(&layout).unwrap(),
        Some("<p>English guide content [daypress_language_switcher]</p>"),
    )
    .expect("save en layout");

    let de_draft = db
        .create_translation_draft(en_post.id, "de")
        .expect("create de translation draft");
    assert_eq!(de_draft.lang_code, "de");
    assert_eq!(
        de_draft.translation_group_id,
        format!("trgrp_post_{}", en_post.id)
    );

    // Publish the German translation at slug `willkommen`
    let de_pub = db
        .update_post(
            de_draft.id,
            &UpdatePostRequest {
                title: Some("Willkommen Leitfaden".into()),
                slug: Some("willkommen".into()),
                content: Some("<p>Deutscher Inhalt [daypress_language_switcher]</p>".into()),
                status: Some("publish".into()),
                lang_code: Some("de".into()),
                ..Default::default()
            },
        )
        .expect("update de post")
        .expect("de post exists");
    assert_eq!(de_pub.link, "http://localhost:8090/de/willkommen/");

    // Verify same-slug translations across different languages do not collide (`contact` in en + `contact` in de)
    let en_contact = db
        .create_post(&CreatePostRequest {
            title: "Contact Us".into(),
            slug: Some("contact".into()),
            content: "<p>English contact</p>".into(),
            status: "publish".into(),
            post_type: "page".into(),
            lang_code: Some("en".into()),
            ..Default::default()
        })
        .expect("create en contact");
    assert_eq!(en_contact.slug, "contact");
    let de_contact = db
        .create_post(&CreatePostRequest {
            title: "Kontakt".into(),
            slug: Some("contact".into()),
            content: "<p>Deutscher Kontakt</p>".into(),
            status: "publish".into(),
            post_type: "page".into(),
            lang_code: Some("de".into()),
            translation_group_id: Some(en_contact.translation_group_id.clone()),
            ..Default::default()
        })
        .expect("create de contact with same slug");
    assert_eq!(
        de_contact.slug, "contact",
        "Same-slug translation in another language must not be suffixed with -2"
    );
    assert_eq!(de_contact.link, "http://localhost:8090/de/contact/");

    // 3. Request English page at /welcome-guide/ and verify hreflang tags + LanguageSwitcher SSR
    let app = create_app(db.clone());
    let res_en = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/welcome-guide/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_en.status(), StatusCode::OK);
    let html_en = String::from_utf8_lossy(
        &axum::body::to_bytes(res_en.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .to_string();
    assert!(
        html_en.contains(r#"hreflang="en" href="http://localhost:8090/welcome-guide/""#),
        "SSR head must include English hreflang"
    );
    assert!(
        html_en.contains(r#"hreflang="de" href="http://localhost:8090/de/willkommen/""#),
        "SSR head must include German hreflang"
    );
    assert!(
        html_en.contains(r#"hreflang="x-default""#),
        "SSR head must include x-default hreflang"
    );
    assert!(
        html_en.contains("dp-lang-switcher"),
        "LanguageSwitcher shortcode/widget must render in SSR"
    );

    // 4. Request German page at /de/willkommen/ and verify localized SSR, <html lang="de">, og:locale, and inLanguage JSON-LD
    let res_de = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/de/willkommen/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_de.status(), StatusCode::OK);
    let html_de = String::from_utf8_lossy(
        &axum::body::to_bytes(res_de.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .to_string();
    assert!(html_de.contains("Willkommen Leitfaden"));
    assert!(
        html_de.contains(r#"<html lang="de""#),
        "German SSR must emit <html lang=\"de\">"
    );
    assert!(
        html_de.contains(r#"property="og:locale" content="de_DE""#),
        "German SSR must emit og:locale content=de_DE"
    );
    assert!(
        html_de.contains(r#"property="og:locale:alternate" content="en_US""#),
        "German SSR must emit og:locale:alternate content=en_US"
    );
    assert!(
        html_de.contains(r#""inLanguage": "de""#) || html_de.contains(r#""inLanguage":"de""#),
        "Schema.org JSON-LD must include inLanguage: de"
    );

    // Verify /de/page/1/ paginated localized blog index returns 200 OK
    let res_de_page1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/de/page/1/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_de_page1.status(), StatusCode::OK);

    // Verify FTS5 search with ?lang=de filters by language
    let res_search_de = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/search?q=Willkommen&lang=de")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_search_de.status(), StatusCode::OK);
    let search_json: Value = serde_json::from_slice(
        &axum::body::to_bytes(res_search_de.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let results = search_json["results"]
        .as_array()
        .expect("search results array");
    assert!(
        results.iter().any(|r| r["lang_code"] == "de"),
        "FTS5 search with ?lang=de must return German post with lang_code='de'"
    );

    // 5. Request /sitemap.xml and verify xhtml:link hreflang alternates
    let res_sm = app
        .oneshot(
            Request::builder()
                .uri("/sitemap.xml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_sm.status(), StatusCode::OK);
    let xml = String::from_utf8_lossy(
        &axum::body::to_bytes(res_sm.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .to_string();
    assert!(xml.contains(r#"xmlns:xhtml="http://www.w3.org/1999/xhtml""#));
    assert!(xml.contains(
        r#"<xhtml:link rel="alternate" hreflang="de" href="http://localhost:8090/de/willkommen/"/>"#
    ));
}

#[tokio::test]
async fn test_v08_editorial_duplication_coauthors_preview_links_and_scheduled_revisions() {
    let db = Database::new_in_memory().expect("in-memory db");

    // 1. Create a second author user ("Alice") for Multi-Author Bylines
    let alice = db
        .create_user(&CreateUserRequest {
            username: "alice".into(),
            name: Some("Alice CoAuthor".into()),
            email: "alice@daypress.dev".into(),
            password: "SecretPass123!".into(),
            role: Some("author".into()),
            ..Default::default()
        })
        .expect("create alice");

    let mut meta_map = serde_json::Map::new();
    meta_map.insert("hero_subtitle".into(), json!("Sub-ms Rust CMS"));

    let post = db
        .create_post(&CreatePostRequest {
            title: "Engineering Deep Dive".into(),
            slug: Some("engineering-deep-dive".into()),
            content: "<p>Original live article body</p>".into(),
            status: "publish".into(),
            post_type: "post".into(),
            coauthors: Some(vec![1, alice.id]),
            lang_code: Some("en".into()),
            meta: Some(Value::Object(meta_map)),
            ..Default::default()
        })
        .expect("create post with coauthors");

    // Verify Multi-Author Byline ("admin & Alice CoAuthor")
    assert_eq!(post.coauthors.len(), 2);
    assert!(
        post.author_name.contains("Alice CoAuthor") && post.author_name.contains("&"),
        "Expected formatted multi-author byline, got: {}",
        post.author_name
    );

    // Verify secondary co-author appears on /author/alice/ archive AND in Schema.org JSON-LD
    let (_, alice_posts) = db
        .get_posts_by_author_slug("alice")
        .expect("alice author archive");
    assert!(
        alice_posts.iter().any(|p| p.id == post.id),
        "Secondary co-author archive must include co-authored post"
    );

    let app = create_app(db.clone());
    let res_live = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/engineering-deep-dive/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let live_html = String::from_utf8_lossy(
        &axum::body::to_bytes(res_live.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .to_string();
    assert!(
        live_html.contains("Alice CoAuthor") && live_html.contains("/author/alice/"),
        "Schema.org JSON-LD must include co-author Person array with author archive URL"
    );

    // 2. 1-Click Post Duplication preserves meta, co-authors, and language as a (Copy) draft
    let duplicated = db
        .duplicate_post(post.id)
        .expect("duplicate_post")
        .expect("duplicated post");
    assert_eq!(duplicated.status, "draft");
    assert!(duplicated.title.rendered.contains("(Copy)"));
    assert_eq!(duplicated.coauthors.len(), 2);
    assert_eq!(
        duplicated.acf.get("hero_subtitle").and_then(|v| v.as_str()),
        Some("Sub-ms Rust CMS")
    );

    // 3. Shareable Secret Draft Preview Link (no login required + X-Robots-Tag: noindex, nofollow)
    let token_rec = db
        .create_preview_token(duplicated.id, 48, Some("External Reviewer"), 1)
        .expect("create preview token");
    let raw_token = token_rec.raw_token.expect("raw_token returned on creation");

    let res_prev = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/preview/{}", raw_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_prev.status(), StatusCode::OK);
    assert_eq!(
        res_prev
            .headers()
            .get("x-robots-tag")
            .and_then(|v| v.to_str().ok()),
        Some("noindex, nofollow")
    );
    let prev_html = String::from_utf8_lossy(
        &axum::body::to_bytes(res_prev.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .to_string();
    assert!(prev_html.contains("Daypress Secret Draft Preview"));
    assert!(prev_html.contains("Engineering Deep Dive (Copy)"));

    // Revoke token -> subsequent request returns 403 Forbidden
    assert!(db
        .revoke_preview_token(duplicated.id, token_rec.id)
        .expect("revoke token"));
    let res_revoked = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/preview/{}", raw_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_revoked.status(), StatusCode::FORBIDDEN);

    // 4. Scheduled Revision to an Already-Published Post
    let sched_summary = db
        .schedule_published_post_revision(
            post.id,
            &ScheduleRevisionRequest {
                title: Some("Engineering Deep Dive v2.0 (Updated)".into()),
                content: Some("<p>Scheduled v2.0 content now live!</p>".into()),
                excerpt: Some("Updated excerpt".into()),
                layout: None,
                date: Some("2099-01-01 12:00:00".into()),
                date_gmt: Some("2099-01-01 12:00:00".into()),
            },
            Some(1),
        )
        .expect("schedule revision");
    assert_eq!(sched_summary.parent_post_id, post.id);

    // Live post remains untouched before due time
    let still_v1 = db.get_post_by_id(post.id).unwrap().unwrap();
    assert_eq!(still_v1.title.rendered, "Engineering Deep Dive");
    assert_eq!(
        still_v1.scheduled_revision_id,
        Some(sched_summary.revision_post_id)
    );

    // Move scheduled revision due time into the past and trigger lazy promotion
    let due_summary = db
        .schedule_published_post_revision(
            post.id,
            &ScheduleRevisionRequest {
                title: Some("Engineering Deep Dive v2.0 (Updated)".into()),
                content: Some("<p>Scheduled v2.0 content now live!</p>".into()),
                excerpt: Some("Updated excerpt".into()),
                layout: None,
                date: Some("2020-01-01 00:00:00".into()),
                date_gmt: Some("2020-01-01 00:00:00".into()),
            },
            Some(1),
        )
        .expect("reschedule in past");
    assert_eq!(due_summary.revision_post_id, sched_summary.revision_post_id);

    let promoted_count = db.publish_due_scheduled_posts().expect("promote due");
    assert!(promoted_count >= 1);

    let now_v2 = db.get_post_by_id(post.id).unwrap().unwrap();
    assert_eq!(
        now_v2.title.rendered,
        "Engineering Deep Dive v2.0 (Updated)"
    );
    assert!(now_v2
        .content
        .rendered
        .contains("Scheduled v2.0 content now live!"));
    assert_eq!(now_v2.scheduled_revision_id, None);
}

#[tokio::test]
async fn test_v08_ast_revision_diff_cherry_pick_and_mcp_tools() {
    let db = Database::new_in_memory().expect("in-memory db");

    // 1. Create a post with initial AST layout v1, then update to AST layout v2
    let layout_v1 = DaypressLayout {
        version: 1,
        global_styles: None,
        root: DaypressNode {
            id: "root".into(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![
                DaypressNode {
                    id: "hero_heading".into(),
                    kind: NodeKind::Heading {
                        text: "Original V1 Headline".into(),
                        level: 1,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "cta_btn".into(),
                    kind: NodeKind::Button {
                        text: "V1 CTA".into(),
                        url: "/v1".into(),
                        target: "_self".into(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
            ],
        },
    };
    let post = db
        .create_post(&CreatePostRequest {
            title: "Diff Test Page".into(),
            slug: Some("diff-test-page".into()),
            content: "<h1>Original V1 Headline</h1>".into(),
            status: "publish".into(),
            post_type: "page".into(),
            ..Default::default()
        })
        .expect("create page");
    db.save_post_layout(
        post.id,
        &serde_json::to_string(&layout_v1).unwrap(),
        Some("<h1>Original V1 Headline</h1>"),
    )
    .expect("save v1 layout");

    let layout_v2 = DaypressLayout {
        version: 1,
        global_styles: None,
        root: DaypressNode {
            id: "root".into(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![
                DaypressNode {
                    id: "hero_heading".into(),
                    kind: NodeKind::Heading {
                        text: "Modified V2 Headline".into(),
                        level: 1,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "cta_btn".into(),
                    kind: NodeKind::Button {
                        text: "V2 CTA".into(),
                        url: "/v2".into(),
                        target: "_self".into(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
            ],
        },
    };
    db.save_post_layout(
        post.id,
        &serde_json::to_string(&layout_v2).unwrap(),
        Some("<h1>Modified V2 Headline</h1>"),
    )
    .expect("save v2 layout");

    let revs = db.get_post_revisions(post.id).expect("get revisions");
    assert!(!revs.is_empty());
    // Find the revision containing "Original V1 Headline"
    let v1_rev_id = revs
        .iter()
        .find(|r| {
            db.diff_post_revisions(post.id, r.id, None)
                .map(|d| d.modified_count > 0)
                .unwrap_or(false)
        })
        .map(|r| r.id)
        .expect("find v1 revision");

    let diff_report = db
        .diff_post_revisions(post.id, v1_rev_id, None)
        .expect("diff revisions");
    assert!(diff_report.modified_count >= 1);
    assert!(diff_report
        .nodes
        .iter()
        .any(|n| n.node_id == "hero_heading"));

    // 2. Cherry-pick ONLY `hero_heading` from `v1_rev_id`, keeping `cta_btn` at V2
    db.cherry_pick_revision_node(post.id, v1_rev_id, "hero_heading")
        .expect("cherry pick hero_heading");
    let patched_json = db.get_post_layout(post.id).unwrap().unwrap();
    assert!(patched_json.contains("Original V1 Headline"));
    assert!(patched_json.contains("V2 CTA"));

    // 3. Verify v0.8 MCP Tools via JSON-RPC
    let db_arc = std::sync::Arc::new(db.clone());
    let cms = CmsService::new(
        db_arc.clone(),
        db_arc.clone(),
        db_arc.clone(),
        db_arc.clone(),
        db_arc,
    );
    let rpc_langs = handle_mcp_rpc(
        &cms,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(1)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_list_languages",
                "arguments": {}
            }),
        },
    );
    let langs_res = rpc_langs.result.expect("mcp languages result");
    let langs_text = langs_res["content"][0]["text"].as_str().unwrap_or("");
    assert!(langs_text.contains("\"default_language\""));

    let rpc_dup = handle_mcp_rpc(
        &cms,
        JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: json!({
                "name": "daypress_duplicate_post",
                "arguments": { "post_id": post.id }
            }),
        },
    );
    let dup_res = rpc_dup.result.expect("mcp duplicate result");
    let dup_text = dup_res["content"][0]["text"].as_str().unwrap_or("");
    assert!(dup_text.contains("(Copy)"));
}

#[tokio::test]
async fn test_v08_polylang_and_wpml_wxr_migration_end_to_end() {
    let db = Database::new_in_memory().expect("in-memory db");

    // Realistic Polylang + WPML WXR export with:
    // - Polylang `language` terms (`en`, `de` with locale de_DE, `ar` with rtl=1)
    // - Polylang `post_translations` serialized group mapping (`en` -> 501, `de` -> 502, `ar` -> 503)
    // - Same-slug pages across locales (`pricing` in EN, `pricing` in DE, `pricing` in AR)
    // - WPML `_wpml_import_translation_group` + `_wpml_language` postmeta on posts 601 & 602
    let wxr_xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
    xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
    xmlns:content="http://purl.org/rss/1.0/modules/content/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:wp="http://wordpress.org/export/1.2/">
<channel>
    <title>Global Polylang &amp; WPML Corp</title>
    <link>https://example.org</link>
    <description>Multilingual Enterprise Site</description>
    <wp:wxr_version>1.2</wp:wxr_version>
    <wp:term>
        <wp:term_id>10</wp:term_id>
        <wp:term_taxonomy>language</wp:term_taxonomy>
        <wp:term_slug>en</wp:term_slug>
        <wp:term_name><![CDATA[English]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:6:"locale";s:5:"en_US";s:3:"rtl";i:0;s:9:"flag_code";s:2:"us";}]]></wp:term_description>
    </wp:term>
    <wp:term>
        <wp:term_id>11</wp:term_id>
        <wp:term_taxonomy>language</wp:term_taxonomy>
        <wp:term_slug>de</wp:term_slug>
        <wp:term_name><![CDATA[Deutsch]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:6:"locale";s:5:"de_DE";s:3:"rtl";i:0;s:9:"flag_code";s:2:"de";}]]></wp:term_description>
    </wp:term>
    <wp:term>
        <wp:term_id>12</wp:term_id>
        <wp:term_taxonomy>language</wp:term_taxonomy>
        <wp:term_slug>ar</wp:term_slug>
        <wp:term_name><![CDATA[العربية]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:6:"locale";s:2:"ar";s:3:"rtl";i:1;s:9:"flag_code";s:2:"sa";}]]></wp:term_description>
    </wp:term>
    <wp:term>
        <wp:term_id>20</wp:term_id>
        <wp:term_taxonomy>post_translations</wp:term_taxonomy>
        <wp:term_slug>pll_pricing_group</wp:term_slug>
        <wp:term_name><![CDATA[pll_pricing_group]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:2:"en";i:501;s:2:"de";i:502;s:2:"ar";i:503;}]]></wp:term_description>
    </wp:term>
    <item>
        <title>Enterprise Pricing</title>
        <wp:post_id>501</wp:post_id>
        <wp:post_date>2026-05-01 10:00:00</wp:post_date>
        <wp:post_date_gmt>2026-05-01 10:00:00</wp:post_date_gmt>
        <wp:post_name>pricing</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>page</wp:post_type>
        <category domain="language" nicename="en"><![CDATA[English]]></category>
        <category domain="post_translations" nicename="pll_pricing_group"><![CDATA[pll_pricing_group]]></category>
        <content:encoded><![CDATA[<p>English pricing page [daypress_language_switcher]</p>]]></content:encoded>
    </item>
    <item>
        <title>Unternehmenspreise</title>
        <wp:post_id>502</wp:post_id>
        <wp:post_date>2026-05-01 10:05:00</wp:post_date>
        <wp:post_date_gmt>2026-05-01 10:05:00</wp:post_date_gmt>
        <wp:post_name>pricing</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>page</wp:post_type>
        <category domain="language" nicename="de"><![CDATA[Deutsch]]></category>
        <category domain="post_translations" nicename="pll_pricing_group"><![CDATA[pll_pricing_group]]></category>
        <content:encoded><![CDATA[<p>Deutsche Preise [daypress_language_switcher]</p>]]></content:encoded>
    </item>
    <item>
        <title>الأسعار للمؤسسات</title>
        <wp:post_id>503</wp:post_id>
        <wp:post_date>2026-05-01 10:10:00</wp:post_date>
        <wp:post_date_gmt>2026-05-01 10:10:00</wp:post_date_gmt>
        <wp:post_name>pricing</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>page</wp:post_type>
        <category domain="language" nicename="ar"><![CDATA[العربية]]></category>
        <category domain="post_translations" nicename="pll_pricing_group"><![CDATA[pll_pricing_group]]></category>
        <content:encoded><![CDATA[<p>صفحة الأسعار باللغة العربية [daypress_language_switcher]</p>]]></content:encoded>
    </item>
    <item>
        <title>WPML Release Notes</title>
        <wp:post_id>601</wp:post_id>
        <wp:post_date>2026-05-02 10:00:00</wp:post_date>
        <wp:post_date_gmt>2026-05-02 10:00:00</wp:post_date_gmt>
        <wp:post_name>release-notes</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>post</wp:post_type>
        <content:encoded><![CDATA[<p>English WPML post</p>]]></content:encoded>
        <wp:postmeta>
            <wp:meta_key>_wpml_import_translation_group</wp:meta_key>
            <wp:meta_value><![CDATA[wpml_trid_900]]></wp:meta_value>
        </wp:postmeta>
        <wp:postmeta>
            <wp:meta_key>_wpml_language</wp:meta_key>
            <wp:meta_value><![CDATA[en]]></wp:meta_value>
        </wp:postmeta>
    </item>
    <item>
        <title>WPML Versionshinweise</title>
        <wp:post_id>602</wp:post_id>
        <wp:post_date>2026-05-02 10:05:00</wp:post_date>
        <wp:post_date_gmt>2026-05-02 10:05:00</wp:post_date_gmt>
        <wp:post_name>versionshinweise</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>post</wp:post_type>
        <content:encoded><![CDATA[<p>Deutscher WPML Beitrag</p>]]></content:encoded>
        <wp:postmeta>
            <wp:meta_key>_wpml_import_translation_group</wp:meta_key>
            <wp:meta_value><![CDATA[wpml_trid_900]]></wp:meta_value>
        </wp:postmeta>
        <wp:postmeta>
            <wp:meta_key>_wpml_language</wp:meta_key>
            <wp:meta_value><![CDATA[de]]></wp:meta_value>
        </wp:postmeta>
    </item>
</channel>
</rss>"#;

    let stats = db
        .import_wxr(std::io::Cursor::new(wxr_xml.as_bytes()))
        .expect("import polylang+wpml wxr");
    assert!(stats.imported_posts + stats.imported_pages >= 5);

    // Verify languages (`en`, `de` with locale de_DE, `ar` with is_rtl = true)
    let langs = db.list_languages().expect("list languages");
    let de_lang = langs
        .iter()
        .find(|l| l.code == "de")
        .expect("de language imported");
    assert_eq!(de_lang.locale, "de_DE");
    let ar_lang = langs
        .iter()
        .find(|l| l.code == "ar")
        .expect("ar language imported");
    assert!(
        ar_lang.is_rtl,
        "Arabic must be marked RTL from Polylang serialized description"
    );

    // Verify same-slug pages (`pricing` in EN, DE, and AR) all preserved slug `pricing` and linked together
    let en_pricing = db
        .get_post_by_lang_and_slug("en", "pricing")
        .expect("query en pricing")
        .expect("en pricing exists");
    let de_pricing = db
        .get_post_by_lang_and_slug("de", "pricing")
        .expect("query de pricing")
        .expect("de pricing exists");
    let ar_pricing = db
        .get_post_by_lang_and_slug("ar", "pricing")
        .expect("query ar pricing")
        .expect("ar pricing exists");
    assert_eq!(en_pricing.title.rendered, "Enterprise Pricing");
    assert_eq!(de_pricing.title.rendered, "Unternehmenspreise");
    assert_eq!(ar_pricing.title.rendered, "الأسعار للمؤسسات");
    assert_eq!(en_pricing.translations.get("de"), Some(&de_pricing.id));
    assert_eq!(en_pricing.translations.get("ar"), Some(&ar_pricing.id));

    // Verify WPML postmeta translation group linked posts 601 & 602
    let wpml_de = db
        .get_post_by_lang_and_slug("de", "versionshinweise")
        .expect("query wpml de")
        .expect("wpml de exists");
    assert!(
        wpml_de.translations.contains_key("en"),
        "WPML _wpml_import_translation_group must link EN and DE posts"
    );

    // Verify SSR on /ar/pricing/ renders <html lang="ar" dir="rtl"> and hreflang cluster
    let app = create_app(db.clone());
    let res_ar = app
        .oneshot(
            Request::builder()
                .uri("/ar/pricing/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_ar.status(), StatusCode::OK);
    let html_ar = String::from_utf8_lossy(
        &axum::body::to_bytes(res_ar.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .to_string();
    assert!(
        html_ar.contains(r#"<html lang="ar" dir="rtl""#),
        "Arabic SSR must emit <html lang=\"ar\" dir=\"rtl\">"
    );
    assert!(
        html_ar.contains(r#"hreflang="de" href="http://localhost:8090/de/pricing/""#),
        "Arabic SSR must include German hreflang alternate"
    );
}

#[tokio::test]
async fn test_v0_8_5_wp_admin_qol_and_editor_ergonomics() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let db = Database::new_in_memory().expect("in-memory db");

    // 1. User Screen Preferences persistence & merge
    let initial_prefs = db.get_user_screen_preferences(1).expect("get prefs");
    assert!(initial_prefs.is_object());

    let saved_prefs = db
        .save_user_screen_preferences(
            1,
            &serde_json::json!({
                "preferences": {
                    "sidebar_collapsed": true,
                    "media_view_mode": "list",
                    "editor_post": {
                        "columns": 1,
                        "hidden": { "metabox-custom-fields": true }
                    }
                }
            }),
        )
        .expect("save prefs");
    assert_eq!(saved_prefs["sidebar_collapsed"], serde_json::json!(true));
    assert_eq!(saved_prefs["media_view_mode"], serde_json::json!("list"));
    assert_eq!(saved_prefs["editor_post"]["columns"], serde_json::json!(1));

    // 2. Sticky, Password, Comment Status & Ping Status persistence
    let created = db
        .create_post(&CreatePostRequest {
            title: "Sticky Protected Post".to_string(),
            content: "<p>Hello QoL</p>".to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            sticky: Some(true),
            password: Some("secret123".to_string()),
            comment_status: Some("closed".to_string()),
            ping_status: Some("closed".to_string()),
            ..Default::default()
        })
        .expect("create sticky post");
    assert!(created.sticky, "Created post must be sticky");
    assert_eq!(created.password, "secret123");
    assert_eq!(created.comment_status, "closed");
    assert_eq!(created.ping_status, "closed");

    let updated = db
        .update_post(
            created.id,
            &UpdatePostRequest {
                sticky: Some(false),
                password: Some(String::new()),
                comment_status: Some("open".to_string()),
                ping_status: Some("open".to_string()),
                ..Default::default()
            },
        )
        .expect("update sticky post")
        .expect("post exists");
    assert!(!updated.sticky, "Post sticky should be cleared");
    assert_eq!(updated.password, "");
    assert_eq!(updated.comment_status, "open");
    assert_eq!(updated.ping_status, "open");

    // 3. Empty Trash for post_type
    let trashed = db
        .create_post(&CreatePostRequest {
            title: "To Be Purged".to_string(),
            content: "<p>Trash</p>".to_string(),
            status: "trash".to_string(),
            post_type: "post".to_string(),
            ..Default::default()
        })
        .expect("create trashed post");
    let purged_count = db
        .empty_trash_posts(Some("post"))
        .expect("empty trash posts");
    assert!(purged_count >= 1);
    assert!(
        db.get_post_by_id(trashed.id).expect("query").is_none(),
        "Trashed post must be permanently deleted"
    );

    // 4. REST user-preferences endpoint (must reject unauthenticated access and serve authenticated users)
    let app = create_app(db.clone());
    let unauth_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/user-preferences")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth_res.status(), StatusCode::UNAUTHORIZED);

    let res = app
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/user-preferences")
                .header("Authorization", "Basic YWRtaW46YWRtaW4=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
