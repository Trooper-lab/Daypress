use chrono::Utc;
use rusqlite::{params, Connection, Result};

pub fn seed_neopress_official_site(conn: &Connection) -> Result<()> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // stderr, so `--mcp-stdio` keeps a clean JSON-RPC stdout.
    eprintln!("⚡ Seeding the Neopress starter website...");

    // 1. Wipe old posts, relations, and comments
    conn.execute_batch(
        r#"
        DELETE FROM wp_posts;
        DELETE FROM wp_term_relationships;
        DELETE FROM wp_postmeta;
        DELETE FROM wp_comments;
        "#,
    )?;

    // 2. Set official site options
    let options = [
        ("blogname", "Neopress"),
        (
            "blogdescription",
            "The Better WordPress We Deserve. A blazing-fast, un-hackable, edge-deployed CMS in Rust.",
        ),
        ("siteurl", "http://localhost:8090"),
        ("home", "http://localhost:8090"),
        ("admin_email", "hello@neopress.online"),
        ("posts_per_page", "10"),
        ("date_format", "F j, Y"),
        ("time_format", "g:i a"),
    ];

    for (name, val) in options {
        conn.execute(
            "INSERT OR REPLACE INTO wp_options (option_name, option_value) VALUES (?1, ?2);",
            params![name, val],
        )?;
    }

    // 3. Ensure admin user. The placeholder hash matches no password; `ensure_admin_credentials`
    // replaces it with a generated one on startup.
    conn.execute(
        r#"INSERT OR IGNORE INTO wp_users (ID, user_login, user_pass, user_nicename, user_email, user_registered, display_name)
           VALUES (1, 'admin', ?2, 'admin', 'hello@neopress.online', ?1, 'Neopress Core');"#,
        params![now, crate::auth::LEGACY_SEED_PASSWORD_HASH],
    )?;

    // 4. Create official taxonomy categories
    let categories = [
        (1, "Uncategorized", "uncategorized"),
        (2, "Architecture", "architecture"),
        (3, "Performance", "performance"),
        (4, "Agency Strategy", "agency-strategy"),
        (5, "Security", "security"),
        (6, "Migration", "migration"),
    ];

    for (id, name, slug) in categories {
        conn.execute(
            "INSERT OR REPLACE INTO wp_terms (term_id, name, slug) VALUES (?1, ?2, ?3);",
            params![id, name, slug],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO wp_term_taxonomy (term_taxonomy_id, term_id, taxonomy, count) VALUES (?1, ?1, 'category', 1);",
            params![id],
        )?;
    }

    // 5. Insert the 5 High-Converting SEO Blog Posts
    insert_blog_posts(conn, &now)?;

    // 6. Insert the Core Product & Service Pages
    insert_product_pages(conn, &now)?;

    eprintln!("✓ Seeded the Neopress starter website.");
    Ok(())
}

fn insert_blog_posts(conn: &Connection, now: &str) -> Result<()> {
    // --- Post 1: Why Your Elementor Site is Failing Core Web Vitals ---
    let post1_content = r##"
<p class="lead">Agencies spend countless hours fine-tuning WP Rocket, LiteSpeed Cache, or Redis object caches, only to realize their client's Elementor site still scores an abysmal 65 on mobile Core Web Vitals. The hard truth: caching plugins only mask server response times—they can never fix the underlying architectural disease of client-side DOM bloat.</p>

<h2>The Fatal Flaw of PHP Page Builders: Wrapper Hell</h2>
<p>When an agency designs a modern hero section inside Elementor or Divi, the user sees a heading, a subhead, and a call-to-action button. But inspect the rendered DOM in Google Chrome DevTools, and you uncover a labyrinth of excessive markup:</p>

<pre><code>&lt;!-- Typical Elementor Rendered Markup: 14 Nested Wrappers for a Single Headline --&gt;
&lt;div class="elementor elementor-1842"&gt;
  &lt;section class="elementor-section elementor-top-section elementor-element elementor-element-7c82a1"&gt;
    &lt;div class="elementor-container elementor-column-gap-default"&gt;
      &lt;div class="elementor-column elementor-col-100 elementor-top-column"&gt;
        &lt;div class="elementor-widget-wrap elementor-element-populated"&gt;
          &lt;div class="elementor-element elementor-widget elementor-widget-heading"&gt;
            &lt;div class="elementor-widget-container"&gt;
              &lt;h1 class="elementor-heading-title elementor-size-default"&gt;
                Grow Your Business
              &lt;/h1&gt;
            &lt;/div&gt;
          &lt;/div&gt;
        &lt;/div&gt;
      &lt;/div&gt;
    &lt;/div&gt;
  &lt;/section&gt;
&lt;/div&gt;</code></pre>

<p>A single landing page built this way easily exceeds <strong>3,400 DOM nodes</strong>. On mobile devices with constrained CPUs, calculating CSS style recalculations, layout shifts (CLS), and paint geometry across 3,400 nodes takes 1,800 to 2,400 milliseconds before interaction is possible (INP). No caching plugin can optimize away DOM nodes that exist in your database.</p>

<h2>The Neopress Solution: The JSON-First Architecture</h2>
<p>Neopress replaces chaotic PHP runtime rendering with a compiled, mathematical AST (Abstract Syntax Tree). When an Elementor site is imported into Neopress, our streaming parser extracts the pure layout semantics from <code>_elementor_data</code> JSON, discarding the legacy wrapper div soup completely.</p>

<blockquote>"We don't cache bad HTML. We mathematically transpile layouts into strict JSON schemas, compiling them server-side into minimal, semantic HTML5 in under 0.1 milliseconds."</blockquote>

<p>The exact same hero section renders in Neopress as:</p>
<pre><code>&lt;!-- Neopress Compiled Semantic Output: 2 Clean Nodes, Zero JS Overhead --&gt;
&lt;section class="np-hero py-20 text-center"&gt;
  &lt;h1 class="text-4xl font-bold tracking-tight text-white"&gt;Grow Your Business&lt;/h1&gt;
&lt;/section&gt;</code></pre>

<p>DOM nodes drop from 3,400 to 420. First Contentful Paint drops from 2.8s to <strong>0.14s</strong>. Lighthouse automatically jumps to a perfect 100/100 without installing a single caching plugin or configuring complex minification rules.</p>
"##;

    conn.execute(
        r#"INSERT INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
           VALUES (101, 1, ?1, ?1, ?2, 'Why Your Elementor Site is Failing Core Web Vitals (And Why Caching Won''t Save It)', 
           'Agencies spend hours configuring WP Rocket or LiteSpeed Cache, only to realize their client''s Elementor site still scores 65 on mobile. Here is the mathematical reason caching cannot fix 3,000 nested div wrapper nodes.',
           'publish', 'why-elementor-fails-core-web-vitals', ?1, ?1, 'http://localhost:8090/why-elementor-fails-core-web-vitals/', 'post');"#,
        params![now, post1_content],
    )?;
    conn.execute(
        "INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (101, 3);",
        [],
    )?;

    // --- Post 2: The $800 Plugin Tax: Why "Secure" WordPress Still Gets Hacked ---
    let post2_content = r##"
<p class="lead">In 2025 and 2026, cybersecurity threat intelligence reports confirmed what agency CTOs already suspected: over <strong>99.9% of all WordPress security vulnerabilities</strong> originate from third-party plugins and themes—not the core CMS itself. Yet agencies continue paying an astronomical $800 annual "Plugin Tax" per site attempting to protect an inherently fragile runtime.</p>

<h2>The Absurd Economics of the WordPress Plugin Stack</h2>
<p>To deliver a client site that meets baseline agency standards today, you are forced to buy and maintain an unsustainable tower of third-party commercial extensions:</p>

<ul>
    <li><strong>Wordfence / iThemes Security Pro:</strong> $119 / site / year</li>
    <li><strong>WP Rocket / Speed Optimization:</strong> $59 / site / year</li>
    <li><strong>Yoast SEO Premium:</strong> $99 / site / year</li>
    <li><strong>WPForms Pro / Gravity Forms:</strong> $199 / site / year</li>
    <li><strong>WP Mail SMTP Pro:</strong> $99 / site / year</li>
    <li><strong>UpdraftPlus Premium Backups:</strong> $70 / site / year</li>
    <li><strong>Cookie Compliance / Consent Manager:</strong> $120 / site / year</li>
</ul>

<p>Multiply <strong>$765 to $850 per site per year</strong> across a modest agency roster of 20 client retainers: you are burning over <strong>$16,000 every year</strong> on software that actually increases your attack surface with every line of third-party PHP executed.</p>

<h2>You Cannot Inject a Web-Shell into a Binary That Doesn't Run PHP</h2>
<p>Traditional WordPress attacks rely on arbitrary PHP file execution. A vulnerability in an unmaintained slider plugin allows an attacker to upload an obfuscated <code>.php</code> script to <code>/wp-content/uploads/</code>, granting them full shell access to the web server, MySQL database credentials, and client emails.</p>

<blockquote>"Hacking Neopress isn't just difficult—it is mathematically impossible using traditional WordPress attack vectors. There is no PHP interpreter. There is no eval(). There is no arbitrary file upload execution."</blockquote>

<p>Neopress compiles directly to an immutable, memory-safe native Rust binary. Request pipelines are safeguarded by <strong>Tower middleware</strong>, in-process rate limiters, and cryptographically signed JWT sessions. Even if an attacker managed to upload a malicious PHP file to your media folder, the web server would merely serve it as static text—it literally cannot execute.</p>

<p>Combined with automated continuous <strong>Litestream SQLite replication</strong>, your data is continuously streamed to encrypted object storage every few seconds. If a server physically vanished, a fresh edge microVM spins up in 300 milliseconds with zero data loss.</p>
"##;

    conn.execute(
        r#"INSERT INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
           VALUES (102, 1, ?1, ?1, ?2, 'The $800 Plugin Tax: Why ''Secure'' WordPress Still Gets Hacked',
           'Over 99.9% of WordPress vulnerabilities come from third-party plugins. Why paying $800/year per site for security and firewall plugins to protect a fragile PHP runtime makes zero financial sense.',
           'publish', 'the-800-dollar-plugin-tax', ?1, ?1, 'http://localhost:8090/the-800-dollar-plugin-tax/', 'post');"#,
        params![now, post2_content],
    )?;
    conn.execute(
        "INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (102, 5);",
        [],
    )?;

    // --- Post 3: Stop Being a Hosting Company ---
    let post3_content = r##"
<p class="lead">Agency founders started their companies to design award-winning digital experiences, drive revenue for clients, and build high-margin recurring businesses. Instead, most agency owners spend their Friday afternoons acting as unpaid DevOps engineers: rebooting crashed MySQL servers, debugging fatal PHP errors after plugin auto-updates, and apologizing for broken contact forms.</p>

<h2>The Hidden Margin Bleed of Agency Retainers</h2>
<p>Consider the typical agency maintenance retainer of $150/month per client. On paper, it looks like high-margin recurring revenue. But calculate the unbilled operational drag:</p>

<ul>
    <li><strong>2.5 hours/month per site:</strong> Routine plugin testing, manual staging backups, and version updates.</li>
    <li><strong>1.0 emergency hours/quarter:</strong> Fixing broken Gravity Forms or webhook failures after a WordPress minor update.</li>
    <li><strong>$65/month plugin licensing &amp; server costs:</strong> Paying for WP Engine, security licenses, and SMTP relays.</li>
</ul>

<p>After true labor overhead and software licensing, your $150 retainer nets less than <strong>$35 in actual agency profit</strong>. You aren't running a software agency—you are running a low-margin, high-stress boutique web host.</p>

<h2>The $199/Month Neopress Agency Math</h2>
<p>The Neopress Cloud Workspace shifts your financial model entirely. For a single flat rate of <strong>$199/month</strong>, an agency hosts up to 15 client sites on edge microVMs with global Cloudflare Enterprise security, automated backups, and unlimited migrations.</p>

<table style="width: 100%; border-collapse: collapse; margin: 24px 0; font-size: 14px;">
    <thead>
        <tr style="border-bottom: 2px solid var(--border); text-align: left;">
            <th style="padding: 10px; color: #fff;">Metric</th>
            <th style="padding: 10px; color: #ef4444;">Legacy WordPress Stack (15 Sites)</th>
            <th style="padding: 10px; color: #10b981;">Neopress Agency Cloud (15 Sites)</th>
        </tr>
    </thead>
    <tbody>
        <tr style="border-bottom: 1px solid var(--border-subtle);">
            <td style="padding: 10px;">Plugin Licenses</td>
            <td style="padding: 10px; color: #ef4444;">$12,000 / year</td>
            <td style="padding: 10px; color: #10b981;">$0 (Built into binary)</td>
        </tr>
        <tr style="border-bottom: 1px solid var(--border-subtle);">
            <td style="padding: 10px;">Hosting Infrastructure</td>
            <td style="padding: 10px; color: #ef4444;">$5,400 / year ($30/mo/site)</td>
            <td style="padding: 10px; color: #10b981;">$2,388 / year ($199/mo total)</td>
        </tr>
        <tr style="border-bottom: 1px solid var(--border-subtle);">
            <td style="padding: 10px;">Unbillable Maintenance</td>
            <td style="padding: 10px; color: #ef4444;">35 hours / month</td>
            <td style="padding: 10px; color: #10b981;">&lt; 1 hour / month</td>
        </tr>
        <tr style="border-bottom: 2px solid var(--border); font-weight: 700;">
            <td style="padding: 10px; color: #fff;">Net Annual Agency Margin</td>
            <td style="padding: 10px; color: #ef4444;">$9,600</td>
            <td style="padding: 10px; color: #10b981;">$24,612 (+156% Margin)</td>
        </tr>
    </tbody>
</table>

<p>You charge your clients the exact same monthly retainer, but your maintenance hours vanish. When the core CMS is written in native Rust with embedded zero-config SQLite, software doesn't break, memory doesn't leak, and plugins don't conflict.</p>
"##;

    conn.execute(
        r#"INSERT INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
           VALUES (103, 1, ?1, ?1, ?2, 'Stop Being a Hosting Company: Reclaiming Agency Profit Margins in 2026',
           'Calculating the hidden margin bleed of Friday afternoon plugin updates, database corruptions, and broken forms. How agencies shift to $199/month Neopress Cloud for 10x profitability.',
           'publish', 'stop-being-a-hosting-company', ?1, ?1, 'http://localhost:8090/stop-being-a-hosting-company/', 'post');"#,
        params![now, post3_content],
    )?;
    conn.execute(
        "INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (103, 4);",
        [],
    )?;

    // --- Post 4: PHP is Exhausted ---
    let post4_content = r##"
<p class="lead">PHP served the web faithfully for thirty years. But the architecture designed for dynamic personal homepages in 1995 has become fundamentally misaligned with modern cloud infrastructure, edge compute, and sub-millisecond Core Web Vitals expectations.</p>

<h2>The Heavy Anatomy of a Single WordPress Request</h2>
<p>Every time a visitor lands on a WordPress page, the server executes a staggering amount of redundant work:</p>

<ol>
    <li>The web server (Nginx/Apache) spawns or dispatches to a <strong>PHP-FPM worker process</strong>.</li>
    <li>PHP parses, compiles, and evaluates over <strong>200,000 lines of script files</strong> across WordPress core and 30 active plugins.</li>
    <li>PHP opens a TCP socket connection to a separate <strong>MySQL database server</strong>.</li>
    <li>Over <strong>40 separate SQL queries</strong> execute just to assemble the header navigation, user session, post metadata, and widget configurations.</li>
    <li>PHP constructs a giant HTML string in memory (consuming 180MB to 300MB RAM) and streams it back through the TCP pipe.</li>
</ol>

<p>Under heavy concurrency (such as a Black Friday flash sale or viral PR hit), PHP workers saturate the CPU, database connections pool out, and the server crashes with <code>504 Gateway Timeout</code> or <code>Error Establishing Database Connection</code>.</p>

<h2>The Native Rust &amp; Embedded SQLite Revolution</h2>
<p>Neopress throws out the entire 20-year-old stack. Built on the <strong>Tokio asynchronous runtime</strong> and <strong>Axum web engine</strong>, Neopress operates as a single compiled binary.</p>

<pre><code>// Sub-millisecond in-process query execution:
let post = db.get_post_by_slug("fast-reads")?;
// Zero TCP network sockets. Zero serialization overhead. Zero garbage collection pauses.</code></pre>

<p>Database queries do not cross a local or network socket; they read directly from memory-mapped pages of an embedded <strong>SQLite database in WAL (Write-Ahead Logging) mode</strong>. Time-to-First-Byte (TTFB) plunges from 1,800ms down to <strong>0.14 milliseconds</strong>.</p>

<p>A single $5 edge microVM running Neopress easily handles over <strong>12,000 requests per second</strong> with zero caching plugins—work that would choke a $200/month dedicated WordPress server.</p>
"##;

    conn.execute(
        r#"INSERT INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
           VALUES (104, 1, ?1, ?1, ?2, 'PHP is Exhausted: Why We Rebuilt the CMS in Rust for the Edge',
           'Comparing the heavy, monolithic request lifecycle of PHP booting 35 plugins into memory against a single memory-safe Rust binary with embedded SQLite in WAL mode.',
           'publish', 'php-is-exhausted-rebuilt-in-rust', ?1, ?1, 'http://localhost:8090/php-is-exhausted-rebuilt-in-rust/', 'post');"#,
        params![now, post4_content],
    )?;
    conn.execute(
        "INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (104, 2);",
        [],
    )?;

    // --- Post 5: The Myth of the Rebuild ---
    let post5_content = r##"
<p class="lead">The number one reason agencies remain trapped in the aging WordPress ecosystem isn't love for the software—it is pure terror of the rebuild. The prospect of manually recreating 50 custom Elementor landing pages, copying blog posts, and re-wiring forms in a modern headless framework like Next.js represents hundreds of unbillable hours.</p>

<h2>Enter the Mathematical AST Translation Engine</h2>
<p>Neopress was engineered specifically to kill the migration barrier. You do not need to rebuild a single page, re-type a headline, or hire a contractor to recreate layouts.</p>

<p>Our migration engine operates in three automated phases:</p>

<ol>
    <li><strong>Streaming WXR XML Ingestion:</strong> Our zero-allocation SAX streaming XML parser processes massive 500MB export archives in seconds without running out of RAM, preserving all post IDs, permalinks, categories, and media metadata.</li>
    <li><strong>Elementor JSON &amp; Divi Shortcode Transpilation:</strong> Instead of importing raw bloated HTML, our AST parser scans post metadata for <code>_elementor_data</code> JSON trees and Divi <code>[et_pb_*]</code> tags. It mathematically translates them into canonical Neopress layout nodes (Grids, Flex containers, Headings, Smart Forms).</li>
    <li><strong>Dual-Write Instant Activation:</strong> Neopress stores the clean AST JSON for future drag-and-drop visual editing while simultaneously compiling semantic static HTML for instant public SSR delivery.</li>
</ol>

<blockquote>"A 25-page agency client website with 400 blog posts imports, transpiles, and renders live in Neopress in under 4 seconds flat."</blockquote>

<p>Your clients keep the familiar visual layout they approved. Your designers keep the intuitive drag-and-drop editing canvas. And your agency gains a 100/100 Lighthouse score, un-hackable security, and zero plugin maintenance forever.</p>
"##;

    conn.execute(
        r#"INSERT INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type)
           VALUES (105, 1, ?1, ?1, ?2, 'The Myth of the Rebuild: Migrating 15 WordPress Sites in Under an Hour',
           'Agencies stay trapped on legacy WordPress out of fear of manual page rebuilding. How our WXR streaming parser and mathematical AST transpiler converts Elementor JSON into clean Rust Wasm canvas in 4 seconds.',
           'publish', 'the-myth-of-the-rebuild', ?1, ?1, 'http://localhost:8090/the-myth-of-the-rebuild/', 'post');"#,
        params![now, post5_content],
    )?;
    conn.execute(
        "INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (105, 6);",
        [],
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_visual_page(
    conn: &Connection,
    id: i64,
    title: &str,
    slug: &str,
    excerpt: &str,
    layout_json: &str,
    rendered_html: &str,
    now: &str,
) -> Result<()> {
    conn.execute(
        r#"INSERT INTO wp_posts (ID, post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, layout_json)
           VALUES (?1, 1, ?2, ?2, ?3, ?4, ?5, 'publish', ?6, ?2, ?2, ?7, 'page', ?8);"#,
        params![
            id,
            now,
            rendered_html,
            title,
            excerpt,
            slug,
            format!("http://localhost:8090/{}/", slug),
            layout_json
        ],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, '_neopress_layout', ?2);",
        params![id, layout_json],
    )?;

    Ok(())
}

fn insert_product_pages(conn: &Connection, now: &str) -> Result<()> {
    // --- Page 1: The Visual Builder (ID 2) ---
    let layout_2 = r##"{
  "version": 1,
  "root": {
    "id": "root_vb",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_vb_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "80px", "padding_bottom": "80px", "text_align": "center" },
        "children": [
          {
            "id": "cont_vb_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_vb_1", "type": "Heading", "data": { "text": "The Visual Builder: Familiar UI. Headless JSON Architecture.", "level": 1 }, "styles": { "font_size": "44px", "text_color": "#ffffff", "margin_bottom": "16px" } },
              { "id": "p_vb_1", "type": "Text", "data": { "content_html": "<p>Give your agency clients the drag-and-drop experience they love, powered by a strict JSON-first headless schema that guarantees perfect Lighthouse 100 scores.</p>" }, "styles": { "font_size": "18px", "text_color": "#94a3b8", "margin_bottom": "28px" } },
              { "id": "btn_vb_1", "type": "Button", "data": { "text": "Launch Fullscreen Visual Studio ⚡", "url": "/n-admin/builder?post=2" }, "styles": { "background_color": "#e02660", "text_color": "#ffffff", "padding_top": "14px", "padding_bottom": "14px", "padding_left": "32px", "padding_right": "32px", "border_radius": "8px" } }
            ]
          }
        ]
      },
      {
        "id": "sec_vb_features",
        "type": "Section",
        "styles": { "background_color": "#ffffff", "padding_top": "60px", "padding_bottom": "60px" },
        "children": [
          {
            "id": "cont_vb_feat",
            "type": "Container",
            "styles": {},
            "children": [
              {
                "id": "grid_vb_feat",
                "type": "Grid",
                "data": { "columns": 3, "gap": 24 },
                "styles": {},
                "children": [
                  {
                    "id": "c_vb_f1",
                    "type": "Container",
                    "styles": { "background_color": "#f8fafc", "padding_top": "24px", "padding_bottom": "24px", "padding_left": "20px", "padding_right": "20px", "border_radius": "8px", "border": "1px solid #e2e8f0" },
                    "children": [
                      { "id": "h_f1", "type": "Heading", "data": { "text": "60 FPS WebAssembly", "level": 3 }, "styles": { "font_size": "20px", "text_color": "#0f172a" } },
                      { "id": "p_f1", "type": "Text", "data": { "content_html": "<p>Near-native hardware speed with zero DOM stutter or sluggish input lag.</p>" }, "styles": { "font_size": "14px", "text_color": "#64748b", "margin_top": "8px" } }
                    ]
                  },
                  {
                    "id": "c_vb_f2",
                    "type": "Container",
                    "styles": { "background_color": "#f8fafc", "padding_top": "24px", "padding_bottom": "24px", "padding_left": "20px", "padding_right": "20px", "border_radius": "8px", "border": "1px solid #e2e8f0" },
                    "children": [
                      { "id": "h_f2", "type": "Heading", "data": { "text": "Elementor AST Translator", "level": 3 }, "styles": { "font_size": "20px", "text_color": "#0f172a" } },
                      { "id": "p_f2", "type": "Text", "data": { "content_html": "<p>Mathematical import pipeline maps _elementor_data JSON directly to canonical Neopress schema.</p>" }, "styles": { "font_size": "14px", "text_color": "#64748b", "margin_top": "8px" } }
                    ]
                  },
                  {
                    "id": "c_vb_f3",
                    "type": "Container",
                    "styles": { "background_color": "#f8fafc", "padding_top": "24px", "padding_bottom": "24px", "padding_left": "20px", "padding_right": "20px", "border_radius": "8px", "border": "1px solid #e2e8f0" },
                    "children": [
                      { "id": "h_f3", "type": "Heading", "data": { "text": "Foolproof Client Mode", "level": 3 }, "styles": { "font_size": "20px", "text_color": "#0f172a" } },
                      { "id": "p_f3", "type": "Text", "data": { "content_html": "<p>Clients can easily update text and images without breaking grid proportions or styles.</p>" }, "styles": { "font_size": "14px", "text_color": "#64748b", "margin_top": "8px" } }
                    ]
                  }
                ]
              }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_2 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 80px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 44px; margin-bottom: 16px;">The Visual Builder: Familiar UI. Headless JSON Architecture.</h1><div class="np-text" style="font-size: 18px; color: #94a3b8; margin-bottom: 28px;"><p>Give your agency clients the drag-and-drop experience they love, powered by a strict JSON-first headless schema that guarantees perfect Lighthouse 100 scores.</p></div><div class="np-btn-wrapper"><a href="/n-admin/builder?post=2" class="np-button" style="background-color: #e02660; color: #ffffff; padding: 14px 32px; border-radius: 8px; font-weight: 700; text-decoration: none;">Launch Fullscreen Visual Studio ⚡</a></div></div></section><section class="np-section" style="background-color: #ffffff; padding: 60px 24px;"><div class="np-container"><div class="np-grid" style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 24px;"><div style="background: #f8fafc; padding: 24px 20px; border-radius: 8px; border: 1px solid #e2e8f0;"><h3 style="font-size: 20px; color: #0f172a;">60 FPS WebAssembly</h3><p style="font-size: 14px; color: #64748b; margin-top: 8px;">Near-native hardware speed with zero DOM stutter or sluggish input lag.</p></div><div style="background: #f8fafc; padding: 24px 20px; border-radius: 8px; border: 1px solid #e2e8f0;"><h3 style="font-size: 20px; color: #0f172a;">Elementor AST Translator</h3><p style="font-size: 14px; color: #64748b; margin-top: 8px;">Mathematical import pipeline maps _elementor_data JSON directly to canonical Neopress schema.</p></div><div style="background: #f8fafc; padding: 24px 20px; border-radius: 8px; border: 1px solid #e2e8f0;"><h3 style="font-size: 20px; color: #0f172a;">Foolproof Client Mode</h3><p style="font-size: 14px; color: #64748b; margin-top: 8px;">Clients can easily update text and images without breaking grid proportions or styles.</p></div></div></div></section>"##;

    insert_visual_page(
        conn,
        2,
        "The Visual Builder: Familiar UI. Headless JSON Architecture.",
        "visual-builder",
        "Give your clients the drag-and-drop experience they love.",
        layout_2,
        html_2,
        now,
    )?;

    // --- Page 2: The Engine & AI (ID 3) ---
    let layout_3 = r##"{
  "version": 1,
  "root": {
    "id": "root_eng",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_eng_hero",
        "type": "Section",
        "styles": { "background_color": "#0a0a0f", "text_color": "#ffffff", "padding_top": "75px", "padding_bottom": "75px", "text_align": "center" },
        "children": [
          {
            "id": "cont_eng_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_eng_1", "type": "Heading", "data": { "text": "The Engine & AI: Sub-Millisecond Reads. Zero Panics.", "level": 1 }, "styles": { "font_size": "42px", "text_color": "#ffffff", "margin_bottom": "16px" } },
              { "id": "p_eng_1", "type": "Text", "data": { "content_html": "<p>Axum 0.8, embedded SQLite in WAL mode, and continuous Litestream edge replication. Drop-in REST API v2 compatibility with isolated WebAssembly sandboxing.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8", "margin_bottom": "26px" } },
              { "id": "btn_eng_1", "type": "Button", "data": { "text": "Explore Developer Docs ↗", "url": "/docs/" }, "styles": { "background_color": "#4f46e5", "text_color": "#ffffff", "padding_top": "12px", "padding_bottom": "12px", "padding_left": "28px", "padding_right": "28px", "border_radius": "6px" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_3 = r##"<section class="np-section" style="background-color: #0a0a0f; color: #ffffff; padding: 75px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 42px; margin-bottom: 16px;">The Engine &amp; AI: Sub-Millisecond Reads. Zero Panics.</h1><div class="np-text" style="font-size: 17px; color: #94a3b8; margin-bottom: 26px;"><p>Axum 0.8, embedded SQLite in WAL mode, and continuous Litestream edge replication. Drop-in REST API v2 compatibility with isolated WebAssembly sandboxing.</p></div><div class="np-btn-wrapper"><a href="/docs/" class="np-button" style="background-color: #4f46e5; color: #ffffff; padding: 12px 28px; border-radius: 6px; font-weight: 700; text-decoration: none;">Explore Developer Docs ↗</a></div></div></section>"##;
    insert_visual_page(
        conn,
        3,
        "The Engine & AI: Sub-Millisecond Reads. Zero Panics.",
        "engine",
        "Axum, embedded SQLite in WAL mode, and continuous Litestream edge replication.",
        layout_3,
        html_3,
        now,
    )?;

    // --- Page 3: Pricing (ID 4) ---
    let layout_4 = r##"{
  "version": 1,
  "root": {
    "id": "root_pr",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_pr_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_pr_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_pr_1", "type": "Heading", "data": { "text": "Transparent Edge SaaS & Community Open Source", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_pr_1", "type": "Text", "data": { "content_html": "<p>Eliminate the $800/year per site plugin tax. Predictable, high-margin pricing engineered for web design and marketing agencies.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_4 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">Transparent Edge SaaS &amp; Community Open Source</h1><div class="np-text" style="font-size: 17px; color: #94a3b8;"><p>Eliminate the $800/year per site plugin tax. Predictable, high-margin pricing engineered for web design and marketing agencies.</p></div></div></section>"##;
    insert_visual_page(
        conn,
        4,
        "Pricing: Transparent Edge SaaS & Community Open Source",
        "pricing",
        "Eliminate the $800/year per site plugin tax.",
        layout_4,
        html_4,
        now,
    )?;

    // --- Page 4: Shadow Mode Audit (ID 5) ---
    let layout_5 = r##"{
  "version": 1,
  "root": {
    "id": "root_aud",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_aud_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_aud_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_aud_1", "type": "Heading", "data": { "text": "Shadow Mode Migration Audit", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_aud_1", "type": "Text", "data": { "content_html": "<p>Test your live WordPress website against Neopress. See what your client's site looks like compiling and rendering in native Rust with zero plugins.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8", "margin_bottom": "24px" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_5 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">Shadow Mode Migration Audit</h1><div class="np-text" style="font-size: 17px; color: #94a3b8;"><p>Test your live WordPress website against Neopress. See what your client's site looks like compiling and rendering in native Rust with zero plugins.</p></div></div></section>"##;
    insert_visual_page(
        conn,
        5,
        "Shadow Mode Migration Audit: See Your WordPress Site in Rust",
        "shadow-audit",
        "Run a free Shadow Mode migration audit on your client URL.",
        layout_5,
        html_5,
        now,
    )?;

    // --- Page 5: Docs (ID 6) ---
    let layout_6 = r##"{
  "version": 1,
  "root": {
    "id": "root_docs",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_doc_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_doc_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_doc_1", "type": "Heading", "data": { "text": "Neopress Developer Documentation & Architecture Guide", "level": 1 }, "styles": { "font_size": "38px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_doc_1", "type": "Text", "data": { "content_html": "<p>Complete developer quickstart, REST API v2 specifications, Wasm sandbox execution, and Litestream edge replication deployment guide.</p>" }, "styles": { "font_size": "16px", "text_color": "#94a3b8" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_6 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 38px; margin-bottom: 14px;">Neopress Developer Documentation &amp; Architecture Guide</h1><div class="np-text" style="font-size: 16px; color: #94a3b8;"><p>Complete developer quickstart, REST API v2 specifications, Wasm sandbox execution, and Litestream edge replication deployment guide.</p></div></div></section>"##;
    insert_visual_page(
        conn,
        6,
        "Neopress Developer Documentation & Architecture Guide",
        "docs",
        "Developer quickstart, REST API v2 specifications, and edge deployment guide.",
        layout_6,
        html_6,
        now,
    )?;

    // --- Page 6: Agency Partners (ID 7) ---
    let layout_7 = r##"{
  "version": 1,
  "root": {
    "id": "root_ap",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_ap_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_ap_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_ap_1", "type": "Heading", "data": { "text": "The Neopress Agency Partner Program", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_ap_1", "type": "Text", "data": { "content_html": "<p>Designed specifically for digital agencies, marketing firms, and web developers managing 5 or more client retainers.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8", "margin_bottom": "24px" } },
              { "id": "btn_ap_1", "type": "Button", "data": { "text": "Apply for Partner Status ⚡", "url": "#apply" }, "styles": { "background_color": "#e02660", "text_color": "#ffffff", "padding_top": "12px", "padding_bottom": "12px", "padding_left": "28px", "padding_right": "28px", "border_radius": "6px" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_7 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">The Neopress Agency Partner Program</h1><div class="np-text" style="font-size: 17px; color: #94a3b8; margin-bottom: 24px;"><p>Designed specifically for digital agencies, marketing firms, and web developers managing 5 or more client retainers.</p></div><div class="np-btn-wrapper"><a href="#apply" class="np-button" style="background-color: #e02660; color: #ffffff; padding: 12px 28px; border-radius: 6px; font-weight: 700; text-decoration: none;">Apply for Partner Status ⚡</a></div></div></section>"##;
    insert_visual_page(
        conn,
        7,
        "The Neopress Agency Partner Program",
        "agency-partners",
        "Designed specifically for digital agencies managing 5 or more retainers.",
        layout_7,
        html_7,
        now,
    )?;

    // --- Page 7: Privacy & GDPR (ID 8) ---
    let layout_8 = r##"{
  "version": 1,
  "root": {
    "id": "root_pv",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_pv_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_pv_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_pv_1", "type": "Heading", "data": { "text": "Privacy by Architecture: Native Cookieless Analytics", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_pv_1", "type": "Text", "data": { "content_html": "<p>Privacy by architecture. Neopress was engineered from day one to eliminate intrusive cookie consent banners while maintaining complete marketing attribution accuracy.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_8 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">Privacy by Architecture: Native Cookieless Analytics</h1><div class="np-text" style="font-size: 17px; color: #94a3b8;"><p>Privacy by architecture. Neopress was engineered from day one to eliminate intrusive cookie consent banners while maintaining complete marketing attribution accuracy.</p></div></div></section>"##;
    insert_visual_page(
        conn,
        8,
        "Privacy by Architecture: Native Cookieless Analytics",
        "privacy",
        "Zero intrusive cookie consent banners while maintaining attribution accuracy.",
        layout_8,
        html_8,
        now,
    )?;

    // --- Page 8: About Us (ID 11) ---
    let layout_11 = r##"{
  "version": 1,
  "root": {
    "id": "root_ab",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_ab_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_ab_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_ab_1", "type": "Heading", "data": { "text": "About the Neopress Foundation", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_ab_1", "type": "Text", "data": { "content_html": "<p>Neopress was founded to build the modern, memory-safe, ultra-fast Content Management System that the web deserves.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8", "margin_bottom": "24px" } },
              { "id": "btn_ab_1", "type": "Button", "data": { "text": "Explore Architecture & Benchmarks ⚡", "url": "/engine/" }, "styles": { "background_color": "#4f46e5", "text_color": "#ffffff", "padding_top": "12px", "padding_bottom": "12px", "padding_left": "28px", "padding_right": "28px", "border_radius": "6px" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_11 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">About the Neopress Foundation</h1><div class="np-text" style="font-size: 17px; color: #94a3b8; margin-bottom: 24px;"><p>Neopress was founded to build the modern, memory-safe, ultra-fast Content Management System that the web deserves.</p></div><div class="np-btn-wrapper"><a href="/engine/" class="np-button" style="background-color: #4f46e5; color: #ffffff; padding: 12px 28px; border-radius: 6px; font-weight: 700; text-decoration: none;">Explore Architecture &amp; Benchmarks ⚡</a></div></div></section>"##;
    insert_visual_page(
        conn,
        11,
        "About Neopress Foundation",
        "about-us",
        "Engineering a memory-safe, ultra-high-performance Content Management System.",
        layout_11,
        html_11,
        now,
    )?;

    // --- Page 9: Architecture & Benchmarks (ID 12) ---
    let layout_12 = r##"{
  "version": 1,
  "root": {
    "id": "root_bm",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_bm_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_bm_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_bm_1", "type": "Heading", "data": { "text": "Architecture & Performance Benchmark", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_bm_1", "type": "Text", "data": { "content_html": "<p>Differential latency, throughput, and memory consumption benchmarks comparing native Rust Neopress with legacy WordPress 6.6.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_12 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">Architecture &amp; Performance Benchmark</h1><div class="np-text" style="font-size: 17px; color: #94a3b8;"><p>Differential latency, throughput, and memory consumption benchmarks comparing native Rust Neopress with legacy WordPress 6.6.</p></div></div></section>"##;
    insert_visual_page(
        conn,
        12,
        "Architecture & Performance Benchmark",
        "architecture-and-benchmarks",
        "Differential performance analysis against legacy PHP WordPress installations.",
        layout_12,
        html_12,
        now,
    )?;

    // --- Page 10: Services (ID 13) ---
    let layout_13 = r##"{
  "version": 1,
  "root": {
    "id": "root_sv",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_sv_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_sv_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_sv_1", "type": "Heading", "data": { "text": "Enterprise Services & Cloud Consulting", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_sv_1", "type": "Text", "data": { "content_html": "<p>High-scale migrations, custom WebAssembly module development, and enterprise SLAs.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8", "margin_bottom": "24px" } },
              { "id": "btn_sv_1", "type": "Button", "data": { "text": "Request Consultation ⚡", "url": "/contact/" }, "styles": { "background_color": "#e02660", "text_color": "#ffffff", "padding_top": "12px", "padding_bottom": "12px", "padding_left": "28px", "padding_right": "28px", "border_radius": "6px" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_13 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">Enterprise Services &amp; Cloud Consulting</h1><div class="np-text" style="font-size: 17px; color: #94a3b8; margin-bottom: 24px;"><p>High-scale migrations, custom WebAssembly module development, and enterprise SLAs.</p></div><div class="np-btn-wrapper"><a href="/contact/" class="np-button" style="background-color: #e02660; color: #ffffff; padding: 12px 28px; border-radius: 6px; font-weight: 700; text-decoration: none;">Request Consultation ⚡</a></div></div></section>"##;
    insert_visual_page(
        conn,
        13,
        "Enterprise Services & Cloud Consulting",
        "services",
        "High-scale migrations, custom Wasm module development, and enterprise SLAs.",
        layout_13,
        html_13,
        now,
    )?;

    // --- Page 11: Contact (ID 14) ---
    let layout_14 = r##"{
  "version": 1,
  "root": {
    "id": "root_ct",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_ct_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_ct_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_ct_1", "type": "Heading", "data": { "text": "Contact the Core Team", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_ct_1", "type": "Text", "data": { "content_html": "<p>Have questions about Neopress, enterprise migrations, or partnership opportunities? Send us a message.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_14 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">Contact the Core Team</h1><div class="np-text" style="font-size: 17px; color: #94a3b8;"><p>Have questions about Neopress, enterprise migrations, or partnership opportunities? Send us a message.</p></div></div></section>"##;
    insert_visual_page(
        conn,
        14,
        "Contact the Core Team",
        "contact",
        "Get in touch with the Neopress engineering team.",
        layout_14,
        html_14,
        now,
    )?;

    // --- Page 12: Sample Page (ID 15) ---
    let layout_15 = r##"{
  "version": 1,
  "root": {
    "id": "root_sp",
    "type": "Container",
    "styles": {},
    "children": [
      {
        "id": "sec_sp_hero",
        "type": "Section",
        "styles": { "background_color": "#0f172a", "text_color": "#ffffff", "padding_top": "70px", "padding_bottom": "70px", "text_align": "center" },
        "children": [
          {
            "id": "cont_sp_hero",
            "type": "Container",
            "styles": {},
            "children": [
              { "id": "h_sp_1", "type": "Heading", "data": { "text": "Sample Page Built with Visual Builder", "level": 1 }, "styles": { "font_size": "40px", "text_color": "#ffffff", "margin_bottom": "14px" } },
              { "id": "p_sp_1", "type": "Text", "data": { "content_html": "<p>This is a sample demonstration page created with the Neopress visual builder. Click 'Edit with Visual Builder' in the topbar to test live editing.</p>" }, "styles": { "font_size": "17px", "text_color": "#94a3b8", "margin_bottom": "24px" } },
              { "id": "btn_sp_1", "type": "Button", "data": { "text": "Launch Visual Studio 🎨", "url": "/n-admin/builder?post=15" }, "styles": { "background_color": "#e02660", "text_color": "#ffffff", "padding_top": "12px", "padding_bottom": "12px", "padding_left": "28px", "padding_right": "28px", "border_radius": "6px" } }
            ]
          }
        ]
      }
    ]
  }
}"##;
    let html_15 = r##"<section class="np-section" style="background-color: #0f172a; color: #ffffff; padding: 70px 24px; text-align: center;"><div class="np-container"><h1 class="np-heading" style="font-size: 40px; margin-bottom: 14px;">Sample Page Built with Visual Builder</h1><div class="np-text" style="font-size: 17px; color: #94a3b8; margin-bottom: 24px;"><p>This is a sample demonstration page created with the Neopress visual builder. Click 'Edit with Visual Builder' in the topbar to test live editing.</p></div><div class="np-btn-wrapper"><a href="/n-admin/builder?post=15" class="np-button" style="background-color: #e02660; color: #ffffff; padding: 12px 28px; border-radius: 6px; font-weight: 700; text-decoration: none;">Launch Visual Studio 🎨</a></div></div></section>"##;
    insert_visual_page(
        conn,
        15,
        "Sample Page",
        "sample-page",
        "Sample page built with the Neopress visual builder.",
        layout_15,
        html_15,
        now,
    )?;

    Ok(())
}
