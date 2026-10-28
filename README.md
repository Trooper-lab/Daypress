# Neopress

[![Neopress CI](https://github.com/Trooper-lab/neopress/actions/workflows/ci.yml/badge.svg)](https://github.com/Trooper-lab/neopress/actions/workflows/ci.yml)
[![License: AGPLv3 + Commercial](https://img.shields.io/badge/license-AGPLv3%20%7C%20Commercial-blue.svg)](LICENSE)

**Neopress is a self-hosted CMS written in Rust.** It gives you a WordPress-style admin, a visual page builder, and a WordPress-compatible REST API in a single binary with an embedded SQLite database: no PHP, no MySQL.

The long-term goal is a CMS that most WordPress sites can move to with their content and design intact. **We are not there yet.** Today Neopress is good for building new sites, and it has an experimental WordPress importer that we want to improve together with the community.

> **Status: early preview (v0.1).** Expect rough edges, missing features, and breaking changes before 1.0. Don't use it yet for sites where downtime or data loss would hurt, and keep backups. The sections below list what works and, just as important, what doesn't.

| Public site | NeoStudio visual builder |
| :---: | :---: |
| ![A page built with NeoStudio, shown on the public site](docs/screenshots/public-page.png) | ![The NeoStudio visual builder editing the same page](docs/screenshots/admin-builder.png) |

<sub>The screenshots use made-up demo content.</sub>

---

## What works today

**Running it**
- One binary built with Axum and Tokio, with an embedded SQLite database (WAL mode). Static assets are compiled into the binary.
- CI builds and tests every change on Linux and Windows.

**Writing and managing content**
- Posts, pages, categories, tags, and navigation menus.
- A post editor with visual, block, and HTML source modes.
- Media uploads, with thumbnail, medium, and WebP versions generated automatically.
- Comments with moderation and discussion settings (who can comment, manual approval).
- Users with application passwords, plus personal data export and erase tools.
- Blog index, category, tag, and author archives with pagination, plus site search.

**Designing the site**
- **NeoStudio**, a drag-and-drop page builder: sections, containers, grids, headings, text, buttons, images, video, and more. Layouts are stored as JSON and rendered on the server.
- A header and footer manager with starter templates and per-page rules.
- A customizer for colors, fonts, and custom CSS.

**Built-in site features**
- SEO fields per post, `sitemap.xml`, `robots.txt` (including "discourage search engines"), redirects, and a 404 log.
- Forms Studio, with email notifications over SMTP.
- SQLite backup snapshots you can create, download, and restore.
- Simple built-in visitor analytics.

**Security basics**
- No default password: the first start generates one (or uses `NEOPRESS_ADMIN_PASSWORD`).
- Passwords are hashed with argon2id, and failed logins are rate-limited.
- The admin area and the write APIs require authentication: a session cookie, an application password (Basic auth), or a JWT.

**APIs**
- A subset of the WordPress REST API (`/wp-json/wp/v2/`): posts, pages, media, categories, tags, comments, users, settings, menus, search, and more.
- Neopress-specific endpoints under `/wp-json/neopress/v1/` for imports, builder layouts, forms, and analytics.
- A built-in [MCP](https://modelcontextprotocol.io) server, so AI assistants such as Claude can read and edit content ([details below](#connecting-an-ai-assistant-mcp)).

**Migration from WordPress (experimental)**
- A WordPress plugin (`neopress-migrator.zip`, downloadable from **Tools → Import**) that exports content, menus, media, theme CSS, and Elementor or Divi global styles as one JSON bundle.
- An importer for standard WordPress export files (WXR), up to 500 MB.
- Elementor, Divi, and Gutenberg layouts are converted into NeoStudio layouts, and old post and term IDs are remapped, so parents, menus, featured images, and comments stay linked.
- Data from Yoast SEO, Rank Math, Contact Form 7, WPForms, Redirection, and Joinchat is converted into the matching built-in features.

![The Neopress admin dashboard](docs/screenshots/admin-dashboard.png)

---

## What doesn't work yet

This list is long on purpose. If something here matters to you, an issue or pull request is very welcome.

**Installing and updating**
- There are no prebuilt binaries, Docker image, or installer yet. You build from source with Rust.
- There is no update checker or automatic update. To update, you rebuild and restart.
- The database schema may still change before 1.0, so back up before upgrading.
- macOS is not tested in CI.

**WordPress compatibility**
- **WordPress plugins and themes do not run.** Neopress does not execute PHP. Only the plugins listed above have their data converted; everything else is lost or needs rebuilding.
- The REST API covers the common endpoints, not all of WordPress's. Tools that rely on less common endpoints may not work.
- No WooCommerce or other e-commerce.
- No multisite and no multilingual support (WPML, Polylang).

**Migration quality**
- "Most WordPress sites in one click" is the goal, not something we have measured. So far the importer is tested against four fixture sites: WordPress theme unit test data, a Gutenberg site, an Elementor starter site, and a custom theme with ACF fields. There are also unit tests for Divi layouts.
- Expect pages that look different from the original, especially with Elementor Pro widgets, Divi modules, and custom theme code. You will usually need to fix some pages by hand.
- Your [migration reports](https://github.com/Trooper-lab/neopress/issues/new/choose) are the fastest way to improve this.

**Content and publishing**
- No RSS or Atom feeds.
- No scheduled posts and no revision history.
- No "lost password" email flow. If you lose the admin password, you have to reset it in the database.
- No two-factor authentication.
- Roles are coarse: administrators manage everything, while editors, authors, and contributors can all edit and publish all content.
- The admin is English-only, with no translation system yet.

**Builder and design**
- The NeoStudio canvas preview does not exactly match the published page yet. For example, the site header shows unstyled and grids stack into one column in the canvas, while the public page renders correctly.
- A fresh install is seeded with a demo website about Neopress itself (pages and posts). Delete or replace it before building your site.

**Performance and security**
- There are no published benchmarks yet, and no caching layer or CDN integration.
- The code has not had an external security audit. Please report problems privately; see [SECURITY.md](SECURITY.md).

**Code**
- The `domain / ports / application` layering is only partly applied. Many request handlers still talk to the database adapter directly.

---

## Where help is wanted

- Prebuilt binaries and a Docker image
- RSS feeds, scheduled posts, and revisions
- Password reset by email and two-factor login
- Real per-role permissions
- Migration reports and fixes for real-world Elementor, Divi, and Gutenberg sites
- Neutral starter content for new installs
- Translations for the admin

See [CONTRIBUTING.md](CONTRIBUTING.md) to get started.

---

## Quickstart

### 1. Build and run

You need a recent stable [Rust toolchain](https://rustup.rs).

```bash
git clone https://github.com/Trooper-lab/neopress.git
cd neopress
cargo run --release -p neopress-server --bin neopress
```

By default the server listens on port 8090 and stores its database in `data/neopress.sqlite`. Change them with `--port` / `PORT` and `--db` / `DATABASE_PATH`.

- Public site: <http://localhost:8090/>
- Admin: <http://localhost:8090/n-admin/>
- NeoStudio builder: <http://localhost:8090/n-admin/builder>
- REST API: <http://localhost:8090/wp-json/>

### 2. First login

There is no default password. On first start Neopress creates an `admin` account and prints a generated password **once** to the console (stderr):

```text
🔐 Generated a password for 'admin' (shown only once):
   <generated-password>
```

Log in at <http://localhost:8090/login> and change it under **Users → Profile**. To choose the initial password yourself (for example in CI), set `NEOPRESS_ADMIN_PASSWORD` before the first start.

Repeated failed logins from the same address are throttled (5 attempts per 15 minutes per account).

### 3. Email (SMTP)

Form notifications are sent over SMTP. Configure it with environment variables and restart. Then use **Settings → General → Send test email** to check the setup. Without `NEOPRESS_SMTP_HOST` nothing is sent, and the email log records `not_sent`.

| Variable | Meaning |
| :--- | :--- |
| `NEOPRESS_SMTP_HOST` | SMTP server (required to send) |
| `NEOPRESS_SMTP_PORT` | Port (default 587; 465 for `tls`, 25 for `none`) |
| `NEOPRESS_SMTP_SECURITY` | `starttls` (default), `tls`, or `none` |
| `NEOPRESS_SMTP_USER` / `NEOPRESS_SMTP_PASSWORD` | Credentials, if your server needs them |
| `NEOPRESS_SMTP_FROM` / `NEOPRESS_SMTP_FROM_NAME` | Sender (defaults: admin email and site title) |

Set the site's admin email under **Settings → General**. A new install uses `admin@example.com` until you change it.

---

## Migrating a WordPress site (experimental)

1. In Neopress, go to **Tools → Import** and download `neopress-migrator.zip`.
2. In WordPress, go to **Plugins → Add New → Upload Plugin**, upload the zip, and activate it.
3. In WordPress, go to **Tools → Neopress Migrator** and click **Export Full JSON Bundle**.
4. Back in Neopress, upload the `.json` file under **Tools → Import**.

If you can't install plugins, you can import a standard WordPress export file (**Tools → Export → All content** in WordPress) instead. That gives you the content without the theme CSS and builder global styles.

![The import screen in the Neopress admin](docs/screenshots/admin-import.png)

---

## Connecting an AI assistant (MCP)

Neopress includes a [Model Context Protocol](https://modelcontextprotocol.io) server, so assistants such as Claude Desktop, Claude Code, or Cursor can list, write, and edit content.

**Stdio** (the assistant starts Neopress itself; build with `cargo build --release` first):

```json
{
  "mcpServers": {
    "neopress": {
      "command": "/absolute/path/to/neopress/target/release/neopress",
      "args": ["--mcp-stdio", "--db", "/absolute/path/to/neopress/data/neopress.sqlite"]
    }
  }
}
```

**HTTP** (against a running server): send JSON-RPC 2.0 requests to `POST /mcp`. This requires an administrator, for example with an application password created under **Users → Profile**:

```bash
curl -X POST http://localhost:8090/mcp \
  -u "admin:<application-password>" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

| Tool | What it does |
| :--- | :--- |
| `neopress_list_posts` | List posts or pages, filtered by status or keyword |
| `neopress_get_post` | Get one post or page by `id` or `slug` |
| `neopress_create_post` | Create a post or page, as a draft or published |
| `neopress_update_post` | Update title, content, excerpt, status, or SEO fields |
| `neopress_get_post_layout` | Read a page's NeoStudio layout (JSON) |
| `neopress_update_post_layout` | Validate and save a NeoStudio layout |
| `neopress_list_terms` | List categories or tags |
| `neopress_get_settings` | Read site settings |
| `neopress_update_option` | Change a site option, such as the title or tagline |
| `neopress_import_wp_bundle` | Import a WordPress JSON bundle |

---

## More screenshots

| Post editor | Blog index |
| :---: | :---: |
| ![The post editor](docs/screenshots/admin-editor.png) | ![The public blog index](docs/screenshots/public-home.png) |

---

## Project layout

| Path | What's in it |
| :--- | :--- |
| `crates/neopress-server` | The CMS: HTTP routes, admin and public templates, REST API, importers, MCP server, SQLite storage |
| `crates/neopress-ast` | The layout model and the Elementor, Divi, and Gutenberg converters |
| `crates/generated_slugify` | Rust ports of WordPress core functions, checked against output from the original PHP |
| `crates/neopress-transpiler` | Tooling for those PHP-to-Rust ports and their golden-master tests |
| `neopress-migrator.php` | The WordPress plugin that exports a site for import |
| `fixtures/` | Test fixtures, including the four sample WordPress sites |

More detail is in [ARCHITECTURE.md](ARCHITECTURE.md) and [API.md](API.md).

## Development

```bash
cargo test --workspace                                  # 258 tests
cargo clippy --workspace --all-targets -- -D warnings  # no warnings allowed
cargo fmt --check
```

---

## 📄 License (AGPLv3 + Commercial Dual-License)

Neopress is distributed under a **Dual-License** model:

1. **Open-Source Self-Hosted License ([GNU Affero General Public License v3.0 — AGPL-3.0-or-later](LICENSE))**:
   - Free to install, self-host, modify, and redistribute under the terms of the AGPLv3.
2. **Commercial License ([LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md))**:
   - Available for organizations embedding Neopress into proprietary software or operating closed-source managed SaaS offerings without AGPLv3 network disclosure obligations.

Neopress also includes material from WordPress (ported core functions, admin stylesheets, and the Dashicons font) that remains under the GNU GPL; see [NOTICE](NOTICE). The WordPress migrator plugin (`neopress-migrator.php`) is licensed under GPL-2.0-or-later.

"WordPress" is a trademark of the WordPress Foundation. Neopress is an independent project and is not affiliated with or endorsed by the WordPress Foundation.

---

## 🤝 Contributing & Community

- Read [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup and pull request process. Contributors sign a one-time [CLA](CLA.md).
- Ask questions in [GitHub Discussions](https://github.com/Trooper-lab/neopress/discussions), and report bugs or migration problems as [issues](https://github.com/Trooper-lab/neopress/issues/new/choose).
- Report security vulnerabilities privately, as described in [SECURITY.md](SECURITY.md).
- Everyone in the project follows our [Code of Conduct](CODE_OF_CONDUCT.md).
