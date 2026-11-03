//! DayStudio 0.5 Canvas Renderers for Dynamic & Theme Blocks (`scripts/canvas_pro_dynamic_render.rs`).

pub const JS_CANVAS_PRO_DYNAMIC_RENDER: &str = r###"
(function() {
    window._dpSymbolTemplatesCache = null;

    async function loadSymbolTemplatesCache(force) {
        if (window._dpSymbolTemplatesCache && !force) return window._dpSymbolTemplatesCache;
        try {
            if (window.StudioApi && window.StudioApi.getTemplates) {
                const list = await window.StudioApi.getTemplates("section_symbol");
                window._dpSymbolTemplatesCache = Array.isArray(list) ? list : [];
            } else {
                const res = await fetch("/wp-json/daypress/v1/templates?template_type=section_symbol", { credentials: "include" });
                window._dpSymbolTemplatesCache = res.ok ? await res.json() : [];
            }
        } catch (e) {
            window._dpSymbolTemplatesCache = [];
        }
        return window._dpSymbolTemplatesCache;
    }
    window.loadSymbolTemplatesCache = loadSymbolTemplatesCache;

    function resolveActiveMegaMenuIndex(navNode, items) {
        const state = window.DayStudioState;
        const activeId = state ? state.activeElementId : null;

        // 1. If the currently selected element on canvas is inside one of navNode's MegaMenuContainer children, keep that item's Mega Menu open!
        if (activeId && Array.isArray(navNode.children)) {
            for (let idx = 0; idx < items.length; idx++) {
                const megaNodeId = items[idx] && items[idx].mega_menu_node_id;
                if (!megaNodeId) continue;
                if (activeId === megaNodeId || (window.isDescendant && window.isDescendant(megaNodeId, activeId))) {
                    window._activeMegaMenuState = { navNodeId: navNode.id, itemIndex: idx };
                    return idx;
                }
            }
        }

        // 2. Otherwise check explicit open state
        if (window._activeMegaMenuState && window._activeMegaMenuState.navNodeId === navNode.id) {
            const idx = window._activeMegaMenuState.itemIndex;
            if (idx >= 0 && idx < items.length) return idx;
        }
        return -1;
    }

    function renderProNavMenuWidget(node) {
        const d = node.data || {};
        const el = document.createElement("nav");
        el.className = "dp-pro-block dp-nav-menu";
        el.style.cssText = "display:flex;flex-direction:column;gap:10px;width:100%;position:relative;";

        const rawItems = Array.isArray(d.items) && d.items.length
            ? d.items
            : [
                { title: "Products", url: "/products", icon: "fa-solid fa-layer-group", badge: "2026" },
                { title: "Solutions", url: "/solutions", icon: "fa-solid fa-bolt" },
                { title: "Developers", url: "/docs", icon: "fa-solid fa-code" },
                { title: "Pricing", url: "/pricing", icon: "" }
            ];

        const activeMegaIdx = resolveActiveMegaMenuIndex(node, rawItems);
        const isMobileBp = (window.DayStudioState && window.DayStudioState.currentBreakpoint === "mobile") || !!d.mobile_force_open;
        const mobileHamburgerEnabled = d.mobile_hamburger !== false;

        if (isMobileBp && mobileHamburgerEnabled) {
            if (!window._mobileMenuOpenMap) window._mobileMenuOpenMap = {};
            if (window._mobileMenuOpenMap[node.id] === undefined) window._mobileMenuOpenMap[node.id] = true;
            const isDrawerOpen = !!window._mobileMenuOpenMap[node.id];

            const hamColor = d.hamburger_color || "currentColor";
            const hamBg = d.hamburger_bg || "rgba(147,161,255,0.14)";
            const drawerBg = d.mobile_menu_bg || "#0f172a";
            const drawerText = d.mobile_menu_text || "#f8fafc";

            const mobHeader = document.createElement("div");
            mobHeader.style.cssText = "display:flex;align-items:center;justify-content:space-between;width:100%;gap:10px;";
            const btnToggle = document.createElement("button");
            btnToggle.type = "button";
            btnToggle.className = "dp-menu-toggle";
            btnToggle.style.cssText = `display:inline-flex;flex-direction:column;justify-content:center;align-items:center;gap:4px;background:${hamBg};color:${hamColor};border:1px solid rgba(148,163,184,0.28);border-radius:6px;padding:8px 10px;cursor:pointer;`;
            btnToggle.innerHTML = `<span style="display:block;width:18px;height:2px;background:${hamColor};border-radius:2px;"></span><span style="display:block;width:18px;height:2px;background:${hamColor};border-radius:2px;"></span><span style="display:block;width:18px;height:2px;background:${hamColor};border-radius:2px;"></span>`;
            btnToggle.onclick = (e) => {
                e.stopPropagation();
                window._mobileMenuOpenMap[node.id] = !isDrawerOpen;
                if (window.selectElement) window.selectElement(node.id);
                if (window.renderCanvas) window.renderCanvas();
            };

            const mobHint = document.createElement("span");
            mobHint.style.cssText = "font-size:10px;font-family:var(--day-font-mono);color:var(--day-signal,#93A1FF);opacity:0.85;";
            mobHint.textContent = isDrawerOpen ? "Mobile Hamburger (Open)" : "Mobile Hamburger (Closed)";
            mobHeader.appendChild(mobHint);
            mobHeader.appendChild(btnToggle);
            el.appendChild(mobHeader);

            if (isDrawerOpen) {
                const drawer = document.createElement("div");
                drawer.className = "dp-nav-menu-list is-open";
                drawer.style.cssText = `display:flex;flex-direction:column;gap:8px;width:100%;background:${drawerBg};color:${drawerText};padding:14px;border-radius:10px;border:1px solid rgba(148,163,184,0.22);box-sizing:border-box;position:static;`;
                rawItems.forEach((it) => {
                    const label = (typeof it === "string") ? it : (it.title || it.label || "Link");
                    const icon = (typeof it === "object" && it && it.icon) ? `<i class="${it.icon}" style="margin-right:8px;opacity:0.85;"></i>` : "";
                    const badge = (typeof it === "object" && it && it.badge) ? `<span class="dp-nav-badge" style="margin-left:auto;">${it.badge}</span>` : "";
                    const rowLink = document.createElement("a");
                    rowLink.href = (typeof it === "object" && it.url) || "#";
                    rowLink.style.cssText = `display:flex;align-items:center;padding:8px 10px;border-radius:6px;color:${drawerText};text-decoration:none;font-weight:600;font-size:13.5px;background:rgba(255,255,255,0.03);`;
                    rowLink.innerHTML = `${icon}<span>${label}</span>${badge}`;
                    rowLink.onclick = (e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        if (window.selectElement) window.selectElement(node.id);
                    };
                    drawer.appendChild(rowLink);
                });
                el.appendChild(drawer);
            }
            return el;
        }

        const barRow = document.createElement("div");
        const alignMap = { left: "flex-start", center: "center", right: "flex-end", justify: "space-between" };
        const justify = alignMap[d.align || "left"] || "flex-start";
        barRow.style.cssText = `display:flex;align-items:center;justify-content:${justify};gap:22px;flex-wrap:wrap;width:100%;`;

        rawItems.forEach((it, idx) => {
            const label = (typeof it === "string") ? it : (it.title || it.label || "Link");
            const icon = (typeof it === "object" && it && it.icon) ? `<i class="${it.icon} dp-nav-icon" style="margin-right:5px;"></i>` : "";
            const badge = (typeof it === "object" && it && it.badge) ? `<span class="dp-nav-badge" style="margin-left:5px;">${it.badge}</span>` : "";
            const desc = (typeof it === "object" && it && it.description) ? `<span class="dp-nav-desc">${it.description}</span>` : "";
            const hasInlineMega = !!(typeof it === "object" && it && it.mega_menu_node_id && Array.isArray(node.children) && node.children.some(c => c.id === it.mega_menu_node_id));
            const hasTemplateMega = !!(typeof it === "object" && it && Number(it.mega_menu_template_id || 0) > 0);
            const hasMega = hasInlineMega || hasTemplateMega;
            const isOpen = idx === activeMegaIdx;

            const itemWrap = document.createElement("div");
            itemWrap.className = "dp-nav-item" + (hasMega ? " menu-item-has-megamenu" : "");
            itemWrap.style.cssText = "display:inline-flex;align-items:center;gap:4px;position:relative;";

            const linkEl = document.createElement("a");
            linkEl.href = (typeof it === "object" && it.url) || "#";
            linkEl.className = "dp-nav-link";
            linkEl.style.cssText = `color:inherit;text-decoration:none;font-weight:600;font-size:14px;display:inline-flex;align-items:center;gap:4px;padding:6px 8px;border-radius:6px;transition:all 0.15s ease;${isOpen ? "background:rgba(147,161,255,0.14);color:var(--day-signal,#93A1FF);" : ""}`;
            const caret = hasMega
                ? `<i class="fa-solid fa-chevron-${isOpen ? 'up' : 'down'}" style="font-size:10px;opacity:0.75;margin-left:3px;color:${isOpen ? 'var(--day-signal,#93A1FF)' : 'inherit'};"></i>`
                : "";
            linkEl.innerHTML = `<span>${icon}${label}${badge}${caret}${desc}</span>`;

            linkEl.onclick = (e) => {
                e.preventDefault();
                e.stopPropagation();
                if (hasMega) {
                    window._activeMegaMenuState = isOpen ? null : { navNodeId: node.id, itemIndex: idx };
                    if (window.selectElement) window.selectElement(node.id);
                    if (window.renderCanvas) window.renderCanvas();
                } else {
                    if (window.selectElement) window.selectElement(node.id);
                }
            };

            itemWrap.appendChild(linkEl);
            barRow.appendChild(itemWrap);
        });

        el.appendChild(barRow);

        if (activeMegaIdx >= 0 && rawItems[activeMegaIdx] && typeof rawItems[activeMegaIdx] === "object") {
            const activeItem = rawItems[activeMegaIdx];
            const megaNode = (activeItem.mega_menu_node_id && Array.isArray(node.children))
                ? node.children.find(c => c.id === activeItem.mega_menu_node_id)
                : null;

            const tray = document.createElement("div");
            tray.className = "dp-megamenu-canvas-tray";
            tray.style.cssText = "width:100%;margin-top:8px;border:1px solid var(--day-signal,#93A1FF);border-radius:12px;background:#0f172a;box-shadow:0 20px 50px rgba(0,0,0,0.45);overflow:hidden;";

            const toolbar = document.createElement("div");
            toolbar.style.cssText = "display:flex;align-items:center;justify-content:space-between;gap:8px;padding:8px 12px;background:var(--day-chassis,#111214);border-bottom:1px solid var(--day-rule,#393836);flex-wrap:wrap;";
            toolbar.innerHTML = `
                <div style="display:flex;align-items:center;gap:8px;">
                    <span style="font-size:10.5px;font-weight:700;text-transform:uppercase;letter-spacing:0.06em;color:var(--day-signal,#93A1FF);">
                        <i class="fa-solid fa-table-columns"></i> Mega Menu Canvas: "${activeItem.title || 'Item'}"
                    </span>
                    <span style="font-size:10px;color:var(--day-muted,#A7A297);">Click any element inside or drag blocks from Left (+) Drawer</span>
                </div>
                <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap;">
                    <button type="button" class="cad-btn sm secondary" data-mega-act="add-col" title="Add a new Column to this Mega Menu"><i class="fa-solid fa-plus"></i> Add Column</button>
                    <button type="button" class="cad-btn sm secondary" data-mega-act="preset-saas" title="Apply 2026 SaaS 3-Column + Spotlight Preset"><i class="fa-solid fa-wand-magic-sparkles"></i> 3-Col SaaS</button>
                    <button type="button" class="cad-btn sm secondary" data-mega-act="preset-matrix" title="Apply 4-Column Enterprise Matrix + Footer Bar"><i class="fa-solid fa-grip"></i> 4-Col Matrix</button>
                    <button type="button" class="cad-btn sm secondary" data-mega-act="select-surface" title="Select Mega Menu Container to style background, padding & border"><i class="fa-solid fa-Sliders"></i> Style Surface</button>
                    <button type="button" class="cad-btn sm danger" data-mega-act="close" title="Close Mega Menu Canvas"><i class="fa-solid fa-xmark"></i> Close</button>
                </div>
            `;

            toolbar.onclick = (e) => e.stopPropagation();
            toolbar.querySelector("[data-mega-act='add-col']").onclick = (e) => {
                e.stopPropagation();
                if (window.addMegaMenuColumnToItem) window.addMegaMenuColumnToItem(node, activeMegaIdx);
            };
            toolbar.querySelector("[data-mega-act='preset-saas']").onclick = (e) => {
                e.stopPropagation();
                if (window.ensureMegaMenuContainerForItem) {
                    window.ensureMegaMenuContainerForItem(node, activeMegaIdx, "saas_3col");
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                }
            };
            toolbar.querySelector("[data-mega-act='preset-matrix']").onclick = (e) => {
                e.stopPropagation();
                if (window.ensureMegaMenuContainerForItem) {
                    window.ensureMegaMenuContainerForItem(node, activeMegaIdx, "enterprise_4col");
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                }
            };
            toolbar.querySelector("[data-mega-act='select-surface']").onclick = (e) => {
                e.stopPropagation();
                if (megaNode && window.selectElement) window.selectElement(megaNode.id);
            };
            toolbar.querySelector("[data-mega-act='close']").onclick = (e) => {
                e.stopPropagation();
                window._activeMegaMenuState = null;
                if (window.selectElement) window.selectElement(node.id);
                if (window.renderCanvas) window.renderCanvas();
            };

            tray.appendChild(toolbar);

            const bodyWrap = document.createElement("div");
            bodyWrap.style.cssText = "padding:16px;";

            if (megaNode && window.buildDomNode) {
                const liveDom = window.buildDomNode(megaNode);
                if (liveDom) bodyWrap.appendChild(liveDom);
            } else if (Number(activeItem.mega_menu_template_id || 0) > 0) {
                const tplId = Number(activeItem.mega_menu_template_id);
                const cachedSym = (window._dpSymbolTemplatesCache || []).find(s => Number(s.id) === tplId);
                bodyWrap.innerHTML = `
                    <div style="padding:18px;border:1px dashed var(--day-rule,#393836);border-radius:8px;display:flex;align-items:center;justify-content:space-between;gap:12px;">
                        <div>
                            <div style="font-size:12px;font-weight:700;color:var(--day-ink,#EFE9DE);"><i class="fa-solid fa-clone" style="color:var(--day-violet);"></i> Linked to Global Symbol: ${cachedSym ? cachedSym.title : ('Template #' + tplId)}</div>
                            <div style="font-size:11px;color:var(--day-muted,#A7A297);margin-top:2px;">Convert to an inline Visual Mega Menu Section to edit columns and blocks directly below this navbar.</div>
                        </div>
                        <button type="button" class="cad-btn primary" id="btn-convert-tpl-to-inline-mega"><i class="fa-solid fa-wand-magic-sparkles"></i> Edit Inline on Canvas</button>
                    </div>
                `;
                const convBtn = bodyWrap.querySelector("#btn-convert-tpl-to-inline-mega");
                if (convBtn) {
                    convBtn.onclick = (e) => {
                        e.stopPropagation();
                        if (window.ensureMegaMenuContainerForItem) {
                            window.ensureMegaMenuContainerForItem(node, activeMegaIdx, "from_symbol");
                            if (window.renderCanvas) window.renderCanvas();
                        }
                    };
                }
            }

            tray.appendChild(bodyWrap);
            el.appendChild(tray);
        }

        return el;
    }
    window.renderProNavMenuWidget = renderProNavMenuWidget;

    function applyClientSymbolOverrides(node, overrides) {
        if (!node || !overrides) return;
        const ov = overrides[node.id];
        if (ov && node.data) {
            if (ov.text) {
                if (node.type === "Text") node.data.content_html = ov.text;
                else node.data.text = ov.text;
            }
            if (ov.src && node.type === "Image") node.data.src = ov.src;
            if (ov.alt && node.type === "Image") node.data.alt = ov.alt;
            if (ov.url) node.data.url = ov.url;
        }
        if (Array.isArray(node.children)) {
            node.children.forEach(c => applyClientSymbolOverrides(c, overrides));
        }
    }
    window.applyClientSymbolOverrides = applyClientSymbolOverrides;
})();
"###;
