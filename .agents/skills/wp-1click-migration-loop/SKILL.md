---
name: wp-1click-migration-loop
description: >-
  Migrate an existing WordPress website (Elementor, Divi, Gutenberg, Classic
  themes, ACF, Yoast/Rank Math, Contact Form 7/WPForms, WPML/Polylang) into
  Daypress and verify relational and visual parity. Use this skill both when
  helping a user import their real WordPress site and when running the core
  contributor Docker verification loop (:8085 / :8080 -> :8090).
---

# WordPress-to-Daypress 1-Click Migration & Verification Skill

Use this skill whenever a user wants to **migrate an existing WordPress website into Daypress**, or when verifying the migration engine against reference WordPress containers.

---

## Part A: Migrating a User's WordPress Website into Daypress

Daypress supports two migration formats:
1. **Full 1-Click JSON Bundle (`daypress-migrator.zip` — Recommended)**: Exports posts, pages, Custom Post Types, ACF field groups, menus, media attachments, active theme CSS, Customizer settings, and Elementor/Divi global styles in a single JSON bundle.
2. **Standard WordPress XML Export (WXR 1.2 `.xml` up to 500 MB)**: Exported from WordPress under **Tools → Export → All content** (for sites where installing a plugin isn't possible).

### Step 1: Export from WordPress
- **With Plugin (Best Fidelity)**:
  1. Download `daypress-migrator.zip` from Daypress (`http://localhost:8090/d-admin/import` or `/d-admin/api/download-migrator-plugin`, also located in the repo root).
  2. In WordPress, go to **Plugins → Add New → Upload Plugin**, upload `daypress-migrator.zip`, and activate it.
  3. Go to **Tools → Daypress Migrator** and click **Export Full JSON Bundle**.
- **Without Plugin (Standard WXR XML)**:
  1. In WordPress, go to **Tools → Export → All content** and download the `.xml` file.

### Step 2: Run Dry-Run Migration Fidelity Analyzer (Optional Pre-Check)
Before mutating SQLite, you can analyze a JSON bundle via `POST /wp-json/daypress/v1/import/analyze` (or the **Analyze Migration Fidelity** button in `/d-admin/import`) to get an `AstFidelityReport`:
- `overall_fidelity_score_pct`: Percentage of blocks converted into native DayStudio AST nodes.
- `builder_breakdown`: Counts for Elementor, Divi, Gutenberg, and Classic posts.
- `unresolved_shortcodes`: Any third-party shortcodes that may need replacing with native DayStudio blocks.

### Step 3: Execute the Import
- **Via MCP (`daypress_import_wp_bundle`)**: Pass the JSON bundle to `daypress_import_wp_bundle`.
- **Via REST API (JSON Bundle)**:
  ```bash
  curl -X POST http://localhost:8090/wp-json/daypress/v1/import \
    -u "admin:<application-password>" \
    -H "Content-Type: application/json" \
    --data-binary @wordpress-bundle.json
  ```
- **Via REST API (WXR `.xml` Stream)**:
  ```bash
  curl -X POST http://localhost:8090/wp-json/daypress/v1/import-wxr \
    -u "admin:<application-password>" \
    -F "file=@wordpress-export.xml"
  ```
- **Via Browser UI**: Upload the `.json` or `.xml` file at `http://localhost:8090/d-admin/import` (or during Step 3 of `/setup`).

### Step 4: Post-Migration Inspection & Polish with AI
What Daypress converts automatically:
- **Page Builders**: Elementor (containers, grids, Pro widgets, global Kit colors/fonts), Divi 4/5 (`[et_pb_*]` modules), and Gutenberg blocks (`core/*` + `theme.json`) are transpiled into native `DaypressLayout` AST nodes.
- **Plugins**: Advanced Custom Fields (ACF) & CPT UI schemas, Yoast SEO & Rank Math meta, Contact Form 7 & WPForms forms (into FormStudio), Redirection rules, WPML & Polylang translations, and Co-Authors Plus bylines.

After import, always help the user verify and polish their site:
1. Call `daypress_get_settings` and `daypress_list_posts` (`post_type: "any"`) to confirm `page_on_front`, menus, and all pages imported cleanly.
2. Run `daypress_audit_post` on the homepage and top landing pages to spot any broken links, missing images, or contrast issues.
3. If any page had a third-party shortcode fallback (`ShortcodeFallback`) or custom widget that needs upgrading, inspect its layout with `daypress_get_post_layout` and replace that node with a native DayStudio widget (`LoopGrid`, `Form`, `PriceCard`, `Accordion`, `Tabs`, etc.) via `daypress_update_post_layout`.

---

## Part B: Core Contributor Reference Container Verification Loop

When developing or testing Daypress's migration engine in `crates/daypress-ast` or `crates/daypress-server/src/adapters/database/migration.rs`:

### Reference Environments
- **Reference WordPress Container**: `daypress-ref-wp` (`http://localhost:8085`)
- **Secondary WordPress Container**: `secondary-wp` (`http://localhost:8080`)
- **Daypress Rust Server**: `http://localhost:8090`

### 1. Seed or Inspect the Source WordPress Site
Ensure the reference WordPress container has representative pages (Elementor `_elementor_data` with Flexbox containers & widgets, Divi `[et_pb_*]` shortcodes, Gutenberg blocks, hierarchical pages, categories, tags, featured images, Primary Navigation menu, and Custom CSS):
```powershell
docker cp seed_wp.php daypress-ref-wp:/var/www/html/seed_wp.php
docker exec daypress-ref-wp php /var/www/html/seed_wp.php
```

### 2. Trigger 1-Click Migration
Deploy `daypress-migrator.php` to the WordPress container (`wp-content/plugins/daypress-migrator.php`) and run the migration to Daypress:
```powershell
docker cp daypress-migrator.php daypress-ref-wp:/var/www/html/wp-content/plugins/daypress-migrator.php
```
Or export the migration bundle via PHP CLI and `POST` it to `http://localhost:8090/wp-json/daypress/v1/import`.

### 3. Differential Parity Checks
Verify across both `/wp-json/wp/v2/*` and public SSR HTML (`GET /`, `GET /{slug}/`, `GET /category/{slug}/`):
1. **Relational Integrity**:
   - `page_on_front` and `show_on_front` render the exact front page at `/`.
   - `post_parent` hierarchy matches source WordPress.
   - `_thumbnail_id` (`featured_media`) resolves to the migrated image in `/uploads/`.
   - Categories, tags, and navigation menus (`wp_nav_menus`) match source WordPress.
2. **AST Layout & Visual Parity**:
   - Elementor containers/sections/widgets and Divi shortcodes render semantic HTML (`dp-section`, `dp-container`, `dp-grid`, `dp-heading`, etc.) with global Kit CSS variables (`--e-global-color-*`) and customizer CSS.
   - Zero editor artifacts (`contenteditable="true"`, `.elementor-grid-handle`) appear on public pages.
3. **Visual Builder Roundtrip**:
   - Opening `/d-admin/builder?post={id}` loads the migrated AST with all styles intact, and saving updates both `layout_json` and `post_content` without corruption.
