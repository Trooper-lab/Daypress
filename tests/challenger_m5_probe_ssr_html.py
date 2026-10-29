import os
import sys
import subprocess
import time
import urllib.request
import json
import sqlite3

TEST_PORT = 8098
BASE_URL = f"http://127.0.0.1:{TEST_PORT}"
TEST_DB = "data/test_m5_probe.sqlite"

def main():
    os.makedirs("data", exist_ok=True)
    for ext in ["", "-shm", "-wal", "-journal"]:
        p = TEST_DB + ext
        if os.path.exists(p):
            try:
                os.remove(p)
            except Exception:
                pass

    bin_path = os.path.abspath("target/debug/daypress.exe")
    proc = subprocess.Popen(
        [bin_path, "--port", str(TEST_PORT), "--db", TEST_DB],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
    )

    try:
        ready = False
        for _ in range(30):
            time.sleep(0.3)
            try:
                with urllib.request.urlopen(f"{BASE_URL}/wp-json", timeout=1) as resp:
                    if resp.status == 200:
                        ready = True
                        break
            except Exception:
                pass
        if not ready:
            print("Server failed to start")
            return

        # Seed post 1 with Pro blocks layout JSON (similar to what builder saves)
        layout_json = {
            "version": 1,
            "root": {
                "id": "root_node",
                "type": "Root",
                "styles": {},
                "children": [
                    {
                        "id": "sec_1",
                        "type": "Section",
                        "styles": {"padding_top": "40px", "padding_bottom": "40px"},
                        "children": [
                            {
                                "id": "cont_1",
                                "type": "Container",
                                "styles": {},
                                "children": [
                                    {
                                        "id": "pricecard_1",
                                        "type": "PriceCard",
                                        "data": {},
                                        "styles": {},
                                        "children": []
                                    },
                                    {
                                        "id": "countdown_1",
                                        "type": "Countdown",
                                        "data": {},
                                        "styles": {},
                                        "children": []
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }

        conn = sqlite3.connect(TEST_DB)
        cur = conn.cursor()
        cur.execute(
            """
            INSERT INTO posts (
                ID, post_author, post_date, post_date_gmt, post_content, post_title,
                post_excerpt, post_status, post_name, post_modified, post_modified_gmt,
                guid, post_type, layout_json
            ) VALUES (
                1, 1, datetime('now'), datetime('now'),
                '<p>Test</p>',
                'Pro Blocks Showcase',
                'Excerpt',
                'publish', 'pro-blocks-showcase', datetime('now'), datetime('now'),
                'http://127.0.0.1:8098/pro-blocks-showcase/', 'page', ?
            );
            """,
            (json.dumps(layout_json),)
        )
        conn.commit()
        conn.close()

        # Fetch /pro-blocks-showcase/
        with urllib.request.urlopen(f"{BASE_URL}/pro-blocks-showcase/") as resp:
            html = resp.read().decode("utf-8")

        print("=== PROBE SSR HTML RESULTS ===")
        print(f"Has .dp-price-card in HTML: {'.dp-price-card' in html}")
        print(f"Has .dp-price-card in CSS style tag: {'.dp-price-card {' in html or '.dp-price-card{' in html}")
        print(f"Has .dp-countdown in CSS style tag: {'.dp-countdown {' in html or '.dp-countdown{' in html}")

        # Check for PriceCard rendered HTML snippet
        if "dp-price-card" in html:
            idx = html.find("dp-price-card")
            start = max(0, idx - 40)
            end = min(len(html), idx + 350)
            print("\nRendered Snippet:")
            print(html[start:end])

    finally:
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
    main()
