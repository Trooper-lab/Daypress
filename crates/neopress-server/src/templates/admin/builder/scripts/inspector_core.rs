//! Right Webflow-Class Inspector coordinator, Tab Bar, Selector, Pseudo-States and the
//! layer-aware style mutation API (`scripts/inspector_core.rs`).
//!
//! Style layers: base (desktop) -> `tablet` -> `mobile` breakpoint overrides, plus `hover`,
//! `focus` and `active` pseudo-state layers (pseudo-states apply on every breakpoint).

pub const JS_INSPECTOR_CORE: &str = r###"
(function() {
    const BP_LABEL = { desktop: "Desktop", tablet: "Tablet", mobile: "Mobile" };

    function hasKeys(o) { return !!(o && typeof o === "object" && Object.keys(o).length); }

    function renderInspector() {
        const inspector = document.getElementById("neostudio-inspector");
        if (!inspector) return;

        const state = window.NeoStudioState;
        const id = state.activeElementId;
        const node = id ? window.findNodeById(id) : null;

        if (window.applyForcedState) window.applyForcedState();

        if (!node) {
            inspector.innerHTML = `
                <div class="inspector-empty-state">
                    <i class="fa-solid fa-arrow-pointer inspector-empty-icon"></i>
                    <p>Select any block on the canvas or layers navigator to edit styling and settings.</p>
                </div>
            `;
            return;
        }

        // Preserve scroll position across re-renders of the same element
        const prevScroll = document.getElementById("inspector-panels-scroll");
        const keepScroll = prevScroll && inspector.dataset.nodeId === node.id ? prevScroll.scrollTop : 0;
        inspector.dataset.nodeId = node.id;

        const styles = node.styles || {};
        const layerKey = getEditLayerKey();
        const layerInfo = describeLayer(layerKey);
        const layerCount = hasKeys(getLayer(node, false)) ? Object.keys(getLayer(node, false)).filter(k => !window.NeoStyle.LAYER_KEYS.includes(k)).length : 0;
        const stateDot = (s) => (s !== "default" && hasKeys(styles[s])) ? '<span class="state-dot"></span>' : "";

        inspector.innerHTML = `
            <div class="inspector-tab-nav">
                <button class="insp-tab-btn ${state.currentInspectorTab === 'style' ? 'active' : ''}" id="tab-insp-style">
                    <i class="fa-solid fa-paintbrush"></i> Style
                </button>
                <button class="insp-tab-btn ${state.currentInspectorTab === 'settings' ? 'active' : ''}" id="tab-insp-settings">
                    <i class="fa-solid fa-gear"></i> Settings
                </button>
            </div>

            <div class="inspector-selector-bar">
                <div class="selector-target-row">
                    <div class="target-badge">
                        <i class="fa-solid fa-cube" style="color:var(--neo-signal);"></i>
                        <span>${node.type}</span>
                        <span class="target-id">#${node.id}</span>
                    </div>
                </div>

                <div class="classes-wrapper">
                    ${renderClassPills(node)}
                    <button class="btn-add-class" id="btn-add-class-tag">+ Class</button>
                </div>

                ${state.currentInspectorTab === 'style' ? `
                <div class="state-switcher">
                    ${["default", "hover", "focus", "active"].map(s => `
                        <button class="state-btn ${state.pseudoState === s ? 'active' : ''}" data-state="${s}">${s.charAt(0).toUpperCase() + s.slice(1)}${stateDot(s)}</button>
                    `).join("")}
                </div>

                <div class="breakpoint-indicator-row">
                    <span class="layer-info ${layerKey ? 'is-override' : ''}" title="${layerInfo.hint}">${layerInfo.label}</span>
                    <div class="bp-cascade-dots">
                        ${["desktop", "tablet", "mobile"].map(bp => {
                            const hasOv = bp === "desktop" ? false : hasKeys(styles[bp]);
                            return `<span class="bp-dot ${state.currentBreakpoint === bp ? 'active-bp' : ''} ${hasOv ? 'has-override' : ''}" data-bp="${bp}" title="${BP_LABEL[bp]}${hasOv ? ' (has overrides)' : ''}"></span>`;
                        }).join("")}
                    </div>
                    ${layerCount ? `<button class="btn-layer-reset" id="btn-reset-layer" title="Clear all ${layerCount} style(s) set on this layer">Reset ${layerCount}</button>` : ""}
                </div>` : ""}
            </div>

            <div id="inspector-panels-scroll" style="flex: 1; overflow-y: auto;"></div>
        `;

        document.getElementById("tab-insp-style").onclick = () => { state.currentInspectorTab = "style"; renderInspector(); };
        document.getElementById("tab-insp-settings").onclick = () => { state.currentInspectorTab = "settings"; renderInspector(); };

        inspector.querySelectorAll(".state-btn").forEach(btn => {
            btn.onclick = () => {
                state.pseudoState = btn.dataset.state;
                renderInspector();
            };
        });

        inspector.querySelectorAll(".bp-dot[data-bp]").forEach(dot => {
            dot.onclick = () => { if (window.setBreakpoint) window.setBreakpoint(dot.dataset.bp); };
        });

        const btnReset = document.getElementById("btn-reset-layer");
        if (btnReset) btnReset.onclick = () => clearLayerStyles();

        const btnAddClass = document.getElementById("btn-add-class-tag");
        if (btnAddClass) {
            btnAddClass.onclick = () => {
                const input = document.createElement("input");
                input.type = "text";
                input.className = "cad-input font-mono";
                input.style.cssText = "width:96px;height:20px;font-size:10.5px;padding:0 4px;";
                input.placeholder = "class-name";
                btnAddClass.replaceWith(input);
                input.focus();
                let done = false;
                const commit = () => {
                    if (done) return;
                    done = true;
                    const added = input.value.trim().split(/\s+/).map(c => c.replace(/^\.+/, "")).filter(Boolean);
                    if (added.length) {
                        if (!node.styles) node.styles = {};
                        const classes = new Set(String(node.styles.css_classes || "").split(/\s+/).filter(Boolean));
                        added.forEach(c => classes.add(c));
                        node.styles.css_classes = Array.from(classes).join(" ");
                        refreshNode(node);
                        window.pushHistorySnapshot();
                    }
                    renderInspector();
                };
                input.onblur = commit;
                input.onkeydown = (ke) => {
                    if (ke.key === "Enter") commit();
                    if (ke.key === "Escape") { done = true; renderInspector(); }
                };
            };
        }

        const scrollArea = document.getElementById("inspector-panels-scroll");
        if (state.currentInspectorTab === "style") {
            if (window.renderBoxModelWidget) window.renderBoxModelWidget(scrollArea, node);
            if (window.renderLayoutPanels) window.renderLayoutPanels(scrollArea, node);
            if (window.renderVisualPanels) window.renderVisualPanels(scrollArea, node);
            if (window.renderEffectsPanels) window.renderEffectsPanels(scrollArea, node);
            if (node.data && node.data._locked) {
                scrollArea.querySelectorAll("input, select, textarea, button").forEach(c => { c.disabled = true; });
            }
        } else {
            if (window.renderContentSettings) window.renderContentSettings(scrollArea, node);
        }
        scrollArea.scrollTop = keepScroll;
    }
    window.renderInspector = renderInspector;

    function describeLayer(layerKey) {
        const bp = window.NeoStudioState.currentBreakpoint;
        switch (layerKey) {
            case "hover": case "focus": case "active":
                return { label: `${layerKey.toUpperCase()} state · all breakpoints`, hint: `Styles apply on :${layerKey} and override every breakpoint` };
            case "tablet":
                return { label: "TABLET override · ≤1024px", hint: "Inherits Desktop; values set here apply to tablet and mobile" };
            case "mobile":
                return { label: "MOBILE override · ≤767px", hint: "Inherits Tablet → Desktop; values set here apply to mobile only" };
            default:
                return { label: `${BP_LABEL[bp] || "Desktop"} · base styles`, hint: "Base styles cascade down to tablet and mobile" };
        }
    }

    function renderClassPills(node) {
        const raw = node.styles && (node.styles.css_classes || node.styles.custom_classes);
        if (!raw) return "";
        return String(raw).split(/\s+/).filter(Boolean).map(c => `
            <span class="class-pill">
                <i class="fa-solid fa-tag" style="font-size:8px;"></i>
                <span>.${c}</span>
                <span class="class-pill-remove" data-cls="${c}" onclick="window.removeClassFromNode('${node.id}', this.dataset.cls)">&times;</span>
            </span>
        `).join("");
    }

    function removeClassFromNode(nodeId, cls) {
        const node = window.findNodeById(nodeId);
        if (!node || !node.styles) return;
        const classes = String(node.styles.css_classes || "").split(/\s+/).filter(c => c && c !== cls);
        if (classes.length) node.styles.css_classes = classes.join(" "); else delete node.styles.css_classes;
        refreshNode(node);
        window.pushHistorySnapshot();
        renderInspector();
    }
    window.removeClassFromNode = removeClassFromNode;

    // -------------------------------------------------------------
    // STYLE LAYER API
    // -------------------------------------------------------------
    function getEditLayerKey() {
        const state = window.NeoStudioState;
        if (state.pseudoState && state.pseudoState !== "default") return state.pseudoState;
        if (state.currentBreakpoint === "tablet" || state.currentBreakpoint === "mobile") return state.currentBreakpoint;
        return null;
    }
    window.getEditLayerKey = getEditLayerKey;

    function getLayer(node, create) {
        if (!node) return null;
        if (!node.styles) { if (!create) return null; node.styles = {}; }
        const key = getEditLayerKey();
        if (!key) return node.styles;
        if (!node.styles[key] && create) node.styles[key] = {};
        return node.styles[key] || null;
    }
    window.getStyleLayer = getLayer;

    function isSet(v) { return v !== undefined && v !== null && String(v).trim() !== ""; }

    /** Value explicitly set on the layer currently being edited. */
    function getLayerValue(node, prop) {
        const layer = getLayer(node, false);
        return layer && isSet(layer[prop]) ? String(layer[prop]) : "";
    }
    window.getLayerValue = getLayerValue;

    /** Value the element would get from other layers if the current layer did not set it. */
    function getInheritedValue(node, prop) {
        const s = (node && node.styles) || {};
        const key = getEditLayerKey();
        const bp = window.NeoStudioState.currentBreakpoint;
        let chain;
        if (key === "tablet") chain = [s];
        else if (key === "mobile") chain = [s.tablet, s];
        else if (key) chain = bp === "mobile" ? [s.mobile, s.tablet, s] : bp === "tablet" ? [s.tablet, s] : [s];
        else chain = [];
        for (const layer of chain) {
            if (layer && isSet(layer[prop])) return String(layer[prop]);
        }
        return "";
    }
    window.getInheritedValue = getInheritedValue;

    function refreshNode(node) {
        const domEl = document.querySelector(`#neostudio-canvas [data-node-id="${node.id}"]`);
        if (domEl && window.applyNodeStyles) window.applyNodeStyles(domEl, node);
        if (window.applyForcedState) window.applyForcedState();
        if (window.updateSelectionChrome) window.updateSelectionChrome();
        if (window.updateBoxOverlays) window.updateBoxOverlays(node.id);
    }
    window.refreshNodeStyles = refreshNode;

    function activeEditableNode() {
        const state = window.NeoStudioState;
        const node = state.activeElementId ? window.findNodeById(state.activeElementId) : null;
        if (!node) return null;
        if (node.data && node.data._locked) {
            if (window.showToast) window.showToast("Cannot edit styles of locked element", "warning");
            return null;
        }
        return node;
    }

    function writeProp(node, prop, val) {
        const key = getEditLayerKey();
        const layer = getLayer(node, true);
        if (!isSet(val)) delete layer[prop]; else layer[prop] = String(val).trim();
        if (key && !hasKeys(node.styles[key])) delete node.styles[key];
    }

    /**
     * Sets (or clears, when `val` is empty) a style property on the current edit layer and
     * live-updates the canvas. Pass `{ commit: true }` to also record an undo snapshot.
     */
    function setLocalStyle(prop, val, opts) {
        const node = activeEditableNode();
        if (!node) return;
        writeProp(node, prop, val);
        refreshNode(node);
        window.markDirty(true);
        if (opts && opts.commit) window.pushHistorySnapshot();
    }
    window.setLocalStyle = setLocalStyle;

    function setLocalStyles(map, opts) {
        const node = activeEditableNode();
        if (!node) return;
        Object.keys(map).forEach(k => writeProp(node, k, map[k]));
        refreshNode(node);
        window.markDirty(true);
        if (opts && opts.commit) window.pushHistorySnapshot();
    }
    window.setLocalStyles = setLocalStyles;

    function commitStyleChange() {
        window.pushHistorySnapshot();
    }
    window.commitStyleChange = commitStyleChange;

    function clearLayerStyles() {
        const node = activeEditableNode();
        if (!node || !node.styles) return;
        const key = getEditLayerKey();
        if (key) {
            delete node.styles[key];
        } else {
            const keep = {};
            window.NeoStyle.LAYER_KEYS.forEach(k => { if (node.styles[k]) keep[k] = node.styles[k]; });
            ["css_classes", "hide_desktop", "hide_tablet", "hide_mobile", "link_url", "raw_settings"].forEach(k => {
                if (node.styles[k] !== undefined) keep[k] = node.styles[k];
            });
            node.styles = keep;
        }
        refreshNode(node);
        window.pushHistorySnapshot();
        renderInspector();
    }
    window.clearLayerStyles = clearLayerStyles;
})();
"###;
