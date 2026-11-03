//! DayStudio 0.6 Filterable Collection Grid (`LoopGrid`) & Structured Field (`{{acf.*}}`) Inspector (`scripts/canvas_pro_collection_inspector.rs`).

pub const JS_CANVAS_PRO_COLLECTION_INSPECTOR: &str = r###"
(function() {
    function loadContentModelCache(force) {
        if (!force && window._dpContentModelCache) {
            return Promise.resolve(window._dpContentModelCache);
        }
        if (!force && window._dpContentModelPromise) {
            return window._dpContentModelPromise;
        }
        window._dpContentModelPromise = Promise.all([
            fetch("/wp-json/daypress/v1/content-types", { credentials: "same-origin" }).then(r => r.ok ? r.json() : []),
            fetch("/wp-json/daypress/v1/custom-taxonomies", { credentials: "same-origin" }).then(r => r.ok ? r.json() : []),
            fetch("/wp-json/daypress/v1/field-groups", { credentials: "same-origin" }).then(r => r.ok ? r.json() : [])
        ]).then(([post_types, taxonomies, field_groups]) => {
            window._dpContentModelCache = {
                post_types: Array.isArray(post_types) ? post_types : [],
                taxonomies: Array.isArray(taxonomies) ? taxonomies : [],
                field_groups: Array.isArray(field_groups) ? field_groups : []
            };
            if (window.DYNAMIC_TAG_META) {
                (window._dpContentModelCache.field_groups || []).forEach(grp => {
                    (grp.fields || []).forEach(f => {
                        if (f && f.name) {
                            const key = "acf." + f.name;
                            window.DYNAMIC_TAG_META[key] = {
                                token: `{{acf.${f.name}}}`,
                                label: `${f.label || f.name} (${grp.title})`,
                                sample: f.default_value || f.placeholder || `[${f.label || f.name}]`
                            };
                        }
                    });
                });
            }
            return window._dpContentModelCache;
        }).catch(() => {
            window._dpContentModelCache = { post_types: [], taxonomies: [], field_groups: [] };
            return window._dpContentModelCache;
        });
        return window._dpContentModelPromise;
    }
    window.loadContentModelCache = loadContentModelCache;

    function injectAcfOptionsIntoDynamicSelect(ext, node) {
        const sel = ext && ext.querySelector("#dp-dyn-tag-select");
        if (!sel || !node) return;
        loadContentModelCache(false).then(cm => {
            if (!cm || !Array.isArray(cm.field_groups) || cm.field_groups.length === 0) return;
            if (sel.querySelector("optgroup[data-acf-group]")) return;
            const currentDyn = (node.styles && node.styles.raw_settings && node.styles.raw_settings["data-dynamic"]) || "";
            cm.field_groups.forEach(grp => {
                if (!Array.isArray(grp.fields) || grp.fields.length === 0) return;
                const og = document.createElement("optgroup");
                og.setAttribute("data-acf-group", grp.group_key || grp.title);
                og.label = `Structured Fields: ${grp.title}`;
                grp.fields.forEach(f => {
                    if (!f || !f.name) return;
                    const val = `acf.${f.name}`;
                    const opt = document.createElement("option");
                    opt.value = val;
                    opt.textContent = `{{acf.${f.name}}} — ${f.label || f.name}`;
                    if (currentDyn === val) opt.selected = true;
                    og.appendChild(opt);
                });
                sel.appendChild(og);
            });
        });
    }
    window.injectAcfOptionsIntoDynamicSelect = injectAcfOptionsIntoDynamicSelect;

    function renderLoopGridCollectionInspector(ext, node) {
        if (!ext || !node || node.type !== "LoopGrid") return;
        if (!node.data) node.data = {};
        const d = node.data;

        const panel = document.createElement("div");
        panel.className = "cad-panel open";
        panel.innerHTML = `<div class="cad-panel-body" style="color:var(--day-muted);">Loading Collection Grid schema...</div>`;
        ext.appendChild(panel);

        loadContentModelCache(false).then(cm => {
            const cpts = cm.post_types || [];
            const taxes = cm.taxonomies || [];
            const curPt = d.post_type || "post";
            const curSkin = d.card_skin || "editorial";
            const curSort = d.default_sort || "date_desc";
            const curTax = d.taxonomy || "";
            const metaFieldsStr = Array.isArray(d.meta_fields) ? d.meta_fields.join(", ") : (d.meta_fields || "");

            // Also update the basic Post Type <select> in #widget-specific-fields if present
            const basicPtSel = document.querySelector("#widget-specific-fields select[data-key='post_type']");
            if (basicPtSel) {
                basicPtSel.innerHTML = `<option value="post" ${curPt === "post" ? "selected" : ""}>Posts (post)</option>`
                    + `<option value="page" ${curPt === "page" ? "selected" : ""}>Pages (page)</option>`
                    + cpts.map(c => `<option value="${c.slug}" ${curPt === c.slug ? "selected" : ""}>${c.label_plural || c.plural_name || c.slug} (${c.slug})</option>`).join("");
            }

            const taxOpts = `<option value="" ${!curTax ? "selected" : ""}>Auto-detect from Post Type</option>`
                + `<option value="category" ${curTax === "category" ? "selected" : ""}>Categories (category)</option>`
                + `<option value="post_tag" ${curTax === "post_tag" ? "selected" : ""}>Tags (post_tag)</option>`
                + taxes.map(t => `<option value="${t.slug}" ${curTax === t.slug ? "selected" : ""}>${t.label_plural || t.plural_name || t.slug} (${t.slug})</option>`).join("");

            panel.innerHTML = `
                <div class="cad-panel-header">
                    <span class="cad-panel-title"><i class="fa-solid fa-filter" style="color:var(--day-signal);"></i> Filterable Collection Grid</span>
                    <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
                </div>
                <div class="cad-panel-body">
                    <div class="cad-row">
                        <span class="cad-label">Card Skin</span>
                        <select class="cad-select" style="flex:1;" data-coll-key="card_skin">
                            <option value="editorial" ${curSkin === "editorial" ? "selected" : ""}>Editorial (Default)</option>
                            <option value="boxed" ${curSkin === "boxed" ? "selected" : ""}>Boxed Card</option>
                            <option value="minimal" ${curSkin === "minimal" ? "selected" : ""}>Minimal Index</option>
                            <option value="overlay" ${curSkin === "overlay" ? "selected" : ""}>Overlay Cover</option>
                            <option value="horizontal" ${curSkin === "horizontal" ? "selected" : ""}>Horizontal Split</option>
                        </select>
                    </div>
                    <div class="cad-row">
                        <span class="cad-label">Taxonomy</span>
                        <select class="cad-select" style="flex:1;" data-coll-key="taxonomy">${taxOpts}</select>
                    </div>
                    <div class="cad-row">
                        <span class="cad-label">Default Sort</span>
                        <select class="cad-select" style="flex:1;" data-coll-key="default_sort">
                            <option value="date_desc" ${curSort === "date_desc" ? "selected" : ""}>Newest First</option>
                            <option value="date_asc" ${curSort === "date_asc" ? "selected" : ""}>Oldest First</option>
                            <option value="title_asc" ${curSort === "title_asc" ? "selected" : ""}>Title A–Z</option>
                            <option value="title_desc" ${curSort === "title_desc" ? "selected" : ""}>Title Z–A</option>
                        </select>
                    </div>
                    <div class="cad-sub-label" style="margin-top:6px;">Interactive Visitor Controls</div>
                    <div class="cad-row"><span class="cad-label">Filter Bar Pills</span><input type="checkbox" data-coll-bool="enable_filter_bar" ${d.enable_filter_bar ? "checked" : ""}></div>
                    <div class="cad-row"><span class="cad-label">Sort Dropdown</span><input type="checkbox" data-coll-bool="enable_sort" ${d.enable_sort ? "checked" : ""}></div>
                    <div class="cad-row"><span class="cad-label">Instant Search</span><input type="checkbox" data-coll-bool="enable_search" ${d.enable_search ? "checked" : ""}></div>
                    <div class="cad-sub-label" style="margin-top:6px;">Card Elements &amp; Structured Badges</div>
                    <div class="cad-row"><span class="cad-label">Show Image</span><input type="checkbox" data-coll-bool="show_thumbnail" ${d.show_thumbnail !== false ? "checked" : ""}><span class="cad-label">Excerpt</span><input type="checkbox" data-coll-bool="show_excerpt" ${d.show_excerpt !== false ? "checked" : ""}></div>
                    <div class="cad-row"><span class="cad-label">Show Meta</span><input type="checkbox" data-coll-bool="show_meta" ${d.show_meta !== false ? "checked" : ""}><span class="cad-label">Pagination</span><input type="checkbox" data-coll-bool="show_pagination" ${d.show_pagination ? "checked" : ""}></div>
                    <div class="cad-row" style="flex-direction:column;align-items:stretch;gap:4px;">
                        <span class="cad-label">Structured Field Badges (comma-separated keys)</span>
                        <input type="text" class="cad-input font-mono" data-coll-meta-fields="1" value="${metaFieldsStr.replace(/"/g, "&quot;")}" placeholder="client_name, headline_metric, location">
                    </div>
                    <div class="cad-row" style="margin-top:4px;">
                        <a href="/d-admin/content-types" target="_blank" style="font-size:11px;color:var(--day-signal,#93A1FF);text-decoration:none;font-weight:600;">&#8599; Manage Content Types &amp; Structured Fields</a>
                    </div>
                </div>
            `;

            const hdr = panel.querySelector(".cad-panel-header");
            if (hdr) hdr.onclick = () => panel.classList.toggle("open");

            panel.querySelectorAll("[data-coll-key]").forEach(sel => {
                sel.onchange = () => {
                    const k = sel.getAttribute("data-coll-key");
                    node.data[k] = sel.value || null;
                    if (window.markDirty) window.markDirty(true);
                    if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                    if (window.renderCanvas) window.renderCanvas();
                };
            });
            panel.querySelectorAll("[data-coll-bool]").forEach(chk => {
                chk.onchange = () => {
                    const k = chk.getAttribute("data-coll-bool");
                    node.data[k] = !!chk.checked;
                    if (window.markDirty) window.markDirty(true);
                    if (window.pushHistorySnapshot) window.pushHistorySnapshot();
                    if (window.renderCanvas) window.renderCanvas();
                };
            });
            const mfInput = panel.querySelector("[data-coll-meta-fields]");
            if (mfInput) {
                mfInput.oninput = () => {
                    node.data.meta_fields = mfInput.value.split(",").map(s => s.trim()).filter(Boolean);
                    if (window.markDirty) window.markDirty(true);
                    if (window.renderCanvas) window.renderCanvas();
                };
            }
        });
    }
    window.renderLoopGridCollectionInspector = renderLoopGridCollectionInspector;
})();
"###;
