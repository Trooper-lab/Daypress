import os
import struct

screenshot_dir = os.path.abspath("tests/screenshots")
expected_files = [
    ("01_neostudio_builder_desktop_default.png", (1200, 900)),
    ("02_neostudio_builder_navigator_active.png", (1200, 900)),
    ("03_neostudio_builder_theme_templates_drawer.png", (1200, 900)),
    ("04_neostudio_builder_box_model_inspector.png", (1200, 900)),
    ("05_neostudio_builder_xray_mode.png", (1200, 900)),
    ("06_neostudio_builder_responsive_tablet.png", (1200, 900)),
    ("07_neostudio_builder_responsive_mobile.png", (1200, 900)),
    ("08_neostudio_builder_create_template_modal.png", (1200, 900)),
    ("09_neostudio_builder_display_conditions_modal.png", (1200, 900)),
    ("10_neostudio_builder_save_published_state.png", (1200, 900)),
    ("11_ssr_frontpage_desktop_1200.png", (1200, None)),
    ("12_ssr_frontpage_tablet_768.png", (768, None)),
    ("13_ssr_frontpage_mobile_375.png", (375, None)),
    ("14_ssr_single_post_desktop_1200.png", (1200, None)),
    ("15_ssr_single_post_tablet_768.png", (768, None)),
    ("16_ssr_single_post_mobile_375.png", (375, None)),
    ("17_ssr_category_archive_desktop_1200.png", (1200, None)),
    ("18_ssr_category_archive_tablet_768.png", (768, None)),
    ("19_ssr_category_archive_mobile_375.png", (375, None)),
    ("20_ssr_404_error_page_desktop_1200.png", (1200, None)),
    ("21_ssr_404_error_page_mobile_375.png", (375, None)),
]

PNG_MAGIC = b"\x89PNG\r\n\x1a\n"
IEND_MAGIC = b"\x00\x00\x00\x00IEND\xaeB`\x82"

print(f"Auditing {len(expected_files)} screenshots in {screenshot_dir}...\n")
header = f"{'#':<3} {'Filename':<48} {'Size (B)':<10} {'Size (KB)':<10} {'Res (WxH)':<12} {'Magic':<8} {'IEND':<6} {'Dim Check':<10}"
print(header)
print("-" * len(header))

all_passed = True
for idx, (fn, (exp_w, exp_h)) in enumerate(expected_files, 1):
    path = os.path.join(screenshot_dir, fn)
    if not os.path.exists(path):
        print(f"{idx:02d}  {fn:<48} MISSING")
        all_passed = False
        continue

    size = os.path.getsize(path)
    size_kb = size / 1024.0

    with open(path, "rb") as f:
        data = f.read()

    has_magic = data[:8] == PNG_MAGIC
    has_iend = data[-12:] == IEND_MAGIC
    w, h = struct.unpack(">II", data[16:24])

    dim_ok = True
    if exp_w is not None and w != exp_w:
        dim_ok = False
    if exp_h is not None and h != exp_h:
        dim_ok = False

    size_ok = 10000 <= size <= 500000

    row_ok = has_magic and has_iend and dim_ok and size_ok
    if not row_ok:
        all_passed = False

    status_magic = "OK" if has_magic else "FAIL"
    status_iend = "OK" if has_iend else "FAIL"
    status_dim = "PASS" if dim_ok else f"FAIL({exp_w}x{exp_h})"

    print(f"{idx:02d}  {fn:<48} {size:<10} {size_kb:<10.1f} {f'{w}x{h}':<12} {status_magic:<8} {status_iend:<6} {status_dim:<10}")

print("-" * len(header))
print(f"\nFinal Audit Result: {'100% VERIFIED' if all_passed else 'FAILED'}")
if not all_passed:
    exit(1)
