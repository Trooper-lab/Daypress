//! Shared Inspector Control Kit: panels, rows, unit-aware length inputs, selects, segmented
//! groups, color wells, sliders, label scrubbing and layer-aware binding (`scripts/inspector_controls.rs`).

pub const JS_INSPECTOR_CONTROLS: &str = r###"
(function() {
    const NSI = window.NSInspector = {};
    const panelOpen = {};
    const DEFAULT_CLOSED = { position: true, effects: true, customcss: true, gridchild: true };

    const U_LEN = ["px", "%", "rem", "em", "vw", "vh"];
    const U_AUTO = ["px", "%", "rem", "em", "vw", "vh", "auto"];
    const UNIT_PROPS = {
        padding_top: U_LEN, padding_right: U_LEN, padding_bottom: U_LEN, padding_left: U_LEN,
        margin_top: U_AUTO, margin_right: U_AUTO, margin_bottom: U_AUTO, margin_left: U_AUTO,
        font_size: ["px", "rem", "em", "%", "vw"], line_height: ["", "px", "%", "rem", "em"],
        letter_spacing: ["px", "em", "rem", "%"], gap: U_LEN, grid_gap: U_LEN,
        width: [...U_AUTO, "fit-content"], min_width: U_AUTO, max_width: ["px", "%", "rem", "em", "vw", "ch", "none"],
        height: [...U_AUTO, "fit-content"], min_height: U_AUTO, max_height: ["px", "%", "rem", "em", "vh", "vw", "none"],
        top: U_AUTO, right: U_AUTO, bottom: U_AUTO, left: U_AUTO,
        border_radius: ["px", "%", "rem", "em"], border_width: ["px", "rem", "em"]
    };

    const LENGTH_PROPS = new Set(Object.keys(UNIT_PROPS).filter(k => k !== "line_height"));
    const STEP = { opacity: 0.05, line_height: 0.1, font_weight: 100, z_index: 1 };
    NSI.LENGTH_PROPS = LENGTH_PROPS;
    NSI.UNIT_PROPS = UNIT_PROPS;

    const esc = (v) => String(v === undefined || v === null ? "" : v).replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
    NSI.esc = esc;
    const defaultUnitFor = (prop) => prop === "line_height" ? "" : (UNIT_PROPS[prop] ? "px" : "");
    const isKeywordUnit = (u) => u === "auto" || u === "none" || u === "fit-content";

    function splitStoredLength(prop, raw, fallbackUnit) {
        const dflt = fallbackUnit !== undefined ? fallbackUnit : defaultUnitFor(prop);
        const s = String(raw === undefined || raw === null ? "" : raw).trim();
        if (!s) return { num: "", unit: dflt, isKeyword: false, isExpression: false };
        const low = s.toLowerCase();
        if (isKeywordUnit(low) || low === "inherit" || low === "initial") {
            return { num: low, unit: isKeywordUnit(low) ? low : dflt, isKeyword: true, isExpression: false };
        }
        const m = /^(-?\d*\.?\d+)\s*(px|%|rem|em|vw|vh|vmin|vmax|ch|pt|fr)?$/i.exec(s);
        if (m) return { num: m[1], unit: m[2] ? m[2].toLowerCase() : (prop === "line_height" ? "" : dflt), isKeyword: false, isExpression: false };
        return { num: s, unit: dflt, isKeyword: false, isExpression: true };
    }
    NSI.splitStoredLength = splitStoredLength;

    function parseTypedLength(prop, raw, activeUnit) {
        const s = String(raw === undefined || raw === null ? "" : raw).trim();
        const dflt = (activeUnit !== undefined && !isKeywordUnit(activeUnit)) ? activeUnit : defaultUnitFor(prop);
        if (!s) return { css: "", num: "", unit: dflt };
        if (s === "-" || s === "." || s === "-." || /^-?\d+\.$/.test(s)) return { partial: true };
        const low = s.toLowerCase();
        if (isKeywordUnit(low) || low === "inherit" || low === "initial") {
            return { css: low, num: low, unit: isKeywordUnit(low) ? low : dflt, detectedUnit: isKeywordUnit(low) ? low : null };
        }
        const mUnit = /^(-?\d*\.?\d+)\s*(?:px|%|rem|em|vw|vh|vmin|vmax|ch|pt|fr)?\s*(px|%|rem|em|vw|vh|vmin|vmax|ch|pt|fr)$/i.exec(s);
        if (mUnit) return { css: mUnit[1] + mUnit[2].toLowerCase(), num: mUnit[1], unit: mUnit[2].toLowerCase(), detectedUnit: mUnit[2].toLowerCase() };
        if (/^(-?\d*\.?\d+)$/.test(s)) return { css: s + dflt, num: s, unit: dflt, detectedUnit: null };
        return { css: s, num: s, unit: dflt, isExpression: true };
    }

    NSI.panel = function(container, id, icon, title, bodyHtml) {
        const open = panelOpen[id] !== undefined ? panelOpen[id] : !DEFAULT_CLOSED[id];
        const panel = document.createElement("div");
        panel.className = "cad-panel" + (open ? " open" : "");
        panel.dataset.panelId = id;
        panel.innerHTML = `<div class="cad-panel-header"><span class="cad-panel-title"><i class="fa-solid ${icon}"></i> ${title}<span class="panel-set-dot" title="Has values on this layer"></span></span><i class="fa-solid fa-chevron-right cad-panel-chevron"></i></div><div class="cad-panel-body">${bodyHtml}</div>`;
        panel.querySelector(".cad-panel-header").onclick = () => { panelOpen[id] = panel.classList.toggle("open"); };
        container.appendChild(panel);
        return panel;
    };

    NSI.row = (label, props, inner, attrs) =>
        `<div class="cad-row ns-row" ${attrs || ""}><span class="cad-label" data-label-for="${props}">${label}</span><div class="ns-ctl">${inner}</div></div>`;

    NSI.pair = (l1, p1, c1, l2, p2, c2, attrs) =>
        `<div class="cad-row ns-pair" ${attrs || ""}><span class="cad-label" data-label-for="${p1}">${l1}</span><div class="ns-ctl">${c1}</div><span class="cad-label" data-label-for="${p2}">${l2}</span><div class="ns-ctl">${c2}</div></div>`;

    NSI.input = (prop, ph, list) => {
        const units = UNIT_PROPS[prop];
        if (units && !list) {
            const opts = units.map(u => `<option value="${esc(u)}">${u === "" ? "—" : (u === "fit-content" ? "fit" : u)}</option>`).join("");
            return `<div class="ns-unit-wrap"><input type="text" class="cad-input font-mono ns-input" data-prop="${prop}" data-ph="${esc(ph || "")}" spellcheck="false" autocomplete="off"><select class="ns-unit-badge" data-unit-prop="${prop}" tabindex="-1" title="Select unit or type unit directly (e.g. 24px, 100%, 2rem, auto)">${opts}</select></div>`;
        }
        return `<input type="text" class="cad-input font-mono ns-input" data-prop="${prop}" data-ph="${esc(ph || "")}" spellcheck="false" autocomplete="off" ${list ? `list="${list}"` : ""}>`;
    };

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

    function normalizeValue(prop, raw, activeUnit) {
        let v = String(raw === undefined || raw === null ? "" : raw).trim();
        if (!v) return "";
        if (UNIT_PROPS[prop]) {
            const parsed = parseTypedLength(prop, v, activeUnit);
            return parsed.partial ? v : parsed.css;
        }
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
        const varFallback = /#[0-9a-f]{3,6}/i.exec(s);
        if (varFallback) return toHex(varFallback[0]);
        const m = /rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/i.exec(s);
        if (m) return "#" + [m[1], m[2], m[3]].map(n => Math.min(255, +n).toString(16).padStart(2, "0")).join("");
        return "";
    }
    NSI.toHex = toHex;

    const canvasEl = (node) => node ? document.querySelector(`#neostudio-canvas [data-node-id="${node.id}"]`) : null;
    NSI.canvasEl = canvasEl;

    function computedOf(node, prop) {
        const el = canvasEl(node), css = window.NeoStyle.cssPropFor(prop);
        if (!el || !css) return "";
        let v = getComputedStyle(el).getPropertyValue(css).trim();
        if (prop === "font_family") v = v.split(",")[0].replace(/["']/g, "").trim();
        if (/color$/.test(prop)) v = toHex(v) || v;
        if (v === "none" && /^(box_shadow|transform|filter|backdrop_filter|max_width|max_height)$/.test(prop)) return "none";
        if (prop === "line_height" && /px$/.test(v)) {
            const fs = parseFloat(getComputedStyle(el).fontSize) || 16, lhPx = parseFloat(v);
            if (lhPx > 0) return String(Math.round((lhPx / fs) * 100) / 100);
        }
        return v;
    }
    NSI.computedOf = computedOf;

    const effective = (node, prop) => window.getLayerValue(node, prop) || window.getInheritedValue(node, prop) || computedOf(node, prop);
    NSI.effective = effective;
    const short = (v, n) => { v = String(v || ""); return v.length > (n || 18) ? v.slice(0, (n || 18) - 1) + "…" : v; };
    NSI.short = short;

    NSI.refresh = function(root, node) {
        root.querySelectorAll("[data-prop]").forEach(ctl => {
            const prop = ctl.dataset.prop, val = window.getLayerValue(node, prop), inh = window.getInheritedValue(node, prop), comp = computedOf(node, prop);
            if (ctl.classList.contains("ns-seg")) {
                const eff = val || inh || comp;
                ctl.querySelectorAll(".seg-btn").forEach(b => {
                    b.classList.toggle("active", !!val && b.dataset.val === val);
                    b.classList.toggle("ghost", !val && b.dataset.val === eff);
                });
            } else if (ctl.tagName === "SELECT") {
                if (val && !Array.from(ctl.options).some(o => o.value === val)) {
                    const o = document.createElement("option"); o.value = val; o.textContent = val; ctl.appendChild(o);
                }
                if (document.activeElement !== ctl) ctl.value = val;
                ctl.options[0].textContent = "— " + short(inh || comp || "default", 16);
                ctl.classList.toggle("is-unset", !val);
            } else if (UNIT_PROPS[prop]) {
                const unitSel = root.querySelector(`[data-unit-prop="${prop}"]`), fallbackRaw = inh || comp || ctl.dataset.ph || "";
                const spVal = val ? splitStoredLength(prop, val, ctl.dataset.unit) : null, spPh = splitStoredLength(prop, fallbackRaw);
                const activeUnit = spVal ? spVal.unit : (ctl.dataset.unit !== undefined ? ctl.dataset.unit : spPh.unit);
                if (unitSel && document.activeElement !== unitSel) {
                    if (activeUnit !== undefined && !Array.from(unitSel.options).some(o => o.value === activeUnit)) {
                        const o = document.createElement("option"); o.value = activeUnit; o.textContent = activeUnit || "—"; unitSel.appendChild(o);
                    }
                    unitSel.value = activeUnit;
                    unitSel.classList.toggle("is-keyword", isKeywordUnit(activeUnit));
                }
                if (document.activeElement !== ctl) {
                    ctl.value = spVal ? spVal.num : "";
                    ctl.placeholder = short(spPh.num || fallbackRaw, 18);
                }
            } else if (document.activeElement !== ctl) {
                ctl.value = val;
                ctl.placeholder = short(inh || comp || ctl.dataset.ph || "", 22);
            }
        });
        if (window.refreshFontControls) window.refreshFontControls(root, node);
        root.querySelectorAll("[data-swatch]").forEach(sw => {
            const prop = sw.dataset.swatch, hex = toHex(effective(node, prop));
            if (document.activeElement !== sw) sw.value = hex || "#000000";
            sw.classList.toggle("is-empty", !window.getLayerValue(node, prop));
        });
        root.querySelectorAll("[data-range]").forEach(r => {
            const v = parseFloat(effective(node, r.dataset.range));
            if (document.activeElement !== r && !isNaN(v)) r.value = v;
        });
        root.querySelectorAll("[data-label-for]").forEach(lbl => {
            const props = lbl.dataset.labelFor.split(",").filter(Boolean);
            const set = props.some(p => window.getLayerValue(node, p)), inherited = !set && props.some(p => window.getInheritedValue(node, p));
            const scrubbable = props.length === 1 && (UNIT_PROPS[props[0]] || STEP[props[0]]);
            lbl.classList.toggle("has-override", set);
            lbl.classList.toggle("is-inherited", inherited);
            lbl.classList.toggle("is-scrubbable", !!scrubbable && !set);
            lbl.title = set ? "Set on this layer — click to reset (or drag horizontally to scrub)" : (scrubbable ? "Drag horizontally to scrub value" : (inherited ? "Inherited from another layer" : ""));
        });
        root.querySelectorAll("[data-show-if]").forEach(row => {
            const m = /^(\w+)(!?=)(.*)$/.exec(row.dataset.showIf);
            if (!m) return;
            const hit = m[3].split("|").includes(effective(node, m[1]));
            row.style.display = (m[2] === "=" ? hit : !hit) ? "" : "none";
        });
        root.querySelectorAll(".cad-panel").forEach(p => {
            const any = Array.from(p.querySelectorAll("[data-prop]")).some(c => window.getLayerValue(node, c.dataset.prop));
            p.classList.toggle("has-values", any);
        });
    };

    NSI.bind = function(root, node) {
        const refreshSoon = () => NSI.refresh(root, node);
        const set = (prop, v, commit) => { window.setLocalStyle(prop, v, { commit: !!commit }); refreshSoon(); };
        let commitTimer = 0;
        const commitSoon = () => { clearTimeout(commitTimer); commitTimer = setTimeout(() => window.commitStyleChange(), 450); };

        root.querySelectorAll("input.ns-input[data-prop], textarea.ns-textarea[data-prop]").forEach(ctl => {
            const prop = ctl.dataset.prop, unitSel = root.querySelector(`[data-unit-prop="${prop}"]`);
            let original = "";
            ctl.addEventListener("focus", () => { original = window.getLayerValue(node, prop) || ""; });
            ctl.addEventListener("input", () => {
                if (UNIT_PROPS[prop]) {
                    const activeU = ctl.dataset.unit !== undefined ? ctl.dataset.unit : (unitSel ? unitSel.value : defaultUnitFor(prop));
                    const parsed = parseTypedLength(prop, ctl.value, activeU);
                    if (parsed.partial) return;
                    if (parsed.detectedUnit) { ctl.dataset.unit = parsed.detectedUnit; if (unitSel) unitSel.value = parsed.detectedUnit; }
                    set(prop, parsed.css, false);
                } else {
                    set(prop, normalizeValue(prop, ctl.value), false);
                }
                const range = root.querySelector(`[data-range="${prop}"]`);
                if (range && !isNaN(parseFloat(ctl.value))) range.value = parseFloat(ctl.value);
            });
            ctl.addEventListener("change", () => {
                if (UNIT_PROPS[prop]) {
                    const activeU = ctl.dataset.unit !== undefined ? ctl.dataset.unit : (unitSel ? unitSel.value : defaultUnitFor(prop));
                    const parsed = parseTypedLength(prop, ctl.value, activeU);
                    if (!parsed.partial) {
                        if (parsed.detectedUnit) ctl.dataset.unit = parsed.detectedUnit;
                        ctl.value = parsed.num;
                        set(prop, parsed.css, true);
                    }
                } else {
                    const v = normalizeValue(prop, ctl.value);
                    ctl.value = v;
                    set(prop, v, true);
                }
            });
            ctl.addEventListener("keydown", (e) => {
                if (e.key === "Enter" && ctl.tagName !== "TEXTAREA") { ctl.blur(); return; }
                if (e.key === "Escape") { set(prop, original, false); ctl.blur(); return; }
                if ((e.key === "ArrowUp" || e.key === "ArrowDown") && ctl.tagName !== "TEXTAREA") {
                    const rawCur = window.getLayerValue(node, prop) || window.getInheritedValue(node, prop) || computedOf(node, prop) || "0";
                    const sp = splitStoredLength(prop, rawCur, ctl.dataset.unit || (unitSel && unitSel.value));
                    const numVal = parseFloat(ctl.value.trim() !== "" ? ctl.value : sp.num);
                    if (isNaN(numVal)) return;
                    e.preventDefault();
                    const base = STEP[prop] || 1, step = e.shiftKey ? base * 10 : (e.altKey ? base / 10 : base);
                    let n = numVal + (e.key === "ArrowUp" ? step : -step);
                    if (prop === "opacity") n = Math.min(1, Math.max(0, n));
                    if (/^(padding|font_size|width|height|min_|max_|border_|gap|grid_gap)/.test(prop) && n < 0) n = 0;
                    n = Math.round(n * 100) / 100;
                    if (UNIT_PROPS[prop]) {
                        const u = isKeywordUnit(sp.unit) ? defaultUnitFor(prop) : sp.unit;
                        ctl.dataset.unit = u;
                        if (unitSel) unitSel.value = u;
                        ctl.value = String(n);
                        set(prop, n + u, false);
                    } else {
                        ctl.value = String(n);
                        set(prop, String(n), false);
                    }
                    commitSoon();
                }
            });
        });

        root.querySelectorAll("select.ns-unit-badge[data-unit-prop]").forEach(unitSel => {
            const prop = unitSel.dataset.unitProp, ctl = root.querySelector(`input.ns-input[data-prop="${prop}"]`);
            unitSel.addEventListener("change", () => {
                const u = unitSel.value;
                if (ctl) ctl.dataset.unit = u;
                if (isKeywordUnit(u)) { if (ctl) ctl.value = u; set(prop, u, true); return; }
                const curTyped = ctl ? ctl.value.trim() : "", mNum = /^(-?\d*\.?\d+)/.exec(curTyped);
                if (mNum) {
                    if (ctl) ctl.value = mNum[1];
                    set(prop, mNum[1] + u, true);
                } else {
                    const sp = splitStoredLength(prop, window.getLayerValue(node, prop), u);
                    if (sp.num && !sp.isKeyword && !isNaN(parseFloat(sp.num))) {
                        if (ctl) ctl.value = sp.num;
                        set(prop, sp.num + u, true);
                    } else if (ctl) {
                        if (isKeywordUnit(curTyped)) ctl.value = "";
                        ctl.focus();
                    }
                }
            });
        });

        root.querySelectorAll("select.ns-select[data-prop]").forEach(ctl => ctl.addEventListener("change", () => set(ctl.dataset.prop, ctl.value, true)));
        root.querySelectorAll(".ns-seg[data-prop]").forEach(group => {
            const prop = group.dataset.prop;
            group.querySelectorAll(".seg-btn").forEach(btn => btn.addEventListener("click", () => {
                const cur = window.getLayerValue(node, prop);
                set(prop, cur === btn.dataset.val ? "" : btn.dataset.val, true);
            }));
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
        root.querySelectorAll("[data-clear]").forEach(btn => btn.addEventListener("click", () => set(btn.dataset.clear, "", true)));
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
            const props = lbl.dataset.labelFor.split(",").filter(Boolean), prop = props[0];
            const canScrub = props.length === 1 && (UNIT_PROPS[prop] || STEP[prop]);
            lbl.addEventListener("mousedown", (e) => {
                if (e.button !== 0) return;
                const startX = e.clientX, rawStart = window.getLayerValue(node, prop) || window.getInheritedValue(node, prop) || computedOf(node, prop) || "0";
                const sp = splitStoredLength(prop, rawStart), startNum = parseFloat(sp.num) || 0;
                const unit = isKeywordUnit(sp.unit) ? defaultUnitFor(prop) : sp.unit, base = STEP[prop] || 1;
                let moved = false;
                const onMove = (me) => {
                    if (!canScrub) return;
                    const dx = me.clientX - startX;
                    if (!moved && Math.abs(dx) < 3) return;
                    moved = true;
                    document.body.style.cursor = "ew-resize";
                    document.body.style.userSelect = "none";
                    let n = startNum + dx * (me.shiftKey ? base * 5 : (me.altKey ? base * 0.1 : base));
                    if (prop === "opacity") n = Math.min(1, Math.max(0, n));
                    if (/^(padding|font_size|width|height|min_|max_|border_|gap|grid_gap)/.test(prop) && n < 0) n = 0;
                    n = Math.round(n * 100) / 100;
                    set(prop, UNIT_PROPS[prop] ? (n + unit) : String(n), false);
                };
                const onUp = () => {
                    document.body.style.cursor = "";
                    document.body.style.userSelect = "";
                    window.removeEventListener("mousemove", onMove);
                    window.removeEventListener("mouseup", onUp);
                    if (moved) window.commitStyleChange();
                    else if (lbl.classList.contains("has-override")) {
                        const map = {};
                        props.forEach(p => { map[p] = ""; });
                        window.setLocalStyles(map, { commit: true });
                        NSI.refresh(root, node);
                    }
                };
                window.addEventListener("mousemove", onMove);
                window.addEventListener("mouseup", onUp);
            });
        });

        NSI.refresh(root, node);
    };
})();
"###;
