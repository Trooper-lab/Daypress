//! Shared Inspector Control Kit: panels, rows, inputs, selects, segmented groups, color wells,
//! sliders, value normalization and layer-aware binding (`scripts/inspector_controls.rs`).
//!
//! Every control carries `data-prop`. `NSInspector.bind()` wires it to the current style layer:
//! - the field shows the value set on the layer being edited (desktop/tablet/mobile/hover/...),
//! - the placeholder shows the inherited value (or the live computed value on the canvas),
//! - its label turns blue when set on this layer (click to reset) and amber when inherited.

pub const JS_INSPECTOR_CONTROLS: &str = r###"
(function() {
    const NSI = window.NSInspector = {};
    const panelOpen = {};
    const DEFAULT_CLOSED = { position: true, effects: true, customcss: true, gridchild: true };

    const LENGTH_PROPS = new Set(["padding_top", "padding_right", "padding_bottom", "padding_left",
        "margin_top", "margin_right", "margin_bottom", "margin_left", "font_size", "letter_spacing", "gap", "grid_gap",
        "width", "min_width", "max_width", "height", "min_height", "max_height", "top", "right", "bottom", "left",
        "border_radius", "border_width"]);
    const STEP = { opacity: 0.05, line_height: 0.1, font_weight: 100 };
    NSI.LENGTH_PROPS = LENGTH_PROPS;

    function esc(v) {
        return String(v === undefined || v === null ? "" : v)
            .replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
    }
    NSI.esc = esc;

    // -------------------------------------------------------------
    // MARKUP BUILDERS
    // -------------------------------------------------------------
    NSI.panel = function(container, id, icon, title, bodyHtml) {
        const open = panelOpen[id] !== undefined ? panelOpen[id] : !DEFAULT_CLOSED[id];
        const panel = document.createElement("div");
        panel.className = "cad-panel" + (open ? " open" : "");
        panel.dataset.panelId = id;
        panel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid ${icon}"></i> ${title}<span class="panel-set-dot" title="Has values on this layer"></span></span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">${bodyHtml}</div>`;
        panel.querySelector(".cad-panel-header").onclick = () => { panelOpen[id] = panel.classList.toggle("open"); };
        container.appendChild(panel);
        return panel;
    };

    NSI.row = (label, props, inner, attrs) =>
        `<div class="cad-row ns-row" ${attrs || ""}><span class="cad-label" data-label-for="${props}">${label}</span><div class="ns-ctl">${inner}</div></div>`;

    NSI.pair = (l1, p1, c1, l2, p2, c2, attrs) =>
        `<div class="cad-row ns-pair" ${attrs || ""}>
            <span class="cad-label" data-label-for="${p1}">${l1}</span><div class="ns-ctl">${c1}</div>
            <span class="cad-label" data-label-for="${p2}">${l2}</span><div class="ns-ctl">${c2}</div>
        </div>`;

    NSI.input = (prop, ph, list) =>
        `<input type="text" class="cad-input font-mono ns-input" data-prop="${prop}" data-ph="${esc(ph || "")}" spellcheck="false" autocomplete="off" ${list ? `list="${list}"` : ""}>`;

    NSI.select = (prop, options) =>
        `<select class="cad-select ns-select" data-prop="${prop}"><option value="">—</option>${options.map(o => {
            const [v, l] = Array.isArray(o) ? o : [o, o];
            return `<option value="${esc(v)}">${l}</option>`;
        }).join("")}</select>`;

    NSI.seg = (prop, options) =>
        `<div class="cad-segmented ns-seg" data-prop="${prop}">${options.map(([v, html, title]) =>
            `<button type="button" class="seg-btn" data-val="${esc(v)}" title="${esc(title || v)}">${html}</button>`).join("")}</div>`;

    NSI.color = (prop) =>
        `<div class="ns-color"><input type="color" class="color-swatch-box" data-swatch="${prop}">${NSI.input(prop)}<button type="button" class="ns-clear" data-clear="${prop}" title="Clear">&times;</button></div>`;

    NSI.range = (prop, min, max, step) =>
        `<div class="ns-range"><input type="range" min="${min}" max="${max}" step="${step}" data-range="${prop}">${NSI.input(prop)}</div>`;

    NSI.textarea = (prop, ph) =>
        `<textarea class="cad-textarea font-mono ns-textarea" data-prop="${prop}" data-ph="${esc(ph || "")}" placeholder="${esc(ph || "")}" spellcheck="false"></textarea>`;

    // -------------------------------------------------------------
    // VALUE HELPERS
    // -------------------------------------------------------------
    function normalizeValue(prop, raw) {
        let v = String(raw === undefined || raw === null ? "" : raw).trim();
        if (!v) return "";
        if (LENGTH_PROPS.has(prop) && /^-?\d*\.?\d+$/.test(v)) return v + "px";
        if (/(color)$/.test(prop) && /^[0-9a-f]{3}([0-9a-f]{3})?([0-9a-f]{2})?$/i.test(v)) return "#" + v;
        return v;
    }
    NSI.normalizeValue = normalizeValue;

    function toHex(v) {
        if (!v) return "";
        const s = String(v).trim();
        if (/^#[0-9a-f]{6}$/i.test(s)) return s.toLowerCase();
        if (/^#[0-9a-f]{3}$/i.test(s)) return "#" + s.slice(1).split("").map(c => c + c).join("").toLowerCase();
        if (/^#[0-9a-f]{8}$/i.test(s)) return s.slice(0, 7).toLowerCase();
        const m = /rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/i.exec(s);
        if (m) return "#" + [m[1], m[2], m[3]].map(n => Math.min(255, +n).toString(16).padStart(2, "0")).join("");
        return "";
    }
    NSI.toHex = toHex;

    function canvasEl(node) {
        return node ? document.querySelector(`#neostudio-canvas [data-node-id="${node.id}"]`) : null;
    }
    NSI.canvasEl = canvasEl;

    function computedOf(node, prop) {
        const el = canvasEl(node);
        const css = window.NeoStyle.cssPropFor(prop);
        if (!el || !css) return "";
        let v = getComputedStyle(el).getPropertyValue(css).trim();
        if (prop === "font_family") v = v.split(",")[0].replace(/["']/g, "").trim();
        if (/color$/.test(prop)) v = toHex(v) || v;
        if (v === "none" && /^(box_shadow|transform|filter|backdrop_filter|max_width|max_height)$/.test(prop)) return "none";
        return v;
    }
    NSI.computedOf = computedOf;

    function effective(node, prop) {
        return window.getLayerValue(node, prop) || window.getInheritedValue(node, prop) || computedOf(node, prop);
    }
    NSI.effective = effective;

    function short(v, n) { v = String(v || ""); return v.length > (n || 18) ? v.slice(0, (n || 18) - 1) + "…" : v; }

    // -------------------------------------------------------------
    // REFRESH (sync every control with the current layer + canvas)
    // -------------------------------------------------------------
    NSI.refresh = function(root, node) {
        root.querySelectorAll("[data-prop]").forEach(ctl => {
            const prop = ctl.dataset.prop;
            const val = window.getLayerValue(node, prop);
            const inh = window.getInheritedValue(node, prop);
            if (ctl.classList.contains("ns-seg")) {
                const eff = val || inh || computedOf(node, prop);
                ctl.querySelectorAll(".seg-btn").forEach(b => {
                    b.classList.toggle("active", !!val && b.dataset.val === val);
                    b.classList.toggle("ghost", !val && b.dataset.val === eff);
                });
            } else if (ctl.tagName === "SELECT") {
                if (val && !Array.from(ctl.options).some(o => o.value === val)) {
                    const o = document.createElement("option"); o.value = val; o.textContent = val; ctl.appendChild(o);
                }
                if (document.activeElement !== ctl) ctl.value = val;
                ctl.options[0].textContent = "— " + short(inh || computedOf(node, prop) || "default", 16);
                ctl.classList.toggle("is-unset", !val);
            } else if (document.activeElement !== ctl) {
                ctl.value = val;
                ctl.placeholder = short(inh || computedOf(node, prop) || ctl.dataset.ph || "", 22);
            }
        });
        root.querySelectorAll("[data-swatch]").forEach(sw => {
            const prop = sw.dataset.swatch;
            const hex = toHex(effective(node, prop));
            if (document.activeElement !== sw) sw.value = hex || "#000000";
            sw.classList.toggle("is-empty", !window.getLayerValue(node, prop));
        });
        root.querySelectorAll("[data-range]").forEach(r => {
            const v = parseFloat(effective(node, r.dataset.range));
            if (document.activeElement !== r && !isNaN(v)) r.value = v;
        });
        root.querySelectorAll("[data-label-for]").forEach(lbl => {
            const props = lbl.dataset.labelFor.split(",").filter(Boolean);
            const set = props.some(p => window.getLayerValue(node, p));
            const inherited = !set && props.some(p => window.getInheritedValue(node, p));
            lbl.classList.toggle("has-override", set);
            lbl.classList.toggle("is-inherited", inherited);
            lbl.title = set ? "Set on this layer — click to reset" : inherited ? "Inherited from another layer — set a value to override" : "";
        });
        root.querySelectorAll("[data-show-if]").forEach(row => {
            const m = /^(\w+)(!?=)(.*)$/.exec(row.dataset.showIf);
            if (!m) return;
            const eff = effective(node, m[1]);
            const hit = m[3].split("|").includes(eff);
            row.style.display = (m[2] === "=" ? hit : !hit) ? "" : "none";
        });
        root.querySelectorAll(".cad-panel").forEach(p => {
            const any = Array.from(p.querySelectorAll("[data-prop]")).some(c => window.getLayerValue(node, c.dataset.prop));
            p.classList.toggle("has-values", any);
        });
    };

    // -------------------------------------------------------------
    // BINDING
    // -------------------------------------------------------------
    NSI.bind = function(root, node) {
        // Synchronous refresh: labels, ghosts and show-if rows must reflect a change before the next interaction
        const refreshSoon = () => NSI.refresh(root, node);
        const set = (prop, v, commit) => { window.setLocalStyle(prop, v, { commit: !!commit }); refreshSoon(); };
        let commitTimer = 0;
        const commitSoon = () => { clearTimeout(commitTimer); commitTimer = setTimeout(() => window.commitStyleChange(), 450); };

        root.querySelectorAll("input.ns-input[data-prop], textarea.ns-textarea[data-prop]").forEach(ctl => {
            const prop = ctl.dataset.prop;
            let original = "";
            ctl.addEventListener("focus", () => { original = ctl.value; });
            ctl.addEventListener("input", () => {
                // Live preview while typing; partial values ("1.") are simply ignored by the browser
                set(prop, normalizeValue(prop, ctl.value), false);
                const range = root.querySelector(`[data-range="${prop}"]`);
                if (range && !isNaN(parseFloat(ctl.value))) range.value = parseFloat(ctl.value);
            });
            ctl.addEventListener("change", () => {
                const v = normalizeValue(prop, ctl.value);
                ctl.value = v;
                set(prop, v, true);
            });
            ctl.addEventListener("keydown", (e) => {
                if (e.key === "Enter" && ctl.tagName !== "TEXTAREA") { ctl.blur(); return; }
                if (e.key === "Escape") { ctl.value = original; set(prop, normalizeValue(prop, original), false); ctl.blur(); return; }
                if ((e.key === "ArrowUp" || e.key === "ArrowDown") && ctl.tagName !== "TEXTAREA") {
                    const cur = ctl.value.trim() || window.getInheritedValue(node, prop) || computedOf(node, prop) || "0";
                    const m = /^(-?\d*\.?\d+)([a-z%]*)$/i.exec(cur);
                    if (!m) return;
                    e.preventDefault();
                    const base = STEP[prop] || 1;
                    const step = e.shiftKey ? base * 10 : (e.altKey ? base / 10 : base);
                    let n = parseFloat(m[1]) + (e.key === "ArrowUp" ? step : -step);
                    if (prop === "opacity") n = Math.min(1, Math.max(0, n));
                    if (/^padding|font_size|width|height/.test(prop) && n < 0) n = 0;
                    n = Math.round(n * 100) / 100;
                    const unit = m[2] || (LENGTH_PROPS.has(prop) ? "px" : "");
                    ctl.value = n + unit;
                    set(prop, ctl.value, false);
                    commitSoon();
                }
            });
        });

        root.querySelectorAll("select.ns-select[data-prop]").forEach(ctl => {
            ctl.addEventListener("change", () => set(ctl.dataset.prop, ctl.value, true));
        });

        root.querySelectorAll(".ns-seg[data-prop]").forEach(group => {
            const prop = group.dataset.prop;
            group.querySelectorAll(".seg-btn").forEach(btn => {
                btn.addEventListener("click", () => {
                    const cur = window.getLayerValue(node, prop);
                    set(prop, cur === btn.dataset.val ? "" : btn.dataset.val, true);
                });
            });
        });

        root.querySelectorAll("[data-swatch]").forEach(sw => {
            const prop = sw.dataset.swatch;
            sw.addEventListener("input", () => {
                const txt = root.querySelector(`input.ns-input[data-prop="${prop}"]`);
                if (txt) txt.value = sw.value;
                set(prop, sw.value, false);
            });
            sw.addEventListener("change", () => set(prop, sw.value, true));
        });

        root.querySelectorAll("[data-clear]").forEach(btn => {
            btn.addEventListener("click", () => set(btn.dataset.clear, "", true));
        });

        root.querySelectorAll("[data-range]").forEach(r => {
            const prop = r.dataset.range;
            r.addEventListener("input", () => {
                const txt = root.querySelector(`input.ns-input[data-prop="${prop}"]`);
                if (txt) txt.value = r.value;
                set(prop, r.value, false);
            });
            r.addEventListener("change", () => set(prop, r.value, true));
        });

        root.querySelectorAll("[data-label-for]").forEach(lbl => {
            lbl.addEventListener("click", () => {
                if (!lbl.classList.contains("has-override")) return;
                const map = {};
                lbl.dataset.labelFor.split(",").filter(Boolean).forEach(p => { map[p] = ""; });
                window.setLocalStyles(map, { commit: true });
                NSI.refresh(root, node);
            });
        });

        NSI.refresh(root, node);
    };
})();
"###;
