//! DayStudio 0.5 Hook Bootstrapper for Synced Global Symbols (`SymbolInstance`),
//! Mega Menus (`NavMenu`), and Dynamic Field Binding (`data-dynamic`) (`scripts/canvas_pro_dynamic.rs`).

pub const JS_CANVAS_PRO_DYNAMIC: &str = r###"
(function() {
    function renderProDynamicWidget(node) {
        const type = node.type;
        const d = node.data || {};
        const el = document.createElement("div");

        switch (type) {
            case "NavMenu":
                return window.renderProNavMenuWidget ? window.renderProNavMenuWidget(node) : el;

            case "MegaMenuContainer": {
                el.className = "dp-pro-block dp-megamenu-container";
                el.style.cssText = "background:#0f172a;border:1px solid rgba(148,163,184,0.16);border-radius:12px;padding:24px;width:100%;box-sizing:border-box;";
                if (Array.isArray(node.children) && node.children.length > 0 && window.buildDomNode) {
                    node.children.forEach(child => {
                        const childDom = window.buildDomNode(child);
                        if (childDom) el.appendChild(childDom);
                    });
                } else {
                    el.innerHTML = `<div style="padding:24px;border:1px dashed var(--day-rule,#393836);border-radius:8px;text-align:center;color:var(--day-muted,#A7A297);font-size:12px;">
                        <i class="fa-solid fa-table-columns" style="color:var(--day-signal,#93A1FF);margin-right:6px;"></i>
                        Empty Mega Menu Section — Drag Rows, Columns, Grids, or Widgets here from the Left (+) Drawer.
                    </div>`;
                }
                return el;
            }

            case "SymbolInstance": {
                el.className = "dp-pro-block dp-symbol-instance";
                el.style.cssText = "position:relative;border:1.5px solid var(--day-violet,#C2C9FF);border-radius:10px;padding:12px;";
                const symId = d.symbol_id || 0;
                const symTitle = d.label || (symId ? `Symbol #${symId}` : "Unlinked Symbol");
                const badge = document.createElement("div");
                badge.style.cssText = "display:inline-flex;align-items:center;gap:6px;background:var(--day-surface-elevated,#23262A);border:1px solid var(--day-rule,#393836);color:var(--day-violet,#C2C9FF);font-size:10px;font-weight:700;padding:3px 9px;border-radius:999px;margin-bottom:8px;letter-spacing:0.04em;text-transform:uppercase;";
                badge.innerHTML = `<i class="fa-solid fa-clone"></i> Synced Symbol: ${symTitle}`;
                el.appendChild(badge);

                const bodyWrap = document.createElement("div");
                bodyWrap.className = "dp-symbol-instance-body";
                if (Array.isArray(node.children) && node.children.length > 0 && window.buildDomNode) {
                    const overrides = d.overrides || {};
                    node.children.forEach(child => {
                        const cloned = JSON.parse(JSON.stringify(child));
                        if (window.applyClientSymbolOverrides) window.applyClientSymbolOverrides(cloned, overrides);
                        const dom = window.buildDomNode(cloned);
                        if (dom) bodyWrap.appendChild(dom);
                    });
                } else {
                    bodyWrap.innerHTML = `<div style="padding:14px;color:var(--day-muted,#A7A297);font-size:11.5px;text-align:center;">Select a Global Symbol in the Right Inspector to render its synced layout.</div>`;
                }
                el.appendChild(bodyWrap);
                return el;
            }

            case "PopupModal": {
                el.className = "dp-pro-block dp-popup-modal";
                el.style.cssText = "background:#111827;border:2px dashed #3b82f6;border-radius:12px;padding:24px;";
                el.innerHTML = `<div style="font-size:11px;color:#60a5fa;font-weight:700;text-transform:uppercase;margin-bottom:8px;"><i class="fa-solid fa-window-restore"></i> Popup Modal (#${d.modal_id || "promo-popup"})</div>
                    <div style="color:#94a3b8;font-size:13px;">Trigger: <code>${d.trigger_selector || ".open-popup"}</code></div>`;
                return el;
            }

            case "LoopGrid": {
                const cols = d.columns || 3;
                const count = Math.min(d.posts_per_page || 3, 6);
                el.className = "dp-pro-block dp-loop-grid-preview";
                let toolbarHtml = "";
                if (d.enable_filter_bar || d.enable_sort || d.enable_search) {
                    toolbarHtml = `<div style="display:flex;justify-content:space-between;align-items:center;gap:10px;margin-bottom:14px;padding-bottom:10px;border-bottom:1px solid #e2e8f0;flex-wrap:wrap;">
                        <div style="display:flex;gap:6px;">
                            ${d.enable_filter_bar ? `<span style="background:#0f172a;color:#fff;font-size:11px;font-weight:600;padding:4px 10px;border-radius:3px;">All</span><span style="background:#f8fafc;border:1px solid #cbd5e1;color:#334155;font-size:11px;padding:4px 10px;border-radius:3px;">Category A</span><span style="background:#f8fafc;border:1px solid #cbd5e1;color:#334155;font-size:11px;padding:4px 10px;border-radius:3px;">Category B</span>` : ""}
                        </div>
                        <div style="display:flex;gap:8px;align-items:center;">
                            ${d.enable_search ? `<span style="background:#f8fafc;border:1px solid #cbd5e1;color:#64748b;font-size:11px;padding:4px 10px;border-radius:3px;">Search...</span>` : ""}
                            ${d.enable_sort ? `<span style="background:#f8fafc;border:1px solid #cbd5e1;color:#334155;font-size:11px;padding:4px 10px;border-radius:3px;">Sort: ${d.default_sort || "Newest"} ▾</span>` : ""}
                        </div>
                    </div>`;
                }
                const chips = Array.isArray(d.meta_fields) && d.meta_fields.length > 0
                    ? `<div style="display:flex;gap:4px;flex-wrap:wrap;margin:6px 0;">${d.meta_fields.map(k => `<span style="font-size:10px;font-family:monospace;background:#f1f5f9;border:1px solid #cbd5e1;color:#334155;padding:1px 6px;border-radius:3px;">{{acf.${k}}}</span>`).join("")}</div>`
                    : "";
                let cards = "";
                for (let i = 1; i <= count; i++) {
                    cards += `<div style="background:#ffffff;border:1px solid #e2e8f0;border-radius:8px;overflow:hidden;">
                        ${d.show_thumbnail !== false && d.card_skin !== "minimal" ? `<div style="height:110px;background:linear-gradient(135deg,#1e293b,#334155);display:flex;align-items:center;justify-content:center;color:#64748b;font-size:20px;"><i class="fa-regular fa-image"></i></div>` : ""}
                        <div style="padding:14px;">
                            ${d.show_meta !== false ? `<div style="font-size:11px;color:#2563eb;font-weight:600;margin-bottom:4px;">${(d.post_type || "post").toUpperCase()} &bull; ${(d.card_skin || "editorial").toUpperCase()}</div>` : ""}
                            <div style="font-size:14px;font-weight:700;color:#0f172a;margin-bottom:6px;">Dynamic ${(d.post_type || "post")} #${i}</div>
                            ${chips}
                            ${d.show_excerpt !== false ? `<div style="font-size:12px;color:#64748b;line-height:1.4;">Queried live from SQLite with filter &amp; sort controls...</div>` : ""}
                        </div>
                    </div>`;
                }
                el.innerHTML = `${toolbarHtml}<div style="display:grid;grid-template-columns:repeat(${cols},1fr);gap:16px;">${cards}</div>`;
                return el;
            }

            case "TableOfContents": {
                el.className = "dp-pro-block dp-toc";
                el.style.cssText = "background:#f8fafc;border:1px solid #e2e8f0;border-radius:10px;padding:18px;color:#0f172a;";
                el.innerHTML = `<div style="font-weight:700;font-size:14px;margin-bottom:10px;"><i class="fa-solid fa-list-ul" style="color:#2563eb;margin-right:6px;"></i>${d.title || "Table of Contents"}</div>
                    <ul style="margin:0;padding-left:18px;font-size:13px;color:#475569;line-height:1.8;">
                        <li>1. Overview &amp; Architecture</li>
                        <li>2. Key Benchmarks (H${d.min_depth || 2}–H${d.max_depth || 4})</li>
                        <li>3. Deployment Guide</li>
                    </ul>`;
                return el;
            }

            case "Breadcrumbs": {
                const sep = d.separator || "/";
                el.className = "dp-pro-block dp-breadcrumbs";
                el.style.cssText = "font-size:13px;color:#64748b;display:flex;gap:8px;align-items:center;";
                el.innerHTML = `<a href="#" style="color:#2563eb;text-decoration:none;">${d.home_text || "Home"}</a> <span>${sep}</span> <span>Category</span> <span>${sep}</span> <strong style="color:inherit;">Current Page</strong>`;
                return el;
            }

            case "LanguageSwitcher": {
                const mode = d.display_style || d.display_mode || "pills";
                const showFlags = d.show_flags !== false;
                const showNames = d.show_names !== false;
                const showCodes = !!d.show_codes;
                const fmt = (flag, name, code) => {
                    const p = [];
                    if (showFlags) p.push(flag);
                    if (showNames) p.push(name);
                    if (showCodes || (!showFlags && !showNames)) p.push(code.toUpperCase());
                    return p.join(" ");
                };
                el.className = "dp-pro-block dp-language-switcher";
                if (mode === "dropdown") {
                    el.innerHTML = `<select style="padding:6px 12px;border-radius:6px;border:1px solid #cbd5e1;font-size:13px;font-weight:600;">
                        <option selected>${fmt("🇺🇸", "English", "en")}</option>
                        <option>${fmt("🇩🇪", "Deutsch", "de")}</option>
                        <option>${fmt("🇳🇱", "Nederlands", "nl")}</option>
                    </select>`;
                } else {
                    el.style.cssText = "display:inline-flex;align-items:center;gap:6px;flex-wrap:wrap;";
                    el.innerHTML = `<span style="display:inline-flex;align-items:center;gap:5px;padding:5px 10px;border-radius:6px;font-size:13px;font-weight:700;border:1px solid #3b82f6;background:rgba(59,130,246,0.12);">${fmt("🇺🇸", "English", "en")}</span>
                        <span style="display:inline-flex;align-items:center;gap:5px;padding:5px 10px;border-radius:6px;font-size:13px;font-weight:500;border:1px solid rgba(148,163,184,0.35);opacity:0.85;">${fmt("🇩🇪", "Deutsch", "de")}</span>
                        <span style="display:inline-flex;align-items:center;gap:5px;padding:5px 10px;border-radius:6px;font-size:13px;font-weight:500;border:1px solid rgba(148,163,184,0.35);opacity:0.85;">${fmt("🇳🇱", "Nederlands", "nl")}</span>`;
                }
                return el;
            }

            case "SearchForm": {
                el.className = "dp-pro-block dp-search-form";
                el.style.cssText = "display:flex;gap:8px;";
                el.innerHTML = `<input type="search" placeholder="${d.placeholder || "Search..."}" style="flex:1;padding:9px 12px;border:1px solid #cbd5e1;border-radius:6px;font-size:13px;" readonly>
                    <button type="button" style="background:#2563eb;color:#fff;border:none;border-radius:6px;padding:9px 16px;font-weight:600;font-size:13px;">${d.button_text || "Search"}</button>`;
                return el;
            }

            case "LoginForm": {
                el.className = "dp-pro-block dp-login-form";
                el.style.cssText = "background:#ffffff;border:1px solid #e2e8f0;border-radius:10px;padding:20px;color:#0f172a;max-width:360px;";
                el.innerHTML = `<div style="margin-bottom:12px;"><label style="display:block;font-size:12px;font-weight:600;margin-bottom:4px;">${d.label_username || "Username or Email"}</label><input type="text" style="width:100%;padding:8px;border:1px solid #cbd5e1;border-radius:6px;" readonly></div>
                    <div style="margin-bottom:12px;"><label style="display:block;font-size:12px;font-weight:600;margin-bottom:4px;">${d.label_password || "Password"}</label><input type="password" style="width:100%;padding:8px;border:1px solid #cbd5e1;border-radius:6px;" readonly></div>
                    <button type="button" style="width:100%;background:#2563eb;color:#fff;border:none;border-radius:6px;padding:10px;font-weight:600;">Log In</button>`;
                return el;
            }

            case "NativeForm":
            case "Form": {
                const title = d.title || "Contact Us";
                const btnText = d.button_text || "Send Message";
                const formId = d.form_id || 1;
                el.className = "dp-pro-block dp-form";
                el.style.cssText = "background:#ffffff;border:1px solid #e2e8f0;border-radius:12px;padding:24px;color:#0f172a;";
                el.innerHTML = `<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
                        <h4 style="margin:0;font-size:16px;font-weight:700;">${title}</h4>
                        <span style="font-size:11px;background:#eff6ff;color:#2563eb;padding:2px 8px;border-radius:999px;font-weight:600;">Form #${formId}</span>
                    </div>
                    <div style="display:grid;gap:10px;">
                        <input type="text" placeholder="Your Name" style="width:100%;padding:9px 12px;border:1px solid #cbd5e1;border-radius:6px;font-size:13px;" readonly>
                        <input type="email" placeholder="you@example.com" style="width:100%;padding:9px 12px;border:1px solid #cbd5e1;border-radius:6px;font-size:13px;" readonly>
                        <textarea rows="3" placeholder="Message..." style="width:100%;padding:9px 12px;border:1px solid #cbd5e1;border-radius:6px;font-size:13px;" readonly></textarea>
                        <button type="button" style="background:#2563eb;color:#fff;border:none;border-radius:6px;padding:10px 18px;font-weight:600;width:fit-content;">${btnText}</button>
                    </div>`;
                return el;
            }
        }
        return null;
    }
    window.renderProDynamicWidget = renderProDynamicWidget;

    document.addEventListener("DOMContentLoaded", () => {
        setTimeout(() => {
            if (window.renderInspector && !window._dpInspectorHooked) {
                window._dpInspectorHooked = true;
                const origRenderInspector = window.renderInspector;
                window.renderInspector = function() {
                    origRenderInspector.apply(this, arguments);
                    const state = window.DayStudioState;
                    const activeId = state ? state.activeElementId : null;
                    const node = activeId && window.findNodeById ? window.findNodeById(activeId) : null;
                    if (node && window.renderProInspectorExtensions) {
                        window.renderProInspectorExtensions(node);
                        const ext = document.getElementById("dp-pro-inspector-extensions");
                        if (ext && window.injectAcfOptionsIntoDynamicSelect) {
                            window.injectAcfOptionsIntoDynamicSelect(ext, node);
                        }
                        if (ext && node.type === "LoopGrid" && window.renderLoopGridCollectionInspector) {
                            window.renderLoopGridCollectionInspector(ext, node);
                        }
                    }
                };
            }

            if (window.renderActiveDrawer && !window._dpDrawerHooked) {
                window._dpDrawerHooked = true;
                const origRenderActiveDrawer = window.renderActiveDrawer;
                window.renderActiveDrawer = function(drawerId, container) {
                    origRenderActiveDrawer.apply(this, arguments);
                    if (drawerId === "elements" && container && window.renderSymbolsLibraryIntoDrawer) {
                        window.renderSymbolsLibraryIntoDrawer(container);
                    }
                };
                const panelContainer = document.getElementById("drawer-panel-container");
                if (panelContainer && window.DayStudioState && window.DayStudioState.activeDrawer === "elements" && window.renderSymbolsLibraryIntoDrawer) {
                    window.renderSymbolsLibraryIntoDrawer(panelContainer);
                }
            }

            if (window.getWidgetStarterDefaults && !window._dpStarterDefaultsHooked) {
                window._dpStarterDefaultsHooked = true;
                const origGetDefaults = window.getWidgetStarterDefaults;
                window.getWidgetStarterDefaults = function(type) {
                    if (type === "SymbolInstance") {
                        return { symbol_id: 0, label: "Global Symbol Instance", overrides: {} };
                    }
                    return origGetDefaults ? origGetDefaults.apply(this, arguments) : {};
                };
            }
        }, 40);
    });
})();
"###;
