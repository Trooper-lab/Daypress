//! Interactive Trapezoidal Box-Model Scrubber Engine and Unit Switcher (`scripts/inspector_box_model.rs`).
//!
//! - Drag a value horizontally to scrub (Shift = x10, Alt = x0.1); Alt+click resets that side.
//! - Click (or double-click) to type any CSS length (`24`, `24px`, `10%`, `2rem`, `auto`); empty clears the side.
//! - Auto-detects typed units (`px`, `%`, `rem`, `em`, `vh`, `vw`) and syncs the unit selector.

pub const JS_INSPECTOR_BOX_MODEL: &str = r###"
(function() {
    let currentUnit = "px";
    let isLinked = false;
    const SIDES = ["top", "right", "bottom", "left"];
    const BOX_UNITS = ["px", "%", "rem", "em", "vh", "vw"];

    function renderBoxModelWidget(container, node) {
        const boxContainer = document.createElement("div");
        boxContainer.className = "neo-box-model-container";
        const facet = (group, side) =>
            `<div class="neo-box-facet facet-${side}" data-prop="${group}_${side}" data-side="${side}" title="${group} ${side} — click to type (e.g. 24px, 10%, auto), drag to scrub, Alt+click to reset"><span class="neo-val"></span></div>`;

        const unitOpts = BOX_UNITS.map(u => `<option value="${u}" ${u === currentUnit ? "selected" : ""}>${u}</option>`).join("");

        boxContainer.innerHTML = `
            <div class="neo-box-model-widget">
                <div class="neo-box-margin-outer">
                    <span class="neo-box-label">MARGIN</span>
                    ${SIDES.map(s => facet("margin", s)).join("")}
                    <div class="neo-box-padding-inner">
                        <span class="neo-box-label" style="color:var(--neo-signal-light);">PADDING</span>
                        ${SIDES.map(s => facet("padding", s)).join("")}
                        <div class="neo-box-content-core" title="Rendered element size"></div>
                    </div>
                </div>
                <div class="neo-box-tools">
                    <div class="neo-unit-toggle">
                        <span>Unit:</span>
                        <select class="btn-unit-cycle" id="btn-cycle-box-unit" title="Default unit for scrubbing and unit-less values">${unitOpts}</select>
                    </div>
                    <div style="display:flex;gap:4px;">
                        <button class="btn-link-quads" id="btn-box-center" title="Center horizontally (margin-left/right: auto)"><i class="fa-solid fa-arrows-left-right-to-line"></i></button>
                        <button class="btn-link-quads ${isLinked ? 'is-linked' : ''}" id="btn-toggle-box-link" title="Link all 4 sides"><i class="fa-solid fa-link"></i></button>
                    </div>
                </div>
            </div>
        `;
        container.appendChild(boxContainer);
        paint(boxContainer, node);
        initScrubbers(boxContainer, node);

        const unitSel = boxContainer.querySelector("#btn-cycle-box-unit");
        if (unitSel) {
            unitSel.onchange = () => {
                currentUnit = unitSel.value;
            };
        }
        boxContainer.querySelector("#btn-toggle-box-link").onclick = (e) => {
            isLinked = !isLinked;
            e.currentTarget.classList.toggle("is-linked", isLinked);
        };
        boxContainer.querySelector("#btn-box-center").onclick = () => {
            window.setLocalStyles({ margin_left: "auto", margin_right: "auto" }, { commit: true });
            paint(boxContainer, node);
        };
    }
    window.renderBoxModelWidget = renderBoxModelWidget;

    function cssSide(prop) { return prop.replace("_", "-"); }

    function computedPx(node, prop) {
        const el = window.NSInspector ? window.NSInspector.canvasEl(node) : null;
        if (!el) return 0;
        return Math.round(parseFloat(getComputedStyle(el).getPropertyValue(cssSide(prop))) || 0);
    }

    function displayVal(v) {
        const s = String(v).trim();
        return /^-?\d*\.?\d+px$/.test(s) ? String(parseFloat(s)) : s;
    }

    function paint(container, node) {
        container.querySelectorAll(".neo-box-facet").forEach(f => {
            const prop = f.dataset.prop;
            const val = window.getLayerValue(node, prop);
            const inh = window.getInheritedValue(node, prop);
            const span = f.querySelector(".neo-val");
            f.classList.remove("is-set", "is-inherited", "is-computed");
            if (val) { span.textContent = displayVal(val); f.classList.add("is-set"); }
            else if (inh) { span.textContent = displayVal(inh); f.classList.add("is-inherited"); }
            else { span.textContent = computedPx(node, prop); f.classList.add("is-computed"); }
        });
        const el = window.NSInspector ? window.NSInspector.canvasEl(node) : null;
        const core = container.querySelector(".neo-box-content-core");
        if (core) core.innerHTML = el ? `${Math.round(el.offsetWidth)} &times; ${Math.round(el.offsetHeight)}` : "&mdash;";
    }
    window.repaintBoxModel = function() {
        const c = document.querySelector("#neostudio-inspector .neo-box-model-container");
        const id = window.NeoStudioState.activeElementId;
        const node = id ? window.findNodeById(id) : null;
        if (c && node) paint(c, node);
    };

    function targets(prop) {
        if (!isLinked) return [prop];
        const group = prop.split("_")[0];
        return SIDES.map(s => group + "_" + s);
    }

    function applyValue(node, prop, value, commit) {
        const map = {};
        targets(prop).forEach(p => { map[p] = value; });
        window.setLocalStyles(map, { commit: !!commit });
    }

    function startingPoint(node, prop) {
        const raw = window.getLayerValue(node, prop) || window.getInheritedValue(node, prop);
        const m = /^(-?\d*\.?\d+)([a-z%]*)$/i.exec(String(raw || "").trim());
        if (m) return { val: parseFloat(m[1]), unit: m[2] || currentUnit };
        return { val: computedPx(node, prop), unit: currentUnit === "px" ? "px" : currentUnit };
    }

    function openFacetInput(container, facet, node, prop, isPadding) {
        if (facet.querySelector("input.neo-box-model-inline-input")) return;
        const valEl = facet.querySelector(".neo-val");
        const input = document.createElement("input");
        input.type = "text";
        input.className = "neo-box-model-inline-input";
        input.value = window.getLayerValue(node, prop) || "";
        input.placeholder = valEl.textContent.trim();
        valEl.style.display = "none";
        facet.appendChild(input);
        input.focus();
        input.select();

        let done = false;
        function finish(apply) {
            if (done) return;
            done = true;
            if (apply) {
                let v = input.value.trim();
                const mUnit = /^(-?\d*\.?\d+)\s*(?:px|%|rem|em|vh|vw)?\s*(px|%|rem|em|vh|vw)$/i.exec(v);
                if (mUnit) {
                    let n = parseFloat(mUnit[1]);
                    if (isPadding && n < 0) n = 0;
                    const u = mUnit[2].toLowerCase();
                    currentUnit = u;
                    const unitSel = container.querySelector("#btn-cycle-box-unit");
                    if (unitSel) unitSel.value = u;
                    v = n + u;
                } else if (/^-?\d*\.?\d+$/.test(v)) {
                    let n = parseFloat(v);
                    if (isPadding && n < 0) n = 0;
                    v = n + currentUnit;
                }
                applyValue(node, prop, v, true);
            }
            input.remove();
            valEl.style.display = "";
            paint(container, node);
        }
        input.onblur = () => finish(true);
        input.onkeydown = (ke) => {
            ke.stopPropagation();
            if (ke.key === "Enter") finish(true);
            if (ke.key === "Escape") finish(false);
        };
    }

    function initScrubbers(container, node) {
        container.querySelectorAll(".neo-box-facet").forEach(facet => {
            const prop = facet.dataset.prop;
            const isPadding = prop.startsWith("padding");

            facet.addEventListener("mousedown", (e) => {
                if (e.target.tagName === "INPUT" || e.button !== 0) return;
                e.preventDefault();
                e.stopPropagation();
                if (node.data && node.data._locked) return;

                if (e.altKey) {
                    applyValue(node, prop, "", true);
                    paint(container, node);
                    return;
                }

                const startX = e.clientX;
                const start = startingPoint(node, prop);
                const fine = start.unit !== "px";
                let moved = false;
                document.body.style.cursor = "ew-resize";
                document.body.style.userSelect = "none";

                function onMouseMove(me) {
                    const dx = me.clientX - startX;
                    if (!moved && Math.abs(dx) < 3) return;
                    moved = true;
                    const mult = (me.shiftKey ? 10 : (me.altKey ? 0.1 : 1)) * (fine ? 0.1 : 1);
                    let n = start.val + dx * mult;
                    n = fine || me.altKey ? Math.round(n * 10) / 10 : Math.round(n);
                    if (isPadding && n < 0) n = 0;
                    applyValue(node, prop, n + start.unit, false);
                    paint(container, node);
                }

                function onMouseUp() {
                    document.body.style.cursor = "";
                    document.body.style.userSelect = "";
                    window.removeEventListener("mousemove", onMouseMove);
                    window.removeEventListener("mouseup", onMouseUp);
                    if (moved) {
                        window.commitStyleChange();
                    } else {
                        openFacetInput(container, facet, node, prop, isPadding);
                    }
                }

                window.addEventListener("mousemove", onMouseMove);
                window.addEventListener("mouseup", onMouseUp);
            });
        });
    }
})();
"###;
