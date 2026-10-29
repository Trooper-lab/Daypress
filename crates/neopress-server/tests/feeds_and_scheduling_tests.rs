//! RSS 2.0 feeds (`/feed/`, `/category/{slug}/feed/`, `/tag/{slug}/feed/`) and scheduled
//! (`future`) posts: hidden from every public read path until their date passes.

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    Router,
};
use neopress_server::{create_app, Database};
use serde_json::{json, Value};
use tower::ServiceExt;

const ADMIN_AUTH: &str = "Basic YWRtaW46YWRtaW4="; // admin:admin

struct Resp {
    status: StatusCode,
    content_type: String,
    body: String,
}

impl Resp {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
}

async fn send(app: &Router, method: &str, uri: &str, auth: bool, body: Option<Value>) -> Resp {
    let mut builder = Request::builder().method(method).uri(uri);
    if auth {
        builder = builder.header("Authorization", ADMIN_AUTH);
    }
    let req = match body {
        Some(json) => builder
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&json).unwrap()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    Resp {
        status,
        content_type,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

async fn get(app: &Router, uri: &str) -> Resp {
    send(app, "GET", uri, false, None).await
}

async fn get_auth(app: &Router, uri: &str) -> Resp {
    send(app, "GET", uri, true, None).await
}

async fn create(app: &Router, endpoint: &str, body: Value) -> Value {
    let res = send(app, "POST", endpoint, true, Some(body)).await;
    assert_eq!(
        res.status,
        StatusCode::CREATED,
        "{}: {}",
        endpoint,
        res.body
    );
    res.json()
}

fn setup() -> (Database, Router) {
    let db = Database::new_in_memory().expect("in-memory db");
    db.set_option("blogname", "Feed Test Site").unwrap();
    db.set_option("siteurl", "https://example.com").unwrap();
    db.set_option("posts_per_rss", "50").unwrap();
    let app = create_app(db.clone());
    (db, app)
}

fn assert_well_formed_xml(xml: &str) {
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!(
                "feed is not well-formed XML at byte {}: {e}\n{xml}",
                reader.buffer_position()
            ),
        }
    }
}

fn utc_in(days: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::days(days))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}

fn ids(list: &Value) -> Vec<i64> {
    list.as_array()
        .map(|a| a.iter().filter_map(|p| p["id"].as_i64()).collect())
        .unwrap_or_default()
}

#[tokio::test]
async fn feed_lists_only_published_posts_with_escaped_fields() {
    let (db, app) = setup();
    let tricky = create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({
            "title": "Fish & Chips: \"5 < 6\" & 'more' >\u{0007}",
            "content": "<p>Crispy <img src=\"/uploads/fish.png\"> batter</p><p>Literal ]]> marker</p>",
            "status": "publish",
            "date": "2001-01-02T10:00:00"
        }),
    )
    .await;
    create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({ "title": "Older Feed Post", "content": "<p>Old</p>", "status": "publish", "date": "2001-01-01T10:00:00" }),
    )
    .await;
    create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({ "title": "Secret Draft Post", "content": "<p>Not yet</p>", "status": "draft" }),
    )
    .await;
    create(
        &app,
        "/wp-json/wp/v2/pages",
        json!({ "title": "About Page Not In Feed", "content": "<p>Page</p>", "status": "publish" }),
    )
    .await;

    let res = get(&app, "/feed/").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.content_type, "application/rss+xml; charset=utf-8");
    let xml = res.body;
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert_well_formed_xml(&xml);
    assert!(xml.contains("<rss version=\"2.0\""));
    assert!(xml.contains("<title>Feed Test Site</title>"));
    assert!(xml.contains("<link>https://example.com</link>"));
    assert!(xml.contains(
        "<atom:link href=\"https://example.com/feed/\" rel=\"self\" type=\"application/rss+xml\" />"
    ));
    assert!(xml.contains("<lastBuildDate>"));
    assert!(xml.contains(
        "<title>Fish &amp; Chips: &quot;5 &lt; 6&quot; &amp; &#39;more&#39; &gt;</title>"
    ));
    assert!(!xml.contains('\u{0007}'), "invalid XML 1.0 char leaked");
    let link = tricky["link"].as_str().unwrap();
    assert!(xml.contains(&format!("<link>{}</link>", link)));
    assert!(xml.contains("<pubDate>Tue, 02 Jan 2001 10:00:00 +0000</pubDate>"));
    assert!(xml.contains("<dc:creator>"));
    assert!(xml.contains("isPermaLink=\"false\">https://example.com/?p="));
    // Post HTML is CDATA with `]]>` split, and root-relative URLs made absolute.
    assert!(xml.contains("]]]]><![CDATA[>"));
    assert!(xml.contains("src=\"https://example.com/uploads/fish.png\""));
    // Drafts and pages stay out; newest first.
    assert!(!xml.contains("Secret Draft Post"));
    assert!(!xml.contains("About Page Not In Feed"));
    let newer = xml.find("Fish &amp; Chips").unwrap();
    let older = xml.find("Older Feed Post").unwrap();
    assert!(newer < older, "items must be newest first");

    let no_slash = get(&app, "/feed").await;
    assert_eq!(no_slash.status, StatusCode::OK);
    assert!(no_slash.content_type.starts_with("application/rss+xml"));

    // Settings → Reading → "Syndication feeds show the most recent N items".
    db.set_option("posts_per_rss", "2").unwrap();
    let limited = get(&app, "/feed/").await.body;
    assert_eq!(limited.matches("<item>").count(), 2);
}

#[tokio::test]
async fn term_feeds_filter_posts_and_routes_still_resolve() {
    let (_db, app) = setup();
    let cat = create(
        &app,
        "/wp-json/wp/v2/categories",
        json!({ "name": "Rust & Friends", "slug": "rust-friends" }),
    )
    .await;
    let tag = create(
        &app,
        "/wp-json/wp/v2/tags",
        json!({ "name": "axum", "slug": "axum" }),
    )
    .await;
    let cat_id = cat["id"].as_i64().unwrap();
    let tag_id = tag["id"].as_i64().unwrap();
    create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({
            "title": "Category Feed Post A",
            "slug": "category-feed-post-a",
            "content": "<p>In the category</p>",
            "status": "publish",
            "categories": [cat_id],
            "tags": [tag_id]
        }),
    )
    .await;
    create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({ "title": "Unrelated Feed Post B", "content": "<p>Elsewhere</p>", "status": "publish" }),
    )
    .await;
    create(
        &app,
        "/wp-json/wp/v2/pages",
        json!({ "title": "Routing Check Page", "slug": "routing-check-page", "content": "<p>Page body</p>", "status": "publish" }),
    )
    .await;

    for uri in [
        "/category/rust-friends/feed/",
        "/category/rust-friends/feed",
    ] {
        let res = get(&app, uri).await;
        assert_eq!(res.status, StatusCode::OK, "{uri}");
        assert_eq!(res.content_type, "application/rss+xml; charset=utf-8");
        assert_well_formed_xml(&res.body);
        assert!(res
            .body
            .contains("<title>Feed Test Site - Rust &amp; Friends</title>"));
        assert!(res
            .body
            .contains("href=\"https://example.com/category/rust-friends/feed/\" rel=\"self\""));
        assert!(res.body.contains("Category Feed Post A"));
        assert!(res.body.contains("<category>Rust &amp; Friends</category>"));
        assert!(res.body.contains("<category>axum</category>"));
        assert!(!res.body.contains("Unrelated Feed Post B"));
    }
    for uri in ["/tag/axum/feed/", "/tag/axum/feed"] {
        let res = get(&app, uri).await;
        assert_eq!(res.status, StatusCode::OK, "{uri}");
        assert_well_formed_xml(&res.body);
        assert!(res.body.contains("<title>Feed Test Site - axum</title>"));
        assert!(res.body.contains("Category Feed Post A"));
        assert!(!res.body.contains("Unrelated Feed Post B"));
    }
    for uri in [
        "/category/no-such-category/feed/",
        "/category/no-such-category/feed",
        "/tag/no-such-tag/feed/",
        "/tag/no-such-tag/feed",
    ] {
        assert_eq!(get(&app, uri).await.status, StatusCode::NOT_FOUND, "{uri}");
    }

    // The feed routes do not shadow archives, posts or pages.
    let archive = get(&app, "/category/rust-friends/").await;
    assert_eq!(archive.status, StatusCode::OK);
    assert!(archive.body.contains("Category Feed Post A"));
    assert!(!archive.body.contains("<?xml"));
    let tag_archive = get(&app, "/tag/axum/").await;
    assert_eq!(tag_archive.status, StatusCode::OK);
    assert!(tag_archive.body.contains("Category Feed Post A"));
    for uri in ["/category-feed-post-a/", "/category-feed-post-a"] {
        let single = get(&app, uri).await;
        assert_eq!(single.status, StatusCode::OK, "{uri}");
        assert!(single.body.contains("Category Feed Post A"));
    }
    let page = get(&app, "/routing-check-page/").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains("Page body"));
}

#[tokio::test]
async fn public_head_advertises_the_rss_feed() {
    let (_db, app) = setup();
    let expected = r#"<link rel="alternate" type="application/rss+xml" title="Feed Test Site &raquo; Feed" href="/feed/" />"#;
    let home = get(&app, "/").await;
    assert_eq!(home.status, StatusCode::OK);
    assert!(home.body.contains(expected), "home head lacks feed link");
    let post = create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({ "title": "Head Link Post", "slug": "head-link-post", "content": "<p>Hi</p>", "status": "publish" }),
    )
    .await;
    assert!(post["id"].as_i64().is_some());
    let single = get(&app, "/head-link-post/").await;
    assert!(
        single.body.contains(expected),
        "single head lacks feed link"
    );
}

#[tokio::test]
async fn post_scheduled_via_rest_stays_hidden_until_its_date() {
    let (_db, app) = setup();
    let when = utc_in(3);
    let cat = create(
        &app,
        "/wp-json/wp/v2/categories",
        json!({ "name": "Launch News", "slug": "launch-news" }),
    )
    .await;
    let tag = create(
        &app,
        "/wp-json/wp/v2/tags",
        json!({ "name": "launch", "slug": "launch" }),
    )
    .await;
    let post = create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({
            "title": "Scheduled Launch Announcement",
            "slug": "scheduled-launch",
            "content": "<p>Zebrafish launch details</p>",
            "status": "publish",
            "date": when,
            "categories": [cat["id"]],
            "tags": [tag["id"]]
        }),
    )
    .await;
    assert_eq!(post["status"], "future", "publish + future date schedules");
    assert_eq!(post["date"], when.as_str());
    assert_eq!(post["date_gmt"], when.as_str());
    let id = post["id"].as_i64().unwrap();

    // Explicit `future` is accepted; `future` with a past date publishes.
    let explicit = create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({ "title": "Explicit Future", "content": "x", "status": "future", "date_gmt": utc_in(5) }),
    )
    .await;
    assert_eq!(explicit["status"], "future");
    let backdated = create(
        &app,
        "/wp-json/wp/v2/posts",
        json!({ "title": "Backdated", "content": "x", "status": "future", "date": "2020-01-01T00:00:00" }),
    )
    .await;
    assert_eq!(backdated["status"], "publish");
    let page = create(
        &app,
        "/wp-json/wp/v2/pages",
        json!({ "title": "Scheduled Landing Page", "slug": "scheduled-landing", "content": "<p>Soon</p>", "status": "publish", "date": when }),
    )
    .await;
    assert_eq!(page["status"], "future");
    let page_id = page["id"].as_i64().unwrap();
    let bad = send(
        &app,
        "POST",
        "/wp-json/wp/v2/posts",
        true,
        Some(json!({ "title": "Bad date", "content": "x", "date": "next tuesday" })),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);

    // Hidden from every public HTML path.
    let title = "Scheduled Launch Announcement";
    for uri in [
        "/",
        "/category/launch-news/",
        "/tag/launch/",
        "/author/admin/",
        "/search?s=Zebrafish",
        "/?s=Zebrafish",
    ] {
        let res = get(&app, uri).await;
        assert!(!res.body.contains(title), "{uri} leaked the scheduled post");
    }
    assert_eq!(
        get(&app, "/scheduled-launch/").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, &format!("/?p={id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, "/scheduled-landing/").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&app, &format!("/?page_id={page_id}")).await.status,
        StatusCode::NOT_FOUND
    );
    // Feeds and sitemap.
    for uri in ["/feed/", "/category/launch-news/feed/", "/tag/launch/feed/"] {
        let res = get(&app, uri).await;
        assert_eq!(res.status, StatusCode::OK, "{uri}");
        assert!(!res.body.contains(title), "{uri} leaked the scheduled post");
    }
    let sitemap = get(&app, "/sitemap.xml").await.body;
    assert!(!sitemap.contains("scheduled-launch"));
    assert!(!sitemap.contains("scheduled-landing"));
    // Unauthenticated REST reads.
    for uri in [
        "/wp-json/wp/v2/posts?per_page=100",
        "/wp-json/wp/v2/posts?per_page=100&status=future",
        "/wp-json/wp/v2/posts?slug=scheduled-launch",
    ] {
        assert!(!ids(&get(&app, uri).await.json()).contains(&id), "{uri}");
    }
    assert_eq!(
        get(&app, &format!("/wp-json/wp/v2/posts/{id}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert!(!ids(&get(&app, "/wp-json/wp/v2/pages?per_page=100").await.json()).contains(&page_id));
    assert_eq!(
        get(&app, &format!("/wp-json/wp/v2/pages/{page_id}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert!(!ids(&get(&app, "/wp-json/wp/v2/search?search=Scheduled")
        .await
        .json())
    .contains(&id));

    // Editors still see it.
    let future_list = get_auth(&app, "/wp-json/wp/v2/posts?status=future&per_page=100").await;
    assert!(ids(&future_list.json()).contains(&id));
    let any_list = get_auth(&app, "/wp-json/wp/v2/posts?status=any&per_page=100").await;
    assert!(ids(&any_list.json()).contains(&id));
    let single = get_auth(&app, &format!("/wp-json/wp/v2/posts/{id}")).await;
    assert_eq!(single.status, StatusCode::OK);
    assert_eq!(single.json()["status"], "future");
    let admin_list = get_auth(&app, "/n-admin/edit").await;
    assert_eq!(admin_list.status, StatusCode::OK);
    assert!(admin_list.body.contains(title));
    assert!(admin_list.body.contains(">Scheduled</span>"));
    assert!(admin_list.body.contains("Scheduled (2)"));
    let editor = get_auth(&app, &format!("/n-admin/post?post={id}&action=edit")).await;
    assert_eq!(editor.status, StatusCode::OK);
    assert!(editor
        .body
        .contains(r#"<option value="future" selected>Scheduled</option>"#));
    assert!(editor
        .body
        .contains("Scheduled for <time id=\"post_schedule_label\""));
    assert!(editor
        .body
        .contains(&format!("data-scheduled-gmt=\"{when}\"")));

    // Moving the date into the past publishes it.
    let updated = send(
        &app,
        "PUT",
        &format!("/wp-json/wp/v2/posts/{id}"),
        true,
        Some(json!({ "date_gmt": "2020-06-01T12:00:00" })),
    )
    .await;
    assert_eq!(updated.status, StatusCode::OK);
    assert_eq!(updated.json()["status"], "publish");
    assert_eq!(get(&app, "/scheduled-launch/").await.status, StatusCode::OK);
    assert!(get(&app, "/feed/").await.body.contains(title));

    // And moving a published post's date ahead schedules it again.
    let rescheduled = send(
        &app,
        "PUT",
        &format!("/wp-json/wp/v2/posts/{id}"),
        true,
        Some(json!({ "status": "publish", "date": utc_in(2) })),
    )
    .await;
    assert_eq!(rescheduled.json()["status"], "future");
    assert_eq!(
        get(&app, "/scheduled-launch/").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn due_scheduled_posts_are_promoted_on_the_next_request() {
    let (db, app) = setup();
    // Imported scheduled posts keep their `future` status and date.
    let bundle = serde_json::from_value(json!({
        "posts": [
            { "title": "Imported Due Post", "slug": "imported-due-post", "content": "<p>Due now</p>", "status": "future", "date": "2020-05-01 08:00:00" },
            { "title": "Imported Later Post", "slug": "imported-later-post", "content": "<p>Later</p>", "status": "future", "date": "2099-05-01 08:00:00" }
        ]
    }))
    .unwrap();
    db.import_bundle(&bundle).unwrap();
    let due = db.get_post_by_slug("imported-due-post").unwrap().unwrap();
    assert_eq!(due.status, "future", "nothing promoted before a request");

    let feed = get(&app, "/feed/").await;
    assert!(feed.body.contains("Imported Due Post"));
    assert!(!feed.body.contains("Imported Later Post"));
    let due = db.get_post_by_slug("imported-due-post").unwrap().unwrap();
    assert_eq!(due.status, "publish");
    let later = db.get_post_by_slug("imported-later-post").unwrap().unwrap();
    assert_eq!(later.status, "future");

    assert_eq!(
        get(&app, "/imported-due-post/").await.status,
        StatusCode::OK
    );
    assert_eq!(
        get(&app, "/imported-later-post/").await.status,
        StatusCode::NOT_FOUND
    );
    let listed = ids(&get(&app, "/wp-json/wp/v2/posts?per_page=100").await.json());
    assert!(listed.contains(&due.id));
    assert!(!listed.contains(&later.id));
    assert!(get(&app, "/sitemap.xml")
        .await
        .body
        .contains("imported-due-post"));
    assert_eq!(db.publish_due_scheduled_posts().unwrap(), 0);
}

#[tokio::test]
async fn editor_offers_schedule_option_with_datetime_picker() {
    let (_db, app) = setup();
    let res = get_auth(&app, "/n-admin/post-new").await;
    assert_eq!(res.status, StatusCode::OK);
    let html = res.body;
    assert!(html.contains(r#"<option value="future" >Scheduled</option>"#));
    assert!(html.contains(r#"<input type="datetime-local" id="post_schedule_at""#));
    assert!(html.contains(r#"id="post_schedule_group" data-initial-status="""#));
    assert!(html.contains("np-publish-btn"));
    // The picked time is sent to the REST API as `date_gmt` with the save request.
    assert!(html.contains("function npScheduleOverrides(status, quiet)"));
    assert!(html.contains("return { date_gmt: npUtcIso(d) };"));
    assert!(html.contains("...scheduleFields"));
    assert!(html.contains("'Schedule Post'"));

    let statuses = get(&app, "/wp-json/wp/v2/statuses").await.json();
    assert_eq!(statuses["future"]["name"], "Scheduled");
}
