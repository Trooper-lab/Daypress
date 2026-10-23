//! Left Rail tabs controller, Drawer state machine and Add Elements catalog (`scripts/drawers.rs`).

pub const JS_DRAWERS: &str = r###"
(function() {
    function initRailController() {
        const railBtns = document.querySelectorAll(".rail-tab-btn[data-drawer]");
        const drawer = document.getElementById("neostudio-drawer");
        const panelContainer = document.getElementById("drawer-panel-container");

        railBtns.forEach(btn => {
            btn.addEventListener("click", () => {
                const target = btn.dataset.drawer;
                const state = window.NeoStudioState;

                if (state.activeDrawer === target && state.drawerOpen) {
                    // Toggle close
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

        // Pinned Bottom buttons
        const btnSettings = document.getElementById("rail-btn-settings");
        if (btnSettings) btnSettings.onclick = () => document.getElementById("settings-modal")?.classList.add("open");
        const btnHelp = document.getElementById("rail-btn-help");
        if (btnHelp) btnHelp.onclick = () => document.getElementById("modal-shortcuts")?.classList.add("open");

        // Render default Elements drawer
        renderActiveDrawer("elements", panelContainer);
    }

    function renderActiveDrawer(drawerId, container) {
        if (!container) return;
        if (drawerId === "elements") renderElementsDrawer(container);
        else if (drawerId === "navigator") window.renderNavigatorDrawer && window.renderNavigatorDrawer(container);
        else if (drawerId === "templates") window.renderThemeDrawer && window.renderThemeDrawer(container);
        else if (drawerId === "tokens") renderTokensDrawer(container);
        else if (drawerId === "history") renderHistoryDrawer(container);
    }
    window.renderActiveDrawer = renderActiveDrawer;

    // -------------------------------------------------------------
    // ADD ELEMENTS DRAWER: 35+ PRO BLOCKS
    // -------------------------------------------------------------
    const BLOCK_CATALOG = [
        {
            category: "ACF & Theme Blocks",
            items: [
                { type: "ACF_WhiteSection", title: "ACF White Sec", icon: "fa-solid fa-layer-group" },
                { type: "ACF_BlackAngle", title: "ACF Black Angle", icon: "fa-solid fa-shapes" },
                { type: "ACF_BlueCTA", title: "ACF Blue CTA", icon: "fa-solid fa-bullhorn" },
                { type: "ACF_ThreeCol", title: "ACF 3-Col Cards", icon: "fa-solid fa-table-columns" },
                { type: "ACF_Logos", title: "ACF Logos", icon: "fa-solid fa-Handshake" },
                { type: "ACF_Trainingen", title: "Showcase Grid", icon: "fa-solid fa-graduation-cap" },
                { type: "ACF_Faq", title: "ACF FAQ", icon: "fa-solid fa-circle-question" },
                { type: "ACF_Quotes", title: "ACF Quotes", icon: "fa-solid fa-quote-left" }
            ]
        },
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
                <input type="search" class="drawer-search-input" id="elem-search-input" placeholder="Search 35+ Pro blocks (Q or /)...">
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
                    e.dataTransfer.setData("application/x-neostudio-palette", item.type);
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

    const ACF_SNIPPETS = {
        ACF_WhiteSection: {
            name: "ACF Section: White Content",
            raw: `<section class="section white"><div class="container"><div class="row justify-content-center"><div class="col-md-10"><h2>CONTENT SECTION</h2><p>Add your rich text and block components here.</p><div class="wp-block-buttons"><div class="wp-block-button"><a class="wp-block-button__link wp-element-button" href="/contact/">Get in touch</a></div></div></div></div></div></section>`
        },
        ACF_BlackAngle: {
            name: "ACF Section: Black Angle",
            raw: `<section class="section black angle"><div class="container"><div class="row justify-content-center"><div class="col-md-10"><h2>HIGH-CONTRAST SECTION</h2><p>High-contrast section with modern accent background.</p><a href="/about/" class="btn btn-primary">Learn more</a></div></div></div></section>`
        },
        ACF_BlueCTA: {
            name: "ACF Section: Blue CTA",
            raw: `<section class="section blue angle"><div class="container"><div class="row justify-content-center"><div class="col-md-10"><h2>PRIMARY CALL TO ACTION</h2><p>Featured call-to-action section in theme primary color.</p><a href="/contact/" class="btn btn-primary">Schedule a consultation</a></div></div></div></section>`
        },
        ACF_ThreeCol: {
            name: "ACF Block: 3-Col Swiper",
            raw: `<section class="section white"><div class="container"><h2>THREE-COLUMN FEATURES</h2><div class="row"><div class="col-md-4"><div class="card"><div class="card-body"><h3>Feature One</h3><p>Fast, modular architecture designed for performance.</p><a href="#" class="btn btn-primary">Read more</a></div></div></div><div class="col-md-4"><div class="card"><div class="card-body"><h3>Feature Two</h3><p>Native Rust execution with zero PHP latency.</p><a href="#" class="btn btn-primary">Read more</a></div></div></div><div class="col-md-4"><div class="card"><div class="card-body"><h3>Feature Three</h3><p>1-Click seamless migration from legacy WordPress.</p><a href="#" class="btn btn-primary">Read more</a></div></div></div></div></div></section>`
        },
        ACF_Logos: {
            name: "ACF Block: Client Logos",
            raw: `<section class="section logo white"><div class="container"><h2 class="text-center">TRUSTED PARTNERS</h2><div class="logo-grid" style="display: flex; justify-content: space-around; align-items: center; gap: 24px; flex-wrap: wrap;"><div class="logo-item" style="font-weight: 700; font-size: 18px; color: #64748b;">PARTNER 1</div><div class="logo-item" style="font-weight: 700; font-size: 18px; color: #64748b;">PARTNER 2</div><div class="logo-item" style="font-weight: 700; font-size: 18px; color: #64748b;">PARTNER 3</div><div class="logo-item" style="font-weight: 700; font-size: 18px; color: #64748b;">PARTNER 4</div></div></div></section>`
        },
        ACF_Trainingen: {
            name: "ACF Block: Showcase Carousel",
            raw: `<section class="section black angle"><div class="container"><h2>FEATURE SHOWCASE</h2><div class="row"><div class="col-md-4"><div class="card"><div class="card-body"><h3>Showcase Item</h3><p>Detailed overview of key deliverables and outcomes.</p><a href="/showcase/" class="btn btn-primary">View details</a></div></div></div></div></div></section>`
        },
        ACF_Faq: {
            name: "ACF Block: FAQ Accordion",
            raw: `<section class="section white"><div class="container"><h2>FREQUENTLY ASKED QUESTIONS</h2><div class="accordion"><div class="accordion-item"><h3 class="accordion-header"><button class="accordion-button" type="button">What features are supported?</button></h3><div class="accordion-collapse collapse show"><div class="accordion-body"><p>Neopress natively supports Elementor, Divi, Gutenberg block themes, and ACF fields with sub-millisecond SSR latency.</p></div></div></div></div></div></section>`
        },
        ACF_Quotes: {
            name: "ACF Block: Quotes Slider",
            raw: `<section class="section blue angle"><div class="container"><h2>CLIENT TESTIMONIALS</h2><div class="swiper quoteSwiper"><div class="swiper-wrapper"><div class="swiper-slide"><blockquote><p>"Neopress delivered instant acceleration and rock-solid stability."</p><footer>— Technology Director</footer></blockquote></div></div></div></div></section>`
        }
    };

    function createNodeFromPaletteType(type) {
        if (ACF_SNIPPETS[type]) {
            return {
                id: window.generateId("acf_sec"),
                type: "CustomHtml",
                data: {
                    _name: ACF_SNIPPETS[type].name,
                    raw: ACF_SNIPPETS[type].raw
                },
                styles: {},
                children: []
            };
        }
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
        const state = window.NeoStudioState;
        const newNode = createNodeFromPaletteType(type);

        if (ACF_SNIPPETS[type]) {
            state.pageAst.root.children = state.pageAst.root.children || [];
            state.pageAst.root.children.push(newNode);
        } else {
            const currentId = state.activeElementId;
            const current = currentId ? window.findNodeById(currentId) : null;

            if (current && (current.type === "Container" || current.type === "Section" || current.type === "Column")) {
                current.children = current.children || [];
                current.children.push(newNode);
            } else if (current) {
                const parent = window.findParentNode(currentId);
                if (parent && parent.children) {
                    const idx = parent.children.findIndex(c => c.id === currentId);
                    parent.children.splice(idx + 1, 0, newNode);
                }
            } else {
                // Append to root inside new Section + Container
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
        }

        window.markDirty(true);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        if (window.selectElement) window.selectElement(newNode.id);
        if (window.showToast) window.showToast(`Added ${newNode.data && newNode.data._name ? newNode.data._name : type}`);
    }
    window.insertWidgetFromPalette = insertWidgetFromPalette;

    function insertWidgetRelativeTo(type, targetId, position) {
        const state = window.NeoStudioState;
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

        if (position === "inside" || ["Section", "Container", "Column", "Grid", "Flex"].includes(targetNode.type) && position !== "before" && position !== "after") {
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

    function renderTokensDrawer(container) {
        container.innerHTML = `
            <div class="drawer-header"><span class="drawer-title">Design Tokens</span></div>
            <div class="drawer-scroll-body" style="padding: 16px;">
                <h4 style="font-size: 11px; text-transform: uppercase; color: var(--neo-muted); margin-bottom: 10px;">5 Named Hex Palette</h4>
                <div style="display: flex; flex-direction: column; gap: 8px;">
                    <div style="display:flex; align-items:center; gap:8px;"><div style="width:18px;height:18px;background:#0D0F12;border:1px solid #242933;border-radius:2px;"></div><span style="font-size:12px;">Obsidian Chassis (#0D0F12)</span></div>
                    <div style="display:flex; align-items:center; gap:8px;"><div style="width:18px;height:18px;background:#15181E;border:1px solid #242933;border-radius:2px;"></div><span style="font-size:12px;">Graphite Surface (#15181E)</span></div>
                    <div style="display:flex; align-items:center; gap:8px;"><div style="width:18px;height:18px;background:#242933;border:1px solid #3B4455;border-radius:2px;"></div><span style="font-size:12px;">Hairline Rule (#242933)</span></div>
                    <div style="display:flex; align-items:center; gap:8px;"><div style="width:18px;height:18px;background:#F2F4F8;border:1px solid #242933;border-radius:2px;"></div><span style="font-size:12px;">Alabaster Ink (#F2F4F8)</span></div>
                    <div style="display:flex; align-items:center; gap:8px;"><div style="width:18px;height:18px;background:#8A93A6;border:1px solid #242933;border-radius:2px;"></div><span style="font-size:12px;">Slate Muted (#8A93A6)</span></div>
                </div>
            </div>
        `;
    }

    function renderHistoryDrawer(container) {
        const state = window.NeoStudioState;
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
                <p style="font-size: 12px; color: var(--neo-muted); margin-bottom: 12px;">Snapshots: ${state.historyStack.length} / ${state.maxHistory} (Current: #${state.historyIndex + 1})</p>
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
