"""
DayStudio Style Inspector — full functional verification (Playwright, headless Chromium).

Boots an isolated Daypress instance, seeds a "style lab" page and drives EVERY style control in the
inspector, asserting the computed style on the canvas element after each change. Then saves and
verifies the public SSR page renders identically (desktop, tablet media query, :hover rules).
"""
import json, os, sqlite3, subprocess, sys, time, urllib.request

from playwright.sync_api import sync_playwright

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORT = 8097
BASE = f"http://127.0.0.1:{PORT}"
DB = os.path.join(REPO, "data", "test_style_lab.sqlite")
SHOTS = os.path.join(REPO, "tests", "screenshots", "style_inspector")
os.makedirs(SHOTS, exist_ok=True)

LAYOUT = {
    "version": 1,
    "root": {
        "id": "root", "type": "Root", "styles": {},
        "children": [
            {
                "id": "sec_a", "type": "Section", "styles": {"background_color": "#ffffff", "padding_top": "40px", "padding_bottom": "40px"},
                "children": [{
                    "id": "cont_a", "type": "Container", "styles": {},
                    "children": [
                        {"id": "h_a", "type": "Heading", "data": {"text": "Style Lab Heading", "level": 2}, "styles": {}},
                        {"id": "btn_a", "type": "Button", "data": {"text": "Click me", "url": "#"}, "styles": {"background_color": "#111111", "text_color": "#ffffff", "padding_top": "10px", "padding_bottom": "10px", "padding_left": "16px", "padding_right": "16px"}},
                        {"id": "img_a", "type": "Image", "data": {"src": "/static/daypress-icon.svg", "alt": "icon"}, "styles": {"width": "120px"}},
                        # Imported (nested) styles: builder edits must not be shadowed by these
                        {"id": "txt_nested", "type": "Text", "data": {"content_html": "<p>Nested import text</p>"},
                         "styles": {"padding": {"top": "10px"}, "typography": {"color": "#123456"}, "z_index": 3}},
                    ],
                }],
            },
            {"id": "custom_a", "type": "CustomHtml",
             "data": {"raw": "<section class=\"section white\" style=\"padding: 12px;\"><div class=\"container\"><h2>ACF Section</h2></div></section>"},
             "styles": {}},
        ],
    },
}

results = []
def check(name, cond, detail=""):
    results.append((name, bool(cond), detail))
    print(("  [PASS] " if cond else "  [FAIL] ") + name + (f"  ({detail})" if detail and not cond else ""))


class Server:
    def __init__(self):
        self.proc = None
    def start(self):
        for ext in ["", "-shm", "-wal", "-journal"]:
            if os.path.exists(DB + ext):
                os.remove(DB + ext)
        exe = os.path.join(REPO, "target", "debug", "daypress.exe")
        self.proc = subprocess.Popen([exe, "--port", str(PORT), "--db", DB], cwd=REPO,
                                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(60):
            time.sleep(0.3)
            try:
                with urllib.request.urlopen(f"{BASE}/wp-json", timeout=1) as r:
                    if r.status == 200:
                        return
            except Exception:
                pass
        raise RuntimeError("server did not start")
    def stop(self):
        if self.proc:
            self.proc.terminate()
            try: self.proc.wait(timeout=3)
            except Exception: self.proc.kill()


def seed():
    s = json.dumps(LAYOUT)
    con = sqlite3.connect(DB)
    con.execute("""INSERT OR REPLACE INTO posts (ID, post_author, post_date, post_date_gmt, post_content, post_title,
        post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, layout_json)
        VALUES (1, 1, datetime('now'), datetime('now'), '<p>lab</p>', 'Style Lab', '', 'publish', 'style-lab',
        datetime('now'), datetime('now'), 'http://x/style-lab/', 'page', ?)""", (s,))
    con.execute("INSERT OR REPLACE INTO postmeta (post_id, meta_key, meta_value) VALUES (1, '_daypress_layout', ?)", (s,))
    con.commit(); con.close()


def run():
    server = Server(); server.start()
    errors = []
    try:
        seed()
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            ctx = browser.new_context(viewport={"width": 1440, "height": 1000})
            ctx.add_cookies([{"name": "daypress_admin", "value": "1", "domain": "127.0.0.1", "path": "/"}])
            page = ctx.new_page()
            page.on("console", lambda m: errors.append(m.text) if m.type == "error" and "favicon" not in m.text and "fonts.g" not in m.text else None)
            page.on("pageerror", lambda e: errors.append("PAGEERROR " + str(e)))
            page.goto(f"{BASE}/studio?post_id=1", wait_until="networkidle")
            page.wait_for_selector('#daystudio-canvas [data-node-id="h_a"]')

            INS = "#daystudio-inspector"
            def cs(node_id, prop):
                return page.evaluate("([id, p]) => getComputedStyle(document.querySelector(`#daystudio-canvas [data-node-id='${id}']`))[p]", [node_id, prop])
            def select(node_id):
                page.click(f'#daystudio-canvas [data-node-id="{node_id}"]')
                page.wait_for_selector(f'{INS} .target-id:has-text("#{node_id}")')
            def open_panel(pid):
                panel = page.locator(f'{INS} .cad-panel[data-panel-id="{pid}"]')
                if "open" not in (panel.get_attribute("class") or ""):
                    panel.locator(".cad-panel-header").click()
            def typ(prop, value):
                loc = page.locator(f'{INS} input.ns-input[data-prop="{prop}"]').first
                loc.fill(value); loc.press("Enter")
            def seg(prop, val):
                page.locator(f'{INS} .ns-seg[data-prop="{prop}"] .seg-btn[data-val="{val}"]').first.click()
            def sel(prop, val):
                page.locator(f'{INS} select.ns-select[data-prop="{prop}"]').select_option(val)
            def chip(attr, val):
                page.locator(f'{INS} [{attr}="{val}"]').first.click()

            print("\n[1] Typography")
            select("h_a")
            typ("font_size", "40"); check("font-size 40 -> 40px", cs("h_a", "fontSize") == "40px", cs("h_a", "fontSize"))
            typ("text_color", "ff0000"); check("color hex w/o # -> red", cs("h_a", "color") == "rgb(255, 0, 0)", cs("h_a", "color"))
            sel("font_weight", "800"); check("font-weight 800", cs("h_a", "fontWeight") == "800")
            seg("text_align", "center"); check("text-align center", cs("h_a", "textAlign") == "center")
            seg("text_transform", "uppercase"); check("text-transform uppercase", cs("h_a", "textTransform") == "uppercase")
            seg("font_style", "italic"); check("font-style italic", cs("h_a", "fontStyle") == "italic")
            seg("text_decoration", "underline"); check("text-decoration underline", "underline" in cs("h_a", "textDecorationLine"))
            typ("line_height", "1.8"); check("line-height 1.8", cs("h_a", "lineHeight") == "72px", cs("h_a", "lineHeight"))
            typ("letter_spacing", "2"); check("letter-spacing 2px", cs("h_a", "letterSpacing") == "2px")
            typ("font_family", "Poppins"); check("font-family Poppins", "Poppins" in cs("h_a", "fontFamily"), cs("h_a", "fontFamily"))
            typ("text_shadow", "0 2px 4px rgba(0,0,0,0.5)"); check("text-shadow applied", cs("h_a", "textShadow") != "none")
            lbl = page.locator(f'{INS} .cad-label[data-label-for="font_size"]')
            check("label shows has-override", "has-override" in (lbl.get_attribute("class") or ""))

            print("\n[2] Backgrounds")
            typ("background_color", "#00ff00"); check("bg color green", cs("h_a", "backgroundColor") == "rgb(0, 255, 0)")
            chip("data-grad", "linear-gradient(135deg, #2563eb 0%, #7c3aed 100%)"); check("gradient chip", "linear-gradient" in cs("h_a", "backgroundImage"))
            typ("background_image", "/static/daypress-icon.svg"); bi = cs("h_a", "backgroundImage")
            check("bg image + gradient layered", "url(" in bi and "gradient" in bi, bi)
            sel("background_size", "contain"); check("bg size contain", cs("h_a", "backgroundSize").startswith("contain"))
            sel("background_repeat", "no-repeat"); check("bg repeat none", cs("h_a", "backgroundRepeat").startswith("no-repeat"))
            sel("background_position", "top center"); check("bg position", cs("h_a", "backgroundPosition").startswith("50% 0%"), cs("h_a", "backgroundPosition"))

            print("\n[3] Borders")
            typ("border_width", "3"); check("border-width 3 -> auto solid", cs("h_a", "borderTopWidth") == "3px" and cs("h_a", "borderTopStyle") == "solid", cs("h_a", "borderTopStyle"))
            typ("border_color", "#0000ff"); check("border color", cs("h_a", "borderTopColor") == "rgb(0, 0, 255)")
            seg("border_style", "dashed"); check("border style dashed", cs("h_a", "borderTopStyle") == "dashed")
            chip("data-radius", "8px"); check("radius chip 8px", cs("h_a", "borderTopLeftRadius") == "8px")

            print("\n[4] Effects & custom CSS")
            open_panel("effects")
            r = page.locator(f'{INS} input[data-range="opacity"]')
            r.evaluate("el => { el.value = 0.5; el.dispatchEvent(new Event('input', {bubbles:true})); el.dispatchEvent(new Event('change', {bubbles:true})); }")
            check("opacity slider 0.5", cs("h_a", "opacity") == "0.5", cs("h_a", "opacity"))
            chip("data-shadow", "0 12px 32px rgba(0,0,0,0.18)"); check("shadow chip", cs("h_a", "boxShadow") != "none")
            chip("data-transform", "scale(1.05)"); check("transform chip", cs("h_a", "transform").startswith("matrix"))
            typ("transition", "all 0.3s ease"); check("transition", "0.3s" in cs("h_a", "transitionDuration"))
            page.locator(f'{INS} .cad-label[data-label-for="transition"]').click()  # reset so later computed reads are not mid-animation
            check("transition reset via label", cs("h_a", "transitionDuration") == "0s")
            typ("filter", "blur(1px)"); check("filter", "blur" in cs("h_a", "filter"))
            sel("cursor", "pointer"); check("cursor pointer", cs("h_a", "cursor") == "pointer")
            open_panel("customcss")
            ta = page.locator(f'{INS} textarea[data-prop="custom_css"]')
            ta.fill("mix-blend-mode: multiply;"); ta.evaluate("el => el.dispatchEvent(new Event('change'))")
            check("custom CSS applied", cs("h_a", "mixBlendMode") == "multiply")

            print("\n[5] Size & position")
            typ("width", "500"); check("width 500px", cs("h_a", "width") == "500px", cs("h_a", "width"))
            typ("max_width", "90%"); check("max-width 90%", cs("h_a", "maxWidth") == "90%")
            typ("min_height", "80"); check("min-height", cs("h_a", "minHeight") == "80px")
            open_panel("position")
            sel("position", "relative"); typ("top", "5"); check("position relative + top 5px", cs("h_a", "position") == "relative" and cs("h_a", "top") == "5px")
            typ("z_index", "7"); check("z-index 7", cs("h_a", "zIndex") == "7")
            sel("overflow", "hidden"); check("overflow hidden", cs("h_a", "overflow") == "hidden")
            inp = page.locator(f'{INS} input.ns-input[data-prop="width"]')
            inp.click(); inp.press("ArrowUp"); inp.press("Shift+ArrowUp")
            check("ArrowUp / Shift+ArrowUp increments width (+11)", cs("h_a", "width") == "511px", cs("h_a", "width"))

            print("\n[6] Layout (flex & grid on container)")
            select("cont_a")
            seg("display", "flex"); check("display flex", cs("cont_a", "display") == "flex")
            check("flex-only rows visible after Flex", page.locator(f'{INS} .ns-seg[data-prop="flex_direction"]').is_visible())
            seg("flex_direction", "column"); check("direction column", cs("cont_a", "flexDirection") == "column")
            seg("align_items", "center"); check("align-items center", cs("cont_a", "alignItems") == "center")
            sel("justify_content", "space-between"); check("justify space-between", cs("cont_a", "justifyContent") == "space-between")
            seg("flex_wrap", "wrap"); check("flex-wrap", cs("cont_a", "flexWrap") == "wrap")
            typ("gap", "24"); check("gap 24px", cs("cont_a", "rowGap") == "24px")
            seg("display", "grid"); check("display grid", cs("cont_a", "display") == "grid")
            chip("data-grid-cols", "3"); gtc = cs("cont_a", "gridTemplateColumns")
            check("grid 3 cols", len(gtc.split()) == 3, gtc)
            check("grid-only rows visible, flex rows hidden", page.locator(f'{INS} input[data-prop="grid_template_columns"]').is_visible() and not page.locator(f'{INS} .ns-seg[data-prop="flex_direction"]').is_visible())

            print("\n[7] Box model")
            select("btn_a")
            fac = page.locator(f'{INS} .day-box-facet[data-prop="padding_top"]')
            fac.dblclick(); page.keyboard.type("33"); page.keyboard.press("Enter")
            check("padding-top typed 33px", cs("btn_a", "paddingTop") == "33px")
            box = fac.bounding_box()
            page.mouse.move(box["x"] + box["width"] / 2, box["y"] + box["height"] / 2); page.mouse.down()
            page.mouse.move(box["x"] + box["width"] / 2 + 10, box["y"] + box["height"] / 2, steps=5); page.mouse.up()
            check("padding scrub +10 -> 43px", cs("btn_a", "paddingTop") == "43px", cs("btn_a", "paddingTop"))
            page.click(f'{INS} #btn-toggle-box-link')
            mf = page.locator(f'{INS} .day-box-facet[data-prop="margin_left"]')
            mf.dblclick(); page.keyboard.type("2rem"); page.keyboard.press("Enter")
            check("linked margin 2rem on all sides", cs("btn_a", "marginTop") == "32px" and cs("btn_a", "marginRight") == "32px", cs("btn_a", "marginTop"))
            page.click(f'{INS} #btn-toggle-box-link')
            fac.click(modifiers=["Alt"])
            check("Alt+click resets padding-top to 0", cs("btn_a", "paddingTop") == "0px", cs("btn_a", "paddingTop"))

            print("\n[8] Reset via label + undo")
            select("h_a")
            page.locator(f'{INS} .cad-label[data-label-for="letter_spacing"]').click()
            check("label click resets letter-spacing", cs("h_a", "letterSpacing") in ("normal", "0px"), cs("h_a", "letterSpacing"))
            page.evaluate("window.undo()")
            check("undo restores letter-spacing 2px", cs("h_a", "letterSpacing") == "2px", cs("h_a", "letterSpacing"))

            print("\n[9] Breakpoint cascade")
            page.evaluate("window.setBreakpoint('tablet')"); select("h_a")
            fs_lbl = page.locator(f'{INS} .cad-label[data-label-for="font_size"]')
            check("tablet: font-size label shows inherited", "is-inherited" in (fs_lbl.get_attribute("class") or ""))
            check("tablet: placeholder shows desktop value", page.locator(f'{INS} input.ns-input[data-prop="font_size"]').get_attribute("placeholder") == "40px")
            typ("font_size", "22"); check("tablet override 22px on canvas", cs("h_a", "fontSize") == "22px")
            page.evaluate("window.setBreakpoint('mobile')")
            check("mobile inherits tablet 22px", cs("h_a", "fontSize") == "22px")
            page.evaluate("window.setBreakpoint('desktop')")
            check("desktop keeps 40px", cs("h_a", "fontSize") == "40px")

            print("\n[10] Pseudo-states")
            page.click(f'{INS} .state-btn[data-state="hover"]')
            typ("background_color", "#ff00ff")
            check("hover preview forced on canvas", cs("h_a", "backgroundColor") == "rgb(255, 0, 255)")
            check("hover edit didn't touch base", page.evaluate("window.findNodeById('h_a').styles.background_color") == "#00ff00")
            page.click(f'{INS} .state-btn[data-state="focus"]')
            typ("text_color", "#00aaaa"); check("focus preview", cs("h_a", "color") == "rgb(0, 170, 170)")
            page.click(f'{INS} .state-btn[data-state="default"]')
            check("default state restores base bg", cs("h_a", "backgroundColor") == "rgb(0, 255, 0)")

            print("\n[11] Classes, visibility, CustomHtml, nested imports")
            page.click(f'{INS} #btn-add-class-tag'); page.keyboard.type("lab-class"); page.keyboard.press("Enter")
            check("class added to canvas element", page.evaluate("document.querySelector('[data-node-id=h_a]').classList.contains('lab-class')"))
            select("custom_a")
            typ("background_color", "#222222")
            check("CustomHtml root gets bg", cs("custom_a", "backgroundColor") == "rgb(34, 34, 34)")
            check("CustomHtml keeps own inline padding", cs("custom_a", "paddingTop") == "12px")
            select("txt_nested")
            check("nested import padding shows (10px)", cs("txt_nested", "paddingTop") == "10px")
            fac = page.locator(f'{INS} .day-box-facet[data-prop="padding_top"]')
            fac.dblclick(); page.keyboard.type("50"); page.keyboard.press("Enter")
            check("edit beats nested import padding", cs("txt_nested", "paddingTop") == "50px")
            typ("text_color", "#654321"); check("edit beats nested typography color", cs("txt_nested", "color") == "rgb(101, 67, 33)")
            page.screenshot(path=os.path.join(SHOTS, "studio_after_edits.png"))

            print("\n[12] Save + public SSR parity")
            page.evaluate("window.StudioApi.saveLayout()")
            page.wait_for_function("!window.DayStudioState.isSaving && !window.DayStudioState.isDirty", timeout=10000)
            pub = ctx.new_page()
            pub.goto(f"{BASE}/style-lab/", wait_until="networkidle")
            def pcs(sel_, prop):
                return pub.evaluate("([s, p]) => { const e = document.querySelector(s); return e ? getComputedStyle(e)[p] : null; }", [sel_, prop])
            check("SSR heading font-size 40px", pcs('[id="h_a"]', "fontSize") == "40px", pcs('[id="h_a"]', "fontSize"))
            check("SSR heading color red", pcs('[id="h_a"]', "color") == "rgb(255, 0, 0)")
            check("SSR border dashed 3px", pcs('[id="h_a"]', "borderTopStyle") == "dashed" and pcs('[id="h_a"]', "borderTopWidth") == "3px")
            check("SSR opacity / mix-blend (custom css)", pcs('[id="h_a"]', "opacity") == "0.5" and pcs('[id="h_a"]', "mixBlendMode") == "multiply")
            check("SSR class applied", pub.evaluate("document.getElementById('h_a').classList.contains('lab-class')"))
            check("SSR Poppins @import emitted", "fonts.googleapis.com/css2?family=Poppins" in pub.content())
            pub.hover('[id="h_a"]')
            check("SSR :hover bg beats inline base", pcs('[id="h_a"]', "backgroundColor") == "rgb(255, 0, 255)", pcs('[id="h_a"]', "backgroundColor"))
            check("SSR container grid 3 cols", len((pcs('[id="cont_a"]', "gridTemplateColumns") or "").split()) == 3)
            check("SSR button has id + margins 2rem", float(pcs('[id="btn_a"]', "marginTop")[:-2]) == 2 * float(pub.evaluate("getComputedStyle(document.documentElement).fontSize")[:-2]), str(pcs('[id="btn_a"]', "marginTop")) + " " + pub.evaluate("(document.getElementById('btn_a')||{outerHTML:'MISSING'}).outerHTML.slice(0,300)"))
            check("SSR CustomHtml root injected (no wrapper)", pub.evaluate("(() => { const e = document.getElementById('custom_a'); return !!e && e.tagName === 'SECTION' && e.classList.contains('section'); })()"))
            check("SSR CustomHtml bg", pcs('[id="custom_a"]', "backgroundColor") == "rgb(34, 34, 34)")
            check("SSR nested-import edit wins", pcs('[id="txt_nested"]', "paddingTop") == "50px" and pcs('[id="txt_nested"]', "color") == "rgb(101, 67, 33)")
            pub.screenshot(path=os.path.join(SHOTS, "ssr_desktop.png"), full_page=True)
            pub.set_viewport_size({"width": 800, "height": 1000})
            pub.mouse.move(0, 0)
            check("SSR tablet media query 22px", pcs('[id="h_a"]', "fontSize") == "22px", pcs('[id="h_a"]', "fontSize"))
            pub.screenshot(path=os.path.join(SHOTS, "ssr_tablet.png"), full_page=True)

            # Reload studio: persisted values come back in the inspector
            page.reload(wait_until="networkidle"); page.wait_for_selector('#daystudio-canvas [data-node-id="h_a"]')
            select("h_a")
            check("reload: inspector shows saved font-size", page.locator(f'{INS} input.ns-input[data-prop="font_size"]').input_value() == "40px")
            check("reload: canvas keeps styles", cs("h_a", "fontSize") == "40px" and cs("h_a", "opacity") == "0.5")
            browser.close()
    finally:
        server.stop()

    check("no JS console errors", not errors, "; ".join(errors[:5]))
    failed = [r for r in results if not r[1]]
    print(f"\n==== {len(results) - len(failed)}/{len(results)} checks passed ====")
    for f in failed:
        print("  FAILED:", f[0], f[2])
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    run()
