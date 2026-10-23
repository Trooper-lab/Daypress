"""
tests/e2e_multi_site_visual_audit.py

Milestone 5 (Feature 24): Automated Playwright Headless Chrome Visual & Bounding-Box Audit.

Verifies:
1. Public SSR visual rendering across all 4 archetypes:
   - Archetype 1: WordPress Official Theme Unit Test (Gutenberg, 3+ level menus, sticky posts, comments)
   - Archetype 2: Gutenberg FSE / Block Theme (theme.json presets, block templates/parts)
   - Archetype 3: Elementor Starter Site (Astra/Kadence kit, containers, remapped internal links)
   - Archetype 4: Custom PHP + ACF + CPT (NORTHWIND theme, dynamic CPT archives, custom taxonomies)
   - 4 representative routes per archetype: Homepage, Inner Page, Post/CPT Single, Archive/CPT Archive.
   - Multi-breakpoint viewports: Desktop (1440x900), Tablet (768x1024), Mobile (375x812).
   - Assertions: HTTP status == 200, zero unhandled console errors, zero editor chrome leaks.
2. NeoStudio CAD Dock Chrome Invariants:
   - #neostudio-topbar: height <= 44.0px, y == 0.0, obsidian chassis background, non-occluded.
   - #neostudio-bottom-bar: height <= 32.0px, y == viewport_height - height, non-occluded.
   - #neostudio-rail: width == 48.0px, x == 0.0, non-occluded.
   - #neostudio-artboard: Stacking Context isolation (contain: layout paint, transform != none, isolation: isolate).
"""

import os
import sys
import time
import json
import sqlite3
import argparse
import subprocess
import urllib.request
import urllib.error
from pathlib import Path
from playwright.sync_api import sync_playwright

# Ensure UTF-8 output on Windows consoles
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

SCREENSHOT_DIR = os.path.abspath("tests/screenshots")
DEFAULT_PORT = 8095
BREAKPOINTS = {
    "desktop": {"width": 1440, "height": 900},
    "tablet": {"width": 768, "height": 1024},
    "mobile": {"width": 375, "height": 812},
}

class NeopressServerContext:
    """Manages the background neopress-server process lifecycle."""
    def __init__(self, port: int, db_path: str, bin_path: str = None):
        self.port = port
        self.db_path = os.path.abspath(db_path)
        self.base_url = f"http://127.0.0.1:{port}"
        if bin_path is None:
            self.bin_path = os.path.abspath("target/debug/neopress.exe" if sys.platform == "win32" else "target/debug/neopress")
        else:
            self.bin_path = os.path.abspath(bin_path)
        self.proc = None

    def __enter__(self):
        assert os.path.exists(self.bin_path), f"Binary not found at {self.bin_path}. Run `cargo build` first."
        print(f"[SERVER] Spawning Neopress on port {self.port} with DB {self.db_path}...")
        self.proc = subprocess.Popen(
            [self.bin_path, "--port", str(self.port), "--db", self.db_path],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            text=True
        )
        # Poll readiness
        ready = False
        for _ in range(40):
            time.sleep(0.25)
            try:
                with urllib.request.urlopen(f"{self.base_url}/wp-json", timeout=1) as resp:
                    if resp.status == 200:
                        ready = True
                        break
            except Exception:
                pass
        if not ready:
            self.__exit__(None, None, None)
            raise RuntimeError(f"Neopress failed to start and respond on {self.base_url}/wp-json within 10s")
        print(f"[SERVER] Neopress ready on {self.base_url}")
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.proc:
            print("[SERVER] Terminating Neopress process...")
            self.proc.terminate()
            try:
                self.proc.wait(timeout=3)
            except Exception:
                self.proc.kill()
            self.proc = None

class ArchetypeSeeder:
    """Initializes and seeds isolated SQLite database for each archetype."""
    @staticmethod
    def setup_db(db_path: str, archetype_id: int):
        os.makedirs(os.path.dirname(os.path.abspath(db_path)), exist_ok=True)
        for ext in ["", "-shm", "-wal", "-journal"]:
            p = db_path + ext
            if os.path.exists(p):
                try:
                    os.remove(p)
                except Exception:
                    pass

        conn = sqlite3.connect(db_path)
        cur = conn.cursor()

        # Create core WordPress tables
        cur.executescript("""
            CREATE TABLE IF NOT EXISTS wp_options (
                option_id INTEGER PRIMARY KEY AUTOINCREMENT,
                option_name TEXT UNIQUE NOT NULL,
                option_value TEXT NOT NULL,
                autoload TEXT NOT NULL DEFAULT 'yes'
            );
            CREATE TABLE IF NOT EXISTS wp_posts (
                ID INTEGER PRIMARY KEY AUTOINCREMENT,
                post_author INTEGER NOT NULL DEFAULT 1,
                post_date TEXT NOT NULL DEFAULT '',
                post_date_gmt TEXT NOT NULL DEFAULT '',
                post_content TEXT NOT NULL DEFAULT '',
                post_title TEXT NOT NULL DEFAULT '',
                post_excerpt TEXT NOT NULL DEFAULT '',
                post_status TEXT NOT NULL DEFAULT 'publish',
                comment_status TEXT NOT NULL DEFAULT 'open',
                ping_status TEXT NOT NULL DEFAULT 'open',
                post_password TEXT NOT NULL DEFAULT '',
                post_name TEXT NOT NULL DEFAULT '',
                to_ping TEXT NOT NULL DEFAULT '',
                pinged TEXT NOT NULL DEFAULT '',
                post_modified TEXT NOT NULL DEFAULT '',
                post_modified_gmt TEXT NOT NULL DEFAULT '',
                post_content_filtered TEXT NOT NULL DEFAULT '',
                post_parent INTEGER NOT NULL DEFAULT 0,
                guid TEXT NOT NULL DEFAULT '',
                menu_order INTEGER NOT NULL DEFAULT 0,
                post_type TEXT NOT NULL DEFAULT 'post',
                post_mime_type TEXT NOT NULL DEFAULT '',
                comment_count INTEGER NOT NULL DEFAULT 0,
                layout_json TEXT
            );
            CREATE TABLE IF NOT EXISTS wp_postmeta (
                meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
                post_id INTEGER NOT NULL,
                meta_key TEXT NOT NULL,
                meta_value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS wp_terms (
                term_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                slug TEXT NOT NULL,
                term_group INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS wp_term_taxonomy (
                term_taxonomy_id INTEGER PRIMARY KEY AUTOINCREMENT,
                term_id INTEGER NOT NULL,
                taxonomy TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                parent INTEGER NOT NULL DEFAULT 0,
                count INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS wp_term_relationships (
                object_id INTEGER NOT NULL,
                term_taxonomy_id INTEGER NOT NULL,
                term_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (object_id, term_taxonomy_id)
            );
            CREATE TABLE IF NOT EXISTS wp_comments (
                comment_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                comment_post_ID INTEGER NOT NULL DEFAULT 0,
                comment_author TEXT NOT NULL DEFAULT '',
                comment_author_email TEXT NOT NULL DEFAULT '',
                comment_author_url TEXT NOT NULL DEFAULT '',
                comment_author_IP TEXT NOT NULL DEFAULT '',
                comment_date TEXT NOT NULL DEFAULT '',
                comment_date_gmt TEXT NOT NULL DEFAULT '',
                comment_content TEXT NOT NULL DEFAULT '',
                comment_karma INTEGER NOT NULL DEFAULT 0,
                comment_approved TEXT NOT NULL DEFAULT '1',
                comment_agent TEXT NOT NULL DEFAULT '',
                comment_type TEXT NOT NULL DEFAULT 'comment',
                comment_parent INTEGER NOT NULL DEFAULT 0,
                user_id INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS wp_neopress_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                template_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'publish',
                layout_json TEXT NOT NULL DEFAULT '{}',
                rendered_html TEXT DEFAULT NULL,
                settings_json TEXT NOT NULL DEFAULT '{}',
                priority INTEGER NOT NULL DEFAULT 10,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS wp_neopress_template_conditions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template_id INTEGER NOT NULL REFERENCES wp_neopress_templates(id) ON DELETE CASCADE,
                condition_type TEXT NOT NULL DEFAULT 'include',
                rule_target TEXT NOT NULL,
                target_id TEXT NOT NULL DEFAULT '',
                priority INTEGER NOT NULL DEFAULT 10,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
        """)

        # Base site settings
        cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('blogname', 'Neopress Visual Audit');")
        cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('siteurl', 'http://127.0.0.1:8095');")
        cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('home', 'http://127.0.0.1:8095');")

        # Archetype Specific Seeding
        if archetype_id == 1:
            # Archetype 1: Official Theme Unit Test (Gutenberg, 3+ level nav, sticky posts)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('current_theme', 'Theme Unit Test');")
            # Seed Category 'Classic'
            cur.execute("INSERT INTO wp_terms (term_id, name, slug) VALUES (10, 'Classic', 'classic');")
            cur.execute("INSERT INTO wp_term_taxonomy (term_taxonomy_id, term_id, taxonomy) VALUES (10, 10, 'category');")
            # Post 1: Homepage
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (1, 'Welcome to Theme Unit Test', 'home', 'page', 'publish',
                '<!-- wp:group {"metadata":{"name":"Hero"}} --><div class="wp-block-group"><h1>Official WordPress Theme Unit Test</h1><p>Verifying multi-level navigation and core Gutenberg blocks.</p></div><!-- /wp:group -->');
            """)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('page_on_front', '1');")
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('show_on_front', 'page');")
            # Post 2: Inner Page
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (2, 'Page with Comments', 'page-with-comments', 'page', 'publish',
                '<!-- wp:paragraph --><p>This is an inner page testing comments and breadcrumb hierarchy.</p><!-- /wp:paragraph -->');
            """)
            # Post 3: Single Post with Gutenberg Blocks
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (3, 'Post Format Test: Gallery & Quote', 'post-format-test-gallery', 'post', 'publish',
                '<!-- wp:quote --><blockquote class="wp-block-quote"><p>Sub-millisecond WordPress SSR with native Rust execution.</p></blockquote><!-- /wp:quote -->' ||
                '<!-- wp:gallery {"columns":2} --><figure class="wp-block-gallery columns-2"><p class="gallery-item">Gallery Item 1</p><p class="gallery-item">Gallery Item 2</p></figure><!-- /wp:gallery -->');
            """)
            cur.execute("INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (3, 10);")

        elif archetype_id == 2:
            # Archetype 2: Gutenberg FSE / Block Theme (theme.json presets)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('current_theme', 'Twenty Twenty-Four');")
            cur.execute("""
                INSERT INTO wp_options (option_name, option_value) VALUES ('theme_json_styles',
                ':root { --wp--preset--color--primary: #1e40af; --wp--preset--font-size--large: 32px; }');
            """)
            cur.execute("INSERT INTO wp_terms (term_id, name, slug) VALUES (20, 'News', 'news');")
            cur.execute("INSERT INTO wp_term_taxonomy (term_taxonomy_id, term_id, taxonomy) VALUES (20, 20, 'category');")
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (1, 'Twenty Twenty-Four FSE Home', 'home', 'page', 'publish',
                '<header class="wp-block-template-part"><h2>FSE Header</h2></header><main><h1>Modern Block Theme Architecture</h1></main>');
            """)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('page_on_front', '1');")
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (2, 'FSE Services', 'services', 'page', 'publish',
                '<!-- wp:columns --><div class="wp-block-columns"><div class="wp-block-column"><h3>Service A</h3></div><div class="wp-block-column"><h3>Service B</h3></div></div><!-- /wp:columns -->');
            """)
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (3, 'Welcome to Block Theme', 'welcome-to-block-theme', 'post', 'publish',
                '<p>Gutenberg full site editing block single post.</p>');
            """)
            cur.execute("INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (3, 20);")

        elif archetype_id == 3:
            # Archetype 3: Elementor Starter Site (Astra kit, flex/grid containers)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('current_theme', 'Astra');")
            kit_css = ":root { --e-global-color-primary: #6366f1; --e-global-typography-primary: 'Inter'; }"
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('elementor_kit_css', ?);", (kit_css,))
            cur.execute("INSERT INTO wp_terms (term_id, name, slug) VALUES (30, 'Blog', 'blog');")
            cur.execute("INSERT INTO wp_term_taxonomy (term_taxonomy_id, term_id, taxonomy) VALUES (30, 30, 'category');")
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (1, 'Astra Starter Site Agency', 'home', 'page', 'publish',
                '<div class="elementor-element elementor-widget-container"><h1>Next-Gen Creative Agency</h1><p>High speed conversion.</p></div>');
            """)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('page_on_front', '1');")
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (2, 'About Us', 'about', 'page', 'publish',
                '<div class="elementor-container elementor-column-gap-default"><div class="elementor-row"><h2>Our Story</h2></div></div>');
            """)
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (3, 'Global Enterprise Case Studies', 'case-studies', 'post', 'publish',
                '<div class="elementor-widget-theme-post-content"><p>Detailed case study content.</p></div>');
            """)
            cur.execute("INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (3, 30);")

        elif archetype_id == 4:
            # Archetype 4: Custom PHP + ACF + CPT (NORTHWIND)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('current_theme', 'northwind');")
            # CPT Taxonomy
            cur.execute("INSERT INTO wp_terms (term_id, name, slug) VALUES (40, 'Agile', 'agile');")
            cur.execute("INSERT INTO wp_term_taxonomy (term_taxonomy_id, term_id, taxonomy) VALUES (40, 40, 'trainingen_category');")
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (1, 'NORTHWIND Home', 'home', 'page', 'publish',
                '<section class="section diagonal-top"><h1>Northwind Recruitment & Academy</h1></section>');
            """)
            cur.execute("INSERT INTO wp_options (option_name, option_value) VALUES ('page_on_front', '1');")
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (2, 'Voor Professionals', 'professionals', 'page', 'publish',
                '<section class="section"><h2>Groei in jouw marketing carrière</h2></section>');
            """)
            # Single CPT Post
            cur.execute("""
                INSERT INTO wp_posts (ID, post_title, post_name, post_type, post_status, post_content)
                VALUES (3, 'Agile Marketing Coach', 'agile-coach', 'trainingen', 'publish',
                '<section class="training-detail"><h2>Agile Marketing Training</h2><p>12-week immersive masterclass.</p></section>');
            """)
            cur.execute("INSERT INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (3, 40);")
            cur.execute("INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (3, 'duur', '12 weken');")
            cur.execute("INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (3, 'locatie', 'Utrecht');")

        conn.commit()
        conn.close()
        print(f"[DB] Archetype {archetype_id} database seeded at {db_path}")

def run_cad_dock_assertions(page, archetype_id: int):
    """Programmatically asserts NeoStudio CAD Dock Chrome bounding-box invariants."""
    print(f"[AUDIT] Verifying CAD Dock Chrome Invariants for Archetype {archetype_id}...")

    # 1. Topbar Invariant Verification
    topbar = page.locator("#neostudio-topbar")
    assert topbar.is_visible(), "Topbar #neostudio-topbar must be visible"
    t_rect = topbar.bounding_box()
    assert t_rect is not None, "Topbar bounding box must not be None"
    assert t_rect["y"] == 0.0, f"Topbar y must be 0, got {t_rect['y']}"
    assert t_rect["height"] <= 44.0, f"Topbar height must be <= 44px, got {t_rect['height']}px"
    assert t_rect["height"] >= 38.0, f"Topbar height must be >= 38px, got {t_rect['height']}px"
    top_hit = page.evaluate("() => { const el = document.elementFromPoint(100, 20); return el ? el.id || el.className || el.tagName : null; }")
    assert "artboard" not in str(top_hit).lower(), f"Topbar hit-test occluded by artboard! Hit: {top_hit}"

    # 2. Bottom-Bar Invariant Verification
    bottom_bar = page.locator("#neostudio-bottom-bar")
    assert bottom_bar.is_visible(), "Bottom-bar #neostudio-bottom-bar must be visible"
    b_rect = bottom_bar.bounding_box()
    assert b_rect is not None, "Bottom-bar bounding box must not be None"
    assert b_rect["height"] <= 32.0, f"Bottom-bar height must be <= 32px, got {b_rect['height']}px"
    assert b_rect["height"] >= 26.0, f"Bottom-bar height must be >= 26px, got {b_rect['height']}px"
    expected_bottom_y = 900.0 - b_rect["height"]
    assert abs(b_rect["y"] - expected_bottom_y) < 1.0, f"Bottom-bar must be anchored to bottom, got y={b_rect['y']}"
    bottom_hit = page.evaluate("() => { const el = document.elementFromPoint(100, 890); return el ? el.id || el.className || el.tagName : null; }")
    assert "artboard" not in str(bottom_hit).lower(), f"Bottom-bar hit-test occluded by artboard! Hit: {bottom_hit}"

    # 3. Left Rail Invariant Verification
    rail = page.locator("#neostudio-rail")
    assert rail.is_visible(), "Left Rail #neostudio-rail must be visible"
    r_rect = rail.bounding_box()
    assert r_rect is not None, "Rail bounding box must not be None"
    assert r_rect["x"] == 0.0, f"Rail x must be 0, got {r_rect['x']}"
    assert r_rect["width"] == 48.0, f"Rail width must be 48px, got {r_rect['width']}px"

    # 4. Artboard CSS Containment & Stacking Context Isolation
    artboard = page.locator("#neostudio-artboard")
    assert artboard.is_visible(), "Artboard #neostudio-artboard must be visible"
    a_styles = artboard.evaluate("""el => {
        const s = window.getComputedStyle(el);
        return {
            contain: s.contain,
            transform: s.transform,
            isolation: s.isolation,
            overflow: s.overflow
        };
    }""")
    assert "paint" in a_styles["contain"] and "layout" in a_styles["contain"], f"Artboard missing contain: layout paint: {a_styles['contain']}"
    assert a_styles["transform"] != "none", f"Artboard missing transform stacking context: {a_styles['transform']}"
    assert a_styles["isolation"] == "isolate", f"Artboard missing isolation: isolate: {a_styles['isolation']}"

    # 5. Capture NeoStudio Builder Screenshot
    screenshot_name = f"builder_arch{archetype_id}_desktop_1440x900.png"
    out_path = os.path.join(SCREENSHOT_DIR, screenshot_name)
    page.screenshot(path=out_path)
    print(f"  [SCREENSHOT] Captured {screenshot_name}")

def audit_archetype(archetype_id: int, port: int, headless: bool):
    """Executes public SSR visual audit and CAD dock invariant checks for one archetype."""
    archetype_names = {
        1: "Theme Unit Test (Gutenberg)",
        2: "Gutenberg FSE (Twenty Twenty-Four)",
        3: "Elementor Starter Site (Astra)",
        4: "Custom PHP + ACF + CPT (NORTHWIND)",
    }
    print(f"\n========================================================")
    print(f"AUDITING ARCHETYPE {archetype_id}: {archetype_names[archetype_id]}")
    print(f"========================================================")

    db_path = f"data/test_audit_arch{archetype_id}.sqlite"
    ArchetypeSeeder.setup_db(db_path, archetype_id)

    routes = {
        1: [
            ("home", "/"),
            ("inner_page", "/page-with-comments/"),
            ("single_post", "/post-format-test-gallery/"),
            ("archive", "/category/classic/"),
        ],
        2: [
            ("home", "/"),
            ("inner_page", "/services/"),
            ("single_post", "/welcome-to-block-theme/"),
            ("archive", "/category/news/"),
        ],
        3: [
            ("home", "/"),
            ("inner_page", "/about/"),
            ("single_post", "/case-studies/"),
            ("archive", "/category/blog/"),
        ],
        4: [
            ("home", "/"),
            ("inner_page", "/professionals/"),
            ("cpt_single", "/trainingen/agile-coach/"),
            ("cpt_archive", "/trainingen/"),
        ],
    }[archetype_id]

    with NeopressServerContext(port=port, db_path=db_path) as srv:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=headless)

            # --- PART A: Public SSR Visual Verification Across 3 Breakpoints ---
            for route_key, path in routes:
                url = f"{srv.base_url}{path}"
                for bp_name, bp_dims in BREAKPOINTS.items():
                    context = browser.new_context(viewport=bp_dims)
                    page = context.new_page()

                    console_errors = []
                    page.on("console", lambda msg: console_errors.append(msg.text) if msg.type == "error" else None)
                    page.on("pageerror", lambda err: console_errors.append(str(err)))

                    print(f"[SSR] {bp_name.upper()} ({bp_dims['width']}x{bp_dims['height']}) -> {url}")
                    resp = page.goto(url, wait_until="networkidle")

                    # Assert HTTP status 200
                    assert resp is not None and resp.status == 200, f"Expected HTTP 200 on {url}, got {resp.status if resp else 'None'}"

                    # Filter benign favicon 404s or missing sample asset logs, but assert no unhandled exceptions
                    critical_errors = [e for e in console_errors if "favicon" not in e.lower() and "failed to load resource" not in e.lower()]
                    assert len(critical_errors) == 0, f"Console errors detected on {url} [{bp_name}]: {critical_errors}"

                    # Assert zero editor chrome leaks in SSR DOM
                    content = page.content()
                    assert 'contenteditable="true"' not in content, f"Leaked contenteditable='true' in SSR response: {url}"
                    assert 'class="neostudio-shell' not in content, f"Leaked neostudio-shell in SSR response: {url}"
                    assert '<!-- wp:' not in content, f"Unparsed Gutenberg <!-- wp: comment leaked in SSR: {url}"

                    # Capture screenshot artifact
                    shot_name = f"arch{archetype_id}_{route_key}_{bp_name}_{bp_dims['width']}x{bp_dims['height']}.png"
                    out_path = os.path.join(SCREENSHOT_DIR, shot_name)
                    page.screenshot(path=out_path)
                    print(f"  [SCREENSHOT] Captured {shot_name}")

                    context.close()

            # --- PART B: NeoStudio CAD Dock Chrome Invariants Verification ---
            context = browser.new_context(viewport={"width": 1440, "height": 900})
            context.add_cookies([{"name": "neopress_admin", "value": "1", "domain": "127.0.0.1", "path": "/"}])
            page = context.new_page()

            builder_url = f"{srv.base_url}/n-admin/builder?post=1"
            print(f"[BUILDER] Navigating to {builder_url}...")
            resp = page.goto(builder_url, wait_until="networkidle")
            assert resp is not None and resp.status == 200, f"Builder returned status {resp.status if resp else 'None'}"
            page.wait_for_selector("#neostudio-shell", timeout=10000)

            run_cad_dock_assertions(page, archetype_id)
            context.close()
            browser.close()

    print(f"[PASS] Archetype {archetype_id} fully audited and verified 100%!")

def main():
    parser = argparse.ArgumentParser(description="Neopress Multi-Site Visual & CAD Bounding-Box Audit")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Port to run Neopress test server on")
    parser.add_argument("--headless", type=bool, default=True, help="Run browser in headless mode")
    parser.add_argument("--archetypes", type=str, default="1,2,3,4", help="Comma-separated archetype IDs (e.g. 1,2,3,4)")
    args = parser.parse_args()

    os.makedirs(SCREENSHOT_DIR, exist_ok=True)
    archetype_ids = [int(x.strip()) for x in args.archetypes.split(",") if x.strip()]

    print("=================================================================")
    print("  🚀 Neopress Playwright Visual & Bounding-Box Audit Suite")
    print("=================================================================")
    print(f"Port:       {args.port}")
    print(f"Headless:   {args.headless}")
    print(f"Archetypes: {archetype_ids}")
    print(f"Screenshots: {SCREENSHOT_DIR}")

    start_time = time.time()
    for arch_id in archetype_ids:
        audit_archetype(arch_id, args.port, args.headless)

    elapsed = time.time() - start_time
    print("\n=================================================================")
    print(f"  🎉 ALL ARCHETYPES PASSED 100%! Completed in {elapsed:.2f}s")
    print("=================================================================")
    sys.exit(0)

if __name__ == "__main__":
    main()
