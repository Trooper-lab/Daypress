//! DayStudio 0.5 Synced Symbols Library Drawer & Inspector Extensions (`scripts/canvas_pro_symbols_drawer.rs`).

pub const JS_CANVAS_PRO_SYMBOLS_DRAWER: &str = r###"
(function() {
    function renderSymbolsLibraryIntoDrawer(container) {
        if (!container) return;
        const scrollBody = container.querySelector("#elem-catalog-body")
            || container.querySelector(".drawer-scroll-body")
            || container;
        let symSec = scrollBody.querySelector("#dp-symbols-library-section");
        if (!symSec) {
            symSec = document.createElement("div");
            symSec.id = "dp-symbols-library-section";
            symSec.className = "cat-accordion";
            scrollBody.appendChild(symSec);
        }
        window.loadSymbolTemplatesCache(false).then(list => {
            const rows = list.length === 0
                ? `<div style="padding:10px;font-size:11px;color:var(--day-muted);background:var(--day-surface-inset);border:1px dashed var(--day-rule);border-radius:var(--day-radius-sm);text-align:center;">No saved Global Symbols yet. Select any Section or Container on the canvas and click <strong>Convert to Global Symbol</strong>.</div>`
                : list.map(sym => `
                    <div class="repeater-item" style="flex-direction:row;align-items:center;justify-content:space-between;gap:8px;padding:7px 9px;">
                        <div style="min-width:0;flex:1;">
                            <div style="font-size:11.5px;font-weight:600;color:var(--day-ink);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">
                                <i class="fa-solid fa-clone" style="color:var(--day-violet);margin-right:5px;font-size:10px;"></i>${sym.title}
                            </div>
                            <div style="font-size:9.5px;color:var(--day-muted);font-family:var(--day-font-mono);">ID #${sym.id} &bull; Synced Symbol</div>
                        </div>
                        <button type="button" class="cad-btn sm secondary" data-insert-symbol-id="${sym.id}">
                            <i class="fa-solid fa-plus"></i> Insert
                        </button>
                    </div>
                `).join("");

            symSec.innerHTML = `
                <div class="cat-header">
                    <span><i class="fa-solid fa-clone" style="color:var(--day-violet);margin-right:6px;"></i>Synced Global Symbols (${list.length})</span>
                    <button type="button" class="drawer-action-btn" id="btn-refresh-symbols-lib" title="Refresh Symbols" style="width:20px;height:20px;"><i class="fa-solid fa-arrows-rotate" style="font-size:10px;"></i></button>
                </div>
                <div class="cat-body" style="display:flex;flex-direction:column;gap:6px;padding-top:4px;">${rows}</div>
            `;
            const refBtn = symSec.querySelector("#btn-refresh-symbols-lib");
            if (refBtn) {
                refBtn.onclick = (e) => {
                    e.stopPropagation();
                    window.loadSymbolTemplatesCache(true).then(() => renderSymbolsLibraryIntoDrawer(container));
                };
            }
            symSec.querySelectorAll("[data-insert-symbol-id]").forEach(btn => {
                btn.onclick = () => {
                    const sid = parseInt(btn.getAttribute("data-insert-symbol-id"), 10) || 0;
                    if (!sid) return;
                    const sym = (window._dpSymbolTemplatesCache || []).find(s => Number(s.id) === sid);
                    const state = window.DayStudioState;
                    if (!state || !state.pageAst || !state.pageAst.root) return;
                    const newNode = {
                        id: window.generateId ? window.generateId("symbol") : ("symbol_" + Date.now()),
                        type: "SymbolInstance",
                        data: { symbol_id: sid, label: sym ? sym.title : ("Symbol #" + sid), overrides: {} },
                        styles: {},
                        children: (sym && sym.layout && sym.layout.root && Array.isArray(sym.layout.root.children))
                            ? JSON.parse(JSON.stringify(sym.layout.root.children))
                            : []
                    };
                    const currentId = state.activeElementId;
                    const current = currentId && window.findNodeById ? window.findNodeById(currentId) : null;
                    if (current && ["Section", "Container", "Column", "Grid", "Flex", "MegaMenuContainer"].includes(current.type)) {
                        current.children = current.children || [];
                        current.children.push(newNode);
                    } else {
                        state.pageAst.root.children = state.pageAst.root.children || [];
                        state.pageAst.root.children.push(newNode);
                    }
                    if (window.markDirty) window.markDirty(true);
                    if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                    if (window.renderCanvas) window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                    if (window.selectElement) window.selectElement(newNode.id);
                    if (window.showToast) window.showToast(`Inserted Symbol "${sym ? sym.title : sid}"`);
                };
            });
        });
    }
    window.renderSymbolsLibraryIntoDrawer = renderSymbolsLibraryIntoDrawer;

    const DYNAMIC_TAG_META = {
        post_title:     { token: "{{post.title}}",          label: "Post / Page Title",      sample: "Building Sub-Millisecond Websites with Daypress 0.5" },
        post_excerpt:   { token: "{{post.excerpt}}",        label: "Post Excerpt",           sample: "Discover how native Rust SSR, SQLite WAL, and DayStudio Theme Templates replace legacy PHP stacks with sub-millisecond rendering." },
        post_content:   { token: "{{post.content}}",        label: "Full Post Body Content", sample: "<p>This is a live preview of <strong>{{post.content}}</strong>. When a visitor views any blog post matched by this layout, the full article body renders here automatically.</p>" },
        post_date:      { token: "{{post.date}}",           label: "Publish Date",           sample: "November 3, 2026" },
        author_name:    { token: "{{author.name}}",         label: "Author Name",            sample: "Admin" },
        featured_image: { token: "{{post.featured_image}}", label: "Featured Image",         sample: "/static/daypress-icon.svg" },
        post_url:       { token: "{{post.url}}",            label: "Post Permalink URL",     sample: "/blog/building-sub-millisecond-websites/" },
        site_title:     { token: "{{site.title}}",          label: "Site Title",             sample: "Daypress" },
        archive_title:  { token: "{{archive.title}}",       label: "Archive Title",          sample: "All Articles & Engineering Notes" }
    };
    window.DYNAMIC_TAG_META = DYNAMIC_TAG_META;

    function loadSamplePostsForDynamicPreview() {
        if (Array.isArray(window._dpSamplePostsCache)) return Promise.resolve(window._dpSamplePostsCache);
        if (window._dpSamplePostsPromise) return window._dpSamplePostsPromise;
        window._dpSamplePostsPromise = fetch("/wp-json/wp/v2/posts?per_page=10", { credentials: "same-origin" })
            .then(r => r.ok ? r.json() : [])
            .then(list => {
                window._dpSamplePostsCache = Array.isArray(list) ? list : [];
                if (window._dpSamplePostsCache.length > 0 && !window._dpActiveSamplePostId) {
                    window._dpActiveSamplePostId = window._dpSamplePostsCache[0].id;
                }
                if (window.renderCanvas) window.renderCanvas();
                return window._dpSamplePostsCache;
            })
            .catch(() => { window._dpSamplePostsCache = []; return []; });
        return window._dpSamplePostsPromise;
    }
    window.loadSamplePostsForDynamicPreview = loadSamplePostsForDynamicPreview;

    function resolveDynamicPreviewValue(dynTag) {
        const meta = DYNAMIC_TAG_META[dynTag];
        if (!meta) return "";
        if (!Array.isArray(window._dpSamplePostsCache)) {
            loadSamplePostsForDynamicPreview();
        }
        const posts = window._dpSamplePostsCache || [];
        const activePost = posts.find(p => Number(p.id) === Number(window._dpActiveSamplePostId)) || posts[0] || null;
        if (!activePost) return meta.sample;
        const stripHtml = (s) => String(s || "").replace(/<[^>]*>/g, "").trim();
        switch (dynTag) {
            case "post_title": return stripHtml(activePost.title && activePost.title.rendered) || meta.sample;
            case "post_excerpt": return stripHtml(activePost.excerpt && activePost.excerpt.rendered) || meta.sample;
            case "post_content": return (activePost.content && activePost.content.rendered) || meta.sample;
            case "post_date": return activePost.date ? new Date(activePost.date).toLocaleDateString("en-US", { month: "long", day: "numeric", year: "numeric" }) : meta.sample;
            case "author_name": return activePost.author_name || "Admin";
            case "featured_image": return activePost.featured_image_url || meta.sample;
            case "post_url": return activePost.link || ("/" + (activePost.slug || "sample-post") + "/");
            default: return meta.sample;
        }
    }
    window.resolveDynamicPreviewValue = resolveDynamicPreviewValue;

    function applyDynamicCanvasPreview(el, node) {
        if (!el || !node || !node.styles || !node.styles.raw_settings) return;
        const dynTag = node.styles.raw_settings["data-dynamic"];
        if (!dynTag) return;
        const meta = DYNAMIC_TAG_META[dynTag] || { token: `{{${dynTag}}}`, label: dynTag, sample: "" };
        const resolved = resolveDynamicPreviewValue(dynTag);
        el.classList.add("is-dynamic-bound");
        el.setAttribute("data-dynamic-badge", `⚡ ${meta.token} • ${meta.label}`);
        if (node.type === "Heading") {
            el.textContent = resolved;
        } else if (node.type === "Text") {
            el.innerHTML = dynTag === "post_content" ? resolved : `<p>${resolved}</p>`;
        } else if (node.type === "Button") {
            if (dynTag === "post_url") {
                el.href = resolved;
            } else {
                const icon = node.data && node.data.icon ? `<i class="${node.data.icon}"></i> ` : "";
                el.innerHTML = icon ? `${icon}<span>${resolved}</span>` : resolved;
            }
        } else if (node.type === "Image" && dynTag === "featured_image") {
            el.src = resolved;
        }
    }
    window.applyDynamicCanvasPreview = applyDynamicCanvasPreview;

    function attachPanelToggle(panel) {
        const header = panel.querySelector(".cad-panel-header");
        if (header) header.onclick = () => panel.classList.toggle("open");
    }

    function renderProInspectorExtensions(node) {
        const scrollHost = document.getElementById("inspector-panels-scroll")
            || document.getElementById("daystudio-inspector");
        if (!scrollHost || !node) return;
        let ext = document.getElementById("dp-pro-inspector-extensions");
        if (!ext) {
            ext = document.createElement("div");
            ext.id = "dp-pro-inspector-extensions";
            scrollHost.appendChild(ext);
        } else if (ext.parentElement !== scrollHost) {
            scrollHost.appendChild(ext);
        }
        ext.innerHTML = "";

        // 1. Dynamic Field Binding Panel (`data-dynamic`) for Heading, Text, Image, Button
        if (["Heading", "Text", "Image", "Button"].includes(node.type)) {
            if (!node.styles) node.styles = {};
            if (!node.styles.raw_settings) node.styles.raw_settings = {};
            const currentDyn = node.styles.raw_settings["data-dynamic"] || "";
            const meta = DYNAMIC_TAG_META[currentDyn] || null;
            const resolvedPreview = currentDyn ? resolveDynamicPreviewValue(currentDyn).replace(/<[^>]*>/g, "").slice(0, 120) : "";
            const posts = window._dpSamplePostsCache || [];
            const postOpts = posts.length > 0
                ? posts.map(p => `<option value="${p.id}" ${Number(p.id) === Number(window._dpActiveSamplePostId) ? "selected" : ""}>Post #${p.id}: ${(p.title && p.title.rendered ? p.title.rendered.replace(/<[^>]*>/g, "") : p.slug)}</option>`).join("")
                : `<option value="0">Sample Blog Post (Default)</option>`;

            const dynBox = document.createElement("div");
            dynBox.className = "cad-panel open";
            dynBox.innerHTML = `
                <div class="cad-panel-header">
                    <span class="cad-panel-title"><i class="fa-solid fa-bolt" style="color:var(--day-token);"></i> Dynamic Data Binding ${meta ? `<span style="margin-left:6px;padding:1px 6px;border-radius:3px;background:rgba(230,184,74,0.18);border:1px solid rgba(230,184,74,0.45);color:var(--day-token);font-family:var(--day-font-mono);font-size:9.5px;">ACTIVE</span>` : ""}</span>
                    <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
                </div>
                <div class="cad-panel-body">
                    <div class="cad-row">
                        <span class="cad-label">Source Tag</span>
                        <select id="dp-dyn-tag-select" class="cad-select" style="flex:1;">
                            <option value="" ${!currentDyn ? "selected" : ""}>Static Content (None)</option>
                            <option value="post_title" ${currentDyn === "post_title" ? "selected" : ""}>{{post.title}} — Post / Page Title</option>
                            <option value="post_excerpt" ${currentDyn === "post_excerpt" ? "selected" : ""}>{{post.excerpt}} — Post Excerpt</option>
                            <option value="post_content" ${currentDyn === "post_content" ? "selected" : ""}>{{post.content}} — Full Post Body Content</option>
                            <option value="post_date" ${currentDyn === "post_date" ? "selected" : ""}>{{post.date}} — Publish Date</option>
                            <option value="author_name" ${currentDyn === "author_name" ? "selected" : ""}>{{author.name}} — Author Name</option>
                            <option value="featured_image" ${currentDyn === "featured_image" ? "selected" : ""}>{{post.featured_image}} — Featured Image</option>
                            <option value="post_url" ${currentDyn === "post_url" ? "selected" : ""}>{{post.url}} — Post Permalink URL</option>
                            <option value="site_title" ${currentDyn === "site_title" ? "selected" : ""}>{{site.title}} — Site Title</option>
                            <option value="archive_title" ${currentDyn === "archive_title" ? "selected" : ""}>{{archive.title}} — Archive Title</option>
                        </select>
                    </div>
                    ${meta ? `
                    <div style="margin-top:8px;padding:9px 10px;border-radius:6px;background:rgba(230,184,74,0.08);border:1px solid rgba(230,184,74,0.38);">
                        <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:6px;">
                            <span style="font-size:10.5px;font-weight:700;color:var(--day-token);font-family:var(--day-font-mono);"><i class="fa-solid fa-circle-check" style="color:#10b981;margin-right:4px;"></i>Connected: ${meta.token}</span>
                            <button type="button" id="dp-dyn-unbind-btn" class="cad-btn sm secondary" style="height:20px;padding:0 6px;font-size:9.5px;">Unbind</button>
                        </div>
                        <div style="font-size:10.5px;color:var(--day-muted);margin-bottom:4px;">Live Resolved Output:</div>
                        <div style="padding:6px 8px;border-radius:4px;background:var(--day-surface-inset);border:1px solid var(--day-rule);font-size:11.5px;font-weight:600;color:var(--day-ink);word-break:break-word;">${resolvedPreview || "(Empty)"}</div>
                        <div class="cad-row" style="margin-top:7px;margin-bottom:0;">
                            <span class="cad-label" style="font-size:10px;">Preview Post</span>
                            <select id="dp-dyn-sample-post" class="cad-select" style="flex:1;font-size:10.5px;height:24px;">${postOpts}</select>
                        </div>
                    </div>` : `<div style="margin-top:4px;font-size:10.5px;color:var(--day-muted);">Select a dynamic source above to bind this ${node.type} to live database fields.</div>`}
                </div>
            `;
            attachPanelToggle(dynBox);
            ext.appendChild(dynBox);
            const sel = dynBox.querySelector("#dp-dyn-tag-select");
            const refreshDynState = (nextVal) => {
                if (!node.styles) node.styles = {};
                if (!node.styles.raw_settings) node.styles.raw_settings = {};
                if (nextVal) node.styles.raw_settings["data-dynamic"] = nextVal;
                else delete node.styles.raw_settings["data-dynamic"];
                if (window.markDirty) window.markDirty(true);
                if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                if (window.renderCanvas) window.renderCanvas();
                if (window.updateNavigator) window.updateNavigator();
                renderProInspectorExtensions(node);
            };
            if (sel) sel.onchange = () => refreshDynState(sel.value);
            const unbindBtn = dynBox.querySelector("#dp-dyn-unbind-btn");
            if (unbindBtn) unbindBtn.onclick = () => refreshDynState("");
            const postSel = dynBox.querySelector("#dp-dyn-sample-post");
            if (postSel) {
                postSel.onchange = () => {
                    window._dpActiveSamplePostId = parseInt(postSel.value, 10) || 0;
                    if (window.renderCanvas) window.renderCanvas();
                    renderProInspectorExtensions(node);
                };
            }
            if (!Array.isArray(window._dpSamplePostsCache)) {
                loadSamplePostsForDynamicPreview().then(() => renderProInspectorExtensions(node));
            }
        }

        // 2. Convert Section / Container to Global Symbol Action
        if (["Section", "Container"].includes(node.type)) {
            const convBox = document.createElement("div");
            convBox.className = "cad-panel open";
            convBox.innerHTML = `<div class="cad-panel-header"><span class="cad-panel-title"><i class="fa-solid fa-clone" style="color:var(--day-violet);"></i> Reusable Component</span><i class="fa-solid fa-chevron-right cad-panel-chevron"></i></div><div class="cad-panel-body"><button type="button" id="btn-convert-node-to-symbol" class="cad-btn secondary" style="width:100%;"><i class="fa-solid fa-clone" style="color:var(--day-violet);"></i> Convert ${node.type} to Global Symbol</button></div>`;
            attachPanelToggle(convBox);
            ext.appendChild(convBox);
            const btn = convBox.querySelector("#btn-convert-node-to-symbol");
            if (btn) btn.onclick = () => window.convertActiveNodeToSymbol(node);
        }

        // 3. SymbolInstance Inspector (Symbol Selector, Per-Instance Overrides, Detach)
        if (node.type === "SymbolInstance") {
            if (!node.data) node.data = {};
            if (!node.data.overrides) node.data.overrides = {};
            const symPanel = document.createElement("div");
            symPanel.className = "cad-panel open";
            symPanel.innerHTML = `<div class="cad-panel-body" style="color:var(--day-muted);">Loading symbol inspector...</div>`;
            ext.appendChild(symPanel);

            window.loadSymbolTemplatesCache(false).then(list => {
                const currentSymId = Number(node.data.symbol_id || 0);
                const activeSym = list.find(s => Number(s.id) === currentSymId);
                if (activeSym && activeSym.layout && activeSym.layout.root && (!node.children || node.children.length === 0)) {
                    node.children = JSON.parse(JSON.stringify(activeSym.layout.root.children || []));
                }
                const overrideableNodes = [];
                window.collectSymbolOverrideableNodes(node.children || [], overrideableNodes);

                const symOpts = `<option value="0">-- Select Global Symbol --</option>` + list.map(s =>
                    `<option value="${s.id}" ${Number(s.id) === currentSymId ? "selected" : ""}>#${s.id} — ${s.title}</option>`
                ).join("");

                const ovFieldsHtml = overrideableNodes.length === 0
                    ? `<div style="font-size:11px;color:var(--day-muted);">Select a Symbol above to configure per-instance text, link, or image overrides.</div>`
                    : overrideableNodes.map(item => {
                        const ov = (node.data.overrides && node.data.overrides[item.id]) || {};
                        if (item.type === "Image") {
                            return `<div class="repeater-item">
                                <div class="repeater-item-header"><span style="font-size:10px;font-weight:700;color:var(--day-violet);font-family:var(--day-font-mono);">${item.type} (${item.id})</span></div>
                                <div class="cad-row"><span class="cad-label">Image URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-ov-node="${item.id}" data-ov-field="src" value="${(ov.src || item.src || "").replace(/"/g, "&quot;")}"></div>
                                <div class="cad-row"><span class="cad-label">Alt Text</span><input type="text" class="cad-input" style="flex:1;" data-ov-node="${item.id}" data-ov-field="alt" value="${(ov.alt || item.alt || "").replace(/"/g, "&quot;")}"></div>
                            </div>`;
                        }
                        if (item.type === "Button") {
                            return `<div class="repeater-item">
                                <div class="repeater-item-header"><span style="font-size:10px;font-weight:700;color:var(--day-violet);font-family:var(--day-font-mono);">${item.type} (${item.id})</span></div>
                                <div class="cad-row"><span class="cad-label">Label</span><input type="text" class="cad-input" style="flex:1;" data-ov-node="${item.id}" data-ov-field="text" value="${(ov.text || item.text || "").replace(/"/g, "&quot;")}"></div>
                                <div class="cad-row"><span class="cad-label">URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-ov-node="${item.id}" data-ov-field="url" value="${(ov.url || item.url || "").replace(/"/g, "&quot;")}"></div>
                            </div>`;
                        }
                        return `<div class="repeater-item">
                            <div class="repeater-item-header"><span style="font-size:10px;font-weight:700;color:var(--day-violet);font-family:var(--day-font-mono);">${item.type} (${item.id})</span></div>
                            <div class="cad-row"><span class="cad-label">Text</span><input type="text" class="cad-input" style="flex:1;" data-ov-node="${item.id}" data-ov-field="text" value="${(ov.text || item.text || "").replace(/"/g, "&quot;")}"></div>
                        </div>`;
                    }).join("");

                symPanel.innerHTML = `
                    <div class="cad-panel-header">
                        <span class="cad-panel-title"><i class="fa-solid fa-clone" style="color:var(--day-violet);"></i> Synced Symbol Instance</span>
                        <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
                    </div>
                    <div class="cad-panel-body">
                        <div class="cad-row">
                            <span class="cad-label">Source Symbol</span>
                            <select id="dp-sym-instance-select" class="cad-select" style="flex:1;">${symOpts}</select>
                        </div>
                        <div class="cad-sub-label">Instance Overrides</div>
                        <div class="repeater-list">${ovFieldsHtml}</div>
                        <div style="display:flex;gap:6px;margin-top:8px;">
                            <button type="button" id="btn-edit-master-symbol" class="cad-btn secondary" style="flex:1;"><i class="fa-solid fa-pen-to-square"></i> Edit Master</button>
                            <button type="button" id="btn-detach-symbol-instance" class="cad-btn danger" style="flex:1;"><i class="fa-solid fa-link-slash"></i> Detach</button>
                        </div>
                    </div>
                `;
                attachPanelToggle(symPanel);

                const symSel = symPanel.querySelector("#dp-sym-instance-select");
                if (symSel) {
                    symSel.onchange = () => {
                        const nextId = parseInt(symSel.value, 10) || 0;
                        node.data.symbol_id = nextId;
                        const chosen = list.find(s => Number(s.id) === nextId);
                        if (chosen) {
                            node.data.label = chosen.title;
                            node.children = (chosen.layout && chosen.layout.root && Array.isArray(chosen.layout.root.children))
                                ? JSON.parse(JSON.stringify(chosen.layout.root.children))
                                : [];
                        }
                        if (window.markDirty) window.markDirty(true);
                        if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                        if (window.renderCanvas) window.renderCanvas();
                        renderProInspectorExtensions(node);
                    };
                }

                symPanel.querySelectorAll("[data-ov-node]").forEach(inp => {
                    inp.oninput = () => {
                        const targetNodeId = inp.getAttribute("data-ov-node");
                        const field = inp.getAttribute("data-ov-field");
                        if (!node.data.overrides) node.data.overrides = {};
                        if (!node.data.overrides[targetNodeId]) node.data.overrides[targetNodeId] = {};
                        node.data.overrides[targetNodeId][field] = inp.value;
                        if (window.markDirty) window.markDirty(true);
                        if (window.renderCanvas) window.renderCanvas();
                    };
                    if (inp.getAttribute("data-ov-field") === "src" && window.enhanceMediaInput) {
                        window.enhanceMediaInput(inp);
                    }
                });

                const editMasterBtn = symPanel.querySelector("#btn-edit-master-symbol");
                if (editMasterBtn) {
                    editMasterBtn.onclick = () => {
                        if (node.data.symbol_id && window.switchBuilderContext) {
                            window.switchBuilderContext("template", node.data.symbol_id, node.data.label || ("Symbol #" + node.data.symbol_id), "section_symbol");
                        }
                    };
                }

                const detachBtn = symPanel.querySelector("#btn-detach-symbol-instance");
                if (detachBtn) {
                    detachBtn.onclick = () => window.detachSymbolInstance(node);
                }
            });
        }

        // 4. NavMenu 2026 Mega Menu Builder Inspector
        if (node.type === "NavMenu" && window.renderNavMenuMegaMenuInspector) {
            window.renderNavMenuMegaMenuInspector(ext, node);
        }
    }
    window.renderProInspectorExtensions = renderProInspectorExtensions;
})();
"###;
