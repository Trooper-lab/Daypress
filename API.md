# Neopress REST API Reference

Neopress provides drop-in compatibility with the **WordPress REST API v2** specification alongside high-performance extensions under the `/wp-json/neopress/v1/` namespace.

---

## 🧭 Discovery & Global Index

### `GET /wp-json` or `GET /wp-json/`
Returns site metadata, description, URL, and registered namespaces.

#### Example Request
```bash
curl -X GET http://localhost:8080/wp-json/
```

#### Example Response (`200 OK`)
```json
{
  "name": "Neopress Site",
  "description": "Just another Neopress high-performance site",
  "url": "http://localhost:8080",
  "home": "http://localhost:8080",
  "gmt_offset": "0",
  "timezone_string": "",
  "namespaces": [
    "wp/v2",
    "jwt-auth/v1",
    "neopress/v1"
  ],
  "authentication": {
    "application_passwords": {
      "endpoints": {
        "authorization": "/wp-json/wp/v2/users/me/application-passwords"
      }
    },
    "jwt": {
      "endpoints": {
        "token": "/wp-json/jwt-auth/v1/token",
        "validate": "/wp-json/jwt-auth/v1/token/validate"
      }
    },
    "nonce": {
      "endpoint": "/wp-json/neopress/v1/auth/nonce"
    }
  },
  "routes": {
    "/wp/v2/posts": {},
    "/wp/v2/pages": {},
    "/wp/v2/media": {},
    "/wp/v2/categories": {},
    "/wp/v2/tags": {},
    "/wp/v2/users": {},
    "/wp/v2/settings": {},
    "/jwt-auth/v1/token": {},
    "/neopress/v1/auth/nonce": {},
    "/neopress/v1/stats": {},
    "/neopress/v1/vacuum": {},
    "/neopress/v1/integrity-check": {},
    "/neopress/v1/import": {},
    "/neopress/v1/import-wxr": {}
  }
}
```

---

## 🔐 Authentication & Credential Architecture

Neopress supports full standard WordPress headless and admin client authentication mechanisms:

### 1. Application Passwords (WordPress 5.6+ Standard)
Headless clients and external builders (Next.js, Astro, WP-CLI) authenticate using HTTP Basic Auth with user credentials or generated Application Passwords.

#### `POST /wp-json/wp/v2/users/{id}/application-passwords`
Generate a new application password for a user.

##### Request Body
```json
{
  "name": "Next.js Production Builder",
  "app_id": "headless-front"
}
```

##### Example Response (`201 Created`)
```json
{
  "uuid": "00000192e4ab12cd",
  "app_id": "headless-front",
  "name": "Next.js Production Builder",
  "password": "abcd efgh ijkl mnop",
  "created": "2026-10-21 21:00:00"
}
```

##### Authenticating with Application Password
Pass in the `Authorization` header:
```bash
curl -X POST http://localhost:8080/wp-json/wp/v2/posts \
  -H "Authorization: Basic YWRtaW46YWJjZCBlZmdoIGlqa2wgbW5vcA==" \
  -H "Content-Type: application/json" \
  -d '{"title": "Published via App Password", "content": "Hello World"}'
```

### 2. Bearer JWT Tokens (`jwt-auth/v1`)
Neopress supports standard HS256 JWT tokens compatible with the `jwt-auth` WordPress plugin ecosystem.

#### `POST /wp-json/jwt-auth/v1/token`
Generate a signed JWT token using account credentials.

##### Request Body
```json
{
  "username": "admin",
  "password": "admin"
}
```

##### Example Response (`200 OK`)
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_email": "admin@example.com",
  "user_nicename": "admin",
  "user_display_name": "admin"
}
```

#### `POST /wp-json/jwt-auth/v1/token/validate`
Validate an active JWT token.
```bash
curl -X POST http://localhost:8080/wp-json/jwt-auth/v1/token/validate \
  -H "Authorization: Bearer <token>"
```

### 3. REST Nonces (`X-WP-Nonce`)
Frontend scripts, admin editors, and cookie sessions validate writes using WordPress rest nonces:

#### `GET /wp-json/neopress/v1/auth/nonce`
Fetch a cryptographic nonce for action `wp_rest`.
```json
{
  "nonce": "a1b2c3d4e5",
  "action": "wp_rest"
}
```
Pass the nonce in the `X-WP-Nonce` header or query string `?_wpnonce=<nonce>`.

---

## 📝 Posts (`/wp-json/wp/v2/posts`)

### `GET /wp-json/wp/v2/posts`
Retrieve a paginated list of posts.

#### Query Parameters
| Parameter | Type | Default | Description |
|---|---|---|---|
| `per_page` | integer | `10` | Maximum number of posts to return. |
| `slug` | string | - | Filter posts by exact slug. |
| `search` | string | - | Limit results to posts containing the search term. |
| `status` | string | `"publish"` | Post status (`publish`, `future`, `draft`, `pending`, `private`, `trash`, `any`). Visitors only ever get `publish`. |

#### Example Request
```bash
curl -X GET "http://localhost:8080/wp-json/wp/v2/posts?search=rust&per_page=5"
```

---

### `POST /wp-json/wp/v2/posts`
Create a new post.

#### Headers
`Content-Type: application/json`

#### Request Body
```json
{
  "title": "Migrating to Native Rust",
  "content": "<p>Discover how Neopress achieves sub-millisecond query latencies.</p>",
  "status": "publish",
  "slug": "migrating-to-native-rust"
}
```

Optional `date` / `date_gmt` (ISO 8601, e.g. `2026-11-01T09:30:00`; `date_gmt` wins when both are sent) set the publish date. Site time is UTC, so a date without an offset is read as UTC. As in WordPress, a `publish` post dated at least a minute ahead is stored as `future` (scheduled): it stays out of every public page, feed, the sitemap and unauthenticated REST reads until its date passes, then it is published automatically. Sending `status: "future"` with a date that is not ahead publishes the post. An unparseable date returns `400`.

#### Example Response (`201 Created`)
```json
{
  "id": 42,
  "date": "2026-10-21 20:00:00",
  "slug": "migrating-to-native-rust",
  "status": "publish",
  "type": "post",
  "link": "http://localhost:8080/migrating-to-native-rust/",
  "title": {
    "rendered": "Migrating to Native Rust"
  },
  "content": {
    "rendered": "<p>Discover how Neopress achieves sub-millisecond query latencies.</p>"
  },
  "excerpt": {
    "rendered": ""
  },
  "author": 1
}
```

---

### `GET /wp-json/wp/v2/posts/{id}`
Retrieve a single post by its ID.

#### Example Response (`200 OK`)
```json
{
  "id": 42,
  "title": { "rendered": "Migrating to Native Rust" },
  "slug": "migrating-to-native-rust",
  "content": { "rendered": "<p>Content...</p>" },
  "status": "publish"
}
```

---

### `PUT /wp-json/wp/v2/posts/{id}`
Update an existing post.

#### Request Body (Partial updates supported)
```json
{
  "title": "Migrating to Native Rust (Updated)",
  "status": "publish"
}
```

---

### `DELETE /wp-json/wp/v2/posts/{id}`
Delete a post.

#### Example Response (`200 OK`)
```json
{
  "id": 42,
  "status": "trash",
  "message": "Post 42 moved to trash / deleted"
}
```

---

## 📄 Pages (`/wp-json/wp/v2/pages`)

- `GET /wp-json/wp/v2/pages`: List all pages.
- `GET /wp-json/wp/v2/pages/{id}`: Retrieve a single page by ID.

---

## 🖼️ Media & File Uploads (`/wp-json/wp/v2/media`)

### `GET /wp-json/wp/v2/media`
List all uploaded media attachments, complete with calculated dimensions and thumbnail variants.

### `POST /wp-json/wp/v2/media`
Upload an image or document via `multipart/form-data`. Uploading images automatically computes image dimensions, generates standard WordPress thumbnail sizes (`thumbnail`, `medium`, `large`), and produces modern WebP variants.

#### Example Request
```bash
curl -X POST http://localhost:8080/wp-json/wp/v2/media \
  -H "Authorization: Basic YWRtaW46YWRtaW4=" \
  -F "file=@/path/to/diagram.png"
```

#### Example Response (`201 Created`)
```json
{
  "id": 105,
  "date": "2026-10-21T20:15:00",
  "slug": "diagram",
  "title": { "rendered": "diagram" },
  "source_url": "http://localhost:8080/uploads/1729541700_diagram.png",
  "media_type": "image",
  "mime_type": "image/png",
  "media_details": {
    "width": 1200,
    "height": 800,
    "file": "1729541700_diagram.png",
    "filesize": 154200,
    "sizes": {
      "thumbnail": {
        "file": "1729541700_diagram-150x150.png",
        "width": 150,
        "height": 150,
        "mime_type": "image/png",
        "source_url": "http://localhost:8080/uploads/1729541700_diagram-150x150.png"
      },
      "medium": {
        "file": "1729541700_diagram-300x200.png",
        "width": 300,
        "height": 200,
        "mime_type": "image/png",
        "source_url": "http://localhost:8080/uploads/1729541700_diagram-300x200.png"
      },
      "large": {
        "file": "1729541700_diagram-1024x683.png",
        "width": 1024,
        "height": 683,
        "mime_type": "image/png",
        "source_url": "http://localhost:8080/uploads/1729541700_diagram-1024x683.png"
      },
      "webp": {
        "file": "1729541700_diagram.webp",
        "width": 1200,
        "height": 800,
        "mime_type": "image/webp",
        "source_url": "http://localhost:8080/uploads/1729541700_diagram.webp"
      }
    }
  }
}
```

### Dynamic On-The-Fly Thumbnails & Edge CDN Caching
- **High-Performance Static Delivery**: Static files requested from `/uploads/{*file}` or `/wp-content/uploads/{*file}` are served directly with `Cache-Control: public, max-age=31536000, immutable`.
- **On-The-Fly Resizing**: When requesting any dimension crop (e.g. `diagram-300x200.png` or `diagram.webp`) that does not yet exist on disk, Neopress parses the crop parameters from the URI, resizes the base image on the fly, persists the crop to disk, and streams the result.
- **Bulk Import Protection**: During large imports (e.g. 2GB+ media dumps), thumbnail generation runs in non-blocking background workers or dynamically on first request, ensuring instantaneous import without timeouts or CPU exhaustion.

### `DELETE /wp-json/wp/v2/media/{id}`
Delete an attachment and remove its physical file from disk.

---

## 🏷️ Taxonomies (`categories` & `tags`)

- `GET /wp-json/wp/v2/categories`: Retrieve all categories.
- `GET /wp-json/wp/v2/tags`: Retrieve all tags.

---

## 👤 Users (`/wp-json/wp/v2/users`)

- `GET /wp-json/wp/v2/users`: List users.
- `GET /wp-json/wp/v2/users/me`: Retrieve current logged-in user context.
- `POST /wp-json/wp/v2/users/{id}` / `PUT /wp-json/wp/v2/users/{id}`: Update user name or email.

---

## 💬 Comments (`/wp-json/wp/v2/comments`)

Neopress provides full WordPress comments API support for headless discussion systems and the admin moderation queue.

### `GET /wp-json/wp/v2/comments`
List comments with author details, post relations, and moderation status.

#### Example Response (`200 OK`)
```json
[
  {
    "id": 1,
    "post_id": 1,
    "post_title": "Hello world!",
    "author": "A WordPress Commenter",
    "author_email": "wapuu@wordpress.example",
    "author_url": "https://wordpress.org/",
    "author_ip": "127.0.0.1",
    "content": "Hi, this is a comment...",
    "date": "2026-10-21 20:00:00",
    "status": "1"
  }
]
```

---

### `GET /wp-json/wp/v2/comments/{id}`
Retrieve a single comment record by ID.

---

### `POST /wp-json/wp/v2/comments/{id}` or `PUT /wp-json/wp/v2/comments/{id}`
Update comment moderation status (`approved`, `pending`, `spam`, `trash`).

#### Request Body
```json
{
  "status": "spam"
}
```

#### Example Response (`200 OK`)
```json
{
  "success": true,
  "status": "spam"
}
```

---

### `DELETE /wp-json/wp/v2/comments/{id}`
Permanently delete a comment from SQLite.

#### Example Response (`200 OK`)
```json
{
  "deleted": true,
  "previous": {
    "id": 1
  }
}
```

---

## ⚙️ Settings (`/wp-json/wp/v2/settings`)

### `GET /wp-json/wp/v2/settings`
Retrieve blog options (`title`, `description`, `url`, `posts_per_page`).

### `POST /wp-json/wp/v2/settings`
Update site options.

#### Request Body
```json
{
  "title": "Neopress High-Performance",
  "description": "Native Rust CMS"
}
```

---

## 🚀 Neopress v1 System Extensions

Neopress provides dedicated high-performance operational endpoints for database management and atomic migration under `/wp-json/neopress/v1/`.

### `GET /wp-json/neopress/v1/stats`
Telemetry and SQLite storage diagnostics.

#### Example Response (`200 OK`)
```json
{
  "total_posts": 142,
  "total_pages": 8,
  "db_size_kb": 128,
  "page_count": 32,
  "journal_mode": "wal"
}
```

---

### `POST /wp-json/neopress/v1/vacuum`
Defragments the SQLite database and reclaims unused disk pages.

#### Example Response (`200 OK`)
```json
{
  "status": "success",
  "message": "Database successfully vacuumed"
}
```

---

### `GET /wp-json/neopress/v1/integrity-check`
Runs SQLite's low-level `PRAGMA integrity_check` across all B-Trees and indices.

#### Example Response (`200 OK`)
```json
{
  "status": "healthy",
  "integrity": ["ok"]
}
```

---

### `POST /wp-json/neopress/v1/import`
Atomically imports an entire WordPress migration bundle in a single SQLite transaction.

#### Request Body
```json
{
  "options": [
    { "name": "blogname", "value": "Imported Blog" }
  ],
  "posts": [
    {
      "post_title": "Imported Article",
      "post_name": "imported-article",
      "post_content": "<p>Migrated from WordPress.</p>",
      "post_status": "publish",
      "post_type": "post"
    }
  ]
}
```

#### Example Response (`200 OK`)
```json
{
  "status": "success",
  "imported_posts": 1,
  "imported_options": 1
}
```

---

### `POST /wp-json/neopress/v1/import-wxr`
Streaming ingestion of standard WordPress XML export files (WXR 1.2 / Tools &rarr; Export &rarr; All content).
Accepts either `multipart/form-data` with a `.xml` file or raw `application/xml` payloads. Employs a streaming XML parser (`quick-xml`) with near-zero memory footprint for multi-gigabyte exports.

#### Example Request
```bash
curl -X POST http://localhost:8080/wp-json/neopress/v1/import-wxr \
  -H "Authorization: Basic YWRtaW46YWRtaW4=" \
  -F "file=@/path/to/wordpress-export.xml"
```

#### Example Response (`200 OK`)
```json
{
  "success": true,
  "imported_posts": 42,
  "imported_pages": 6,
  "imported_media": 118,
  "imported_categories": 12,
  "imported_tags": 35,
  "imported_comments": 87,
  "message": "Successfully ingested WordPress XML Stream: 42 posts, 6 pages, 118 media attachments, 12 categories, 87 comments."
}
```

---

## 🛠️ Administration & Data Portability APIs

In addition to standard REST API v2 endpoints, Neopress exposes high-performance administration APIs for settings persistence and data portability:

### Settings APIs
- `POST /n-admin/api/options-writing`: Persists writing options (`default_category`, `default_post_format`, `mailserver_url`, `mailserver_port`).
- `POST /n-admin/api/options-reading`: Persists reading options (`show_on_front`, `page_on_front`, `posts_per_page`, `blog_public`).
- `POST /n-admin/api/options-discussion`: Persists discussion options (`default_comment_status`, `thread_comments`, `show_avatars`).
- `POST /n-admin/api/options-media`: Persists media sizes (`thumbnail_size_w`, `medium_size_w`, `large_size_w`, `thumbnail_crop`).
- `POST /n-admin/api/options-permalink`: Persists URL rewrite patterns (`permalink_structure`, `category_base`, `tag_base`).
- `POST /n-admin/api/options-privacy`: Persists privacy policy designated page (`wp_page_for_privacy_policy`).

---

### Data Export & GDPR APIs

#### `GET /n-admin/export-wxr`
Generates and downloads a complete standard WordPress XML export (WXR 1.2 / RSS 2.0) containing all posts, categories, tags, authors, and channel metadata with `Content-Disposition: attachment; filename="neopress.wxr.xml"`.

#### `GET /n-admin/export-json`
Generates a complete JSON backup bundle containing all site settings, posts, users, and comments with `Content-Disposition: attachment; filename="neopress-backup.json"`.

#### `GET /n-admin/api/export-personal-data?user={email_or_username}`
Retrieves all comments and user records associated with the specified identity formatted as a GDPR personal data archive.

#### `POST /n-admin/api/erase-personal-data`
Anonymizes and scrubs personal data (replaces name with `Anonymous`, email with `deleted@privacy.local`, and IP with `0.0.0.0`) across all comment records matching the identifier.

##### Request Body
```json
{
  "identifier": "user@example.com"
}
```

##### Example Response (`200 OK`)
```json
{
  "success": true,
  "items_removed": 3,
  "message": "Personal data anonymized and scrubbed across 3 comments in SQLite"
}
```

---

### Native Subsystems & Visual Builder APIs

#### `GET /wp-json/neopress/v1/layout/{id}`
Retrieves the canonical `NeopressLayout` JSON for a post. Prioritizes the dedicated `layout_json` column on `wp_posts`, with automatic fallback to `wp_postmeta` (`_neopress_layout`).

#### `POST /wp-json/neopress/v1/layout/{id}`
Saves the JSON layout for a post using a transactional dual-write pattern to `layout_json` and `wp_postmeta`, and synchronizes pre-rendered semantic HTML to `wp_posts.post_content` for zero-JS public SSR.

#### `POST /wp-json/neopress/v1/forms/{id}/submit`
Submits data to a declarative form. Validates input, rejects spam submissions via invisible honeypot (`_np_hp`) and cryptographic time-lock tokens, stores the entry in SQLite, and dispatches notification emails and signed webhooks.

#### `POST /wp-json/neopress/v1/analytics/event`
Server-side cookieless pageview ingestion. Hashes `(Client_IP + User_Agent + Daily_Salt)` with zero cookies or local storage, maintaining complete GDPR and ePrivacy compliance without consent banners.

#### `GET /wp-json/neopress/v1/analytics/summary`
*(Authenticated)* Returns high-performance rollup metrics: total unique visitors, pageviews, and top landing page paths.

#### `POST /wp-json/neopress/v1/email/send`
*(Authenticated)* Dispatches a transactional email message through the native email subsystem, recording latency and delivery status to the SQLite audit log.

#### `GET /sitemap.xml`
Dynamic streaming XML sitemap conforming to sitemaps.org 0.9 protocol, indexing all published posts and pages with last-modified timestamps directly from SQLite.

#### `GET /robots.txt`
Standard search crawler directives linking to `/sitemap.xml` with administration route exclusions.

#### `GET /feed/`, `GET /category/{slug}/feed/`, `GET /tag/{slug}/feed/`
RSS 2.0 feeds (`application/rss+xml`) of published blog posts, newest first, limited to Settings → Reading → "Syndication feeds show the most recent" (default 10). Each item has its title, permalink, `guid`, `pubDate`, author (`dc:creator`), categories and tags, an excerpt and the post HTML (`content:encoded`). Term feeds return `404` for an unknown slug. Public pages advertise `/feed/` with `<link rel="alternate" type="application/rss+xml">`.

---

## 🚨 Error Responses

When an error occurs, Neopress returns standard WordPress REST JSON error payloads matching standard HTTP status codes:

```json
{
  "code": "rest_not_found",
  "message": "Resource not found: Post 99999",
  "data": {
    "status": 404
  }
}
```

### Common Error Codes
- `rest_not_found` (`404`): The requested entity ID or slug does not exist.
- `rest_invalid_param` (`400`): Malformed input payload or missing required parameter.
- `database_error` (`500`): Underlying SQLite query constraint violation.
- `internal_server_error` (`500`): Unexpected internal operational failure.
