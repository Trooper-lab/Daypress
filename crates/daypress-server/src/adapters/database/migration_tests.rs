//! Unit and integration tests for bundle/WXR migration and plugin convergence.

use super::Database;
use crate::models::*;
use std::collections::HashMap;

#[test]
fn test_import_bundle_relational_remapping_and_ast_transpilation() {
    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!(
        "daypress_test_bundle_{}.sqlite",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(1)
    ));
    let db = Database::new(&db_path).expect("Database::new");

    let mut page_meta = HashMap::new();
    page_meta.insert(
            "_elementor_data".to_string(),
            r#"[{"id":"s1","elType":"section","elements":[{"id":"c1","elType":"column","elements":[{"id":"w1","elType":"widget","widgetType":"heading","settings":{"title":"Migrated Elementor Hero","header_size":"h1"}}]}]}]"#.to_string(),
        );

    let req = ImportBundleRequest {
            site_settings: Some(ImportSiteSettings {
                title: Some("Migrated Corp".to_string()),
                description: Some("Fast Edge CMS".to_string()),
                email: Some("hello@migrated.corp".to_string()),
                date_format: None,
                time_format: None,
                posts_per_page: Some(12),
                show_on_front: Some("page".to_string()),
                page_on_front: Some(500),
                page_for_posts: Some(501),
                permalink_structure: Some("/%postname%/".to_string()),
                timezone_string: Some("UTC".to_string()),
                default_comment_status: Some("open".to_string()),
                site_icon: Some(900),
                site_url: None,
            }),
            categories: vec![ImportTerm {
                id: Some(77),
                name: "Engineering".to_string(),
                slug: "engineering".to_string(),
                description: "Engineering posts".to_string(),
                parent: Some(0),
            }],
            tags: vec![ImportTerm {
                id: Some(88),
                name: "Rust".to_string(),
                slug: "rust".to_string(),
                description: "Rust lang".to_string(),
                parent: Some(0),
            }],
            media: vec![ImportMedia {
                id: Some(900),
                title: "Hero Banner".to_string(),
                file_name: "hero.webp".to_string(),
                mime_type: "image/webp".to_string(),
                source_url: "http://old-wp.local/wp-content/uploads/2026/05/hero.webp".to_string(),
                original_url: None,
                alt_text: Some("Hero Alt".to_string()),
                file_size: Some(4096),
                base64_data: None,
            }],
            posts: vec![
                ImportPost {
                    id: Some(600),
                    title: "Rust vs PHP 😊".to_string(),
                    content: "<p>Check out http://old-wp.local/wp-content/uploads/2026/05/hero.webp in our post! 😊</p>".to_string(),
                    excerpt: String::new(),
                    slug: "rust-vs-php".to_string(),
                    status: "publish".to_string(),
                    post_type: "post".to_string(),
                    date: Some("2026-05-01 10:00:00".to_string()),
                    modified: None,
                    featured_media: Some(900),
                    parent: Some(0),
                    menu_order: Some(0),
                    author: Some(1),
                    comment_status: None,
                    ping_status: None,
                    template: None,
                    categories: vec![77],
                    tags: vec![88],
                    postmeta: HashMap::new(),
                },
                ImportPost {
                    id: Some(500),
                    title: "Landing Home".to_string(),
                    content: "<p>Legacy fallback</p>".to_string(),
                    excerpt: String::new(),
                    slug: "landing-home".to_string(),
                    status: "publish".to_string(),
                    post_type: "page".to_string(),
                    date: Some("2026-05-01 09:00:00".to_string()),
                    modified: None,
                    featured_media: Some(900),
                    parent: Some(0),
                    menu_order: Some(1),
                    author: Some(1),
                    comment_status: None,
                    ping_status: None,
                    template: None,
                    categories: vec![],
                    tags: vec![],
                    postmeta: page_meta,
                },
                ImportPost {
                    id: Some(501),
                    title: "Child Page".to_string(),
                    content: "<p>Subpage content</p>".to_string(),
                    excerpt: String::new(),
                    slug: "child-page".to_string(),
                    status: "publish".to_string(),
                    post_type: "page".to_string(),
                    date: Some("2026-05-01 09:30:00".to_string()),
                    modified: None,
                    featured_media: None,
                    parent: Some(500),
                    menu_order: Some(2),
                    author: Some(1),
                    comment_status: None,
                    ping_status: None,
                    template: None,
                    categories: vec![],
                    tags: vec![],
                    postmeta: HashMap::new(),
                },
            ],
            theme_mods: None,
            menus: vec![ImportMenu {
                id: Some(1),
                name: "Main Menu".to_string(),
                slug: "main-menu".to_string(),
                locations: vec!["primary".to_string()],
                items: vec![NavMenuItemDto {
                    id: 1,
                    title: "Home".to_string(),
                    url: "http://old-wp.local/?page_id=500".to_string(),
                    target: String::new(),
                    parent: 0,
                    menu_order: 1,
                    ..Default::default()
                }],
            }],
            comments: vec![ImportComment {
                id: Some(301),
                post_id: 600,
                author: "Alice".to_string(),
                author_email: "alice@example.com".to_string(),
                author_url: String::new(),
                author_ip: "127.0.0.1".to_string(),
                date: "2026-05-01 12:00:00".to_string(),
                content: "Great migration!".to_string(),
                approved: "1".to_string(),
                parent: 0,
                user_id: 0,
            }],
            users: vec![],
            daystudio_kit: Some(serde_json::json!({
                "system_colors": [
                    {"_id": "primary", "title": "Primary", "color": "#ff4f00"}
                ]
            })),
            widgets: None,
            plugin_options: HashMap::new(),
            redirects: vec![],
            forms: vec![],
            site_url: None,
        };

    let res = db.import_bundle(&req).expect("import_bundle");
    assert!(res.success);
    assert_eq!(res.migrated_posts, 1);
    assert_eq!(res.migrated_pages, 2);
    assert_eq!(res.migrated_categories, 1);
    assert_eq!(res.migrated_tags, 1);
    assert_eq!(res.migrated_media, 1);

    let landing = db
        .get_post_by_slug("landing-home")
        .unwrap()
        .expect("landing-home exists");
    let child = db
        .get_post_by_slug("child-page")
        .unwrap()
        .expect("child-page exists");
    assert_eq!(child.parent, landing.id);

    let settings = db.get_settings().unwrap();
    assert_eq!(settings.show_on_front, "page");
    assert_eq!(settings.page_on_front, landing.id);
    assert_eq!(settings.page_for_posts, child.id);

    let layout = db.get_post_layout(landing.id).unwrap();
    assert!(layout.is_some());
    assert!(landing.content.rendered.contains("Migrated Elementor Hero"));

    let post = db
        .get_post_by_slug("rust-vs-php")
        .unwrap()
        .expect("rust-vs-php exists");
    assert!(post.category_names.contains(&"Engineering".to_string()));
    assert!(post.tag_names.contains(&"Rust".to_string()));
    assert!(post.featured_image_url.is_some());
    assert_eq!(post.comments.len(), 1);
    assert_eq!(post.comments[0].content.rendered, "Great migration!");

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_password_hashing_and_verification() {
    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!(
        "daypress_test_auth_{}.sqlite",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(2)
    ));
    let db = Database::new(&db_path).expect("Database::new");

    // A fresh install never accepts the historical admin/admin default: the seeded admin
    // receives a generated password on first start.
    assert!(db
        .verify_user_credentials("admin", "admin")
        .unwrap()
        .is_none());
    assert!(db
        .verify_user_credentials("admin", "wrong-password")
        .unwrap()
        .is_none());

    // Newly created user with salted SHA-256 hash
    let user = db
        .create_user(&CreateUserRequest {
            username: "editor_jane".to_string(),
            email: "jane@example.com".to_string(),
            password: "CorrectHorseBatteryStaple!".to_string(),
            name: None,
            first_name: None,
            last_name: None,
            display_name: Some("Jane Editor".to_string()),
            url: None,
            role: Some("Editor".to_string()),
            ..Default::default()
        })
        .expect("create_user");
    assert_eq!(user.username, "editor_jane");
    assert!(db
        .verify_user_credentials("editor_jane", "CorrectHorseBatteryStaple!")
        .unwrap()
        .is_some());
    assert!(db
        .verify_user_credentials("editor_jane", "admin")
        .unwrap()
        .is_none());

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_duplicate_post_and_plugin_convergence_and_sdk() {
    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!(
        "daypress_test_plugins_{}.sqlite",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(3)
    ));
    let db = Database::new(&db_path).expect("Database::new");

    // 1. Create a post with SEO metadata and categories, then duplicate it
    let created = db
        .create_post(&CreatePostRequest {
            title: "Original Pillar Post".to_string(),
            content: "<p>Pillar content</p>".to_string(),
            status: "publish".to_string(),
            slug: Some("original-pillar-post".to_string()),
            post_type: "post".to_string(),
            seo_title: Some("Pillar SEO Title".to_string()),
            seo_description: Some("Pillar SEO Description".to_string()),
            seo_keyword: Some("rust cms".to_string()),
            categories: vec![1],
            ..Default::default()
        })
        .expect("create_post");

    let cloned = db
        .duplicate_post(created.id)
        .expect("duplicate_post")
        .expect("cloned post");
    assert_ne!(cloned.id, created.id);
    assert_eq!(cloned.status, "draft");
    assert!(cloned.title.rendered.contains("(Copy)"));
    assert_eq!(cloned.seo_title, "Pillar SEO Title");
    assert_eq!(cloned.seo_keyword, "rust cms");

    // 2. Slug change on published post automatically creates a 301 redirect
    let _updated = db
        .update_post(
            created.id,
            &UpdatePostRequest {
                slug: Some("new-pillar-slug".to_string()),
                ..Default::default()
            },
        )
        .expect("update_post");

    let matched = db.match_redirect_and_increment("/original-pillar-post");
    assert_eq!(matched, Some(("/new-pillar-slug/".to_string(), 301)));

    // 3. Universal Plugin Convergence via import_bundle (Yoast + CF7 + Redirection + Joinchat)
    let mut postmeta = HashMap::new();
    postmeta.insert(
        "_yoast_wpseo_title".to_string(),
        "Converged Yoast Title".to_string(),
    );
    postmeta.insert(
        "_yoast_wpseo_metadesc".to_string(),
        "Converged Yoast Desc".to_string(),
    );
    postmeta.insert(
        "_yoast_wpseo_focuskw".to_string(),
        "daypress migration".to_string(),
    );

    let mut plugin_options = HashMap::new();
    plugin_options.insert(
        "joinchat".to_string(),
        serde_json::json!({
            "telephone": "+15550199999",
            "message_send": "Hello from converged Joinchat!",
            "button_tip": "Chat on WhatsApp"
        }),
    );

    let bundle = ImportBundleRequest {
        posts: vec![ImportPost {
            id: Some(700),
            title: "Contact Us".to_string(),
            slug: "contact-us-converged".to_string(),
            content: "<p>Get in touch:</p>[contact-form-7 id=\"55\" title=\"Main Contact\"]"
                .to_string(),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            postmeta,
            ..Default::default()
        }],
        forms: vec![ImportForm {
            id: Some(55),
            title: "Main Contact".to_string(),
            source_plugin: "cf7".to_string(),
            raw_form: "[text* your-name] [email* your-email] [textarea your-message]".to_string(),
            recipient_email: "leads@example.com".to_string(),
            schema_json: None,
        }],
        redirects: vec![ImportRedirect {
            source_path: "/legacy-about".to_string(),
            target_url: "/contact-us-converged/".to_string(),
            status_code: 301,
        }],
        plugin_options,
        ..Default::default()
    };

    let import_res = db.import_bundle(&bundle).expect("import_bundle");
    assert!(import_res.success);
    assert_eq!(import_res.migrated_forms, 1);
    assert!(import_res.migrated_redirects >= 1);

    let migrated_page = db
        .get_post_by_slug("contact-us-converged")
        .unwrap()
        .expect("contact-us-converged");
    assert_eq!(migrated_page.seo_title, "Converged Yoast Title");
    assert_eq!(migrated_page.seo_keyword, "daypress migration");
    assert!(migrated_page
        .content
        .rendered
        .contains("[daypress_form id=\""));

    // Expanding shortcodes renders the native form HTML
    let settings = db.get_settings().unwrap_or_default();
    let expanded = crate::plugins::expand_shortcodes_in_content(
        &migrated_page.content.rendered,
        &db,
        &settings,
    );
    assert!(expanded.contains("<form"));
    assert!(expanded.contains("your-email"));

    // WhatsApp plugin was auto-activated and configured from Joinchat
    let wa_plugin = db
        .get_installed_plugin("whatsapp-chat")
        .unwrap()
        .expect("whatsapp-chat");
    assert!(wa_plugin.active);
    assert_eq!(
        wa_plugin.config.get("phone_number").map(|s| s.as_str()),
        Some("+15550199999")
    );

    // 4. Community Plugin SDK installation & SSR hook rendering
    let sdk_manifest = crate::plugins::starter_plugin_sdk_template();
    db.install_custom_plugin(&sdk_manifest, None)
        .expect("install_custom_plugin");
    let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, None);
    assert!(hooks.body_top_html.contains("Powered by Daypress Rust CMS"));

    let _ = std::fs::remove_file(&db_path);
}
