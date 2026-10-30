//! Viewport Drag Resize Handles and Canvas Drag-and-Drop Interaction (`scripts/canvas_drag.rs`).

pub const JS_CANVAS_DRAG: &str = r###"
(function() {
    function initViewportHandles() {
        const artboard = document.getElementById("daystudio-artboard");
        const zoomWrapper = document.getElementById("daystudio-zoom-wrapper");
        const handleLeft = document.getElementById("handle-artboard-left");
        const handleRight = document.getElementById("handle-artboard-right");
        const dimReadout = document.getElementById("lbl-dimension-readout");
        const bpButtons = document.querySelectorAll(".bp-btn");

        if (!artboard || !handleLeft || !handleRight) return;

        function attachHandleDrag(handle, direction) {
            handle.addEventListener("mousedown", (e) => {
                e.preventDefault();
                e.stopPropagation();

                const startX = e.clientX;
                const startWidth = artboard.offsetWidth;
                handle.classList.add("is-dragging");
                artboard.style.transition = "none";
                document.body.style.cursor = "ew-resize";
                document.body.style.userSelect = "none";

                function onMouseMove(me) {
                    const deltaX = me.clientX - startX;
                    // Symmetrical resizing from center
                    const newWidth = Math.round(
                        direction === "right"
                            ? Math.max(320, Math.min(1920, startWidth + deltaX * 2))
                            : Math.max(320, Math.min(1920, startWidth - deltaX * 2))
                    );

                    artboard.style.width = newWidth + "px";
                    artboard.classList.add("is-custom-width");
                    if (zoomWrapper) {
                        zoomWrapper.style.width = newWidth + "px";
                        zoomWrapper.classList.add("is-custom-width");
                    }
                    if (dimReadout) {
                        dimReadout.innerHTML = `${newWidth} <span class="dim-unit">px</span> &times; 100%`;
                    }

                    const nextBp = newWidth <= 480 ? "mobile" : (newWidth <= 1024 ? "tablet" : "desktop");
                    if (window.DayStudioState && window.DayStudioState.currentBreakpoint !== nextBp) {
                        window.DayStudioState.currentBreakpoint = nextBp;
                        artboard.classList.remove("artboard-desktop", "artboard-tablet", "artboard-mobile");
                        artboard.classList.add("artboard-" + nextBp);
                        bpButtons.forEach(b => b.classList.toggle("active", b.dataset.bp === nextBp));
                        if (window.refreshAllNodeStyles) window.refreshAllNodeStyles();
                        if (window.renderInspector) window.renderInspector();
                    }
                    if (window.updateSelectionChrome) window.updateSelectionChrome();
                }

                function onMouseUp(ue) {
                    handle.classList.remove("is-dragging");
                    artboard.style.transition = "";
                    document.body.style.cursor = "";
                    document.body.style.userSelect = "";
                    window.removeEventListener("mousemove", onMouseMove);
                    window.removeEventListener("mouseup", onMouseUp);

                    // Snap to canonical breakpoints when dropped near them
                    const finalWidth = artboard.offsetWidth;
                    if (Math.abs(finalWidth - 1200) < 24) {
                        if (window.setBreakpoint) window.setBreakpoint("desktop");
                    } else if (Math.abs(finalWidth - 768) < 20) {
                        if (window.setBreakpoint) window.setBreakpoint("tablet");
                    } else if (Math.abs(finalWidth - 375) < 16) {
                        if (window.setBreakpoint) window.setBreakpoint("mobile");
                    }
                }

                window.addEventListener("mousemove", onMouseMove);
                window.addEventListener("mouseup", onMouseUp);
            });
        }

        attachHandleDrag(handleLeft, "left");
        attachHandleDrag(handleRight, "right");
    }

    // Canvas Drop Indicator
    function showDropIndicator(targetEl, position) {
        const guide = document.getElementById("daystudio-drop-guide");
        if (!guide || !targetEl) return;
        const rect = targetEl.getBoundingClientRect();
        guide.style.display = "block";
        guide.style.left = rect.left + "px";
        guide.style.width = rect.width + "px";

        if (position === "before") {
            guide.style.top = rect.top + "px";
        } else if (position === "after") {
            guide.style.top = rect.bottom + "px";
        } else {
            // inside bottom
            guide.style.top = (rect.bottom - 4) + "px";
        }
    }
    window.showDropIndicator = showDropIndicator;

    function hideDropIndicator() {
        const guide = document.getElementById("daystudio-drop-guide");
        if (guide) guide.style.display = "none";
    }
    window.hideDropIndicator = hideDropIndicator;

    // -------------------------------------------------------------
    // CANVAS-WIDE DRAG-AND-DROP & BOTTOM SECTION PRESETS
    // -------------------------------------------------------------
    function initCanvasDropzonesAndPresets() {
        const canvas = document.getElementById("daystudio-canvas");
        const bottomDrop = document.getElementById("daystudio-bottom-dropzone");
        const actDragBtn = document.getElementById("act-drag-node");
        const btnOpenPresets = document.getElementById("btn-open-presets");
        const presetSelector = document.getElementById("preset-selector");
        const btnOpenStarters = document.getElementById("btn-open-starters-prompt");

        // Selection badge grip dragstart
        if (actDragBtn) {
            actDragBtn.addEventListener("dragstart", (e) => {
                const curId = window.DayStudioState.activeElementId;
                if (!curId || curId === "root") {
                    e.preventDefault();
                    return;
                }
                const node = window.findNodeById(curId);
                if (node && node.data && node.data._locked) {
                    e.preventDefault();
                    if (window.showToast) window.showToast("Cannot move locked element", "warning");
                    return;
                }
                window._draggedNavNodeId = curId;
                window._draggedPaletteType = null;
                e.dataTransfer.setData("text/plain", curId);
                e.dataTransfer.setData("application/x-daystudio-node", curId);
                e.dataTransfer.effectAllowed = "move";
            });
            actDragBtn.addEventListener("dragend", () => {
                window._draggedNavNodeId = null;
                hideDropIndicator();
            });
        }

        function handleRootDrop(e) {
            e.preventDefault();
            e.stopPropagation();
            hideDropIndicator();
            if (bottomDrop) bottomDrop.style.borderColor = "";

            const rawText = e.dataTransfer ? e.dataTransfer.getData("text/plain") : "";
            const paletteType = window._draggedPaletteType
                || (e.dataTransfer && e.dataTransfer.getData("application/x-daystudio-palette"))
                || (rawText.startsWith("palette:") ? rawText.slice(8) : null);

            if (paletteType) {
                window._draggedPaletteType = null;
                window.DayStudioState.activeElementId = null;
                if (window.insertWidgetFromPalette) window.insertWidgetFromPalette(paletteType);
                return;
            }

            const draggedNodeId = window._draggedNavNodeId || rawText;
            if (draggedNodeId && window.moveNode) {
                window._draggedNavNodeId = null;
                window.moveNode(draggedNodeId, "root", "inside");
            }
        }

        [canvas, bottomDrop].forEach(zone => {
            if (!zone) return;
            zone.addEventListener("dragover", (e) => {
                e.preventDefault();
                e.dataTransfer.dropEffect = window._draggedPaletteType ? "copy" : "move";
                if (zone === bottomDrop) bottomDrop.style.borderColor = "var(--day-signal)";
            });
            zone.addEventListener("dragleave", () => {
                if (zone === bottomDrop) bottomDrop.style.borderColor = "";
            });
            zone.addEventListener("drop", handleRootDrop);
        });

        // Bottom "+ Add Section" Preset Selector
        if (btnOpenPresets && presetSelector) {
            btnOpenPresets.onclick = () => {
                presetSelector.classList.toggle("open");
            };
        }

        if (btnOpenStarters) {
            btnOpenStarters.onclick = () => {
                if (window.openCreateTemplateModal) window.openCreateTemplateModal(null, true);
            };
        }

        document.querySelectorAll(".preset-card[data-cols]").forEach(card => {
            card.onclick = () => {
                const spec = card.dataset.cols;
                insertSectionPreset(spec);
                if (presetSelector) presetSelector.classList.remove("open");
            };
        });
    }

    function insertSectionPreset(spec) {
        const state = window.DayStudioState;
        let ratios = [1];
        if (spec === "2") ratios = [1, 1];
        else if (spec === "3") ratios = [1, 1, 1];
        else if (spec === "4") ratios = [1, 1, 1, 1];
        else if (spec === "33-66") ratios = [1, 2];
        else if (spec === "66-33") ratios = [2, 1];

        const cols = ratios.map((r, i) => ({
            id: window.generateId("col"),
            type: "Column",
            styles: {
                display: "flex",
                flex_direction: "column",
                gap: "12px",
                padding_top: "16px",
                padding_right: "16px",
                padding_bottom: "16px",
                padding_left: "16px",
                width: Math.round((r / ratios.reduce((a, b) => a + b, 0)) * 100) + "%"
            },
            children: [
                {
                    id: window.generateId("h"),
                    type: "Heading",
                    data: { text: `Column ${i + 1}`, level: 3 },
                    styles: { font_size: "20px", text_color: "#0f172a" }
                }
            ]
        }));

        const newSec = {
            id: window.generateId("sec"),
            type: "Section",
            styles: { padding_top: "48px", padding_bottom: "48px", background_color: "#ffffff" },
            children: [
                {
                    id: window.generateId("cont"),
                    type: "Container",
                    styles: { display: "flex", flex_direction: "row", gap: "20px", width: "100%" },
                    children: cols
                }
            ]
        };

        state.pageAst.root.children = state.pageAst.root.children || [];
        state.pageAst.root.children.push(newSec);
        window.pushHistorySnapshot();
        window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        if (window.selectElement) window.selectElement(newSec.id);
        if (window.showToast) window.showToast(`Added ${ratios.length}-Column Section`);
    }
    window.insertSectionPreset = insertSectionPreset;

    document.addEventListener("DOMContentLoaded", () => {
        initViewportHandles();
        initCanvasDropzonesAndPresets();
    });
})();
"###;
