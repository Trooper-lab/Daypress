# Neopress Architecture Specification & Strategic Roadmap

This document provides a comprehensive technical overview and strategic engineering specification for Neopress: system design, crate separation, concurrency model, database persistence, visual page builder architecture, page builder AST transpilation, native subsystem convergence, edge SaaS multi-tenancy, cookieless analytics, and verification pipelines.

---

## 1. System Philosophy & Objectives

Neopress is engineered to satisfy the following core requirements:

1. **Drop-in WordPress REST API Compatibility**: Ensure client applications, mobile apps, headless frameworks (Next.js, Nuxt, Astro), and standard WordPress tools interact with Neopress without client modifications.
2. **Sub-Millisecond Read/Write Performance**: Eliminate external database socket roundtrips (MySQL/PostgreSQL) by utilizing an embedded SQLite engine configured with Write-Ahead Logging (WAL) and memory-mapped I/O.
3. **Interactive 60fps Visual Page Builder**: Provide an instant, client-side WebAssembly (Wasm via Leptos) Single Page Application for drag-and-drop page authoring that saves to a strict, typed JSON Layout Schema, eliminating the lag and bloat of legacy PHP page builders.
4. **70% Market Capture via AST Transpilation**: Natively ingest and transpile Elementor (`_elementor_data` JSON) and Divi (nested shortcode trees) layouts during WXR imports, mapping them mathematically to the Neopress JSON Layout Schema.
5. **Zero "Plugin Tax" Core Convergence**: Replace the bloated, expensive plugin stack (WP Mail SMTP, Yoast/RankMath, WPForms) with compile-time embedded, native Rust subsystems for transactional email (`lettre`), dynamic SEO/sitemaps, and secure forms/webhooks.
6. **Edge SaaS Multi-Tenancy & Continuous Replication**: Enable the $199/month Agency SaaS tier with instant edge deployment (Fly.io), automatic SSL/domain provisioning (Cloudflare for SaaS), and continuous, zero-data-loss SQLite WAL streaming to S3/R2 (Litestream).
7. **Privacy-First European Compliance**: Provide native, cookieless, zero-consent-banner analytics powered by daily rotating salted IP hashes, paired with a typed Leptos-to-JavaScript bridge for Google Tag Manager (Consent Mode v2).
8. **Strict Memory Safety & Zero Panics**: Ensure request handlers never invoke `.unwrap()` or panic on malformed user input or missing database records.
9. **Verified Modernization Pipeline**: Provide deterministic, automated transpilation of legacy WordPress PHP routines into safe Rust using LLM-assisted Golden Master differential testing.

---

## 2. Workspace Organization

Neopress adheres to the standard Cargo **Virtual Workspace** pattern:

```
Neopress/
├── Cargo.toml                    # Root virtual workspace manifest
├── Cargo.lock
├── bin/                          # Cross-platform CLI runner scripts
│   ├── neopress-transpiler
│   └── neopress-transpiler.bat
├── crates/
│   ├── neopress-server/          # Primary Axum server, API, SSR, Database, Native Subsystems
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs           # Server entrypoint, CLI flags, graceful shutdown
│   │   │   ├── lib.rs            # Library entrypoint exporting create_app and services
│   │   │   ├── api.rs            # WordPress REST API v2 & Neopress v1 routes
│   │   │   ├── auth.rs           # Nonces, Application Passwords, JWT, Axum extractors
│   │   │   ├── media.rs          # Image resizing, metadata, WebP, on-the-fly CDN
│   │   │   ├── wxr.rs            # Low-memory streaming WordPress XML (WXR) parser
│   │   │   ├── db.rs             # Rusqlite persistence layer with transactional support
│   │   │   ├── errors.rs         # Strongly typed AppError & WpErrorResponse
│   │   │   ├── handlers.rs       # Request handlers (Public SSR, Admin, Analytics)
│   │   │   ├── models.rs         # WordPress entity schemas & DTOs
│   │   │   ├── email.rs          # Native transactional email engine (lettre)
│   │   │   ├── seo.rs            # Dynamic SEO, JSON-LD, and streaming XML sitemaps
│   │   │   ├── forms.rs          # Native declarative forms, anti-spam & webhook dispatcher
│   │   │   ├── analytics.rs      # Cookieless analytics engine with daily rotating salt
│   │   │   └── templates/        # Modular Server-Side Rendering submodules
│   │   │       ├── mod.rs
│   │   │       ├── utils.rs      # HTML escaping and content sanitization
│   │   │       ├── public.rs     # Public frontend templates (home, single post)
│   │   │       └── admin/        # Admin portal templates (dashboard, settings, etc.)
│   │   ├── static/               # Bundled assets (CSS, JS bridge, builder Wasm bundle)
│   │   └── tests/                # Automated integration tests
│   │       ├── api_tests.rs      # HTTP integration tests via Tower Oneshot
│   │       └── db_tests.rs       # In-memory database persistence tests
│   ├── neopress-builder/         # WebAssembly (Leptos) Visual Drag-and-Drop Page Builder
│   │   ├── Cargo.toml            # wasm32-unknown-unknown target
│   │   └── src/
│   │       ├── lib.rs            # Leptos builder root component & reactive state
│   │       ├── canvas.rs         # 60fps drag-and-drop interactive canvas
│   │       ├── components/       # Layout widgets (Container, Grid, Heading, Form, etc.)
│   │       ├── inspector.rs      # Reactive property inspector & style editor
│   │       └── js_bridge.rs      # GTM Consent Mode v2 & DOM interop bridge
│   ├── neopress-ast/             # Page Builder Transpiler (Elementor & Divi AST parser)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # AST translation engine & schema normalizer
│   │       ├── elementor.rs      # Elementor _elementor_data JSON parser & mapper
│   │       ├── divi.rs           # Divi shortcode grammar tokenizer & AST builder
│   │       └── schema.rs         # Canonical NeopressLayout JSON schema definitions
│   ├── neopress-transpiler/      # AI Transpiler CLI verification agent
│   │   ├── Cargo.toml
│   │   └── src/main.rs           # Multi-attempt compilation and Golden Master testing
│   └── generated_slugify/        # Verified transpiled WordPress slugifier
│       ├── Cargo.toml
│       └── src/lib.rs            # Mathematical equivalence verified against WP
├── schemas/                      # Reference SQLite fixtures & JSON schemas
├── uploads/                      # Media uploads directory
└── .github/workflows/ci.yml      # Automated GitHub Actions CI pipeline
```

---

## 3. High-Level System Architecture

```mermaid
flowchart TD
    subgraph Edge ["Edge Infrastructure Layer (Fly.io & Cloudflare)"]
        Client["Browsers / Mobile Apps / Headless Frontends"] --> CF["Cloudflare for SaaS (SSL / Custom Hostnames)"]
        CF --> Fly["Fly.io Anycast Edge Nodes (Neopress Instances)"]
    end

    subgraph CoreApp ["Neopress Server (Axum 0.8)"]
        Fly --> Router["Axum HTTP Router & Middleware"]
        Router --> Auth["Authentication & Nonces (auth.rs)"]
        
        Auth --> REST["WP REST API v2 & Neopress v1 (api.rs)"]
        Auth --> SSR["Public SSR Engine (templates/public.rs)"]
        Auth --> AdminSSR["Admin Shell SSR (templates/admin/)"]
        Auth --> Subsystems["Native Subsystems (Email, SEO, Forms)"]
        Auth --> Analytics["Cookieless Analytics Ingest (analytics.rs)"]
    end

    subgraph ClientWasm ["Admin Visual Page Builder"]
        AdminSSR -->|Loads SPA| Leptos["Leptos WebAssembly Canvas (neopress-builder)"]
        Leptos -->|REST API (JSON Schema)| REST
        Leptos -->|JS FFI| GTM["GTM Consent Mode v2 Bridge"]
    end

    subgraph Ingestion ["Ingestion & Transpilation Pipeline"]
        WXRUpload["WXR XML Upload"] --> StreamingParser["Streaming XML Parser (wxr.rs)"]
        StreamingParser --> AST["AST Translation Layer (neopress-ast)"]
        AST -->|Parse _elementor_data| ElemTrans["Elementor JSON Parser"]
        AST -->|Parse [et_pb_*]| DiviTrans["Divi Shortcode Parser"]
        ElemTrans --> NeopressJSON["Canonical NeopressLayout JSON"]
        DiviTrans --> NeopressJSON
    end

    subgraph Persistence ["Storage & Disaster Recovery"]
        REST --> DB[("SQLite Database (WAL Mode)")]
        SSR --> DB
        Subsystems --> DB
        Analytics --> DB
        NeopressJSON --> DB
        
        DB -.->|Continuous WAL Replication| Litestream["Litestream Daemon"]
        Litestream -.->|Encrypted S3 Snapshots| S3[("S3 / Cloudflare R2 Bucket")]
    end
```

---

## 4. Web & HTTP Pipeline (`neopress-server`)

### Routing Framework
The HTTP engine is powered by **Axum 0.8** on top of **Tokio** and **Tower HTTP**. The routing hierarchy is structured into dedicated operational domains:

```mermaid
flowchart TD
    Req["Incoming HTTP Request"] --> Trace["TraceLayer (Request Logging)"]
    Trace --> Cors["CorsLayer (Cross-Origin Resource Sharing)"]
    Cors --> HostRouting["Multi-Tenant Host Router (Host Header)"]
    HostRouting --> Router["Axum Router"]

    Router -->|"/wp-json/wp/v2/*"| REST["WordPress REST API v2 Engine"]
    Router -->|"/wp-json/neopress/v1/*"| NRest["Neopress Extended APIs"]
    Router -->|"/n-admin/builder/*"| Builder["Leptos Wasm Visual Page Builder"]
    Router -->|"/n-admin/*"| Admin["Admin Portal Shell (templates/admin/)"]
    Router -->|"/wp-admin/*"| RedirectAdmin["Admin Redirect (/n-admin/)"]
    Router -->|"/sitemap*.xml"| Sitemap["Native Dynamic SEO Sitemap Stream"]
    Router -->|"/n-api/analytics/event"| AnalyticsRoute["Cookieless Analytics Ingest"]
    Router -->|"/static/*, /uploads/*"| Static["ServeDir (Static & Dynamic Media)"]
    Router -->|"/" or "/{slug}/"| PublicSSR["Public Editorial SSR (<1ms TTFB)"]
```

### Decoupled Application Architecture
`main.rs` is solely responsible for binary initialization:
- Parsing command-line flags and environment variables via `clap`.
- Initializing structured logging with `tracing-subscriber` and `EnvFilter`.
- Instantiating the SQLite database with WAL and foreign keys enabled.
- Spawning background worker queues for Litestream replication, email sending, and analytics buffering.
- Binding the `tokio::net::TcpListener`.
- Handling graceful shutdown via `tokio::signal::ctrl_c()`.

---

## 5. Error Handling & WordPress REST Compliance

To ensure API clients receive idiomatic WordPress responses even during error conditions, Neopress implements a dedicated error taxonomy in `errors.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("AST Transpilation error: {0}")]
    Transpilation(String),
}
```

### Error Serialization
`AppError` implements Axum's `IntoResponse` trait, translating internal variants into standard WordPress JSON errors:

```json
{
  "code": "rest_not_found",
  "message": "Resource not found: Post 9999",
  "data": {
    "status": 404
  }
}
```

| `AppError` Variant | HTTP Status | WP Error Code |
|---|---|---|
| `NotFound(msg)` | `404 Not Found` | `rest_not_found` |
| `BadRequest(msg)` | `400 Bad Request` | `rest_invalid_param` |
| `Unauthorized(msg)` | `401 Unauthorized` | `rest_cannot_read` / `rest_cannot_create` |
| `Forbidden(msg)` | `403 Forbidden` | `rest_forbidden` |
| `Database(err)` | `500 Internal Server Error` | `database_error` |
| `Transpilation(err)`| `422 Unprocessable Entity`| `neopress_ast_error` |
| `Internal(msg)` | `500 Internal Server Error` | `internal_server_error` |

No request handler is permitted to invoke `.unwrap()` or panic.

---

## 6. Storage Engine & Database Design

### Embedded SQLite Persistence
Database interactions are managed through `rusqlite` with a thread-safe connection handle:

```rust
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}
```

### Performance & Concurrency Optimizations
1. **Write-Ahead Logging (WAL)**: `PRAGMA journal_mode = WAL;` enables concurrent readers while a single writer operates without lock starvation.
2. **Synchronous Mode**: `PRAGMA synchronous = NORMAL;` provides safe durability without excessive fsync penalties.
3. **Foreign Keys**: `PRAGMA foreign_keys = ON;` guarantees relational integrity across posts, taxonomies, options, forms, and analytics.
4. **Memory-Mapped I/O**: `PRAGMA mmap_size = 268435456;` (256MB) eliminates memory copy overhead between kernel and user space.
5. **Schema Versioning**: Monitored via `PRAGMA user_version;` to execute incremental migrations automatically upon startup.

---

## 7. The Visual Builder: Wasm (Leptos) vs. Public SSR

Legacy WordPress relies heavily on server-rendered textareas (TinyMCE) or JavaScript-heavy block editors (Gutenberg) and monstrous third-party plugins (Elementor, Divi) that inject megabytes of bloated JavaScript and CSS. Neopress resolves this through a **Hybrid Architecture**:

- **Public Frontend**: Ultra-high-speed Server-Side Rendering (SSR) via Axum. Delivers static semantic HTML/CSS generated from the canonical JSON layout with sub-millisecond TTFB ($<1\text{ms}$) and zero client runtime overhead.
- **Editorial Experience**: A client-side WebAssembly (Wasm) Single Page Application built with **Leptos** (`crates/neopress-builder`). Runs directly in the administrator's browser at native 60fps with real-time drag-and-drop, reactive property panels, and zero server roundtrips during editing.

```mermaid
sequenceDiagram
    autonumber
    participant Admin as Site Administrator
    participant Leptos as Leptos Wasm SPA (neopress-builder)
    participant Axum as Neopress Server (Axum)
    participant DB as SQLite Storage
    participant Visitor as Public Site Visitor

    Admin->>Axum: GET /n-admin/builder/?post_id=42
    Axum-->>Admin: HTML Shell + neopress_builder.wasm (Cached)
    Admin->>Leptos: Initialize reactive signals & canvas
    Leptos->>Axum: GET /wp-json/neopress/v1/layout/42
    Axum->>DB: Query _neopress_layout from wp_postmeta
    DB-->>Axum: Layout JSON Schema
    Axum-->>Leptos: 200 OK (NeopressLayout JSON)
    Leptos-->>Admin: Render 60fps interactive visual canvas
    
    Admin->>Leptos: Drag & Drop Container, Edit Headings, Adjust Flex Gap
    Note over Leptos: Local reactive state mutations (undo/redo stack)
    
    Admin->>Leptos: Click "Save Page"
    Leptos->>Axum: POST /wp-json/neopress/v1/layout/42 (NeopressLayout JSON)
    Axum->>Axum: Pre-render semantic HTML from JSON Schema
    Axum->>DB: Store JSON in wp_postmeta & HTML in wp_posts.post_content
    Axum-->>Leptos: 200 OK { "status": "published" }

    Visitor->>Axum: GET /landing-page/
    Axum->>DB: Read pre-rendered post_content (<0.1ms)
    Axum-->>Visitor: Pure semantic HTML/CSS (Zero JS overhead, sub-1ms TTFB)
```

### The Canonical JSON Layout Schema (`neopress-ast/src/schema.rs`)

All pages authored or transpiled in Neopress conform to a strict, typed layout tree:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeopressLayout {
    pub version: u32,
    pub root: NeopressNode,
    pub global_styles: GlobalStyleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeopressNode {
    pub id: String,
    pub kind: NodeKind,
    pub styles: StyleProperties,
    pub children: Vec<NeopressNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum NodeKind {
    Section,
    Container,
    Grid { columns: u8, gap: u32 },
    Flex { direction: FlexDirection, justify: JustifyContent, align: AlignItems },
    Heading { text: String, level: u8 },
    Text { content_html: String },
    Image { src: String, alt: String, width: Option<u32>, height: Option<u32> },
    Button { text: String, url: String, target: String },
    Form { form_id: i64 },
    CustomHtml { raw: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleProperties {
    pub padding: Spacing,
    pub margin: Spacing,
    pub typography: TypographyConfig,
    pub background: BackgroundConfig,
    pub border: BorderConfig,
    pub responsive: ResponsiveOverrides,
}
```

### Key Advantages
1. **Total Decoupling**: Visual structure is isolated from the storage engine. Layouts can be inspected, diffed with Git, and transformed into any target format.
2. **Instant Pre-Rendering**: The server renders HTML from the JSON schema once during mutation; public reads incur zero layout computation overhead.
3. **No Shortcode Hell**: Eliminates legacy regex-based PHP shortcode parsing at request runtime.

---

## 8. Page Builder Transpilation Engine (Elementor & Divi)

To capture the 70% WordPress market share dominated by visual builders, Neopress incorporates a native **AST (Abstract Syntax Tree) Translation Layer** (`crates/neopress-ast`) directly into the WXR streaming parser (`wxr.rs`).

```mermaid
flowchart TD
    WXRItem["WXR `<item>` Stream"] --> DetectBuilder{"Detect Builder Signature"}

    DetectBuilder -->|"Meta: _elementor_data"| ElementorPipeline["Elementor AST Parser"]
    DetectBuilder -->|"Content contains `[et_pb_*`"| DiviPipeline["Divi Shortcode Tokenizer & Parser"]
    DetectBuilder -->|"Standard HTML"| StandardPipeline["Standard Gutenberg / Classic Processor"]

    subgraph Elementor Transpiler
        ElementorPipeline --> ParseJSON["Parse raw _elementor_data JSON"]
        ParseJSON --> ElemSection["Map 'section' -> NodeKind::Section"]
        ParseJSON --> ElemCol["Map 'column' -> NodeKind::Flex (Basis %)"]
        ParseJSON --> ElemWidget["Map 'widget' (heading, text, image, etc.)"]
        ElemWidget --> ElemStyle["Normalize Elementor CSS properties & responsive tokens"]
    end

    subgraph Divi Transpiler
        DiviPipeline --> DiviLexer["Tokenize Shortcode Stream: [et_pb_* ... [/et_pb_*]"]
        DiviLexer --> DiviTree["Build Divi Shortcode Tree (Section -> Row -> Column)"]
        DiviTree --> DiviLayout["Map column layouts (e.g., 4_4, 1_2_1_2, 1_3_2_3)"]
        DiviTree --> DiviModules["Map modules (et_pb_text, et_pb_image, et_pb_cta)"]
    end

    ElemStyle --> EmitCanonical["Emit Canonical NeopressNode AST"]
    DiviModules --> EmitCanonical
    StandardPipeline --> EmitCanonical

    EmitCanonical --> PersistPostMeta["Write _neopress_layout JSON to wp_postmeta"]
    EmitCanonical --> SynthesizeHTML["Synthesize Semantic HTML to wp_posts.post_content"]
```

### 1. Elementor Transpilation
Elementor stores page definitions as nested JSON inside `_elementor_data` postmeta. Neopress streams this JSON through `serde_json` and performs structural normalization:
- **Sections & Inner Sections**: Converted to `NodeKind::Section` and `NodeKind::Container`.
- **Columns**: Column widths expressed as percentages (`_column_size`) are converted to CSS Flexbox basis and Grid track tokens.
- **Widgets**: Mapped to native Neopress widgets (`heading`, `text-editor`, `image`, `icon-box`, `counter`, `accordion`, `form`).
- **Style Normalization**: Custom Elementor paddings, margins, responsive breakpoints (`tablet`, `mobile`), and typography settings are converted into canonical `StyleProperties`.

### 2. Divi Transpilation
Divi stores page layouts as deeply nested BBCode shortcodes in `post_content` (e.g. `[et_pb_section][et_pb_row][et_pb_column type="1_2"][et_pb_text]...[/et_pb_text][/et_pb_column]...[/et_pb_row][/et_pb_section]`).
- **Divi Lexer**: Tokenizes open tags, attribute keys, quoted attribute values, text bodies, and close tags.
- **Divi AST Parser**: Reconstructs the hierarchy of Section -> Row -> Column -> Module.
- **Column Layout Resolution**: Maps fractional Divi column types (`4_4`, `1_2, 1_2`, `1_3, 2_3`, `1_4, 1_4, 1_4, 1_4`) to proportional CSS Grid / Flexbox tokens in `NeopressNode`.
- **Module Attribute Mapping**: Parses colors, padding strings (`30px|||`), custom CSS rules, and animation classes directly into Neopress layout attributes.

### 3. Golden Master Transpilation Testing
Every transpilation mapping is mathematically certified through differential test suites:
- Input: Production Elementor and Divi exported XML fixtures.
- Execution: Parsed by `neopress-ast`.
- Output: Validated against golden visual snapshot assertions to ensure layout integrity.

---

## 9. Core Convergence: The Native Subsystems (Eliminating the "Plugin Tax")

A typical WordPress site incurs \$600–\$1,200/year in agency plugin licensing and suffering severe latency penalties from bloated PHP plugins. Neopress eliminates this "Plugin Tax" by embedding the core functionality directly into the compiled Rust binary:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   THE NEOPRESS NATIVE CONVERGENCE                      │
├───────────────────────┬────────────────────────────────────────────────┤
│ Legacy WordPress      │ Neopress Native Rust Subsystem                 │
├───────────────────────┼────────────────────────────────────────────────┤
│ WP Mail SMTP ($99/yr) │ native lettre SMTP/SES/SendGrid & Spool Engine │
│ Yoast SEO ($99/yr)    │ Native JSON-LD, Dynamic OG & Streaming Sitemap │
│ WPForms Pro ($199/yr) │ Declarative Forms, Honeypot & Signed Webhooks  │
│ WP Rocket ($59/yr)    │ Built-in SQLite WAL + Static Asset On-The-Fly  │
│ Cookiebot ($140/yr)   │ Native Cookieless Analytics (Zero Banner)      │
└───────────────────────┴────────────────────────────────────────────────┘
```

### 1. Native Email Subsystem (`crates/neopress-server/src/email.rs`)
Powered by the **`lettre`** crate, replacing WP Mail SMTP:
- **Multiple Transports**: Direct SMTP, Amazon SES, SendGrid, Mailgun, or local Postfix/Sendmail with automatic fallback.
- **Asynchronous Spooling**: Outbound messages are pushed to an in-memory Tokio mpsc channel and written to a SQLite spool queue (`wp_neopress_email_queue`), ensuring HTTP request handlers never block on external SMTP handshakes.
- **DKIM Signing & TLS**: Built-in cryptographic DKIM header signing using RSA/Ed25519 keys stored in `wp_options`.
- **Audit & Delivery Logging**: Complete delivery lifecycle tracking in `wp_neopress_email_log` (recipient, subject, timestamp, delivery status, latency, error details).
- **Interactive Admin View**: Configurable at `/n-admin/options-email.php` with a built-in diagnostic test email dispatcher.

### 2. Native Dynamic SEO Engine (`crates/neopress-server/src/seo.rs`)
Replaces Yoast SEO and RankMath:
- **On-the-Fly JSON-LD Generation**: Emits schema.org compliant structured data (`Article`, `BreadcrumbList`, `Organization`, `WebSite`, `Product`) tailored dynamically to each content type.
- **OpenGraph & Twitter Cards**: Generates complete meta tags (`og:title`, `og:description`, `og:image`, `twitter:card`) derived from post titles, excerpts, and featured media.
- **Streaming XML Sitemaps**: High-performance streaming sitemap endpoints (`/sitemap.xml`, `/sitemap-posts.xml`, `/sitemap-pages.xml`, `/sitemap-taxonomies.xml`) reading directly from SQLite with index caching, handling 50,000+ URLs in under 10ms.
- **Canonical URLs & Redirect Manager**: Automatically resolves trailing slash conventions, permalink updates, and custom 301/302 redirects without .htaccess rules.

### 3. Native Forms, Anti-Spam & Webhook Engine (`crates/neopress-server/src/forms.rs`)
Replaces WPForms and Gravity Forms:
- **Declarative Form Schema**: Forms are defined as typed JSON objects with client-side and server-side validation rules.
- **Submission Endpoint**: Handled via `POST /wp-json/neopress/v1/forms/{id}/submit`.
- **Zero-CAPTCHA Anti-Spam Engine**:
  1. *Invisible Honeypot Trap*: Hidden fields that bots fill but humans cannot see.
  2. *Cryptographic Time-Lock Token*: HMAC-SHA256 timestamp token verifying that submissions take $>2\text{s}$ (blocking instant bot scripts) and $<24\text{h}$ (blocking replay attacks).
  3. *IP Rate Limiting*: Leaky bucket limiter restricting submissions per IP address.
- **Multi-Action Dispatch Pipeline**: Upon successful submission, Neopress atomically:
  1. Records the submission in `wp_neopress_form_entries`.
  2. Dispatches an alert email to the site owner via the native `lettre` engine.
  3. Emits an asynchronous HTTP webhook (with HMAC-SHA256 signature header `X-Neopress-Signature`) to Zapier, Make, n8n, or agency CRMs.

---

## 10. Edge Infrastructure & Agency SaaS Multi-Tenancy

For the **\$199/month Agency SaaS** offering, Neopress is engineered to scale across distributed edge compute clusters with sub-second provisioning and continuous disaster recovery.

```mermaid
flowchart TD
    subgraph GlobalEdge ["Anycast Edge Network (Fly.io / Cloudflare)"]
        ClientReq["Client Request: client-site.com"] --> CF["Cloudflare for SaaS (Edge SSL / Custom Hostnames)"]
        CF --> FlyProxy["Fly.io Anycast Proxy (Edge Nearest Region)"]
    end

    subgraph Compute ["Neopress Micro-Instances"]
        FlyProxy --> Machine["Neopress Edge Instance (Axum)"]
        Machine --> TenantResolve{"Resolve Tenant by Host Header"}
        TenantResolve --> TenantDB[("tenants/agency_123/neopress.sqlite\nWAL Mode")]
    end

    subgraph Replication ["Litestream Continuous Replication"]
        TenantDB -.->|Stream WAL Frames| LitestreamDaemon["Litestream Process"]
        LitestreamDaemon -->|TLS / S3 API| ObjectStore[("Encrypted S3 / Cloudflare R2 Bucket")]
    end

    subgraph Management ["Agency Management API"]
        AgencyPortal["Agency SaaS Dashboard"] --> ProvisionAPI["POST /neopress/v1/saas/provision"]
        ProvisionAPI --> InitDB["Seed Template SQLite DB"]
        ProvisionAPI --> CFAPI["Cloudflare for SaaS API: Register Custom Hostname"]
        ProvisionAPI --> InitLitestream["Register Litestream Replication Target"]
    end
```

### 1. Continuous SQLite WAL Replication (Litestream)
Embedded SQLite requires robust disaster recovery on ephemeral edge nodes. Neopress couples directly with **Litestream**:
- **Continuous Frame Streaming**: Litestream monitors the SQLite `-wal` file and replicates committed transaction frames to S3/Cloudflare R2 within seconds.
- **Point-in-Time Recovery (PITR)**: Agencies can restore a client's database to any specific second in the past 30 days.
- **Sub-500ms Cold Start Recovery**: If an edge node restarts or relocates, Litestream reconstitutes the SQLite database from the latest snapshot and replayed WAL deltas before the Axum listener binds.
- **Zero Data Loss (RPO $<1\text{s}$, RTO $<5\text{s}$)**.

### 2. Multi-Tenant Custom Domains via Cloudflare for SaaS
Neopress integrates with Cloudflare for SaaS via Cloudflare API v4:
1. **Automated Hostname Registration**: When an agency adds a client domain (`example.com`), Neopress invokes the Cloudflare API to provision a Custom Hostname.
2. **Automated SSL/TLS Provisioning**: Cloudflare provisions and renews SSL certificates (Let's Encrypt / Google Trust Services) with zero manual intervention.
3. **CNAME Fallback Routing**: Clients simply configure a CNAME pointing to `origin.agency-saas.com`.
4. **Tenant Context Extraction**: An Axum middleware inspects the incoming `Host` header, maps the domain to the associated `tenant_id`, and routes queries to the corresponding isolated SQLite database.

---

## 11. Cookieless Analytics & The Leptos JS Bridge

European agencies require strict GDPR, ePrivacy Directive, and PECR compliance without annoying cookie consent banners degrading user experience. Neopress includes a native **Privacy-First Analytics Subsystem** and a **Leptos-to-JavaScript Bridge** for Google Tag Manager Consent Mode v2.

```mermaid
flowchart LR
    Visitor["Site Visitor (Page View)"] --> Axum["Axum Analytics Middleware"]
    
    subgraph PrivacyEngine ["Cookieless Hashing Engine"]
        Axum --> Extract["Extract: IP + User-Agent + Daily Salt"]
        Extract --> Hash["Compute SHA-256 (Client_IP + UA + Salt_Day)"]
        Hash --> Discard["Discard Raw IP & UA Immediately"]
        Discard --> VisitorID["Visitor Token (Valid for 24h only)"]
    end

    VisitorID --> Channel["Lock-Free Tokio Channel (mpsc)"]
    Channel --> Worker["Background Ingestion Worker"]
    Worker --> SQLiteTable[("wp_neopress_analytics_events")]

    subgraph GTMBridge ["Leptos Wasm JS Bridge"]
        LeptosBuilder["Leptos Visual Builder / SPA"] --> JSBridge["wasm-bindgen Bridge"]
        JSBridge --> Consent["gtag('consent', 'default', { ... })"]
        Consent --> DataLayer["window.dataLayer.push({ event: '...' })"]
    end
```

### 1. Native Cookieless Analytics Engine (`crates/neopress-server/src/analytics.rs`)
- **Daily Rotating Salt Hashing**:
  $$\text{VisitorToken} = \text{SHA-256}(\text{ClientIP} \parallel \text{UserAgent} \parallel \text{DailySalt} \parallel \text{SiteSecret})$$
  Because `DailySalt` changes cryptographically every 24 hours at midnight UTC, visitors cannot be tracked across different days or different websites.
- **Zero Cookies & Zero Storage**: No `Cookie`, `localStorage`, or `IndexedDB` access is used. The tracking is 100% compliant with GDPR without requiring a consent banner.
- **Lock-Free Asynchronous Ingest**: Requests dispatch analytics events into a lock-free Tokio `mpsc::channel(10000)`. A background task batches writes into SQLite every 2 seconds, maintaining sub-millisecond HTTP response latencies.
- **Built-in Dashboard**: Native visualizations in `/n-admin/analytics.php` displaying unique visitors, page views, bounce rate, referrers, device breakdown, and geo-location (derived from edge country headers).

### 2. Leptos-to-JavaScript Bridge (Google Tag Manager Consent Mode v2)
For clients that choose to integrate Google Tag Manager or external tracking:
- **Wasm FFI Interop**: `crates/neopress-builder/src/js_bridge.rs` implements typed bindings via `wasm-bindgen` and `web-sys`.
- **Default Consent Enforcement**: The initial HTML shell dispatches strict Consent Mode v2 defaults prior to any marketing scripts loading:
  ```javascript
  gtag('consent', 'default', {
    'ad_storage': 'denied',
    'analytics_storage': 'denied',
    'ad_user_data': 'denied',
    'ad_personalization': 'denied'
  });
  ```
- **Reactive Signal Synchronization**: When a user updates consent preferences within a Leptos component, reactive Rust signals immediately dispatch `gtag('consent', 'update', ...)` updates to `window.dataLayer`.

---

## 12. Security & Production Hardening

- **SQL Injection Prevention**: All queries in `db.rs` utilize parameterized statements (`params![...]` or named parameters). Dynamic string interpolation into SQL queries is strictly prohibited.
- **Path Traversal Protection**: Upload handlers sanitize filenames and restrict uploads strictly within the designated `uploads/` directory.
- **Resource Constraints**: Upload body limits and pagination limits are enforced by default to prevent denial-of-service (DoS) via unbounded memory allocations.
- **Memory Safety**: 100% Rust codebase; unsafe blocks are forbidden in application logic.
- **Zero Panics**: All recoverable errors are mapped to `AppError` and translated to standard HTTP status codes.

---

## 13. Authentication & Nonce Subsystem (`auth.rs`)

Neopress enforces write permissions across all mutating REST endpoints (`POST`, `PUT`, `DELETE`), ensuring zero-friction interoperability with standard WordPress headless clients and mobile apps:

```mermaid
flowchart TD
    Req["Incoming Mutating Request"] --> Extractor["AuthUser Axum Extractor"]
    Extractor --> CheckAuthHeader{"Authorization Header?"}
    
    CheckAuthHeader -->|"Bearer <jwt>"| VerifyJWT["Verify HS256 JWT Token & Claims"]
    CheckAuthHeader -->|"Basic <base64>"| VerifyBasic["Verify User Password or Application Password"]
    
    CheckAuthHeader -->|"None"| CheckNonce{"X-WP-Nonce Header or ?_wpnonce?"}
    CheckNonce -->|"Present"| VerifyNonce["Verify HMAC-SHA256 Nonce (12h tick)"]
    CheckNonce -->|"None"| CheckCookie{"Admin Session Cookie?"}
    
    CheckCookie -->|"Valid"| Allow["Authenticated Context (AuthUser)"]
    CheckCookie -->|"None / Invalid"| Deny["401 Unauthorized ('rest_cannot_create')"]
    
    VerifyJWT -->|"Valid"| Allow
    VerifyJWT -->|"Invalid"| Deny
    VerifyBasic -->|"Valid"| Allow
    VerifyBasic -->|"Invalid"| Deny
    VerifyNonce -->|"Valid"| Allow
    VerifyNonce -->|"Invalid"| Deny
```

### Key Capabilities
1. **Application Passwords (WP 5.6+)**: Managed via `GET/POST/DELETE /wp-json/wp/v2/users/{id}/application-passwords`. Stored securely as SHA-256 hashes in `wp_application_passwords`.
2. **Bearer JWT Tokens (`jwt-auth/v1`)**: Tokens signed via HS256 with user context, issued at `POST /wp-json/jwt-auth/v1/token` and validated at `/wp-json/jwt-auth/v1/token/validate`.
3. **Cryptographic Nonces (`X-WP-Nonce`)**: 10-character HMAC-SHA256 nonces computed against 12-hour validity ticks (with 24-hour total verification tolerance matching WordPress core).

---

## 14. Media Asset Pipeline & On-The-Fly CDN (`media.rs`)

To eliminate thumbnail generation bottlenecks during multi-gigabyte data migrations, Neopress combines asynchronous background workers with an edge-optimized on-the-fly thumbnail engine:

```mermaid
flowchart LR
    Upload["Media Upload / Ingest"] --> StoreOriginal["Store Original to disk in uploads/"]
    StoreOriginal --> SaveMeta["Save _wp_attachment_metadata to wp_postmeta"]
    SaveMeta --> SpawnBg["Background Worker (tokio spawn_blocking)"]
    
    Client["Client GET /uploads/image-300x200.jpg or .webp"] --> ExistsCheck{"File exists on disk?"}
    ExistsCheck -->|"Yes (Static Hit)"| ServeCached["Stream file with Cache-Control: public, max-age=31536000"]
    ExistsCheck -->|"No (Dynamic Crop)"| DeriveOriginal["Parse crop dimensions & locate base image"]
    DeriveOriginal --> ResizeImage["Resize / Transcode via 'image' crate"]
    ResizeImage --> WriteDisk["Persist crop to disk"]
    WriteDisk --> ServeCached
```

### Advantages
- **Instantaneous Bulk Imports**: Importing 2GB+ of images does not block requests or run out of memory.
- **Modern WebP Transcoding**: Automatically derives and serves modern `.webp` variants for all uploaded graphics.
- **Complete Path Preservation**: Standard WordPress cropped file naming (`{file}-{width}x{height}.{ext}`) is preserved for drop-in theme and frontend compatibility.

---

## 15. Streaming WXR Ingestion Subsystem (`wxr.rs`)

Rather than requiring custom JSON bundles, Neopress directly accepts native WordPress XML export files (`.xml` / WXR 1.2):

- **Event-Driven Streaming (`quick-xml`)**: Operates on a reusable byte buffer without reading the entire XML document into memory.
- **Bounded Memory Profile**: Memory consumption remains $O(1)$ regardless of whether the dump is 5MB or 2GB.
- **Atomic Persistence**: Channels, taxonomies, posts, pages, attachments, postmeta (`_thumbnail_id`, `_elementor_data`), and comments are committed inside atomic SQLite transactions.
- **Inline AST Transpilation**: During post parsing, Elementor JSON and Divi shortcodes are converted into `NeopressLayout` JSON trees before committing to disk.

---

## 16. WordPress Administration Architecture (10 Menus & 32 Submenus)

Neopress delivers 100% route and visual compatibility with the complete default WordPress administration hierarchy:

```mermaid
flowchart TD
    AdminRoot["/n-admin/ (or /wp-admin/ redirect)"] --> Shell["Two-Tier Nav Shell (layout.rs)"]
    
    Shell --> M1["1. Dashboard (Home, Updates)"]
    Shell --> M2["2. Posts (All Posts, Add New, Categories, Tags)"]
    Shell --> M3["3. Media (Library, Add New)"]
    Shell --> M4["4. Pages (All Pages, Add New)"]
    Shell --> M5["5. Comments (All Comments)"]
    Shell --> M6["6. Appearance (Themes, Customize, Widgets, Menus, Theme File Editor)"]
    Shell --> M7["7. Plugins (Installed Plugins, Add New, Plugin File Editor)"]
    Shell --> M8["8. Users (All Users, Add New, Profile)"]
    Shell --> M9["9. Tools (Available Tools, Import, Export, Site Health, Export/Erase Personal Data)"]
    Shell --> M10["10. Settings (General, Writing, Reading, Discussion, Media, Permalinks, Privacy, Email, SEO, Forms)"]
```

### Route Mapping & Implementation Ledger

| Top-Level Menu | Submenu Item | Admin Route URL | Template Handler | DB & Data Persistence |
|---|---|---|---|---|
| **Dashboard** | Home | `/n-admin/index.php` | `render_admin_dashboard` | `wp_posts`, `wp_comments`, `wp_options` |
| | Updates | `/n-admin/update-core.php` | `render_wp_updates_view` | Core release versions, schema re-sync, VACUUM |
| **Posts** | All Posts | `/n-admin/edit.php` | `render_admin_posts` | `wp_posts` (`post_type='post'`), pagination & trash |
| | Add New | `/n-admin/post-new.php` | `render_admin_post_new` | Visual Builder launch or classic authoring |
| | Categories | `/n-admin/edit-tags.php?taxonomy=category` | `render_admin_tags` | `wp_terms`, `wp_term_taxonomy` (`category`) |
| | Tags | `/n-admin/edit-tags.php?taxonomy=post_tag` | `render_admin_tags` | `wp_terms`, `wp_term_taxonomy` (`post_tag`) |
| **Media** | Library | `/n-admin/upload.php` | `render_admin_media` | `wp_posts` (`post_type='attachment'`), WebP thumbnails |
| | Add New | `/n-admin/media-new.php` | `render_wp_media_new_view` | Multi-file drag & drop upload dropzone |
| **Pages** | All Pages | `/n-admin/edit.php?post_type=page` | `render_admin_posts` | `wp_posts` (`post_type='page'`) hierarchical listing |
| | Add New | `/n-admin/post-new.php?post_type=page` | `render_admin_post_new` | Visual Builder launch or classic authoring |
| **Comments** | All Comments | `/n-admin/edit-comments.php` | `render_wp_comments_view` | `wp_comments` moderation: Approve, Spam, Trash, Delete |
| **Appearance** | Themes | `/n-admin/themes.php` | `render_wp_themes_view` | Active theme display, theme switching |
| | Customize | `/n-admin/customize.php` | `render_wp_customize_view` | Live responsive preview iframe with real-time controls |
| | Widgets | `/n-admin/widgets.php` | `render_wp_widgets_view` | Drag-and-drop widget placement for Sidebar & Footers |
| | Menus | `/n-admin/nav-menus.php` | `render_wp_nav_menus_view` | Navigation menu structure, page links, hierarchy |
| | Theme File Editor | `/n-admin/theme-editor.php` | `render_wp_theme_editor_view` | Theme source inspector with syntax highlighting |
| **Plugins** | Installed Plugins | `/n-admin/plugins.php` | `render_wp_plugins_view` | Plugin cards, active/inactive state toggles |
| | Add New | `/n-admin/plugin-install.php` | `render_wp_plugin_install_view` | Plugin search directory & `.zip` bundle upload |
| | Plugin File Editor | `/n-admin/plugin-editor.php` | `render_wp_plugin_editor_view` | Sandboxed extension source code inspector |
| **Users** | All Users | `/n-admin/users.php` | `render_admin_users` | User directory, roles, and profile management |
| | Add New | `/n-admin/user-new.php` | `render_admin_user_new` | User creation with role assignments & validation |
| | Profile | `/n-admin/profile.php` | `render_admin_profile` | User password, email, and Application Passwords |
| **Tools** | Available Tools | `/n-admin/tools.php` | `render_wp_tools_view` | System diagnostics & VACUUM storage utilities |
| | Import | `/n-admin/import.php` | `render_wp_import_view` | Streaming WordPress WXR XML & AST transpiler |
| | Export | `/n-admin/export.php` | `render_wp_export_view` | Streaming WXR XML and complete JSON backup generator |
| | Site Health | `/n-admin/site-health.php` | `render_wp_site_health_view` | Security audits, SQLite diagnostics, Litestream status |
| | Export Personal Data | `/n-admin/export-personal-data.php` | `render_wp_export_personal_data_view` | GDPR compliance personal data exporter |
| | Erase Personal Data | `/n-admin/erase-personal-data.php` | `render_wp_erase_personal_data_view` | GDPR data anonymizer across SQLite tables |
| **Settings** | General | `/n-admin/options-general.php` | `render_admin_settings` | Site title, tagline, URLs, admin email (`wp_options`) |
| | Writing | `/n-admin/options-writing.php` | `render_wp_settings_writing_view` | Default category, post formats, mail server settings |
| | Reading | `/n-admin/options-reading.php` | `render_wp_settings_reading_view` | Front page display, posts per page, RSS limits |
| | Discussion | `/n-admin/options-discussion.php` | `render_wp_settings_discussion_view` | Comments open/closed, thread nesting, avatars |
| | Media | `/n-admin/options-media.php` | `render_wp_settings_media_view` | Thumbnail, medium, and large dimensions |
| | Permalinks | `/n-admin/options-permalink.php` | `render_wp_settings_permalink_view` | URL rewrite structures (`/%postname%/`, date, numeric) |
| | Privacy | `/n-admin/options-privacy.php` | `render_wp_settings_privacy_view` | Designated privacy policy page assignment |
| | Email (Native) | `/n-admin/options-email.php` | `render_wp_settings_email_view` | SMTP/SES credentials, DKIM keys, email logs |
| | SEO (Native) | `/n-admin/options-seo.php` | `render_wp_settings_seo_view` | Meta templates, JSON-LD defaults, XML sitemaps |
| | Forms (Native) | `/n-admin/options-forms.php` | `render_wp_settings_forms_view` | Form definitions, submissions, webhook endpoints |
| | Analytics (Native)| `/n-admin/analytics.php` | `render_wp_analytics_dashboard` | Cookieless privacy metrics, visitor trends |

---

## 17. The Verification Loop (`neopress-transpiler`)

To port complex legacy WordPress business logic into Rust with mathematical certainty, Neopress uses an automated **Golden Master Verification Loop**:

```mermaid
sequenceDiagram
    autonumber
    participant Dev as Developer / CLI
    participant Agent as Transpiler Agent
    participant LLM as LLM Provider (rig-core)
    participant FS as Local Filesystem
    participant Cargo as Cargo Test Runner

    Dev->>Agent: invoke (PHP Target + Golden Master JSON)
    Agent->>LLM: Prompt with PHP code & test vectors
    LLM-->>Agent: Transpiled Rust implementation
    Agent->>FS: Write code to crates/generated_<target>/src/lib.rs
    Agent->>Cargo: Run `cargo test --manifest-path ...`
    alt All Tests Pass
        Cargo-->>Agent: Tests OK (100% Match)
        Agent-->>Dev: Success! Certified Rust Artifact Generated
    else Tests Fail
        Cargo-->>Agent: Assertion / Compile Errors
        Agent->>LLM: Self-Correction Prompt (Code + Errors)
        Note over Agent,LLM: Iterates up to --max-retries (default: 5)
    end
```

---

## 18. Strategic Roadmap & Execution Phasing

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                 STRATEGIC ROADMAP                                      │
├─────────────────────┬───────────────────────────────┬──────────────────────────────────┤
│ Phase               │ Deliverables                  │ Status                           │
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 1: Foundation │ Axum HTTP, Rusqlite WAL,      │ COMPLETED (100% Passing Tests)   │
│                     │ WP REST v2, Auth, Nonces,     │                                  │
│                     │ Media WebP Engine, 32 Admin   │                                  │
│                     │ Menus, Streaming WXR Ingest   │                                  │
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 2: Transpiler │ neopress-ast crate:           │ IN IMPLEMENTATION                │
│ & Visual Builder    │ - Elementor JSON AST parser   │ (Architecture specified)         │
│                     │ - Divi shortcode grammar      │                                  │
│                     │ - Leptos Wasm Page Builder    │                                  │
│                     │ - NeopressLayout JSON schema  │                                  │
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 3: Native     │ - lettre Email Engine         │ ARCHITECTED                      │
│ Convergence         │ - Dynamic SEO / XML Sitemaps  │                                  │
│                     │ - Declarative Forms & Webhooks│                                  │
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 4: Edge SaaS  │ - Litestream WAL Replication  │ ARCHITECTED                      │
│ & Privacy           │ - Cloudflare for SaaS API     │                                  │
│                     │ - Multi-tenant routing proxy  │                                  │
│                     │ - Cookieless daily analytics  │                                  │
│                     │ - Leptos GTM Consent Mode v2  │                                  │
└─────────────────────┴───────────────────────────────┴──────────────────────────────────┘
```
