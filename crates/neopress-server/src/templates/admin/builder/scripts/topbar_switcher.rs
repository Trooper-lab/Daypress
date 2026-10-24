//! Topbar Page / Post / Theme-Template context switcher dropdown with live search
//! (`scripts/topbar_switcher.rs`).

pub const JS_TOPBAR_SWITCHER: &str = r###"
(function() {
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
        const topbar = document.getElementById("neostudio-topbar");
        if (!btn || !dropdown || !listEl) return;

        // The topbar is a fixed 40px `overflow: hidden` strip. A dropdown nested inside it gets clipped,
        // and focusing its search field scrolls the whole topbar out of view. Portal the dropdown to
        // <body> and position it against the button instead.
        document.body.appendChild(dropdown);
        if (topbar) topbar.addEventListener("scroll", () => { topbar.scrollTop = 0; topbar.scrollLeft = 0; });

        let loaded = false;
        let loadToken = 0;
        let searchTimer = 0;
        let kbdIndex = -1;

        const decoder = document.createElement("textarea");
        function plainTitle(it) {
            const raw = (it && it.title && typeof it.title === "object") ? (it.title.rendered || it.title.raw || "") : ((it && it.title) || "");
            decoder.innerHTML = String(raw);
            return decoder.value.trim() || "Untitled";
        }

        function positionDropdown() {
            const r = btn.getBoundingClientRect();
            const width = Math.min(360, window.innerWidth - 16);
            const left = Math.max(8, Math.min(r.left, window.innerWidth - width - 8));
            dropdown.style.position = "fixed";
            const barBottom = topbar ? topbar.getBoundingClientRect().bottom : r.bottom;
            const top = Math.max(r.bottom, barBottom) + 4;
            dropdown.style.top = top + "px";
            dropdown.style.left = left + "px";
            dropdown.style.width = width + "px";
            dropdown.style.maxHeight = Math.max(200, window.innerHeight - top - 16) + "px";
        }

        function isOpen() { return dropdown.classList.contains("open"); }

        function openDropdown() {
            positionDropdown();
            dropdown.classList.add("open");
            btn.setAttribute("aria-expanded", "true");
            if (searchEl) {
                searchEl.value = "";
                setTimeout(() => searchEl.focus({ preventScroll: true }), 20);
            }
            kbdIndex = -1;
            if (loaded) renderSwitcherItems("");
            loadAndRenderSwitcher("");
        }

        function closeDropdown(refocus) {
            dropdown.classList.remove("open");
            btn.setAttribute("aria-expanded", "false");
            if (refocus) btn.focus({ preventScroll: true });
        }

        function mergeById(base, extra) {
            const seen = new Set(base.map(i => i.id));
            return base.concat(extra.filter(i => !seen.has(i.id)));
        }

        async function loadAndRenderSwitcher(search) {
            const token = ++loadToken;
            if (!loaded) listEl.innerHTML = `<div class="switcher-empty">Loading pages &amp; templates…</div>`;
            try {
                const [pp, templates] = await Promise.all([
                    window.StudioApi.getPagesAndPosts(search),
                    search ? Promise.resolve(switcherCache.templates) : window.StudioApi.getTemplates().catch(() => [])
                ]);
                if (token !== loadToken) return; // a newer request superseded this one
                if (search) {
                    switcherCache.pages = mergeById(switcherCache.pages, pp.pages || []);
                    switcherCache.posts = mergeById(switcherCache.posts, pp.posts || []);
                } else {
                    switcherCache = { pages: pp.pages || [], posts: pp.posts || [], templates: Array.isArray(templates) ? templates : [] };
                }
                loaded = true;
                if (isOpen()) renderSwitcherItems(searchEl ? searchEl.value.trim().toLowerCase() : "");
            } catch (e) {
                if (token === loadToken && !loaded) listEl.innerHTML = `<div class="switcher-empty is-error">Could not load pages: ${e.message || e}</div>`;
            }
        }

        function rows() { return Array.from(listEl.querySelectorAll(".switcher-item")); }

        function setKbd(i) {
            const all = rows();
            if (!all.length) { kbdIndex = -1; return; }
            kbdIndex = (i + all.length) % all.length;
            all.forEach((r, idx) => r.classList.toggle("kbd-active", idx === kbdIndex));
            all[kbdIndex].scrollIntoView({ block: "nearest" });
        }

        async function activate(it, itemKind) {
            closeDropdown(false);
            const ctx = window.NeoStudioState.currentContext || {};
            const same = itemKind === "template"
                ? (ctx.type === "template" && String(ctx.id) === String(it.id))
                : (ctx.type === "post" && Number(ctx.id) === Number(it.id));
            if (same) return;
            if (itemKind === "template") {
                if (window.switchStudioContext) await window.switchStudioContext(it.id);
            } else {
                await switchPostContext(Object.assign({}, it, { type: itemKind, title: { rendered: plainTitle(it) } }));
            }
        }

        function renderSwitcherItems(q) {
            listEl.innerHTML = "";
            kbdIndex = -1;
            const ctx = window.NeoStudioState.currentContext || {};

            function addGroup(label, items, itemKind) {
                const filtered = items.filter(it => {
                    if (!q) return true;
                    const hay = (plainTitle(it) + " " + (it.slug || "") + " " + (it.template_type || "")).toLowerCase();
                    return hay.includes(q);
                });
                if (!filtered.length) return;
                const hdr = document.createElement("div");
                hdr.className = "switcher-group-label";
                hdr.textContent = `${label} (${filtered.length})`;
                listEl.appendChild(hdr);

                filtered.forEach(it => {
                    const isActive = (itemKind === "template" && ctx.type === "template" && String(ctx.id) === String(it.id))
                        || (itemKind !== "template" && ctx.type === "post" && Number(ctx.id) === Number(it.id));
                    const row = document.createElement("div");
                    row.className = "switcher-item" + (isActive ? " active" : "");
                    row.setAttribute("role", "option");
                    const title = document.createElement("span");
                    title.className = "switcher-item-title";
                    title.textContent = plainTitle(it);
                    title.title = plainTitle(it);
                    const badge = document.createElement("span");
                    badge.className = "template-type-badge";
                    badge.textContent = itemKind === "template" ? (it.template_type || "TPL") : (it.status && it.status !== "publish" ? it.status : itemKind);
                    row.append(title, badge);
                    row.addEventListener("click", (e) => { e.stopPropagation(); activate(it, itemKind); });
                    row.addEventListener("mousemove", () => setKbd(rows().indexOf(row)));
                    row._switcherItem = [it, itemKind];
                    listEl.appendChild(row);
                });
            }

            addGroup("Pages", switcherCache.pages, "page");
            addGroup("Posts", switcherCache.posts, "post");
            addGroup("Theme Templates", switcherCache.templates, "template");

            if (!listEl.children.length) {
                const empty = document.createElement("div");
                empty.className = "switcher-empty";
                empty.textContent = q ? `No matches for "${q}" — use "New Page" to create it.` : "No pages yet.";
                listEl.appendChild(empty);
            }
        }

        btn.setAttribute("aria-haspopup", "listbox");
        btn.setAttribute("aria-expanded", "false");
        btn.onclick = (e) => {
            e.stopPropagation();
            if (isOpen()) closeDropdown(false); else openDropdown();
        };

        // Clicks inside the dropdown must never bubble to the document "outside click" closer
        dropdown.addEventListener("click", (e) => e.stopPropagation());

        if (searchEl) {
            searchEl.addEventListener("input", () => {
                const q = searchEl.value.trim().toLowerCase();
                renderSwitcherItems(q);
                // Only the first 100 items are preloaded: ask the server for the rest
                clearTimeout(searchTimer);
                if (q.length >= 2) searchTimer = setTimeout(() => loadAndRenderSwitcher(q), 220);
            });
            searchEl.addEventListener("keydown", (e) => {
                if (e.key === "ArrowDown") { e.preventDefault(); setKbd(kbdIndex + 1); }
                else if (e.key === "ArrowUp") { e.preventDefault(); setKbd(kbdIndex - 1); }
                else if (e.key === "Enter") {
                    e.preventDefault();
                    const all = rows();
                    const row = all[kbdIndex >= 0 ? kbdIndex : 0];
                    if (row && row._switcherItem) activate(row._switcherItem[0], row._switcherItem[1]);
                } else if (e.key === "Escape") {
                    e.preventDefault();
                    closeDropdown(true);
                }
            });
        }

        if (btnNewPage) {
            btnNewPage.onclick = async (e) => {
                e.stopPropagation();
                const name = (searchEl && searchEl.value.trim()) ? searchEl.value.trim() : "New Studio Page";
                try {
                    const created = await window.StudioApi.createQuickPage(name);
                    closeDropdown(false);
                    loaded = false;
                    await switchPostContext(Object.assign({}, created, { type: "page" }));
                } catch(err) {
                    if (window.showToast) window.showToast("Failed to create page: " + err.message);
                }
            };
        }

        if (btnNewTpl) {
            btnNewTpl.onclick = (e) => {
                e.stopPropagation();
                closeDropdown(false);
                if (window.openCreateTemplateModal) window.openCreateTemplateModal();
            };
        }

        document.addEventListener("click", (e) => {
            if (isOpen() && !dropdown.contains(e.target) && !btn.contains(e.target)) closeDropdown(false);
        });
        document.addEventListener("keydown", (e) => {
            if (e.key === "Escape" && isOpen()) closeDropdown(true);
        });
        window.addEventListener("resize", () => { if (isOpen()) positionDropdown(); });
    }

    document.addEventListener("DOMContentLoaded", initTemplateSwitcher);
})();
"###;
