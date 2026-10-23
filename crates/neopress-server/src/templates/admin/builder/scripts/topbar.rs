//! Top Studio Bar actions, breakpoint switcher, zoom engine, X-Ray and shortcuts (`scripts/topbar.rs`).

pub const JS_TOPBAR: &str = r###"
(function() {
    // -------------------------------------------------------------
    // RESPONSIVE BREAKPOINT SWITCHER
    // -------------------------------------------------------------
    function initBreakpointSwitcher() {
        const bpButtons = document.querySelectorAll(".bp-btn");
        const artboard = document.getElementById("neostudio-artboard");
        const dimReadout = document.getElementById("lbl-dimension-readout");

        bpButtons.forEach(btn => {
            btn.addEventListener("click", () => {
                const targetBp = btn.dataset.bp; // "desktop" | "tablet" | "mobile"
                setBreakpoint(targetBp);
            });
        });

        function setBreakpoint(bp) {
            window.NeoStudioState.currentBreakpoint = bp;
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

            if (window.renderInspector) window.renderInspector();
            if (window.updateSelectionChrome) window.updateSelectionChrome();
        }
        window.setBreakpoint = setBreakpoint;
    }

    // -------------------------------------------------------------
    // CANVAS ZOOM ENGINE (50% - 150%)
    // -------------------------------------------------------------
    function initZoomEngine() {
        const zoomWrapper = document.getElementById("neostudio-zoom-wrapper");
        const lblZoom = document.getElementById("lbl-zoom-level");
        const hudZoom = document.getElementById("hud-stat-zoom");
        const btnIn = document.getElementById("btn-zoom-in");
        const btnOut = document.getElementById("btn-zoom-out");
        const btnReset = document.getElementById("btn-zoom-reset");
        const btnMenu = document.getElementById("btn-zoom-menu");

        function applyZoom(lvl) {
            lvl = Math.max(0.5, Math.min(1.5, Math.round(lvl * 100) / 100));
            window.NeoStudioState.zoomLevel = lvl;
            const pct = Math.round(lvl * 100) + "%";
            if (zoomWrapper) zoomWrapper.style.transform = `scale(${lvl})`;
            if (lblZoom) lblZoom.textContent = pct;
            if (hudZoom) hudZoom.textContent = pct;
            if (window.updateSelectionChrome) window.updateSelectionChrome();
        }

        if (btnIn) btnIn.onclick = () => applyZoom(window.NeoStudioState.zoomLevel + 0.25);
        if (btnOut) btnOut.onclick = () => applyZoom(window.NeoStudioState.zoomLevel - 0.25);
        if (btnReset) btnReset.onclick = () => applyZoom(1.0);
        if (btnMenu) {
            btnMenu.onclick = () => {
                const levels = [0.5, 0.75, 1.0, 1.25];
                let idx = levels.indexOf(window.NeoStudioState.zoomLevel);
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
        const artboard = document.getElementById("neostudio-artboard");
        const hudXray = document.getElementById("hud-stat-xray");

        function toggleXray() {
            window.NeoStudioState.xrayMode = !window.NeoStudioState.xrayMode;
            const active = window.NeoStudioState.xrayMode;
            if (btnXray) btnXray.classList.toggle("active", active);
            if (artboard) artboard.classList.toggle("neostudio-xray", active);
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
                if (window.NeoStudioState.activeElementId) {
                    e.preventDefault();
                    if (window.duplicateNode) window.duplicateNode(window.NeoStudioState.activeElementId);
                }
            } else if (e.key === "Delete" || e.key === "Backspace") {
                const targetTag = e.target ? e.target.tagName : "";
                if (targetTag !== "INPUT" && targetTag !== "TEXTAREA" && !e.target.isContentEditable) {
                    if (window.NeoStudioState.activeElementId && window.NeoStudioState.activeElementId !== "root") {
                        e.preventDefault();
                        if (window.deleteNode) window.deleteNode(window.NeoStudioState.activeElementId);
                    }
                }
            }
        });
    }

    // -------------------------------------------------------------
    // PAGE & TEMPLATE CONTEXT SWITCHER
    // -------------------------------------------------------------
    let switcherCache = { pages: [], posts: [], templates: [] };

    async function switchPostContext(post) {
        const state = window.NeoStudioState;
        if (state.isDirty && window.StudioApi) {
            try { await window.StudioApi.saveLayout(); } catch(e) {}
        }
        try {
            const res = await window.StudioApi.getPostLayout(post.id);
            const titleText = (post.title && post.title.rendered) ? post.title.rendered : (post.title || "Untitled");
            state.postId = post.id;
            state.postSlug = post.slug || "";
            state.postType = post.type || "page";
            state.currentContext = {
                type: "post",
                id: post.id,
                title: titleText,
                slug: post.slug || "",
                postType: post.type || "page"
            };
            const hasSavedChildren = res && res.layout && res.layout.root && Array.isArray(res.layout.root.children) && res.layout.root.children.length > 0;
            state.pageAst = window.normalizeAst(hasSavedChildren ? res.layout : null);
            if (!hasSavedChildren && state.pageAst.root.children[0] && state.pageAst.root.children[0].children[0] && state.pageAst.root.children[0].children[0].children[0]) {
                state.pageAst.root.children[0].children[0].children[0].data.text = titleText;
            }
            state.activeElementId = null;
            if (window.history && window.history.pushState) {
                window.history.pushState({}, "", `/n-admin/builder.php?post=${post.id}`);
            }
            if (window.updateViewLiveLink) window.updateViewLiveLink();
            window.markDirty(false);
            window.pushHistorySnapshot();
            window.markDirty(false);
            window.renderCanvas();
            if (window.updateNavigator) window.updateNavigator();
            if (window.renderInspector) window.renderInspector();
            if (window.showToast) window.showToast(`Switched to "${titleText}"`);
        } catch(err) {
            if (window.showToast) window.showToast("Failed to switch page: " + err.message);
        }
    }
    window.switchPostContext = switchPostContext;

    function initTemplateSwitcher() {
        const btn = document.getElementById("btn-switch-template");
        const dropdown = document.getElementById("topbar-switcher-dropdown");
        const listEl = document.getElementById("switcher-items-container");
        const searchEl = document.getElementById("switcher-search-input");
        const btnNewPage = document.getElementById("switcher-btn-new-page");
        const btnNewTpl = document.getElementById("switcher-btn-new-template");
        if (!btn || !dropdown || !listEl) return;

        async function loadAndRenderSwitcher() {
            listEl.innerHTML = `<div style="padding:10px;font-size:11px;color:var(--neo-muted);">Loading pages & templates...</div>`;
            try {
                const [pp, templates] = await Promise.all([
                    window.StudioApi.getPagesAndPosts(),
                    window.StudioApi.getTemplates().catch(() => [])
                ]);
                switcherCache = { pages: pp.pages || [], posts: pp.posts || [], templates: templates || [] };
                renderSwitcherItems(searchEl ? searchEl.value.trim().toLowerCase() : "");
            } catch(e) {
                listEl.innerHTML = `<div style="padding:10px;font-size:11px;color:var(--neo-crimson);">Error loading list</div>`;
            }
        }

        function renderSwitcherItems(q) {
            listEl.innerHTML = "";
            const ctx = window.NeoStudioState.currentContext || {};

            function addGroup(label, items, itemKind) {
                const filtered = items.filter(it => {
                    const t = (it.title && it.title.rendered) ? it.title.rendered : (it.title || "");
                    return !q || t.toLowerCase().includes(q);
                });
                if (filtered.length === 0) return;
                const hdr = document.createElement("div");
                hdr.className = "switcher-group-label";
                hdr.textContent = label;
                listEl.appendChild(hdr);

                filtered.forEach(it => {
                    const row = document.createElement("div");
                    const isActive = (itemKind === "template" && ctx.type === "template" && ctx.id === it.id)
                        || (itemKind !== "template" && ctx.type === "post" && Number(ctx.id) === Number(it.id));
                    row.className = "switcher-item" + (isActive ? " active" : "");
                    const titleText = (it.title && it.title.rendered) ? it.title.rendered : (it.title || "Untitled");
                    const badgeText = itemKind === "template" ? (it.template_type || "TPL") : itemKind.toUpperCase();
                    row.innerHTML = `
                        <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">${titleText}</span>
                        <span class="template-type-badge">${badgeText}</span>
                    `;
                    row.onclick = async () => {
                        dropdown.classList.remove("open");
                        if (itemKind === "template") {
                            if (window.switchStudioContext) await window.switchStudioContext(it.id);
                        } else {
                            await switchPostContext(Object.assign({}, it, { type: itemKind }));
                        }
                    };
                    listEl.appendChild(row);
                });
            }

            addGroup("Pages", switcherCache.pages, "page");
            addGroup("Posts", switcherCache.posts, "post");
            addGroup("Theme Templates", switcherCache.templates, "template");

            if (!listEl.children.length) {
                listEl.innerHTML = `<div style="padding:10px;font-size:11px;color:var(--neo-muted);text-align:center;">No matches found</div>`;
            }
        }

        btn.onclick = (e) => {
            e.stopPropagation();
            const isOpen = dropdown.classList.toggle("open");
            if (isOpen) {
                loadAndRenderSwitcher();
                if (searchEl) { searchEl.value = ""; setTimeout(() => searchEl.focus(), 30); }
            }
        };

        if (searchEl) {
            searchEl.onclick = (e) => e.stopPropagation();
            searchEl.oninput = () => renderSwitcherItems(searchEl.value.trim().toLowerCase());
        }

        if (btnNewPage) {
            btnNewPage.onclick = async (e) => {
                e.stopPropagation();
                const name = (searchEl && searchEl.value.trim()) ? searchEl.value.trim() : "New Studio Page";
                try {
                    const created = await window.StudioApi.createQuickPage(name);
                    dropdown.classList.remove("open");
                    await switchPostContext(Object.assign({}, created, { type: "page" }));
                } catch(err) {
                    if (window.showToast) window.showToast("Failed to create page: " + err.message);
                }
            };
        }

        if (btnNewTpl) {
            btnNewTpl.onclick = (e) => {
                e.stopPropagation();
                dropdown.classList.remove("open");
                if (window.openCreateTemplateModal) window.openCreateTemplateModal();
            };
        }

        document.addEventListener("click", (e) => {
            if (!dropdown.contains(e.target) && e.target !== btn) {
                dropdown.classList.remove("open");
            }
        });
    }

    document.addEventListener("DOMContentLoaded", () => {
        initBreakpointSwitcher();
        initZoomEngine();
        initXrayToggle();
        initTopbarActions();
        initTemplateSwitcher();
    });
})();
"###;
