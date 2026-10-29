//! Left Sidebar Custom CSS & Element Classes Drawer (`scripts/css_drawer.rs`).
//!
//! Unified workstation panel in the leftmost sidebar (`data-drawer="css"`) allowing
//! users to edit the live Custom CSS stylesheet AND assign/toggle CSS classes on
//! any element in a single view with 0ms canvas feedback and SQLite persistence.

pub const JS_CSS_DRAWER: &str = r###"
(function() {
    function esc(s) {
        return String(s == null ? "" : s)
            .replace(/&/g, "&amp;")
            .replace(/"/g, "&quot;")
            .replace(/</g, "&lt;");
    }

    function getClassesArray(node) {
        if (!node || !node.styles || !node.styles.css_classes) return [];
        return String(node.styles.css_classes)
            .split(/\s+/)
            .map(c => c.trim().replace(/^\.+/, ""))
            .filter(Boolean);
    }

    function setClassesArray(node, arr) {
        if (!node) return;
        node.styles = node.styles || {};
        const unique = Array.from(new Set(arr.map(c => c.trim().replace(/^\.+/, "")).filter(Boolean)));
        if (unique.length > 0) {
            node.styles.css_classes = unique.join(" ");
        } else {
            delete node.styles.css_classes;
        }
        const el = document.querySelector(`#daystudio-canvas [data-node-id="${node.id}"]`);
        if (el && window.applyNodeStyles) window.applyNodeStyles(el, node);
        if (window.renderInspector) window.renderInspector();
        const state = window.DayStudioState;
        if (state) state.isDirty = true;
    }

    function collectAllNodes(root, list = [], depth = 0) {
        if (!root) return list;
        if (root.id && root.id !== "root") {
            list.push({ node: root, depth });
        }
        for (const child of (root.children || [])) {
            collectAllNodes(child, list, root.id === "root" ? 0 : depth + 1);
        }
        return list;
    }

    function extractStylesheetClasses() {
        const gs = window.ensureGlobalStyles ? window.ensureGlobalStyles() : (window.DayStudioState?.pageAst?.global_styles || {});
        const css = String(gs.custom_css || "");
        const set = new Set();

        // Strip comments before scanning class selectors
        const clean = css.replace(/\/\*[\s\S]*?\*\//g, "");
        const regex = /(?:^|[\s,+~>({]):?\.([a-zA-Z_-][a-zA-Z0-9_-]*)/gm;
        let m;
        while ((m = regex.exec(clean)) !== null) {
            const cls = m[1];
            if (cls && !cls.startsWith("dp-") && !cls.startsWith("daystudio-")) {
                set.add(cls);
            }
        }

        // Also include classes already assigned to any node in the page AST
        const root = window.DayStudioState?.pageAst?.root;
        const walk = (n) => {
            if (!n) return;
            getClassesArray(n).forEach(c => set.add(c));
            (n.children || []).forEach(walk);
        };
        walk(root);

        return Array.from(set).sort();
    }

    function ensureClassRuleInStylesheet(clsName) {
        const cleanCls = String(clsName || "").trim().replace(/^\.+/, "");
        if (!cleanCls) return;
        const gs = window.ensureGlobalStyles ? window.ensureGlobalStyles() : null;
        if (!gs) return;
        const editor = document.getElementById("css-drawer-global-css");
        const currentCss = editor ? editor.value : String(gs.custom_css || "");
        const escaped = cleanCls.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        const ruleRegex = new RegExp(`\\.${escaped}\\b`);

        if (!ruleRegex.test(currentCss)) {
            const prefix = currentCss.trim().length > 0 ? currentCss.trimEnd() + "\n\n" : "";
            const nextCss = `${prefix}.${cleanCls} {\n  \n}\n`;
            gs.custom_css = nextCss;
            if (editor) {
                editor.value = nextCss;
                editor.focus();
                const pos = nextCss.length - 3;
                editor.setSelectionRange(pos, pos);
            }
            if (window.syncGlobalTokensCss) window.syncGlobalTokensCss();
            renderDetectedClassChips();
        } else if (editor) {
            const match = ruleRegex.exec(currentCss);
            editor.focus();
            if (match) {
                editor.setSelectionRange(match.index, match.index + cleanCls.length + 1);
            }
        }
    }

    function renderSelectedElementSection() {
        const box = document.getElementById("css-drawer-selection-box");
        if (!box) return;

        const state = window.DayStudioState;
        const root = state?.pageAst?.root;
        const allNodes = collectAllNodes(root);
        const activeId = state?.activeElementId;
        const activeNode = (activeId && activeId !== "root" && window.findNodeById)
            ? window.findNodeById(activeId)
            : null;
        const activeClasses = getClassesArray(activeNode);

        const optionsHtml = allNodes.map(({ node, depth }) => {
            const indent = "\u00A0\u00A0".repeat(Math.min(depth, 5));
            const label = (node.data && node.data._name) || node.type || "Element";
            const sel = node.id === activeId ? "selected" : "";
            return `<option value="${esc(node.id)}" ${sel}>${indent}${esc(label)} (#${esc(node.id)})</option>`;
        }).join("");

        const pillsHtml = activeNode
            ? (activeClasses.length > 0
                ? activeClasses.map(c => `
                    <span class="css-cls-pill" data-cls="${esc(c)}" title="Click to jump/create .${esc(c)} in CSS" style="display:inline-flex; align-items:center; gap:5px; padding:3px 8px; background:rgba(99,102,241,0.16); border:1px solid rgba(99,102,241,0.4); border-radius:4px; font-family:var(--font-mono, monospace); font-size:11px; color:#e0e7ff; cursor:pointer;">
                        <span>.${esc(c)}</span>
                        <button type="button" class="css-cls-remove" data-cls="${esc(c)}" title="Remove class .${esc(c)}" style="background:none; border:none; color:#94a3b8; cursor:pointer; font-size:11px; padding:0 2px; line-height:1;">&times;</button>
                    </span>
                `).join("")
                : `<div style="font-size:11px; color:var(--cad-text-muted); font-style:italic;">No classes assigned yet. Add one below or click a stylesheet chip.</div>`)
            : `<div style="font-size:11px; color:var(--cad-text-muted);">Select an element on the canvas or from the dropdown above to assign CSS classes.</div>`;

        box.innerHTML = `
            <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:6px;">
                <span style="font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:0.06em; color:var(--cad-text-muted);">Target Element</span>
                ${activeNode ? `<span style="font-size:10px; font-family:var(--font-mono, monospace); color:#818cf8;">${esc(activeNode.type)} #${esc(activeNode.id)}</span>` : ""}
            </div>
            <select class="cad-select" id="css-drawer-node-picker" style="width:100%; height:28px; font-size:11px; margin-bottom:8px;">
                <option value="">— Select Canvas Element —</option>
                ${optionsHtml}
            </select>

            <div style="font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:0.06em; color:var(--cad-text-muted); margin-bottom:5px;">Assigned Element Classes</div>
            <div id="css-drawer-active-pills" style="display:flex; flex-wrap:wrap; gap:5px; min-height:26px; padding:6px; background:rgba(0,0,0,0.2); border:1px solid var(--cad-border); border-radius:4px; margin-bottom:8px;">
                ${pillsHtml}
            </div>

            <div style="display:flex; gap:4px; margin-bottom:8px;">
                <input type="text" class="cad-input" id="css-drawer-class-input" placeholder="e.g. card-editorial hero-cta" ${activeNode ? "" : "disabled"} style="flex:1; height:26px; font-size:11px; font-family:var(--font-mono, monospace);">
                <button type="button" class="cad-btn secondary" id="css-drawer-add-class-btn" ${activeNode ? "" : "disabled"} title="Add class to selected element" style="height:26px; padding:0 8px; font-size:11px;">+ Class</button>
                <button type="button" class="cad-btn primary" id="css-drawer-scaffold-rule-btn" ${activeNode ? "" : "disabled"} title="Add class to element AND create .class { } rule in Custom CSS" style="height:26px; padding:0 8px; font-size:11px;">+ Rule</button>
            </div>

            <details style="margin-top:4px;">
                <summary style="font-size:10px; font-weight:600; color:var(--cad-text-muted); cursor:pointer; user-select:none;">Element Inline CSS Declarations (${activeNode ? "#" + esc(activeNode.id) : "none"})</summary>
                <textarea class="cad-input" id="css-drawer-inline-css" rows="2" ${activeNode ? "" : "disabled"} placeholder="backdrop-filter: blur(8px); cursor: pointer;" style="width:100%; margin-top:5px; font-family:var(--font-mono, monospace); font-size:11px; resize:vertical;">${esc(activeNode?.styles?.custom_css || "")}</textarea>
            </details>
        `;

        const picker = box.querySelector("#css-drawer-node-picker");
        if (picker) {
            picker.onchange = () => {
                const val = picker.value;
                if (val && window.selectElement) {
                    window.selectElement(val);
                }
            };
        }

        const addInput = box.querySelector("#css-drawer-class-input");
        const addBtn = box.querySelector("#css-drawer-add-class-btn");
        const scaffoldBtn = box.querySelector("#css-drawer-scaffold-rule-btn");

        const commitAddClass = (andScaffoldRule) => {
            if (!activeNode || !addInput) return;
            const raw = addInput.value.trim();
            if (!raw) return;
            const added = raw.split(/\s+/).map(c => c.replace(/^\.+/, "").trim()).filter(Boolean);
            if (!added.length) return;
            const merged = [...getClassesArray(activeNode), ...added];
            setClassesArray(activeNode, merged);
            addInput.value = "";
            renderSelectedElementSection();
            renderDetectedClassChips();
            if (andScaffoldRule) {
                ensureClassRuleInStylesheet(added[0]);
            }
        };

        if (addBtn) addBtn.onclick = () => commitAddClass(false);
        if (scaffoldBtn) scaffoldBtn.onclick = () => commitAddClass(true);
        if (addInput) {
            addInput.onkeydown = (e) => {
                if (e.key === "Enter") {
                    e.preventDefault();
                    commitAddClass(e.shiftKey);
                }
            };
        }

        box.querySelectorAll(".css-cls-remove").forEach(btn => {
            btn.onclick = (e) => {
                e.stopPropagation();
                if (!activeNode) return;
                const target = btn.dataset.cls;
                const next = getClassesArray(activeNode).filter(c => c !== target);
                setClassesArray(activeNode, next);
                renderSelectedElementSection();
                renderDetectedClassChips();
            };
        });

        box.querySelectorAll(".css-cls-pill").forEach(pill => {
            pill.onclick = () => {
                ensureClassRuleInStylesheet(pill.dataset.cls);
            };
        });

        const inlineCss = box.querySelector("#css-drawer-inline-css");
        if (inlineCss && activeNode) {
            inlineCss.oninput = () => {
                activeNode.styles = activeNode.styles || {};
                if (inlineCss.value.trim()) {
                    activeNode.styles.custom_css = inlineCss.value;
                } else {
                    delete activeNode.styles.custom_css;
                }
                const el = document.querySelector(`#daystudio-canvas [data-node-id="${activeNode.id}"]`);
                if (el && window.applyNodeStyles) window.applyNodeStyles(el, activeNode);
                if (state) state.isDirty = true;
            };
        }
    }

    function renderDetectedClassChips() {
        const chipContainer = document.getElementById("css-drawer-detected-chips");
        if (!chipContainer) return;

        const classes = extractStylesheetClasses();
        const activeId = window.DayStudioState?.activeElementId;
        const activeNode = (activeId && activeId !== "root" && window.findNodeById)
            ? window.findNodeById(activeId)
            : null;
        const activeSet = new Set(getClassesArray(activeNode));

        if (classes.length === 0) {
            chipContainer.innerHTML = `<span style="font-size:11px; color:var(--cad-text-muted);">Define any <code>.class-name { ... }</code> below and it will appear here as a 1-click toggle chip.</span>`;
            return;
        }

        chipContainer.innerHTML = classes.map(cls => {
            const isOn = activeSet.has(cls);
            const bg = isOn ? "rgba(16,185,129,0.18)" : "rgba(255,255,255,0.04)";
            const border = isOn ? "rgba(16,185,129,0.5)" : "var(--cad-border)";
            const color = isOn ? "#6ee7b7" : "var(--cad-text)";
            const prefix = isOn ? "✓" : "+";
            return `<button type="button" class="css-detected-chip" data-cls="${esc(cls)}" title="${activeNode ? `Click to toggle .${esc(cls)} on #${esc(activeNode.id)}` : `Select an element first to toggle .${esc(cls)}`}" style="padding:2px 7px; border-radius:4px; background:${bg}; border:1px solid ${border}; color:${color}; font-family:var(--font-mono, monospace); font-size:10px; cursor:pointer;">${prefix} .${esc(cls)}</button>`;
        }).join("");

        chipContainer.querySelectorAll(".css-detected-chip").forEach(chip => {
            chip.onclick = () => {
                const cls = chip.dataset.cls;
                if (!activeNode) {
                    ensureClassRuleInStylesheet(cls);
                    if (window.showToast) window.showToast("Select an element on the canvas to toggle ." + cls, "info");
                    return;
                }
                const current = getClassesArray(activeNode);
                const next = current.includes(cls)
                    ? current.filter(c => c !== cls)
                    : [...current, cls];
                setClassesArray(activeNode, next);
                renderSelectedElementSection();
                renderDetectedClassChips();
            };
        });
    }

    function renderCssDrawer(container) {
        if (!container) return;
        const gs = window.ensureGlobalStyles ? window.ensureGlobalStyles() : (window.DayStudioState?.pageAst?.global_styles || {});
        if (window.syncGlobalTokensCss) window.syncGlobalTokensCss();

        container.innerHTML = `
            <div class="drawer-header" style="display:flex; align-items:center; justify-content:space-between;">
                <span class="drawer-title"><i class="fa-solid fa-file-code" style="margin-right:6px; color:#818cf8;"></i>Custom CSS &amp; Classes</span>
                <button type="button" class="cad-btn primary" id="css-drawer-save-top" style="height:24px; padding:0 10px; font-size:11px;">
                    <i class="fa-solid fa-floppy-disk"></i> Save
                </button>
            </div>
            <div class="drawer-scroll-body" style="padding:10px; display:flex; flex-direction:column; gap:10px;">
                <div id="css-drawer-selection-box" style="padding:9px; background:rgba(255,255,255,0.02); border:1px solid var(--cad-border); border-radius:6px;"></div>

                <div style="padding:9px; background:rgba(255,255,255,0.02); border:1px solid var(--cad-border); border-radius:6px;">
                    <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:6px;">
                        <span style="font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:0.06em; color:var(--cad-text-muted);">Stylesheet Classes (1-Click Toggle)</span>
                    </div>
                    <div id="css-drawer-detected-chips" style="display:flex; flex-wrap:wrap; gap:4px; max-height:92px; overflow-y:auto;"></div>
                </div>

                <div style="display:flex; flex-direction:column; flex:1; min-height:240px; padding:9px; background:rgba(255,255,255,0.02); border:1px solid var(--cad-border); border-radius:6px;">
                    <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:6px;">
                        <span style="font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:0.06em; color:var(--cad-text-muted);">Global &amp; Page Custom CSS</span>
                        <div style="display:flex; gap:3px;">
                            <button type="button" class="cad-btn secondary css-snip-btn" data-snip="class" style="height:20px; padding:0 6px; font-size:10px;">+ .class</button>
                            <button type="button" class="cad-btn secondary css-snip-btn" data-snip="hover" style="height:20px; padding:0 6px; font-size:10px;">+ :hover</button>
                            <button type="button" class="cad-btn secondary css-snip-btn" data-snip="media" style="height:20px; padding:0 6px; font-size:10px;">+ @media</button>
                        </div>
                    </div>
                    <textarea class="cad-input" id="css-drawer-global-css" spellcheck="false" placeholder="/* Define reusable classes and custom rules */\n.editorial-card {\n  border: 1px solid var(--rule);\n  padding: 24px;\n}" style="width:100%; flex:1; min-height:220px; font-family:var(--font-mono, monospace); font-size:11px; line-height:1.45; resize:vertical;">${esc(gs.custom_css || "")}</textarea>
                </div>

                <button type="button" class="cad-btn primary" id="css-drawer-save-bottom" style="width:100%; height:32px; font-size:12px;">
                    <i class="fa-solid fa-cloud-arrow-up"></i> Save CSS &amp; Classes to SQLite
                </button>
            </div>
        `;

        renderSelectedElementSection();
        renderDetectedClassChips();

        const globalEditor = container.querySelector("#css-drawer-global-css");
        if (globalEditor) {
            globalEditor.oninput = () => {
                gs.custom_css = globalEditor.value;
                if (window.syncGlobalTokensCss) window.syncGlobalTokensCss();
                if (window.DayStudioState) window.DayStudioState.isDirty = true;
                renderDetectedClassChips();
            };
        }

        container.querySelectorAll(".css-snip-btn").forEach(btn => {
            btn.onclick = () => {
                if (!globalEditor) return;
                const kind = btn.dataset.snip;
                const activeId = window.DayStudioState?.activeElementId;
                const activeNode = (activeId && activeId !== "root" && window.findNodeById) ? window.findNodeById(activeId) : null;
                const firstCls = getClassesArray(activeNode)[0] || "custom-block";
                let snippet = "";
                if (kind === "class") snippet = `.${firstCls} {\n  \n}\n`;
                else if (kind === "hover") snippet = `.${firstCls}:hover {\n  transform: translateY(-2px);\n}\n`;
                else if (kind === "media") snippet = `@media (max-width: 768px) {\n  .${firstCls} {\n    \n  }\n}\n`;
                const cur = globalEditor.value.trimEnd();
                globalEditor.value = (cur ? cur + "\n\n" : "") + snippet;
                gs.custom_css = globalEditor.value;
                if (window.syncGlobalTokensCss) window.syncGlobalTokensCss();
                renderDetectedClassChips();
                globalEditor.focus();
            };
        });

        const saveHandler = () => {
            if (window.pushHistorySnapshot) window.pushHistorySnapshot();
            if (window.saveCurrentLayout) window.saveCurrentLayout();
        };
        const saveTop = container.querySelector("#css-drawer-save-top");
        if (saveTop) saveTop.onclick = saveHandler;
        const saveBot = container.querySelector("#css-drawer-save-bottom");
        if (saveBot) saveBot.onclick = saveHandler;
    }

    function refreshCssDrawerSelection() {
        const state = window.DayStudioState;
        if (!state || state.activeDrawer !== "css" || !state.drawerOpen) return;
        renderSelectedElementSection();
        renderDetectedClassChips();
    }

    window.renderCssDrawer = renderCssDrawer;
    window.refreshCssDrawerSelection = refreshCssDrawerSelection;
})();
"###;
