//! Selection Chrome, Bounding Box, Clipboard/Wrap Actions, Bidirectional Highlight & Breadcrumbs (`scripts/canvas_selection.rs`).

pub const JS_CANVAS_SELECTION: &str = r###"
(function() {
    window._dayStudioClipboard = null;

    function selectElement(id, opts) {
        const state = window.DayStudioState;
        state.activeElementId = id;

        // If selected from Navigator and node is inside a Mega Menu, ensure canvas renders it open
        let targetEl = id ? document.querySelector(`#daystudio-canvas [data-node-id="${id}"]`) : null;
        if (id && !targetEl && window.renderCanvas) {
            window.renderCanvas();
            targetEl = document.querySelector(`#daystudio-canvas [data-node-id="${id}"]`);
        }

        // Highlight active node on canvas
        document.querySelectorAll("#daystudio-canvas .daystudio-node.is-selected").forEach(el => el.classList.remove("is-selected"));
        if (targetEl) {
            targetEl.classList.add("is-selected");
            if (opts && opts.fromNavigator) {
                targetEl.scrollIntoView({ behavior: "smooth", block: "nearest", inline: "nearest" });
                targetEl.classList.remove("is-flash-highlight");
                void targetEl.offsetWidth;
                targetEl.classList.add("is-flash-highlight");
                setTimeout(() => targetEl.classList.remove("is-flash-highlight"), 800);
                setTimeout(updateSelectionChrome, 160);
                setTimeout(updateSelectionChrome, 350);
            }
        }

        if (!(opts && opts.fromNavigator) && window.expandNavigatorToNode) {
            window.expandNavigatorToNode(id);
        }
        updateSelectionChrome();
        updateBreadcrumbs(id);

        if (window.renderInspector) window.renderInspector();
        if (window.syncNavigatorSelection) window.syncNavigatorSelection(id);
        if (window.updateBoxOverlays) window.updateBoxOverlays(id);
        if (window.refreshCssDrawerSelection) window.refreshCssDrawerSelection();
    }
    window.selectElement = selectElement;

    function onNodeHover(id) {
        document.querySelectorAll("#daystudio-canvas .daystudio-node.is-hovered").forEach(el => el.classList.remove("is-hovered"));
        document.querySelectorAll(".nav-tree-item.is-canvas-hovered").forEach(el => el.classList.remove("is-canvas-hovered"));
        if (!id) return;
        const canvasEl = document.querySelector(`#daystudio-canvas [data-node-id="${id}"]`);
        if (canvasEl) canvasEl.classList.add("is-hovered");
        const navRow = document.querySelector(`.nav-tree-branch[data-node-id="${id}"] > .nav-tree-item`);
        if (navRow) navRow.classList.add("is-canvas-hovered");
    }
    window.onNodeHover = onNodeHover;

    function updateSelectionChrome() {
        const chrome = document.getElementById("daystudio-selection-chrome");
        const id = window.DayStudioState.activeElementId;
        if (!chrome || !id || id === "root") {
            if (chrome) chrome.style.display = "none";
            return;
        }

        const targetEl = document.querySelector(`#daystudio-canvas [data-node-id="${id}"]`);
        if (!targetEl) {
            chrome.style.display = "none";
            return;
        }

        const rect = targetEl.getBoundingClientRect();
        const viewport = document.getElementById("daystudio-canvas-viewport");
        if (!viewport) return;
        const vpRect = viewport.getBoundingClientRect();

        const top = rect.top - vpRect.top + viewport.scrollTop;
        const left = rect.left - vpRect.left + viewport.scrollLeft;

        chrome.style.display = "block";
        chrome.style.top = top + "px";
        chrome.style.left = left + "px";
        chrome.style.width = rect.width + "px";
        chrome.style.height = rect.height + "px";
        chrome.classList.toggle("badge-inside", top < 26);

        const node = window.findNodeById(id);
        if (node) {
            chrome.dataset.selType = (node.type || "Element").toLowerCase();
            const lblKind = document.getElementById("sel-badge-kind");
            const lblId = document.getElementById("sel-badge-id");
            const dynTag = node.styles && node.styles.raw_settings && node.styles.raw_settings["data-dynamic"];
            const dynMeta = dynTag && window.DYNAMIC_TAG_META && window.DYNAMIC_TAG_META[dynTag];
            if (lblKind) lblKind.textContent = (node.data && node.data._name) || node.type || "Element";
            if (lblId) lblId.textContent = dynTag ? (`⚡ ${dynMeta ? dynMeta.token : dynTag} • #${node.id}`) : ("#" + node.id);
        }
    }
    window.updateSelectionChrome = updateSelectionChrome;

    function reassignCloneIds(node) {
        node.id = window.generateId(node.type ? node.type.toLowerCase() : "node");
        if (node.children) {
            for (const child of node.children) reassignCloneIds(child);
        }
    }
    window.reassignCloneIds = reassignCloneIds;

    function copyNode(id) {
        const targetId = id || window.DayStudioState.activeElementId;
        if (!targetId || targetId === "root") return false;
        const node = window.findNodeById(targetId);
        if (!node) return false;
        const cloned = JSON.parse(JSON.stringify(node));
        window._dayStudioClipboard = cloned;
        window._daystudioClipboard = cloned;
        if (window.showToast) window.showToast(`Copied ${node.type} (#${node.id})`);
        return true;
    }
    window.copyNode = copyNode;

    function cutNode(id) {
        const targetId = id || window.DayStudioState.activeElementId;
        if (!targetId || targetId === "root") return false;
        if (!copyNode(targetId)) return false;
        deleteNode(targetId);
        if (window.showToast) window.showToast("Cut element to clipboard");
        return true;
    }
    window.cutNode = cutNode;

    function pasteNode(targetId) {
        const clip = window._dayStudioClipboard || window._daystudioClipboard;
        if (!clip) {
            if (window.showToast) window.showToast("Clipboard is empty", "warning");
            return false;
        }
        const state = window.DayStudioState;
        const destId = targetId || state.activeElementId || "root";
        const destNode = destId === "root" ? state.pageAst.root : window.findNodeById(destId);
        if (!destNode) return false;

        const clone = JSON.parse(JSON.stringify(clip));
        reassignCloneIds(clone);

        const isCont = ["Root", "Section", "Container", "Column", "Grid", "Flex", "MegaMenuContainer"].includes(destNode.type) || destId === "root";
        if (isCont) {
            destNode.children = destNode.children || [];
            destNode.children.push(clone);
        } else {
            const parent = window.findParentNode(destId);
            if (parent && Array.isArray(parent.children)) {
                const idx = parent.children.findIndex(c => c.id === destId);
                parent.children.splice(idx + 1, 0, clone);
            } else {
                state.pageAst.root.children.push(clone);
            }
        }

        window.markDirty(true);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        selectElement(clone.id);
        if (window.showToast) window.showToast(`Pasted ${clone.type}`);
        return true;
    }
    window.pasteNode = pasteNode;

    function wrapNodeIn(id, wrapperType) {
        if (!id || id === "root") return false;
        const node = window.findNodeById(id);
        if (!node) return false;
        if (node.data && node.data._locked) {
            if (window.showToast) window.showToast("Cannot wrap locked element", "warning");
            return false;
        }
        const parent = window.findParentNode(id);
        if (!parent || !Array.isArray(parent.children)) return false;
        const idx = parent.children.findIndex(c => c.id === id);
        if (idx === -1) return false;

        const wType = wrapperType || "Container";
        const wrapper = {
            id: window.generateId(wType.toLowerCase()),
            type: wType,
            data: wType === "Grid" ? { columns: 2, gap: 20 } : {},
            styles: wType === "Section"
                ? { padding_top: "48px", padding_bottom: "48px" }
                : (wType === "Grid" ? { display: "grid", grid_template_columns: "repeat(2, 1fr)", gap: "20px" } : {}),
            children: [node]
        };

        parent.children.splice(idx, 1, wrapper);
        window.markDirty(true);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        selectElement(wrapper.id);
        if (window.showToast) window.showToast(`Wrapped in ${wType}`);
        return true;
    }
    window.wrapNodeIn = wrapNodeIn;

    function duplicateNode(id) {
        if (!id || id === "root") return;
        const node = window.findNodeById(id);
        if (node && node.data && node.data._locked) {
            if (window.showToast) window.showToast("Cannot duplicate locked element", "warning");
            return;
        }
        const parent = window.findParentNode(id);
        if (!parent || !parent.children) return;
        if (parent.data && parent.data._locked) {
            if (window.showToast) window.showToast("Cannot duplicate in locked container", "warning");
            return;
        }

        const idx = parent.children.findIndex(c => c.id === id);
        if (idx === -1) return;

        const orig = parent.children[idx];
        const clone = JSON.parse(JSON.stringify(orig));
        reassignCloneIds(clone);

        parent.children.splice(idx + 1, 0, clone);
        window.markDirty(true);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        selectElement(clone.id);
    }
    window.duplicateNode = duplicateNode;

    function deleteNode(id) {
        if (!id || id === "root") return;
        const node = window.findNodeById(id);
        if (node && node.data && node.data._locked) {
            if (window.showToast) window.showToast("Cannot delete locked element", "warning");
            return;
        }
        const parent = window.findParentNode(id);
        if (!parent || !parent.children) return;
        if (parent.data && parent.data._locked) {
            if (window.showToast) window.showToast("Cannot delete child of locked container", "warning");
            return;
        }

        const idx = parent.children.findIndex(c => c.id === id);
        if (idx === -1) return;

        parent.children.splice(idx, 1);
        window.DayStudioState.activeElementId = parent.id;
        window.markDirty(true);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        selectElement(parent.id);
    }
    window.deleteNode = deleteNode;

    function initActionChips() {
        const actParent = document.getElementById("act-parent-node");
        const actDup = document.getElementById("act-dup-node");
        const actSaveSym = document.getElementById("act-save-symbol-node");
        const actDel = document.getElementById("act-del-node");

        if (actParent) {
            actParent.onclick = (e) => {
                e.stopPropagation();
                const curId = window.DayStudioState.activeElementId;
                if (!curId) return;
                const parent = window.findParentNode(curId);
                if (parent && parent.id) selectElement(parent.id);
            };
        }
        if (actDup) {
            actDup.onclick = (e) => {
                e.stopPropagation();
                const curId = window.DayStudioState.activeElementId;
                if (curId) duplicateNode(curId);
            };
        }
        if (actSaveSym) {
            actSaveSym.onclick = (e) => {
                e.stopPropagation();
                const curId = window.DayStudioState.activeElementId;
                if (curId && window.convertToGlobalSymbol) window.convertToGlobalSymbol(curId);
            };
        }
        if (actDel) {
            actDel.onclick = (e) => {
                e.stopPropagation();
                const curId = window.DayStudioState.activeElementId;
                if (curId) deleteNode(curId);
            };
        }
    }

    function updateBreadcrumbs(id) {
        const trail = document.getElementById("daystudio-breadcrumbs");
        if (!trail) return;
        trail.innerHTML = "";

        if (!id || id === "root") {
            const rootSpan = document.createElement("span");
            rootSpan.className = "bc-item active";
            rootSpan.dataset.id = "root";
            rootSpan.textContent = "Body";
            rootSpan.onclick = () => selectElement("root", { fromNavigator: true });
            trail.appendChild(rootSpan);
            return;
        }

        const ancestors = [];
        let curr = window.findNodeById(id);
        while (curr) {
            ancestors.unshift(curr);
            curr = window.findParentNode(curr.id);
        }

        ancestors.forEach((node, idx) => {
            const span = document.createElement("span");
            span.className = "bc-item" + (idx === ancestors.length - 1 ? " active" : "");
            span.dataset.id = node.id;
            span.textContent = (node.data && node.data._name) || node.type || "Node";
            span.onclick = () => selectElement(node.id, { fromNavigator: true });
            trail.appendChild(span);

            if (idx < ancestors.length - 1) {
                const sep = document.createElement("span");
                sep.className = "bc-sep";
                sep.textContent = ">";
                trail.appendChild(sep);
            }
        });
    }
    window.updateBreadcrumbs = updateBreadcrumbs;

    document.addEventListener("DOMContentLoaded", () => {
        initActionChips();
        const vp = document.getElementById("daystudio-canvas-viewport");
        if (vp) vp.addEventListener("scroll", updateSelectionChrome);
        window.addEventListener("resize", updateSelectionChrome);

        const canvas = document.getElementById("daystudio-canvas");
        if (canvas) {
            canvas.addEventListener("mouseleave", () => onNodeHover(null));
        }

        if (vp) {
            vp.addEventListener("click", (e) => {
                if (!e.target.closest("#daystudio-canvas [data-node-id]") && !e.target.closest("button, input, select, textarea, a, #daystudio-selection-chrome, .artboard-handle")) {
                    selectElement(null);
                }
            });
            vp.addEventListener("contextmenu", (e) => {
                const nodeEl = e.target.closest("#daystudio-canvas [data-node-id]");
                if (nodeEl && nodeEl.dataset.nodeId && window.openNodeContextMenu) {
                    e.preventDefault();
                    e.stopPropagation();
                    selectElement(nodeEl.dataset.nodeId);
                    window.openNodeContextMenu(e.clientX, e.clientY, nodeEl.dataset.nodeId);
                } else if (window.openNodeContextMenu) {
                    e.preventDefault();
                    e.stopPropagation();
                    selectElement(null);
                    window.openNodeContextMenu(e.clientX, e.clientY, "root");
                }
            });
        }
    });
})();
"###;
