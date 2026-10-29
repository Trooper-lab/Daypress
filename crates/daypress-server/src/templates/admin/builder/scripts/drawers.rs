//! Left Rail tabs controller, Drawer state machine and Add Elements catalog (`scripts/drawers.rs`).

pub const JS_DRAWERS: &str = r###"
(function() {
    function initRailController() {
        const railBtns = document.querySelectorAll(".rail-tab-btn[data-drawer]");
        const drawer = document.getElementById("daystudio-drawer");
        const panelContainer = document.getElementById("drawer-panel-container");

        railBtns.forEach(btn => {
            btn.addEventListener("click", () => {
                const target = btn.dataset.drawer;
                const state = window.DayStudioState;

                if (state.activeDrawer === target && state.drawerOpen) {
                    state.drawerOpen = false;
                    drawer.classList.add("drawer-closed");
                    btn.classList.remove("active");
                } else {
                    state.activeDrawer = target;
                    state.drawerOpen = true;
                    drawer.classList.remove("drawer-closed");
                    railBtns.forEach(b => b.classList.remove("active"));
                    btn.classList.add("active");
                    renderActiveDrawer(target, panelContainer);
                }
            });
        });

        const btnSettings = document.getElementById("rail-btn-settings");
        if (btnSettings) btnSettings.onclick = () => document.getElementById("settings-modal")?.classList.add("open");
        const btnHelp = document.getElementById("rail-btn-help");
        if (btnHelp) btnHelp.onclick = () => document.getElementById("modal-shortcuts")?.classList.add("open");

        renderActiveDrawer("elements", panelContainer);
    }

    function renderActiveDrawer(drawerId, container) {
        if (!container) return;
        if (drawerId === "elements") renderElementsDrawer(container);
        else if (drawerId === "navigator") window.renderNavigatorDrawer && window.renderNavigatorDrawer(container);
        else if (drawerId === "templates") window.renderThemeDrawer && window.renderThemeDrawer(container);
        else if (drawerId === "tokens") window.renderTokensDrawer && window.renderTokensDrawer(container);
        else if (drawerId === "css") window.renderCssDrawer && window.renderCssDrawer(container);
        else if (drawerId === "history") renderHistoryDrawer(container);
    }
    window.renderActiveDrawer = renderActiveDrawer;

    const BLOCK_CATALOG = [
        {
            category: "Layout",
            items: [
                { type: "Section", title: "Section", icon: "fa-solid fa-square" },
                { type: "Container", title: "Container", icon: "fa-solid fa-box" },
                { type: "Column", title: "Column", icon: "fa-solid fa-table-columns" },
                { type: "Grid", title: "CSS Grid", icon: "fa-solid fa-border-all" },
                { type: "Flex", title: "Flexbox", icon: "fa-solid fa-arrows-left-right" }
            ]
        },
        {
            category: "Basic",
            items: [
                { type: "Heading", title: "Heading", icon: "fa-solid fa-heading" },
                { type: "Text", title: "Text Block", icon: "fa-solid fa-paragraph" },
                { type: "Button", title: "Button", icon: "fa-solid fa-arrow-pointer" },
                { type: "Image", title: "Image", icon: "fa-solid fa-image" },
                { type: "Video", title: "Video", icon: "fa-solid fa-film" },
                { type: "Spacer", title: "Spacer", icon: "fa-solid fa-arrows-up-down" },
                { type: "Divider", title: "Divider", icon: "fa-solid fa-minus" },
                { type: "Icon", title: "Vector Icon", icon: "fa-solid fa-icons" }
            ]
        },
        {
            category: "Pro Marketing",
            items: [
                { type: "PriceCard", title: "Price Card", icon: "fa-solid fa-credit-card" },
                { type: "CallToAction", title: "Call To Action", icon: "fa-solid fa-bullhorn" },
                { type: "Countdown", title: "Countdown", icon: "fa-solid fa-clock" },
                { type: "FlipBox", title: "3D Flip Box", icon: "fa-solid fa-arrows-rotate" },
                { type: "AnimatedHeadline", title: "Anim Headline", icon: "fa-solid fa-wand-magic-sparkles" },
                { type: "Testimonial", title: "Testimonial", icon: "fa-solid fa-quote-left" },
                { type: "StarRating", title: "Star Rating", icon: "fa-solid fa-star" },
                { type: "ProgressBar", title: "Progress Bar", icon: "fa-solid fa-bars-progress" },
                { type: "Alert", title: "Alert Box", icon: "fa-solid fa-triangle-exclamation" },
                { type: "Benchmark", title: "Benchmark HUD", icon: "fa-solid fa-gauge-high" },
                { type: "SocialIcons", title: "Social Icons", icon: "fa-solid fa-share-nodes" }
            ]
        },
        {
            category: "Dynamic & Theme",
            items: [
                { type: "NavMenu", title: "Nav Menu", icon: "fa-solid fa-bars" },
                { type: "LoopGrid", title: "Loop Grid", icon: "fa-solid fa-table-cells" },
                { type: "TableOfContents", title: "TOC Index", icon: "fa-solid fa-list-ol" },
                { type: "Breadcrumbs", title: "Breadcrumbs", icon: "fa-solid fa-angle-right" },
                { type: "SearchForm", title: "Search Form", icon: "fa-solid fa-magnifying-glass" },
                { type: "LoginForm", title: "Login Form", icon: "fa-solid fa-right-to-bracket" }
            ]
        }
    ];

    function renderElementsDrawer(container) {
        container.innerHTML = `
            <div class="drawer-header">
                <span class="drawer-title">Add Elements</span>
            </div>
            <div class="drawer-search-well">
                <input type="search" class="drawer-search-input" id="elem-search-input" placeholder="Search native blocks (Q or /)...">
            </div>
            <div class="drawer-scroll-body" id="elem-catalog-body"></div>
        `;

        const body = document.getElementById("elem-catalog-body");
        renderCategoryAccordions(body, BLOCK_CATALOG);

        const search = document.getElementById("elem-search-input");
        if (search) {
            search.oninput = () => {
                const q = search.value.toLowerCase().trim();
                filterCatalog(q);
            };
        }
    }

    function renderCategoryAccordions(container, catalog) {
        container.innerHTML = "";
        catalog.forEach(cat => {
            const acc = document.createElement("div");
            acc.className = "cat-accordion";
            acc.innerHTML = `
                <div class="cat-header">
                    <span class="cat-title">${cat.category}</span>
                    <span class="cat-count">${cat.items.length}</span>
                </div>
                <div class="cat-grid"></div>
            `;
            const grid = acc.querySelector(".cat-grid");
            cat.items.forEach(item => {
                const card = document.createElement("div");
                card.className = "widget-card";
                card.dataset.widgetType = item.type;
                card.draggable = true;
                card.setAttribute("draggable", "true");
                card.innerHTML = `
                    <i class="${item.icon} widget-icon"></i>
                    <span class="widget-title">${item.title}</span>
                `;
                card.onclick = () => insertWidgetFromPalette(item.type);
                card.ondragstart = (e) => {
                    window._draggedPaletteType = item.type;
                    window._draggedNavNodeId = null;
                    e.dataTransfer.setData("text/plain", "palette:" + item.type);
                    e.dataTransfer.setData("application/x-daystudio-palette", item.type);
                    e.dataTransfer.effectAllowed = "copyMove";
                    card.classList.add("is-dragging");
                };
                card.ondragend = () => {
                    window._draggedPaletteType = null;
                    card.classList.remove("is-dragging");
                    if (window.hideDropIndicator) window.hideDropIndicator();
                };
                grid.appendChild(card);
            });
            container.appendChild(acc);
        });
    }

    function createNodeFromPaletteType(type) {
        const starterData = window.getWidgetStarterDefaults ? window.getWidgetStarterDefaults(type) : {};
        return {
            id: window.generateId(type.toLowerCase()),
            type: type,
            data: starterData,
            styles: {},
            children: []
        };
    }

    function insertWidgetFromPalette(type) {
        const state = window.DayStudioState;
        const newNode = createNodeFromPaletteType(type);
        const currentId = state.activeElementId;
        const current = currentId ? window.findNodeById(currentId) : null;

        if (current && ["Container", "Section", "Column", "Grid", "Flex"].includes(current.type)) {
            current.children = current.children || [];
            current.children.push(newNode);
        } else if (current) {
            const parent = window.findParentNode(currentId);
            if (parent && parent.children) {
                const idx = parent.children.findIndex(c => c.id === currentId);
                parent.children.splice(idx + 1, 0, newNode);
            }
        } else {
            const newSec = {
                id: window.generateId("sec"),
                type: "Section",
                styles: { padding_top: "60px", padding_bottom: "60px" },
                children: [
                    {
                        id: window.generateId("cont"),
                        type: "Container",
                        styles: {},
                        children: [ newNode ]
                    }
                ]
            };
            state.pageAst.root.children = state.pageAst.root.children || [];
            state.pageAst.root.children.push(newSec);
        }

        window.markDirty(true);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        if (window.selectElement) window.selectElement(newNode.id);
        if (window.showToast) window.showToast(`Added ${type}`);
    }
    window.insertWidgetFromPalette = insertWidgetFromPalette;

    function insertWidgetRelativeTo(type, targetId, position) {
        const newNode = createNodeFromPaletteType(type);

        if (!targetId || targetId === "root") {
            insertWidgetFromPalette(type);
            return;
        }

        const targetNode = window.findNodeById(targetId);
        if (!targetNode) {
            insertWidgetFromPalette(type);
            return;
        }

        if (position === "inside" || (["Section", "Container", "Column", "Grid", "Flex"].includes(targetNode.type) && position !== "before" && position !== "after")) {
            targetNode.children = targetNode.children || [];
            targetNode.children.push(newNode);
        } else {
            const parent = window.findParentNode(targetId);
            if (parent && parent.children) {
                const idx = parent.children.findIndex(c => c.id === targetId);
                const insertIdx = position === "before" ? idx : idx + 1;
                parent.children.splice(insertIdx, 0, newNode);
            } else {
                targetNode.children = targetNode.children || [];
                targetNode.children.push(newNode);
            }
        }

        window.markDirty(true);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        if (window.selectElement) window.selectElement(newNode.id);
        if (window.showToast) window.showToast(`Dropped ${type}`);
    }
    window.insertWidgetRelativeTo = insertWidgetRelativeTo;

    function filterCatalog(query) {
        document.querySelectorAll(".widget-card").forEach(card => {
            const title = card.querySelector(".widget-title").textContent.toLowerCase();
            const match = !query || title.includes(query);
            card.style.display = match ? "flex" : "none";
        });
    }

    function renderHistoryDrawer(container) {
        const state = window.DayStudioState;
        const itemsHtml = state.historyStack.map((snap, idx) => {
            const isCurrent = idx === state.historyIndex;
            return `<div class="switcher-item ${isCurrent ? 'active' : ''}" data-hist-idx="${idx}" style="margin-bottom:4px;">
                <span>Snapshot #${idx + 1} ${idx === 0 ? '(Initial)' : ''}</span>
                <span class="template-type-badge">${isCurrent ? 'ACTIVE' : 'RESTORE'}</span>
            </div>`;
        }).reverse().join("");
        container.innerHTML = `
            <div class="drawer-header"><span class="drawer-title">Audit History</span></div>
            <div class="drawer-scroll-body" style="padding: 16px;">
                <p style="font-size: 12px; color: var(--day-muted); margin-bottom: 12px;">Snapshots: ${state.historyStack.length} / ${state.maxHistory} (Current: #${state.historyIndex + 1})</p>
                <div style="display: flex; gap: 8px; margin-bottom: 12px;">
                    <button class="cad-btn secondary" style="flex:1;" onclick="window.undo()"><i class="fa-solid fa-arrow-rotate-left"></i> Undo</button>
                    <button class="cad-btn secondary" style="flex:1;" onclick="window.redo()"><i class="fa-solid fa-arrow-rotate-right"></i> Redo</button>
                </div>
                <div>${itemsHtml}</div>
            </div>
        `;
        container.querySelectorAll("[data-hist-idx]").forEach(row => {
            row.onclick = () => {
                const idx = parseInt(row.dataset.histIdx, 10);
                if (!isNaN(idx) && state.historyStack[idx]) {
                    state.historyIndex = idx;
                    state.pageAst = JSON.parse(state.historyStack[idx]);
                    window.markDirty(true);
                    window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                    if (window.renderInspector) window.renderInspector();
                    renderHistoryDrawer(container);
                }
            };
        });
    }

    document.addEventListener("DOMContentLoaded", () => {
        initRailController();
    });
})();
"###;
