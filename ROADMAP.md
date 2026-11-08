# Daypress Roadmap (`0.1` → `1.0`)

This document is the canonical milestone tracker for Daypress (`https://daypress.dev/#roadmap`). Agents and contributors working in this repository MUST update the checkboxes and status badges here (and sync `README.md` + `https://daypress.dev/#roadmap`) whenever a milestone or sub-feature ships.

---

## Milestone Summary

| Version | Milestone | Status |
| :--- | :--- | :--- |
| **`0.1`** | Core, Daystudio, REST API & MCP | **Shipped** |
| **`0.2`** | Static Edge Publisher, 4-Step Onboarding & AI Agent Launchpad, Self-Updater & Canvas Parity | **Shipped** |
| **`0.3.0`** | TOTP 2FA, Fine-Grained Role Capabilities, Migration Fidelity Analyzer & GHCR Docker | **Shipped** |
| **`0.4.0`** | FormStudio & Submissions Vault, Full Analytics Dashboard, Privacy/Cookie Banner & `<50µs` Hot Cache | **Shipped** |
| **`0.5.1`** | Dynamic Theme Templates, Synced Components (Symbols), Mega Menus, Media DAM & Edge TLS/R2 | **Shipped** |
| **`0.6.0`** | Custom Post Types, Structured Fields (ACF Parity) & Filterable Collection Grids | **Shipped** |
| **`0.7.0`** | SQLite FTS5 Instant Search, Auto-OG Social Cards, Rich Schema.org & Live SEO/WCAG Linter | **Shipped** |
| **`0.8.0`** | Multilingual (`i18n`), Editorial Workflows & Visual Side-by-Side AST Revision Diffs | **Shipped** |
| **`0.8.5`** | WordPress-Native Admin & Editor Ergonomics (Screen Options, Movable/Collapsible Meta Boxes & Blocks, Quick/Bulk Edit) | **Shipped** |
| **`0.9.0`** | Built-in Newsletters, Outbound Webhooks, Zero-Flicker A/B Testing & WebAssembly Crates | **Shipped** |
| **`1.0`** | First Stable Release | **Up next** |

---

## Detailed Milestone Checklists

### `0.1` — Core, Daystudio, REST API & MCP (`Shipped`)
- [x] Single Axum + Tokio executable with embedded SQLite (WAL mode)
- [x] Daystudio visual builder (`DaypressLayout` JSON AST + SSR HTML/CSS compiler)
- [x] WordPress REST API v2 subset (`/wp-json/wp/v2/*`) and native v1 routes (`/wp-json/daypress/v1/*`)
- [x] RSS 2.0 feeds, scheduled future posts, Lettre SMTP email, Argon2id authentication
- [x] Built-in Model Context Protocol (MCP) server (`POST /mcp` and `--mcp-stdio`)
- [x] Experimental WordPress JSON bundle (`daypress-migrator.zip`) & streaming 500 MB WXR XML importer

### `0.2` — Static Edge Publisher, 4-Step Onboarding & AI Agent Launchpad, Self-Updater & Canvas Parity (`Shipped`)
- [x] 4-step browser onboarding wizard (`/setup`) & AI Agent Launchpad (`/d-admin/getting-started`)
- [x] Static Edge Publisher (`/d-admin/publish`) for 1-click deploys to Cloudflare Pages (with Auto-DNS CNAME), Netlify (SHA-1 delta), or `.zip` bundles with rollback
- [x] In-app self-updater (`/d-admin/updates`) with SHA-256 verification, pre-update SQLite `VACUUM INTO` snapshot, hot-swap, and 1-click rollback
- [x] Automatic post & page revision snapshots with restore/delete and bulk cleanup
- [x] Atom 1.0 feeds (`/feed/atom/`), comment feeds (`/comments/feed/`), and configurable site timezone
- [x] Daystudio dual-theme (`Daylight` / `Night Edition`) color token pickers (`color` & `dark_color`), 1-click Topbar Light/Dark canvas preview toggle, and responsive canvas parity (`1200px` / `768px` / `375px`)

### `0.3.0` — TOTP 2FA, Fine-Grained Role Capabilities, Migration Fidelity Analyzer & GHCR Docker (`Shipped`)
- [x] RFC 6238 TOTP Two-Factor Authentication (`/d-admin/profile` & `/login/2fa`) with 8 single-use recovery codes, enforced across interactive login, JWT token exchange (`POST /wp-json/daypress/v1/auth/token/2fa`), and Basic Auth (`X-Daypress-OTP`), plus CLI reset (`--disable-2fa`)
- [x] Fine-grained WordPress-parity RBAC capabilities (`Administrator`, `Editor`, `Author`, `Contributor`, `Subscriber`) enforced across `/d-admin/*` and `/wp-json/*`
- [x] Dry-run & post-import Migration Fidelity Analyzer (`POST /wp-json/daypress/v1/import/analyze` & `/d-admin/import` scorecard)
- [x] Extended Elementor 3.16+ (`nested-tabs`, `nested-accordion`, theme builder widgets, Lottie, hotspots), Divi (`et_pb_team_member`, `et_pb_gallery`, `et_pb_shop`, `et_pb_post_nav`), and Gutenberg (`core/details`, `core/query`, `core/navigation`, `core/search`, `core/social-links`, `core/audio`) AST converters
- [x] Published multi-arch Docker images (`linux/amd64`, `linux/arm64`) on GitHub Container Registry (`ghcr.io/trooper-lab/daypress`)

---

### `0.4.0` — FormStudio & Submissions Vault, Full Analytics Dashboard, Privacy/Cookie Banner & `<50µs` Hot Cache (`Shipped`)
- [x] **All-in-One FormStudio (`/d-admin/forms` — CF7 + Ninja Forms + WPForms Parity)**:
  - [x] Multi-column rows (`1/2`, `1/3`, `2/3`), multi-step/paginated forms with progress bars, and conditional field/routing logic
  - [x] Rich field library: text, email, tel, URL, number, textarea, select, radio cards, multi-checkboxes, file uploads (MIME/size validation), date/time, star rating/NPS, range slider, calculated fields (instant quote/price estimators), hidden UTM/referrer capture, and GDPR consent checkbox
  - [x] Admin alert email + user confirmation auto-responder with `{{field}}` merge tags, custom inline success message or redirect URL, and per-form outbound webhook URL
  - [x] Anti-spam suite: honeypot + timestamp trap + IP rate-limiting + optional Cloudflare Turnstile / hCaptcha
  - [x] First-class MCP tools: `daypress_list_forms`, `daypress_get_form`, `daypress_create_form`, `daypress_update_form`, `daypress_list_form_submissions`
- [x] **Fail-Safe SQLite Submissions Vault**:
  - [x] Commit every submission to SQLite *before* SMTP send, tracking `email_status` (`sent`, `failed`, `queued`, `disabled`) and `email_error`
  - [x] 1-click **Retry / Resend Email** action in `/d-admin/forms`, inbox workflow (`Unread`, `Read`, `Starred`, `Spam`, `Trash`), search/filter, and 1-click CSV/JSON export
- [x] **Full Plausible-Style Cookieless Analytics Dashboard (`/d-admin/analytics`)**:
  - [x] Pin **Analytics** in the left `#day-sidebar` menu and add a traffic overview card on `/d-admin/`
  - [x] Interactive daily/hourly time-series chart with date-range selector (`24h`, `7d`, `30d`, `90d`, `12m`, `all`)
  - [x] Breakdown tables for Top Pages, Top Referrers, UTM Sources/Mediums/Campaigns, Countries, Devices (`Desktop` / `Mobile` / `Tablet`), and Form/Goal Conversions
  - [x] Parse and send `utm_source`, `utm_medium`, `utm_campaign` in the public analytics beacon and expose `daypress_get_analytics` over MCP
- [x] **Native Zero-Bloat GDPR / Privacy & Cookie Consent Banner**:
  - [x] Built-in customizable consent banner & script-category manager (`Necessary`, `Analytics`, `Marketing`) with zero external JS dependencies
- [x] **Production Performance & Cache Hardening**:
  - [x] Pre-compressed Gzip in-memory SSR hot cache (`<50µs` P99 latency) with ETag / `304 Not Modified`, `X-Daypress-Cache: HIT|MISS|BYPASS`, automatic invalidation, and Cloudflare CDN cache purge on save

### `0.5.1` — Dynamic Theme Templates, Synced Components (Symbols), Mega Menus, Media DAM & Edge TLS/R2 (`Shipped`)
- [x] Visual Single Post, Archive, Category, Author, and 404 templates in Daystudio with dynamic field bindings (`{{post.title}}`, `{{post.date}}`, `{{author.name}}`, `{{post.featured_image}}`)
- [x] Figma-style **Synced Global Components (Symbols)** in Daystudio with per-instance text/media/link overrides
- [x] Visual **Mega Menus** & site-wide **Announcement Bars / Slide-In Modals**
- [x] **Media Library DAM Upgrade (`/d-admin/upload`)**:
  - [x] Virtual media folders & tags
  - [x] **Replace Media File Without Changing URL/ID** (`Enable Media Replace` parity)
  - [x] Dual AVIF + WebP encoding, focal-point crop picker, sanitized SVG uploads, and Unused Asset Scanner
- [x] **External Edge & OS Packaging (`v0.4.1` / `v0.5`)**:
  - [x] Built-in automatic Let's Encrypt TLS (ACME) for direct `:443` deployment without an external reverse proxy
  - [x] S3 & Cloudflare R2 media offload + continuous SQLite replica streaming
  - [x] Code-signed release binaries and `.deb` / `.rpm` / Homebrew packages

### `0.6.0` — Custom Post Types, Structured Fields (ACF Parity) & Filterable Collection Grids (`Shipped`)
- [x] Zero-code Custom Post Type (CPT) & Custom Taxonomy builder in `/d-admin/` (`/d-admin/content-types`) with 1-click starter blueprints (Case Studies, Team, Docs, Jobs, Events, Portfolio, Testimonials)
- [x] Typed Structured Field Schemas (`text`, `textarea`, `wysiwyg`, `number`, `date`, `url`, `true_false`, `select`, `checkbox`, `media`, `post_object`, `repeater`) with dynamic `{{acf.*}}` SSR token bindings
- [x] Automatic 1-click migration of **Advanced Custom Fields (ACF)** (`acf-field-group`, `acf-field`, `acf-post-type`, `acf-taxonomy`) and **Custom Post Type UI** (`cptui_post_types`, `cptui_taxonomies`) schemas & flattened repeater postmeta
- [x] Filterable, sortable, searchable, and paginated **Daystudio Collection Grids** (`LoopGrid` / `CollectionGrid`) bound to any CPT or taxonomy

### `0.7.0` — SQLite FTS5 Instant Search, Auto-OG Social Cards, Rich Schema.org & Live SEO/WCAG Linter (`Shipped`)
- [x] Native **SQLite FTS5** full-text search index with BM25 ranking, typo tolerance, snippet highlighting, and a drop-in `Cmd+K` public search modal widget
- [x] Automatic Rust-rendered **OpenGraph Social Share Cards (`og:image`)** generated dynamically from post title, kicker, author, and brand tokens
- [x] Rich **Schema.org JSON-LD** generator (`Article`, `FAQPage`, `LocalBusiness`, `Product`, `BreadcrumbList`) + Auto Table of Contents (TOC) & reading-time widget
- [x] Real-time **Daystudio WCAG Contrast, Broken-Link & SEO Linter** inside the visual builder

### `0.8.0` — Multilingual (`i18n`), Editorial Workflows & Visual Side-by-Side AST Revision Diffs (`Shipped`)
- [x] Multi-language content linking (`/en/`, `/de/`, `/nl/`) with automatic `hreflang` sitemaps, language switcher widget, localized `/d-admin/` UI, and **WPML / Polylang** migration converter
- [x] Editorial workflow upgrades: **1-Click Post/Page Duplication**, multi-author bylines (`Co-Authors Plus` parity), **Shareable Secret Draft Preview Links** (no login required), and scheduled revisions to already-published pages
- [x] **Visual Side-by-Side AST Revision Diffs** & per-node cherry-pick restore inside Daystudio

### `0.8.5` — WordPress-Native Admin & Editor Ergonomics (`Shipped`)
- [x] **Post & Page Editor Ergonomics (`/d-admin/post-new`, `/d-admin/post-edit`)**:
  - [x] **Screen Options Drawer** (`⚙ Screen Options ▾`) with per-user meta box visibility checkboxes, 1-Column / 2-Column layout toggle, and Collapse/Expand/Reset controls persisted to `usermeta` (`_daypress_screen_prefs` via `/wp-json/daypress/v1/user-preferences`)
  - [x] **Collapsible, Reorderable & Drag-and-Drop Meta Boxes** (`↑` / `↓` / `⇄` column move / `▾` collapse + HTML5 drag-and-drop) across all main and sidebar editor panels
  - [x] **Collapsible, Drag-and-Drop & Hideable Block Stack**: Always-available block inserter (Gutenberg core blocks + custom ACF blocks), per-block `▾`/`▸` collapse, `Collapse All` / `Expand All`, and `👁`/`🙈` mute/hide block without deleting (`data-dp-hidden="true"`)
  - [x] **Distraction-Free Fullscreen Mode** (`⛶ Fullscreen`), **Sticky Post** checkbox, **Password-Protected Post** field, standalone **Discussion** meta box (`comment_status` / `ping_status`), and dedicated **Excerpt** meta box
- [x] **Posts & Pages List Table Ergonomics (`/d-admin/posts`, `/d-admin/pages`)**:
  - [x] Inline **Quick Edit** row drawer (Title, Slug, Status, Date, Author, Password, Categories, Template, Menu Order, Sticky, Allow Comments)
  - [x] Multi-row **Bulk Edit** drawer (`POST /wp-json/daypress/v1/posts/bulk-edit`), **Empty Trash** button (`POST /wp-json/daypress/v1/posts/empty-trash`), clickable column sorting (`Title`, `Author`, `Date`), Month/Category/Author filter bar, and List Table **Screen Options**
- [x] **Dashboard, Comments, Appearance & Media Library Ergonomics**:
  - [x] Collapsible, movable & hideable **Dashboard Widgets** with **Recent Comments Moderation** card and **Collapsible Left Admin Sidebar** (`64px` icon rail)
  - [x] **Comments** inline Reply, inline Quick Edit, multi-row Bulk Actions (`POST /wp-json/daypress/v1/comments/bulk`), and `Empty Spam` / `Empty Trash`
  - [x] **Menus** nested hierarchy indentation (`← Outdent` / `→ Indent`), `↑`/`↓` reorder, `New tab` toggle, and Posts/Categories quick-add; **Widgets** `↑`/`↓` reorder; **Media Library** `⊞ Grid` / `☰ List` view toggle and `☑ Bulk Select` mode

### `0.9.0` — Built-in Newsletters, Outbound Webhooks, Zero-Flicker A/B Testing & WebAssembly Crates (`Shipped`)
- [x] Native subscriber lists with double opt-in, member-gated posts, and automated new-post email digests via SMTP/SES/Resend (`MailPoet` / `Ghost` parity)
- [x] Branded shortlinks (`/go/*`) with click tracking and **HMAC-Signed Outbound Webhooks** (Slack, Discord, HubSpot, Zapier)
- [x] **Zero-Flicker Server-Side A/B Split Testing** on any Daystudio heading, CTA, or section with built-in conversion stats & Real-User Core Web Vitals (`LCP` / `INP` / `CLS`) telemetry
- [x] Sandboxed **WebAssembly (`.wasm`) Extension Crates** with explicit capability permissions

### `1.0` — First Stable Release (`Up next`)
- [ ] External third-party security audit
- [ ] Published reproducible load, latency, and memory benchmark suite (Daypress vs. WordPress + PHP-FPM/Redis)
- [ ] Stable SQLite schema & WebAssembly extension ABI backward-compatibility guarantees
- [ ] Production sign-off on **70%+ 1-click migration fidelity** across real-world Elementor, Divi, Gutenberg, and Classic WordPress websites
