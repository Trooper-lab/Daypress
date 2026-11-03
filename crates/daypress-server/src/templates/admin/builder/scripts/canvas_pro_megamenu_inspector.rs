//! DayStudio 0.5 Visual Mega Menu Builder & NavMenu Inspector (`scripts/canvas_pro_megamenu_inspector.rs`).

pub const JS_CANVAS_PRO_MEGAMENU_INSPECTOR: &str = r###"
(function() {
    function uid(prefix) {
        return window.generateId
            ? window.generateId(prefix)
            : (prefix + "_" + Math.random().toString(36).slice(2, 8));
    }

    function ensureMegaMenuContainerForItem(navNode, idx, presetKey) {
        if (!navNode || !navNode.data || !Array.isArray(navNode.data.items)) return null;
        const item = navNode.data.items[idx];
        if (!item) return null;
        if (!Array.isArray(navNode.children)) navNode.children = [];

        let megaNode = item.mega_menu_node_id
            ? navNode.children.find(c => c.id === item.mega_menu_node_id)
            : null;

        if (!megaNode) {
            const newId = uid("mega_sec");
            megaNode = {
                id: newId,
                type: "MegaMenuContainer",
                data: { trigger: "hover", width: item.mega_menu_width || "980px", label: (item.title || "Item") + " Mega Menu" },
                styles: {
                    background_color: "#0f172a",
                    padding_top: "22px",
                    padding_right: "22px",
                    padding_bottom: "22px",
                    padding_left: "22px",
                    border_radius: "12px"
                },
                children: []
            };
            navNode.children.push(megaNode);
            item.mega_menu_node_id = newId;
            if (!item.mega_menu_width) item.mega_menu_width = "980px";
        }

        const buildPreset = window.buildMegaMenuPresetChildren || (() => []);
        if (presetKey === "from_symbol" && Number(item.mega_menu_template_id || 0) > 0) {
            const sym = (window._dpSymbolTemplatesCache || []).find(s => Number(s.id) === Number(item.mega_menu_template_id));
            if (sym && sym.layout && sym.layout.root && Array.isArray(sym.layout.root.children)) {
                megaNode.children = JSON.parse(JSON.stringify(sym.layout.root.children));
            } else if (!megaNode.children || megaNode.children.length === 0) {
                megaNode.children = buildPreset("saas_3col", item.title);
            }
        } else if (presetKey || !megaNode.children || megaNode.children.length === 0) {
            megaNode.children = buildPreset(presetKey || "saas_3col", item.title);
        }

        window._activeMegaMenuState = { navNodeId: navNode.id, itemIndex: idx };
        if (window.markDirty) window.markDirty(true);
        if (window.pushHistorySnapshot) window.pushHistorySnapshot();
        return megaNode;
    }
    window.ensureMegaMenuContainerForItem = ensureMegaMenuContainerForItem;

    function addMegaMenuColumnToItem(navNode, idx) {
        const megaNode = ensureMegaMenuContainerForItem(navNode, idx, null);
        if (!megaNode) return;
        const makeCard = window.makeMegaMenuFeatureCard || (() => ({ id: uid("t"), type: "Text", data: { content_html: "<p>Item</p>" }, styles: {}, children: [] }));
        const newCol = {
            id: uid("col"),
            type: "Column",
            data: {},
            styles: { gap: "10px", padding_top: "12px", padding_right: "12px", padding_bottom: "12px", padding_left: "12px", border_radius: "8px", background_color: "rgba(255,255,255,0.03)" },
            children: [
                {
                    id: uid("h"),
                    type: "Heading",
                    data: { text: "New Menu Column", level: 5 },
                    styles: { text_color: "#f8fafc", font_size: "14px", font_weight: "700" },
                    children: []
                },
                makeCard("fa-solid fa-star", "Featured Item", "Click to edit or drag blocks into this column", "#")
            ]
        };

        const firstGrid = (megaNode.children || []).find(c => c.type === "Grid" || c.type === "Flex" || c.type === "Container");
        if (firstGrid && Array.isArray(firstGrid.children)) {
            firstGrid.children.push(newCol);
            const colCount = firstGrid.children.length;
            if (firstGrid.type === "Grid") {
                if (!firstGrid.styles) firstGrid.styles = {};
                firstGrid.styles.grid_template_columns = `repeat(${Math.min(colCount, 6)}, 1fr)`;
            }
        } else {
            megaNode.children.push(newCol);
        }

        if (window.markDirty) window.markDirty(true);
        if (window.pushHistorySnapshot) window.pushHistorySnapshot();
        if (window.renderCanvas) window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        if (window.showToast) window.showToast("Added column to Mega Menu");
    }
    window.addMegaMenuColumnToItem = addMegaMenuColumnToItem;

    function getActiveMegaMenuContainerForNav(navNode) {
        if (!navNode || !Array.isArray(navNode.children)) return null;
        if (window._activeMegaMenuState && window._activeMegaMenuState.navNodeId === navNode.id) {
            const items = (navNode.data && navNode.data.items) || [];
            const activeItem = items[window._activeMegaMenuState.itemIndex];
            if (activeItem && activeItem.mega_menu_node_id) {
                return navNode.children.find(c => c.id === activeItem.mega_menu_node_id) || null;
            }
        }
        return null;
    }
    window.getActiveMegaMenuContainerForNav = getActiveMegaMenuContainerForNav;

    function renderNavMenuMegaMenuInspector(ext, node) {
        if (!node.data) node.data = {};
        if (!Array.isArray(node.data.items) || node.data.items.length === 0) {
            node.data.items = [
                { id: 1, title: "Products", url: "/products", target: "", icon: "fa-solid fa-layer-group", badge: "2026", description: "", mega_menu_template_id: null },
                { id: 2, title: "Solutions", url: "/solutions", target: "", icon: "fa-solid fa-bolt", badge: "", description: "", mega_menu_template_id: null },
                { id: 3, title: "Developers", url: "/docs", target: "", icon: "fa-solid fa-code", badge: "", description: "", mega_menu_template_id: null },
                { id: 4, title: "Pricing", url: "/pricing", target: "", icon: "", badge: "", description: "", mega_menu_template_id: null }
            ];
        }

        const navPanel = document.createElement("div");
        navPanel.className = "cad-panel open";
        navPanel.innerHTML = `<div class="cad-panel-body" style="color:var(--day-muted);">Loading 2026 Mega Menu Builder...</div>`;
        ext.appendChild(navPanel);

        window.loadSymbolTemplatesCache(false).then(list => {
            const itemsHtml = node.data.items.map((it, idx) => {
                const tplId = Number(it.mega_menu_template_id || 0);
                const hasInlineNode = !!(it.mega_menu_node_id && Array.isArray(node.children) && node.children.some(c => c.id === it.mega_menu_node_id));
                const isCanvasOpen = !!(window._activeMegaMenuState && window._activeMegaMenuState.navNodeId === node.id && window._activeMegaMenuState.itemIndex === idx);
                const symOpts = `<option value="0">-- None (No Linked Symbol) --</option>` + list.map(s =>
                    `<option value="${s.id}" ${Number(s.id) === tplId ? "selected" : ""}>#${s.id} — ${s.title}</option>`
                ).join("");

                return `<div class="repeater-item" data-nav-item-card="${idx}">
                    <div class="repeater-item-header">
                        <span style="font-size:10.5px;font-weight:700;color:var(--day-ink);display:flex;align-items:center;gap:6px;">
                            <span style="color:var(--day-signal);font-family:var(--day-font-mono);">#${idx + 1}</span>
                            <span>${(it.title || "Menu Item").replace(/</g, "&lt;")}</span>
                            ${hasInlineNode ? `<span style="font-size:9px;padding:1px 6px;border-radius:999px;background:var(--day-signal-subtle);color:var(--day-signal);border:1px solid var(--day-signal);">MEGA SECTION</span>` : ""}
                        </span>
                        <button type="button" class="repeater-remove-btn" data-nav-del="${idx}" title="Remove Menu Item"><i class="fa-solid fa-trash-can"></i></button>
                    </div>
                    <div class="cad-row"><span class="cad-label">Label</span><input type="text" class="cad-input" style="flex:1;" data-nav-idx="${idx}" data-nav-field="title" value="${(it.title || "").replace(/"/g, "&quot;")}"></div>
                    <div class="cad-row"><span class="cad-label">URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-nav-idx="${idx}" data-nav-field="url" value="${(it.url || "").replace(/"/g, "&quot;")}"></div>
                    <div class="cad-row"><span class="cad-label">Icon</span><input type="text" class="cad-input font-mono" style="flex:1;" data-nav-idx="${idx}" data-nav-field="icon" data-icon-field="1" placeholder="Pick an icon..." value="${(it.icon || "").replace(/"/g, "&quot;")}"></div>
                    <div class="cad-row">
                        <span class="cad-label">Badge</span>
                        <input type="text" class="cad-input" style="width:74px;" data-nav-idx="${idx}" data-nav-field="badge" placeholder="NEW" value="${(it.badge || "").replace(/"/g, "&quot;")}">
                        <span class="cad-label" style="width:auto;margin-left:4px;">Width</span>
                        <select class="cad-select" style="flex:1;" data-nav-idx="${idx}" data-nav-field="mega_menu_width">
                            <option value="980px" ${(it.mega_menu_width || "980px") === "980px" ? "selected" : ""}>Wide (980px)</option>
                            <option value="1140px" ${it.mega_menu_width === "1140px" ? "selected" : ""}>Full Container (1140px)</option>
                            <option value="720px" ${it.mega_menu_width === "720px" ? "selected" : ""}>Compact (720px)</option>
                            <option value="100%" ${it.mega_menu_width === "100%" ? "selected" : ""}>Full Bleed (100%)</option>
                        </select>
                    </div>
                    <div class="cad-row"><span class="cad-label">Subtitle</span><input type="text" class="cad-input" style="flex:1;" data-nav-idx="${idx}" data-nav-field="description" placeholder="Optional item subtitle" value="${(it.description || "").replace(/"/g, "&quot;")}"></div>

                    <div class="cad-sub-label" style="margin-top:4px;padding-top:6px;">Visual Mega Menu Section Builder</div>
                    ${hasInlineNode ? `
                        <div style="display:flex;flex-wrap:wrap;gap:5px;">
                            <button type="button" class="cad-btn sm ${isCanvasOpen ? 'primary' : 'secondary'}" style="flex:1;" data-mega-toggle-canvas="${idx}">
                                <i class="fa-solid fa-${isCanvasOpen ? 'eye-slash' : 'pen-ruler'}"></i> ${isCanvasOpen ? 'Close Canvas Tray' : 'Edit Mega Menu on Canvas'}
                            </button>
                            <button type="button" class="cad-btn sm secondary" data-mega-add-col="${idx}" title="Add Column to Mega Menu">
                                <i class="fa-solid fa-plus"></i> Column
                            </button>
                            <button type="button" class="cad-btn sm danger" data-mega-remove-sec="${idx}" title="Remove Visual Mega Menu Section">
                                <i class="fa-solid fa-xmark"></i>
                            </button>
                        </div>
                        <div style="display:flex;gap:4px;margin-top:4px;">
                            <button type="button" class="cad-btn sm secondary" style="flex:1;" data-mega-preset="${idx}" data-preset-key="saas_3col">3-Col SaaS</button>
                            <button type="button" class="cad-btn sm secondary" style="flex:1;" data-mega-preset="${idx}" data-preset-key="enterprise_4col">4-Col Matrix</button>
                            <button type="button" class="cad-btn sm secondary" style="flex:1;" data-mega-preset="${idx}" data-preset-key="blank_row">3 Blank Cols</button>
                        </div>
                    ` : `
                        <div style="display:flex;flex-direction:column;gap:5px;">
                            <button type="button" class="cad-btn primary" style="width:100%;" data-mega-create-sec="${idx}" data-preset-key="saas_3col">
                                <i class="fa-solid fa-table-columns"></i> + Add Visual Mega Menu Section (3-Col SaaS)
                            </button>
                            <div style="display:flex;gap:5px;">
                                <button type="button" class="cad-btn sm secondary" style="flex:1;" data-mega-create-sec="${idx}" data-preset-key="enterprise_4col"><i class="fa-solid fa-grip"></i> 4-Col Enterprise</button>
                                <button type="button" class="cad-btn sm secondary" style="flex:1;" data-mega-create-sec="${idx}" data-preset-key="blank_row"><i class="fa-solid fa-columns"></i> Blank Columns</button>
                            </div>
                        </div>
                    `}

                    <div class="cad-row" style="margin-top:4px;">
                        <span class="cad-label">Or Symbol</span>
                        <select class="cad-select" style="flex:1;" data-nav-idx="${idx}" data-nav-field="mega_menu_template_id">${symOpts}</select>
                    </div>
                </div>`;
            }).join("");

            navPanel.innerHTML = `
                <div class="cad-panel-header">
                    <span class="cad-panel-title"><i class="fa-solid fa-bars" style="color:var(--day-signal);"></i> Navigation &amp; 2026 Mega Menu</span>
                    <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
                </div>
                <div class="cad-panel-body">
                    <div class="cad-row">
                        <span class="cad-label">Alignment</span>
                        <select class="cad-select" style="flex:1;" id="dp-nav-align-select">
                            <option value="left" ${(node.data.align || "left") === "left" ? "selected" : ""}>Left</option>
                            <option value="center" ${node.data.align === "center" ? "selected" : ""}>Center</option>
                            <option value="right" ${node.data.align === "right" ? "selected" : ""}>Right</option>
                            <option value="justify" ${node.data.align === "justify" ? "selected" : ""}>Space Between</option>
                        </select>
                    </div>
                    <div class="cad-sub-label" style="margin-top:4px;">Mobile Hamburger &amp; Drawer Styling</div>
                    <div class="cad-row">
                        <label style="display:flex;align-items:center;gap:6px;font-size:11px;color:var(--day-ink);cursor:pointer;flex:1;">
                            <input type="checkbox" id="dp-nav-mob-ham" ${node.data.mobile_hamburger !== false ? "checked" : ""}> Enable Hamburger on Mobile
                        </label>
                        <label style="display:flex;align-items:center;gap:6px;font-size:11px;color:var(--day-signal);cursor:pointer;">
                            <input type="checkbox" id="dp-nav-mob-preview" ${node.data.mobile_force_open ? "checked" : ""}> Preview Mobile
                        </label>
                    </div>
                    <div class="cad-row">
                        <span class="cad-label">Icon Color</span>
                        <input type="text" class="cad-input font-mono" style="flex:1;" data-mob-style="hamburger_color" placeholder="#ffffff" value="${node.data.hamburger_color || ''}">
                        <span class="cad-label" style="width:auto;margin-left:4px;">Btn BG</span>
                        <input type="text" class="cad-input font-mono" style="flex:1;" data-mob-style="hamburger_bg" placeholder="rgba(147,161,255,0.14)" value="${node.data.hamburger_bg || ''}">
                    </div>
                    <div class="cad-row">
                        <span class="cad-label">Drawer BG</span>
                        <input type="text" class="cad-input font-mono" style="flex:1;" data-mob-style="mobile_menu_bg" placeholder="#0f172a" value="${node.data.mobile_menu_bg || ''}">
                        <span class="cad-label" style="width:auto;margin-left:4px;">Text</span>
                        <input type="text" class="cad-input font-mono" style="flex:1;" data-mob-style="mobile_menu_text" placeholder="#f8fafc" value="${node.data.mobile_menu_text || ''}">
                    </div>
                    <div class="repeater-list">${itemsHtml}</div>
                    <button type="button" id="btn-add-nav-menu-item" class="cad-btn secondary" style="width:100%;margin-top:6px;">
                        <i class="fa-solid fa-plus"></i> Add Menu Item
                    </button>
                </div>
            `;

            const header = navPanel.querySelector(".cad-panel-header");
            if (header) header.onclick = () => navPanel.classList.toggle("open");

            if (window.enhanceIconInput) {
                navPanel.querySelectorAll("input[data-icon-field='1']").forEach(inp => window.enhanceIconInput(inp));
            }

            const alignSel = navPanel.querySelector("#dp-nav-align-select");
            if (alignSel) {
                alignSel.onchange = () => {
                    node.data.align = alignSel.value;
                    if (window.markDirty) window.markDirty(true);
                    if (window.renderCanvas) window.renderCanvas();
                };
            }
            const mobHamChk = navPanel.querySelector("#dp-nav-mob-ham");
            if (mobHamChk) {
                mobHamChk.onchange = () => {
                    node.data.mobile_hamburger = mobHamChk.checked;
                    if (window.markDirty) window.markDirty(true);
                    if (window.renderCanvas) window.renderCanvas();
                };
            }
            const mobPrevChk = navPanel.querySelector("#dp-nav-mob-preview");
            if (mobPrevChk) {
                mobPrevChk.onchange = () => {
                    node.data.mobile_force_open = mobPrevChk.checked;
                    if (window.renderCanvas) window.renderCanvas();
                };
            }
            navPanel.querySelectorAll("[data-mob-style]").forEach(inp => {
                inp.oninput = () => {
                    const k = inp.getAttribute("data-mob-style");
                    node.data[k] = inp.value;
                    if (!node.styles) node.styles = {};
                    if (!node.styles.raw_settings) node.styles.raw_settings = {};
                    if (inp.value.trim()) node.styles.raw_settings[k] = inp.value.trim();
                    else delete node.styles.raw_settings[k];
                    if (window.markDirty) window.markDirty(true);
                    if (window.renderCanvas) window.renderCanvas();
                };
            });

            navPanel.querySelectorAll("[data-nav-field]").forEach(ctrl => {
                const handler = () => {
                    const idx = parseInt(ctrl.getAttribute("data-nav-idx"), 10);
                    const field = ctrl.getAttribute("data-nav-field");
                    if (!node.data.items[idx]) return;
                    if (field === "mega_menu_template_id") {
                        const v = parseInt(ctrl.value, 10) || 0;
                        node.data.items[idx].mega_menu_template_id = v > 0 ? v : null;
                    } else {
                        node.data.items[idx][field] = ctrl.value;
                    }
                    if (window.markDirty) window.markDirty(true);
                    if (window.renderCanvas) window.renderCanvas();
                };
                ctrl.oninput = handler;
                ctrl.onchange = handler;
            });

            navPanel.querySelectorAll("[data-mega-create-sec], [data-mega-preset]").forEach(btn => {
                btn.onclick = () => {
                    const idx = parseInt(btn.getAttribute("data-mega-create-sec") || btn.getAttribute("data-mega-preset"), 10);
                    const preset = btn.getAttribute("data-preset-key") || "saas_3col";
                    ensureMegaMenuContainerForItem(node, idx, preset);
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                    if (window.renderProInspectorExtensions) window.renderProInspectorExtensions(node);
                    if (window.showToast) window.showToast("Mega Menu Section opened on canvas");
                };
            });

            navPanel.querySelectorAll("[data-mega-toggle-canvas]").forEach(btn => {
                btn.onclick = () => {
                    const idx = parseInt(btn.getAttribute("data-mega-toggle-canvas"), 10);
                    const isOpen = window._activeMegaMenuState && window._activeMegaMenuState.navNodeId === node.id && window._activeMegaMenuState.itemIndex === idx;
                    window._activeMegaMenuState = isOpen ? null : { navNodeId: node.id, itemIndex: idx };
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.renderProInspectorExtensions) window.renderProInspectorExtensions(node);
                };
            });

            navPanel.querySelectorAll("[data-mega-add-col]").forEach(btn => {
                btn.onclick = () => {
                    const idx = parseInt(btn.getAttribute("data-mega-add-col"), 10);
                    addMegaMenuColumnToItem(node, idx);
                };
            });

            navPanel.querySelectorAll("[data-mega-remove-sec]").forEach(btn => {
                btn.onclick = () => {
                    const idx = parseInt(btn.getAttribute("data-mega-remove-sec"), 10);
                    const item = node.data.items[idx];
                    if (item && item.mega_menu_node_id && Array.isArray(node.children)) {
                        node.children = node.children.filter(c => c.id !== item.mega_menu_node_id);
                        delete item.mega_menu_node_id;
                    }
                    if (window._activeMegaMenuState && window._activeMegaMenuState.itemIndex === idx) {
                        window._activeMegaMenuState = null;
                    }
                    if (window.markDirty) window.markDirty(true);
                    if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                    if (window.renderProInspectorExtensions) window.renderProInspectorExtensions(node);
                };
            });

            navPanel.querySelectorAll("[data-nav-del]").forEach(btn => {
                btn.onclick = () => {
                    const idx = parseInt(btn.getAttribute("data-nav-del"), 10);
                    const removed = node.data.items[idx];
                    if (removed && removed.mega_menu_node_id && Array.isArray(node.children)) {
                        node.children = node.children.filter(c => c.id !== removed.mega_menu_node_id);
                    }
                    node.data.items.splice(idx, 1);
                    if (window.markDirty) window.markDirty(true);
                    if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                    if (window.renderProInspectorExtensions) window.renderProInspectorExtensions(node);
                };
            });

            const addBtn = navPanel.querySelector("#btn-add-nav-menu-item");
            if (addBtn) {
                addBtn.onclick = () => {
                    node.data.items.push({
                        id: Date.now(),
                        title: "New Item",
                        url: "/",
                        target: "",
                        icon: "fa-solid fa-compass",
                        badge: "",
                        description: "",
                        mega_menu_template_id: null
                    });
                    if (window.markDirty) window.markDirty(true);
                    if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.renderProInspectorExtensions) window.renderProInspectorExtensions(node);
                };
            }
        });
    }
    window.renderNavMenuMegaMenuInspector = renderNavMenuMegaMenuInspector;
})();
"###;
