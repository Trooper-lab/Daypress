//! Integration tests for:
//! 1. Sidebar icons visibility in `/d-admin/*`
//! 2. Post revisions (automatic creation, REST listing, restore, delete, and bulk cleanup tool)
//! 3. Atom 1.0 feeds & RSS/Atom comment feeds (`/feed/atom/`, `/comments/feed/`, `/comments/feed/atom/`, `/{slug}/feed/`)
//! 4. Site timezone support (`timezone_string`, `gmt_offset`, naive local date conversion to `date_gmt`)
//! 5. Complete WordPress-parity User Profile (`/d-admin/profile`, usermeta persistence, session invalidation, application passwords)

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use daypress_server::{create_app, Database};
use serde_json::{json, Value};
use tower::ServiceExt;

const ADMIN_BASIC: &str = "Basic YWRtaW46YWRtaW4=";

fn admin_req(method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", ADMIN_BASIC);
    match body {
        Some(val) => builder.body(Body::from(val.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

fn anon_get(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn body_text(res: axum::response::Response) -> String {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[tokio::test]
async fn test_sidebar_icons_visible_in_admin_shell() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db);

    let res = app
        .clone()
        .oneshot(admin_req("GET", "/d-admin/", None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let html = body_text(res).await;

    assert!(
        !html.contains(".nav-icon { display: none; }"),
        "sidebar .nav-icon must not be hidden"
    );
    assert!(
        html.contains(".nav-icon {") && html.contains("width: 16px;"),
        "sidebar .nav-icon must have explicit dimensions"
    );
    assert!(
        html.contains("<svg class=\"nav-icon\""),
        "sidebar nav items must render SVG icons"
    );
}

#[tokio::test]
async fn test_post_revisions_lifecycle_and_cleaning_tool() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db);

    // 1. Create a post (v1)
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/wp/v2/posts",
            Some(json!({
                "title": "Revision Test v1",
                "slug": "revision-test",
                "content": "<p>Original body v1</p>",
                "status": "publish"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created: Value = serde_json::from_str(&body_text(res).await).unwrap();
    let post_id = created["id"].as_i64().unwrap();

    // 2. Update the post twice (v2, v3) -> creates 2 revision snapshots
    for (ver, text) in [("v2", "<p>Body v2</p>"), ("v3", "<p>Body v3</p>")] {
        let res = app
            .clone()
            .oneshot(admin_req(
                "PUT",
                &format!("/wp-json/wp/v2/posts/{post_id}"),
                Some(json!({
                    "title": format!("Revision Test {ver}"),
                    "content": text,
                    "status": "publish"
                })),
            ))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    // 3. List revisions via REST API
    let res = app
        .clone()
        .oneshot(admin_req(
            "GET",
            &format!("/wp-json/wp/v2/posts/{post_id}/revisions"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let revs: Vec<Value> = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(revs.len(), 2, "two updates must create two revisions");

    // Oldest revision is v1 (last in DESC list)
    let v1_rev_id = revs.last().unwrap()["id"].as_i64().unwrap();
    assert_eq!(
        revs.last().unwrap()["title"]["rendered"],
        "Revision Test v1"
    );

    // 4. Restore v1 revision
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            &format!("/wp-json/wp/v2/posts/{post_id}/revisions/{v1_rev_id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let restored: Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(restored["title"]["rendered"], "Revision Test v1");
    assert!(restored["content"]["rendered"]
        .as_str()
        .unwrap_or("")
        .contains("Original body v1"));

    // 5. Clean revisions keeping only 1 per post
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/daypress/v1/revisions/clean",
            Some(json!({ "keep_per_post": 1 })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let cleaned: Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert!(cleaned["deleted_count"].as_i64().unwrap() >= 1);
    assert_eq!(cleaned["remaining_count"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn test_atom_and_comments_feeds_and_head_autodiscovery() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db);

    // Create a post and an approved comment
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/wp/v2/posts",
            Some(json!({
                "title": "Feed Verification Post",
                "slug": "feed-verification-post",
                "content": "<p>Syndicated content body</p>",
                "status": "publish"
            })),
        ))
        .await
        .unwrap();
    let post: Value = serde_json::from_str(&body_text(res).await).unwrap();
    let post_id = post["id"].as_i64().unwrap();

    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/wp/v2/comments",
            Some(json!({
                "post": post_id,
                "author_name": "Ada Lovelace",
                "author_email": "ada@example.com",
                "content": "First analytical comment!",
                "status": "approved"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 1. Atom 1.0 Feed (/feed/atom/)
    let res = app.clone().oneshot(anon_get("/feed/atom/")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let ctype = res
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ctype.contains("application/atom+xml"));
    let atom_xml = body_text(res).await;
    assert!(atom_xml.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\""));
    assert!(atom_xml.contains("Feed Verification Post"));

    // 2. Global Comments RSS 2.0 Feed (/comments/feed/)
    let res = app
        .clone()
        .oneshot(anon_get("/comments/feed/"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let comments_rss = body_text(res).await;
    assert!(comments_rss.contains("<rss version=\"2.0\""));
    assert!(comments_rss.contains("Ada Lovelace"));
    assert!(comments_rss.contains("First analytical comment!"));

    // 3. Global Comments Atom Feed (/comments/feed/atom/)
    let res = app
        .clone()
        .oneshot(anon_get("/comments/feed/atom/"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let comments_atom = body_text(res).await;
    assert!(comments_atom.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\""));
    assert!(comments_atom.contains("First analytical comment!"));

    // 4. Single Post Comments Feed (/{slug}/feed/)
    let res = app
        .clone()
        .oneshot(anon_get("/feed-verification-post/feed/"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let single_feed = body_text(res).await;
    assert!(single_feed.contains("Comments on:"));
    assert!(single_feed.contains("First analytical comment!"));

    // 5. Public <head> feed discovery links
    let res = app
        .clone()
        .oneshot(anon_get("/feed-verification-post/"))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let page_html = body_text(res).await;
    assert!(page_html.contains("href=\"/feed/\""));
    assert!(page_html.contains("href=\"/feed/atom/\""));
    assert!(page_html.contains("href=\"/comments/feed/\""));
    assert!(page_html.contains("href=\"/feed-verification-post/feed/\""));
}

#[tokio::test]
async fn test_site_timezone_settings_and_local_to_gmt_conversion() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db);

    // 1. Set site timezone to UTC+02:00
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/wp/v2/settings",
            Some(json!({
                "timezone_string": "UTC+02:00"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let settings: Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(settings["timezone"], "UTC+02:00");

    // 2. Check /wp-json root advertises timezone_string & gmt_offset
    let res = app.clone().oneshot(anon_get("/wp-json")).await.unwrap();
    let root: Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(root["timezone_string"], "UTC+02:00");
    assert_eq!(root["gmt_offset"].as_f64().unwrap(), 2.0);

    // 3. Create a post with a naive local date (14:30:00 in UTC+02:00 -> 12:30:00 UTC)
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/wp/v2/posts",
            Some(json!({
                "title": "Timezone Aware Post",
                "slug": "tz-aware-post",
                "content": "<p>Scheduled in local time</p>",
                "status": "publish",
                "date": "2025-05-10T14:30:00"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created: Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(created["date"], "2025-05-10T14:30:00");
    assert_eq!(created["date_gmt"], "2025-05-10T12:30:00");
}

#[tokio::test]
async fn test_complete_user_profile_settings_and_session_invalidation() {
    let db = Database::new_in_memory().unwrap();
    let old_cookie = daypress_server::auth::session_cookie_for(1, &db);
    let app = create_app(db);

    // 1. Update all WordPress user profile fields via /wp-json/wp/v2/users/me
    let res = app
        .clone()
        .oneshot(admin_req(
            "POST",
            "/wp-json/wp/v2/users/me",
            Some(json!({
                "first_name": "Grace",
                "last_name": "Hopper",
                "nickname": "Admiral",
                "display_name": "Grace Hopper",
                "description": "Pioneer of compiler engineering.",
                "locale": "nl_NL",
                "rich_editing": false,
                "syntax_highlighting": false,
                "comment_shortcuts": true,
                "admin_color": "midnight",
                "show_admin_bar_front": false,
                "avatar_url": "/uploads/grace.png"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let updated: Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(updated["first_name"], "Grace");
    assert_eq!(updated["last_name"], "Hopper");
    assert_eq!(updated["nickname"], "Admiral");
    assert_eq!(updated["display_name"], "Grace Hopper");
    assert_eq!(updated["description"], "Pioneer of compiler engineering.");
    assert_eq!(updated["locale"], "nl_NL");
    assert_eq!(updated["rich_editing"], false);
    assert_eq!(updated["syntax_highlighting"], false);
    assert_eq!(updated["comment_shortcuts"], true);
    assert_eq!(updated["admin_color"], "midnight");
    assert_eq!(updated["show_admin_bar_front"], false);
    assert_eq!(updated["avatar_url"], "/uploads/grace.png");

    // 2. Verify /d-admin/profile renders the saved settings and admin_color
    let res = app
        .clone()
        .oneshot(admin_req("GET", "/d-admin/profile", None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let profile_html = body_text(res).await;
    assert!(profile_html.contains("data-admin-color=\"midnight\""));
    assert!(profile_html.contains("Pioneer of compiler engineering."));
    assert!(profile_html.contains("Log Out Everywhere Else"));

    // 3. Verify old session cookie works before destroy-others, and is rejected after destroy-others
    let req_before = Request::builder()
        .method("GET")
        .uri("/wp-json/wp/v2/users/me")
        .header(header::COOKIE, &old_cookie)
        .body(Body::empty())
        .unwrap();
    let res_before = app.clone().oneshot(req_before).await.unwrap();
    assert_eq!(res_before.status(), StatusCode::OK);

    let destroy_req = Request::builder()
        .method("POST")
        .uri("/wp-json/daypress/v1/users/1/sessions/destroy-others")
        .header(header::COOKIE, &old_cookie)
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();
    let destroy_res = app.clone().oneshot(destroy_req).await.unwrap();
    assert_eq!(destroy_res.status(), StatusCode::OK);
    let set_cookie = destroy_res
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let new_cookie = set_cookie.split(';').next().unwrap().to_string();

    // Old cookie must now be rejected (401)
    let req_old = Request::builder()
        .method("GET")
        .uri("/wp-json/wp/v2/users/me")
        .header(header::COOKIE, &old_cookie)
        .body(Body::empty())
        .unwrap();
    let res_old = app.clone().oneshot(req_old).await.unwrap();
    assert_eq!(res_old.status(), StatusCode::UNAUTHORIZED);

    // New cookie returned by destroy-others must stay authenticated (200)
    let req_new = Request::builder()
        .method("GET")
        .uri("/wp-json/wp/v2/users/me")
        .header(header::COOKIE, &new_cookie)
        .body(Body::empty())
        .unwrap();
    let res_new = app.clone().oneshot(req_new).await.unwrap();
    assert_eq!(res_new.status(), StatusCode::OK);
}
