//! Dynamic & Theme/Form Pro Blocks Canvas DOM Builders — Part 3: Symbols Left Drawer & Instance Overrides Inspector (`scripts/canvas_pro_symbols_drawer.rs`).

pub const JS_CANVAS_PRO_SYMBOLS_DRAWER: &str = r###"
    function renderSymbolsLibraryCardHtml() {
        const cardsHtml = symbolsList.length === 0
            ? `<div style="padding:12px;border:1px dashed rgba(148,163,184,0.35);border-radius:6px;font-size:11.5px;color:#94a3b8;text-align:center;">
                No Global Symbols yet. Select any section on the canvas and click <strong>+ Save Selected Section as Global Symbol</strong> above.
               </div>`
            : symbolsList.map(function(sym) {
                return `
                    <div class="dp-symbol-lib-item" style="background:rgba(15,23,42,0.55);border:1px solid rgba(124,58,237,0.35);border-radius:7px;padding:10px;display:flex;flex-direction:column;gap:8px;">
                        <div style="display:flex;justify-content:space-between;align-items:center;gap:6px;">
                            <div style="display:flex;align-items:center;gap:6px;min-width:0;">
                                <i class="fa-solid fa-cubes" style="color:#a78bfa;font-size:12px;"></i>
                                <strong style="font-size:12px;color:#f8fafc;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${escHtml(sym.title)}</strong>
                            </div>
                            <span style="font-size:10px;font-family:monospace;color:#a78bfa;background:rgba(124,58,237,0.18);padding:1px 6px;border-radius:4px;">#${sym.id}</span>
                        </div>
                        <div style="display:flex;gap:6px;">
                            <button type="button" class="cad-btn primary dp-btn-insert-symbol" data-symbol-id="${sym.id}" style="flex:1;height:26px;font-size:11px;background:#7c3aed;border-color:#7c3aed;color:#fff;cursor:pointer;border-radius:5px;">
                                <i class="fa-solid fa-plus"></i> Insert Symbol
                            </button>
                            <a href="/d-admin/builder?template_id=${sym.id}" class="cad-btn secondary" style="height:26px;padding:0 8px;font-size:11px;display:inline-flex;align-items:center;text-decoration:none;border-radius:5px;" title="Edit Master Symbol">
                                <i class="fa-solid fa-pen-to-square"></i>
                            </a>
                        </div>
                    </div>
                `;
            }).join("");

        return `
            <div id="dp-symbols-library-section" style="margin-top:14px;padding:12px;border-top:1px solid rgba(148,163,184,0.2);">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">
                    <span style="font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:0.05em;color:#a78bfa;">
                        <i class="fa-solid fa-cubes" style="margin-right:5px;"></i>Synced Symbols (${symbolsList.length})
                    </span>
                    <button type="button" id="dp-btn-refresh-symbols" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:11px;" title="Refresh Symbols from SQLite">
                        <i class="fa-solid fa-rotate"></i>
                    </button>
                </div>
                <button type="button" id="dp-btn-save-section-symbol" class="cad-btn primary" style="width:100%;height:30px;font-size:11.5px;font-weight:600;margin-bottom:10px;background:linear-gradient(135deg,#7c3aed,#4f46e5);border:none;color:#fff;border-radius:6px;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:6px;">
                    <i class="fa-solid fa-cubes"></i> + Save Selected Section as Global Symbol
                </button>
                <div id="dp-symbols-drawer-list" style="display:flex;flex-direction:column;gap:8px;">
                    ${cardsHtml}
                </div>
            </div>
        `;
    }

    function bindSymbolsLibraryEvents(rootContainer) {
        if (!rootContainer) return;
        const saveBtn = rootContainer.querySelector("#dp-btn-save-section-symbol");
        if (saveBtn && !saveBtn.dataset.bound) {
            saveBtn.dataset.bound = "1";
            saveBtn.addEventListener("click", function(e) {
                e.preventDefault();
                saveSelectedSectionAsGlobalSymbol();
            });
        }
        const refreshBtn = rootContainer.querySelector("#dp-btn-refresh-symbols");
        if (refreshBtn && !refreshBtn.dataset.bound) {
            refreshBtn.dataset.bound = "1";
            refreshBtn.addEventListener("click", async function(e) {
                e.preventDefault();
                await fetchSectionSymbols(true);
                renderSymbolsDrawerContent();
            });
        }
        rootContainer.querySelectorAll(".dp-btn-insert-symbol").forEach(function(btn) {
            if (btn.dataset.bound) return;
            btn.dataset.bound = "1";
            btn.addEventListener("click", function(e) {
                e.preventDefault();
                const symId = btn.getAttribute("data-symbol-id");
                insertSymbolInstanceOntoCanvas(symId);
            });
        });
    }

    function renderSymbolsDrawerContent() {
        const drawerPanel = document.getElementById("drawer-panel-container");
        if (!drawerPanel) return;
        const symbolsRailBtn = document.getElementById("rail-btn-symbols");
        if (symbolsRailBtn && symbolsRailBtn.classList.contains("active")) {
            drawerPanel.innerHTML = `
                <div class="drawer-header" style="padding:12px 14px;border-bottom:1px solid rgba(148,163,184,0.18);display:flex;justify-content:space-between;align-items:center;">
                    <strong style="font-size:12.5px;text-transform:uppercase;letter-spacing:0.05em;">Synced Global Symbols</strong>
                    <span style="font-size:10.5px;color:#a78bfa;font-family:monospace;">0.5 Reusable AST</span>
                </div>
                ${renderSymbolsLibraryCardHtml()}
            `;
            bindSymbolsLibraryEvents(drawerPanel);
            return;
        }

        const existingSec = drawerPanel.querySelector("#dp-symbols-library-section");
        if (existingSec) {
            const temp = document.createElement("div");
            temp.innerHTML = renderSymbolsLibraryCardHtml();
            const newSec = temp.firstElementChild;
            existingSec.replaceWith(newSec);
            bindSymbolsLibraryEvents(drawerPanel);
        }
    }

    function enhanceLeftDrawerPalette() {
        if (isEnhancingDrawer) return;
        const drawerPanel = document.getElementById("drawer-panel-container");
        if (!drawerPanel) return;

        const elementsRailBtn = document.getElementById("rail-btn-elements");
        const templatesRailBtn = document.getElementById("rail-btn-templates");
        const isElementsActive = !elementsRailBtn || elementsRailBtn.classList.contains("active");
        const isTemplatesActive = templatesRailBtn && templatesRailBtn.classList.contains("active");

        if (!isElementsActive && !isTemplatesActive) return;

        isEnhancingDrawer = true;
        try {
            if (isElementsActive && !drawerPanel.querySelector('[data-type="SymbolInstance"], [data-widget="SymbolInstance"], #dp-tile-symbol-instance')) {
                const grids = drawerPanel.querySelectorAll(".elements-grid, .widget-grid, .palette-grid");
                const targetGrid = grids.length > 0 ? grids[grids.length - 1] : null;
                if (targetGrid) {
                    const sampleTile = targetGrid.firstElementChild;
                    const tile = document.createElement("div");
                    tile.id = "dp-tile-symbol-instance";
                    tile.className = sampleTile ? sampleTile.className : "element-card";
                    tile.setAttribute("draggable", "true");
                    tile.setAttribute("data-type", "SymbolInstance");
                    tile.setAttribute("data-kind", "SymbolInstance");
                    tile.setAttribute("data-widget", "SymbolInstance");
                    tile.title = "Insert Synced Global Symbol (SymbolInstance)";
                    tile.innerHTML = `
                        <i class="fa-solid fa-cubes" style="color:#a78bfa;"></i>
                        <span>SymbolInstance</span>
                    `;
                    tile.addEventListener("dragstart", function(e) {
                        if (e.dataTransfer) {
                            e.dataTransfer.setData("text/plain", "SymbolInstance");
                            e.dataTransfer.setData("application/daystudio-widget", "SymbolInstance");
                        }
                    });
                    tile.addEventListener("click", function() {
                        insertSymbolInstanceOntoCanvas(symbolsList[0] ? symbolsList[0].id : 0);
                    });
                    targetGrid.appendChild(tile);
                }
            }

            if (!drawerPanel.querySelector("#dp-symbols-library-section")) {
                const wrapper = document.createElement("div");
                wrapper.innerHTML = renderSymbolsLibraryCardHtml();
                if (wrapper.firstElementChild) {
                    drawerPanel.appendChild(wrapper.firstElementChild);
                    bindSymbolsLibraryEvents(drawerPanel);
                }
            }
        } finally {
            isEnhancingDrawer = false;
        }
    }

    function ensureDefaultNavMenuItems(node) {
        if (!node.data) node.data = {};
        let items = Array.isArray(node.items) ? node.items : (Array.isArray(node.data.items) ? node.data.items : null);
        if (!items || items.length === 0) {
            items = [
                { id: "nav-1", label: "Home", url: "/", icon: "", badge: "", description: "", mega_menu_template_id: null },
                { id: "nav-2", label: "Products", url: "/products", icon: "fa-solid fa-cubes", badge: "0.5", description: "Dynamic Templates & Symbols", mega_menu_template_id: null },
                { id: "nav-3", label: "Pricing", url: "/pricing", icon: "", badge: "", description: "", mega_menu_template_id: null },
                { id: "nav-4", label: "Documentation", url: "/docs", icon: "", badge: "", description: "", mega_menu_template_id: null },
                { id: "nav-5", label: "Blog", url: "/blog", icon: "", badge: "", description: "", mega_menu_template_id: null }
            ];
        }
        node.items = items;
        node.data.items = items;
        return items;
    }

    function enhanceInspectorPanel() {
        if (isEnhancingInspector) return;
        const inspector = document.getElementById("daystudio-inspector");
        if (!inspector) return;

        const selNode = getSelectedNode();
        const selType = selNode ? (selNode.type || selNode.kind || "") : "";

        isEnhancingInspector = true;
        try {
            const oldBox = inspector.querySelector("#dp-pro-inspector-extensions");
            if (oldBox) oldBox.remove();

            if (!selNode || selNode.id === "root" || selType === "Root") {
                return;
            }

            const extContainer = document.createElement("div");
            extContainer.id = "dp-pro-inspector-extensions";
            extContainer.style.cssText = "padding:12px;border-top:1px solid rgba(148,163,184,0.2);display:flex;flex-direction:column;gap:14px;";

            let htmlParts = [];

            if (selType === "SymbolInstance") {
                const currentSymId = Number(selNode.symbol_id ?? (selNode.data && selNode.data.symbol_id) ?? 0);
                const overrides = selNode.overrides || (selNode.data && selNode.data.overrides) || {};
                const symTpl = symbolsCache[currentSymId] || symbolsList.find(function(s) { return Number(s.id) === currentSymId; });

                let masterChildren = [];
                if (symTpl && symTpl.layout) {
                    const layoutObj = typeof symTpl.layout === "string" ? JSON.parse(symTpl.layout) : symTpl.layout;
                    if (layoutObj && layoutObj.root) {
                        masterChildren = collectOverridableSymbolChildren(layoutObj.root, []);
                    }
                }
                Object.keys(overrides).forEach(function(cid) {
                    if (!masterChildren.some(function(mc) { return mc.id === cid; })) {
                        masterChildren.push({ id: cid, type: "Custom", text: "", src: "", alt: "", url: "" });
                    }
                });

                const symbolSelectOptions = `<option value="0">-- Select Master Global Symbol --</option>` +
                    symbolsList.map(function(s) {
                        const sel = Number(s.id) === currentSymId ? "selected" : "";
                        return `<option value="${s.id}" ${sel}>#${s.id} — ${escHtml(s.title)}</option>`;
                    }).join("");

                const childOverridesHtml = masterChildren.length === 0
                    ? `<div style="font-size:11px;color:#94a3b8;padding:8px;background:rgba(15,23,42,0.4);border-radius:5px;">
                        Add a child node ID below to override <code>text</code>, <code>src</code>, <code>alt</code>, or <code>url</code> for this instance.
                       </div>`
                    : masterChildren.map(function(child) {
                        const curOv = overrides[child.id] || {};
                        return `
                            <div class="dp-sym-override-card" data-child-id="${escHtml(child.id)}" style="background:rgba(15,23,42,0.45);border:1px solid rgba(124,58,237,0.3);border-radius:6px;padding:8px;display:flex;flex-direction:column;gap:6px;">
                                <div style="display:flex;justify-content:space-between;align-items:center;">
                                    <span style="font-size:11px;font-weight:700;color:#c4b5fd;">${escHtml(child.type)} <code>#${escHtml(child.id)}</code></span>
                                </div>
                                <input type="text" class="cad-input dp-ov-input dp-ignore-dynamic-target" data-child-id="${escHtml(child.id)}" data-field="text" placeholder="Override text (${escHtml(child.text || 'none')})" value="${escHtml(curOv.text || '')}" style="width:100%;font-size:11.5px;padding:4px 7px;" />
                                <input type="text" class="cad-input dp-ov-input dp-ignore-dynamic-target" data-child-id="${escHtml(child.id)}" data-field="url" placeholder="Override URL (${escHtml(child.url || 'none')})" value="${escHtml(curOv.url || '')}" style="width:100%;font-size:11.5px;padding:4px 7px;" />
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:6px;">
                                    <input type="text" class="cad-input dp-ov-input dp-ignore-dynamic-target" data-child-id="${escHtml(child.id)}" data-field="src" placeholder="Image src" value="${escHtml(curOv.src || '')}" style="width:100%;font-size:11px;padding:4px 7px;" />
                                    <input type="text" class="cad-input dp-ov-input dp-ignore-dynamic-target" data-child-id="${escHtml(child.id)}" data-field="alt" placeholder="Image alt" value="${escHtml(curOv.alt || '')}" style="width:100%;font-size:11px;padding:4px 7px;" />
                                </div>
                            </div>
                        `;
                    }).join("");

                htmlParts.push(`
                    <div id="dp-inspector-symbol-overrides" style="background:rgba(124,58,237,0.08);border:1px solid rgba(124,58,237,0.35);border-radius:8px;padding:10px;display:flex;flex-direction:column;gap:8px;">
                        <div style="display:flex;justify-content:space-between;align-items:center;">
                            <strong style="font-size:11.5px;text-transform:uppercase;letter-spacing:0.04em;color:#c4b5fd;">
                                <i class="fa-solid fa-cubes" style="margin-right:5px;"></i>Instance Overrides
                            </strong>
                            <span style="font-size:10px;color:#a78bfa;font-family:monospace;">Symbol #${currentSymId}</span>
                        </div>
                        <div>
                            <label style="display:block;font-size:11px;color:#94a3b8;margin-bottom:4px;">Linked Master Symbol</label>
                            <select id="dp-select-master-symbol" class="cad-input dp-ignore-dynamic-target" style="width:100%;font-size:11.5px;padding:5px;">
                                ${symbolSelectOptions}
                            </select>
                        </div>
                        <div style="display:flex;flex-direction:column;gap:6px;">
                            ${childOverridesHtml}
                        </div>
                        <div style="display:flex;gap:6px;">
                            <input type="text" id="dp-new-override-child-id" class="cad-input dp-ignore-dynamic-target" placeholder="Child Node ID (e.g. heading-1)" style="flex:1;font-size:11px;padding:4px 7px;" />
                            <button type="button" id="dp-btn-add-override-child" class="cad-btn secondary" style="font-size:11px;padding:0 8px;">+ Target</button>
                        </div>
                        <button type="button" id="btn-detach-symbol-instance" class="cad-btn secondary" style="width:100%;height:28px;font-size:11.5px;font-weight:600;color:#fca5a5;border-color:rgba(248,113,113,0.4);cursor:pointer;display:flex;align-items:center;justify-content:center;gap:6px;">
                            <i class="fa-solid fa-link-slash"></i> Detach Instance
                        </button>
                    </div>
                `);
            } else {
                htmlParts.push(`
                    <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;">
                        <button type="button" id="dp-inspector-save-as-symbol" class="cad-btn secondary" style="width:100%;height:28px;font-size:11px;display:flex;align-items:center;justify-content:center;gap:6px;cursor:pointer;">
                            <i class="fa-solid fa-cubes" style="color:#a78bfa;"></i> + Save Selected Section as Global Symbol
                        </button>
                    </div>
                `);
            }
"###;
