//! Live Box-Model Margin (Amber) and Padding (Cobalt) Guide Overlays (`scripts/canvas_overlays.rs`).

pub const JS_CANVAS_OVERLAYS: &str = r###"
(function() {
    function updateBoxOverlays(nodeId) {
        const overlays = document.getElementById("daystudio-box-overlays");
        if (!overlays) return;

        if (!nodeId) {
            overlays.style.display = "none";
            return;
        }

        const el = document.querySelector(`[data-node-id="${nodeId}"]`);
        if (!el) {
            overlays.style.display = "none";
            return;
        }

        const rect = el.getBoundingClientRect();
        const viewport = document.getElementById("daystudio-canvas-viewport");
        const vpRect = viewport.getBoundingClientRect();
        const computed = window.getComputedStyle(el);

        const mTop = parseFloat(computed.marginTop) || 0;
        const mRight = parseFloat(computed.marginRight) || 0;
        const mBottom = parseFloat(computed.marginBottom) || 0;
        const mLeft = parseFloat(computed.marginLeft) || 0;

        const pTop = parseFloat(computed.paddingTop) || 0;
        const pRight = parseFloat(computed.paddingRight) || 0;
        const pBottom = parseFloat(computed.paddingBottom) || 0;
        const pLeft = parseFloat(computed.paddingLeft) || 0;

        const baseTop = rect.top - vpRect.top + viewport.scrollTop;
        const baseLeft = rect.left - vpRect.left + viewport.scrollLeft;

        overlays.style.display = "block";
        overlays.style.top = (baseTop - mTop) + "px";
        overlays.style.left = (baseLeft - mLeft) + "px";
        overlays.style.width = (rect.width + mLeft + mRight) + "px";
        overlays.style.height = (rect.height + mTop + mBottom) + "px";

        // Margin Overlays (Outer Amber Guides)
        setOverlayFacet("overlay-margin-top", 0, mLeft, mTop, rect.width, mTop);
        setOverlayFacet("overlay-margin-bottom", mTop + rect.height, mLeft, mBottom, rect.width, mBottom);
        setOverlayFacet("overlay-margin-left", 0, 0, rect.height + mTop + mBottom, mLeft, mLeft);
        setOverlayFacet("overlay-margin-right", 0, mLeft + rect.width, rect.height + mTop + mBottom, mRight, mRight);

        // Padding Overlays (Inner Cobalt Guides)
        setOverlayFacet("overlay-padding-top", mTop, mLeft, pTop, rect.width, pTop);
        setOverlayFacet("overlay-padding-bottom", mTop + rect.height - pBottom, mLeft, pBottom, rect.width, pBottom);
        setOverlayFacet("overlay-padding-left", mTop + pTop, mLeft, rect.height - pTop - pBottom, pLeft, pLeft);
        setOverlayFacet("overlay-padding-right", mTop + pTop, mLeft + rect.width - pRight, rect.height - pTop - pBottom, pRight, pRight);
    }
    window.updateBoxOverlays = updateBoxOverlays;

    function setOverlayFacet(id, top, left, height, width, val) {
        const facet = document.getElementById(id);
        if (!facet) return;
        facet.style.top = top + "px";
        facet.style.left = left + "px";
        facet.style.height = Math.max(0, height) + "px";
        facet.style.width = Math.max(0, width) + "px";

        const chip = facet.querySelector(".overlay-chip");
        if (chip) {
            chip.textContent = Math.round(val);
            chip.style.display = val > 0 ? "inline-block" : "none";
        }
    }

    function showBoxOverlays() {
        const id = window.DayStudioState.activeElementId;
        if (id) updateBoxOverlays(id);
    }
    window.showBoxOverlays = showBoxOverlays;

    function hideBoxOverlays() {
        const overlays = document.getElementById("daystudio-box-overlays");
        if (overlays) overlays.style.display = "none";
    }
    window.hideBoxOverlays = hideBoxOverlays;
})();
"###;
