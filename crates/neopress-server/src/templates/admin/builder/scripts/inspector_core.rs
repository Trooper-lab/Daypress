//! Right Webflow-Class Inspector coordinator, Tab Bar, Selector and Pseudo-States (`scripts/inspector_core.rs`).

pub const JS_INSPECTOR_CORE: &str = r###"
(function() {
    function renderInspector() {
        const inspector = document.getElementById("neostudio-inspector");
        if (!inspector) return;

        const state = window.NeoStudioState;
        const id = state.activeElementId;
        const node = id ? window.findNodeById(id) : null;

        if (!node) {
            inspector.innerHTML = `
                <div class="inspector-empty-state">
                    <i class="fa-solid fa-arrow-pointer inspector-empty-icon"></i>
                    <p>Select any block on the canvas or layers navigator to edit styling and settings.</p>
                </div>
            `;
            return;
        }

        // Render Inspector Layout
        inspector.innerHTML = `
            <!-- Tab Bar -->
            <div class="inspector-tab-nav">
                <button class="insp-tab-btn ${state.currentInspectorTab === 'style' ? 'active' : ''}" id="tab-insp-style">
                    <i class="fa-solid fa-paintbrush"></i> Style
                </button>
                <button class="insp-tab-btn ${state.currentInspectorTab === 'settings' ? 'active' : ''}" id="tab-insp-settings">
                    <i class="fa-solid fa-gear"></i> Settings
                </button>
            </div>

            <!-- Selector & Inheritance Bar -->
            <div class="inspector-selector-bar">
                <div class="selector-target-row">
                    <div class="target-badge">
                        <i class="fa-solid fa-cube" style="color:var(--neo-signal);"></i>
                        <span>${node.type}</span>
                        <span class="target-id">#${node.id}</span>
                    </div>
                </div>

                <!-- Classes Manager -->
                <div class="classes-wrapper">
                    ${renderClassPills(node)}
                    <button class="btn-add-class" id="btn-add-class-tag">+ Class</button>
                </div>

                <!-- Pseudo-State Switcher -->
                <div class="state-switcher">
                    <button class="state-btn ${state.pseudoState === 'default' ? 'active' : ''}" data-state="default">Default</button>
                    <button class="state-btn ${state.pseudoState === 'hover' ? 'active' : ''}" data-state="hover">Hover</button>
                    <button class="state-btn ${state.pseudoState === 'focus' ? 'active' : ''}" data-state="focus">Focus</button>
                    <button class="state-btn ${state.pseudoState === 'active' ? 'active' : ''}" data-state="active">Active</button>
                </div>

                <!-- Breakpoint Inheritance Cascade -->
                <div class="breakpoint-indicator-row">
                    <span>Cascade: ${state.currentBreakpoint.toUpperCase()}</span>
                    <div class="bp-cascade-dots">
                        <span class="bp-dot ${state.currentBreakpoint === 'desktop' ? 'active-bp' : ''}" title="Desktop Base"></span>
                        <span class="bp-dot ${state.currentBreakpoint === 'tablet' ? 'active-bp' : ''}" title="Tablet Override"></span>
                        <span class="bp-dot ${state.currentBreakpoint === 'mobile' ? 'active-bp' : ''}" title="Mobile Override"></span>
                    </div>
                </div>
            </div>

            <!-- Scrollable Content Panels -->
            <div id="inspector-panels-scroll" style="flex: 1; overflow-y: auto;"></div>
        `;

        // Tab click listeners
        document.getElementById("tab-insp-style").onclick = () => {
            state.currentInspectorTab = "style";
            renderInspector();
        };
        document.getElementById("tab-insp-settings").onclick = () => {
            state.currentInspectorTab = "settings";
            renderInspector();
        };

        // Pseudo-state buttons
        inspector.querySelectorAll(".state-btn").forEach(btn => {
            btn.onclick = () => {
                const targetState = btn.dataset.state;
                state.pseudoState = targetState;
                const canvasNode = document.querySelector(`[data-node-id="${node.id}"]`);
                if (canvasNode) {
                    canvasNode.classList.toggle("neostudio-force-hover", targetState === "hover");
                }
                renderInspector();
            };
        });

        // Add class button (non-blocking inline CAD input)
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
                const commit = () => {
                    const cls = input.value.trim().replace(/^\.+/, "");
                    if (cls) {
                        if (!node.styles) node.styles = {};
                        const classes = (node.styles.custom_classes || "").split(" ").filter(Boolean);
                        if (!classes.includes(cls)) {
                            classes.push(cls);
                            node.styles.custom_classes = classes.join(" ");
                            window.markDirty(true);
                        }
                    }
                    renderInspector();
                };
                input.onblur = commit;
                input.onkeydown = (ke) => {
                    if (ke.key === "Enter") commit();
                    if (ke.key === "Escape") renderInspector();
                };
            };
        }

        const scrollArea = document.getElementById("inspector-panels-scroll");
        if (state.currentInspectorTab === "style") {
            if (window.renderBoxModelWidget) window.renderBoxModelWidget(scrollArea, node);
            if (window.renderLayoutPanels) window.renderLayoutPanels(scrollArea, node);
            if (window.renderVisualPanels) window.renderVisualPanels(scrollArea, node);
        } else {
            if (window.renderContentSettings) window.renderContentSettings(scrollArea, node);
        }
    }
    window.renderInspector = renderInspector;

    function renderClassPills(node) {
        if (!node.styles || !node.styles.custom_classes) return "";
        const classes = node.styles.custom_classes.split(" ").filter(Boolean);
        return classes.map(c => `
            <span class="class-pill">
                <i class="fa-solid fa-tag" style="font-size:8px;"></i>
                <span>.${c}</span>
                <span class="class-pill-remove" onclick="window.removeClassFromNode('${node.id}', '${c}')">&times;</span>
            </span>
        `).join("");
    }

    function removeClassFromNode(nodeId, cls) {
        const node = window.findNodeById(nodeId);
        if (node && node.styles && node.styles.custom_classes) {
            const classes = node.styles.custom_classes.split(" ").filter(c => c !== cls);
            node.styles.custom_classes = classes.join(" ");
            window.markDirty(true);
            renderInspector();
        }
    }
    window.removeClassFromNode = removeClassFromNode;

    // -------------------------------------------------------------
    // STYLE SETTER (LOCAL & RESPONSIVE & PSEUDO-STATE)
    // -------------------------------------------------------------
    function setLocalStyle(prop, val) {
        const state = window.NeoStudioState;
        const node = state.activeElementId ? window.findNodeById(state.activeElementId) : null;
        if (!node) return;
        if (node.data && node.data._locked) {
            if (window.showToast) window.showToast("Cannot edit styles of locked element", "warning");
            return;
        }
        if (!node.styles) node.styles = {};

        let target = node.styles;
        if (state.pseudoState === "hover") {
            if (!node.styles.hover) node.styles.hover = {};
            target = node.styles.hover;
        } else if (state.currentBreakpoint === "tablet") {
            if (!node.styles.tablet) node.styles.tablet = {};
            target = node.styles.tablet;
        } else if (state.currentBreakpoint === "mobile") {
            if (!node.styles.mobile) node.styles.mobile = {};
            target = node.styles.mobile;
        }

        if (val === "" || val === null || val === undefined) {
            delete target[prop];
        } else {
            target[prop] = val;
        }

        // Direct 60fps DOM update on canvas
        const domEl = document.querySelector(`[data-node-id="${node.id}"]`);
        if (domEl && window.applyNodeStyles) {
            window.applyNodeStyles(domEl, node);
        }

        window.markDirty(true);
        if (window.updateSelectionChrome) window.updateSelectionChrome();
        if (window.updateBoxOverlays) window.updateBoxOverlays(node.id);
    }
    window.setLocalStyle = setLocalStyle;
})();
"###;
