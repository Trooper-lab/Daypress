//! Theme Templates Drawer, In-Studio Switcher, and Display Conditions Manager (`scripts/theme_drawer.rs`).

pub const JS_THEME_DRAWER: &str = r###"
(function() {
    let currentTemplateIdForConditions = null;
    let pendingTemplateDelete = null;
    let cachedStarters = null;

    async function renderThemeDrawer(container) {
        if (!container) return;
        container.innerHTML = `
            <div class="drawer-header">
                <span class="drawer-title">Theme Templates</span>
                <button class="cad-btn primary" id="btn-new-theme-template" style="height: 22px; padding: 0 6px; font-size: 10px;">
                    <i class="fa-solid fa-plus"></i> New
                </button>
            </div>
            <div class="drawer-search-well" style="display:flex; justify-content:space-between; align-items:center;">
                <span style="font-size:11px; color:var(--day-muted);">Header / Footer & Layouts</span>
                <button class="cad-btn secondary" id="btn-open-starters" style="height: 22px; padding: 0 6px; font-size: 10px;">
                    <i class="fa-solid fa-shapes"></i> Starters
                </button>
            </div>
            <div class="drawer-scroll-body" id="theme-templates-body">
                <p style="font-size: 12px; color: var(--day-muted); padding: 8px 0;">Loading templates from SQLite...</p>
            </div>
        `;

        const body = document.getElementById("theme-templates-body");
        try {
            const templates = await window.StudioApi.getTemplates();
            renderTemplatesList(body, templates);
        } catch(e) {
            body.innerHTML = `<p style="font-size: 12px; color: var(--day-crimson); padding: 8px 0;">Error loading templates: ${e.message}</p>`;
        }

        const btnStarters = document.getElementById("btn-open-starters");
        if (btnStarters) btnStarters.onclick = () => openCreateTemplateModal(null, true);

        const btnNew = document.getElementById("btn-new-theme-template");
        if (btnNew) btnNew.onclick = () => openCreateTemplateModal();
    }
    window.renderThemeDrawer = renderThemeDrawer;

    function renderTemplatesList(container, templates) {
        if (!templates || templates.length === 0) {
            container.innerHTML = `<p style="font-size: 12px; color: var(--day-muted); padding: 8px 0;">No custom templates found in SQLite.</p>`;
            return;
        }

        container.innerHTML = "";
        templates.forEach(t => {
            const card = document.createElement("div");
            card.className = "template-card";
            card.dataset.templateId = t.id;
            card.innerHTML = `
                <div class="template-card-header">
                    <span class="template-card-title">${t.title}</span>
                    <span class="template-type-badge">${t.template_type}</span>
                </div>
                <div class="template-condition-pill">
                    <i class="fa-solid fa-filter"></i> ${t.conditions_summary || "Entire Site"}
                </div>
                <div class="template-card-actions">
                    <button class="template-act-btn btn-edit-template"><i class="fa-solid fa-pen-to-square"></i> Edit In Studio</button>
                    <button class="template-act-btn btn-cond-template"><i class="fa-solid fa-sliders"></i> Conditions</button>
                    <button class="template-act-btn btn-del-template" style="color:var(--day-crimson);"><i class="fa-regular fa-trash-can"></i></button>
                </div>
            `;

            card.querySelector(".btn-edit-template").onclick = () => switchStudioContext(t.id);
            card.querySelector(".btn-cond-template").onclick = () => openConditionsModal(t.id, t.title);
            card.querySelector(".btn-del-template").onclick = () => openDeleteConfirmModal(t.id, t.title, () => card.remove());
            container.appendChild(card);
        });
    }

    async function switchStudioContext(templateId) {
        const state = window.DayStudioState;
        if (state.isDirty) {
            try {
                await window.StudioApi.saveLayout();
                if (window.showToast) window.showToast("Saved current layout before switching");
            } catch(e) {
                console.warn("Auto-save before context switch:", e);
            }
        }

        try {
            const template = await window.StudioApi.getTemplate(templateId);
            state.currentContext = { type: "template", id: template.id, title: template.title, templateType: template.template_type };
            state.pageAst = window.normalizeAst(template.layout);
            state.activeElementId = null;

            if (window.updateViewLiveLink) window.updateViewLiveLink();

            window.markDirty(false);
            window.renderCanvas();
            if (window.updateNavigator) window.updateNavigator();
            if (window.renderInspector) window.renderInspector();
            if (window.showToast) window.showToast(`Switched to "${template.title}"`);
        } catch(err) {
            if (window.showToast) window.showToast("Failed to load template: " + err.message);
        }
    }
    window.switchStudioContext = switchStudioContext;

    // -------------------------------------------------------------
    // NON-BLOCKING CREATE TEMPLATE MODAL CONTROLLER
    // -------------------------------------------------------------
    async function openCreateTemplateModal(defaultType, focusStarters) {
        const modal = document.getElementById("modal-create-template");
        const titleInput = document.getElementById("input-template-title");
        const typeSelect = document.getElementById("select-template-type");
        const starterSelect = document.getElementById("select-starter-template");
        const chipsContainer = document.getElementById("starter-chips-list");
        const errBox = document.getElementById("create-template-error");

        if (!modal) return;
        if (errBox) { errBox.style.display = "none"; errBox.textContent = ""; }

        if (defaultType && typeSelect) typeSelect.value = defaultType;
        if (titleInput) {
            const t = typeSelect ? typeSelect.value : "header";
            titleInput.value = t === "header" ? "Custom Header" : (t === "footer" ? "Custom Footer" : "New Template");
        }

        modal.classList.add("open");
        if (titleInput) { titleInput.focus(); titleInput.select(); }

        try {
            if (!cachedStarters) cachedStarters = await window.StudioApi.getStarters();
            populateStarterControls(cachedStarters, starterSelect, chipsContainer, titleInput, typeSelect);
        } catch(e) {
            console.warn("Could not load starters:", e);
        }
        wireCreateTemplateEvents();
    }
    window.openCreateTemplateModal = openCreateTemplateModal;

    function populateStarterControls(starters, selectEl, chipsEl, titleInput, typeSelect) {
        if (selectEl) {
            selectEl.innerHTML = `<option value="blank" selected>Blank Canvas (Clean Root AST)</option>`;
            starters.forEach(s => {
                const opt = document.createElement("option");
                opt.value = s.id || s.starter_id;
                opt.textContent = `[${s.template_type.toUpperCase()}] ${s.title}`;
                selectEl.appendChild(opt);
            });
        }

        if (chipsEl) {
            chipsEl.innerHTML = `
                <div class="starter-chip-card selected" data-starter="blank">
                    <div class="starter-chip-top">
                        <span class="starter-chip-title">Blank Canvas</span>
                        <span class="starter-chip-badge">Root</span>
                    </div>
                    <span class="starter-chip-desc">Clean empty layout container ready for building</span>
                </div>
            `;

            starters.forEach(s => {
                const card = document.createElement("div");
                card.className = "starter-chip-card";
                card.dataset.starter = s.id || s.starter_id;
                card.innerHTML = `
                    <div class="starter-chip-top">
                        <span class="starter-chip-title">${s.title}</span>
                        <span class="starter-chip-badge">${s.template_type}</span>
                    </div>
                    <span class="starter-chip-desc">${s.description || ""}</span>
                `;
                chipsEl.appendChild(card);
            });

            chipsEl.querySelectorAll(".starter-chip-card").forEach(c => {
                c.onclick = () => {
                    chipsEl.querySelectorAll(".starter-chip-card").forEach(x => x.classList.remove("selected"));
                    c.classList.add("selected");
                    const stVal = c.dataset.starter;
                    if (selectEl) selectEl.value = stVal;

                    if (stVal !== "blank") {
                        const match = starters.find(s => (s.id || s.starter_id) === stVal);
                        if (match) {
                            if (titleInput) titleInput.value = match.title;
                            if (typeSelect && match.template_type) typeSelect.value = match.template_type.toLowerCase();
                        }
                    }
                };
            });
        }
    }

    function wireCreateTemplateEvents() {
        const modal = document.getElementById("modal-create-template");
        const btnClose = document.getElementById("btn-close-create-template");
        const btnCancel = document.getElementById("btn-cancel-create-template");
        const btnSubmit = document.getElementById("btn-submit-create-template");
        const titleInput = document.getElementById("input-template-title");
        const typeSelect = document.getElementById("select-template-type");
        const starterSelect = document.getElementById("select-starter-template");
        const errBox = document.getElementById("create-template-error");

        const closeModal = () => modal.classList.remove("open");
        if (btnClose) btnClose.onclick = closeModal;
        if (btnCancel) btnCancel.onclick = closeModal;

        if (btnSubmit) {
            btnSubmit.onclick = async () => {
                const title = titleInput ? titleInput.value.trim() : "";
                if (!title) {
                    if (errBox) { errBox.style.display = "block"; errBox.textContent = "Template name cannot be empty."; }
                    return;
                }

                const type = typeSelect ? typeSelect.value : "header";
                const starter = starterSelect ? starterSelect.value : "blank";
                btnSubmit.disabled = true;
                btnSubmit.innerHTML = `<i class="fa-solid fa-spinner fa-spin"></i> Creating...`;

                try {
                    if (starter === "blank") {
                        await window.StudioApi.createTemplate({
                            title: title, template_type: type,
                            layout: { version: 1, root: { id: "root", type: "Root", children: [] } },
                            status: "publish", priority: 10
                        });
                    } else {
                        const created = await window.StudioApi.installStarter(starter, title);
                        if (created && created.id && created.title !== title) {
                            await window.StudioApi.updateTemplate(created.id, { title: title });
                        }
                    }
                    closeModal();
                    if (window.showToast) window.showToast(`Template "${title}" created successfully`);
                    const container = document.getElementById("drawer-panel-container");
                    if (container) renderThemeDrawer(container);
                } catch(err) {
                    if (errBox) { errBox.style.display = "block"; errBox.textContent = err.message; }
                    if (window.showToast) window.showToast("Error creating template: " + err.message);
                } finally {
                    btnSubmit.disabled = false;
                    btnSubmit.innerHTML = `<i class="fa-solid fa-plus"></i> <span>Create Template</span>`;
                }
            };
        }
    }

    // -------------------------------------------------------------
    // NON-BLOCKING DELETE CONFIRMATION MODAL CONTROLLER
    // -------------------------------------------------------------
    function openDeleteConfirmModal(templateId, title, onSuccess) {
        pendingTemplateDelete = { id: templateId, title, onSuccess };
        const modal = document.getElementById("modal-confirm-delete");
        const msg = document.getElementById("lbl-delete-confirm-msg");
        const btnCancel = document.getElementById("btn-cancel-confirm-delete");
        const btnClose = document.getElementById("btn-close-confirm-delete");
        const btnAction = document.getElementById("btn-confirm-delete-action");

        if (!modal) return;
        if (msg) msg.textContent = `Permanently delete template "${title}" from SQLite?`;
        modal.classList.add("open");

        const close = () => { modal.classList.remove("open"); pendingTemplateDelete = null; };
        if (btnCancel) btnCancel.onclick = close;
        if (btnClose) btnClose.onclick = close;

        if (btnAction) {
            btnAction.onclick = async () => {
                if (!pendingTemplateDelete) return;
                try {
                    await window.StudioApi.deleteTemplate(pendingTemplateDelete.id);
                    if (pendingTemplateDelete.onSuccess) pendingTemplateDelete.onSuccess();
                    if (window.showToast) window.showToast("Template deleted");
                    close();
                } catch(e) {
                    if (window.showToast) window.showToast("Delete failed: " + e.message);
                    close();
                }
            };
        }
    }

    // -------------------------------------------------------------
    // DISPLAY CONDITIONS MODAL
    // -------------------------------------------------------------
    async function openConditionsModal(templateId, title) {
        currentTemplateIdForConditions = templateId;
        const modal = document.getElementById("modal-conditions");
        const lblTitle = document.getElementById("lbl-conditions-title");
        const listContainer = document.getElementById("conditions-list-container");

        if (lblTitle) lblTitle.textContent = `Display Conditions: ${title}`;
        if (!modal || !listContainer) return;

        modal.classList.add("open");
        listContainer.innerHTML = "<p>Loading conditions...</p>";

        try {
            const [conditions, siteItems] = await Promise.all([
                window.StudioApi.getConditions(templateId),
                window.StudioApi.getPagesAndPosts()
            ]);
            window._cachedCondSiteItems = siteItems;
            renderConditionsRows(listContainer, conditions);
        } catch(e) {
            listContainer.innerHTML = `<p style="color:var(--day-crimson);">Error: ${e.message}</p>`;
        }
    }

    function renderConditionsRows(container, conditions) {
        container.innerHTML = "";
        const siteItems = window._cachedCondSiteItems || { pages: [], posts: [] };
        const allItems = [...(siteItems.pages || []).map(p => ({ id: String(p.id), label: `[Page] ${p.title?.rendered || p.title || p.slug} (#${p.id})` })), ...(siteItems.posts || []).map(p => ({ id: String(p.id), label: `[Post] ${p.title?.rendered || p.title || p.slug} (#${p.id})` }))];
        const list = Array.isArray(conditions) && conditions.length > 0
            ? conditions
            : [{ condition_type: "include", rule_target: "entire_site", target_id: "", priority: 10 }];

        list.forEach(cond => {
            const rt = cond.rule_target === "post_id" ? "page" : (cond.rule_target || "entire_site");
            const needsTarget = rt === "page" || rt === "post";
            const row = document.createElement("div");
            row.className = "cad-row condition-rule-row";
            row.style.cssText = "margin-bottom: 8px; gap: 8px; background: var(--day-surface-inset); padding: 8px; border-radius: 4px; flex-wrap: wrap;";
            const optsHtml = allItems.map(it => `<option value="${it.id}" ${String(cond.target_id || "") === it.id ? "selected" : ""}>${it.label}</option>`).join("");
            row.innerHTML = `
                <select class="cad-select cond-type" style="width: 96px;">
                    <option value="include" ${cond.condition_type === "include" ? "selected" : ""}>INCLUDE</option>
                    <option value="exclude" ${cond.condition_type === "exclude" ? "selected" : ""}>EXCLUDE</option>
                </select>
                <select class="cad-select cond-target" style="flex: 1; min-width: 150px;">
                    <option value="entire_site" ${rt === "entire_site" ? "selected" : ""}>Entire Site (All Pages & Posts)</option>
                    <option value="front_page" ${rt === "front_page" ? "selected" : ""}>Front Page Only</option>
                    <option value="all_pages" ${rt === "all_pages" ? "selected" : ""}>All Static Pages</option>
                    <option value="all_posts" ${rt === "all_posts" ? "selected" : ""}>All Blog Posts</option>
                    <option value="page" ${rt === "page" ? "selected" : ""}>Specific Page</option>
                    <option value="post" ${rt === "post" ? "selected" : ""}>Specific Blog Post</option>
                    <option value="archive" ${rt === "archive" ? "selected" : ""}>All Archives</option>
                    <option value="not_found" ${rt === "not_found" ? "selected" : ""}>404 Not Found Page</option>
                </select>
                <select class="cad-select cond-target-id" style="width: 165px; display: ${needsTarget ? "block" : "none"};">
                    <option value="">-- Select Item --</option>
                    ${optsHtml}
                </select>
                <input type="number" class="cad-input cond-priority font-mono" style="width: 64px;" value="${cond.priority || 10}" title="Priority">
                <button class="cad-btn secondary btn-del-cond" style="color: var(--day-crimson);"><i class="fa-solid fa-xmark"></i></button>
            `;
            const targetSel = row.querySelector(".cond-target");
            const idSel = row.querySelector(".cond-target-id");
            targetSel.onchange = () => {
                const v = targetSel.value;
                idSel.style.display = (v === "page" || v === "post") ? "block" : "none";
            };
            row.querySelector(".btn-del-cond").onclick = () => row.remove();
            container.appendChild(row);
        });

        const btnAdd = document.getElementById("btn-add-condition-rule");
        if (btnAdd) {
            btnAdd.onclick = () => {
                const dummy = { condition_type: "include", rule_target: "entire_site", target_id: "", priority: 10 };
                renderConditionsRows(container, [...getConditionsFromModal(), dummy]);
            };
        }
        const btnSave = document.getElementById("btn-save-conditions");
        if (btnSave) {
            btnSave.onclick = async () => {
                try {
                    await window.StudioApi.setConditions(currentTemplateIdForConditions, getConditionsFromModal());
                    document.getElementById("modal-conditions").classList.remove("open");
                    if (window.showToast) window.showToast("Conditions updated successfully");
                    const drawerCont = document.getElementById("drawer-panel-container");
                    if (drawerCont) renderThemeDrawer(drawerCont);
                } catch(err) {
                    if (window.showToast) window.showToast("Error saving conditions: " + err.message);
                }
            };
        }
        const btnCancel = document.getElementById("btn-cancel-conditions");
        const btnClose = document.getElementById("btn-close-conditions");
        if (btnCancel) btnCancel.onclick = () => document.getElementById("modal-conditions").classList.remove("open");
        if (btnClose) btnClose.onclick = () => document.getElementById("modal-conditions").classList.remove("open");
    }

    function getConditionsFromModal() {
        const list = [];
        document.querySelectorAll(".condition-rule-row").forEach(r => {
            const rt = r.querySelector(".cond-target").value;
            const idEl = r.querySelector(".cond-target-id");
            const tid = (rt === "page" || rt === "post") && idEl ? idEl.value : "";
            list.push({ condition_type: r.querySelector(".cond-type").value, rule_target: rt, target_id: tid, priority: parseInt(r.querySelector(".cond-priority").value, 10) || 10 });
        });
        return list;
    }
})();
"###;
