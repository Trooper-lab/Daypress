import os
import sys
from PIL import Image, ImageStat

SCREENSHOTS_DIR = os.path.abspath("tests/screenshots")

EXPECTED_FILES = [
    ("01_daystudio_builder_desktop_default.png", 1200, 900),
    ("02_daystudio_builder_navigator_active.png", 1200, 900),
    ("03_daystudio_builder_theme_templates_drawer.png", 1200, 900),
    ("04_daystudio_builder_box_model_inspector.png", 1200, 900),
    ("05_daystudio_builder_xray_mode.png", 1200, 900),
    ("06_daystudio_builder_responsive_tablet.png", 1200, 900),
    ("07_daystudio_builder_responsive_mobile.png", 1200, 900),
    ("08_daystudio_builder_create_template_modal.png", 1200, 900),
    ("09_daystudio_builder_display_conditions_modal.png", 1200, 900),
    ("10_daystudio_builder_save_published_state.png", 1200, 900),
    ("11_ssr_frontpage_desktop_1200.png", 1200, None),
    ("12_ssr_frontpage_tablet_768.png", 768, None),
    ("13_ssr_frontpage_mobile_375.png", 375, None),
    ("14_ssr_single_post_desktop_1200.png", 1200, None),
    ("15_ssr_single_post_tablet_768.png", 768, None),
    ("16_ssr_single_post_mobile_375.png", 375, None),
    ("17_ssr_category_archive_desktop_1200.png", 1200, None),
    ("18_ssr_category_archive_tablet_768.png", 768, None),
    ("19_ssr_category_archive_mobile_375.png", 375, None),
    ("20_ssr_404_error_page_desktop_1200.png", 1200, None),
    ("21_ssr_404_error_page_mobile_375.png", 375, None),
]

CAD_TOKENS = {
    "Obsidian Chassis (#0D0F12)": (13, 15, 18),
    "Graphite Surface (#15181E)": (21, 24, 30),
    "Hairline Rule (#242933)": (36, 41, 51),
    "Cobalt Signal (#2563EB)": (37, 99, 235),
    "Amber Token (#F59E0B)": (245, 158, 11),
}

def analyze_screenshots():
    print(f"=== CHALLENGER AUDIT: Inspecting {len(EXPECTED_FILES)} Screenshots ===")
    all_passed = True
    results = []

    for filename, exp_w, exp_h in EXPECTED_FILES:
        filepath = os.path.join(SCREENSHOTS_DIR, filename)
        if not os.path.exists(filepath):
            print(f"[FAIL] Missing file: {filename}")
            all_passed = False
            continue

        size_bytes = os.path.getsize(filepath)
        if size_bytes < 5000:
            print(f"[FAIL] Suspiciously small file ({size_bytes} bytes): {filename}")
            all_passed = False
            continue

        try:
            with Image.open(filepath) as img:
                w, h = img.size
                fmt = img.format
                mode = img.mode

                # Format check
                if fmt != "PNG":
                    print(f"[FAIL] Non-PNG format ({fmt}): {filename}")
                    all_passed = False

                # Width check
                if w != exp_w:
                    print(f"[FAIL] Width mismatch for {filename}: expected {exp_w}, got {w}")
                    all_passed = False

                # Height check (if fixed)
                if exp_h is not None and h != exp_h:
                    print(f"[FAIL] Height mismatch for {filename}: expected {exp_h}, got {h}")
                    all_passed = False

                # Stat variance check for blank / placeholder detection
                stat = ImageStat.Stat(img)
                variances = stat.var
                avg_var = sum(variances) / len(variances) if variances else 0
                is_blank = avg_var < 5.0

                if is_blank:
                    print(f"[FAIL] Image appears blank or single-color (avg variance {avg_var:.2f}): {filename}")
                    all_passed = False

                # Color token presence check for builder screenshots (01 to 10)
                tokens_found = {}
                if filename.startswith(("01", "02", "03", "04", "05", "06", "07", "08", "09", "10")):
                    colors = img.convert("RGB").getcolors(maxcolors=2000000)
                    if colors:
                        palette = [c[1] for c in colors]
                        for token_name, target_rgb in CAD_TOKENS.items():
                            # check if exact or within delta <= 3
                            found = any(
                                abs(r - target_rgb[0]) <= 3 and
                                abs(g - target_rgb[1]) <= 3 and
                                abs(b - target_rgb[2]) <= 3
                                for (r, g, b) in palette
                            )
                            tokens_found[token_name] = found

                results.append({
                    "filename": filename,
                    "dimensions": f"{w}x{h}",
                    "size_kb": f"{size_bytes / 1024:.1f} KB",
                    "variance": f"{avg_var:.1f}",
                    "is_blank": is_blank,
                    "tokens": tokens_found,
                })

                status_str = "PASS" if not is_blank else "FAIL(BLANK)"
                print(f"[{status_str}] {filename:45s} | {w}x{h:<5} | {size_bytes/1024:6.1f} KB | Var: {avg_var:6.1f}")
        except Exception as e:
            print(f"[FAIL] Error reading {filename}: {e}")
            all_passed = False

    print("\n=== CAD Tokens Check on 01_daystudio_builder_desktop_default.png ===")
    r01 = next((r for r in results if r["filename"].startswith("01")), None)
    if r01:
        for t_name, t_found in r01["tokens"].items():
            print(f"  {t_name:32s}: {'FOUND' if t_found else 'NOT DETECTED'}")

    print("\n=== CAD Tokens Check on 04_daystudio_builder_box_model_inspector.png ===")
    r04 = next((r for r in results if r["filename"].startswith("04")), None)
    if r04:
        for t_name, t_found in r04["tokens"].items():
            print(f"  {t_name:32s}: {'FOUND' if t_found else 'NOT DETECTED'}")

    return all_passed

if __name__ == "__main__":
    success = analyze_screenshots()
    sys.exit(0 if success else 1)
