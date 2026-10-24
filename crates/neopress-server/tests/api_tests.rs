use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use base64::Engine;
use neopress_server::{create_app, Database};
use tower::ServiceExt;

#[tokio::test]
async fn test_api_root_endpoint() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/wp-json/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "WordPress Reference");
    assert!(json["namespaces"]
        .as_array()
        .unwrap()
        .iter()
        .any(|ns| ns == "wp/v2"));
}

#[tokio::test]
async fn test_api_posts_endpoints() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    // List posts
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/posts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let posts: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(posts.as_array().unwrap().len() >= 2);

    // Create post via API
    let new_post_json = serde_json::json!({
        "title": "API Created Post",
        "content": "<p>Content from HTTP POST</p>",
        "status": "publish",
        "slug": "api-created-post"
    });

    // Verify unauthenticated write fails with 401 Unauthorized
    let unauth_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_post_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauth_response.status(), StatusCode::UNAUTHORIZED);
    let unauth_body = to_bytes(unauth_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let unauth_err: serde_json::Value = serde_json::from_slice(&unauth_body).unwrap();
    assert_eq!(unauth_err["code"], "rest_cannot_create");
    assert_eq!(unauth_err["data"]["status"], 401);

    // Authenticated write via Basic Auth (admin:admin)
    let auth_header = format!(
        "Basic {}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "admin:admin")
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_post_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["title"]["rendered"], "API Created Post");
    assert_eq!(created["slug"], "api-created-post");
    let created_id = created["id"].as_i64().unwrap();

    // Get created post by ID
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/wp-json/wp/v2/posts/{}", created_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Update post via PUT (authenticated)
    let update_json = serde_json::json!({
        "title": "Updated API Post Title"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/wp-json/wp/v2/posts/{}", created_id))
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Delete post via DELETE (authenticated)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/wp-json/wp/v2/posts/{}", created_id))
                .header("Authorization", &auth_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_not_found_standard_error_format() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/posts/999999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(err["code"], "rest_not_found");
    assert_eq!(err["data"]["status"], 404);
}

#[tokio::test]
async fn test_public_ssr_rendering() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    // Home page
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Hello world!"));
    assert!(html.contains("Neopress"));

    // Single post page
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/hello-world/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Welcome to WordPress"));
    assert!(!html.contains("id=\"wpadminbar\""));
    assert!(html.contains("Log In"));

    // Logged-in user on home page (renders topbar)
    let logged_in_home = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Cookie", "neopress_admin=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logged_in_home.status(), StatusCode::OK);
    let home_body = to_bytes(logged_in_home.into_body(), usize::MAX)
        .await
        .unwrap();
    let home_html = String::from_utf8_lossy(&home_body);
    assert!(home_html.contains("id=\"wpadminbar\""));
    assert!(home_html.contains("admin-bar"));
    assert!(home_html.contains("Howdy, <span class=\"display-name\">admin</span>"));
    assert!(home_html.contains("Customize"));
    assert!(home_html.contains("Dashboard"));

    // Logged-in user on single post (renders topbar with Edit with Visual Builder)
    let logged_in_single = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/hello-world/")
                .header("Cookie", "neopress_admin=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logged_in_single.status(), StatusCode::OK);
    let single_body = to_bytes(logged_in_single.into_body(), usize::MAX)
        .await
        .unwrap();
    let single_html = String::from_utf8_lossy(&single_body);
    assert!(single_html.contains("id=\"wpadminbar\""));
    assert!(single_html.contains("Edit with Visual Builder"));
    assert!(single_html.contains("/n-admin/builder?post=1"));
}

#[tokio::test]
async fn test_admin_dashboard_rendering() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/n-admin/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Dashboard"));
    assert!(html.contains("Total Posts"));
}

#[tokio::test]
async fn test_authentication_application_passwords_and_jwt() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);
    let basic_admin = format!(
        "Basic {}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "admin:admin")
    );

    // 1. Nonce Authentication (X-WP-Nonce)
    let nonce_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/neopress/v1/auth/nonce")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(nonce_res.status(), StatusCode::OK);
    let body = to_bytes(nonce_res.into_body(), usize::MAX).await.unwrap();
    let nonce_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let nonce = nonce_json["nonce"].as_str().unwrap();

    let post_with_nonce = serde_json::json!({
        "title": "Post Created with Nonce",
        "content": "<p>Created via X-WP-Nonce header</p>",
        "status": "publish"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("X-WP-Nonce", nonce)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&post_with_nonce).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 2. Application Passwords API (WordPress 5.6+)
    let app_pass_req = serde_json::json!({
        "name": "Headless Frontend Client",
        "app_id": "nextjs-dashboard"
    });
    let create_app_pass_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/users/1/application-passwords")
                .header("Authorization", &basic_admin)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&app_pass_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_app_pass_res.status(), StatusCode::CREATED);
    let body = to_bytes(create_app_pass_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let app_pass_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let raw_app_pass = app_pass_json["password"].as_str().unwrap();

    // Authenticate write using the newly generated Application Password
    let app_pass_auth = format!(
        "Basic {}",
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("admin:{}", raw_app_pass)
        )
    );
    let post_with_app_pass = serde_json::json!({
        "title": "Post Created with App Password",
        "content": "<p>Authenticated via standard WP Application Password</p>",
        "status": "publish"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Authorization", &app_pass_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&post_with_app_pass).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // 3. JWT Authentication (jwt-auth standard)
    let jwt_req = serde_json::json!({
        "username": "admin",
        "password": "admin"
    });
    let token_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/jwt-auth/v1/token")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&jwt_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(token_res.status(), StatusCode::OK);
    let body = to_bytes(token_res.into_body(), usize::MAX).await.unwrap();
    let token_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = token_json["token"].as_str().unwrap();

    // Validate JWT
    let validate_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/jwt-auth/v1/token/validate")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(validate_res.status(), StatusCode::OK);

    // Write using Bearer JWT
    let post_with_jwt = serde_json::json!({
        "title": "Post Created with Bearer JWT",
        "content": "<p>Authenticated via JWT token</p>",
        "status": "publish"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&post_with_jwt).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_media_asset_handling_and_dynamic_crop() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);
    let basic_admin = format!(
        "Basic {}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "admin:admin")
    );

    // Valid 2x2 PNG binary
    let png_bytes: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x08, 0x02, 0x00, 0x00, 0x00, 0xFD,
        0xD4, 0x9A, 0x73, 0x00, 0x00, 0x00, 0x10, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let boundary = "---------------------------boundary987654321";
    let mut form_body = Vec::new();
    form_body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    form_body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"test-image.png\"\r\n",
    );
    form_body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    form_body.extend_from_slice(png_bytes);
    form_body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/media")
                .header("Authorization", &basic_admin)
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={}", boundary),
                )
                .body(Body::from(form_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let media_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(media_json["mime_type"], "image/png");
    assert_eq!(media_json["media_type"], "image");
    assert!(media_json["media_details"]["sizes"].is_object());

    // Verify static serving and Cache-Control
    let source_url = media_json["source_url"].as_str().unwrap();
    let get_media = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(source_url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_media.status(), StatusCode::OK);
    assert_eq!(
        get_media
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap(),
        "public, max-age=31536000, immutable"
    );
}

#[tokio::test]
async fn test_wxr_xml_stream_ingestion_api() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);
    let basic_admin = format!(
        "Basic {}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "admin:admin")
    );

    let wxr_payload = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wfw="http://wellformedweb.org/CommentAPI/"
	xmlns:dc="http://purl.org/dc/elements/1.1/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>WXR Ingested Site</title>
	<description>Streamed WordPress XML Import</description>
	<wp:category>
		<wp:term_id>10</wp:term_id>
		<wp:category_nicename>cloud-native</wp:category_nicename>
		<wp:cat_name>Cloud Native</wp:cat_name>
	</wp:category>
	<item>
		<title>Streamed Article via WXR</title>
		<wp:post_name>streamed-article-via-wxr</wp:post_name>
		<content:encoded><![CDATA[<p>Zero-friction drag and drop migration accomplished!</p>]]></content:encoded>
		<wp:post_date>2026-10-21 21:00:00</wp:post_date>
		<wp:status>publish</wp:status>
		<wp:post_type>post</wp:post_type>
		<category domain="category" nicename="cloud-native"><![CDATA[Cloud Native]]></category>
		<wp:comment>
			<wp:comment_author>Migrator Bot</wp:comment_author>
			<wp:comment_content>Welcome from WXR stream!</wp:comment_content>
		</wp:comment>
	</item>
</channel>
</rss>"#;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/neopress/v1/import-wxr")
                .header("Authorization", &basic_admin)
                .header("Content-Type", "application/xml")
                .body(Body::from(wxr_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let import_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(import_json["success"], true);
    assert_eq!(import_json["imported_posts"], 1);

    // Verify imported post is visible via standard REST API
    let get_posts = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/posts?slug=streamed-article-via-wxr")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_posts.status(), StatusCode::OK);
    let posts_body = to_bytes(get_posts.into_body(), usize::MAX).await.unwrap();
    let posts_arr: serde_json::Value = serde_json::from_slice(&posts_body).unwrap();
    assert_eq!(posts_arr.as_array().unwrap().len(), 1);
    assert_eq!(
        posts_arr[0]["title"]["rendered"],
        "Streamed Article via WXR"
    );
}

#[tokio::test]
async fn test_user_auth_login_logout_and_management() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    // 1. GET /wp-login.php returns 200 OK with login form
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-login.php")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Log In"));
    assert!(html.contains("user_login"));

    // 2. POST /wp-login.php with invalid credentials shows error
    let bad_login = "log=admin&pwd=wrongpassword";
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-login.php")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from(bad_login))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Unknown username or incorrect password"));

    // 3. POST /wp-login.php with valid credentials sets cookie and redirects
    let good_login = "log=admin&pwd=admin&redirect_to=/n-admin/";
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-login.php")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from(good_login))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(res.headers().get("location").unwrap(), "/n-admin/");
    let cookie_val = res.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(cookie_val.contains("neopress_admin=1"));
    assert!(cookie_val.contains("HttpOnly"));

    // 4. GET /wp-login.php?action=logout clears cookie and redirects to loggedout=true
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-login.php?action=logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        res.headers().get("location").unwrap(),
        "/wp-login.php?loggedout=true"
    );
    let cookie_clear = res.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(cookie_clear.contains("Max-Age=0"));

    // 5. GET /wp-login.php?loggedout=true shows logged out banner
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-login.php?loggedout=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("You are now logged out"));

    // 6. View Admin Users and Add New User pages
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/users.php")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Users"));
    assert!(html.contains("+ Add New User"));

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/user-new.php")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Add New User"));
    assert!(html.contains("newUsername"));

    // 7. Add a new user via API (POST /wp-json/wp/v2/users)
    let new_user_json = serde_json::json!({
        "username": "sarah_editor",
        "email": "sarah@example.com",
        "password": "editorpassword123",
        "first_name": "Sarah",
        "last_name": "Connor",
        "display_name": "Sarah Connor",
        "role": "Editor"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/users")
                .header("Cookie", "neopress_admin=1")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_user_json).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let user_obj: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let created_user_id = user_obj["id"].as_i64().unwrap();
    assert_eq!(user_obj["username"], "sarah_editor");
    assert_eq!(user_obj["role"], "Editor");

    // 8. Log in as the newly created user
    let user_login = "log=sarah_editor&pwd=editorpassword123";
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-login.php")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from(user_login))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    let cookie_val = res.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(cookie_val.contains(&format!("neopress_admin={}", created_user_id)));

    // 9. New user appears on /n-admin/users.php list
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/users.php")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("sarah_editor"));
    assert!(html.contains("sarah@example.com"));

    // 10. Delete the newly created user
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/wp-json/wp/v2/users/{}", created_user_id))
                .header("Cookie", "neopress_admin=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_all_wp_admin_menu_routes_rendering() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    // List of all 32 default WordPress menu & submenu route URLs
    let menu_routes = vec![
        // 1. Dashboard
        ("/n-admin/index.php", "Dashboard"),
        ("/n-admin/update-core.php", "WordPress Updates"),
        // 2. Posts
        ("/n-admin/edit.php", "Posts"),
        ("/n-admin/post-new.php", "Add New"),
        ("/n-admin/edit-tags.php?taxonomy=category", "Categories"),
        ("/n-admin/edit-tags.php?taxonomy=post_tag", "Tags"),
        // 3. Media
        ("/n-admin/upload.php", "Media Library"),
        ("/n-admin/media-new.php", "Upload New Media"),
        // 4. Pages
        ("/n-admin/edit.php?post_type=page", "Pages"),
        ("/n-admin/post-new.php?post_type=page", "Add New"),
        // 5. Comments
        ("/n-admin/edit-comments.php", "Comments"),
        // 6. Appearance
        ("/n-admin/themes.php", "Themes"),
        ("/n-admin/customize.php", "Customizer"),
        ("/n-admin/widgets.php", "Widgets"),
        ("/n-admin/nav-menus.php", "Menus"),
        ("/n-admin/theme-editor.php", "Theme File Editor"),
        // 7. Plugins
        ("/n-admin/plugins.php", "Plugins"),
        ("/n-admin/plugin-install.php", "Add Plugins"),
        ("/n-admin/plugin-editor.php", "Plugin File Editor"),
        // 8. Users
        ("/n-admin/users.php", "Users"),
        ("/n-admin/user-new.php", "Add New User"),
        ("/n-admin/profile.php", "Profile"),
        // 9. Tools
        ("/n-admin/tools.php", "Tools"),
        ("/n-admin/import.php", "Import"),
        ("/n-admin/export.php", "Export"),
        ("/n-admin/site-health.php", "Site Health"),
        ("/n-admin/export-personal-data.php", "Export Personal Data"),
        ("/n-admin/erase-personal-data.php", "Erase Personal Data"),
        // 10. Settings
        ("/n-admin/options-general.php", "General Settings"),
        ("/n-admin/options-writing.php", "Writing Settings"),
        ("/n-admin/options-reading.php", "Reading Settings"),
        ("/n-admin/options-discussion.php", "Discussion Settings"),
        ("/n-admin/options-media.php", "Media Settings"),
        ("/n-admin/options-permalink.php", "Permalink Settings"),
        ("/n-admin/options-privacy.php", "Privacy Settings"),
    ];

    for (url, expected_keyword) in menu_routes {
        let res = app
            .clone()
            .oneshot(Request::builder().uri(url).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(
            res.status(),
            StatusCode::OK,
            "Failed GET on menu route: {}",
            url
        );

        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(
            html.contains(expected_keyword),
            "Expected keyword '{}' not found in response for {}",
            expected_keyword,
            url
        );
        // Ensure the two-tier sidebar navigation shell is present
        assert!(
            html.contains("id=\"neo-sidebar\""),
            "Sidebar missing in {}",
            url
        );
    }
}

#[tokio::test]
async fn test_admin_interactive_apis_and_tools() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    // 1. Comments REST API moderation
    // Verify initial comment listing
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/comments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let comments: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(!comments.as_array().unwrap().is_empty());
    let comment_id = comments[0]["id"].as_i64().unwrap();

    // Single comment GET
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/wp-json/wp/v2/comments/{}", comment_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Update status to spam
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/wp-json/wp/v2/comments/{}", comment_id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({ "status": "spam" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Delete comment
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/wp-json/wp/v2/comments/{}?force=true", comment_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Settings Save Endpoints
    // Writing
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/options-writing")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "default_category": 1,
                        "default_post_format": "status",
                        "use_smilies": true,
                        "default_pingback_flag": true,
                        "mailserver_url": "mail.test.local",
                        "mailserver_login": "admin@test.local",
                        "mailserver_pass": "pass123",
                        "mailserver_port": 110
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Reading
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/options-reading")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "show_on_front": "posts",
                        "page_on_front": 0,
                        "page_for_posts": 0,
                        "posts_per_page": 12,
                        "posts_per_rss": 10,
                        "blog_public": true
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Discussion
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/options-discussion")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "default_ping_status": "open",
                        "default_comment_status": "open",
                        "require_name_email": true,
                        "comment_registration": false,
                        "close_comments_for_old_posts": false,
                        "close_comments_days_old": 14,
                        "thread_comments": true,
                        "thread_comments_depth": 5,
                        "comment_moderation": false,
                        "comment_previously_approved": true,
                        "show_avatars": true
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Media
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/options-media")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "thumbnail_size_w": 180,
                        "thumbnail_size_h": 180,
                        "thumbnail_crop": true,
                        "medium_size_w": 350,
                        "medium_size_h": 350,
                        "large_size_w": 1024,
                        "large_size_h": 1024,
                        "uploads_use_yearmonth_folders": true
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Permalinks
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/options-permalink")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "permalink_structure": "/%postname%/",
                        "category_base": "cat",
                        "tag_base": "tag"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Privacy
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/options-privacy")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "wp_page_for_privacy_policy": 2
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Tools Export & Personal Data APIs
    // WXR export download
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/export-wxr")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "application/xml; charset=utf-8"
    );
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let xml = String::from_utf8_lossy(&body);
    assert!(xml.contains("<rss version=\"2.0\""));

    // JSON export download
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/export-json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "application/json; charset=utf-8"
    );

    // Export Personal Data API
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/api/export-personal-data?user=admin@example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Erase Personal Data API
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/erase-personal-data")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "identifier": "admin@example.com"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_wp_admin_redirects_compatibility() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let app = create_app(db);

    // 1. /wp-admin redirects to /n-admin/
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-admin")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(res.headers().get("location").unwrap(), "/n-admin/");

    // 2. /wp-admin/edit.php redirects to /n-admin/edit.php
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-admin/edit.php")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(res.headers().get("location").unwrap(), "/n-admin/edit.php");

    // 3. /wp-admin/options-general.php redirects to /n-admin/options-general.php
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-admin/options-general.php")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        res.headers().get("location").unwrap(),
        "/n-admin/options-general.php"
    );
}

#[tokio::test]
async fn test_visual_builder_layout_endpoints() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());
    let admin_auth = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("admin:admin")
    );

    // 1. GET default layout for Post 1
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/neopress/v1/layout/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let layout_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(layout_json["post_id"], 1);
    assert_eq!(layout_json["layout"]["version"], 1);

    // 2. POST save layout for Post 1
    let save_payload = serde_json::json!({
        "layout": {
            "version": 1,
            "root": {
                "id": "sec_custom_1",
                "type": "Section",
                "styles": { "padding": { "top": "40px" } },
                "children": []
            }
        },
        "rendered_html": "<section class=\"custom-sec\"><p>Compiled Leptos Layout</p></section>"
    });

    let save_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/neopress/v1/layout/1")
                .header("Authorization", &admin_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(save_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save_res.status(), StatusCode::OK);

    // 3. GET updated layout
    let get_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/neopress/v1/layout/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
    let get_body = to_bytes(get_res.into_body(), usize::MAX).await.unwrap();
    let updated_json: serde_json::Value = serde_json::from_slice(&get_body).unwrap();
    assert_eq!(updated_json["layout"]["root"]["id"], "sec_custom_1");

    // 4. Verify public SSR returns the compiled pre-rendered HTML without JS overhead
    let ssr_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/hello-world/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ssr_res.status(), StatusCode::OK);
    let ssr_html = String::from_utf8(
        to_bytes(ssr_res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(ssr_html.contains("Compiled Leptos Layout"));

    // 5. Verify clean modern builder routes without .php
    for clean_url in &["/n-admin/builder?post=1", "/n-admin/studio?post=1", "/studio?post=1", "/n-admin/builder.php?post=1"] {
        let builder_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(*clean_url)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(builder_res.status(), StatusCode::OK, "Failed on builder route: {}", clean_url);
    }
}

#[tokio::test]
async fn test_native_forms_and_antispam_endpoints() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());

    // 1. Submit valid form
    let valid_payload = serde_json::json!({
        "data": {
            "name": "Enterprise Client",
            "email": "client@acme.corp",
            "message": "Interested in $199/mo Agency SaaS"
        }
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/neopress/v1/forms/1/submit")
                .header("Content-Type", "application/json")
                .header("X-Forwarded-For", "203.0.113.195")
                .header("User-Agent", "Mozilla/5.0")
                .body(Body::from(valid_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json_resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_resp["success"], true);
    assert!(json_resp["entry_id"].as_i64().unwrap() > 0);

    // 2. Submit bot spam with honeypot triggered
    let spam_payload = serde_json::json!({
        "data": {
            "name": "Spam Bot",
            "_np_hp": "automated spam payload"
        }
    });

    let spam_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/neopress/v1/forms/1/submit")
                .header("Content-Type", "application/json")
                .body(Body::from(spam_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(spam_res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cookieless_analytics_endpoints() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());
    let admin_auth = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("admin:admin")
    );

    // 1. Ingest pageview event (public endpoint, zero cookie)
    let event_payload = serde_json::json!({
        "page_url": "/agency-solutions",
        "referrer": "https://linkedin.com",
        "device_type": "desktop",
        "utm_source": "linkedin",
        "utm_medium": "cpc",
        "utm_campaign": "agency_tier"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/neopress/v1/analytics/event")
                .header("Content-Type", "application/json")
                .header("X-Forwarded-For", "198.51.100.42")
                .header("User-Agent", "Chrome/124.0")
                .header("CF-IPCountry", "DE")
                .body(Body::from(event_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Fetch analytics summary (authenticated)
    let summary_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/neopress/v1/analytics/summary")
                .header("Authorization", &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(summary_res.status(), StatusCode::OK);
    let summary_body = to_bytes(summary_res.into_body(), usize::MAX).await.unwrap();
    let summary_json: serde_json::Value = serde_json::from_slice(&summary_body).unwrap();
    assert_eq!(summary_json["total_visitors"], 1);
    assert_eq!(summary_json["total_pageviews"], 1);
    assert_eq!(
        summary_json["top_pages"][0]["page_url"],
        "/agency-solutions"
    );
}

#[tokio::test]
async fn test_wxr_page_builder_transpilation_elementor_and_divi() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());
    let admin_auth = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("admin:admin")
    );

    let wxr_payload = r##"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wp="http://wordpress.org/export/1.2/">
<channel>
    <!-- 1. Elementor Post -->
    <item>
        <title>Imported Elementor Landing Page</title>
        <wp:post_name>imported-elementor-landing</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>post</wp:post_type>
        <content:encoded><![CDATA[Raw fallback content]]></content:encoded>
        <wp:postmeta>
            <wp:meta_key>_elementor_data</wp:meta_key>
            <wp:meta_value><![CDATA[[{"id":"sec_elem_1","elType":"section","settings":{"background_color":"#111827"},"elements":[{"id":"col_1","elType":"column","settings":{"_column_size":100},"elements":[{"id":"hdg_1","elType":"widget","widgetType":"heading","settings":{"title":"Elementor Transpiled Heading","header_size":"h1"}},{"id":"btn_1","elType":"widget","widgetType":"button","settings":{"text":"Click Elementor","link":{"url":"https://elementor-migrated.com"}}}]}]}]]]></wp:meta_value>
        </wp:postmeta>
    </item>

    <!-- 2. Divi Post -->
    <item>
        <title>Imported Divi Campaign</title>
        <wp:post_name>imported-divi-campaign</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>post</wp:post_type>
        <content:encoded><![CDATA[[et_pb_section background_color="#0e7490"][et_pb_row][et_pb_column type="1_2"][et_pb_text]<h3>Divi Transpiled Heading</h3><p>Divi Text Content</p>[/et_pb_text][/et_pb_column][et_pb_column type="1_2"][et_pb_button button_text="Click Divi" button_url="/divi-target"][/et_pb_button][/et_pb_column][/et_pb_row][/et_pb_section]]]></content:encoded>
    </item>
</channel>
</rss>"##;

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/neopress/v1/import-wxr")
                .header("Authorization", &admin_auth)
                .header("Content-Type", "application/xml")
                .body(Body::from(wxr_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let import_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(import_json["success"], true);
    assert_eq!(import_json["imported_posts"], 2);

    // 1. Verify Elementor Transpiled Content via REST & Layout
    let elem_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/posts?slug=imported-elementor-landing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(elem_res.status(), StatusCode::OK);
    let elem_body = to_bytes(elem_res.into_body(), usize::MAX).await.unwrap();
    let elem_posts: serde_json::Value = serde_json::from_slice(&elem_body).unwrap();
    let elem_post = &elem_posts[0];
    let elem_id = elem_post["id"].as_i64().unwrap();
    let elem_html = elem_post["content"]["rendered"].as_str().unwrap();

    assert!(elem_html.contains("<section id=\"sec_elem_1\""));
    assert!(elem_html.contains("<h1 id=\"hdg_1\""));
    assert!(elem_html.contains("Elementor Transpiled Heading</h1>"));
    assert!(elem_html.contains("href=\"https://elementor-migrated.com\""));

    // Check Elementor Layout API
    let layout_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/wp-json/neopress/v1/layout/{}", elem_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(layout_res.status(), StatusCode::OK);
    let layout_body = to_bytes(layout_res.into_body(), usize::MAX).await.unwrap();
    let layout_json: serde_json::Value = serde_json::from_slice(&layout_body).unwrap();
    assert_eq!(
        layout_json["layout"]["root"]["children"][0]["id"],
        "sec_elem_1"
    );

    // 2. Verify Divi Transpiled Content via REST & Layout
    let divi_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/posts?slug=imported-divi-campaign")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(divi_res.status(), StatusCode::OK);
    let divi_body = to_bytes(divi_res.into_body(), usize::MAX).await.unwrap();
    let divi_posts: serde_json::Value = serde_json::from_slice(&divi_body).unwrap();
    let divi_post = &divi_posts[0];
    let divi_id = divi_post["id"].as_i64().unwrap();
    let divi_html = divi_post["content"]["rendered"].as_str().unwrap();

    assert!(divi_html.contains("Divi Transpiled Heading"));
    assert!(divi_html.contains("href=\"/divi-target\""));

    // Check Divi Layout API
    let divi_layout_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/wp-json/neopress/v1/layout/{}", divi_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(divi_layout_res.status(), StatusCode::OK);
    let divi_layout_body = to_bytes(divi_layout_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let divi_layout_json: serde_json::Value = serde_json::from_slice(&divi_layout_body).unwrap();
    assert!(!divi_layout_json["layout"]["root"]["children"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn test_native_subsystems_seo_sitemap_and_email_api() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());
    let admin_auth = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("admin:admin")
    );

    // 1. Dynamic XML Sitemap
    let sitemap_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/sitemap.xml")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sitemap_res.status(), StatusCode::OK);
    let sitemap_body = to_bytes(sitemap_res.into_body(), usize::MAX).await.unwrap();
    let sitemap_xml = String::from_utf8(sitemap_body.to_vec()).unwrap();
    assert!(sitemap_xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(sitemap_xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
    assert!(sitemap_xml.contains("<loc>http://localhost:8090/sample-page/</loc>"));

    // 2. Robots.txt
    let robots_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/robots.txt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(robots_res.status(), StatusCode::OK);
    let robots_body = to_bytes(robots_res.into_body(), usize::MAX).await.unwrap();
    let robots_txt = String::from_utf8(robots_body.to_vec()).unwrap();
    assert!(robots_txt.contains("Sitemap: http://localhost:8090/sitemap.xml"));
    assert!(robots_txt.contains("Disallow: /n-admin/"));

    // 3. Admin Analytics View
    let analytics_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/analytics.php")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(analytics_res.status(), StatusCode::OK);
    let analytics_body = to_bytes(analytics_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let analytics_html = String::from_utf8(analytics_body.to_vec()).unwrap();
    assert!(analytics_html.contains("Cookieless Analytics"));
    assert!(analytics_html.contains("Unique Daily Visitors"));

    // 4. Native Email Dispatch
    let email_payload = serde_json::json!({
        "to": "client@enterprise.com",
        "subject": "New Agency Inquiry Received",
        "body_html": "<p>You have a new submission from Neopress Native Forms.</p>",
        "body_text": "You have a new submission from Neopress Native Forms."
    });

    let email_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/neopress/v1/email/send")
                .header("Authorization", &admin_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(email_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(email_res.status(), StatusCode::OK);

    // Verify email was committed to SQLite audit log
    let email_logs = db.get_email_logs(10).unwrap();
    assert_eq!(email_logs.len(), 1);
    assert_eq!(email_logs[0].recipient, "client@enterprise.com");
    assert_eq!(email_logs[0].subject, "New Agency Inquiry Received");
    assert_eq!(email_logs[0].status, "delivered");
}

#[tokio::test]
async fn test_wp_rest_v2_completeness_admin_persistence_and_public_archives() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());
    let admin_auth = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("admin:admin")
    );

    // 1. Verify X-WP-Total and X-WP-TotalPages headers on /wp-json/wp/v2/posts
    let posts_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/wp/v2/posts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(posts_res.status(), StatusCode::OK);
    assert!(posts_res.headers().get("x-wp-total").is_some());
    assert!(posts_res.headers().get("x-wp-totalpages").is_some());

    // 2. Create Category and Tag via REST API
    let cat_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/categories")
                .header("Authorization", &admin_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "Edge Systems",
                        "description": "Global edge architectures"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cat_res.status(), StatusCode::CREATED);
    let cat_body = to_bytes(cat_res.into_body(), usize::MAX).await.unwrap();
    let cat_json: serde_json::Value = serde_json::from_slice(&cat_body).unwrap();
    assert_eq!(cat_json["slug"], "edge-systems");
    let cat_id = cat_json["id"].as_i64().unwrap();

    let tag_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/tags")
                .header("Authorization", &admin_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "München & 東京"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tag_res.status(), StatusCode::CREATED);
    let tag_body = to_bytes(tag_res.into_body(), usize::MAX).await.unwrap();
    let tag_json: serde_json::Value = serde_json::from_slice(&tag_body).unwrap();
    let tag_id = tag_json["id"].as_i64().unwrap();

    // 3. Create Post with multi-byte UTF-8 content (>140 chars) & assigned category/tag
    let utf8_post_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/wp/v2/posts")
                .header("Authorization", &admin_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "title": "UTF-8 Edge Benchmark München 東京 🚀",
                        "content": "<p>日本語のマルチバイト文字列テスト 🚀⚡🦀 München Zürich São Paulo — repeating multi-byte characters across index 140: äöüß東京🚀⚡🦀äöüß東京🚀⚡🦀äöüß東京🚀⚡🦀äöüß東京🚀⚡🦀äöüß東京🚀⚡🦀</p>",
                        "status": "publish",
                        "categories": [cat_id],
                        "tags": [tag_id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(utf8_post_res.status(), StatusCode::CREATED);

    // 4. Verify Home Page, Category Archive, Tag Archive, Author Archive, and Search SSR do not panic on UTF-8
    for uri in [
        "/",
        "/category/edge-systems",
        "/author/admin",
        "/?s=M%C3%BCnchen",
        "/non-existent-404-slug/",
    ] {
        let res = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        if uri == "/non-existent-404-slug/" {
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        } else {
            assert_eq!(res.status(), StatusCode::OK, "Failed URI: {}", uri);
        }
    }

    // 5. Verify WP REST v2 Discovery Endpoints (types, statuses, taxonomies, themes, plugins, menus, widgets, search)
    for uri in [
        "/wp-json/wp/v2/types",
        "/wp-json/wp/v2/statuses",
        "/wp-json/wp/v2/taxonomies",
        "/wp-json/wp/v2/themes",
        "/wp-json/wp/v2/plugins",
        "/wp-json/wp/v2/menus",
        "/wp-json/wp/v2/widgets",
        "/wp-json/wp/v2/search?search=Hello",
    ] {
        let res = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK, "Failed REST endpoint: {}", uri);
    }

    // 6. Verify Admin Persistence APIs (/n-admin/api/theme, /n-admin/api/customizer, /n-admin/api/plugins)
    let customizer_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/customizer")
                .header("Cookie", "neopress_admin=1")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "title": "Custom Agency Portal",
                        "custom_css": ".custom-test-banner { color: #ff4f00; }"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(customizer_res.status(), StatusCode::OK);

    // Verify custom CSS and site title appear in public SSR
    let home_res = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let home_html = String::from_utf8(
        to_bytes(home_res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(home_html.contains("Custom Agency Portal"));
    assert!(home_html.contains(".custom-test-banner { color: #ff4f00; }"));
}

#[tokio::test]
async fn test_plugins_ecosystem_seed_status_tabs_and_actions() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());

    // 1. Initial admin plugins page loads cleanly with all 8 built-in plugins seeded in SQLite
    let plugins_page_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/plugins.php")
                .header("Cookie", "neopress_admin=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(plugins_page_res.status(), StatusCode::OK);
    let plugins_html = String::from_utf8(
        to_bytes(plugins_page_res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    // Verify status tabs and filter controls
    assert!(plugins_html.contains("status-tab"));
    assert!(plugins_html.contains("pluginSearchInput"));
    assert!(plugins_html.contains("bulkPluginAction"));

    // Verify all 8 built-in plugins are listed
    assert!(plugins_html.contains("Neopress SEO &amp; Redirection Suite") || plugins_html.contains("Neopress SEO"));
    assert!(plugins_html.contains("Neopress Forms Studio &amp; Lead Inbox") || plugins_html.contains("Neopress Forms"));
    assert!(plugins_html.contains("Snapshot &amp; Hot Backup Manager") || plugins_html.contains("Snapshot"));
    assert!(plugins_html.contains("Floating WhatsApp &amp; Lead Chat Button") || plugins_html.contains("WhatsApp"));
    assert!(plugins_html.contains("WPCode Header, Footer &amp; Custom Snippets") || plugins_html.contains("WPCode"));
    assert!(plugins_html.contains("Neopress WAL Optimizer"));
    assert!(plugins_html.contains("WebP Image Engine"));
    assert!(plugins_html.contains("WXR Streaming XML"));

    // Verify forms shortcode expansion works out of the box because neopress-forms is active
    let sample_content = "Please reach out to us: [contact-form-7 id=\"1\" title=\"Contact\"]";
    let settings = db.get_settings().unwrap();
    let expanded = neopress_server::plugins::expand_shortcodes_in_content(sample_content, &db, &settings);
    assert!(expanded.contains("np-native-form-card") || expanded.contains("np-form-"));

    // 2. Add Plugins page (/n-admin/plugin-install.php) loads cleanly with curated extensions
    let install_page_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/plugin-install.php")
                .header("Cookie", "neopress_admin=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(install_page_res.status(), StatusCode::OK);
    let install_html = String::from_utf8(
        to_bytes(install_page_res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(install_html.contains("1-Click Community Extensions"));
    assert!(install_html.contains("Top Promo Announcement Bar"));
    assert!(install_html.contains("Lightweight Cookie Consent"));
    assert!(install_html.contains("Scroll Reading Progress Bar"));
    assert!(install_html.contains("quickInstallPlugin"));

    // 3. Test toggle plugin status (deactivate toggleable plugin like whatsapp-chat)
    let toggle_deactivate_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/plugins")
                .header("Cookie", "neopress_admin=1")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "slug": "whatsapp-chat",
                        "active": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(toggle_deactivate_res.status(), StatusCode::OK);
    assert!(!db.is_plugin_active("whatsapp-chat"));

    // Reactivate whatsapp-chat
    let toggle_activate_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/plugins")
                .header("Cookie", "neopress_admin=1")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "slug": "whatsapp-chat",
                        "active": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(toggle_activate_res.status(), StatusCode::OK);
    assert!(db.is_plugin_active("whatsapp-chat"));

    // 4. Test core always-on plugin protection: cannot deactivate neopress-backups
    let toggle_core_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/plugins")
                .header("Cookie", "neopress_admin=1")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "slug": "neopress-backups",
                        "active": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(toggle_core_res.status(), StatusCode::BAD_REQUEST);
    assert!(db.is_plugin_active("neopress-backups"));

    // 5. Test 1-click install of a community extension (cookie-consent)
    let install_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/n-admin/api/plugin-install")
                .header("Cookie", "neopress_admin=1")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "manifest": {
                            "slug": "cookie-consent",
                            "name": "Lightweight Cookie Consent",
                            "version": "1.0.0",
                            "author": "Neopress Community",
                            "description": "GDPR and privacy cookie compliance banner with opt-in storage",
                            "script": "console.log('cookie-consent active');",
                            "admin_settings_schema": []
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(install_res.status(), StatusCode::OK);
    assert!(db.is_plugin_active("cookie-consent"));

    // Verify community plugin shows up in installed plugins view
    let plugins_page_res2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/n-admin/plugins.php")
                .header("Cookie", "neopress_admin=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let plugins_html2 = String::from_utf8(
        to_bytes(plugins_page_res2.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(plugins_html2.contains("Lightweight Cookie Consent"));
    assert!(plugins_html2.contains("Community"));
}
