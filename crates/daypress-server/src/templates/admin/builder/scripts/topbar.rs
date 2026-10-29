//! Top Studio Bar actions, breakpoint switcher, zoom engine, X-Ray and shortcuts (`scripts/topbar.rs`).

pub const JS_TOPBAR: &str = r###"
(function() {
    // -------------------------------------------------------------
    // RESPONSIVE BREAKPOINT SWITCHER
    // -------------------------------------------------------------
    function initBreakpointSwitcher() {
        const bpButtons = document.querySelectorAll(".bp-btn");
        const artboard = document.getElementById("daystudio-artboard");
        const dimReadout = document.getElementById("lbl-dimension-readout");

        bpButtons.forEach(btn => {
            btn.addEventListener("click", () => {
                const targetBp = btn.dataset.bp; // "desktop" | "tablet" | "mobile"
                setBreakpoint(targetBp);
            });
        });

        function setBreakpoint(bp) {
            window.DayStudioState.currentBreakpoint = bp;
            bpButtons.forEach(b => {
                if (b.dataset.bp === bp) b.classList.add("active");
                else b.classList.remove("active");
            });

            if (artboard) {
                artboard.classList.remove("artboard-desktop", "artboard-tablet", "artboard-mobile");
                artboard.classList.add("artboard-" + bp);
            }

            if (dimReadout) {
                if (bp === "tablet") dimReadout.innerHTML = `768 <span class="dim-unit">px</span> &times; 100%`;
                else if (bp === "mobile") dimReadout.innerHTML = `375 <span class="dim-unit">px</span> &times; 100%`;
                else dimReadout.innerHTML = `1200 <span class="dim-unit">px</span> &times; 100%`;
            }

            const hudDim = document.getElementById("hud-stat-dimensions");
            if (hudDim) {
                if (bp === "tablet") hudDim.textContent = "768 × 1024 px";
                else if (bp === "mobile") hudDim.textContent = "375 × 667 px";
                else hudDim.textContent = "1200 × 800 px";
            }

            // Re-resolve every node's breakpoint cascade so tablet/mobile overrides show on the artboard
            if (window.refreshAllNodeStyles) window.refreshAllNodeStyles();
            if (window.renderInspector) window.renderInspector();
            if (window.updateSelectionChrome) window.updateSelectionChrome();
        }
        window.setBreakpoint = setBreakpoint;
    }

    // -------------------------------------------------------------
    // CANVAS ZOOM ENGINE (50% - 150%)
    // -------------------------------------------------------------
    function initZoomEngine() {
        const zoomWrapper = document.getElementById("daystudio-zoom-wrapper");
        const lblZoom = document.getElementById("lbl-zoom-level");
        const hudZoom = document.getElementById("hud-stat-zoom");
        const btnIn = document.getElementById("btn-zoom-in");
        const btnOut = document.getElementById("btn-zoom-out");
        const btnReset = document.getElementById("btn-zoom-reset");
        const btnMenu = document.getElementById("btn-zoom-menu");

        function applyZoom(lvl) {
            lvl = Math.max(0.5, Math.min(1.5, Math.round(lvl * 100) / 100));
            window.DayStudioState.zoomLevel = lvl;
            const pct = Math.round(lvl * 100) + "%";
            if (zoomWrapper) zoomWrapper.style.transform = `scale(${lvl})`;
            if (lblZoom) lblZoom.textContent = pct;
            if (hudZoom) hudZoom.textContent = pct;
            if (window.updateSelectionChrome) window.updateSelectionChrome();
        }

        if (btnIn) btnIn.onclick = () => applyZoom(window.DayStudioState.zoomLevel + 0.25);
        if (btnOut) btnOut.onclick = () => applyZoom(window.DayStudioState.zoomLevel - 0.25);
        if (btnReset) btnReset.onclick = () => applyZoom(1.0);
        if (btnMenu) {
            btnMenu.onclick = () => {
                const levels = [0.5, 0.75, 1.0, 1.25];
                let idx = levels.indexOf(window.DayStudioState.zoomLevel);
                idx = (idx + 1) % levels.length;
                applyZoom(levels[idx]);
            };
        }
    }

    // -------------------------------------------------------------
    // STRUCTURAL X-RAY / WIREFRAME MODE
    // -------------------------------------------------------------
    function initXrayToggle() {
        const btnXray = document.getElementById("btn-toggle-xray");
        const artboard = document.getElementById("daystudio-artboard");
        const hudXray = document.getElementById("hud-stat-xray");

        function toggleXray() {
            window.DayStudioState.xrayMode = !window.DayStudioState.xrayMode;
            const active = window.DayStudioState.xrayMode;
            if (btnXray) btnXray.classList.toggle("active", active);
            if (artboard) artboard.classList.toggle("daystudio-xray", active);
            if (hudXray) hudXray.textContent = active ? "X-Ray: ON" : "X-Ray: OFF";
        }

        if (btnXray) btnXray.onclick = toggleXray;
        window.toggleXray = toggleXray;
    }

    // -------------------------------------------------------------
    // TOPBAR ACTIONS & HOTKEYS
    // -------------------------------------------------------------
    function initTopbarActions() {
        const btnPublish = document.getElementById("btn-primary-publish");
        if (btnPublish) {
            btnPublish.onclick = () => {
                if (window.StudioApi) window.StudioApi.saveLayout();
            };
        }

        const btnUndo = document.getElementById("btn-undo");
        if (btnUndo) btnUndo.onclick = () => window.undo();

        const btnRedo = document.getElementById("btn-redo");
        if (btnRedo) btnRedo.onclick = () => window.redo();

        if (window.updateViewLiveLink) window.updateViewLiveLink();

        // Keyboard Shortcuts
        document.addEventListener("keydown", (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === "s") {
                e.preventDefault();
                if (window.StudioApi) window.StudioApi.saveLayout();
            } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "z" || e.key === "Z")) {
                e.preventDefault();
                window.redo();
            } else if ((e.ctrlKey || e.metaKey) && e.key === "z") {
                e.preventDefault();
                window.undo();
            } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "x" || e.key === "X")) {
                e.preventDefault();
                if (window.toggleXray) window.toggleXray();
            } else if ((e.ctrlKey || e.metaKey) && (e.key === "d" || e.key === "D")) {
                if (window.DayStudioState.activeElementId) {
                    e.preventDefault();
                    if (window.duplicateNode) window.duplicateNode(window.DayStudioState.activeElementId);
                }
            } else if (e.key === "Delete" || e.key === "Backspace") {
                const targetTag = e.target ? e.target.tagName : "";
                if (targetTag !== "INPUT" && targetTag !== "TEXTAREA" && !e.target.isContentEditable) {
                    if (window.DayStudioState.activeElementId && window.DayStudioState.activeElementId !== "root") {
                        e.preventDefault();
                        if (window.deleteNode) window.deleteNode(window.DayStudioState.activeElementId);
                    }
                }
            }
        });
    }

    document.addEventListener("DOMContentLoaded", () => {
        initBreakpointSwitcher();
        initZoomEngine();
        initXrayToggle();
        initTopbarActions();
    });
})();
"###;
