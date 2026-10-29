"""
Empirical Adversarial Test Harness:
Verifies DayStudio Artboard CSS Containment and CAD Dock Invariants under hostile theme styles.
"""

import os
import sys
import time
import json
import sqlite3
import subprocess
import urllib.request

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

from playwright.sync_api import sync_playwright

TEST_PORT = 8199
BASE_URL = f"http://127.0.0.1:{TEST_PORT}"
TEST_DB = "data/test_adversarial_m4.sqlite"
SCREENSHOT_PATH = os.path.abspath("tests/screenshots/adversarial_containment_proof.png")

# Hostile theme stylesheet designed to break layout chrome and bleed out
HOSTILE_THEME_CSS = """
:root {
    --day-chassis: #ff0000 !important;
    --day-topbar-h: 200px !important;
}
html, body {
    background: #ff00ff !important;
    margin: 100px !important;
    padding: 100px !important;
}
body {
    font-size: 50px !important;
    color: #ffff00 !important;
    line-height: 4 !important;
}
/* Fixed header attempting to completely occlude the topbar */
header {
    position: fixed !important;
    top: 0 !important;
    left: 0 !important;
    width: 100vw !important;
    height: 300px !important;
    z-index: 999999 !important;
    background: rgba(255, 0, 0, 0.95) !important;
    border: 10px solid lime !important;
}
/* Fixed nav attempting to completely occlude the bottom bar and left rail */
nav {
    position: fixed !important;
    bottom: 0 !important;
    left: 0 !important;
    width: 100vw !important;
    height: 200px !important;
    z-index: 999999 !important;
    background: yellow !important;
}
* {
    box-sizing: content-box !important;
}
"""

HOSTILE_KIT_CSS = """
.section {
    position: fixed !important;
    top: 0 !important;
    right: 0 !important;
    width: 600px !important;
    height: 600px !important;
    z-index: 999999 !important;
    background: cyan !important;
}
"""

def setup_test_database():
    os.makedirs(os.path.dirname(os.path.abspath(TEST_DB)), exist_ok=True)
    for ext in ["", "-shm", "-wal", "-journal"]:
        p = TEST_DB + ext
        if os.path.exists(p):
            try:
                os.remove(p)
            except Exception:
                pass

    conn = sqlite3.connect(TEST_DB)
    cur = conn.cursor()

    # Create tables
    cur.execute("""
        CREATE TABLE IF NOT EXISTS options (
            option_id INTEGER PRIMARY KEY AUTOINCREMENT,
            option_name TEXT UNIQUE NOT NULL,
            option_value TEXT NOT NULL,
            autoload TEXT NOT NULL DEFAULT 'yes'
        );
    """)
    cur.execute("""
        CREATE TABLE IF NOT EXISTS posts (
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
    """)
    cur.execute("""
        CREATE TABLE IF NOT EXISTS postmeta (
            meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER NOT NULL,
            meta_key TEXT NOT NULL,
            meta_value TEXT NOT NULL
        );
    """)

    # Seed settings with hostile theme css
    cur.execute("INSERT INTO options (option_name, option_value) VALUES ('blogname', 'Daypress Adversarial');")
    cur.execute("INSERT INTO options (option_name, option_value) VALUES ('custom_css', ?);", (HOSTILE_THEME_CSS,))
    cur.execute("INSERT INTO options (option_name, option_value) VALUES ('elementor_kit_css', ?);", (HOSTILE_KIT_CSS,))

    # Seed post 1 with custom content
    post_html = '<section class="section"><h2>Adversarial Section</h2><header><h3>Hostile Artboard Header</h3></header><nav><a href="#">Hostile Nav</a></nav></section>'
    cur.execute("""
        INSERT INTO posts (ID, post_title, post_name, post_type, post_status, post_content)
        VALUES (1, 'Adversarial Test Page', 'adversarial-test', 'page', 'publish', ?);
    """, (post_html,))

    conn.commit()
    conn.close()
    print("[DB] Adversarial test database prepared with hostile theme CSS.")

def run_adversarial_containment_audit():
    setup_test_database()

    bin_path = os.path.abspath("target/debug/daypress.exe")
    assert os.path.exists(bin_path), f"Binary {bin_path} not found"

    print(f"[SERVER] Starting Daypress on port {TEST_PORT}...")
    proc = subprocess.Popen(
        [bin_path, "--port", str(TEST_PORT), "--db", TEST_DB],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True
    )

    ready = False
    for _ in range(40):
        time.sleep(0.25)
        try:
            with urllib.request.urlopen(f"{BASE_URL}/wp-json", timeout=1) as resp:
                if resp.status == 200:
                    ready = True
                    break
        except Exception:
            pass

    if not ready:
        proc.kill()
        raise RuntimeError("Daypress failed to start")

    print(f"[SERVER] Daypress ready on {BASE_URL}")

    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(viewport={"width": 1440, "height": 900})
            context.add_cookies([{
                "name": "daypress_admin",
                "value": "1",
                "domain": "127.0.0.1",
                "path": "/"
            }])
            page = context.new_page()

            url = f"{BASE_URL}/studio?post_id=1"
            print(f"[BROWSER] Navigating to {url}...")
            page.goto(url, wait_until="networkidle")
            page.wait_for_selector("#daystudio-shell", timeout=10000)

            # -------------------------------------------------------------
            # 1. Topbar CAD Chrome Invariant Verification
            # -------------------------------------------------------------
            topbar = page.locator("#daystudio-topbar")
            assert topbar.is_visible(), "Topbar must be visible"
            topbar_rect = topbar.bounding_box()
            assert topbar_rect is not None
            print(f"[TOPBAR] rect: {topbar_rect}")

            # Topbar height MUST be <= 44px
            assert topbar_rect["height"] <= 44.0, f"Topbar height must be <= 44px, got {topbar_rect['height']}px"
            assert topbar_rect["height"] >= 38.0, f"Topbar height must be >= 38px, got {topbar_rect['height']}px"
            assert topbar_rect["y"] == 0.0, f"Topbar must be at y=0, got {topbar_rect['y']}"

            # Topbar background color must NOT be red (#ff0000)
            topbar_bg = topbar.evaluate("el => window.getComputedStyle(el).backgroundColor")
            print(f"[TOPBAR] background-color: {topbar_bg}")
            assert "255, 0, 0" not in topbar_bg, f"Topbar background contaminated by hostile CSS: {topbar_bg}"

            # Hit-test topbar: point (100, 20)
            top_hit = page.evaluate("() => { const el = document.elementFromPoint(100, 20); return el ? el.tagName + '#' + el.id + '.' + el.className : null; }")
            print(f"[TOPBAR HIT-TEST at (100,20)]: {top_hit}")
            assert "artboard" not in top_hit.lower(), f"Topbar covered by artboard content! Hit: {top_hit}"
            assert "section" not in top_hit.lower(), f"Topbar covered by section content! Hit: {top_hit}"

            # -------------------------------------------------------------
            # 2. Bottom-Bar CAD Chrome Invariant Verification
            # -------------------------------------------------------------
            bottom_bar = page.locator("#daystudio-bottom-bar")
            assert bottom_bar.is_visible(), "Bottom bar must be visible"
            bottom_rect = bottom_bar.bounding_box()
            assert bottom_rect is not None
            print(f"[BOTTOM-BAR] rect: {bottom_rect}")

            # Bottom bar height MUST be <= 32px
            assert bottom_rect["height"] <= 32.0, f"Bottom bar height must be <= 32px, got {bottom_rect['height']}px"
            assert bottom_rect["height"] >= 26.0, f"Bottom bar height must be >= 26px, got {bottom_rect['height']}px"
            expected_bottom_y = 900 - bottom_rect["height"]
            assert abs(bottom_rect["y"] - expected_bottom_y) < 1.0, f"Bottom bar not at viewport bottom: {bottom_rect['y']}"

            # Bottom bar background color must NOT be yellow
            bottom_bg = bottom_bar.evaluate("el => window.getComputedStyle(el).backgroundColor")
            print(f"[BOTTOM-BAR] background-color: {bottom_bg}")
            assert "255, 255, 0" not in bottom_bg, f"Bottom bar background contaminated: {bottom_bg}"

            # Hit-test bottom bar: point (100, 890)
            bottom_hit = page.evaluate("() => { const el = document.elementFromPoint(100, 890); return el ? el.tagName + '#' + el.id + '.' + el.className : null; }")
            print(f"[BOTTOM-BAR HIT-TEST at (100,890)]: {bottom_hit}")
            assert "artboard" not in bottom_hit.lower(), f"Bottom bar covered by artboard content! Hit: {bottom_hit}"
            assert "yellow" not in bottom_hit.lower(), f"Bottom bar covered by hostile nav! Hit: {bottom_hit}"

            # -------------------------------------------------------------
            # 3. Left Rail (<nav id="daystudio-rail">) Verification
            # -------------------------------------------------------------
            rail = page.locator("#daystudio-rail")
            assert rail.is_visible(), "Left Rail must be visible"
            rail_rect = rail.bounding_box()
            assert rail_rect is not None
            print(f"[LEFT RAIL] rect: {rail_rect}")
            assert rail_rect["width"] == 48.0, f"Left Rail width must be 48px, got {rail_rect['width']}"
            assert rail_rect["x"] == 0.0, f"Left Rail must be at x=0, got {rail_rect['x']}"
            rail_bg = rail.evaluate("el => window.getComputedStyle(el).backgroundColor")
            assert "255, 255, 0" not in rail_bg, f"Left Rail nav poisoned by theme nav style: {rail_bg}"

            # -------------------------------------------------------------
            # 4. Studio Shell & Body Containment Verification
            # -------------------------------------------------------------
            shell = page.locator("#daystudio-shell")
            shell_rect = shell.bounding_box()
            assert shell_rect is not None
            assert shell_rect["width"] == 1440.0, f"Shell width must be 1440px, got {shell_rect['width']}"
            assert shell_rect["height"] == 900.0, f"Shell height must be 900px, got {shell_rect['height']}"

            shell_styles = shell.evaluate("""el => {
                const s = window.getComputedStyle(el);
                return {
                    marginTop: s.marginTop,
                    paddingTop: s.paddingTop,
                    backgroundColor: s.backgroundColor
                };
            }""")
            print(f"[SHELL STYLES]: {shell_styles}")
            assert shell_styles["marginTop"] == "0px", f"Hostile body margin leaked into shell: {shell_styles['marginTop']}"
            assert shell_styles["paddingTop"] == "0px", f"Hostile body padding leaked into shell: {shell_styles['paddingTop']}"
            assert "255, 0, 255" not in shell_styles["backgroundColor"], f"Hostile magenta background leaked into shell: {shell_styles['backgroundColor']}"

            # -------------------------------------------------------------
            # 5. Artboard CSS Containment & Stacking Context Isolation
            # -------------------------------------------------------------
            artboard = page.locator("#daystudio-artboard")
            assert artboard.is_visible(), "Artboard must be visible"
            artboard_styles = artboard.evaluate("""el => {
                const s = window.getComputedStyle(el);
                return {
                    contain: s.contain,
                    transform: s.transform,
                    isolation: s.isolation,
                    overflow: s.overflow
                };
            }""")
            print(f"[ARTBOARD STYLES]: {artboard_styles}")
            assert "paint" in artboard_styles["contain"], f"Artboard missing contain: paint, got {artboard_styles['contain']}"
            assert "layout" in artboard_styles["contain"], f"Artboard missing contain: layout, got {artboard_styles['contain']}"
            assert artboard_styles["transform"] != "none", f"Artboard missing transform stacking context, got {artboard_styles['transform']}"
            assert artboard_styles["isolation"] == "isolate", f"Artboard missing isolation: isolate, got {artboard_styles['isolation']}"

            # Verify hostile fixed header inside artboard is contained
            artboard_rect = artboard.bounding_box()
            assert artboard_rect is not None
            hostile_header = page.locator("#daystudio-artboard header").first
            if hostile_header.count() > 0:
                header_rect = hostile_header.bounding_box()
                print(f"[HOSTILE HEADER IN ARTBOARD] rect: {header_rect}, artboard_rect: {artboard_rect}")
                if header_rect is not None:
                    # Due to transform: translateZ(0) / contain: paint, position: fixed is contained to artboard
                    assert header_rect["y"] >= artboard_rect["y"] - 1.0, f"Hostile header escaped artboard vertically: y={header_rect['y']} vs artboard y={artboard_rect['y']}"

            # -------------------------------------------------------------
            # 6. Flanking Resize Handles Unclipped Verification
            # -------------------------------------------------------------
            handle_left = page.locator("#handle-artboard-left")
            handle_right = page.locator("#handle-artboard-right")
            assert handle_left.is_visible(), "Left resize handle must be visible"
            assert handle_right.is_visible(), "Right resize handle must be visible"
            left_rect = handle_left.bounding_box()
            right_rect = handle_right.bounding_box()
            print(f"[RESIZE HANDLES] left: {left_rect}, right: {right_rect}")
            assert left_rect is not None and left_rect["width"] > 0
            assert right_rect is not None and right_rect["width"] > 0

            # Capture visual screenshot artifact
            os.makedirs(os.path.dirname(SCREENSHOT_PATH), exist_ok=True)
            page.screenshot(path=SCREENSHOT_PATH)
            print(f"[SUCCESS] Adversarial visual proof screenshot captured to {SCREENSHOT_PATH}")

            browser.close()

        print("\n========================================================")
        print("ALL EMPIRICAL ADVERSARIAL CONTAINMENT TESTS PASSED 100%!")
        print("========================================================")

    finally:
        print("[CLEANUP] Terminating server...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except Exception:
            proc.kill()
        for ext in ["", "-shm", "-wal", "-journal"]:
            p = TEST_DB + ext
            if os.path.exists(p):
                try:
                    os.remove(p)
                except Exception:
                    pass

if __name__ == "__main__":
    run_adversarial_containment_audit()
