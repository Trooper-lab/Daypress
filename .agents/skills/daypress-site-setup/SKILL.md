---
name: daypress-site-setup
description: >-
  Install, start, onboard, and configure a Daypress website for a user. Use this
  skill when helping a user download or run Daypress (binary, Docker, or Cargo),
  complete first-time onboarding, connect an AI assistant via MCP (--mcp-stdio or
  POST /mcp), or configure site identity, themes, brand colors, navigation menus,
  SMTP email, and TLS.
---

# Daypress Website Setup & Configuration Skill

Use this skill whenever a user asks you to **install Daypress, start their server, connect an AI agent via MCP, or configure their site's identity, theme, menus, and settings**.

---

## 1. Installing & Starting Daypress

Daypress is a single self-contained Rust binary with an embedded SQLite database (`WAL` mode). It requires **zero PHP and zero MySQL**.

### Option A: Prebuilt Binary (Linux, macOS, Windows)
Download the latest release archive from `https://github.com/Trooper-lab/daypress/releases`, extract it into a dedicated site directory, and run:
```bash
# Linux / macOS
DAYPRESS_ADMIN_PASSWORD="PickAStrongPassword123!" ./daypress --port 8090 --db data/daypress.sqlite

# Windows (PowerShell)
$env:DAYPRESS_ADMIN_PASSWORD="PickAStrongPassword123!"
.\daypress.exe --port 8090 --db data/daypress.sqlite
```
> **macOS note:** Clear Gatekeeper quarantine on first download with `xattr -d com.apple.quarantine daypress`.

### Option B: Docker Compose
```bash
git clone https://github.com/Trooper-lab/daypress.git
cd daypress
DAYPRESS_ADMIN_PASSWORD="PickAStrongPassword123!" docker compose up -d
```

### Option C: From Rust Source
```bash
cargo run --release -p daypress-server --bin daypress -- --port 8090 --db data/daypress.sqlite
```

### Key Local URLs
- **Public Website**: `http://localhost:8090/`
- **First-Time 4-Step Setup Wizard**: `http://localhost:8090/setup`
- **Admin Dashboard**: `http://localhost:8090/d-admin/`
- **AI Agent Launchpad (1-Click MCP Key Generator)**: `http://localhost:8090/d-admin/getting-started`
- **DayStudio Visual Builder**: `http://localhost:8090/d-admin/builder`
- **MCP HTTP Endpoint**: `POST http://localhost:8090/mcp`
- **WordPress REST API v2**: `http://localhost:8090/wp-json/wp/v2/`

---

## 2. Connecting Your AI Assistant via MCP

Daypress includes a native **Model Context Protocol (MCP)** server with 38 tools. You can connect via **Stdio** (local machine, no password needed) or **HTTP JSON-RPC 2.0** (local or remote server).

### Mode 1: Local Stdio MCP (`--mcp-stdio`)
When running on the same machine as the SQLite database (`data/daypress.sqlite`):

```bash
# Claude Code CLI
claude mcp add daypress -- /absolute/path/to/daypress --mcp-stdio --db /absolute/path/to/data/daypress.sqlite
```

Or in `.cursor/mcp.json`, Antigravity MCP config, or `claude_desktop_config.json`:
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

### Mode 2: HTTP MCP (`POST /mcp`)
When connecting over HTTP to a running Daypress instance:
1. Have the user open `http://localhost:8090/d-admin/getting-started` and click **⚡ Generate AI Agent Key & Config** (or `POST /wp-json/wp/v2/users/1/application-passwords` while authenticated).
2. Base64-encode `<username>:<application-password>` and register the HTTP MCP server:
   ```bash
   claude mcp add --transport http daypress http://localhost:8090/mcp \
     --header "Authorization: Basic <BASE64_TOKEN>"
   ```
3. Or invoke any MCP tool directly via `curl` / `Invoke-RestMethod`:
   ```bash
   curl -X POST http://localhost:8090/mcp \
     -u "admin:<application-password>" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"daypress_get_settings","arguments":{}}}'
   ```

---

## 3. Completing First-Time Onboarding Programmatically

If `http://localhost:8090/login` redirects to `/setup`, the site is awaiting first-time onboarding. You can complete onboarding in one `POST /setup` form request (or via MCP `daypress_update_option` calls):

```bash
curl -X POST http://localhost:8090/setup \
  -H "Content-Type: application/x-www-form-urlencoded" \
  --data-urlencode "site_title=Acme Studio" \
  --data-urlencode "tagline=Architecture & Product Design" \
  --data-urlencode "username=admin" \
  --data-urlencode "email=hello@example.com" \
  --data-urlencode "password=PickAStrongPassword123!" \
  --data-urlencode "confirm_password=PickAStrongPassword123!" \
  --data-urlencode "starter_mode=blank" \
  --data-urlencode "theme=Daypress Daylight" \
  --data-urlencode "typography_scale=serif" \
  --data-urlencode "brand_color=#a8401b" \
  --data-urlencode "network_bind_mode=localhost"
```

### Starter Modes (`starter_mode`)
- `"starter"`: Creates a designed Front Page plus a sample `"Hello world!"` post.
- `"blank"`: Creates a clean, minimal front page with zero sample blog posts—**recommended when an AI agent is building the website from scratch**.
- `"migrate"`: Prepares the site for importing a WordPress `.json` bundle or `.xml` WXR export.

---

## 4. Configuring Site Identity, Theme, Reading & Menus via MCP

Once connected via MCP, configure the site's core options using `daypress_update_option`:

### A. Site Identity & Timezone
```json
{ "name": "blogname", "value": "Acme Studio" }
{ "name": "blogdescription", "value": "Architecture & Digital Craft" }
{ "name": "siteurl", "value": "https://acmestudio.com" }
{ "name": "home", "value": "https://acmestudio.com" }
{ "name": "timezone_string", "value": "Europe/Amsterdam" }
```

### B. Built-in Theme Presets
Daypress ships with three curated editorial/studio theme atmospheres. Set both `template` and `stylesheet`:
- **`"Daypress Daylight"`**: Warm paper (`#f3eee4`), deep ink (`#16171a`), and terracotta oxide (`#a8401b`). Ideal for publications, agencies, essays, and editorial brands.
- **`"Daypress Modern"`**: Crisp alabaster (`#f8fafc`), slate (`#0f172a`), and cobalt (`#2563eb`). Ideal for SaaS, product sites, and documentation.
- **`"Daypress Night"`**: Dark obsidian (`#090d16`), crisp white (`#f8fafc`), and sky cyan (`#38bdf8`). Ideal for developer tools, AI products, and creative portfolios.

### C. Front Page & Blog Index (`show_on_front` / `page_on_front`)
To display a custom DayStudio page at `/`:
1. Create or import the home page (or pass `"set_as_front_page": true` to `daypress_import_html_design`).
2. Or set manually:
   - `daypress_update_option`: `{"name": "show_on_front", "value": "page"}`
   - `daypress_update_option`: `{"name": "page_on_front", "value": "<home_page_id>"}`
   - `daypress_update_option`: `{"name": "page_for_posts", "value": "<blog_page_id>"}`

### D. Global Design Tokens (`daypress_update_global_styles`)
Use `daypress_update_global_styles` to set site-wide `DaystudioKit` colors, typography scales, container width, and custom CSS:
```json
{
  "global_styles": {
    "content_width": 1200,
    "widgets_space": 24,
    "system_colors": [
      { "id": "primary", "title": "Primary", "color": "#16171a", "dark_color": "#f3eee4" },
      { "id": "secondary", "title": "Secondary", "color": "#5c5f66", "dark_color": "#a0a6b2" },
      { "id": "text", "title": "Text", "color": "#16171a", "dark_color": "#e5e7eb" },
      { "id": "accent", "title": "Accent", "color": "#a8401b", "dark_color": "#e06d44" }
    ],
    "system_typography": [
      { "id": "primary", "title": "Display Heading", "font_family": "'Newsreader', Georgia, serif", "font_size": "48px", "font_weight": "500", "line_height": "1.12" },
      { "id": "text", "title": "Body Copy", "font_family": "'Schibsted Grotesk', system-ui, sans-serif", "font_size": "17px", "font_weight": "400", "line_height": "1.65" }
    ],
    "button_style": {
      "background_color": "#a8401b",
      "text_color": "#ffffff",
      "border_radius": "6px"
    },
    "custom_css": "/* Optional site-wide CSS rules */"
  }
}
```

### E. Password Recovery & 2FA CLI Reset
If a user ever loses their admin password or 2FA authenticator device:
```bash
daypress reset-password admin --db data/daypress.sqlite --sign-out-everyone --disable-2fa
```
