//! Dynamic & Theme/Form Pro Blocks Canvas DOM Builders — Part 4: MegaMenu & Dynamic Token Inspector (`scripts/canvas_pro_megamenu_inspector.rs`).

pub const JS_CANVAS_PRO_MEGAMENU_INSPECTOR: &str = r###"
            if (selType === "NavMenu") {
                const items = ensureDefaultNavMenuItems(selNode);
                const navItemsEditorHtml = items.map(function(item, idx) {
                    const curMegaId = item.mega_menu_template_id != null ? String(item.mega_menu_template_id) : "";
                    const itemMegaOptions = `<option value="">None (Standard Link)</option>` +
                        megaMenuTemplatesList.map(function(t) {
                            const sel = String(t.id) === curMegaId ? "selected" : "";
                            return `<option value="${t.id}" ${sel}>[${escHtml(t.template_type)}] ${escHtml(t.title)} (#${t.id})</option>`;
                        }).join("");

                    return `
                        <div class="dp-nav-item-config-card" data-item-index="${idx}" style="background:rgba(15,23,42,0.5);border:1px solid rgba(148,163,184,0.25);border-radius:6px;padding:8px;display:flex;flex-direction:column;gap:6px;">
                            <div style="display:flex;justify-content:space-between;align-items:center;">
                                <strong style="font-size:11px;color:#e2e8f0;">#${idx + 1}: ${escHtml(item.label || item.title || "Menu Item")}</strong>
                                <button type="button" class="dp-nav-item-remove" data-item-index="${idx}" style="background:none;border:none;color:#f87171;cursor:pointer;font-size:11px;" title="Remove Item">&times;</button>
                            </div>
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:6px;">
                                <input type="text" class="cad-input dp-nav-item-input dp-ignore-dynamic-target" data-item-index="${idx}" data-field="label" placeholder="Label" value="${escHtml(item.label || item.title || '')}" style="font-size:11px;padding:4px 6px;" />
                                <input type="text" class="cad-input dp-nav-item-input dp-ignore-dynamic-target" data-item-index="${idx}" data-field="url" placeholder="URL (/products)" value="${escHtml(item.url || item.href || '')}" style="font-size:11px;padding:4px 6px;" />
                            </div>
                            <div>
                                <label style="display:block;font-size:10.5px;color:#94a3b8;margin-bottom:2px;">Mega Menu Panel (<code>mega_menu_template_id</code>)</label>
                                <select class="cad-input dp-nav-item-input dp-ignore-dynamic-target" data-item-index="${idx}" data-field="mega_menu_template_id" style="width:100%;font-size:11px;padding:4px 6px;">
                                    ${itemMegaOptions}
                                </select>
                            </div>
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:6px;">
                                <input type="text" class="cad-input dp-nav-item-input dp-ignore-dynamic-target" data-item-index="${idx}" data-field="icon" placeholder="Icon (fa-solid fa-bolt)" value="${escHtml(item.icon || '')}" style="font-size:11px;padding:4px 6px;" />
                                <input type="text" class="cad-input dp-nav-item-input dp-ignore-dynamic-target" data-item-index="${idx}" data-field="badge" placeholder="Badge (NEW / 0.5)" value="${escHtml(item.badge || '')}" style="font-size:11px;padding:4px 6px;" />
                            </div>
                            <input type="text" class="cad-input dp-nav-item-input dp-ignore-dynamic-target" data-item-index="${idx}" data-field="description" placeholder="Item description subtitle..." value="${escHtml(item.description || '')}" style="width:100%;font-size:11px;padding:4px 6px;" />
                        </div>
                    `;
                }).join("");

                htmlParts.push(`
                    <div id="dp-inspector-megamenu-config" style="background:rgba(37,99,235,0.08);border:1px solid rgba(37,99,235,0.35);border-radius:8px;padding:10px;display:flex;flex-direction:column;gap:8px;">
                        <div style="display:flex;justify-content:space-between;align-items:center;">
                            <strong style="font-size:11.5px;text-transform:uppercase;letter-spacing:0.04em;color:#93c5fd;">
                                <i class="fa-solid fa-sitemap" style="margin-right:5px;"></i>Visual Mega Menu Items
                            </strong>
                            <button type="button" id="dp-btn-add-nav-item" class="cad-btn primary" style="font-size:10.5px;padding:2px 8px;height:22px;cursor:pointer;">+ Add Item</button>
                        </div>
                        <div style="display:flex;flex-direction:column;gap:8px;">
                            ${navItemsEditorHtml}
                        </div>
                    </div>
                `);
            }

            const activeDynamic = (selNode.data && selNode.data["data-dynamic"])
                || (selNode.attributes && selNode.attributes["data-dynamic"])
                || "";
            const tokenOptionsHtml = `<option value="">Insert Dynamic Token {{...}}</option>` +
                DYNAMIC_TOKENS.map(function(dt) {
                    const sel = dt.token === activeDynamic ? "selected" : "";
                    return `<option value="${escHtml(dt.token)}" ${sel}>${escHtml(dt.token)} — ${escHtml(dt.label)}</option>`;
                }).join("");

            const tokenPillsHtml = DYNAMIC_TOKENS.map(function(dt) {
                const isBound = dt.token === activeDynamic;
                const pillStyle = isBound
                    ? "background:#2563eb;color:#ffffff;border:1px solid #3b82f6;"
                    : "background:rgba(15,23,42,0.6);color:#cbd5e1;border:1px solid rgba(148,163,184,0.28);";
                return `<button type="button" class="dp-dynamic-token-pill" data-token="${escHtml(dt.token)}" title="Bind ${escHtml(dt.label)}" style="${pillStyle}border-radius:4px;padding:3px 6px;font-size:10.5px;font-family:monospace;cursor:pointer;">${escHtml(dt.token)}</button>`;
            }).join("");

            htmlParts.push(`
                <div id="dp-inspector-dynamic-binding" style="background:rgba(15,23,42,0.45);border:1px solid rgba(148,163,184,0.25);border-radius:8px;padding:10px;display:flex;flex-direction:column;gap:8px;">
                    <div style="display:flex;justify-content:space-between;align-items:center;">
                        <strong style="font-size:11px;text-transform:uppercase;letter-spacing:0.04em;color:#e2e8f0;">
                            <i class="fa-solid fa-bolt" style="color:#f59e0b;margin-right:4px;"></i>Dynamic Field Binding
                        </strong>
                        ${activeDynamic ? `<span style="font-size:10px;color:#38bdf8;font-family:monospace;">data-dynamic="${escHtml(activeDynamic)}"</span>` : ""}
                    </div>
                    <select id="dp-dynamic-token-select" class="cad-input dp-ignore-dynamic-target" style="width:100%;font-size:11.5px;padding:5px;">
                        ${tokenOptionsHtml}
                    </select>
                    <div style="display:flex;flex-wrap:wrap;gap:4px;">
                        ${tokenPillsHtml}
                    </div>
                </div>
            `);

            extContainer.innerHTML = htmlParts.join("");
            inspector.appendChild(extContainer);

            const saveAsSymBtn = extContainer.querySelector("#dp-inspector-save-as-symbol");
            if (saveAsSymBtn) {
                saveAsSymBtn.addEventListener("click", function() {
                    saveSelectedSectionAsGlobalSymbol();
                });
            }

            const masterSymSel = extContainer.querySelector("#dp-select-master-symbol");
            if (masterSymSel) {
                masterSymSel.addEventListener("change", function() {
                    const nextId = Number(masterSymSel.value || 0);
                    selNode.symbol_id = nextId;
                    if (!selNode.data) selNode.data = {};
                    selNode.data.symbol_id = nextId;
                    const found = symbolsCache[nextId];
                    if (found) selNode.data.symbol_title = found.title;
                    notifyStudioMutation(selNode.id);
                });
            }

            extContainer.querySelectorAll(".dp-ov-input").forEach(function(input) {
                input.addEventListener("input", function() {
                    const cid = input.getAttribute("data-child-id");
                    const field = input.getAttribute("data-field");
                    if (!cid || !field) return;
                    if (!selNode.overrides) selNode.overrides = {};
                    if (!selNode.data) selNode.data = {};
                    if (!selNode.data.overrides) selNode.data.overrides = selNode.overrides;
                    if (!selNode.overrides[cid]) selNode.overrides[cid] = {};
                    selNode.overrides[cid][field] = input.value;
                    selNode.data.overrides = selNode.overrides;
                    try {
                        if (typeof renderCanvas === "function") renderCanvas();
                    } catch (_) {}
                });
            });

            const addOvChildBtn = extContainer.querySelector("#dp-btn-add-override-child");
            if (addOvChildBtn) {
                addOvChildBtn.addEventListener("click", function() {
                    const cidInput = extContainer.querySelector("#dp-new-override-child-id");
                    const cid = cidInput ? cidInput.value.trim() : "";
                    if (!cid) return;
                    if (!selNode.overrides) selNode.overrides = {};
                    if (!selNode.overrides[cid]) {
                        selNode.overrides[cid] = { text: "", src: "", alt: "", url: "" };
                    }
                    if (!selNode.data) selNode.data = {};
                    selNode.data.overrides = selNode.overrides;
                    enhanceInspectorPanel();
                });
            }

            const detachBtn = extContainer.querySelector("#btn-detach-symbol-instance");
            if (detachBtn) {
                detachBtn.addEventListener("click", function() {
                    detachSymbolInstance(selNode.id);
                });
            }

            const addNavBtn = extContainer.querySelector("#dp-btn-add-nav-item");
            if (addNavBtn) {
                addNavBtn.addEventListener("click", function() {
                    const items = ensureDefaultNavMenuItems(selNode);
                    items.push({
                        id: "nav-" + Date.now().toString(36),
                        label: "New Link",
                        url: "#",
                        icon: "",
                        badge: "",
                        description: "",
                        mega_menu_template_id: null
                    });
                    notifyStudioMutation(selNode.id);
                });
            }

            extContainer.querySelectorAll(".dp-nav-item-remove").forEach(function(btn) {
                btn.addEventListener("click", function() {
                    const idx = Number(btn.getAttribute("data-item-index"));
                    const items = ensureDefaultNavMenuItems(selNode);
                    if (idx >= 0 && idx < items.length) {
                        items.splice(idx, 1);
                        notifyStudioMutation(selNode.id);
                    }
                });
            });

            extContainer.querySelectorAll(".dp-nav-item-input").forEach(function(inp) {
                const evtName = inp.tagName === "SELECT" ? "change" : "input";
                inp.addEventListener(evtName, function() {
                    const idx = Number(inp.getAttribute("data-item-index"));
                    const field = inp.getAttribute("data-field");
                    const items = ensureDefaultNavMenuItems(selNode);
                    if (!items[idx] || !field) return;
                    if (field === "mega_menu_template_id") {
                        items[idx].mega_menu_template_id = inp.value ? Number(inp.value) : null;
                    } else {
                        items[idx][field] = inp.value;
                        if (field === "label") items[idx].title = inp.value;
                        if (field === "url") items[idx].href = inp.value;
                    }
                    try {
                        if (typeof renderCanvas === "function") renderCanvas();
                    } catch (_) {}
                });
            });

            const tokenSel = extContainer.querySelector("#dp-dynamic-token-select");
            if (tokenSel) {
                tokenSel.addEventListener("change", function() {
                    if (tokenSel.value) {
                        applyDynamicTokenToSelection(tokenSel.value);
                    }
                });
            }
            extContainer.querySelectorAll(".dp-dynamic-token-pill").forEach(function(pill) {
                pill.addEventListener("click", function(e) {
                    e.preventDefault();
                    const tok = pill.getAttribute("data-token");
                    applyDynamicTokenToSelection(tok);
                });
            });
        } finally {
            isEnhancingInspector = false;
        }
    }
"###;
