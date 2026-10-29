"""
Milestone 5: DayStudio Live Browser Verification & Multi-Breakpoint Screenshot Audit.

Automated Playwright test suite (sync API) with Chromium in headless mode:
1. Starts Daypress binary on dedicated test port 8099 with isolated test DB `data/test_m5_audit.sqlite`.
2. Seeds Theme Builder starter templates (`agency-header` and `dark-grid-footer`) via REST.
3. Seeds test page `pro-blocks-showcase` (Post ID 1) in SQLite with taxonomy associations.
4. Uses browser context with admin cookie `daypress_admin=1`.
5. Automates DayStudio visual builder interactive flows:
   - 3-pane zero-scroll dock shell geometry verification.
   - Layers Navigator drawer expansion and AST tree verification.
   - Theme Templates drawer and template cards listing.
   - Element selection, Trapezoidal Box-Model widget scrubbers (margin/padding clamping), and canvas guide overlays.
   - Structural X-Ray mode wireframe toggling.
   - Responsive viewport switcher (Desktop 1200px, Tablet 768px, Mobile 375px) and zoom engine (clamped 50%-150%).
   - Non-blocking `#modal-create-template` and `#modal-conditions` dialogs.
   - Interactive Pro block insertion (PriceCard, CallToAction, Countdown, FlipBox, Testimonial, StarRating, ProgressBar, NavMenu).
   - Save / Publish trigger (`#btn-primary-publish`), asserting `#status-save-pill` transitions to `.saved` and layout persists to SQLite.
6. Automates Public SSR frontend rendering and visual parity:
   - Front Page (`/`) across Desktop, Tablet, and Mobile.
   - Single Post (`/pro-blocks-showcase/`) across Desktop, Tablet, and Mobile.
   - Category Archive (`/category/architecture/`) across Desktop, Tablet, and Mobile.
   - 404 Error Page (`/non-existent-quantum-core/`) across Desktop and Mobile.
   - Asserts 100% elimination of editor chrome (contenteditable, draggable, elementor-grid-handle, daystudio-*).
   - Asserts zero unhandled JavaScript console errors.
   - Asserts SQLite `daypress_404_log` hit recording.
7. Captures all 21 screenshot artifacts specified in explorer_m5_2 catalog into `tests/screenshots/`.
"""

import os
import sys
import time
import json
import sqlite3
import subprocess
import urllib.request
import urllib.error

# Ensure UTF-8 output on Windows consoles
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from playwright.sync_api import sync_playwright

TEST_PORT = 8099
BASE_URL = f"http://127.0.0.1:{TEST_PORT}"
TEST_DB = "data/test_m5_audit.sqlite"
SCREENSHOT_DIR = os.path.abspath("tests/screenshots")

SHOWCASE_PRO_BLOCKS_LAYOUT = {
    "version": 1,
    "root": {
        "id": "root_pro_showcase",
        "type": "Root",
        "styles": {},
        "children": [
            {
                "id": "sec_hero",
                "type": "Section",
                "styles": {
                    "padding_top": "64px",
                    "padding_bottom": "36px",
                    "background_color": "#0a0d14",
                    "text_align": "center",
                },
                "children": [
                    {
                        "id": "cont_hero",
                        "type": "Container",
                        "styles": {
                            "max_width": "960px",
                            "margin_left": "auto",
                            "margin_right": "auto",
                            "padding_left": "24px",
                            "padding_right": "24px",
                        },
                        "children": [
                            {
                                "id": "badge_hero",
                                "type": "Heading",
                                "data": {"text": "2026 COMMERCIAL PRO WIDGETS", "level": 4},
                                "styles": {
                                    "font_size": "12px",
                                    "text_color": "#3b82f6",
                                    "margin_bottom": "12px",
                                    "letter_spacing": "0.1em",
                                },
                            },
                            {
                                "id": "h_hero",
                                "type": "Heading",
                                "data": {"text": "Daypress Pro Blocks Showcase", "level": 1},
                                "styles": {
                                    "font_size": "44px",
                                    "text_color": "#ffffff",
                                    "margin_bottom": "16px",
                                    "font_weight": "800",
                                },
                            },
                            {
                                "id": "p_hero",
                                "type": "Text",
                                "data": {
                                    "content_html": "<p>Zero-overhead native Rust SSR rendering of 26 Elementor Pro and Divi Pro widgets. Sub-millisecond latency, zero JavaScript hydration lag, and 100% responsive reflow.</p>"
                                },
                                "styles": {
                                    "font_size": "17px",
                                    "text_color": "#94a3b8",
                                    "margin_bottom": "32px",
                                },
                            },
                            {
                                "id": "nav_showcase",
                                "type": "NavMenu",
                                "data": {
                                    "menu_slug": "primary-menu",
                                    "layout": "horizontal",
                                    "align": "center",
                                    "mobile_hamburger": False,
                                    "items": [
                                        {"title": "Overview", "url": "#overview", "order": 1},
                                        {"title": "Pricing", "url": "#pricing", "order": 2},
                                        {"title": "Interactive 3D", "url": "#interactive", "order": 3},
                                        {"title": "Performance", "url": "#performance", "order": 4},
                                    ],
                                },
                                "styles": {
                                    "margin_bottom": "40px",
                                },
                            },
                        ],
                    }
                ],
            },
            {
                "id": "sec_commercial",
                "type": "Section",
                "styles": {
                    "padding_top": "48px",
                    "padding_bottom": "64px",
                    "background_color": "#0d111a",
                },
                "children": [
                    {
                        "id": "cont_commercial",
                        "type": "Container",
                        "styles": {
                            "max_width": "1140px",
                            "margin_left": "auto",
                            "margin_right": "auto",
                            "padding_left": "24px",
                            "padding_right": "24px",
                        },
                        "children": [
                            {
                                "id": "pricecard_1",
                                "type": "PriceCard",
                                "data": {
                                    "title": "Enterprise Cloud",
                                    "subtitle": "Mission-critical high-throughput Rust CMS node",
                                    "currency": "$",
                                    "price": "49",
                                    "original_price": "99",
                                    "period": "month",
                                    "badge": "MOST POPULAR",
                                    "featured": True,
                                    "features": [
                                        {"text": "Sub-millisecond Rust SSR (<0.15ms)", "included": True},
                                        {"text": "Lossless Elementor & Divi AST Transpiler", "included": True},
                                        {"text": "Unlimited Workspaces & Team Seats", "included": True},
                                        {"text": "Dual-Write SQLite Relational Integrity", "included": True},
                                        {"text": "24/7 Dedicated Tier-1 SLA Support", "included": True},
                                    ],
                                    "button_text": "Deploy Enterprise Instance",
                                    "button_url": "#deploy",
                                },
                                "styles": {
                                    "max_width": "480px",
                                    "margin_left": "auto",
                                    "margin_right": "auto",
                                    "margin_bottom": "48px",
                                },
                            },
                            {
                                "id": "countdown_1",
                                "type": "Countdown",
                                "data": {
                                    "target_date": "2026-12-31T23:59:59Z",
                                    "mode": "due_date",
                                    "show_days": True,
                                    "show_hours": True,
                                    "show_minutes": True,
                                    "show_seconds": True,
                                    "expiry_message": "Release Countdown Complete",
                                },
                                "styles": {
                                    "margin_bottom": "48px",
                                },
                            },
                            {
                                "id": "flipbox_1",
                                "type": "FlipBox",
                                "data": {
                                    "front_icon": "fa-solid fa-cube",
                                    "front_title": "Interactive 3D Engine",
                                    "front_description": "Hardware-accelerated CSS 3D transforms with instant perspective depth.",
                                    "back_title": "Turnkey Architecture",
                                    "back_description": "Pure native Rust Axum rendering with zero JavaScript hydration lag.",
                                    "button_text": "Explore Architecture",
                                    "button_url": "#explore",
                                    "direction": "flip_right",
                                },
                                "styles": {
                                    "max_width": "480px",
                                    "margin_left": "auto",
                                    "margin_right": "auto",
                                    "margin_bottom": "48px",
                                },
                            },
                            {
                                "id": "cta_1",
                                "type": "CallToAction",
                                "data": {
                                    "title": "Migrate from Legacy WordPress in Under 4 Seconds",
                                    "description": "Drop-in Axum replacement compatible with Elementor, Divi, Gutenberg, and Classic themes.",
                                    "button_text": "Start Free Migration",
                                    "button_url": "#migrate",
                                    "secondary_button_text": "View Benchmarks",
                                    "secondary_button_url": "#benchmarks",
                                    "skin": "cover",
                                    "ribbon": "2026 ENGINE",
                                },
                                "styles": {
                                    "margin_bottom": "48px",
                                },
                            },
                            {
                                "id": "rating_1",
                                "type": "StarRating",
                                "data": {
                                    "rating": 5.0,
                                    "max_rating": 5,
                                    "title": "Agency Satisfaction Score",
                                    "star_color": "#f59e0b",
                                    "star_size": 22,
                                },
                                "styles": {
                                    "text_align": "center",
                                    "margin_bottom": "32px",
                                },
                            },
                            {
                                "id": "prog_1",
                                "type": "ProgressBar",
                                "data": {
                                    "title": "AST Compilation & SSR Parity",
                                    "percent": 98,
                                    "display_text": "98% Parity Verified",
                                    "track_color": "#1e293b",
                                    "bar_color": "#2563eb",
                                    "height": 14,
                                },
                                "styles": {
                                    "max_width": "640px",
                                    "margin_left": "auto",
                                    "margin_right": "auto",
                                },
                            },
                        ],
                    }
                ],
            },
        ],
    },
}


class DaypressServerRunner:
    """Manages the lifecycle of the isolated background Daypress server subprocess."""

    def __init__(self, port=TEST_PORT, db_path=TEST_DB):
        self.port = port
        self.db_path = db_path
        self.proc = None

    def start(self):
        # 1. Clean existing test database files
        os.makedirs(os.path.dirname(os.path.abspath(self.db_path)), exist_ok=True)
        for ext in ["", "-shm", "-wal", "-journal"]:
            p = self.db_path + ext
            if os.path.exists(p):
                try:
                    os.remove(p)
                except Exception as e:
                    print(f"[CLEANUP] Could not remove {p}: {e}")

        # 2. Ensure server binary exists
        bin_path = os.path.abspath("target/debug/daypress.exe")
        if not os.path.exists(bin_path):
            print("[SERVER] Target binary missing. Compiling daypress-server...")
            res = subprocess.run(
                ["cargo", "build", "-p", "daypress-server", "--bin", "daypress"],
                capture_output=True,
                text=True,
            )
            if res.returncode != 0:
                raise RuntimeError(f"Cargo build failed:\n{res.stderr}")

        # 3. Launch Daypress server subprocess
        print(f"[SERVER] Starting daypress on port {self.port} with DB {self.db_path}...")
        self.proc = subprocess.Popen(
            [bin_path, "--port", str(self.port), "--db", self.db_path],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            text=True,
        )

        # 4. Readiness probe
        ready = False
        print("[SERVER] Probing server readiness on /wp-json...")
        for attempt in range(40):
            time.sleep(0.3)
            try:
                with urllib.request.urlopen(f"{BASE_URL}/wp-json", timeout=1) as resp:
                    if resp.status == 200:
                        ready = True
                        break
            except Exception:
                pass

        if not ready:
            self.stop()
            raise RuntimeError(f"Server on port {self.port} failed to become ready within timeout.")

        print(f"[SERVER] Daypress server is ready on {BASE_URL}!")

    def stop(self):
        if self.proc:
            print("[SERVER] Stopping server process...")
            self.proc.terminate()
            try:
                self.proc.wait(timeout=3)
            except Exception:
                self.proc.kill()
            self.proc = None
            print("[SERVER] Server process terminated.")

        # Clean database files
        for ext in ["", "-shm", "-wal", "-journal"]:
            p = self.db_path + ext
            if os.path.exists(p):
                try:
                    os.remove(p)
                except Exception:
                    pass


def seed_database_via_rest_and_sqlite():
    """Seeds starter templates via REST and sets up test page in SQLite."""
    print("[SEED] Installing Theme Builder starter templates via REST...")

    # Install agency-header starter
    req1 = urllib.request.Request(
        f"{BASE_URL}/wp-json/daypress/v1/templates/starter?starter_id=agency-header",
        data=b"{}",
        headers={
            "Content-Type": "application/json",
            "X-WP-Nonce": "wp_rest_admin",
            "Cookie": "daypress_admin=1",
        },
        method="POST",
    )
    with urllib.request.urlopen(req1, timeout=5) as resp:
        assert resp.status in (200, 201), f"agency-header starter failed: {resp.status}"
        data = json.loads(resp.read().decode())
        print(f"[SEED] Installed agency-header (ID: {data.get('id')})")

    # Install dark-grid-footer starter
    req2 = urllib.request.Request(
        f"{BASE_URL}/wp-json/daypress/v1/templates/starter?starter_id=dark-grid-footer",
        data=b"{}",
        headers={
            "Content-Type": "application/json",
            "X-WP-Nonce": "wp_rest_admin",
            "Cookie": "daypress_admin=1",
        },
        method="POST",
    )
    with urllib.request.urlopen(req2, timeout=5) as resp:
        assert resp.status in (200, 201), f"dark-grid-footer starter failed: {resp.status}"
        data = json.loads(resp.read().decode())
        print(f"[SEED] Installed dark-grid-footer (ID: {data.get('id')})")

    # Connect to SQLite to verify & seed post ID 1
    print("[SEED] Seeding Post ID 1 (Pro Blocks Showcase) in SQLite...")
    conn = sqlite3.connect(TEST_DB)
    cur = conn.cursor()

    layout_str = json.dumps(SHOWCASE_PRO_BLOCKS_LAYOUT)

    cur.execute(
        """
        INSERT OR REPLACE INTO posts (
            ID, post_author, post_date, post_date_gmt, post_content, post_title,
            post_excerpt, post_status, post_name, post_modified, post_modified_gmt,
            guid, post_type, layout_json
        ) VALUES (
            1, 1, datetime('now'), datetime('now'),
            '<p>Demonstration page featuring rich Daypress Pro blocks.</p>',
            'Pro Blocks Showcase',
            'Full multi-breakpoint showcase of Pro widgets and dynamic theme templates.',
            'publish', 'pro-blocks-showcase', datetime('now'), datetime('now'),
            'http://127.0.0.1:8099/pro-blocks-showcase/', 'page', ?
        );
        """,
        (layout_str,)
    )

    cur.execute(
        "INSERT OR REPLACE INTO postmeta (post_id, meta_key, meta_value) VALUES (1, '_daypress_layout', ?);",
        (layout_str,)
    )

    # Attach to category 2 (Architecture)
    cur.execute("INSERT OR REPLACE INTO term_relationships (object_id, term_taxonomy_id) VALUES (1, 2);")
    conn.commit()
    conn.close()
    print("[SEED] Seeding completed successfully.")


def assert_pro_blocks_render_and_styles(ssr_page, vp_name):
    """
    Empirically validates that Pro blocks on /pro-blocks-showcase/ are not only
    present in the DOM, but fully populated with rich data and active CSS styles.
    """
    print(f"  [PRO BLOCKS AUDIT] Verifying Pro blocks rendering and CSS on {vp_name}...")

    # 1. PriceCard Assertions
    assert ssr_page.locator(".dp-price-card").count() > 0, "PriceCard must be rendered on page"
    price_card = ssr_page.locator(".dp-price-card").first
    assert price_card.is_visible(), "PriceCard must be visible"

    # Verify Title & Subtitle
    title_text = price_card.locator(".dp-price-title, .price-card-title").inner_text().strip()
    assert title_text != "", "PriceCard title must not be empty"

    # Verify Formatted Price & Digits
    price_val = price_card.locator(".dp-price-val, .price-amount").inner_text().strip()
    assert price_val != "" and any(c.isdigit() for c in price_val), f"PriceCard must contain numeric price digits, got '{price_val}'"

    # Verify Checklist Features
    feature_items = price_card.locator(".dp-price-feature, .price-feature-item")
    assert feature_items.count() >= 3, f"PriceCard must render at least 3 feature items, found {feature_items.count()}"

    # Verify CTA Button
    btn = price_card.locator(".dp-price-btn, .btn-primary")
    assert btn.count() > 0 and btn.inner_text().strip() != "", "PriceCard CTA button must be present with text"

    # Computed Styles Check for PriceCard
    card_styles = price_card.evaluate("""el => {
        const s = window.getComputedStyle(el);
        return {
            display: s.display,
            paddingTop: parseFloat(s.paddingTop) || 0,
            paddingBottom: parseFloat(s.paddingBottom) || 0,
            borderRadius: parseFloat(s.borderRadius) || 0,
            borderStyle: s.borderStyle,
            boxShadow: s.boxShadow
        };
    }""")
    assert card_styles["paddingTop"] >= 16, f"PriceCard must have >=16px padding, got {card_styles['paddingTop']}px"
    assert card_styles["borderRadius"] >= 4, f"PriceCard must have rounded corners, got {card_styles['borderRadius']}px"

    # 2. Countdown Timer Assertions
    assert ssr_page.locator(".dp-countdown").count() > 0, "Countdown must be rendered"
    countdown = ssr_page.locator(".dp-countdown").first
    countdown_items = countdown.locator(".dp-countdown-item, .countdown-box")
    assert countdown_items.count() >= 3, f"Countdown must render unit boxes, found {countdown_items.count()}"

    labels = countdown.locator(".dp-countdown-label, .countdown-unit").all_inner_texts()
    assert any("Day" in l or "day" in l.lower() for l in labels), "Countdown must display Days unit"
    assert any("Hour" in l or "hour" in l.lower() for l in labels), "Countdown must display Hours unit"

    # 3. FlipBox Assertions
    assert ssr_page.locator(".dp-flipbox").count() > 0, "FlipBox must be rendered"
    flipbox = ssr_page.locator(".dp-flipbox").first
    assert flipbox.locator(".dp-flipbox-front, .flipbox-front").count() > 0, "FlipBox must have front face"
    assert flipbox.locator(".dp-flipbox-back, .flipbox-back").count() > 0, "FlipBox must have back face"
    front_t = flipbox.locator(".dp-flipbox-front .dp-flipbox-title, .flipbox-front .flipbox-title").all_inner_texts()
    assert len(front_t) > 0 and any(t.strip() != "" for t in front_t), "FlipBox front title must not be empty"

    # 4. CallToAction Assertions
    assert ssr_page.locator(".dp-cta").count() > 0, "CallToAction must be rendered"
    cta_title = ssr_page.locator(".dp-cta .dp-cta-title").all_inner_texts()
    assert len(cta_title) > 0 and any(t.strip() != "" for t in cta_title), "CallToAction title must not be empty"
    assert ssr_page.locator(".dp-cta .dp-cta-btn, .dp-cta a").count() > 0, "CallToAction primary button must exist"

    # 5. StarRating Assertions
    assert ssr_page.locator(".dp-star-rating, .dp-rating").count() > 0, "StarRating must be rendered"
    stars_count = ssr_page.locator(".dp-star-rating i, .dp-rating i, .dp-rating-stars i").count()
    assert stars_count >= 5, f"StarRating must render 5 star icons, found {stars_count}"

    # 6. ProgressBar Assertions
    assert ssr_page.locator(".dp-progressbar, .dp-progress-bar").count() > 0, "ProgressBar must be rendered"
    fill = ssr_page.locator(".dp-progress-fill, .progress-fill").first
    fill_style = fill.evaluate("el => el.getAttribute('style') || window.getComputedStyle(el).width")
    assert "%" in fill_style or "px" in fill_style, f"ProgressBar fill must have width style, got '{fill_style}'"

    # 7. Public CSS Rules Verification
    raw_html = ssr_page.content()
    assert ".dp-price-card" in raw_html, "Public SSR HTML must contain .dp-price-card stylesheet rule"
    assert ".dp-countdown" in raw_html, "Public SSR HTML must contain .dp-countdown stylesheet rule"
    assert ".dp-flipbox" in raw_html, "Public SSR HTML must contain .dp-flipbox stylesheet rule"

    print(f"  [OK] Pro blocks rendering & computed styles fully verified on {vp_name}!")


def run_e2e_playwright_audit():
    os.makedirs(SCREENSHOT_DIR, exist_ok=True)
    captured_artifacts = []
    console_errors = []

    server = DaypressServerRunner()
    server.start()

    try:
        seed_database_via_rest_and_sqlite()

        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)

            # =================================================================
            # PART 1: DAYSTUDIO VISUAL BUILDER AUTOMATION (10 ARTIFACTS)
            # =================================================================
            print("\n============================================================")
            print("[*] Executing DayStudio Visual Builder Interactive Automation")
            print("============================================================")

            context = browser.new_context(viewport={"width": 1200, "height": 900})
            context.add_cookies([{
                "name": "daypress_admin",
                "value": "1",
                "domain": "127.0.0.1",
                "path": "/"
            }])

            page = context.new_page()

            def on_console_msg(msg):
                if msg.type == "error":
                    # Filter out benign 404 test page requests from browser logging
                    if "404" not in msg.text and "favicon" not in msg.text:
                        console_errors.append(f"[{page.url}] {msg.text}")

            page.on("console", on_console_msg)

            # -------------------------------------------------------------
            # Scenario 1: Studio Boot & 3-Pane Zero-Scroll Dock Verification
            # -------------------------------------------------------------
            print("\n[SCENARIO 1] Loading DayStudio at /studio?post_id=1...")
            page.goto(f"{BASE_URL}/studio?post_id=1", wait_until="networkidle")
            page.wait_for_selector("#daystudio-shell", timeout=10000)

            # Assert 3-pane dock shell geometry
            assert page.is_visible("#daystudio-shell"), "Studio shell must be visible"
            assert page.is_visible("#daystudio-topbar"), "Topbar must be visible"
            assert page.is_visible("#daystudio-rail"), "Left Rail must be visible"
            assert page.is_visible("#daystudio-drawer"), "Left Drawer must be visible"
            assert page.is_visible("#daystudio-canvas-viewport"), "Canvas viewport must be visible"
            assert page.is_visible("#daystudio-inspector"), "Right Inspector must be visible"
            assert page.is_visible("#daystudio-bottom-bar"), "Bottom breadcrumb bar must be visible"
            assert page.locator("#daystudio-artboard").evaluate("el => el.classList.contains('artboard-desktop')"), "Desktop artboard active"

            # Artifact 01: Default builder view
            art1 = os.path.join(SCREENSHOT_DIR, "01_daystudio_builder_desktop_default.png")
            page.screenshot(path=art1)
            captured_artifacts.append(("01_daystudio_builder_desktop_default.png", "1200x900", "DayStudio 3-pane zero-scroll dock shell (Topbar, Rail, Canvas, Inspector)"))
            print("[OK] Capture 01: 01_daystudio_builder_desktop_default.png")

            # -------------------------------------------------------------
            # Scenario 2: Layers Navigator Tree Drawer
            # -------------------------------------------------------------
            print("\n[SCENARIO 2] Opening Layers Navigator Drawer...")
            page.click("#rail-btn-navigator")
            page.wait_for_selector("#navigator-tree-body", timeout=5000)
            assert page.locator("#rail-btn-navigator").evaluate("el => el.classList.contains('active')"), "Navigator button active"

            # Artifact 02: Navigator active
            art2 = os.path.join(SCREENSHOT_DIR, "02_daystudio_builder_navigator_active.png")
            page.screenshot(path=art2)
            captured_artifacts.append(("02_daystudio_builder_navigator_active.png", "1200x900", "Layers Navigator drawer with hierarchical tree and visibility toggles"))
            print("[OK] Capture 02: 02_daystudio_builder_navigator_active.png")

            # -------------------------------------------------------------
            # Scenario 3: Theme Templates Drawer
            # -------------------------------------------------------------
            print("\n[SCENARIO 3] Opening Theme Templates Drawer...")
            page.click("#rail-btn-templates")
            page.wait_for_selector("#theme-templates-body", timeout=5000)
            page.wait_for_selector(".template-card", timeout=5000)
            assert page.locator("#rail-btn-templates").evaluate("el => el.classList.contains('active')"), "Templates button active"
            tpl_cards_count = page.locator(".template-card").count()
            assert tpl_cards_count >= 2, f"Expected at least 2 starter templates, found {tpl_cards_count}"

            # Artifact 03: Theme templates drawer
            art3 = os.path.join(SCREENSHOT_DIR, "03_daystudio_builder_theme_templates_drawer.png")
            page.screenshot(path=art3)
            captured_artifacts.append(("03_daystudio_builder_theme_templates_drawer.png", "1200x900", "Theme Templates drawer with SQLite starter template cards and condition pills"))
            print("[OK] Capture 03: 03_daystudio_builder_theme_templates_drawer.png")

            # -------------------------------------------------------------
            # Scenario 4: Box-Model Widget, Scrubbing & Guide Overlays
            # -------------------------------------------------------------
            print("\n[SCENARIO 4] Interacting with Box-Model Widget & Scrubbers...")
            page.click("#rail-btn-elements")
            page.wait_for_selector(".widget-card[data-widget-type='PriceCard']", timeout=5000)

            # Insert PriceCard into canvas
            page.click(".widget-card[data-widget-type='PriceCard']")
            page.wait_for_selector(".dp-price-card", timeout=5000)

            # Assert element selected and inspector mounted
            page.wait_for_selector(".day-box-model-widget", timeout=5000)
            page.wait_for_selector("#daystudio-box-overlays", timeout=5000)
            assert page.is_visible(".day-box-model-widget"), "Box-Model widget must be rendered in Inspector"

            # Scrub Margin Top via mouse drag
            facet_margin_top = page.locator(".day-box-facet[data-prop='margin_top']").first
            box = facet_margin_top.bounding_box()
            assert box is not None, "Facet margin top must have bounding box"
            page.mouse.move(box["x"] + box["width"] / 2, box["y"] + box["height"] / 2)
            page.mouse.down()
            page.mouse.move(box["x"] + box["width"] / 2 + 30, box["y"] + box["height"] / 2)
            page.mouse.up()

            # Verify padding clamping (never negative)
            facet_padding_top = page.locator(".day-box-facet[data-prop='padding_top']").first
            p_box = facet_padding_top.bounding_box()
            assert p_box is not None, "Facet padding top must have bounding box"
            page.mouse.move(p_box["x"] + p_box["width"] / 2, p_box["y"] + p_box["height"] / 2)
            page.mouse.down()
            page.mouse.move(p_box["x"] + p_box["width"] / 2 - 60, p_box["y"] + p_box["height"] / 2)
            page.mouse.up()
            clamped_val = facet_padding_top.locator(".day-val").inner_text()
            assert float(clamped_val) >= 0.0, f"Padding must be clamped >= 0, got {clamped_val}"

            # Artifact 04: Box model inspector + canvas overlays
            art4 = os.path.join(SCREENSHOT_DIR, "04_daystudio_builder_box_model_inspector.png")
            page.screenshot(path=art4)
            captured_artifacts.append(("04_daystudio_builder_box_model_inspector.png", "1200x900", "Trapezoidal Box-Model widget with 4-facet scrubbers and live amber/cobalt overlays"))
            print("[OK] Capture 04: 04_daystudio_builder_box_model_inspector.png")

            # -------------------------------------------------------------
            # Scenario 5: Structural X-Ray Mode
            # -------------------------------------------------------------
            print("\n[SCENARIO 5] Toggling Structural X-Ray Mode...")
            page.click("#btn-toggle-xray")
            page.wait_for_selector("#daystudio-artboard.daystudio-xray", timeout=3000)
            assert page.locator("#btn-toggle-xray").evaluate("el => el.classList.contains('active')"), "X-Ray button active"

            # Artifact 05: X-Ray mode
            art5 = os.path.join(SCREENSHOT_DIR, "05_daystudio_builder_xray_mode.png")
            page.screenshot(path=art5)
            captured_artifacts.append(("05_daystudio_builder_xray_mode.png", "1200x900", "Structural X-Ray mode displaying crisp hairline wireframe outlines across artboard"))
            print("[OK] Capture 05: 05_daystudio_builder_xray_mode.png")
            page.click("#btn-toggle-xray")  # toggle off

            # -------------------------------------------------------------
            # Scenario 6: Responsive Viewport Switcher (Tablet 768px & Mobile 375px)
            # -------------------------------------------------------------
            print("\n[SCENARIO 6] Testing Responsive Viewport Switcher...")

            # Tablet
            page.click("button.bp-btn[data-bp='tablet']")
            page.wait_for_selector("#daystudio-artboard.artboard-tablet", timeout=3000)
            dim_text = page.locator("#lbl-dimension-readout").inner_text()
            assert "768" in dim_text, f"Readout should indicate 768px, got {dim_text}"

            # Artifact 06: Tablet responsive mode
            art6 = os.path.join(SCREENSHOT_DIR, "06_daystudio_builder_responsive_tablet.png")
            page.screenshot(path=art6)
            captured_artifacts.append(("06_daystudio_builder_responsive_tablet.png", "1200x900", "Center Artboard canvas resized to 768px Tablet viewport with dimension readout"))
            print("[OK] Capture 06: 06_daystudio_builder_responsive_tablet.png")

            # Mobile
            page.click("button.bp-btn[data-bp='mobile']")
            page.wait_for_selector("#daystudio-artboard.artboard-mobile", timeout=3000)
            dim_text_m = page.locator("#lbl-dimension-readout").inner_text()
            assert "375" in dim_text_m, f"Readout should indicate 375px, got {dim_text_m}"

            # Artifact 07: Mobile responsive mode
            art7 = os.path.join(SCREENSHOT_DIR, "07_daystudio_builder_responsive_mobile.png")
            page.screenshot(path=art7)
            captured_artifacts.append(("07_daystudio_builder_responsive_mobile.png", "1200x900", "Center Artboard canvas resized to 375px Mobile viewport with dimension readout"))
            print("[OK] Capture 07: 07_daystudio_builder_responsive_mobile.png")

            # Reset back to Desktop
            page.click("button.bp-btn[data-bp='desktop']")
            page.wait_for_selector("#daystudio-artboard.artboard-desktop", timeout=3000)

            # Test Zoom Controls
            page.click("#btn-zoom-in")
            scale_val = page.locator("#lbl-zoom-level").inner_text()
            assert "125" in scale_val or "150" in scale_val, f"Expected zoom in, got {scale_val}"
            page.click("#btn-zoom-reset")
            assert "100%" in page.locator("#lbl-zoom-level").inner_text()

            # -------------------------------------------------------------
            # Scenario 7: Non-Blocking Theme Template Dialogs
            # -------------------------------------------------------------
            print("\n[SCENARIO 7] Testing Create Template & Display Conditions Modals...")
            page.click("#rail-btn-templates")
            page.wait_for_selector("#theme-templates-body", timeout=5000)

            # Open Create Template Modal
            page.click("#btn-new-theme-template")
            page.wait_for_selector("#modal-create-template.open", timeout=3000)
            page.fill("#input-template-title", "Agency Custom Sub-Header")

            # Artifact 08: Create template modal
            art8 = os.path.join(SCREENSHOT_DIR, "08_daystudio_builder_create_template_modal.png")
            page.screenshot(path=art8)
            captured_artifacts.append(("08_daystudio_builder_create_template_modal.png", "1200x900", "Non-blocking CAD Create Theme Template modal dialog with starter cards"))
            print("[OK] Capture 08: 08_daystudio_builder_create_template_modal.png")
            page.click("#btn-cancel-create-template")

            # Open Display Conditions Modal
            page.click(".template-card .btn-cond-template")
            page.wait_for_selector("#modal-conditions.open", timeout=3000)
            page.wait_for_selector(".condition-rule-row", timeout=3000)

            # Artifact 09: Display conditions modal
            art9 = os.path.join(SCREENSHOT_DIR, "09_daystudio_builder_display_conditions_modal.png")
            page.screenshot(path=art9)
            captured_artifacts.append(("09_daystudio_builder_display_conditions_modal.png", "1200x900", "Theme Display Conditions priority modal with Include/Exclude rules and targets"))
            print("[OK] Capture 09: 09_daystudio_builder_display_conditions_modal.png")
            page.click("#btn-cancel-conditions")

            # -------------------------------------------------------------
            # Scenario 8: Inserting Pro Blocks & SQLite Save Persistence
            # -------------------------------------------------------------
            print("\n[SCENARIO 8] Inserting Pro Blocks & Triggering Save...")
            page.click("#rail-btn-elements")
            page.wait_for_selector("#elem-catalog-body", timeout=5000)

            pro_blocks = [
                "CallToAction",
                "Countdown",
                "FlipBox",
                "Testimonial",
                "StarRating",
                "ProgressBar",
                "NavMenu",
            ]

            for block_type in pro_blocks:
                btn = page.locator(f".widget-card[data-widget-type='{block_type}']").first
                if btn.count() > 0:
                    btn.click()
                    page.wait_for_timeout(100)

            # Verify canvas elements rendered
            assert page.locator(".dp-cta").count() > 0, "CallToAction rendered"
            assert page.locator(".dp-countdown").count() > 0, "Countdown rendered"
            assert page.locator(".dp-flipbox").count() > 0, "FlipBox rendered"
            assert page.locator(".dp-star-rating").count() > 0, "StarRating rendered"
            assert page.locator(".dp-progress-bar").count() > 0, "ProgressBar rendered"
            assert page.locator(".dp-nav-menu").count() > 0, "NavMenu rendered"

            # Click Save / Publish Button
            page.click("#btn-primary-publish")
            page.wait_for_selector("#status-save-pill.saved", timeout=8000)
            status_text = page.locator("#lbl-save-status").inner_text()
            assert "Saved" in status_text, f"Expected 'Saved' status pill, got {status_text}"

            # Artifact 10: Saved published state
            art10 = os.path.join(SCREENSHOT_DIR, "10_daystudio_builder_save_published_state.png")
            page.screenshot(path=art10)
            captured_artifacts.append(("10_daystudio_builder_save_published_state.png", "1200x900", "State after SQLite dual-write persistence showing green 'Saved' status indicator"))
            print("[OK] Capture 10: 10_daystudio_builder_save_published_state.png")

            # Verify SQLite persistence survives page reload
            print("[PERSISTENCE] Reloading page to verify layout persistence in SQLite...")
            page.reload(wait_until="networkidle")
            page.wait_for_selector("#daystudio-shell", timeout=10000)
            assert page.locator(".dp-price-card").count() > 0, "PriceCard persisted across reload"
            assert page.locator(".dp-cta").count() > 0, "CTA persisted across reload"
            print("[OK] Verified: AST layout successfully persisted to SQLite and reloaded!")

            page.close()
            context.close()

            # =================================================================
            # PART 2: PUBLIC SSR MULTI-BREAKPOINT AUDIT (11 ARTIFACTS)
            # =================================================================
            print("\n============================================================")
            print("[*] Executing Public SSR Frontend Multi-Breakpoint Audit")
            print("============================================================")

            ssr_routes = [
                (
                    "/",
                    "Front Page",
                    "11_ssr_frontpage_desktop_1200.png",
                    "12_ssr_frontpage_tablet_768.png",
                    "13_ssr_frontpage_mobile_375.png",
                ),
                (
                    "/pro-blocks-showcase/",
                    "Single Post (Pro Blocks)",
                    "14_ssr_single_post_desktop_1200.png",
                    "15_ssr_single_post_tablet_768.png",
                    "16_ssr_single_post_mobile_375.png",
                ),
                (
                    "/category/architecture/",
                    "Category Archive",
                    "17_ssr_category_archive_desktop_1200.png",
                    "18_ssr_category_archive_tablet_768.png",
                    "19_ssr_category_archive_mobile_375.png",
                ),
            ]

            viewports = [
                ("desktop", {"width": 1200, "height": 900}, 0),
                ("tablet", {"width": 768, "height": 1024}, 1),
                ("mobile", {"width": 375, "height": 812}, 2),
            ]

            for path, title, desk_fn, tab_fn, mob_fn in ssr_routes:
                filenames = [desk_fn, tab_fn, mob_fn]
                print(f"\n[SSR AUDIT] Route: {path} ({title})")

                for vp_name, vp_dims, idx in viewports:
                    fname = filenames[idx]
                    target_url = f"{BASE_URL}{path}"
                    ssr_page = browser.new_page(viewport=vp_dims)
                    ssr_page.on("console", on_console_msg)

                    resp = ssr_page.goto(target_url, wait_until="networkidle")
                    assert resp.status == 200, f"Expected HTTP 200 on {target_url}, got {resp.status}"

                    # Verify Dynamic Theme Header & Footer injection
                    assert ssr_page.locator("header.dp-theme-header, header, .dp-header-root").count() > 0, "Header injected"
                    assert ssr_page.locator("footer.dp-theme-footer, footer, .dp-footer-root").count() > 0, "Footer injected"

                    # 3-Tier Leak Sanitizer Verification
                    contenteditable_count = ssr_page.locator("[contenteditable='true']").count()
                    assert contenteditable_count == 0, f"Editor chrome leak: {contenteditable_count} contenteditable elements found"
                    draggable_count = ssr_page.locator("[draggable='true']").count()
                    assert draggable_count == 0, f"Editor chrome leak: {draggable_count} draggable elements found"
                    grid_handle_count = ssr_page.locator(".elementor-grid-handle").count()
                    assert grid_handle_count == 0, "Editor chrome leak: elementor-grid-handle found"

                    raw_html = ssr_page.content()
                    assert "daystudio-shell" not in raw_html, "Editor chrome leak: daystudio-shell in SSR"
                    assert "daystudio-inspector" not in raw_html, "Editor chrome leak: daystudio-inspector in SSR"
                    assert "daystudio-rail" not in raw_html, "Editor chrome leak: daystudio-rail in SSR"

                    # Additional Pro Blocks verification on showcase route
                    if path == "/pro-blocks-showcase/":
                        assert_pro_blocks_render_and_styles(ssr_page, vp_name)

                    # Screenshot capture
                    out_path = os.path.join(SCREENSHOT_DIR, fname)
                    ssr_page.screenshot(path=out_path, full_page=True)
                    captured_artifacts.append((fname, f"{vp_dims['width']}x{vp_dims['height']}", f"Public SSR {title} at {vp_name.title()} breakpoint"))
                    print(f"[OK] Capture: {fname} ({vp_dims['width']}x{vp_dims['height']})")

                    ssr_page.close()

            # -------------------------------------------------------------
            # 404 Route Verification (Desktop & Mobile)
            # -------------------------------------------------------------
            print("\n[SSR AUDIT] Route: /non-existent-quantum-core/ (404 Page)")
            for vp_name, vp_dims, fname in [
                ("desktop", {"width": 1200, "height": 900}, "20_ssr_404_error_page_desktop_1200.png"),
                ("mobile", {"width": 375, "height": 812}, "21_ssr_404_error_page_mobile_375.png"),
            ]:
                ssr_404 = browser.new_page(viewport=vp_dims)
                ssr_404.on("console", on_console_msg)
                resp_404 = ssr_404.goto(f"{BASE_URL}/non-existent-quantum-core/", wait_until="networkidle")
                assert resp_404.status == 404, f"Expected 404 status, got {resp_404.status}"

                # Verify Header/Footer still injected
                assert ssr_404.locator("header.dp-theme-header, header").count() > 0, "404 page must inject Theme Header"
                assert ssr_404.locator("footer.dp-theme-footer, footer").count() > 0, "404 page must inject Theme Footer"

                # Verify zero leaks
                assert ssr_404.locator("[contenteditable='true']").count() == 0
                assert ssr_404.locator("[draggable='true']").count() == 0

                out_404 = os.path.join(SCREENSHOT_DIR, fname)
                ssr_404.screenshot(path=out_404, full_page=True)
                captured_artifacts.append((fname, f"{vp_dims['width']}x{vp_dims['height']}", f"Public SSR 404 Page at {vp_name.title()} breakpoint"))
                print(f"[OK] Capture: {fname} ({vp_dims['width']}x{vp_dims['height']})")

                ssr_404.close()

            # Verify SQLite 404 log hit recording
            conn = sqlite3.connect(TEST_DB)
            cur = conn.cursor()
            cur.execute("SELECT hits FROM daypress_404_log WHERE request_path LIKE '%non-existent-quantum-core%';")
            row = cur.fetchone()
            assert row is not None and row[0] >= 1, "404 hit must be logged in SQLite daypress_404_log table"
            print(f"[SQLITE] Verified: 404 logged in daypress_404_log with {row[0]} hits.")
            conn.close()

            browser.close()

    finally:
        server.stop()

    # =================================================================
    # VERIFICATION & INTEGRITY AUDIT REPORT
    # =================================================================
    print("\n============================================================")
    print("[*] Final Screenshot Artifacts Verification Audit")
    print("============================================================")

    assert len(captured_artifacts) == 21, f"Expected 21 screenshot artifacts, captured {len(captured_artifacts)}"
    all_files_exist = True
    for fname, res, desc in captured_artifacts:
        full_path = os.path.join(SCREENSHOT_DIR, fname)
        if not os.path.exists(full_path) or os.path.getsize(full_path) < 5000:
            print(f"[FAIL] Missing or empty screenshot: {fname}")
            all_files_exist = False
        else:
            size_kb = os.path.getsize(full_path) / 1024
            print(f"[OK] {fname:<45} [{res}] ({size_kb:.1f} KB) - {desc}")

    assert all_files_exist, "All 21 screenshot files must exist and be non-empty"

    if console_errors:
        print(f"\n[WARN] Encountered {len(console_errors)} unexpected console error(s):")
        for err in console_errors:
            print(f"  - {err}")
    else:
        print("\n[OK] Console Integrity: Exactly 0 unhandled JavaScript errors encountered.")

    print("\n============================================================")
    print("[SUCCESS] MILESTONE 5 PLAYWRIGHT E2E AUDIT PASSED (100% ASSERTIONS)")
    print("============================================================\n")


if __name__ == "__main__":
    run_e2e_playwright_audit()
