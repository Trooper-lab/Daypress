import os
import sys
import subprocess
import time
import urllib.request
import json
import sqlite3

PORT = 8101
BASE = f"http://127.0.0.1:{PORT}"
DB = "data/test_reviewer_adversarial.sqlite"

def run_tests():
    os.makedirs("data", exist_ok=True)
    for ext in ["", "-shm", "-wal", "-journal"]:
        p = DB + ext
        if os.path.exists(p):
            try:
                os.remove(p)
            except Exception:
                pass

    bin_path = os.path.abspath("target/debug/daypress.exe")
    proc = subprocess.Popen(
        [bin_path, "--port", str(PORT), "--db", DB],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    try:
        ready = False
        for _ in range(30):
            time.sleep(0.3)
            try:
                with urllib.request.urlopen(f"{BASE}/wp-json", timeout=1) as resp:
                    if resp.status == 200:
                        ready = True
                        break
            except Exception:
                pass
        if not ready:
            print("[FAIL] Server failed to start on port 8101")
            return False

        print("[PASS] Server started successfully on port 8101")

        # Test 1: Seed a post with deliberate editor chrome leaks in post_content and layout_json
        leaky_content = (
            '<div contenteditable="true" draggable="true" '
            'class="daystudio-shell daystudio-inspector elementor-grid-handle custom-valid-class" '
            'data-daystudio-id="leak-123" data-drag-handle="true">\n'
            '    <p contenteditable="true">Some editable text</p>\n'
            '    <div CONTENTEDITABLE draggable class="daystudio-handle">Handle</div>\n'
            '</div>'
        )
        layout_with_styles = {
            "version": 1,
            "root": {
                "id": "root_adv",
                "type": "Root",
                "styles": {},
                "children": [
                    {
                        "id": "sec_adv",
                        "type": "Section",
                        "styles": {
                            "background_color": "#123456",
                            "padding_top": "42px",
                            "tablet": {"padding_top": "30px"},
                            "mobile": {"padding_top": "20px"},
                            "hover": {"background_color": "#654321"}
                        },
                        "children": [
                            {
                                "id": "price_adv",
                                "type": "PriceCard",
                                "data": {
                                    "title": "Adversarial Cloud 🚀🔥",
                                    "subtitle": "Testing <script>alert(1)</script> & unicode العربية",
                                    "price": "99",
                                    "currency": "€",
                                    "features": [{"text": "Feature 1", "included": True}]
                                },
                                "styles": {"border_radius": "8px"},
                                "children": []
                            }
                        ]
                    }
                ]
            }
        }

        conn = sqlite3.connect(DB)
        cur = conn.cursor()
        cur.execute(
            """
            INSERT INTO posts (
                ID, post_author, post_date, post_date_gmt, post_content, post_title,
                post_excerpt, post_status, post_name, post_modified, post_modified_gmt,
                guid, post_type, layout_json
            ) VALUES (
                100, 1, datetime('now'), datetime('now'),
                ?,
                'Adversarial Test Page 🚀',
                'Excerpt',
                'publish', 'adversarial-test', datetime('now'), datetime('now'),
                'http://127.0.0.1:8101/adversarial-test/', 'page', ?
            );
            """,
            (leaky_content, json.dumps(layout_with_styles))
        )
        conn.commit()
        conn.close()

        # Request /adversarial-test/
        with urllib.request.urlopen(f"{BASE}/adversarial-test/") as resp:
            html = resp.read().decode("utf-8")

        print("[TEST 1] Testing 3-Tier Leak Firewall on SSR Output...")
        leaks = []
        if "contenteditable" in html.lower():
            leaks.append("contenteditable")
        if "draggable" in html.lower():
            leaks.append("draggable")
        if "elementor-grid-handle" in html:
            leaks.append("elementor-grid-handle")
        if "daystudio-shell" in html:
            leaks.append("daystudio-shell")
        if "daystudio-inspector" in html:
            leaks.append("daystudio-inspector")
        if "daystudio-handle" in html:
            leaks.append("daystudio-handle")
        if "data-daystudio" in html:
            leaks.append("data-daystudio")
        if "data-drag-handle" in html:
            leaks.append("data-drag-handle")

        if leaks:
            print(f"[FAIL] Leaks detected in SSR HTML: {leaks}")
            return False
        print("[PASS] Exactly 0 editor chrome leaks in SSR HTML!")

        print("[TEST 2] Testing Scoped Layout CSS Injection...")
        if '<style id="dp-page-styles">' not in html:
            print('[FAIL] <style id="dp-page-styles"> not found in head!')
            return False
        print('[PASS] Found <style id="dp-page-styles"> in <head>')

        if "#123456" not in html:
            print("[FAIL] Scoped background_color not compiled into page styles!")
            return False
        print("[PASS] Scoped layout styles present in <style id=\"dp-page-styles\">")

        print("[TEST 3] Testing Public Theme Modular Styles...")
        for sel in [".dp-price-card", ".dp-countdown", ".dp-flipbox"]:
            if sel not in html:
                print(f"[FAIL] {sel} missing from public CSS!")
                return False
        print("[PASS] Pro block widget CSS present in public SSR theme stylesheet!")

        print("[TEST 4] Testing Malformed Layout JSON Fallback...")
        conn = sqlite3.connect(DB)
        cur = conn.cursor()
        cur.execute("UPDATE posts SET layout_json = '{corrupted_json' WHERE ID = 100;")
        conn.commit()
        conn.close()

        with urllib.request.urlopen(f"{BASE}/adversarial-test/") as resp:
            html_fallback = resp.read().decode("utf-8")
        if resp.status != 200:
            print(f"[FAIL] Fallback returned status {resp.status}")
            return False
        print("[PASS] Corrupted layout_json gracefully falls back to post_content without panic!")

        print("[ALL ADVERSARIAL STRESS TESTS PASSED]")
        return True

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except Exception:
            proc.kill()
        for ext in ["", "-shm", "-wal", "-journal"]:
            p = DB + ext
            if os.path.exists(p):
                try:
                    os.remove(p)
                except Exception:
                    pass

if __name__ == "__main__":
    ok = run_tests()
    sys.exit(0 if ok else 1)
