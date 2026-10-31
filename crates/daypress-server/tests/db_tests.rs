use daypress_server::models::*;
use daypress_server::Database;

#[test]
fn test_db_initial_seed() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let settings = db.get_settings().expect("Failed to get settings");
    assert_eq!(settings.title, "WordPress Reference");
    assert_eq!(settings.url, "http://localhost:8090");

    let posts = db
        .get_posts("post", Some("publish"), None, 10)
        .expect("Failed to get posts");
    assert!(!posts.is_empty(), "Should contain seeded posts");

    let hello_world = posts.iter().find(|p| p.slug == "hello-world");
    assert!(hello_world.is_some(), "Hello world post must exist");

    let pages = db
        .get_posts("page", Some("publish"), None, 10)
        .expect("Failed to get pages");
    let sample_page = pages.iter().find(|p| p.slug == "sample-page");
    assert!(sample_page.is_some(), "Sample page must exist");
}

#[test]
fn test_create_and_get_post() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let req = CreatePostRequest {
        title: "Test Industry Grade Architecture".to_string(),
        content: "<p>Engineered for high performance and zero downtime.</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("industry-grade-architecture".to_string()),
        post_type: "post".to_string(),
        excerpt: Some("Architecture summary".to_string()),
        parent: Some(0),
        menu_order: Some(1),
        featured_media: Some(0),
        categories: vec![1],
        tags: vec![],
        ..Default::default()
    };

    let post = db.create_post(&req).expect("Failed to create post");
    assert_eq!(post.title.rendered, "Test Industry Grade Architecture");
    assert_eq!(post.slug, "industry-grade-architecture");
    assert_eq!(post.status, "publish");

    // Fetch by ID
    let fetched = db
        .get_post_by_id(post.id)
        .expect("Query failed")
        .expect("Post not found");
    assert_eq!(fetched.id, post.id);
    assert_eq!(fetched.title.rendered, post.title.rendered);
    assert_eq!(fetched.slug, "industry-grade-architecture");
    assert_eq!(fetched.categories, vec![1]);

    // Fetch by Slug
    let by_slug = db
        .get_post_by_slug("industry-grade-architecture")
        .expect("Query failed")
        .expect("Post not found");
    assert_eq!(by_slug.id, post.id);
    assert_eq!(by_slug.title.rendered, post.title.rendered);
}

#[test]
fn test_update_post_transactional() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let req = CreatePostRequest {
        title: "Initial Draft".to_string(),
        content: "<p>Initial text</p>".to_string(),
        status: "draft".to_string(),
        slug: Some("initial-draft".to_string()),
        post_type: "post".to_string(),
        excerpt: None,
        parent: None,
        menu_order: None,
        featured_media: None,
        categories: vec![],
        tags: vec![],
        ..Default::default()
    };

    let post = db.create_post(&req).expect("Failed to create post");
    assert_eq!(post.status, "draft");

    // Update to publish with new title and content
    let update_req = UpdatePostRequest {
        title: Some("Published Title".to_string()),
        content: Some("<p>Updated production content.</p>".to_string()),
        status: Some("publish".to_string()),
        slug: Some("published-title".to_string()),
        excerpt: Some("Updated excerpt".to_string()),
        parent: None,
        menu_order: None,
        featured_media: None,
        categories: Some(vec![1]),
        tags: None,
        ..Default::default()
    };

    let updated = db
        .update_post(post.id, &update_req)
        .expect("Update failed")
        .expect("Post missing");
    assert_eq!(updated.title.rendered, "Published Title");
    assert_eq!(
        updated.content.rendered,
        "<p>Updated production content.</p>"
    );
    assert_eq!(updated.status, "publish");
    assert_eq!(updated.slug, "published-title");
}

#[test]
fn test_delete_post() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let req = CreatePostRequest {
        title: "To Be Deleted".to_string(),
        content: "<p>Goodbye</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("to-be-deleted".to_string()),
        post_type: "post".to_string(),
        excerpt: None,
        parent: None,
        menu_order: None,
        featured_media: None,
        categories: vec![],
        tags: vec![],
        ..Default::default()
    };

    let post = db.create_post(&req).expect("Failed to create post");
    assert!(db.delete_post(post.id).expect("Delete failed"));

    // Verify deleted
    let fetched = db.get_post_by_id(post.id).expect("Query failed");
    assert!(fetched.is_none());
}

#[test]
fn test_get_posts_filtering_and_search() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let req1 = CreatePostRequest {
        title: "Rust Performance Benchmarks".to_string(),
        content: "<p>Sub-millisecond latency with Axum.</p>".to_string(),
        status: "publish".to_string(),
        slug: Some("rust-perf".to_string()),
        post_type: "post".to_string(),
        excerpt: None,
        parent: None,
        menu_order: None,
        featured_media: None,
        categories: vec![1],
        tags: vec![],
        ..Default::default()
    };
    db.create_post(&req1).expect("Failed post 1");

    let req2 = CreatePostRequest {
        title: "Draft Article".to_string(),
        content: "<p>Draft notes.</p>".to_string(),
        status: "draft".to_string(),
        slug: Some("draft-article".to_string()),
        post_type: "post".to_string(),
        excerpt: None,
        parent: None,
        menu_order: None,
        featured_media: None,
        categories: vec![],
        tags: vec![],
        ..Default::default()
    };
    db.create_post(&req2).expect("Failed post 2");

    // Search by title keyword with parameterized query
    let search_results = db
        .get_posts("post", Some("any"), Some("Benchmarks"), 10)
        .expect("Search failed");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].slug, "rust-perf");

    // Status filter
    let drafts = db
        .get_posts("post", Some("draft"), None, 10)
        .expect("Draft query failed");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].slug, "draft-article");
}

#[test]
fn test_settings_and_base_url() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    db.set_option("blogname", "Daypress Production Suite")
        .expect("Set option failed");
    db.set_option("siteurl", "https://daypress.example.com")
        .expect("Set option failed");

    let settings = db.get_settings().expect("Get settings failed");
    assert_eq!(settings.title, "Daypress Production Suite");
    assert_eq!(settings.url, "https://daypress.example.com");

    assert_eq!(db.get_base_url(), "https://daypress.example.com");
}

#[test]
fn test_media_attachment_crud() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let media = db
        .create_attachment(
            "Hero Image",
            "hero.png",
            "image/png",
            45230,
            "/uploads/hero.png",
        )
        .expect("Create attachment failed");

    assert_eq!(media.title.rendered, "Hero Image");
    assert_eq!(media.file_size, 45230);
    assert_eq!(media.source_url, "/uploads/hero.png");

    let list = db.get_media_attachments().expect("List media failed");
    assert!(list.iter().any(|m| m.id == media.id));

    let deleted_url = db
        .delete_media_attachment(media.id)
        .expect("Delete media failed");
    assert_eq!(deleted_url, Some("/uploads/hero.png".to_string()));
}

#[test]
fn test_terms_and_taxonomy() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let categories = db.get_terms("category").expect("Failed to get categories");
    assert!(!categories.is_empty());
    assert_eq!(categories[0].name, "Uncategorized");
    assert_eq!(categories[0].slug, "uncategorized");
}

#[test]
fn test_user_detail_and_update() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let user = db
        .get_user_detail(1)
        .expect("Failed to get user")
        .expect("User 1 missing");
    assert_eq!(user.username, "admin");

    let update = UpdateUserRequest {
        display_name: Some("Lead Architect".to_string()),
        email: Some("architect@daypress.io".to_string()),
        url: Some("https://daypress.io".to_string()),
        password: None,
        ..Default::default()
    };

    let updated = db
        .update_user_detail(1, &update)
        .expect("Update failed")
        .expect("User missing");
    assert_eq!(updated.display_name, "Lead Architect");
    assert_eq!(updated.email, "architect@daypress.io");
}

#[test]
fn test_vacuum_and_integrity_check() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    assert!(db.vacuum().is_ok());
    let integrity = db.integrity_check().expect("Integrity check failed");
    assert_eq!(integrity, "ok");
}

#[test]
fn test_import_bundle_atomicity() {
    let db = Database::new_in_memory().expect("Failed to initialize in-memory DB");

    let bundle = ImportBundleRequest {
        site_settings: Some(ImportSiteSettings {
            title: Some("Imported Production Site".to_string()),
            description: Some("Zero Downtime Migration".to_string()),
            email: Some("ops@daypress.io".to_string()),
            date_format: Some("Y-m-d".to_string()),
            time_format: Some("H:i".to_string()),
            posts_per_page: Some(25),
            ..Default::default()
        }),
        categories: vec![ImportTerm {
            id: None,
            name: "Cloud Architecture".to_string(),
            slug: "cloud-architecture".to_string(),
            description: "Distributed Systems & Cloud".to_string(),
            ..Default::default()
        }],
        tags: vec![ImportTerm {
            id: None,
            name: "Tokio".to_string(),
            slug: "tokio".to_string(),
            description: "Async Runtime".to_string(),
            ..Default::default()
        }],
        posts: vec![ImportPost {
            id: None,
            title: "Migrated Post Title".to_string(),
            slug: "migrated-post-title".to_string(),
            content: "<p>Successfully imported via Daypress 1-Click Engine.</p>".to_string(),
            excerpt: "Import summary".to_string(),
            status: "publish".to_string(),
            post_type: "post".to_string(),
            date: Some("2026-10-21 20:00:00".to_string()),
            categories: vec![],
            tags: vec![],
            featured_media: None,
            ..Default::default()
        }],
        media: vec![],
        theme_mods: None,
        ..Default::default()
    };

    let res = db.import_bundle(&bundle).expect("Import bundle failed");
    assert!(res.success);
    assert_eq!(res.migrated_posts, 1);
    assert_eq!(res.migrated_categories, 1);
    assert_eq!(res.migrated_tags, 1);

    // Verify settings updated
    let settings = db.get_settings().expect("Failed to get settings");
    assert_eq!(settings.title, "Imported Production Site");

    // Verify post exists
    let post = db
        .get_post_by_slug("migrated-post-title")
        .expect("Query failed")
        .expect("Post not found");
    assert_eq!(post.title.rendered, "Migrated Post Title");
}

#[test]
fn test_comments_moderation_crud() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // Seeded comment check
    let comments = db.get_comments(None, 50).expect("Failed to get comments");
    assert!(!comments.is_empty(), "Should contain seeded comments");
    let initial_comment_id = comments[0].id;

    // Comment counts
    let counts = db.get_comment_counts().expect("Failed to get counts");
    assert!(counts.all >= 1);
    assert_eq!(counts.approved, 1);

    // Update status to spam
    let updated = db
        .update_comment_status(initial_comment_id, "spam")
        .expect("Update failed");
    assert!(updated);
    let counts_spam = db.get_comment_counts().expect("Failed to get counts");
    assert_eq!(counts_spam.spam, 1);

    // Update status to trash
    let updated = db
        .update_comment_status(initial_comment_id, "trash")
        .expect("Update failed");
    assert!(updated);
    let counts_trash = db.get_comment_counts().expect("Failed to get counts");
    assert_eq!(counts_trash.trash, 1);

    // Delete comment
    let deleted = db
        .delete_comment(initial_comment_id)
        .expect("Delete failed");
    assert!(deleted);
    let counts_after = db.get_comment_counts().expect("Failed to get counts");
    assert_eq!(counts_after.all, 0);
}

#[test]
fn test_all_settings_sections() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // 1. Writing Settings
    let writing = db.get_writing_settings().expect("Writing settings failed");
    assert_eq!(writing.mailserver_port, 110);
    let new_writing = WritingSettings {
        default_category: 1,
        default_post_format: "aside".to_string(),
        use_smilies: false,
        default_pingback_flag: false,
        mailserver_url: "smtp.company.org".to_string(),
        mailserver_login: "wp@company.org".to_string(),
        mailserver_pass: "secret".to_string(),
        mailserver_port: 993,
    };
    db.update_writing_settings(&new_writing)
        .expect("Update writing failed");
    let fetched_writing = db.get_writing_settings().expect("Fetch writing failed");
    assert_eq!(fetched_writing.default_post_format, "aside");
    assert_eq!(fetched_writing.mailserver_port, 993);

    // 2. Reading Settings
    let reading = db.get_reading_settings().expect("Reading settings failed");
    assert_eq!(reading.posts_per_page, 10);
    let new_reading = ReadingSettings {
        show_on_front: "page".to_string(),
        page_on_front: 2,
        page_for_posts: 1,
        posts_per_page: 25,
        posts_per_rss: 15,
        blog_public: false,
    };
    db.update_reading_settings(&new_reading)
        .expect("Update reading failed");
    let fetched_reading = db.get_reading_settings().expect("Fetch reading failed");
    assert_eq!(fetched_reading.show_on_front, "page");
    assert_eq!(fetched_reading.posts_per_page, 25);
    assert!(!fetched_reading.blog_public);

    // 3. Discussion Settings
    let discussion = db
        .get_discussion_settings()
        .expect("Discussion settings failed");
    assert!(discussion.show_avatars);
    let new_discussion = DiscussionSettings {
        default_ping_status: "closed".to_string(),
        default_comment_status: "closed".to_string(),
        require_name_email: true,
        comment_registration: true,
        close_comments_for_old_posts: true,
        close_comments_days_old: 30,
        thread_comments: true,
        thread_comments_depth: 3,
        comment_moderation: true,
        comment_previously_approved: false,
        show_avatars: false,
    };
    db.update_discussion_settings(&new_discussion)
        .expect("Update discussion failed");
    let fetched_disc = db
        .get_discussion_settings()
        .expect("Fetch discussion failed");
    assert_eq!(fetched_disc.default_comment_status, "closed");
    assert_eq!(fetched_disc.close_comments_days_old, 30);
    assert!(!fetched_disc.show_avatars);

    // 4. Media Settings
    let media = db.get_media_settings().expect("Media settings failed");
    assert_eq!(media.thumbnail_size_w, 150);
    let new_media = MediaSettings {
        thumbnail_size_w: 200,
        thumbnail_size_h: 200,
        thumbnail_crop: false,
        medium_size_w: 400,
        medium_size_h: 400,
        large_size_w: 1200,
        large_size_h: 1200,
        uploads_use_yearmonth_folders: false,
    };
    db.update_media_settings(&new_media)
        .expect("Update media failed");
    let fetched_media = db.get_media_settings().expect("Fetch media failed");
    assert_eq!(fetched_media.thumbnail_size_w, 200);
    assert_eq!(fetched_media.large_size_w, 1200);
    assert!(!fetched_media.uploads_use_yearmonth_folders);

    // 5. Permalink Settings
    let permalink = db
        .get_permalink_settings()
        .expect("Permalink settings failed");
    assert_eq!(permalink.permalink_structure, "/%postname%/");
    let new_permalink = PermalinkSettings {
        permalink_structure: "/%year%/%monthnum%/%postname%/".to_string(),
        category_base: "topics".to_string(),
        tag_base: "labels".to_string(),
    };
    db.update_permalink_settings(&new_permalink)
        .expect("Update permalink failed");
    let fetched_permalink = db.get_permalink_settings().expect("Fetch permalink failed");
    assert_eq!(
        fetched_permalink.permalink_structure,
        "/%year%/%monthnum%/%postname%/"
    );
    assert_eq!(fetched_permalink.category_base, "topics");

    // 6. Privacy Settings
    let privacy = db.get_privacy_settings().expect("Privacy settings failed");
    assert_eq!(privacy.wp_page_for_privacy_policy, 0);
    let new_privacy = PrivacySettings {
        wp_page_for_privacy_policy: 42,
    };
    db.update_privacy_settings(&new_privacy)
        .expect("Update privacy failed");
    let fetched_privacy = db.get_privacy_settings().expect("Fetch privacy failed");
    assert_eq!(fetched_privacy.wp_page_for_privacy_policy, 42);
}

#[test]
fn test_wxr_export_and_privacy_tools() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // 1. WXR Export
    let wxr = db.export_wxr().expect("WXR export failed");
    assert!(wxr.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(wxr.contains("<rss version=\"2.0\""));
    assert!(wxr.contains("<wp:wxr_version>1.2</wp:wxr_version>"));
    assert!(wxr.contains("Hello world!"));

    // 2. Personal Data Export
    let personal = db
        .export_personal_data("commenter@example.com")
        .expect("Personal data export failed");
    assert_eq!(personal["requested_identifier"], "commenter@example.com");
    assert!(!personal["comments"].as_array().unwrap().is_empty());

    // 3. Personal Data Erase
    let scrubbed = db
        .erase_personal_data("commenter@example.com")
        .expect("Personal data erase failed");
    assert_eq!(scrubbed, 1);

    // Verify comment is scrubbed
    let comments = db.get_comments(None, 10).expect("Failed to get comments");
    assert_eq!(comments[0].author, "Anonymous");
    assert_eq!(comments[0].author_email, "deleted@privacy.local");
    assert_eq!(comments[0].author_ip, "0.0.0.0");
}

#[test]
fn test_hybrid_layout_persistence_and_dual_write() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // 1. Initial state: Post 1 (Hello world) has no custom layout
    let layout = db.get_post_layout(1).expect("Failed to query layout");
    assert!(layout.is_none());

    // 2. Save layout with pre-rendered semantic HTML
    let sample_layout =
        r#"{"version":1,"root":{"id":"sec_1","type":"Section","styles":{},"children":[]}}"#;
    let pre_rendered_html = "<section class=\"dp-sec-1\"><div class=\"dp-container\"><p>Fast Rust CMS</p></div></section>";

    db.save_post_layout(1, sample_layout, Some(pre_rendered_html))
        .expect("Failed to save post layout");

    // 3. Fetch layout: should retrieve canonical JSON
    let fetched = db.get_post_layout(1).expect("Failed to retrieve layout");
    assert_eq!(fetched, Some(sample_layout.to_string()));

    // 4. Verify post_content was synchronized for public SSR
    let post = db
        .get_post_by_id(1)
        .expect("Fetch post failed")
        .expect("Post 1 missing");
    assert_eq!(post.content.rendered, pre_rendered_html);
}

#[test]
fn test_native_subsystems_forms_and_analytics() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    // 1. Native Forms
    let form_id = db
        .create_form(
            "Contact Agency",
            r#"{"fields":[{"name":"email","type":"email"}]}"#,
            r#"{"submit_text":"Send Message"}"#,
        )
        .expect("Create form failed");
    assert!(form_id > 0);

    let form = db
        .get_form(form_id)
        .expect("Get form failed")
        .expect("Form missing");
    assert_eq!(form.title, "Contact Agency");

    let entry_id = db
        .submit_form_entry(
            form_id,
            r#"{"email":"client@agency.com","budget":"10k"}"#,
            "a1b2c3d4",
            "Mozilla/5.0",
        )
        .expect("Submit form entry failed");
    assert!(entry_id > 0);

    // 2. Native Cookieless Analytics
    db.record_analytics_event(
        "anon_daily_hash_1",
        "/pricing",
        "https://google.com",
        "desktop",
        "US",
        Some("google"),
        Some("cpc"),
        Some("spring_launch"),
    )
    .expect("Record analytics failed");

    db.record_analytics_event(
        "anon_daily_hash_2",
        "/pricing",
        "",
        "mobile",
        "DE",
        None,
        None,
        None,
    )
    .expect("Record analytics failed");

    let summary = db.get_analytics_summary().expect("Get summary failed");
    assert_eq!(summary.total_visitors, 2);
    assert_eq!(summary.total_pageviews, 2);
    assert_eq!(summary.top_pages[0].page_url, "/pricing");
    assert_eq!(summary.top_pages[0].views, 2);
}

#[test]
fn test_native_email_logging() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");

    db.log_email(
        "client@enterprise.com",
        "Welcome to Daypress Agency SaaS",
        "lettre_smtp",
        "delivered",
        142,
        None,
    )
    .expect("Log email failed");

    let logs = db.get_email_logs(10).expect("Get email logs failed");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].recipient, "client@enterprise.com");
    assert_eq!(logs[0].transport, "lettre_smtp");
    assert_eq!(logs[0].status, "delivered");
    assert_eq!(logs[0].latency_ms, 142);
}
