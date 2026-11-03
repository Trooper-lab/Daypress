//! Navigator Layers Tree, Drag-and-Drop Reordering, Expand/Collapse, Visibility, Lock and Inline Rename (`scripts/navigator.rs`).

pub const JS_NAVIGATOR: &str = r###"
(function() {
    let collapsedNodes = new Set();

    function renderNavigatorDrawer(container) {
        if (!container) return;
        container.innerHTML = `
            <div class="drawer-header"><span class="drawer-title">Navigator</span><div class="drawer-header-actions"><button class="drawer-action-btn" id="nav-btn-expand" title="Expand All"><i class="fa-solid fa-arrows-up-down"></i></button><button class="drawer-action-btn" id="nav-btn-collapse" title="Collapse All"><i class="fa-solid fa-compress"></i></button></div></div>
            <div class="drawer-search-well"><input type="search" class="drawer-search-input" id="nav-search-input" placeholder="Filter layers (Ctrl+F)..."></div>
            <div class="drawer-scroll-body" id="navigator-tree-body"></div>
        `;
        const expandBtn = document.getElementById("nav-btn-expand");
        if (expandBtn) expandBtn.onclick = () => { collapsedNodes.clear(); updateNavigator(); };
        const collapseBtn = document.getElementById("nav-btn-collapse");
        if (collapseBtn) collapseBtn.onclick = () => {
            const root = window.DayStudioState.pageAst ? window.DayStudioState.pageAst.root : null;
            if (root && root.children) root.children.forEach(c => collapsedNodes.add(c.id));
            updateNavigator();
        };
        const searchInput = document.getElementById("nav-search-input");
        if (searchInput) searchInput.oninput = () => filterNavigator(searchInput.value.toLowerCase().trim());
        const treeBody = document.getElementById("navigator-tree-body");
        if (treeBody) { treeBody.onclick = (e) => { if (!e.target.closest(".nav-tree-item") && window.selectElement) window.selectElement(null); }; treeBody.oncontextmenu = (e) => { if (!e.target.closest(".nav-tree-item") && window.openNodeContextMenu) { e.preventDefault(); if (window.selectElement) window.selectElement(null); window.openNodeContextMenu(e.clientX, e.clientY, "root"); } }; }
        updateNavigator();
    }
    window.renderNavigatorDrawer = renderNavigatorDrawer;

    function updateNavigator() {
        const body = document.getElementById("navigator-tree-body");
        if (!body) return;
        body.innerHTML = "";
        const state = window.DayStudioState;
        if (!state.pageAst || !state.pageAst.root) return;
        const rootRow = document.createElement("div");
        rootRow.className = "nav-tree-branch";
        rootRow.dataset.nodeId = "root";
        rootRow.innerHTML = `<div class="nav-tree-item${!state.activeElementId || state.activeElementId === 'root' ? ' active' : ''}" data-node-cat="section"><div class="nav-tree-left"><span class="nav-node-icon"><i class="fa-solid fa-window-maximize"></i></span><span class="nav-node-label">Body (Page & SEO)</span><span class="nav-type-pill">BODY</span></div></div>`;
        rootRow.onclick = () => { if (window.selectElement) window.selectElement("root", { fromNavigator: true }); };
        rootRow.oncontextmenu = (e) => { e.preventDefault(); e.stopPropagation(); if (window.selectElement) window.selectElement("root", { fromNavigator: true }); if (window.openNodeContextMenu) window.openNodeContextMenu(e.clientX, e.clientY, "root"); };
        body.appendChild(rootRow);
        for (const child of (state.pageAst.root.children || [])) {
            const nodeEl = buildNavTreeRow(child, 0);
            if (nodeEl) body.appendChild(nodeEl);
        }
        syncNavigatorSelection(state.activeElementId || "root");
    }
    window.updateNavigator = updateNavigator;

    function moveNode(nodeId, targetId, position) {
        if (!nodeId || !targetId || nodeId === targetId || nodeId === "root") return false;
        if (window.isDescendant && window.isDescendant(nodeId, targetId)) {
            console.warn("Cycle prevention: cannot move node into its own descendant");
            if (window.showToast) window.showToast("Cannot move parent into its own child", "warning");
            return false;
        }
        const state = window.DayStudioState;
        if (!state.pageAst || !state.pageAst.root) return false;
        const sourceNode = window.findNodeById(nodeId);
        const targetNode = window.findNodeById(targetId);
        if (!sourceNode || !targetNode) return false;

        if (sourceNode.data && sourceNode.data._locked) {
            if (window.showToast) window.showToast("Cannot move locked element", "warning");
            return false;
        }

        const sourceParent = window.findParentNode(nodeId);
        if (!sourceParent || !sourceParent.children) return false;
        if (sourceParent.data && sourceParent.data._locked) {
            if (window.showToast) window.showToast("Parent container is locked", "warning");
            return false;
        }

        if (position === "inside") {
            if (targetNode.data && targetNode.data._locked) { if (window.showToast) window.showToast("Cannot drop into locked container", "warning"); return false; }
        } else {
            const targetParent = window.findParentNode(targetId);
            if (targetParent && targetParent.data && targetParent.data._locked) { if (window.showToast) window.showToast("Target container is locked", "warning"); return false; }
        }
        const sourceIdx = sourceParent.children.findIndex(c => c.id === nodeId);
        if (sourceIdx === -1) return false;
        const [detached] = sourceParent.children.splice(sourceIdx, 1);

        if (position === "inside") {
            if (!targetNode.children) targetNode.children = [];
            targetNode.children.push(detached);
        } else {
            const targetParent = window.findParentNode(targetId);
            if (!targetParent || !targetParent.children) {
                if (!targetNode.children) targetNode.children = [];
                targetNode.children.push(detached);
            } else {
                const targetIdx = targetParent.children.findIndex(c => c.id === targetId);
                const insertIdx = position === "before" ? targetIdx : targetIdx + 1;
                targetParent.children.splice(insertIdx, 0, detached);
            }
        }

        window.markDirty(true);
        window.pushHistorySnapshot();
        if (window.renderCanvas) window.renderCanvas();
        updateNavigator();
        if (window.selectElement) window.selectElement(detached.id);
        return true;
    }
    window.moveNode = moveNode;

    function clearDropIndicators() {
        document.querySelectorAll(".day-drop-before, .day-drop-after, .day-drop-inside")
            .forEach(el => el.classList.remove("day-drop-before", "day-drop-after", "day-drop-inside"));
    }

    function computeDropPosition(e, rowEl, node) {
        if (node.id === "root") return "inside";
        const rect = rowEl.getBoundingClientRect();
        const relY = e.clientY - rect.top;
        const canHold = ["Root", "Section", "Container", "Column", "MegaMenuContainer", "PopupModal", "Accordion", "Tabs", "LoopGrid"].includes(node.type)
            || (Array.isArray(node.children) && !["Heading", "Text", "Button", "Image", "PriceCard", "Countdown"].includes(node.type));
        if (canHold) {
            if (relY < rect.height * 0.28) return "before";
            if (relY > rect.height * 0.72) return "after";
            return "inside";
        }
        return relY < rect.height * 0.5 ? "before" : "after";
    }

    function updateDropIndicator(rowEl, pos) {
        rowEl.classList.remove("day-drop-before", "day-drop-after", "day-drop-inside");
        if (pos === "before") rowEl.classList.add("day-drop-before");
        else if (pos === "after") rowEl.classList.add("day-drop-after");
        else if (pos === "inside") rowEl.classList.add("day-drop-inside");
    }

    function buildNavTreeRow(node, depth) {
        if (!node) return null;
        const container = document.createElement("div");
        container.className = "nav-tree-branch";
        container.dataset.nodeId = node.id;

        const row = document.createElement("div");
        row.className = "nav-tree-item" + (node.id === window.DayStudioState.activeElementId ? " active" : "");
        row.dataset.nodeCat = getNodeCategory(node.type);
        const isLocked = node.data && node.data._locked;
        if (isLocked) row.classList.add("is-locked");
        if (node.styles && node.styles.display === "none") row.classList.add("is-hidden");

        row.onmouseenter = () => { if (window.onNodeHover) window.onNodeHover(node.id); };
        row.onmouseleave = () => { if (window.onNodeHover) window.onNodeHover(null); };
        row.oncontextmenu = (e) => {
            e.preventDefault();
            e.stopPropagation();
            if (window.selectElement) window.selectElement(node.id, { fromNavigator: true });
            if (window.openNodeContextMenu) window.openNodeContextMenu(e.clientX, e.clientY, node.id);
        };

        const isDraggable = node.id !== "root" && !isLocked;
        row.draggable = isDraggable;
        if (isDraggable) row.setAttribute("draggable", "true");

        row.ondragstart = (e) => {
            if (!isDraggable || (node.data && node.data._locked) || node.id === "root") {
                e.preventDefault();
                return;
            }
            e.stopPropagation();
            e.dataTransfer.setData("text/plain", node.id);
            e.dataTransfer.setData("application/x-daystudio-node", node.id);
            e.dataTransfer.effectAllowed = "move";
            row.classList.add("is-dragging");
            window._draggedNavNodeId = node.id;
        };

        row.ondragend = (e) => {
            e.stopPropagation();
            row.classList.remove("is-dragging");
            window._draggedNavNodeId = null;
            clearDropIndicators();
        };

        row.ondragover = (e) => {
            e.preventDefault();
            e.stopPropagation();
            if (window._draggedPaletteType) {
                e.dataTransfer.dropEffect = "copy";
                const pos = computeDropPosition(e, row, node);
                updateDropIndicator(row, pos);
                return;
            }
            const draggedId = window._draggedNavNodeId;
            if (!draggedId || draggedId === node.id) {
                e.dataTransfer.dropEffect = "none";
                return;
            }
            if (window.isDescendant && window.isDescendant(draggedId, node.id)) {
                e.dataTransfer.dropEffect = "none";
                return;
            }
            e.dataTransfer.dropEffect = "move";
            const pos = computeDropPosition(e, row, node);
            updateDropIndicator(row, pos);
        };

        row.ondragleave = (e) => {
            e.stopPropagation();
            row.classList.remove("day-drop-before", "day-drop-after", "day-drop-inside");
        };

        row.ondrop = (e) => {
            e.preventDefault();
            e.stopPropagation();
            const rawText = e.dataTransfer ? e.dataTransfer.getData("text/plain") : "";
            const paletteType = window._draggedPaletteType || (rawText.startsWith("palette:") ? rawText.slice(8) : null);
            clearDropIndicators();
            const pos = computeDropPosition(e, row, node);
            if (paletteType && window.insertWidgetRelativeTo) {
                window._draggedPaletteType = null;
                window.insertWidgetRelativeTo(paletteType, node.id, pos);
                return;
            }
            const draggedId = window._draggedNavNodeId || rawText;
            if (!draggedId || draggedId === node.id) return;
            if (window.isDescendant && window.isDescendant(draggedId, node.id)) return;
            moveNode(draggedId, node.id, pos);
        };

        const hasChildren = node.children && node.children.length > 0;
        const isCollapsed = collapsedNodes.has(node.id);
        const left = document.createElement("div");
        left.className = "nav-tree-left";
        for (let i = 0; i < depth; i++) {
            const indent = document.createElement("div");
            indent.className = "nav-indent";
            left.appendChild(indent);
        }
        const chev = document.createElement("span");
        chev.className = "nav-chevron" + (!isCollapsed ? " open" : "");
        if (hasChildren) {
            chev.innerHTML = `<i class="fa-solid fa-chevron-right"></i>`;
            chev.onclick = (e) => {
                e.stopPropagation();
                if (isCollapsed) collapsedNodes.delete(node.id);
                else collapsedNodes.add(node.id);
                updateNavigator();
            };
        }
        left.appendChild(chev);
        const icon = document.createElement("span");
        icon.className = "nav-node-icon";
        icon.innerHTML = `<i class="${getNodeIconClass(node.type)}"></i>`;
        left.appendChild(icon);
        const label = document.createElement("span");
        label.className = "nav-node-label";
        label.textContent = getNodeDisplayLabel(node);
        label.ondblclick = (e) => { e.stopPropagation(); enterInlineRename(label, node); };
        left.appendChild(label);
        const pillText = getNodeShortPill(node);
        if (pillText) {
            const pill = document.createElement("span");
            pill.className = "nav-type-pill";
            if (pillText.startsWith("⚡")) pill.style.cssText = "color:var(--day-token);border-color:rgba(230,184,74,0.45);background:rgba(230,184,74,0.12);";
            pill.textContent = pillText;
            left.appendChild(pill);
        }
        row.appendChild(left);
        const actions = document.createElement("div");
        actions.className = "nav-actions";
        const btnEye = document.createElement("button");
        btnEye.className = "nav-act-btn act-eye";
        const isHidden = node.styles && node.styles.display === "none";
        btnEye.innerHTML = `<i class="${isHidden ? 'fa-regular fa-eye-slash' : 'fa-regular fa-eye'}"></i>`;
        btnEye.onclick = (e) => { e.stopPropagation(); toggleNodeVisibility(node); };
        actions.appendChild(btnEye);
        const btnLock = document.createElement("button");
        btnLock.className = "nav-act-btn act-lock";
        btnLock.innerHTML = `<i class="${isLocked ? 'fa-solid fa-lock' : 'fa-solid fa-lock-open'}"></i>`;
        btnLock.onclick = (e) => { e.stopPropagation(); toggleNodeLock(node); };
        actions.appendChild(btnLock);
        row.appendChild(actions);
        row.onclick = () => { if (window.selectElement) window.selectElement(node.id, { fromNavigator: true }); };
        container.appendChild(row);
        if (hasChildren && !isCollapsed) {
            const childrenCont = document.createElement("div");
            childrenCont.className = "nav-children-wrapper";
            for (const child of node.children) {
                const childRow = buildNavTreeRow(child, depth + 1);
                if (childRow) childrenCont.appendChild(childRow);
            }
            container.appendChild(childrenCont);
        }
        return container;
    }

    function toggleNodeVisibility(node) {
        if (!node.styles) node.styles = {};
        if (node.styles.display === "none") delete node.styles.display;
        else node.styles.display = "none";
        window.markDirty(true);
        window.renderCanvas();
        updateNavigator();
    }
    function toggleNodeLock(node) {
        if (!node.data) node.data = {};
        node.data._locked = !node.data._locked;
        window.markDirty(true);
        updateNavigator();
        if (window.renderInspector) window.renderInspector();
    }
    function enterInlineRename(labelEl, node) {
        const input = document.createElement("input");
        input.type = "text";
        input.className = "nav-rename-input";
        input.value = labelEl.textContent;
        labelEl.replaceWith(input);
        input.focus();
        input.select();
        function commit() {
            const newName = input.value.trim() || node.type;
            if (!node.data) node.data = {};
            node.data._name = newName;
            window.markDirty(true);
            updateNavigator();
            if (window.updateBreadcrumbs) window.updateBreadcrumbs(node.id);
            if (window.updateSelectionChrome) window.updateSelectionChrome();
        }
        input.onblur = commit;
        input.onkeydown = (e) => {
            if (e.key === "Enter") commit();
            if (e.key === "Escape") updateNavigator();
        };
    }
    function getNodeDisplayLabel(node) {
        if (node.data && node.data._name) return node.data._name;
        if (node.type === "CustomHtml" && node.data && node.data.raw) {
            const raw = node.data.raw;
            const headingMatch = raw.match(/<h[1-4][^>]*>([\s\S]*?)<\/h[1-4]>/i);
            const headingText = headingMatch ? headingMatch[1].replace(/<[^>]*>/g, "").trim() : "";
            if (raw.includes("pageheader")) return headingText ? `Hero: ${headingText.slice(0, 26)}` : "Hero Pageheader (ACF)";
            if (raw.includes("logoSwiper") || raw.includes("section logo")) return "ACF Block: Client Logos";
            if (raw.includes("threeSwiper")) return headingText ? `ACF 3-Col: ${headingText.slice(0, 22)}` : "ACF Block: 3-Col Swiper";
            if (raw.includes("trainSwiper")) return "ACF Block: Showcase Carousel";
            if (raw.includes("quoteSwiper")) return "ACF Block: Quotes Slider";
            if (raw.includes("accordion")) return "ACF Block: FAQ Accordion";
            return headingText ? `ACF Section: ${headingText.slice(0, 24)}` : "ACF Theme Section";
        }
        return node.type || "Element";
    }
    window.toggleNodeVisibility = toggleNodeVisibility;
    window.toggleNodeLock = toggleNodeLock;
    window.enterInlineRename = enterInlineRename;

    function getNodeCategory(type) {
        if (["Section", "MegaMenuContainer", "PopupModal"].includes(type)) return "section";
        if (["Container", "Grid", "Flex", "Accordion", "Tabs"].includes(type)) return "container";
        if (type === "Column") return "column";
        if (["SymbolInstance", "NavMenu", "LoopGrid", "PostTitle", "PostContent", "FeaturedImage", "PostMeta", "Breadcrumb", "TableOfContents"].includes(type)) return "symbol";
        if (["Button", "Form", "NativeForm", "CallToAction", "PriceCard", "PriceTable", "Countdown", "LoginForm"].includes(type)) return "action";
        if (["Image", "Video", "Icon", "Counter", "ProgressBar", "StarRating", "SocialIcons", "ShareButtons", "Testimonial"].includes(type)) return "media";
        return "text";
    }
    function getNodeShortPill(node) {
        if (node && node.styles && node.styles.raw_settings && node.styles.raw_settings["data-dynamic"]) return "⚡ DYN";
        const type = (node && node.type) || node;
        const pills = { Section: "SEC", Container: "CONT", Grid: "GRID", Flex: "FLEX", Column: "COL", SymbolInstance: "SYM", NavMenu: "NAV", MegaMenuContainer: "MEGA", PopupModal: "POP", LoopGrid: "LOOP", PostTitle: "⚡ DYN", PostContent: "⚡ DYN" };
        return pills[type] || "";
    }
    function getNodeIconClass(type) {
        const map = { Section: "fa-layer-group", Container: "fa-vector-square", Grid: "fa-border-all", Flex: "fa-arrows-left-right", Column: "fa-table-columns", Heading: "fa-heading", Text: "fa-paragraph", Button: "fa-arrow-pointer", Image: "fa-image", Video: "fa-film", Icon: "fa-icons", PriceCard: "fa-credit-card", Countdown: "fa-clock", NavMenu: "fa-compass", MegaMenuContainer: "fa-sitemap", SymbolInstance: "fa-cube", LoopGrid: "fa-grip", CustomHtml: "fa-code" };
        return "fa-solid " + (map[type] || "fa-cube");
    }
    function expandNavigatorToNode(id) {
        if (!id || !window.findParentNode) return;
        let changed = false, curr = window.findParentNode(id);
        while (curr && curr.id !== "root") {
            if (collapsedNodes.has(curr.id)) { collapsedNodes.delete(curr.id); changed = true; }
            curr = window.findParentNode(curr.id);
        }
        if (changed) updateNavigator();
    }
    window.expandNavigatorToNode = expandNavigatorToNode;

    function syncNavigatorSelection(id) {
        document.querySelectorAll(".nav-tree-item").forEach(el => el.classList.remove("active"));
        if (!id) return;
        const targetRow = document.querySelector(`.nav-tree-branch[data-node-id="${id}"] > .nav-tree-item`);
        if (targetRow) { targetRow.classList.add("active"); targetRow.scrollIntoView({ block: "nearest", behavior: "smooth" }); }
    }
    window.syncNavigatorSelection = syncNavigatorSelection;

    function filterNavigator(query) {
        document.querySelectorAll(".nav-tree-branch").forEach(branch => {
            const label = branch.querySelector(".nav-node-label");
            if (label) branch.style.display = (!query || label.textContent.toLowerCase().includes(query)) ? "" : "none";
        });
    }
})();
"###;
