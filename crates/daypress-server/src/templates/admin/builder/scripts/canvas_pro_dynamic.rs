//! Dynamic & Theme/Form Pro Blocks Canvas DOM Builders (`scripts/canvas_pro_dynamic.rs`).

pub const JS_CANVAS_PRO_DYNAMIC: &str = r###"
    function renderProDynamicWidget(node) {
        if (!node) return null;
        if (node.id) knownNodesMap.set(String(node.id), node);
        const type = node.type || node.kind;
        const data = node.data || {};
        let el = document.createElement("div");

        const dynamicBinding = data["data-dynamic"] || (node.attributes && node.attributes["data-dynamic"]);
        if (dynamicBinding) el.setAttribute("data-dynamic", dynamicBinding);

        switch(type) {
            case "SymbolInstance": {
                const symbolId = Number(node.symbol_id ?? data.symbol_id ?? 0);
                const overrides = node.overrides || data.overrides || {};
                node.symbol_id = symbolId;
                node.overrides = overrides;
                node.data = Object.assign(data, { symbol_id: symbolId, overrides: overrides });

                el.className = "dp-symbol-instance-wrapper";
                el.setAttribute("data-symbol-id", String(symbolId));
                el.style.cssText = "border: 2px solid #7c3aed; border-radius: 10px; background: rgba(124, 58, 237, 0.03); padding: 14px; position: relative;";

                const symTpl = symbolsCache[symbolId] || symbolsList.find(function(s) { return Number(s.id) === symbolId; });
                const symTitle = symTpl ? symTpl.title : (data.symbol_title || ("Global Symbol #" + symbolId));
                const ovCount = Object.keys(overrides).length;

                let previewBodyHtml = "";
                if (symTpl && symTpl.layout) {
                    const layoutObj = typeof symTpl.layout === "string" ? JSON.parse(symTpl.layout) : symTpl.layout;
                    if (layoutObj && layoutObj.root) {
                        previewBodyHtml = renderSymbolAstPreviewHtml(layoutObj.root, overrides);
                    }
                }
                if (!previewBodyHtml) {
                    previewBodyHtml = `<div style="padding:18px;text-align:center;background:#ffffff;border:1px dashed #c4b5fd;border-radius:8px;color:#4c1d95;font-size:13px;"><i class="fa-solid fa-cubes" style="margin-right:6px;color:#7c3aed;"></i>Synced Component Instance: <strong>${escHtml(symTitle)}</strong></div>`;
                }

                el.innerHTML = `
                    <div class="dp-symbol-instance-bar" style="display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;padding:5px 10px;background:#7c3aed;color:#ffffff;border-radius:6px;font-size:11.5px;font-weight:600;">
                        <div style="display:flex;align-items:center;gap:6px;">
                            <i class="fa-solid fa-cubes"></i>
                            <span>Synced Symbol: ${escHtml(symTitle)} (#${symbolId})</span>
                            ${ovCount > 0 ? `<span style="background:rgba(255,255,255,0.2);padding:1px 6px;border-radius:999px;font-size:10px;">${ovCount} override${ovCount === 1 ? "" : "s"}</span>` : ""}
                        </div>
                        <div style="display:flex;align-items:center;gap:6px;">
                            <button type="button" class="dp-canvas-detach-symbol" style="background:rgba(255,255,255,0.18);border:none;color:#fff;padding:2px 8px;border-radius:4px;font-size:10.5px;cursor:pointer;">Detach Instance</button>
                            ${symbolId ? `<a href="/d-admin/builder?template_id=${symbolId}" style="color:#fff;text-decoration:none;font-size:10.5px;background:rgba(255,255,255,0.18);padding:2px 8px;border-radius:4px;">Edit Master</a>` : ""}
                        </div>
                    </div>
                    <div class="dp-symbol-instance-body">${previewBodyHtml}</div>
                `;

                const detachCanvasBtn = el.querySelector(".dp-canvas-detach-symbol");
                if (detachCanvasBtn) {
                    detachCanvasBtn.addEventListener("click", function(ev) {
                        ev.stopPropagation();
                        detachSymbolInstance(node.id);
                    });
                }

                if (!symbolsLoaded) {
                    fetchSectionSymbols(false).then(function() {
                        try { if (typeof renderCanvas === "function") renderCanvas(); } catch (_) {}
                    });
                }
                return el;
            }

            case "NavMenu": {
                el.className = "dp-nav-menu-wrapper";
                const items = ensureDefaultNavMenuItems(node);
                const linksHtml = items.map(function(item, idx) {
                    const label = item.label || item.title || "Item";
                    const url = item.url || item.href || "#";
                    const iconHtml = item.icon ? `<i class="${escHtml(item.icon)}" style="margin-right:5px;font-size:12px;"></i>` : "";
                    const badgeHtml = item.badge
                        ? `<span class="dp-nav-badge" style="margin-left:5px;padding:1px 6px;border-radius:999px;background:var(--day-signal,#2563eb);color:#fff;font-size:10px;font-weight:700;">${escHtml(item.badge)}</span>`
                        : "";
                    const descHtml = item.description
                        ? `<span class="dp-nav-desc" style="display:block;font-size:10.5px;color:#64748b;font-weight:400;">${escHtml(item.description)}</span>`
                        : "";
                    const megaId = item.mega_menu_template_id;
                    const megaCaret = megaId
                        ? `<i class="fa-solid fa-chevron-down" style="margin-left:4px;font-size:10px;opacity:0.75;" title="Mega Menu Template #${escHtml(megaId)}"></i>`
                        : "";
                    const megaTpl = megaId
                        ? (symbolsCache[megaId] || megaMenuTemplatesList.find(function(t) { return Number(t.id) === Number(megaId); }))
                        : null;
                    const megaPanelHtml = megaId
                        ? `<div class="dp-mega-menu-panel" data-mega-template-id="${escHtml(megaId)}" style="margin-top:6px;padding:10px 12px;background:#ffffff;border:1px solid #cbd5e1;border-radius:8px;box-shadow:0 8px 24px rgba(15,23,42,0.08);font-size:11.5px;color:#334155;">
                            <div style="font-size:10px;font-weight:700;text-transform:uppercase;color:var(--day-signal,#2563eb);margin-bottom:4px;">
                                <i class="fa-solid fa-cubes"></i> Mega Menu Panel #${escHtml(megaId)}${megaTpl ? ": " + escHtml(megaTpl.title) : ""}
                            </div>
                            ${megaTpl && megaTpl.layout && megaTpl.layout.root ? renderSymbolAstPreviewHtml(megaTpl.layout.root, {}) : ""}
                           </div>`
                        : "";

                    return `
                        <div class="dp-nav-item-wrap" style="display:inline-flex;flex-direction:column;position:relative;">
                            <a href="${escHtml(url)}" class="${idx === 0 ? 'active' : ''}" data-mega-menu-template-id="${escHtml(megaId || '')}">
                                <span>${iconHtml}${escHtml(label)}${badgeHtml}${megaCaret}</span>
                                ${descHtml}
                            </a>
                            ${megaPanelHtml}
                        </div>
                    `;
                }).join("");

                el.innerHTML = `<nav class="dp-nav-menu" style="display:flex;align-items:flex-start;gap:18px;flex-wrap:wrap;">${linksHtml}</nav>`;
                return el;
            }

            case "LoopGrid": {
                el.className = "dp-loop-grid";
                el.style.display = "grid";
                el.style.gridTemplateColumns = `repeat(${data.columns || 3}, 1fr)`;
                el.style.gap = "20px";
                for (let i = 1; i <= (data.posts_per_page || 3); i++) {
                    const postCard = document.createElement("div");
                    postCard.style.cssText = "background:#fff; border:1px solid #e2e8f0; border-radius:8px; overflow:hidden;";
                    postCard.innerHTML = `
                        <div style="height: 140px; background: #e2e8f0; display:flex; align-items:center; justify-content:center; color:#94a3b8;"><i class="fa-solid fa-image fa-2x"></i></div>
                        <div style="padding: 16px;">
                            <span style="font-size: 11px; color: var(--day-signal); font-weight:600;">ARTICLE ${i}</span>
                            <h4 style="margin: 4px 0 8px 0; font-size: 16px;">Fastest CMS Architecture in 2026</h4>
                            <p style="font-size: 12px; color: #64748b; margin: 0;">Learn how Rust replaces legacy PHP monoliths.</p>
                        </div>
                    `;
                    el.appendChild(postCard);
                }
                return el;
            }

            case "Form":
            case "NativeForm": {
                el.className = "dp-form";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 32px; max-width: 600px;";
                const title = data.title || "Get in Touch";
                const btn = data.button_text || "Submit Inquiry";
                el.innerHTML = `
                    <h3 class="dp-form-title" style="margin: 0 0 20px 0; font-size: 20px; font-weight: 700; color: #0f172a;">${escHtml(title)}</h3>
                    <div class="dp-form-fields" style="display: flex; flex-direction: column; gap: 14px;">
                        <div>
                            <label style="display: block; font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 5px;">Your Name</label>
                            <input type="text" class="dp-input" placeholder="Alex Morgan" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        <div>
                            <label style="display: block; font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 5px;">Work Email</label>
                            <input type="email" class="dp-input" placeholder="alex@company.com" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        <div>
                            <label style="display: block; font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 5px;">Message</label>
                            <textarea class="dp-textarea" rows="3" placeholder="Tell us about your project requirements..." disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc; resize: none;"></textarea>
                        </div>
                        <button type="button" class="dp-button dp-form-submit" style="background: var(--day-signal); color: var(--day-ink-contrast); padding: 12px 24px; border: none; border-radius: 6px; font-weight: 600; font-size: 14px; cursor: default; align-self: flex-start; margin-top: 4px;">${escHtml(btn)}</button>
                    </div>
                `;
                return el;
            }

            case "LoginForm": {
                el.className = "dp-login-form";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 32px; max-width: 420px; box-shadow: 0 4px 16px rgba(0,0,0,0.04);";
                const labelUser = data.label_username || "Username or Email";
                const labelPass = data.label_password || "Password";
                const showRemember = data.remember_me !== false;
                el.innerHTML = `
                    <div style="text-align: center; margin-bottom: 24px;">
                        <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(37,99,235,0.1); color: var(--day-signal); display: inline-flex; align-items: center; justify-content: center; font-size: 20px; margin-bottom: 12px;"><i class="fa-solid fa-lock"></i></div>
                        <h3 style="margin: 0; font-size: 20px; font-weight: 700; color: #0f172a;">Sign In</h3>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 14px;">
                        <div class="dp-form-field">
                            <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">${escHtml(labelUser)}</label>
                            <input type="text" class="dp-input" placeholder="admin" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        <div class="dp-form-field">
                            <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">${escHtml(labelPass)}</label>
                            <input type="password" class="dp-input" value="••••••••" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        ${showRemember ? `<label class="dp-login-remember" style="display: flex; align-items: center; gap: 8px; font-size: 13px; color: #64748b; cursor: default;"><input type="checkbox" checked disabled> Remember Me</label>` : ""}
                        <button type="button" class="dp-button dp-login-btn" style="background: var(--day-signal); color: var(--day-ink-contrast); padding: 11px; border: none; border-radius: 6px; font-weight: 600; font-size: 14px; cursor: default; margin-top: 6px;">Log In</button>
                    </div>
                `;
                return el;
            }

            case "SearchForm": {
                el.className = "dp-search-form";
                el.style.cssText = "display: flex; width: 100%; max-width: 500px;";
                const placeholder = data.placeholder || "Search articles, guides, docs...";
                const btn = data.button_text || "Search";
                el.innerHTML = `
                    <div class="dp-search-input-wrap" style="display: flex; width: 100%; position: relative;">
                        <i class="fa-solid fa-magnifying-glass" style="position: absolute; left: 14px; top: 50%; transform: translateY(-50%); color: #94a3b8; font-size: 14px;"></i>
                        <input type="search" class="dp-search-input" placeholder="${escHtml(placeholder)}" disabled style="flex: 1; padding: 11px 16px 11px 40px; border: 1px solid #cbd5e1; border-right: none; border-radius: 6px 0 0 6px; font-size: 14px; background: #ffffff;">
                        <button type="button" class="dp-search-btn" style="background: var(--day-signal); color: var(--day-ink-contrast); padding: 0 20px; border: none; border-radius: 0 6px 6px 0; font-weight: 600; font-size: 13.5px; cursor: default;">${escHtml(btn)}</button>
                    </div>
                `;
                return el;
            }

            case "Breadcrumbs": {
                el.className = "dp-breadcrumbs";
                el.style.cssText = "padding: 10px 0;";
                const home = data.home_text || "Home";
                const sep = data.separator || ">";
                el.innerHTML = `
                    <nav aria-label="Breadcrumbs">
                        <ol class="dp-breadcrumb-list" style="display: flex; align-items: center; gap: 8px; list-style: none; padding: 0; margin: 0; font-size: 13px;">
                            <li class="dp-breadcrumb-item"><a href="#" style="color: var(--day-signal); text-decoration: none; font-weight: 500;"><i class="fa-solid fa-house" style="font-size: 12px; margin-right: 4px;"></i>${escHtml(home)}</a></li>
                            <li class="dp-breadcrumb-separator" style="color: #94a3b8;">${escHtml(sep)}</li>
                            <li class="dp-breadcrumb-item"><a href="#" style="color: var(--day-signal); text-decoration: none; font-weight: 500;">Documentation</a></li>
                            <li class="dp-breadcrumb-separator" style="color: #94a3b8;">${escHtml(sep)}</li>
                            <li class="dp-breadcrumb-item active" style="color: #64748b; font-weight: 500;"><span>DayStudio Architecture</span></li>
                        </ol>
                    </nav>
                `;
                return el;
            }

            case "TableOfContents": {
                el.className = "dp-toc";
                el.style.cssText = "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 20px; max-width: 380px;";
                const title = data.title || "Table of Contents";
                el.innerHTML = `
                    <div class="dp-toc-header" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; border-bottom: 1px solid #e2e8f0; padding-bottom: 10px;">
                        <h4 class="dp-toc-title" style="margin: 0; font-size: 14px; font-weight: 700; color: #0f172a; text-transform: uppercase; letter-spacing: 0.05em;"><i class="fa-solid fa-list-ul" style="margin-right: 6px; color: var(--day-signal);"></i>${escHtml(title)}</h4>
                        <i class="fa-solid fa-chevron-down" style="font-size: 11px; color: #64748b;"></i>
                    </div>
                    <div class="dp-toc-body">
                        <ul class="dp-toc-list" style="list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; font-size: 13px;">
                            <li><a href="#" style="color: var(--day-signal); text-decoration: none;">1. Introduction & Overview</a></li>
                            <li style="padding-left: 14px;"><a href="#" style="color: #475569; text-decoration: none;">1.1 Tokio Async Runtime</a></li>
                            <li style="padding-left: 14px;"><a href="#" style="color: #475569; text-decoration: none;">1.2 SQLite Dual-Write Engine</a></li>
                            <li><a href="#" style="color: var(--day-signal); text-decoration: none;">2. Performance & Benchmarks</a></li>
                            <li><a href="#" style="color: var(--day-signal); text-decoration: none;">3. Theme Builder Engine</a></li>
                        </ul>
                    </div>
                `;
                return el;
            }

            case "MegaMenuContainer": {
                el.className = "dp-megamenu-container";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 28px; width: 100%; box-shadow: 0 10px 30px rgba(0,0,0,0.08);";
                const trigger = data.trigger || "Products Menu";
                el.innerHTML = `
                    <div style="font-size: 11px; text-transform: uppercase; color: var(--day-signal); font-weight: 700; margin-bottom: 16px; letter-spacing: 0.05em;"><i class="fa-solid fa-layer-group" style="margin-right: 6px;"></i>MegaMenu Preview (Trigger: ${escHtml(trigger)})</div>
                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px;">
                        <div><h5 style="margin:0 0 10px 0;font-size:13.5px;font-weight:700;color:#0f172a;">Core Engine</h5><p style="font-size:12px;color:#64748b;line-height:1.5;margin:0 0 10px 0;">Ultra-low latency Rust web server replacing PHP-FPM.</p><a href="#" style="font-size:12px;color:var(--day-signal);font-weight:600;text-decoration:none;">Learn more &rarr;</a></div>
                        <div><h5 style="margin:0 0 10px 0;font-size:13.5px;font-weight:700;color:#0f172a;">DayStudio CAD</h5><p style="font-size:12px;color:#64748b;line-height:1.5;margin:0 0 10px 0;">Webflow-class visual builder with live box-model overlays.</p><a href="#" style="font-size:12px;color:var(--day-signal);font-weight:600;text-decoration:none;">Explore studio &rarr;</a></div>
                        <div><h5 style="margin:0 0 10px 0;font-size:13.5px;font-weight:700;color:#0f172a;">Theme Builder</h5><p style="font-size:12px;color:#64748b;line-height:1.5;margin:0 0 10px 0;">Dynamic header, footer, and template condition engine.</p><a href="#" style="font-size:12px;color:var(--day-signal);font-weight:600;text-decoration:none;">View templates &rarr;</a></div>
                    </div>
                `;
                return el;
            }

            case "PopupModal": {
                el.className = "dp-popup-modal";
                el.style.cssText = "border: 1px dashed var(--day-signal); border-radius: 12px; padding: 24px; background: rgba(37,99,235,0.03);";
                const modalId = data.modal_id || "promo-popup";
                el.innerHTML = `
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <span style="font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--day-signal);"><i class="fa-solid fa-window-restore" style="margin-right: 6px;"></i>Popup Modal [${escHtml(modalId)}]</span>
                        <button type="button" style="border: none; background: #e2e8f0; width: 24px; height: 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; cursor: default; font-size: 12px;">&times;</button>
                    </div>
                    <div style="background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; padding: 24px; text-align: center; box-shadow: 0 4px 12px rgba(0,0,0,0.05);">
                        <h4 style="margin: 0 0 8px 0; font-size: 18px; font-weight: 700; color: #0f172a;">Exclusive Early Access</h4>
                        <p style="margin: 0 0 16px 0; font-size: 13px; color: #64748b;">Join 1,000+ engineers migrating from legacy PHP monoliths.</p>
                        <a href="#" class="dp-button" style="background: var(--day-signal); color: var(--day-ink-contrast); padding: 8px 18px; border-radius: 6px; font-weight: 600; font-size: 13px; text-decoration: none; display: inline-block;">Reserve Your Spot</a>
                    </div>
                `;
                return el;
            }

            default:
                return null;
        }
    }

    function initProDynamicExtensions() {
        fetchSectionSymbols(false).then(function() {
            enhanceLeftDrawerPalette();
            renderSymbolsDrawerContent();
            enhanceInspectorPanel();
        });

        const inspector = document.getElementById("daystudio-inspector");
        if (inspector) {
            inspector.addEventListener("focusin", function(e) {
                const t = e.target;
                if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA") && !t.classList.contains("dp-ignore-dynamic-target")) {
                    lastFocusedInspectorInput = t;
                }
            });
            const inspectorObserver = new MutationObserver(function() {
                if (!isEnhancingInspector) enhanceInspectorPanel();
            });
            inspectorObserver.observe(inspector, { childList: true });
        }

        const drawerPanel = document.getElementById("drawer-panel-container");
        if (drawerPanel) {
            const drawerObserver = new MutationObserver(function() {
                if (!isEnhancingDrawer) enhanceLeftDrawerPalette();
            });
            drawerObserver.observe(drawerPanel, { childList: true });
        }

        const symbolsRailBtn = document.getElementById("rail-btn-symbols");
        if (symbolsRailBtn) {
            symbolsRailBtn.addEventListener("click", async function() {
                document.querySelectorAll("#daystudio-rail .rail-tab-btn").forEach(function(b) {
                    b.classList.toggle("active", b === symbolsRailBtn);
                });
                const drawer = document.getElementById("daystudio-drawer");
                if (drawer) drawer.classList.add("drawer-open");
                await fetchSectionSymbols(false);
                renderSymbolsDrawerContent();
            });
        }

        const openSymbolsPromptBtn = document.getElementById("btn-open-symbols-prompt");
        if (openSymbolsPromptBtn) {
            openSymbolsPromptBtn.addEventListener("click", function() {
                if (symbolsRailBtn) symbolsRailBtn.click();
                else insertSymbolInstanceOntoCanvas(symbolsList[0] ? symbolsList[0].id : 0);
            });
        }

        const saveSymbolBadgeBtn = document.getElementById("act-save-symbol-node");
        if (saveSymbolBadgeBtn) {
            saveSymbolBadgeBtn.addEventListener("click", function(e) {
                e.preventDefault();
                e.stopPropagation();
                saveSelectedSectionAsGlobalSymbol();
            });
        }

        enhanceLeftDrawerPalette();
        enhanceInspectorPanel();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", initProDynamicExtensions);
    } else {
        setTimeout(initProDynamicExtensions, 0);
    }

    window.renderProDynamicWidget = renderProDynamicWidget;
    window.saveSelectedSectionAsGlobalSymbol = saveSelectedSectionAsGlobalSymbol;
    window.insertSymbolInstanceOntoCanvas = insertSymbolInstanceOntoCanvas;
    window.detachSymbolInstance = detachSymbolInstance;
    window.applyDynamicTokenToSelection = applyDynamicTokenToSelection;
})();
"###;
