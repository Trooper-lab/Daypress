//! Interactive Trapezoidal Box-Model Scrubber Engine and Unit Switcher (`scripts/inspector_box_model.rs`).

pub const JS_INSPECTOR_BOX_MODEL: &str = r###"
(function() {
    let currentUnit = "px";
    let isLinked = false;

    function renderBoxModelWidget(container, node) {
        const boxContainer = document.createElement("div");
        boxContainer.className = "neo-box-model-container";

        // Read current margins & paddings
        const styles = node.styles || {};
        const mTop = getVal(styles, "margin_top", "margin", "top");
        const mRight = getVal(styles, "margin_right", "margin", "right");
        const mBottom = getVal(styles, "margin_bottom", "margin", "bottom");
        const mLeft = getVal(styles, "margin_left", "margin", "left");

        const pTop = getVal(styles, "padding_top", "padding", "top");
        const pRight = getVal(styles, "padding_right", "padding", "right");
        const pBottom = getVal(styles, "padding_bottom", "padding", "bottom");
        const pLeft = getVal(styles, "padding_left", "padding", "left");

        // Read computed dimensions
        const domEl = document.querySelector(`[data-node-id="${node.id}"]`);
        const dims = domEl ? `${Math.round(domEl.offsetWidth)} &times; ${Math.round(domEl.offsetHeight)}` : "&mdash;";

        boxContainer.innerHTML = `
            <div class="neo-box-model-widget">
                <!-- Outer Margin Trapezoid -->
                <div class="neo-box-margin-outer">
                    <span class="neo-box-label">MARGIN</span>
                    <div class="neo-box-facet facet-top" data-prop="margin_top" data-side="top"><span class="neo-val">${mTop}</span></div>
                    <div class="neo-box-facet facet-right" data-prop="margin_right" data-side="right"><span class="neo-val">${mRight}</span></div>
                    <div class="neo-box-facet facet-bottom" data-prop="margin_bottom" data-side="bottom"><span class="neo-val">${mBottom}</span></div>
                    <div class="neo-box-facet facet-left" data-prop="margin_left" data-side="left"><span class="neo-val">${mLeft}</span></div>

                    <!-- Inner Padding Trapezoid -->
                    <div class="neo-box-padding-inner">
                        <span class="neo-box-label" style="color:var(--neo-signal-light);">PADDING</span>
                        <div class="neo-box-facet facet-top" data-prop="padding_top" data-side="top"><span class="neo-val">${pTop}</span></div>
                        <div class="neo-box-facet facet-right" data-prop="padding_right" data-side="right"><span class="neo-val">${pRight}</span></div>
                        <div class="neo-box-facet facet-bottom" data-prop="padding_bottom" data-side="bottom"><span class="neo-val">${pBottom}</span></div>
                        <div class="neo-box-facet facet-left" data-prop="padding_left" data-side="left"><span class="neo-val">${pLeft}</span></div>

                        <!-- Center Core Content -->
                        <div class="neo-box-content-core" title="Element Dimensions">${dims}</div>
                    </div>
                </div>

                <!-- Footer Tools -->
                <div class="neo-box-tools">
                    <div class="neo-unit-toggle">
                        <span>Unit:</span>
                        <button class="btn-unit-cycle" id="btn-cycle-box-unit">${currentUnit}</button>
                    </div>
                    <button class="btn-link-quads ${isLinked ? 'is-linked' : ''}" id="btn-toggle-box-link" title="Link All 4 Sides">
                        <i class="fa-solid fa-link"></i>
                    </button>
                </div>
            </div>
        `;

        container.appendChild(boxContainer);

        // Attach scrubber drag interactions
        initScrubbers(boxContainer, node);

        // Unit toggle
        const btnUnit = boxContainer.querySelector("#btn-cycle-box-unit");
        if (btnUnit) {
            btnUnit.onclick = () => {
                const units = ["px", "rem", "%", "vh"];
                let idx = units.indexOf(currentUnit);
                currentUnit = units[(idx + 1) % units.length];
                btnUnit.textContent = currentUnit;
            };
        }

        // Link toggle
        const btnLink = boxContainer.querySelector("#btn-toggle-box-link");
        if (btnLink) {
            btnLink.onclick = () => {
                isLinked = !isLinked;
                btnLink.classList.toggle("is-linked", isLinked);
            };
        }
    }
    window.renderBoxModelWidget = renderBoxModelWidget;

    function getVal(styles, flatKey, groupKey, side) {
        if (styles[flatKey] !== undefined) return parseFloat(styles[flatKey]) || 0;
        if (styles[groupKey] && styles[groupKey][side] !== undefined) {
            return parseFloat(styles[groupKey][side]) || 0;
        }
        return 0;
    }

    function initScrubbers(container, node) {
        const facets = container.querySelectorAll(".neo-box-facet");
        facets.forEach(facet => {
            const prop = facet.dataset.prop;
            const isPadding = prop.startsWith("padding");

            // Drag-to-scrub
            facet.addEventListener("mousedown", (e) => {
                if (e.target.tagName === "INPUT") return;
                e.preventDefault();
                e.stopPropagation();

                const startX = e.clientX;
                const valEl = facet.querySelector(".neo-val");
                const startVal = parseFloat(valEl.textContent) || 0;
                document.body.style.cursor = "ew-resize";
                document.body.style.userSelect = "none";

                function onMouseMove(me) {
                    const deltaX = me.clientX - startX;
                    const mult = me.shiftKey ? 10 : (me.altKey ? 0.1 : 1);
                    let newVal = Math.round(startVal + deltaX * mult);
                    if (isPadding && newVal < 0) newVal = 0;

                    valEl.textContent = newVal;

                    if (isLinked) {
                        const prefix = isPadding ? "padding_" : "margin_";
                        ["top", "right", "bottom", "left"].forEach(s => {
                            window.setLocalStyle(prefix + s, newVal + currentUnit);
                            const otherFacet = container.querySelector(`[data-prop="${prefix + s}"] .neo-val`);
                            if (otherFacet) otherFacet.textContent = newVal;
                        });
                    } else {
                        window.setLocalStyle(prop, newVal + currentUnit);
                    }
                }

                function onMouseUp() {
                    document.body.style.cursor = "";
                    document.body.style.userSelect = "";
                    window.removeEventListener("mousemove", onMouseMove);
                    window.removeEventListener("mouseup", onMouseUp);
                    window.pushHistorySnapshot();
                }

                window.addEventListener("mousemove", onMouseMove);
                window.addEventListener("mouseup", onMouseUp);
            });

            // Double click to type
            facet.addEventListener("dblclick", (e) => {
                e.stopPropagation();
                const valEl = facet.querySelector(".neo-val");
                const currentVal = valEl.textContent.trim();
                const input = document.createElement("input");
                input.type = "text";
                input.className = "neo-box-model-inline-input";
                input.value = currentVal;
                valEl.style.display = "none";
                facet.appendChild(input);
                input.focus();
                input.select();

                function commit() {
                    let parsed = parseFloat(input.value) || 0;
                    if (isPadding && parsed < 0) parsed = 0;
                    valEl.textContent = parsed;
                    valEl.style.display = "";
                    input.remove();
                    window.setLocalStyle(prop, parsed + currentUnit);
                    window.pushHistorySnapshot();
                }

                input.onblur = commit;
                input.onkeydown = (ke) => {
                    if (ke.key === "Enter") commit();
                    if (ke.key === "Escape") {
                        valEl.style.display = "";
                        input.remove();
                    }
                };
            });
        });
    }
})();
"###;
