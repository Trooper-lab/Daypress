//! Selection Chrome, Bounding Box, Action Chips and Breadcrumb Navigation (`scripts/canvas_selection.rs`).

pub const JS_CANVAS_SELECTION: &str = r###"
(function() {
    function selectElement(id) {
        const state = window.NeoStudioState;
        state.activeElementId = id;

        updateSelectionChrome();
        updateBreadcrumbs(id);

        if (window.renderInspector) window.renderInspector();
        if (window.syncNavigatorSelection) window.syncNavigatorSelection(id);
        if (window.updateBoxOverlays) window.updateBoxOverlays(id);
    }
    window.selectElement = selectElement;

    function updateSelectionChrome() {
        const chrome = document.getElementById("neostudio-selection-chrome");
        const id = window.NeoStudioState.activeElementId;
        if (!chrome || !id) {
            if (chrome) chrome.style.display = "none";
            return;
        }

        const targetEl = document.querySelector(`[data-node-id="${id}"]`);
        if (!targetEl) {
            chrome.style.display = "none";
            return;
        }

        const rect = targetEl.getBoundingClientRect();
        const viewport = document.getElementById("neostudio-canvas-viewport");
        const vpRect = viewport.getBoundingClientRect();

        // Position relative to viewport scroll area
        const top = rect.top - vpRect.top + viewport.scrollTop;
        const left = rect.left - vpRect.left + viewport.scrollLeft;

        chrome.style.display = "block";
        chrome.style.top = top + "px";
        chrome.style.left = left + "px";
        chrome.style.width = rect.width + "px";
        chrome.style.height = rect.height + "px";

        // Update badge labels
        const node = window.findNodeById(id);
        if (node) {
            const lblKind = document.getElementById("sel-badge-kind");
            const lblId = document.getElementById("sel-badge-id");
            if (lblKind) lblKind.textContent = (node.data && node.data._name) || node.type || "Element";
            if (lblId) lblId.textContent = "#" + node.id;
        }
    }
    window.updateSelectionChrome = updateSelectionChrome;

    // Attach Action Chips
    function initActionChips() {
        const actParent = document.getElementById("act-parent-node");
        const actDup = document.getElementById("act-dup-node");
        const actDel = document.getElementById("act-del-node");

        if (actParent) {
            actParent.onclick = (e) => {
                e.stopPropagation();
                const curId = window.NeoStudioState.activeElementId;
                if (!curId) return;
                const parent = window.findParentNode(curId);
                if (parent && parent.id) selectElement(parent.id);
            };
        }

        if (actDup) {
            actDup.onclick = (e) => {
                e.stopPropagation();
                const curId = window.NeoStudioState.activeElementId;
                if (curId) duplicateNode(curId);
            };
        }

        if (actDel) {
            actDel.onclick = (e) => {
                e.stopPropagation();
                const curId = window.NeoStudioState.activeElementId;
                if (curId) deleteNode(curId);
            };
        }
    }

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
        window.pushHistorySnapshot();
        window.renderCanvas();
        selectElement(clone.id);
    }
    window.duplicateNode = duplicateNode;

    function reassignCloneIds(node) {
        node.id = window.generateId(node.type ? node.type.toLowerCase() : "node");
        if (node.children) {
            for (const child of node.children) reassignCloneIds(child);
        }
    }

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
        window.NeoStudioState.activeElementId = parent.id;
        window.pushHistorySnapshot();
        window.renderCanvas();
        selectElement(parent.id);
    }
    window.deleteNode = deleteNode;

    // -------------------------------------------------------------
    // BOTTOM BREADCRUMB TRAIL
    // -------------------------------------------------------------
    function updateBreadcrumbs(id) {
        const trail = document.getElementById("neostudio-breadcrumbs");
        if (!trail) return;
        trail.innerHTML = "";

        if (!id) {
            trail.innerHTML = `<span class="bc-item" data-id="root">Body</span>`;
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
            span.onclick = () => selectElement(node.id);
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
        const vp = document.getElementById("neostudio-canvas-viewport");
        if (vp) vp.addEventListener("scroll", updateSelectionChrome);
        window.addEventListener("resize", updateSelectionChrome);
    });
})();
"###;
