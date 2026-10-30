//! DayStudio Style Engine: style normalization, breakpoint cascade resolution, canvas style
//! application and pseudo-state (`:hover` / `:focus` / `:active`) stylesheet (`scripts/style_engine.rs`).
//!
//! Single source of truth for turning a node's `styles` object into CSS on the artboard. The
//! property map mirrors `daypress_ast::StyleProperties::css_declarations` so the editor canvas and
//! public SSR output stay identical.

pub const JS_STYLE_ENGINE: &str = r###"
(function() {
    // Flat Visual Builder key -> CSS property
    const CSS_MAP = {
        padding_top: "padding-top", padding_right: "padding-right", padding_bottom: "padding-bottom", padding_left: "padding-left",
        margin_top: "margin-top", margin_right: "margin-right", margin_bottom: "margin-bottom", margin_left: "margin-left",
        text_color: "color", font_size: "font-size", font_weight: "font-weight", line_height: "line-height",
        letter_spacing: "letter-spacing", text_align: "text-align", text_transform: "text-transform",
        font_style: "font-style", text_decoration: "text-decoration", text_shadow: "text-shadow",
        background_color: "background-color", background_position: "background-position", background_size: "background-size",
        background_repeat: "background-repeat", background_attachment: "background-attachment",
        border_radius: "border-radius", border_width: "border-width", border_style: "border-style", border_color: "border-color",
        border_top: "border-top", border_right: "border-right", border_bottom: "border-bottom", border_left: "border-left",
        box_shadow: "box-shadow",
        display: "display", flex_direction: "flex-direction", flex_wrap: "flex-wrap", justify_content: "justify-content",
        align_items: "align-items", align_self: "align-self", gap: "gap", flex: "flex",
        grid_template_columns: "grid-template-columns", grid_template_rows: "grid-template-rows", grid_gap: "grid-gap",
        grid_column: "grid-column", grid_row: "grid-row", grid_auto_flow: "grid-auto-flow", justify_items: "justify-items",
        position: "position", top: "top", right: "right", bottom: "bottom", left: "left", z_index: "z-index", overflow: "overflow",
        width: "width", min_width: "min-width", max_width: "max-width", height: "height", min_height: "min-height",
        max_height: "max-height", aspect_ratio: "aspect-ratio", object_fit: "object-fit",
        opacity: "opacity", transform: "transform", transition: "transition", filter: "filter",
        backdrop_filter: "backdrop-filter", cursor: "cursor"
    };
    // Composite keys handled specially (not in CSS_MAP)
    const SPECIAL_KEYS = ["font_family", "background_image", "background_gradient", "custom_css"];
    const LAYER_KEYS = ["tablet", "mobile", "hover", "focus", "active"];
    const STATE_LAYERS = [["hover", ":hover"], ["focus", ":focus"], ["active", ":active"]];

    const GOOGLE_FONTS = ["Inter", "Roboto", "Open Sans", "Lato", "Montserrat", "Poppins", "Raleway", "Nunito", "DM Sans",
        "Plus Jakarta Sans", "Space Grotesk", "Work Sans", "Manrope", "Outfit", "Oswald", "Playfair Display", "Merriweather",
        "Lora", "DM Serif Display", "Libre Baskerville", "JetBrains Mono", "Fira Code"];

    window.DayStyle = { CSS_MAP, SPECIAL_KEYS, LAYER_KEYS, GOOGLE_FONTS };

    function isEmpty(v) { return v === undefined || v === null || (typeof v === "string" && v.trim() === ""); }

    function cssPropFor(key) {
        if (CSS_MAP[key]) return CSS_MAP[key];
        if (key === "font_family") return "font-family";
        if (key === "background_image" || key === "background_gradient") return "background-image";
        return null;
    }
    window.DayStyle.cssPropFor = cssPropFor;

    // -------------------------------------------------------------
    // NORMALIZATION (nested import objects -> flat builder keys)
    // -------------------------------------------------------------
    function parseQuad(v) {
        if (v && typeof v === "object") return { top: v.top, right: v.right, bottom: v.bottom, left: v.left };
        if (typeof v !== "string" && typeof v !== "number") return null;
        const p = String(v).trim().split(/\s+/).filter(Boolean);
        if (!p.length) return null;
        const [t, r = t, b = t, l = r] = p;
        return { top: t, right: r, bottom: b, left: l };
    }

    function adopt(s, flatKey, val) {
        if (isEmpty(s[flatKey]) && !isEmpty(val)) s[flatKey] = String(val);
    }

    function normalizeStyleObject(s) {
        if (!s || typeof s !== "object") return {};
        ["padding", "margin"].forEach(group => {
            if (s[group] === undefined) return;
            const q = parseQuad(s[group]);
            if (q) ["top", "right", "bottom", "left"].forEach(side => adopt(s, group + "_" + side, q[side]));
            delete s[group];
        });
        if (s.typography && typeof s.typography === "object") {
            const t = s.typography;
            adopt(s, "text_color", t.color); adopt(s, "font_size", t.font_size); adopt(s, "font_weight", t.font_weight);
            adopt(s, "text_align", t.text_align); adopt(s, "line_height", t.line_height);
            delete s.typography;
        }
        if (s.background && typeof s.background === "object") {
            const b = s.background;
            adopt(s, "background_color", b.color); adopt(s, "background_image", b.image_url);
            adopt(s, "background_position", b.position); adopt(s, "background_size", b.size);
            delete s.background;
        }
        if (s.border !== undefined) {
            const b = s.border;
            if (b && typeof b === "object") {
                adopt(s, "border_radius", b.radius); adopt(s, "border_width", b.width);
                adopt(s, "border_color", b.color); adopt(s, "border_style", b.style);
            } else if (typeof b === "string") {
                const p = b.trim().split(/\s+/);
                if (p.length >= 3) { adopt(s, "border_width", p[0]); adopt(s, "border_style", p[1]); adopt(s, "border_color", p.slice(2).join(" ")); }
            }
            delete s.border;
        }
        if (!isEmpty(s.custom_classes)) {
            const merged = new Set(String(s.css_classes || "").split(/\s+/).filter(Boolean));
            String(s.custom_classes).split(/\s+/).filter(Boolean).forEach(c => merged.add(c));
            s.css_classes = Array.from(merged).join(" ");
        }
        delete s.custom_classes;
        Object.keys(s).forEach(k => {
            if (LAYER_KEYS.includes(k)) {
                if (s[k] && typeof s[k] === "object") s[k] = normalizeStyleObject(s[k]); else delete s[k];
            } else if (typeof s[k] === "number" && !k.startsWith("hide_")) {
                s[k] = String(s[k]);
            }
        });
        return s;
    }
    window.DayStyle.normalizeStyleObject = normalizeStyleObject;

    function normalizeTreeStyles(node) {
        if (!node) return;
        if (node.styles) node.styles = normalizeStyleObject(node.styles);
        (node.children || []).forEach(normalizeTreeStyles);
    }
    window.DayStyle.normalizeTreeStyles = normalizeTreeStyles;

    // -------------------------------------------------------------
    // CASCADE RESOLUTION
    // -------------------------------------------------------------
    function baseOf(styles) {
        const out = {};
        Object.keys(styles || {}).forEach(k => { if (!LAYER_KEYS.includes(k)) out[k] = styles[k]; });
        return out;
    }

    function resolveStyles(node, bp) {
        const s = (node && node.styles) || {};
        const r = baseOf(s);
        if ((bp === "tablet" || bp === "mobile") && s.tablet) Object.assign(r, baseOf(s.tablet));
        if (bp === "mobile" && s.mobile) Object.assign(r, baseOf(s.mobile));
        return r;
    }
    window.DayStyle.resolveStyles = resolveStyles;

    // -------------------------------------------------------------
    // DECLARATION COMPILER (mirrors StyleProperties::css_declarations)
    // -------------------------------------------------------------
    function cssImage(v) {
        const t = String(v).trim();
        if (/^(url\(|var\(|image-set\()/i.test(t) || /gradient\(/i.test(t) || t === "none") return t;
        return `url("${t.replace(/"/g, "%22")}")`;
    }

    function fontStack(v) {
        const t = String(v).trim();
        if (t.includes(",") || t.startsWith("var(") || t.startsWith("'") || t.startsWith('"')) return t;
        return `'${t}', sans-serif`;
    }

    function splitDecls(block) {
        const out = []; let cur = "", depth = 0, quote = null;
        for (const ch of String(block)) {
            if (quote) { if (ch === quote) quote = null; cur += ch; continue; }
            if (ch === '"' || ch === "'") { quote = ch; cur += ch; continue; }
            if (ch === "(") depth++;
            if (ch === ")") depth--;
            if (ch === ";" && depth <= 0) { if (cur.trim()) out.push(cur.trim()); cur = ""; continue; }
            cur += ch;
        }
        if (cur.trim()) out.push(cur.trim());
        return out;
    }

    function declarations(s) {
        const out = [];
        if (!s) return out;
        Object.keys(CSS_MAP).forEach(k => {
            if (!isEmpty(s[k])) out.push([CSS_MAP[k], String(s[k]).trim()]);
        });
        if (!isEmpty(s.font_family)) out.push(["font-family", fontStack(s.font_family)]);
        const img = !isEmpty(s.background_image) ? cssImage(s.background_image) : null;
        const grad = !isEmpty(s.background_gradient) ? String(s.background_gradient).trim() : null;
        if (img || grad) out.push(["background-image", [grad, img].filter(Boolean).join(", ")]);
        // Width without a style renders nothing in CSS (initial border-style is none)
        if (!isEmpty(s.border_width) && isEmpty(s.border_style) && parseFloat(s.border_width) !== 0) {
            out.push(["border-style", "solid"]);
        }
        if (!isEmpty(s.custom_css)) {
            splitDecls(s.custom_css).forEach(d => {
                const i = d.indexOf(":");
                if (i > 0) out.push([d.slice(0, i).trim().toLowerCase(), d.slice(i + 1).trim()]);
            });
        }
        return out;
    }
    window.DayStyle.declarations = declarations;

    // Structural declarations public SSR puts in front of a layout node's own styles
    // (mirrors daypress-ast render/html/layout.rs), so Grid/Flex/Column lay out the same on the canvas.
    function layoutDeclarations(node, own) {
        const d = (node && node.data) || {};
        const count = (v, dflt) => { const n = parseInt(v, 10); return Number.isFinite(n) && n >= 0 ? n : dflt; };
        if (node.type === "Grid") {
            return [["display", "grid"], ["grid-template-columns", `repeat(${count(d.columns, 3)}, minmax(0, 1fr))`],
                ["gap", `${count(d.gap, 0)}px`]];
        }
        if (node.type === "Flex") {
            const out = [["display", "flex"], ["flex-direction", d.direction || "row"], ["justify-content", d.justify || "flex-start"],
                ["align-items", d.align || "stretch"], ["flex-wrap", d.wrap ? "wrap" : "nowrap"]];
            if (!isEmpty(d.basis)) out.push(["flex", `1 1 ${d.basis}`], ["min-width", "0"], ["gap", "16px"]);
            return out;
        }
        if (node.type === "Column" && !own.some(([p]) => p === "flex" || p === "flex-basis")) return [["flex", "1"]];
        if ((node.type === "Section" || node.type === "Container") && Array.isArray(node.children)) {
            const colCount = node.children.filter(c => c && c.type === "Column").length;
            if (colCount > 1 && !own.some(([p]) => p === "display")) {
                const out = [["display", "flex"]];
                if (!own.some(([p]) => p === "flex-direction")) out.push(["flex-direction", "row"]);
                if (!own.some(([p]) => p === "gap")) out.push(["gap", "24px"]);
                return out;
            }
        }
        return [];
    }
    window.DayStyle.layoutDeclarations = layoutDeclarations;

    function setDecl(el, prop, val) {
        const m = /\s*!important\s*$/i.exec(val);
        el.style.setProperty(prop, m ? val.slice(0, m.index) : val, m ? "important" : "");
    }

    // -------------------------------------------------------------
    // GOOGLE FONT LOADER (canvas parity with SSR @import)
    // -------------------------------------------------------------
    const loadedFonts = new Set();
    function ensureFontLoaded(family) {
        if (isEmpty(family)) return;
        const first = String(family).split(",")[0].trim().replace(/^['"]|['"]$/g, "");
        const known = GOOGLE_FONTS.find(f => f.toLowerCase() === first.toLowerCase());
        if (!known || loadedFonts.has(known)) return;
        loadedFonts.add(known);
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = `https://fonts.googleapis.com/css2?family=${known.replace(/ /g, "+")}:wght@300;400;500;600;700;800&display=swap`;
        document.head.appendChild(link);
    }
    window.DayStyle.ensureFontLoaded = ensureFontLoaded;

    // -------------------------------------------------------------
    // PSEUDO-STATE STYLESHEET (:hover / :focus / :active + forced preview)
    // -------------------------------------------------------------
    const stateRules = new Map();
    function flushStateRules() {
        let tag = document.getElementById("daystudio-state-css");
        if (!tag) {
            tag = document.createElement("style");
            tag.id = "daystudio-state-css";
            document.head.appendChild(tag);
        }
        tag.textContent = Array.from(stateRules.values()).join("\n");
    }
    function updateStateRules(node) {
        const s = node.styles || {};
        const sel = `[data-node-id="${String(node.id).replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"]`;
        let css = "";
        STATE_LAYERS.forEach(([layer, pseudo]) => {
            if (!s[layer]) return;
            const decls = declarations(s[layer]);
            if (!decls.length) return;
            if (s[layer].font_family) ensureFontLoaded(s[layer].font_family);
            const body = decls.map(([p, v]) => `${p}: ${v.replace(/\s*!important\s*$/i, "")} !important;`).join(" ");
            css += `${sel}${pseudo}, ${sel}.daystudio-force-${layer} { ${body} }\n`;
        });
        // Flush synchronously, but only when this node's rules actually changed
        const prev = stateRules.get(node.id) || "";
        if (css === prev) return;
        if (css) stateRules.set(node.id, css); else stateRules.delete(node.id);
        flushStateRules();
    }
    window.resetStateRules = function() { stateRules.clear(); flushStateRules(); };

    function applyForcedState() {
        const state = window.DayStudioState;
        document.querySelectorAll(".daystudio-force-hover, .daystudio-force-focus, .daystudio-force-active")
            .forEach(el => el.classList.remove("daystudio-force-hover", "daystudio-force-focus", "daystudio-force-active"));
        if (!state.activeElementId || state.pseudoState === "default") return;
        const el = document.querySelector(`#daystudio-canvas [data-node-id="${state.activeElementId}"]`);
        if (el) el.classList.add("daystudio-force-" + state.pseudoState);
    }
    window.applyForcedState = applyForcedState;

    // -------------------------------------------------------------
    // CANVAS APPLICATION
    // -------------------------------------------------------------
    function applyNodeStyles(el, node) {
        if (!el || !node) return;
        const state = window.DayStudioState;
        const bp = state.currentBreakpoint;

        // Restore the element's own markup styles (e.g. CustomHtml inline styles) before re-applying,
        // so removed/reset properties really disappear from the canvas.
        if (el.dataset.daystudioBaseStyle === undefined) el.dataset.daystudioBaseStyle = el.getAttribute("style") || "";
        if (el.dataset.daystudioBaseStyle) el.setAttribute("style", el.dataset.daystudioBaseStyle);
        else el.removeAttribute("style");

        const resolved = resolveStyles(node, bp);
        const own = declarations(resolved);
        layoutDeclarations(node, own).forEach(([p, v]) => setDecl(el, p, v));
        own.forEach(([p, v]) => setDecl(el, p, v));
        if (resolved.font_family) ensureFontLoaded(resolved.font_family);

        // Image widgets: object-fit / sizing target the <img> itself
        if (node.type === "Image" && el.tagName !== "IMG") {
            const img = el.querySelector("img");
            if (img) {
                img.style.objectFit = resolved.object_fit || "";
                img.style.width = resolved.object_fit ? "100%" : "";
                img.style.height = resolved.object_fit ? "100%" : "";
            }
        }

        // Custom classes (tracked so removed classes are cleaned up)
        const prev = (el.dataset.daystudioClasses || "").split(/\s+/).filter(Boolean);
        prev.forEach(c => el.classList.remove(c));
        const next = String((node.styles && node.styles.css_classes) || "").split(/\s+/).filter(Boolean);
        next.forEach(c => el.classList.add(c));
        el.dataset.daystudioClasses = next.join(" ");

        // Responsive visibility: keep selectable in the editor, but show as hidden for this breakpoint
        const hidden = !!(node.styles && node.styles["hide_" + bp]);
        el.classList.toggle("daystudio-hidden-bp", hidden);

        updateStateRules(node);
    }
    window.applyNodeStyles = applyNodeStyles;

    function refreshAllNodeStyles() {
        document.querySelectorAll("#daystudio-canvas [data-node-id]").forEach(el => {
            const node = window.findNodeById ? window.findNodeById(el.dataset.nodeId) : null;
            if (node) applyNodeStyles(el, node);
        });
        applyForcedState();
        if (window.updateSelectionChrome) window.updateSelectionChrome();
        const id = window.DayStudioState.activeElementId;
        if (id && window.updateBoxOverlays) window.updateBoxOverlays(id);
    }
    window.refreshAllNodeStyles = refreshAllNodeStyles;
})();
"###;
