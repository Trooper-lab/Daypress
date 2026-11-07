---
name: daypress-publish-deploy
description: >-
  Publish and deploy a user's Daypress website to production. Use this skill
  when deploying a static edge build to Cloudflare Pages, Netlify, or a local
  directory/.zip via daypress_publish_site, or when deploying the dynamic
  Daypress Rust binary/Docker container with built-in Let's Encrypt TLS,
  pre-compressed SSR hot cache, and S3/Cloudflare R2 storage.
---

# Daypress Publishing & Production Deployment Skill

Use this skill whenever a user asks you to **publish, export, host, or deploy their Daypress website to production**.

Daypress supports two production deployment architectures:
1. **Static Edge Publishing (`/d-admin/publish` & `daypress_publish_site`)**: Keep Daypress running locally (on the user's laptop/workstation with their AI agent) and push a pre-rendered static edge build to **Cloudflare Pages**, **Netlify**, or any static host / `.zip`.
2. **Dynamic Self-Hosted Rust Server (Binary or Docker)**: Run the single Daypress binary on a VPS or container platform with `<50µs` in-memory SSR hot caching, live FormStudio submissions, Cookieless Analytics, and built-in automatic Let's Encrypt TLS (`--tls-domain`).

---

## 1. Pre-Launch Checklist (Run Before Deploying)

Before publishing any site to production:
1. **Audit Key Pages (`daypress_audit_post`)**: Verify WCAG 2.1 AA/AAA contrast, heading hierarchy, SEO meta descriptions, and zero broken internal links.
2. **Verify Front Page & Permalinks (`daypress_get_settings`)**: Confirm `show_on_front` and `page_on_front` point to the intended homepage, and `blogname` / `blogdescription` are set.
3. **Check Unused Media (`daypress_scan_unused_media`)**: Optionally clean up unreferenced test uploads.

---

## 2. Mode A: Static Edge Publishing (Cloudflare Pages, Netlify, or Local/ZIP)

Daypress's built-in Static Edge Publisher crawls all published posts, pages, Custom Post Types, category/tag/author archives, RSS/Atom feeds, `sitemap.xml`, `robots.txt`, `404.html`, `_redirects`, OpenGraph social cards (`/og-image/{id}.png`), and a client-side `search-index.json`, rewriting canonical URLs to the user's `production_url`.

### Step 1: Check Current Publish Status
```json
{
  "name": "daypress_get_publish_status",
  "arguments": {}
}
```

### Step 2: Configure Target Provider & Production URL
```json
{
  "name": "daypress_publish_site",
  "arguments": {
    "provider": "cloudflare",
    "production_url": "https://example.com",
    "local_export_dir": "data/static-export"
  }
}
```
- `provider`: `"cloudflare"`, `"netlify"`, `"local"`, or `"zip"`.
- `production_url`: The canonical public domain (used to rewrite `<link rel="canonical">`, `og:url`, `sitemap.xml`, and RSS feeds).

### Step 3: Trigger Static Build & Edge Push
- **Browser UI**: Open `http://localhost:8090/d-admin/publish` to enter Cloudflare API Token / Account ID / Project Name (with 1-Click Custom Domain Auto-DNS) or Netlify Personal Access Token / Site ID, or click **Download Static Build (.zip)**.
- **REST API Trigger**:
  ```bash
  curl -X POST http://localhost:8090/wp-json/daypress/v1/publish/run \
    -u "admin:<application-password>" \
    -H "Content-Type: application/json" \
    -d '{"provider": "local", "production_url": "https://example.com"}'
  ```

---

## 3. Mode B: Dynamic Self-Hosted Server (Single Binary or Docker)

When the user wants live server-side features in production (FormStudio Submissions Vault, Cookieless Analytics ingestion, dynamic FTS5 search, comments, and live `/d-admin/` editing):

### Option 1: Direct HTTPS on Port 443 with Built-in Let's Encrypt ACME TLS
Daypress includes native Rustls ACME TLS (`TLS-ALPN-01`) so it can terminate HTTPS directly without Nginx or Caddy:
```bash
./daypress \
  --port 443 \
  --db /var/lib/daypress/daypress.sqlite \
  --tls-domain example.com \
  --tls-email admin@example.com
```

### Option 2: Docker Compose in Production
```bash
git clone https://github.com/Trooper-lab/daypress.git
cd daypress
# Create .env with DAYPRESS_ADMIN_PASSWORD and DAYPRESS_SMTP_* settings
docker compose up -d
```
All persistent state (`daypress.sqlite`, `uploads/`, and `backups/`) is stored in the `daypress-data` volume.

### Option 3: S3 / Cloudflare R2 Media Offload & Continuous SQLite Replica
Check or configure object storage offload in `/d-admin/options-media` (or inspect status via MCP `daypress_get_storage_s3_status`) to stream uploaded media and SQLite WAL snapshots to Cloudflare R2 or AWS S3.

### Option 4: Database Backups & Compaction
- Run `POST /wp-json/daypress/v1/integrity-check` and `POST /wp-json/daypress/v1/vacuum` to verify and compact the SQLite database.
- Create or download point-in-time SQLite snapshots at `/d-admin/tools` (`POST /d-admin/api/backups`).
