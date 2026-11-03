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
                artboard.style.width = "";
                artboard.classList.remove("artboard-desktop", "artboard-tablet", "artboard-mobile", "is-custom-width");
                const zoomWrapper = document.getElementById("daystudio-zoom-wrapper");
                if (zoomWrapper) { zoomWrapper.style.width = ""; zoomWrapper.classList.remove("is-custom-width"); }
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

            // Re-render dynamic widgets (e.g., mobile hamburger menu) and re-resolve every node's breakpoint cascade
            if (window.renderCanvas) window.renderCanvas();
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
    // ARTBOARD LIGHT / DARK THEME PREVIEW TOGGLE
    // -------------------------------------------------------------
    function initCanvasThemeToggle() {
        const btnTheme = document.getElementById("btn-toggle-canvas-theme");
        const iconTheme = document.getElementById("icon-canvas-theme");
        const lblTheme = document.getElementById("lbl-canvas-theme");
        const artboard = document.getElementById("daystudio-artboard");
        const canvas = document.getElementById("daystudio-canvas");

        function setCanvasTheme(mode) {
            const next = mode === "dark" ? "dark" : "light";
            window.DayStudioState.canvasTheme = next;
            if (artboard) {
                artboard.setAttribute("data-theme", next);
                artboard.classList.toggle("dark", next === "dark");
                artboard.classList.toggle("light", next === "light");
            }
            if (canvas) {
                canvas.setAttribute("data-theme", next);
                canvas.classList.toggle("dark", next === "dark");
                canvas.classList.toggle("light", next === "light");
            }
            document.documentElement.setAttribute("data-theme", next);
            if (btnTheme) btnTheme.classList.toggle("active", next === "dark");
            if (iconTheme) iconTheme.className = next === "dark" ? "fa-solid fa-moon" : "fa-solid fa-sun";
            if (lblTheme) lblTheme.textContent = next === "dark" ? "Dark" : "Light";
            document.querySelectorAll("#daystudio-artboard .tt button[data-t]").forEach(b => {
                b.setAttribute("aria-pressed", b.dataset.t === next ? "true" : "false");
            });
            if (window.renderCanvas) window.renderCanvas();
        }

        // Default artboard preview to Light mode on initial load so site background is always visible
        setCanvasTheme("light");

        if (btnTheme) {
            btnTheme.onclick = () => {
                const cur = window.DayStudioState.canvasTheme || "light";
                setCanvasTheme(cur === "dark" ? "light" : "dark");
            };
        }
        window.setCanvasTheme = setCanvasTheme;
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
            } else if ((e.ctrlKey || e.metaKey) && (e.key === "c" || e.key === "C")) {
                const targetTag = e.target ? e.target.tagName : "";
                if (targetTag !== "INPUT" && targetTag !== "TEXTAREA" && !e.target.isContentEditable && window.DayStudioState.activeElementId) {
                    e.preventDefault();
                    if (window.copyNode) window.copyNode(window.DayStudioState.activeElementId);
                }
            } else if ((e.ctrlKey || e.metaKey) && (e.key === "x" || e.key === "X") && !e.shiftKey) {
                const targetTag = e.target ? e.target.tagName : "";
                if (targetTag !== "INPUT" && targetTag !== "TEXTAREA" && !e.target.isContentEditable && window.DayStudioState.activeElementId) {
                    e.preventDefault();
                    if (window.cutNode) window.cutNode(window.DayStudioState.activeElementId);
                }
            } else if ((e.ctrlKey || e.metaKey) && (e.key === "v" || e.key === "V")) {
                const targetTag = e.target ? e.target.tagName : "";
                if (targetTag !== "INPUT" && targetTag !== "TEXTAREA" && !e.target.isContentEditable && (window._dayStudioClipboard || window._daystudioClipboard)) {
                    e.preventDefault();
                    if (window.pasteNode) window.pasteNode(window.DayStudioState.activeElementId || "root");
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
        initCanvasThemeToggle();
        initTopbarActions();
    });
})();
"###;
