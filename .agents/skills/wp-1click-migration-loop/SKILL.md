---
name: wp-1click-migration-loop
description: >-
  Execute and verify the end-to-end 1-Click Migration loop from a live WordPress
  instance (Docker neopress-ref-wp on :8085 or secondary-wp on :8080) to Neopress (:8090).
  Use this skill to verify content, Elementor/Divi/Gutenberg AST transpilation,
  navigation menus, media, taxonomies, and visual SSR parity.
---

# WordPress-to-Neopress 1-Click Migration Verification Loop

This skill guides the automated verification of Neopress's 1-Click Migration engine (`neopress-migrator.php` -> `/wp-json/neopress/v1/import` and `/wp-json/neopress/v1/import-wxr`).

## Reference Environments
- **Reference WordPress Container**: `neopress-ref-wp` (`http://localhost:8085`)
- **Secondary WordPress Container**: `secondary-wp` (`http://localhost:8080`)
- **Neopress Rust Server**: `http://localhost:8090`

## Verification Loop

### 1. Seed or Inspect the Source WordPress Site
Ensure the reference WordPress container has representative pages (Elementor `_elementor_data` with Flexbox containers & widgets, Divi `[et_pb_*]` shortcodes, Gutenberg blocks, hierarchical pages, categories, tags, featured images, Primary Navigation menu, and Custom CSS):
```powershell
docker cp seed_wp.php neopress-ref-wp:/var/www/html/seed_wp.php
docker exec neopress-ref-wp php /var/www/html/seed_wp.php
```

### 2. Trigger 1-Click Migration
Deploy `neopress-migrator.php` to the WordPress container (`wp-content/plugins/neopress-migrator.php`) and run the migration to Neopress:
```powershell
docker cp neopress-migrator.php neopress-ref-wp:/var/www/html/wp-content/plugins/neopress-migrator.php
```
Or export the migration bundle via PHP CLI and `POST` it to `http://localhost:8090/wp-json/neopress/v1/import`.

### 3. Differential Parity Checks
Verify across both `/wp-json/wp/v2/*` and public SSR HTML (`GET /`, `GET /{slug}/`, `GET /category/{slug}/`):
1. **Relational Integrity**:
   - `page_on_front` and `show_on_front` render the exact front page at `/`.
   - `post_parent` hierarchy matches source WordPress.
   - `_thumbnail_id` (`featured_media`) resolves to the migrated image in `/uploads/`.
   - Categories, tags, and navigation menus (`wp_nav_menus`) match source WordPress.
2. **AST Layout & Visual Parity**:
   - Elementor containers/sections/widgets and Divi shortcodes render semantic HTML (`np-section`, `np-container`, `np-grid`, `np-heading`, etc.) with global Kit CSS variables (`--e-global-color-*`) and customizer CSS.
   - Zero editor artifacts (`contenteditable="true"`, `.elementor-grid-handle`) appear on public pages.
3. **Visual Builder Roundtrip**:
   - Opening `/n-admin/builder?post={id}` loads the migrated AST with all styles intact, and saving updates both `layout_json` and `post_content` without corruption.
