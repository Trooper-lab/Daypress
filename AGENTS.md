# Daypress AI Agent Guide & Workspace Engineering Rules (`AGENTS.md`)

Welcome! This file guides AI coding and website-building assistants (**Claude Code**, **Cursor**, **Antigravity**, **OpenAI Codex**, **Windsurf**, **OpenClaw**, and custom MCP clients) working in a Daypress workspace.

- **Helping a user set up, design, migrate, or publish their website?** Start with **[Part I: AI Website Setup & Site-Building Playbook](#part-i-ai-website-setup--site-building-playbook)** and the specialized skills in [`.agents/skills/`](.agents/skills/).
- **Developing or contributing to Daypress's Rust core engine?** Follow **[Part II: Core Workspace Engineering Mandates](#part-ii-core-workspace-engineering-mandates)**.

---

## Available Agent Skills (`.agents/skills/`)

Load the matching `SKILL.md` file whenever your task matches one of these workflows:

| Skill | Path | When to Use |
| :--- | :--- | :--- |
| **`daypress-site-setup`** | [`.agents/skills/daypress-site-setup/SKILL.md`](.agents/skills/daypress-site-setup/SKILL.md) | Installing, starting, onboarding, and configuring a user's Daypress site (admin account, MCP connection, theme presets, brand colors, navigation menus, SMTP, and TLS). |
| **`daystudio-website-builder`** | [`.agents/skills/daystudio-website-builder/SKILL.md`](.agents/skills/daystudio-website-builder/SKILL.md) | Designing and building pages, front pages, headers, footers, dynamic templates, and synced symbols using `daypress_import_html_design` or `DaypressLayout` AST JSON, importing all page/template images into the Media Library, and running the WCAG/SEO linter. |
| **`daypress-content-forms-i18n`** | [`.agents/skills/daypress-content-forms-i18n/SKILL.md`](.agents/skills/daypress-content-forms-i18n/SKILL.md) | Creating Custom Post Types (CPTs), ACF-parity Structured Field Groups, filterable `LoopGrid` collection grids, FormStudio forms, multilingual (`i18n`) translations, draft preview links, and scheduled revisions. |
| **`daypress-publish-deploy`** | [`.agents/skills/daypress-publish-deploy/SKILL.md`](.agents/skills/daypress-publish-deploy/SKILL.md) | Publishing a Daypress site to **Cloudflare Pages**, **Netlify**, or a local static bundle (`.zip`), or deploying the dynamic Rust binary / Docker container with automatic Let's Encrypt TLS and S3/R2 storage. |
| **`wp-1click-migration-loop`** | [`.agents/skills/wp-1click-migration-loop/SKILL.md`](.agents/skills/wp-1click-migration-loop/SKILL.md) | Migrating an existing WordPress site (Elementor, Divi, Gutenberg, Classic, ACF, Yoast, CF7, WPML/Polylang) via 1-Click JSON bundle or WXR `.xml` and verifying visual & relational parity. |
| **`php-to-rust-transpiler`** | [`.agents/skills/php-to-rust-transpiler/SKILL.md`](.agents/skills/php-to-rust-transpiler/SKILL.md) | *(Core contributors)* Translating WordPress PHP functions/classes into mathematically verified Rust crates against PHP 8.3 Golden Master vectors. |

---

# Part I: AI Website Setup & Site-Building Playbook

Daypress is a self-hosted CMS written in native Rust (`Axum` + `Rusqlite`) with an embedded SQLite database (`WAL` mode), a visual page builder (**DayStudio**), a WordPress-compatible REST API (`/wp-json/wp/v2/*`), and a **built-in Model Context Protocol (MCP) server** (`--mcp-stdio` and `POST /mcp`).

### 1. Starting Daypress & Connecting Your AI Agent

#### A. Start the Server
- **Prebuilt Binary**: Run `./daypress` (Linux/macOS) or `daypress.exe` (Windows).
- **Docker Compose**: Run `docker compose up -d`.
- **From Source**: Run `cargo run --release -p daypress-server --bin daypress`.

By default, Daypress listens on `http://localhost:8090` and stores state in `data/daypress.sqlite` and media in `uploads/`.
- To set a deterministic initial admin password on first boot, set `DAYPRESS_ADMIN_PASSWORD="<strong-password>"` before starting, or complete the 60-second browser setup wizard at `http://localhost:8090/setup`.
- Need to reset a local password from the terminal? Run:
  ```bash
  daypress reset-password admin --db data/daypress.sqlite
  ```

#### B. Connect via MCP (Stdio or HTTP)
1. **Local Stdio MCP (Zero-Auth on the Same Machine)**:
   Invoke the binary directly with `--mcp-stdio` pointing at the SQLite database:
   ```bash
   # Claude Code CLI
   claude mcp add daypress -- /path/to/daypress --mcp-stdio --db /path/to/data/daypress.sqlite
   ```
   Or in `.cursor/mcp.json` / `claude_desktop_config.json`:
   ```json
   {
     "mcpServers": {
       "daypress": {
         "command": "/absolute/path/to/daypress",
         "args": ["--mcp-stdio", "--db", "/absolute/path/to/data/daypress.sqlite"]
       }
     }
   }
   ```
2. **HTTP JSON-RPC 2.0 MCP (`POST http://localhost:8090/mcp`)**:
   - In the Daypress Admin, open **Get Started** (`/d-admin/getting-started`) and click **⚡ Generate AI Agent Key & Config** (or create an Application Password under **Users → Profile**).
   - Send JSON-RPC 2.0 calls (`initialize`, `tools/list`, `tools/call`) to `POST /mcp` with `Authorization: Basic <base64(username:app_password)>`.

---

### 2. End-to-End Recipe: Building a Website for Your User

When a user asks you to **"set up and build my website"**, follow this 6-step workflow:

1. **Inspect Current State**:
   - Call `daypress_get_settings` and `daypress_list_posts` (`post_type: "any"`, `status: "any"`) to see the current site title, active theme, front-page assignment (`page_on_front`), and existing pages.
2. **Configure Brand Identity & Global Design Tokens**:
   - Update site identity via `daypress_update_option`:
     - `blogname`: Site title
     - `blogdescription`: Tagline
     - `template` / `stylesheet`: `"Daypress Daylight"` (warm editorial paper & terracotta), `"Daypress Modern"` (crisp studio alabaster & cobalt), or `"Daypress Night"` (dark obsidian slate)
     - `timezone_string`: e.g. `"Europe/Amsterdam"` or `"America/New_York"`
   - Push brand colors, font pairings, container width, and custom CSS via `daypress_update_global_styles`.
3. **Build Site Chrome (Header, Footer & Navigation)**:
   - Either pass a full HTML document containing `<header>` and `<footer>` to `daypress_import_html_design` (`extract_header_footer: true`), **or** create dedicated `header` and `footer` Dynamic Theme Templates using `daypress_save_template` (`template_type: "header"` / `"footer"`).
4. **Design Pages & Front Page in DayStudio**:
   - **Fastest Path (`daypress_import_html_design`)**: Author a complete, responsive HTML5 + `<style>` design for each page (`Home`, `About`, `Services`, `Pricing`, `Contact`) and call `daypress_import_html_design`. Daypress transpiles the HTML into 100% native DayStudio AST nodes (`Section`, `Container`, `Grid`, `Flex`, `Heading`, `Text`, `Button`, `PriceCard`, etc.) with zero `CustomHtml` fallbacks. Pass `set_as_front_page: true` on the home page.
   - **Precision AST Editing (`daypress_get_post_layout` & `daypress_update_post_layout`)**: Fetch and mutate the `DaypressLayout` AST JSON directly to fine-tune nodes, responsive breakpoints (`tablet`, `mobile`, `hover`), or specialized widgets (`LoopGrid`, `TestimonialCarousel`, `Accordion`, `Tabs`, `Form`, `LanguageSwitcher`).
   - See [`.agents/skills/daystudio-website-builder/SKILL.md`](.agents/skills/daystudio-website-builder/SKILL.md) for the complete AST schema and widget catalog.
5. **Add Structured Content (CPTs, ACF Fields, Forms & Multilingual)**:
   - Need a portfolio, case studies, team directory, job board, docs, or events? Call `daypress_provision_cpt_preset` (`"portfolio"`, `"case_studies"`, `"team"`, `"docs"`, `"jobs"`, `"events"`, or `"testimonials"`) to provision the Custom Post Type, taxonomies, ACF field group, and templates in 1 click.
   - Need a contact, lead-gen, or multi-step wizard form? Call `daypress_create_form` and embed it on any page with a `Form` AST node (`{"type": "Form", "data": {"form_id": 1}}`) or `[daypress_form id="1"]`.
   - See [`.agents/skills/daypress-content-forms-i18n/SKILL.md`](.agents/skills/daypress-content-forms-i18n/SKILL.md).
6. **Import All Images to Media Library, Audit Quality (WCAG + SEO) & Publish**:
   - **Mandatory Media Library Check**: `daypress_import_html_design` and `daypress_update_post_layout` do NOT auto-download external `http(s)://` images or extract inline `data:image/...` URIs. Always upload every page and header/footer image to `/uploads/...` via `daypress_import_wp_bundle` (`bundle.media` with `base64_data`) and rewrite all AST/template image references to `/uploads/<file_name>`.
   - Run `daypress_audit_post` on every key page to verify WCAG 2.1 AA/AAA color contrast, Media Library attachment coverage, heading hierarchy (`H1` → `H2` → `H3`), SEO metadata, and internal links.
   - When ready to launch, use `daypress_publish_site` (or `/d-admin/publish`) to deploy a static edge build to **Cloudflare Pages**, **Netlify**, or a local folder/ZIP—or run Daypress directly with `--tls-domain`. See [`.agents/skills/daypress-publish-deploy/SKILL.md`](.agents/skills/daypress-publish-deploy/SKILL.md).

---

### 3. Complete MCP Tool Reference (47 Built-in Tools)

| Category | MCP Tool | Purpose |
| :--- | :--- | :--- |
| **Posts & Pages** | `daypress_list_posts` | List posts, pages, or CPT items (`post_type`, `status`, `search`, `limit`). |
| | `daypress_get_post` | Fetch a single post/page by `id` or `slug`. |
| | `daypress_create_post` | Create a new post, page, or CPT item (`title`, `content`, `slug`, `status`, `post_type`). |
| | `daypress_update_post` | Update `title`, `content`, `excerpt`, `slug`, `status`, `seo_title`, or `seo_description`. |
| | `daypress_duplicate_post` | Clone any post/page into a `(Copy)` draft preserving layout AST, ACF meta, and taxonomies. |
| | `daypress_list_terms` | List categories, tags, or custom taxonomy terms (`taxonomy`). |
| **DayStudio Visual Builder & Themes** | `daypress_import_html_design` | Transpile a full HTML5/CSS document into 100% native DayStudio AST blocks, extract `<header>`/`<footer>`, compile `DaystudioKit` tokens, and optionally set as Front Page. |
| | `daypress_get_post_layout` | Read a post/page's `DaypressLayout` AST JSON (`post_id`). |
| | `daypress_update_post_layout` | Validate and save a `DaypressLayout` AST JSON and synchronize pre-rendered SSR HTML. |
| | `daypress_update_global_styles` | Update `DaystudioKit` global colors, typography scales, container width, and custom CSS. |
| | `daypress_list_templates` | List Dynamic Theme Templates (`header`, `footer`, `single_post`, `single_page`, `archive`, `error_404`, `symbol`, `popup`). |
| | `daypress_save_template` | Create or update a Dynamic Theme Template, Synced Component (`symbol`), or Popup (`popup`). |
| **Site Settings** | `daypress_get_settings` | Read site title, tagline, URLs, front page settings, active theme, and navigation menus. |
| | `daypress_update_option` | Update any option in SQLite (`blogname`, `blogdescription`, `show_on_front`, `page_on_front`, etc.). |
| **Custom Post Types & ACF Fields** | `daypress_provision_cpt_preset` | 1-click blueprint provisioning (`case_studies`, `team`, `docs`, `jobs`, `events`, `portfolio`, `testimonials`). |
| | `daypress_list_post_types` | List all registered Custom Post Types and item counts. |
| | `daypress_save_post_type` | Create or update a Custom Post Type (`slug`, `singular_label`, `plural_label`, `taxonomies`, etc.). |
| | `daypress_list_field_groups` | List ACF-parity Structured Field Groups and location rules. |
| | `daypress_save_field_group` | Create or update a Structured Field Group (`text`, `wysiwyg`, `number`, `select`, `media`, `repeater`, etc.). |
| **Forms & Analytics** | `daypress_list_forms` | List all FormStudio forms, field schemas, and submission counts. |
| | `daypress_get_form` | Get a single FormStudio form by `id`. |
| | `daypress_create_form` | Create a FormStudio form (`title`, `fields`, `settings`) and return its `[daypress_form id="..."]` shortcode. |
| | `daypress_update_form` | Update an existing FormStudio form's fields or settings. |
| | `daypress_list_form_submissions` | Query the FormStudio Submissions Vault (`form_id`, `status`, `search`, `starred_only`). |
| | `daypress_get_analytics` | Read the Cookieless Analytics summary (`today`, `7d`, `30d`, `90d`, `12m`, `all`). |
| | `daypress_get_web_vitals` | Read Real-User Core Web Vitals (`LCP`, `INP`, `CLS`, `TTFB`) p75 percentiles & route ratings. |
| **Newsletters, Webhooks, A/B & Wasm (`0.9`)** | `daypress_list_subscribers` | List newsletter subscribers and mailing list statistics (`status`, `list`, `search`). |
| | `daypress_create_newsletter_campaign` | Create (and optionally send) a newsletter campaign to confirmed subscribers. |
| | `daypress_manage_shortlinks` | List, create, update, or delete `/go/*` branded shortlinks with click & UTM tracking. |
| | `daypress_manage_webhooks` | List, create, update, delete, or replay HMAC-SHA256 signed outbound webhooks. |
| | `daypress_manage_ab_tests` | List, create, update, promote winner, or delete zero-flicker server-side A/B experiments. |
| | `daypress_manage_wasm_extensions` | List, install, execute, or delete sandboxed `.wasm` extensions with fuel & capability limits. |
| **Search, SEO & WCAG Linter** | `daypress_search` | Query the SQLite FTS5 BM25 instant search index with typo tolerance and highlighted snippets. |
| | `daypress_audit_post` | Run the WCAG 2.1 AA/AAA contrast, broken internal link, heading hierarchy, and SEO linter. |
| **Multilingual & Editorial** | `daypress_list_languages` | List active site languages, default locale, and URL prefix strategy. |
| | `daypress_link_translations` | Link localized posts/pages into a shared `hreflang` translation group. |
| | `daypress_create_preview_link` | Issue a time-bound, revocable secret draft preview URL (`/preview/{token}`). |
| | `daypress_schedule_revision` | Schedule an atomic future content/AST update for an already-published post or page. |
| | `daypress_diff_revisions` | Compute a structural node-by-node AST diff between two revisions (or against live). |
| **1.0 Readiness, Security & Benchmarks** | `daypress_security_audit` | Run the 10-point Security & Production Readiness Audit, SQLite Schema v100 Contract check, Wasm ABI v1 check, and 70%+ WP Migration Corpus Sign-Off. |
| | `daypress_run_benchmark` | Run the reproducible 5-workload performance, latency (`p50`/`p95`/`p99` µs), and memory benchmark suite vs. WordPress + PHP-FPM/Redis. |
| **Media, Storage, Migration & Deploy** | `daypress_list_media_folders` | List or create virtual Media Library DAM folders (`create_name`, `parent_id`). |
| | `daypress_scan_unused_media` | Scan for unreferenced media attachments across posts, templates, and options. |
| | `daypress_get_storage_s3_status` | Read S3 / Cloudflare R2 media offload and continuous SQLite replica status. |
| | `daypress_import_wp_bundle` | Import a 1-Click WordPress Migration JSON bundle with relational ID remapping and AST transpilation. |
| | `daypress_get_publish_status` | Read Static Edge Publisher configuration and published route count. |
| | `daypress_publish_site` | Configure or trigger static edge publishing (`local`, `cloudflare`, or `netlify`). |

---

# Part II: Core Workspace Engineering Mandates

When modifying Daypress's Rust source code (`crates/*`), you MUST adhere to the following workspace rules without exception:

## Mission
Daypress is a drop-in WordPress CMS replacement written in native Rust (`Axum` + `Rusqlite`). Our primary objective is enabling **70%+ of active WordPress websites** (Elementor, Divi, Gutenberg, and Classic themes) to migrate in **1 click** and render an **exact visual and functional copy** with sub-millisecond latency.

## Core Engineering Mandates

1. **Zero Mock / Zero Toast-Only Handlers**:
   - Every Admin Portal screen (`/d-admin/*`) and REST API route (`/wp-json/wp/v2/*` and `/wp-json/daypress/v1/*`) MUST persist to and read from SQLite (`db.rs`).
   - Client-side `showToast(...)` handlers that do not make network requests to persist state are strictly forbidden.

2. **Zero Runtime Panics & Safe UTF-8 Handling**:
   - Never slice strings by raw byte index (`&s[..140]`); always use `.chars().take(N)` or `char_indices()`.
   - Never expose raw `.unwrap()` on user-controlled input, file paths, or optional database rows in HTTP request handlers.

3. **Relational Integrity in 1-Click & WXR Migrations**:
   - Both `db.rs::import_bundle` (`/wp-json/daypress/v1/import`) and `wxr.rs::import_wxr_xml` (`/wp-json/daypress/v1/import-wxr`) MUST maintain deterministic `old_post_id -> new_post_id` and `old_term_id -> new_term_id` mappings (or preserve original IDs) so `post_parent`, `page_on_front`, `page_for_posts`, `_thumbnail_id`, `categories`, `tags`, `nav_menu_item`, and `comments` never break.
   - Always run `daypress_ast::transpile_content` (with `DaystudioKit` global styles when present) during both JSON bundle imports and WXR imports.

4. **Unified Layout AST Schema (`crates/daypress-ast`)**:
   - `DaypressLayout`, `StyleProperties`, and `NodeKind` in `crates/daypress-ast/src/schema/` MUST seamlessly deserialize and serialize both nested AST style objects AND flat Visual Builder style objects (`background_color`, `text_color`, `padding_top`, etc.) as well as all Visual Builder `NodeKind` variants (`Root`, `Column`, `NativeForm`, `Benchmark`, `Image`, `Icon`, `Counter`, `Spacer`, `Accordion`, `Tabs`, `Video`, `Form`, etc.).
   - Public SSR must never leak editor-only DOM attributes (`contenteditable="true"`, `.elementor-grid-handle`).

5. **Golden Master PHP-to-Rust Verification Loop**:
   - Any WordPress core PHP function or class transpiled to Rust (`crates/generated_*`) MUST be verified against a Golden Master JSON fixture generated by executing the actual PHP code (`php` or `docker exec daypress-ref-wp php`) via `generate_golden_master.php` and `daypress-transpiler`.
   - Raw `.php` files must never be placed inside `crates/daypress-server/static/`; store reference PHP fixtures in `fixtures/`.

6. **Never Overwrite Live User Design Edits (`daypress.dev`)**:
   - Before running any website design or content updater script (`deploy_all_pages_mcp.ps1`, `update_navs_mcp.ps1`, or MCP/REST layout updates) when given a design update prompt, ALWAYS run `check_live_design_changes.ps1` to fetch and diff the live `posts` (`layout_json`, `post_modified`, and `revision` rows) and `daypress_templates` (`layout_json`, `settings_json`, `updated_at`) from `daypress.dev` first.
   - Always apply design changes incrementally on top of the live AST/template state so user edits made in the Daystudio Visual Builder are never overwritten.

7. **Canonical Roadmap & Milestone Progress Tracking (`ROADMAP.md`)**:
   - `ROADMAP.md` in the repository root is the single source of truth for the `0.1` → `1.0` release roadmap (mirroring `https://daypress.dev/#roadmap`).
   - Whenever an agent implements or ships a feature or milestone from the roadmap, it MUST update the checklist in `ROADMAP.md`, `README.md`, and `https://daypress.dev/#roadmap`.
   - Current milestone sequence:
     - **`0.1`** (`Shipped`): Core, Daystudio, REST API & MCP
     - **`0.2`** (`Shipped`): Static Edge Publisher, 4-Step Onboarding & AI Agent Launchpad, Self-Updater & Canvas Parity
     - **`0.3.0`** (`Shipped`): TOTP 2FA, Fine-Grained Role Capabilities, Migration Fidelity Analyzer & GHCR Docker
     - **`0.4.0`** (`Shipped`): FormStudio & Submissions Vault, Full Analytics Dashboard (`/d-admin/analytics`), Native Privacy/Cookie Banner & `<50µs` Hot Cache
     - **`0.5.1`** (`Shipped`): Dynamic Theme Templates, Synced Components (Symbols), Mega Menus, Media DAM & Edge TLS/R2
     - **`0.6.0`** (`Shipped`): Custom Post Types, Structured Fields (ACF Parity) & Filterable Collection Grids
     - **`0.7.0`** (`Shipped`): SQLite FTS5 Instant Search, Auto-OG Social Cards, Rich Schema.org & Live SEO/WCAG Linter
     - **`0.8.0`** (`Shipped`): Multilingual (`i18n`), Editorial Workflows & Visual Side-by-Side AST Revision Diffs
     - **`0.8.5`** (`Shipped`): WordPress-Native Admin & Editor Ergonomics (Screen Options, Movable/Collapsible Meta Boxes & Blocks, Quick/Bulk Edit)
     - **`0.9.0`** (`Shipped`): Built-in Newsletters, Outbound Webhooks, Zero-Flicker A/B Testing & WebAssembly Crates
     - **`1.0`** (`Shipped — v1.0.1`): First Stable Release (Automated Security & Readiness Audit, Reproducible Benchmarks, Stable Schema v100 / Wasm ABI v1 & 70%+ WP Migration Sign-off)
