//! Empirical Adversarial Challenger Tests for Theme Builder SSR Pipeline.
//!
//! Stress-tests dynamic SSR placeholder hydration, dynamic tokens, unicode/edge cases,
//! zero runtime panics, and the 3-Tier SSR Leak Firewall.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_ast::{
    render_layout_to_html, DaypressLayout, DaypressNode, NodeKind, StyleProperties,
};
use daypress_server::theme_builder::models::*;
use daypress_server::theme_builder::ssr_injector::{HydrationContext, SsrInjector};
use daypress_server::theme_builder::ThemeTemplateRepository;
use daypress_server::{create_app, Database};
use std::collections::HashMap;
use tower::ServiceExt;

#[test]
fn test_adversarial_placeholder_hydration_unicode_and_special_chars() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let mut settings = db.get_settings().unwrap_or_default();
    settings.title = "Native <Rust> & \"CMS\" 🚀".to_string();
    settings.description = "Zero-Panic & Fast — 🦀".to_string();
    settings.site_logo_url = "https://example.com/logo.svg?v=1&k=2".to_string();

    let mut post = db.get_post_by_id(1).unwrap().expect("Seeded post missing");
    post.title.rendered = "<script>alert('xss')</script> 🦀 🚀 𠜎 𠜱 & <special> \"quotes\" 'single' \u{200B}\u{202E}RTL\u{202C}".to_string();
    post.content.rendered = "<p>Post content with <b>rich tags</b> & entities</p>".to_string();
    post.author_name = "O'Reilly & Sons <Developer>".to_string();
    post.date = "２０２６年１０月２３日T12:00:00Z".to_string(); // non-ASCII unicode date
    post.featured_image_url = None; // Missing featured image

    let mut layout = DaypressLayout {
        version: 1,
        root: DaypressNode {
            id: "root-adversarial".to_string(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![
                DaypressNode {
                    id: "dynamic-post-title".to_string(),
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Fallback Title".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "dynamic-featured-image".to_string(),
                    kind: NodeKind::Image {
                        src: "orig.jpg".to_string(),
                        alt: "orig".to_string(),
                        width: None,
                        height: None,
                        link: None,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "dynamic-post-content".to_string(),
                    kind: NodeKind::CustomHtml {
                        raw: "orig content".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "dynamic-post-date".to_string(),
                    kind: NodeKind::Text {
                        content_html: "orig date".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "dynamic-author-box".to_string(),
                    kind: NodeKind::CustomHtml {
                        raw: "orig author".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "dynamic-site-logo".to_string(),
                    kind: NodeKind::Image {
                        src: "orig_logo.png".to_string(),
                        alt: "".to_string(),
                        width: None,
                        height: None,
                        link: None,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "dynamic-site-title".to_string(),
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "orig site title".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
            ],
        },
        global_styles: None,
    };

    let hyd_ctx = HydrationContext {
        post: Some(&post),
        settings: &settings,
        breadcrumbs: vec![
            BreadcrumbItem {
                title: "Home & Start".to_string(),
                url: "/".to_string(),
                is_current: false,
            },
            BreadcrumbItem {
                title: post.title.rendered.clone(),
                url: "/post-url/".to_string(),
                is_current: true,
            },
        ],
        nav_menus: HashMap::new(),
    };

    // Assert zero panics on hydration
    SsrInjector::hydrate_layout(&mut layout, &hyd_ctx);

    // Assert zero panics on rendering to HTML
    let rendered = render_layout_to_html(&layout);

    // Verify fallback placeholder when featured image is missing
    assert!(
        rendered.contains("/static/placeholder.svg"),
        "Must fallback to placeholder image"
    );

    // Verify unicode astral plane characters survived without byte index corruptions
    assert!(rendered.contains("🦀"), "Must preserve emoji characters");
    assert!(
        rendered.contains("𠜎"),
        "Must preserve astral plane unicode"
    );

    // Verify author box properly escaped O'Reilly & Sons
    assert!(
        rendered.contains("O&#x27;Reilly &amp; Sons &lt;Developer&gt;"),
        "Author name must be safely escaped"
    );

    // Verify site logo used settings site_logo_url
    assert!(rendered.contains("https://example.com/logo.svg?v=1&amp;k=2"));

    // Verify site title hydrated
    assert!(rendered.contains("Native <Rust> & \"CMS\" 🚀"));
}

#[test]
fn test_adversarial_placeholder_hydration_with_none_post_and_empty_title() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let mut settings = db.get_settings().unwrap_or_default();
    settings.title = "Daypress Global".to_string();
    settings.description = "Minimalist".to_string();
    settings.site_logo_url = "".to_string(); // empty logo

    let mut post_empty = db.get_post_by_id(1).unwrap().expect("Seeded post missing");
    post_empty.title.rendered = "".to_string(); // empty title
    post_empty.content.rendered = "".to_string(); // empty content
    post_empty.author_name = "".to_string(); // empty author
    post_empty.date = "".to_string(); // empty date
    post_empty.featured_image_url = Some("".to_string()); // empty image url

    let mut layout = DaypressLayout {
        version: 1,
        root: DaypressNode {
            id: "root-empty".to_string(),
            kind: NodeKind::Root,
            styles: StyleProperties::default(),
            children: vec![
                DaypressNode {
                    id: "post_title_node".to_string(),
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Fallback".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "post_date_node".to_string(),
                    kind: NodeKind::Text {
                        content_html: "Date".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "author_box_node".to_string(),
                    kind: NodeKind::CustomHtml {
                        raw: "Author".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "site_logo_node".to_string(),
                    kind: NodeKind::Image {
                        src: "none".to_string(),
                        alt: "".to_string(),
                        width: None,
                        height: None,
                        link: None,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
            ],
        },
        global_styles: None,
    };

    let hyd_ctx = HydrationContext {
        post: Some(&post_empty),
        settings: &settings,
        breadcrumbs: vec![],
        nav_menus: HashMap::new(),
    };

    SsrInjector::hydrate_layout(&mut layout, &hyd_ctx);
    let html = render_layout_to_html(&layout);

    // Empty author should fallback to "admin"
    assert!(
        html.contains("admin"),
        "Empty author must fallback to admin"
    );
    // Empty site_logo should fallback to "/static/favicon.svg"
    assert!(
        html.contains("/static/favicon.svg"),
        "Empty site logo must fallback to favicon.svg"
    );

    // Test with post: None
    let hyd_ctx_none = HydrationContext {
        post: None,
        settings: &settings,
        breadcrumbs: vec![],
        nav_menus: HashMap::new(),
    };
    let mut layout_none = layout.clone();
    SsrInjector::hydrate_layout(&mut layout_none, &hyd_ctx_none);
    let html_none = render_layout_to_html(&layout_none);
    // Post title with post=None should fallback to settings.title
    assert!(html_none.contains("Daypress Global"));
}

#[test]
fn test_adversarial_dynamic_tokens_replacement() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let mut settings = db.get_settings().unwrap_or_default();
    settings.title = "Daypress Framework".to_string();
    settings.description = "Ultra-fast headless CMS".to_string();

    let mut post = db.get_post_by_id(1).unwrap().expect("Seeded post missing");
    post.title.rendered = "Dynamic SSR & Tokens in Rust".to_string();
    post.author_name = "Ferris Crab".to_string();
    post.date = "2026-10-23T04:00:00Z".to_string();

    let hyd_ctx = HydrationContext {
        post: Some(&post),
        settings: &settings,
        breadcrumbs: vec![],
        nav_menus: HashMap::new(),
    };

    let template_text = "Article: {{post_title}} by {{post_author}} on {{post_date}} | Copyright {{current_year}} {{site_title}} - {{site_description}}";
    let interpolated = SsrInjector::interpolate_tokens(template_text, &hyd_ctx);

    assert!(interpolated.contains("Article: Dynamic SSR & Tokens in Rust"));
    assert!(interpolated.contains("by Ferris Crab"));
    assert!(interpolated.contains("on 2026-10-23"));
    assert!(interpolated.contains("Copyright 2026"));
    assert!(interpolated.contains("Daypress Framework"));
    assert!(interpolated.contains("Ultra-fast headless CMS"));
    assert!(!interpolated.contains("{{post_title}}"));
    assert!(!interpolated.contains("{{current_year}}"));

    // Note: Standard author token is {{post_author}}
    assert_eq!(
        SsrInjector::interpolate_tokens("By {{post_author}}", &hyd_ctx),
        "By Ferris Crab"
    );
}

#[tokio::test]
async fn test_adversarial_leak_firewall_strips_100_percent_editor_chrome() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let repo = ThemeTemplateRepository::from_database(&db);

    // Create a Header template contaminated with all types of builder chrome
    let mut header_layout = DaypressLayout::default();
    header_layout.root.children.push(DaypressNode {
        id: "chrome-infested-header".to_string(),
        kind: NodeKind::Text {
            content_html: r#"
                <div contenteditable="true" draggable="true" class="elementor-grid-handle daystudio-drag-handle daystudio-handle ui-resizable ui-draggable ui-sortable active-element et_pb_element_editing visual-builder-active my-preserved-class"
                     data-daystudio="container"
                     data-daystudio-node-id="node-12345"
                     data-daystudio-type="Header"
                     data-drag-handle="drag-handle-header"
                     data-model-cid="c42"
                     data-elementor-type="header"
                     data-elementor-id="999">
                    <span contenteditable draggable data-daystudio-sub="title">Clean Brand Name</span>
                    <button type="button" CONTENTEDITABLE="TRUE" DRAGGABLE="TRUE" class="daystudio-btn-handle">Click Me</button>
                </div>
            "#.to_string(),
        },
        styles: StyleProperties::default(),
        children: Vec::new(),
    });

    let _hdr = repo
        .create(&CreateTemplateRequest {
            title: "Infested Theme Header".to_string(),
            template_type: TemplateType::Header,
            status: Some("publish".to_string()),
            layout: Some(header_layout),
            layout_json: None,
            settings: Some(TemplateSettings {
                sticky: true,
                ..Default::default()
            }),
            priority: Some(10),
            is_active: Some(true),
            conditions: Some(vec![CreateConditionDto {
                id: None,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::EntireSite,
                target_id: String::new(),
                priority: 10,
            }]),
        })
        .unwrap();

    let app = create_app(db.clone());
    let resp = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body_bytes.to_vec()).unwrap();

    // 1. Content Verification: legitimate content must be present
    assert!(
        html.contains("Clean Brand Name"),
        "Legitimate content must be preserved"
    );
    assert!(
        html.contains("Click Me"),
        "Legitimate button text must be preserved"
    );
    assert!(
        html.contains("my-preserved-class"),
        "Legitimate classes must be preserved"
    );

    // 2. Strict 100% Editor Chrome Stripping Checks:
    assert!(
        !html.contains("contenteditable"),
        "Must NOT contain contenteditable (lowercase)"
    );
    assert!(
        !html.contains("CONTENTEDITABLE"),
        "Must NOT contain CONTENTEDITABLE (uppercase)"
    );
    assert!(
        !html.contains("draggable"),
        "Must NOT contain draggable (lowercase)"
    );
    assert!(
        !html.contains("DRAGGABLE"),
        "Must NOT contain DRAGGABLE (uppercase)"
    );
    assert!(
        !html.contains("elementor-grid-handle"),
        "Must NOT contain elementor-grid-handle"
    );
    assert!(
        !html.contains("daystudio-drag-handle"),
        "Must NOT contain daystudio-drag-handle"
    );
    assert!(
        !html.contains("daystudio-handle"),
        "Must NOT contain daystudio-handle"
    );
    assert!(
        !html.contains("daystudio-btn-handle"),
        "Must NOT contain daystudio-btn-handle"
    );
    assert!(
        !html.contains("ui-resizable"),
        "Must NOT contain ui-resizable"
    );
    assert!(
        !html.contains("ui-draggable"),
        "Must NOT contain ui-draggable"
    );
    assert!(
        !html.contains("ui-sortable"),
        "Must NOT contain ui-sortable"
    );
    assert!(
        !html.contains("active-element"),
        "Must NOT contain active-element"
    );
    assert!(
        !html.contains("et_pb_element_editing"),
        "Must NOT contain et_pb_element_editing"
    );
    assert!(
        !html.contains("visual-builder-active"),
        "Must NOT contain visual-builder-active"
    );
    assert!(
        !html.contains("data-daystudio"),
        "Must NOT contain data-daystudio attributes"
    );
    assert!(
        !html.contains("data-drag-handle"),
        "Must NOT contain data-drag-handle attributes"
    );
    assert!(
        !html.contains("data-model-cid"),
        "Must NOT contain data-model-cid attributes"
    );
    assert!(
        !html.contains("data-elementor"),
        "Must NOT contain data-elementor attributes"
    );

    // 3. Artifact cleanup verification: ensure empty class="" was removed
    assert!(
        !html.contains(r#"class="""#),
        "Must NOT contain empty class attributes"
    );
    assert!(
        !html.contains(r#"class="   ""#),
        "Must NOT contain whitespace class attributes"
    );
}

#[test]
fn test_adversarial_breadcrumbs_and_nav_menu_html_escaping() {
    let breadcrumbs = vec![
        BreadcrumbItem {
            title: "<script>alert('xss')</script>".to_string(),
            url: "/path/?q=1&b=2\"foo".to_string(),
            is_current: false,
        },
        BreadcrumbItem {
            title: "Current & Active 'Page'".to_string(),
            url: "/current/".to_string(),
            is_current: true,
        },
    ];

    let markup = SsrInjector::render_breadcrumbs_html(&breadcrumbs, " &gt; ", "<Home>");

    // Verify all special characters in breadcrumbs are escaped
    assert!(markup.contains("&lt;Home&gt;"), "Home text must be escaped");
    assert!(markup.contains("&amp;gt;"), "Separator must be escaped");
    assert!(
        markup.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"),
        "XSS script in title must be escaped"
    );
    assert!(
        markup.contains("/path/?q=1&amp;b=2&quot;foo"),
        "URL must be escaped"
    );
    assert!(
        markup.contains("Current &amp; Active &#x27;Page&#x27;"),
        "Active page title must be escaped"
    );
    assert!(
        markup.contains(r#"aria-current="page""#),
        "Active breadcrumb must have aria-current"
    );
}
