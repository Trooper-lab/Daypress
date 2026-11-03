//! DayStudio Navigator & Canvas Right-Click CAD Context Menu (`scripts/navigator_context_menu.rs`).

pub const JS_NAVIGATOR_CONTEXT_MENU: &str = r###"
(function() {
    function closeNodeContextMenu() {
        const existing = document.getElementById("dp-nav-context-menu");
        if (existing) existing.remove();
    }
    window.closeNodeContextMenu = closeNodeContextMenu;

    document.addEventListener("click", () => closeNodeContextMenu());
    document.addEventListener("keydown", (e) => {
        if (e.key === "Escape") closeNodeContextMenu();
    });

    function openNodeContextMenu(clientX, clientY, nodeId) {
        closeNodeContextMenu();
        const clip = window._dayStudioClipboard || window._daystudioClipboard;
        const hasClipboard = !!clip;
        const clipLabel = clip && clip.type ? ` (${clip.type})` : "";

        const menu = document.createElement("div");
        menu.id = "dp-nav-context-menu";
        menu.className = "dp-nav-context-menu";
        menu.onclick = (e) => e.stopPropagation();

        if (!nodeId || nodeId === "root") {
            menu.innerHTML = `
                <div class="dp-ctx-header">
                    <span>Page Root / Empty Space</span>
                    <span style="opacity:0.6;">#root</span>
                </div>
                <button type="button" class="dp-ctx-item" data-act="paste-root" ${!hasClipboard ? "disabled" : ""}>
                    <span class="dp-ctx-left"><i class="fa-regular fa-paste" style="color:var(--day-token);"></i> Paste Below Everything${clipLabel}</span>
                    <span class="dp-ctx-kbd">Ctrl+V</span>
                </button>
                <div class="dp-ctx-sep"></div>
                <button type="button" class="dp-ctx-item" data-act="add-section">
                    <span class="dp-ctx-left"><i class="fa-solid fa-layer-group" style="color:var(--day-token);"></i> Add Section Below</span>
                    <span class="dp-ctx-kbd">SEC</span>
                </button>
                <button type="button" class="dp-ctx-item" data-act="add-container">
                    <span class="dp-ctx-left"><i class="fa-solid fa-vector-square" style="color:var(--day-signal);"></i> Add Container Below</span>
                    <span class="dp-ctx-kbd">DIV</span>
                </button>
            `;
            document.body.appendChild(menu);
            const rect = menu.getBoundingClientRect();
            menu.style.left = Math.min(clientX, Math.max(8, window.innerWidth - rect.width - 10)) + "px";
            menu.style.top = Math.min(clientY, Math.max(8, window.innerHeight - rect.height - 10)) + "px";
            menu.querySelectorAll(".dp-ctx-item").forEach(btn => {
                btn.onclick = (e) => {
                    e.stopPropagation();
                    if (btn.disabled) return;
                    const act = btn.dataset.act;
                    closeNodeContextMenu();
                    if (act === "paste-root" && window.pasteNode) window.pasteNode("root");
                    else if (act === "add-section" && window.insertSectionPreset) window.insertSectionPreset("1");
                    else if (act === "add-container") {
                        window.DayStudioState.activeElementId = null;
                        if (window.insertWidgetFromPalette) window.insertWidgetFromPalette("Container");
                    }
                };
            });
            return;
        }

        const node = window.findNodeById ? window.findNodeById(nodeId) : null;
        if (!node) return;

        const isLocked = !!(node.data && node.data._locked);
        const isHidden = !!(node.styles && node.styles.display === "none");
        const displayName = (node.data && node.data._name) ? node.data._name : (node.type || "Element");

        menu.innerHTML = `
            <div class="dp-ctx-header">
                <span>${displayName.slice(0, 22)}</span>
                <span style="opacity:0.6;">#${nodeId.slice(0, 6)}</span>
            </div>
            <button type="button" class="dp-ctx-item" data-act="wrap-container" ${isLocked ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-solid fa-vector-square" style="color:var(--day-signal);"></i> Wrap in Container</span>
                <span class="dp-ctx-kbd">DIV</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="wrap-section" ${isLocked ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-solid fa-layer-group" style="color:var(--day-token);"></i> Wrap in Section</span>
                <span class="dp-ctx-kbd">SEC</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="wrap-grid" ${isLocked ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-solid fa-border-all" style="color:var(--day-emerald);"></i> Wrap in 2-Col Grid</span>
                <span class="dp-ctx-kbd">GRID</span>
            </button>
            <div class="dp-ctx-sep"></div>
            <button type="button" class="dp-ctx-item" data-act="copy">
                <span class="dp-ctx-left"><i class="fa-regular fa-copy"></i> Copy</span>
                <span class="dp-ctx-kbd">Ctrl+C</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="cut" ${isLocked ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-solid fa-scissors"></i> Cut</span>
                <span class="dp-ctx-kbd">Ctrl+X</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="paste" ${!hasClipboard ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-regular fa-paste"></i> Paste Here${clipLabel}</span>
                <span class="dp-ctx-kbd">Ctrl+V</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="paste-root" ${!hasClipboard ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-solid fa-arrow-turn-down" style="color:var(--day-token);"></i> Paste Below Everything</span>
                <span class="dp-ctx-kbd">Root</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="duplicate" ${isLocked ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-regular fa-clone"></i> Duplicate</span>
                <span class="dp-ctx-kbd">Ctrl+D</span>
            </button>
            <div class="dp-ctx-sep"></div>
            <button type="button" class="dp-ctx-item" data-act="rename">
                <span class="dp-ctx-left"><i class="fa-solid fa-i-cursor"></i> Rename Layer</span>
                <span class="dp-ctx-kbd">DblClick</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="toggle-lock">
                <span class="dp-ctx-left"><i class="fa-solid ${isLocked ? 'fa-lock-open' : 'fa-lock'}"></i> ${isLocked ? "Unlock Layer" : "Lock Layer"}</span>
            </button>
            <button type="button" class="dp-ctx-item" data-act="toggle-vis">
                <span class="dp-ctx-left"><i class="fa-regular ${isHidden ? 'fa-eye' : 'fa-eye-slash'}"></i> ${isHidden ? "Show Layer" : "Hide Layer"}</span>
            </button>
            ${node.type !== "SymbolInstance" ? `
            <button type="button" class="dp-ctx-item" data-act="to-symbol" ${isLocked ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-solid fa-cube" style="color:var(--day-violet);"></i> Convert to Global Symbol</span>
            </button>` : ""}
            <div class="dp-ctx-sep"></div>
            <button type="button" class="dp-ctx-item danger" data-act="delete" ${isLocked ? "disabled" : ""}>
                <span class="dp-ctx-left"><i class="fa-solid fa-trash-can"></i> Delete Element</span>
                <span class="dp-ctx-kbd">Del</span>
            </button>
        `;

        document.body.appendChild(menu);

        // Keep context menu inside viewport bounds
        const rect = menu.getBoundingClientRect();
        const maxLeft = Math.max(8, window.innerWidth - rect.width - 10);
        const maxTop = Math.max(8, window.innerHeight - rect.height - 10);
        menu.style.left = Math.min(clientX, maxLeft) + "px";
        menu.style.top = Math.min(clientY, maxTop) + "px";

        menu.querySelectorAll(".dp-ctx-item").forEach(btn => {
            btn.onclick = (e) => {
                e.stopPropagation();
                if (btn.disabled) return;
                const act = btn.dataset.act;
                closeNodeContextMenu();

                if (act === "wrap-container" && window.wrapNodeIn) window.wrapNodeIn(nodeId, "Container");
                else if (act === "wrap-section" && window.wrapNodeIn) window.wrapNodeIn(nodeId, "Section");
                else if (act === "wrap-grid" && window.wrapNodeIn) window.wrapNodeIn(nodeId, "Grid");
                else if (act === "copy" && window.copyNode) window.copyNode(nodeId);
                else if (act === "cut" && window.cutNode) window.cutNode(nodeId);
                else if (act === "paste" && window.pasteNode) window.pasteNode(nodeId);
                else if (act === "paste-root" && window.pasteNode) window.pasteNode("root");
                else if (act === "duplicate" && window.duplicateNode) window.duplicateNode(nodeId);
                else if (act === "delete" && window.deleteNode) window.deleteNode(nodeId);
                else if (act === "toggle-lock" && window.toggleNodeLock) window.toggleNodeLock(node);
                else if (act === "toggle-vis" && window.toggleNodeVisibility) window.toggleNodeVisibility(node);
                else if (act === "to-symbol") {
                    if (window.convertActiveNodeToSymbol) window.convertActiveNodeToSymbol(node);
                    else if (window.convertToGlobalSymbol) window.convertToGlobalSymbol(nodeId);
                }
                else if (act === "rename") {
                    if (window.expandNavigatorToNode) window.expandNavigatorToNode(nodeId);
                    const labelEl = document.querySelector(`.nav-tree-branch[data-node-id="${nodeId}"] > .nav-tree-item .nav-node-label`);
                    if (labelEl && window.enterInlineRename) {
                        window.enterInlineRename(labelEl, node);
                    } else {
                        const nextName = prompt("Layer name:", displayName);
                        if (nextName !== null) {
                            if (!node.data) node.data = {};
                            node.data._name = nextName.trim() || node.type;
                            window.markDirty(true);
                            if (window.updateNavigator) window.updateNavigator();
                            if (window.updateBreadcrumbs) window.updateBreadcrumbs(nodeId);
                        }
                    }
                }
            };
        });
    }
    window.openNodeContextMenu = openNodeContextMenu;
})();
"###;
