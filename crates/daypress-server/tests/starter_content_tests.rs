//! A new install gets neutral, WordPress-style starter content: a first post with a comment,
//! a sample page, and one page built with DayStudio, plus a WordPress-inspired First-Time Setup
//! onboarding wizard when no admin credentials have been configured yet.

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use daypress_server::{create_app, Database};
use tower::ServiceExt;

fn seeded_db() -> Database {
    let db = Database::new_in_memory().expect("in-memory DB");
    db.seed_starter_site().expect("seed starter site");
    db
}

async fn get_page(db: &Database, uri: &str) -> (StatusCode, String) {
    let response = create_app(db.clone())
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8_lossy(&body).into_owned())
}

#[test]
fn starter_site_has_exactly_the_sample_post_and_pages() {
    let db = seeded_db();

    let posts = db.get_posts("post", Some("any"), None, 100).unwrap();
    assert_eq!(posts.len(), 1, "one sample post");
    assert_eq!(posts[0].id, 1);
    assert_eq!(posts[0].title.rendered, "Hello world!");
    assert_eq!(posts[0].slug, "hello-world");
    assert_eq!(posts[0].status, "publish");

    let mut pages = db.get_posts("page", Some("any"), None, 100).unwrap();
    pages.sort_by_key(|p| p.id);
    let titles: Vec<&str> = pages.iter().map(|p| p.title.rendered.as_str()).collect();
    assert_eq!(titles, vec!["Sample Page", "A page built with DayStudio"]);
    assert_eq!(pages[0].id, 2);
    assert_eq!(pages[1].id, 3);

    let settings = db.get_settings().unwrap();
    assert_eq!(settings.description, "A self-hosted CMS written in Rust.");
    assert_eq!(settings.email, "admin@example.com");
    assert_eq!(settings.active_theme, "Daypress Daylight");
    assert_eq!(settings.brand_color, "#a8401b");
    assert_eq!(settings.typography_scale, "serif");
}

#[test]
fn starter_content_makes_no_product_claims() {
    let db = seeded_db();
    let mut items = db.get_posts("post", Some("any"), None, 100).unwrap();
    items.extend(db.get_posts("page", Some("any"), None, 100).unwrap());

    let banned = [
        "Litestream",
        "WebAssembly",
        "Lighthouse",
        "guarantee",
        "Pricing",
        "Agency",
        "Shadow Audit",
        "in under an hour",
    ];
    for item in &items {
        let text = format!(
            "{} {} {}",
            item.title.rendered, item.content.rendered, item.excerpt.rendered
        );
        for word in banned {
            assert!(
                !text.contains(word),
                "starter item {} ({}) mentions {word:?}",
                item.id,
                item.title.rendered
            );
        }
    }
}

#[test]
fn daystudio_sample_page_has_a_valid_layout_with_a_grid() {
    let db = seeded_db();
    let layout_json = db
        .get_post_layout(3)
        .unwrap()
        .expect("sample page has a DayStudio layout");
    let layout: daypress_ast::DaypressLayout =
        serde_json::from_str(&layout_json).expect("layout parses as a DaypressLayout");
    let value = serde_json::to_value(&layout).unwrap();
    assert!(
        value.to_string().contains("\"Grid\""),
        "the sample layout shows a grid"
    );
}

#[tokio::test]
async fn starter_pages_render_on_the_public_site() {
    let db = seeded_db();

    let (status, html) = get_page(&db, "/hello-world/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Hello world!"));
    assert!(html.contains("This is your first post."));
    assert!(html.contains("class=\"daypress-daylight\""));

    let (status, html) = get_page(&db, "/sample-page/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("This is an example page."));

    let (status, html) = get_page(&db, "/daystudio-sample/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("A page built with DayStudio"));
    assert!(html.contains("These three cards sit in a grid"));
    assert!(!html.contains("contenteditable"));
    assert!(!html.contains("elementor-grid-handle"));
    // Words and slugs that look like editor class names survive the SSR sanitizer.
    assert!(html.contains("Edit this page in DayStudio"));

    let (status, html) = get_page(&db, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Hello world!"));
}

#[test]
fn a_new_database_file_starts_with_the_starter_content() {
    let dir = std::env::temp_dir().join(format!(
        "daypress_starter_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let db_path = dir.join("daypress.sqlite");
    {
        let db = Database::new(&db_path).expect("open new database file");
        let hello = db.get_post_by_slug("hello-world").unwrap().expect("post");
        assert_eq!(hello.id, 1);
        let all = db.get_posts("any", Some("any"), None, 100).unwrap();
        assert_eq!(all.len(), 3);
        assert!(
            db.needs_onboarding(),
            "new file database enters onboarding wizard until admin account is configured"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn first_time_onboarding_flow_creates_admin_sets_theme_and_locks_setup() {
    let dir = std::env::temp_dir().join(format!(
        "daypress_onboarding_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let db_path = dir.join("daypress.sqlite");
    {
        let db = Database::new(&db_path).expect("open new database file");
        assert!(db.needs_onboarding());

        // 1. Visiting /d-admin/ or /login redirects to /setup before onboarding is completed.
        let res = create_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/d-admin/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            res.headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/setup")
        );

        let res = create_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            res.headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/setup")
        );

        // 2. GET /setup renders the onboarding wizard.
        let (status, html) = get_page(&db, "/setup").await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Install Daypress &amp; Sign In"));
        assert!(html.contains("Daypress Daylight"));

        // 3. Submitting a valid setup form configures the site, creates the admin, and signs in.
        let form_body = "site_title=The+Daily+Press&tagline=Fast+Rust+Publishing&username=founder&display_name=Jane+Founder&email=jane%40example.com&password=CorrectHorseBatteryStaple99!&theme=Daypress+Modern&starter_mode=starter";
        let res = create_app(db.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/setup")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            res.headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/d-admin/?welcome=1")
        );
        let cookie = res
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|v| v.to_str().ok())
            .expect("session cookie issued");
        assert!(cookie.starts_with("daypress_admin="));

        // 4. Onboarding is now complete and locked against re-running.
        assert!(!db.needs_onboarding());
        let settings = db.get_settings().unwrap();
        assert_eq!(settings.title, "The Daily Press");
        assert_eq!(settings.description, "Fast Rust Publishing");
        assert_eq!(settings.email, "jane@example.com");
        assert_eq!(settings.active_theme, "Daypress Modern");

        let res_locked = create_app(db.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/setup")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_locked.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            res_locked
                .headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/login")
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
