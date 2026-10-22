//! Inspector sidebar Content tab controls for all widget & container NodeKinds.

pub const SCRIPT_INSPECTOR_CONTENT: &str = r###"    // -------------------------------------------------------------
    // PANEL INSPECTOR CONTROLS
    // -------------------------------------------------------------
        function renderInspector() {
        const titleEl = document.getElementById("elementor-panel-header-title");
        const navEl = document.getElementById("panel-nav");
        const bodyEl = document.getElementById("elementor-panel-content-wrapper");
        const menuBtn = document.getElementById("elementor-panel-header-menu-button");
        const addBtn = document.getElementById("elementor-panel-header-add-button");

        // Ensure global styles are applied to canvas
        applyGlobalStyles();

        // -------------------------------------------------------------
        // VIEW 1: HAMBURGER SITE SETTINGS & MENU ROOT
        // -------------------------------------------------------------
        if (currentPanelView === "menu") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Editor";
            titleEl.innerHTML = `<span style="display:flex;align-items:center;gap:6px;"><i class="fa-solid fa-sliders" style="color:var(--e-color-accent);"></i> SITE SETTINGS</span>`;
            navEl.innerHTML = `<div class="elementor-subpanel-header" style="width:100%;"><span style="font-size:11px;font-weight:700;color:#818a91;text-transform:uppercase;">Global Site Configuration</span></div>`;

            bodyEl.innerHTML = `
                <div class="elementor-panel-menu" id="elementor-panel-page-menu">
                    <!-- Group: SITE SETTINGS -->
                    <div class="elementor-panel-menu-group">
                        <div class="elementor-panel-menu-group-title">Site Settings</div>
                        <div class="elementor-panel-menu-items">
                            <div class="elementor-panel-menu-item" id="menu-item-colors">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-palette"></i></div>
                                <div class="elementor-panel-menu-item-title">Global Colors</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-fonts">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-font"></i></div>
                                <div class="elementor-panel-menu-item-title">Global Fonts</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-buttons">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-square-check"></i></div>
                                <div class="elementor-panel-menu-item-title">Buttons Style</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-identity">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-id-card"></i></div>
                                <div class="elementor-panel-menu-item-title">Site Identity</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-layout">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-table-columns"></i></div>
                                <div class="elementor-panel-menu-item-title">Layout & Spacing</div>
                            </div>
                        </div>
                    </div>

                    <!-- Group: PREFERENCES -->
                    <div class="elementor-panel-menu-group">
                        <div class="elementor-panel-menu-group-title">Preferences & Tools</div>
                        <div class="elementor-panel-menu-items">
                            <div class="elementor-panel-menu-item" id="menu-item-user-pref">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-sliders"></i></div>
                                <div class="elementor-panel-menu-item-title">User Preferences</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-shortcuts">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-keyboard"></i></div>
                                <div class="elementor-panel-menu-item-title">Keyboard Shortcuts</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-finder">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-magnifying-glass"></i></div>
                                <div class="elementor-panel-menu-item-title">Finder (Ctrl+E)</div>
                            </div>
                        </div>
                    </div>

                    <!-- Group: NAVIGATE FROM PAGE -->
                    <div class="elementor-panel-menu-group">
                        <div class="elementor-panel-menu-group-title">Navigate From Page</div>
                        <div class="elementor-panel-menu-items">
                            <a class="elementor-panel-menu-item" href="/?p=${POST_ID}" target="_blank">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-arrow-up-right-from-square"></i></div>
                                <div class="elementor-panel-menu-item-title">View Live Page</div>
                            </a>
                            <a class="elementor-panel-menu-item" href="/n-admin/post.php?action=edit&post=${POST_ID}">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-pen-nib"></i></div>
                                <div class="elementor-panel-menu-item-title">Exit to Post Editor</div>
                            </a>
                            <a class="elementor-panel-menu-item" href="/n-admin/">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-arrow-right-from-bracket"></i></div>
                                <div class="elementor-panel-menu-item-title">Exit to Dashboard</div>
                            </a>
                        </div>
                    </div>
                </div>
            `;

            document.getElementById("menu-item-colors").onclick = () => { currentPanelView = "site-settings-colors"; renderInspector(); };
            document.getElementById("menu-item-fonts").onclick = () => { currentPanelView = "site-settings-fonts"; renderInspector(); };
            document.getElementById("menu-item-buttons").onclick = () => { currentPanelView = "site-settings-buttons"; renderInspector(); };
            document.getElementById("menu-item-identity").onclick = () => { currentPanelView = "site-settings-identity"; renderInspector(); };
            document.getElementById("menu-item-layout").onclick = () => { currentPanelView = "site-settings-layout"; renderInspector(); };
            document.getElementById("menu-item-user-pref").onclick = () => { currentPanelView = "user-preferences"; renderInspector(); };
            document.getElementById("menu-item-shortcuts").onclick = () => {
                openShortcutsModal();
            };
            document.getElementById("menu-item-finder").onclick = () => {
                openFinderModal();
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 2: SUBPANEL - GLOBAL COLORS
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-colors") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `GLOBAL COLORS`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            const colors = pageAst.global_styles.system_colors || [];
            const customColors = pageAst.global_styles.custom_colors || [];

            let colorsHtml = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">System Colors</div>
            `;

            colors.forEach(c => {
                colorsHtml += `
                    <div class="elementor-color-token-row">
                        <div class="color-token-meta">
                            <div class="color-token-preview" style="background:${c.color};" id="swatch-${c.id}"></div>
                            <div class="color-token-info">
                                <span class="color-token-title">${c.title}</span>
                                <span class="color-token-code" id="hex-${c.id}">${c.color}</span>
                            </div>
                        </div>
                        <div class="color-token-input-wrapper">
                            <input type="color" value="${c.color}" data-color-id="${c.id}" class="system-color-input">
                        </div>
                    </div>
                `;
            });

            colorsHtml += `
                <div class="globals-section-header" style="margin-top:16px;">
                    <span>Custom Colors</span>
                    <button class="elementor-back-btn" id="btn-add-custom-color" style="font-size:11px;color:var(--e-color-accent);">
                        <i class="fa-solid fa-plus"></i> Add Color
                    </button>
                </div>
            `;

            if (customColors.length === 0) {
                colorsHtml += `<div style="font-size:11px;color:#818a91;font-style:italic;">No custom colors defined yet. Click '+ Add Color' to create one.</div>`;
            } else {
                customColors.forEach((cc, idx) => {
                    colorsHtml += `
                        <div class="elementor-color-token-row">
                            <div class="color-token-meta">
                                <div class="color-token-preview" style="background:${cc.color};"></div>
                                <div class="color-token-info">
                                    <span class="color-token-title">${cc.title}</span>
                                    <span class="color-token-code">${cc.color}</span>
                                </div>
                            </div>
                            <div style="display:flex;align-items:center;gap:6px;">
                                <input type="color" value="${cc.color}" data-custom-idx="${idx}" class="custom-color-input">
                                <button class="elementor-back-btn btn-del-color" data-del-idx="${idx}" style="color:#ef4444;"><i class="fa-solid fa-trash-can"></i></button>
                            </div>
                        </div>
                    `;
                });
            }

            colorsHtml += `
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-colors">
                            <i class="fa-solid fa-floppy-disk"></i> Apply & Save Palette
                        </button>
                    </div>
                </div>
            `;

            bodyEl.innerHTML = colorsHtml;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };


            bodyEl.querySelectorAll(".system-color-input").forEach(input => {
                input.oninput = (e) => {
                    window._userModifiedGlobalStyles = true;
                    const cid = e.target.dataset.colorId;
                    const val = e.target.value;
                    const c = colors.find(x => x.id === cid);
                    if (c) c.color = val;
                    const swatch = document.getElementById("swatch-" + cid);
                    if (swatch) swatch.style.background = val;
                    const hex = document.getElementById("hex-" + cid);
                    if (hex) hex.textContent = val;
                    applyGlobalStyles();
                };
            });

            bodyEl.querySelectorAll(".custom-color-input").forEach(input => {
                input.oninput = (e) => {
                    window._userModifiedGlobalStyles = true;
                    const idx = parseInt(e.target.dataset.customIdx);
                    if (customColors[idx]) {
                        customColors[idx].color = e.target.value;
                        applyGlobalStyles();
                    }
                };
            });

            bodyEl.querySelectorAll(".btn-del-color").forEach(btn => {
                btn.onclick = () => {
                    window._userModifiedGlobalStyles = true;
                    const idx = parseInt(btn.dataset.delIdx);
                    customColors.splice(idx, 1);
                    applyGlobalStyles();
                    renderInspector();
                };
            });

            const addCustomBtn = document.getElementById("btn-add-custom-color");
            if (addCustomBtn) {
                addCustomBtn.onclick = () => {
                    const name = prompt("Enter Custom Color Title:", "Brand Highlight");
                    if (name) {
                        window._userModifiedGlobalStyles = true;
                        const newId = "custom_" + Math.random().toString(36).substr(2, 6);
                        customColors.push({ id: newId, title: name, color: "#8b5cf6" });
                        applyGlobalStyles();
                        renderInspector();
                    }
                };
            }

            document.getElementById("btn-save-colors").onclick = () => {
                window._userModifiedGlobalStyles = true;
                applyGlobalStyles();
                savePage();
                showToast("Global Colors updated & saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 3: SUBPANEL - GLOBAL FONTS
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-fonts") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `GLOBAL FONTS`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            const typography = pageAst.global_styles.system_typography || [];
            const fontFamilies = ["Fira Sans, sans-serif", "Roboto, sans-serif", "Inter, sans-serif", "Poppins, sans-serif", "Montserrat, sans-serif", "Roboto Slab, serif", "Playfair Display, serif", "system-ui, sans-serif"];

            let fontsHtml = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">System Typography Tokens</div>
            `;

            typography.forEach(t => {
                let optionsHtml = "";
                fontFamilies.forEach(f => {
                    const sel = t.font_family.includes(f.split(",")[0]) ? "selected" : "";
                    optionsHtml += `<option value="${f}" ${sel}>${f.split(",")[0]}</option>`;
                });

                fontsHtml += `
                    <div class="elementor-control-row" style="background:#191b1d;padding:12px;border:1px solid var(--e-border);border-radius:5px;">
                        <div class="elementor-control-label">
                            <span>${t.title} Font</span>
                            <span style="font-size:10px;color:#818a91;">${t.font_size || '16px'} (${t.font_weight || '400'})</span>
                        </div>
                        <select class="elementor-select font-family-select" data-type-id="${t.id}">
                            ${optionsHtml}
                        </select>
                    </div>
                `;
            });

            fontsHtml += `
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-fonts">
                            <i class="fa-solid fa-floppy-disk"></i> Apply & Save Typography
                        </button>
                    </div>
                </div>
            `;

            bodyEl.innerHTML = fontsHtml;
            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };


            bodyEl.querySelectorAll(".font-family-select").forEach(sel => {
                sel.onchange = (e) => {
                    const tid = e.target.dataset.typeId;
                    const item = typography.find(x => x.id === tid);
                    if (item) {
                        item.font_family = e.target.value;
                        applyGlobalStyles();
                    }
                };
            });

            document.getElementById("btn-save-fonts").onclick = () => {
                applyGlobalStyles();
                savePage();
                showToast("Global Fonts updated & saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 4: SUBPANEL - BUTTONS STYLE
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-buttons") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `BUTTONS STYLE`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Default Button Appearance</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Background Color</label>
                        <div class="color-picker-control">
                            <input type="color" class="color-swatch" id="btn-def-bg" value="#e02660">
                            <input type="text" class="elementor-input" value="#e02660" readonly>
                        </div>
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Text Color</label>
                        <div class="color-picker-control">
                            <input type="color" class="color-swatch" id="btn-def-txt" value="#ffffff">
                            <input type="text" class="elementor-input" value="#ffffff" readonly>
                        </div>
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Border Radius (px)</label>
                        <input type="number" class="elementor-input" id="btn-def-rad" value="6" min="0" max="50">
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-button-style">
                            <i class="fa-solid fa-floppy-disk"></i> Save Button Presets
                        </button>
                    </div>
                </div>
            `;

            const bDef = pageAst.global_styles.button_style || { background_color: "#e02660", text_color: "#ffffff", border_radius: "6px" };
            const bgIn = document.getElementById("btn-def-bg");
            const txtIn = document.getElementById("btn-def-txt");
            const radIn = document.getElementById("btn-def-rad");
            if (bgIn && bDef.background_color) bgIn.value = bDef.background_color;
            if (txtIn && bDef.text_color) txtIn.value = bDef.text_color;
            if (radIn && bDef.border_radius) radIn.value = parseInt(bDef.border_radius) || 6;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            document.getElementById("btn-save-button-style").onclick = () => {
                pageAst.global_styles.button_style = {
                    background_color: bgIn ? bgIn.value : "#e02660",
                    text_color: txtIn ? txtIn.value : "#ffffff",
                    border_radius: (radIn ? radIn.value : "6") + "px"
                };
                applyGlobalStyles();
                savePage();
                showToast("Global Button styling updated & saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 5: SUBPANEL - LAYOUT & SPACING
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-layout") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `LAYOUT & SPACING`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            const cw = pageAst.global_styles.content_width || 1140;
            const ws = pageAst.global_styles.widgets_space || 20;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Container Geometry</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Content Width (px)</label>
                        <input type="number" class="elementor-input" id="layout-content-width" value="${cw}" min="600" max="1920">
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Widgets Space / Gap (px)</label>
                        <input type="number" class="elementor-input" id="layout-widgets-space" value="${ws}" min="0" max="60">
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-layout">
                            <i class="fa-solid fa-floppy-disk"></i> Save Layout Defaults
                        </button>
                    </div>
                </div>
            `;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            document.getElementById("layout-content-width").oninput = (e) => {
                pageAst.global_styles.content_width = parseInt(e.target.value) || 1140;
                applyGlobalStyles();
            };
            document.getElementById("layout-widgets-space").oninput = (e) => {
                pageAst.global_styles.widgets_space = parseInt(e.target.value) || 20;
                applyGlobalStyles();
            };
            document.getElementById("btn-save-layout").onclick = () => {
                applyGlobalStyles();
                savePage();
                showToast("Layout geometry saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 6: SUBPANEL - SITE IDENTITY
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-identity") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `SITE IDENTITY`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Identity Settings</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Site Title</label>
                        <input type="text" class="elementor-input" id="identity-title" value="Neopress High-Throughput Engine">
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Site Tagline</label>
                        <input type="text" class="elementor-input" id="identity-tagline" value="Native Rust &bull; Tokio Async &bull; Zero PHP Overhead">
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-identity">
                            <i class="fa-solid fa-floppy-disk"></i> Update Site Identity
                        </button>
                    </div>
                </div>
            `;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            fetch("/wp-json/wp/v2/settings", { headers: { "X-WP-Nonce": "wp_rest_admin" }, credentials: "include" }).then(r => r.json()).then(s => {
                const titleIn = document.getElementById("identity-title");
                const tagIn = document.getElementById("identity-tagline");
                if (titleIn && s.title) titleIn.value = s.title;
                if (tagIn && s.description) tagIn.value = s.description;
            }).catch(() => {});
            document.getElementById("btn-save-identity").onclick = async () => {
                const t = document.getElementById("identity-title").value;
                const d = document.getElementById("identity-tagline").value;
                try {
                    const res = await fetch("/wp-json/wp/v2/settings", {
                        method: "POST",
                        headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                        credentials: "include",
                        body: JSON.stringify({ title: t, description: d })
                    });
                    if (res.ok) {
                        document.title = "Elementor Visual Builder – " + t;
                        showToast("Site Identity updated in SQLite!");
                    } else {
                        showToast("Failed to update site settings");
                    }
                } catch(e) {
                    showToast("Site identity updated locally");
                }
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 7: SUBPANEL - USER PREFERENCES
        // -------------------------------------------------------------
        if (currentPanelView === "user-preferences") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `USER PREFERENCES`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Editor Studio Interface</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">UI Theme</label>
                        <select class="elementor-select" id="pref-theme">
                            <option value="dark" selected>Dark Studio (Default)</option>
                            <option value="light">Light Studio</option>
                        </select>
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Panel Width</label>
                        <select class="elementor-select" id="pref-panel-width">
                            <option value="300" selected>300px (Default)</option>
                            <option value="360">360px (Wide)</option>
                            <option value="420">420px (Ultra-wide)</option>
                        </select>
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-pref">
                            <i class="fa-solid fa-check"></i> Save Preferences
                        </button>
                    </div>
                </div>
            `;

            const wrapper = document.getElementById("elementor-editor-wrapper");
            const panel = document.getElementById("elementor-panel");
            const themeSel = document.getElementById("pref-theme");
            const widthSel = document.getElementById("pref-panel-width");

            const savedTheme = localStorage.getItem("neopress_studio_theme") || "dark";
            const savedWidth = localStorage.getItem("neopress_panel_width") || "300";
            if (themeSel) themeSel.value = savedTheme;
            if (widthSel) widthSel.value = savedWidth;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            if (themeSel) {
                themeSel.onchange = (e) => {
                    const th = e.target.value;
                    if (th === "light") wrapper.classList.add("theme-light");
                    else wrapper.classList.remove("theme-light");
                    localStorage.setItem("neopress_studio_theme", th);
                };
            }
            if (widthSel) {
                widthSel.onchange = (e) => {
                    panel.style.width = e.target.value + "px";
                    localStorage.setItem("neopress_panel_width", e.target.value);
                };
            }
            document.getElementById("btn-save-pref").onclick = () => {
                showToast("User Preferences saved to local storage!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 8: GLOBALS TAB DRAWER (currentPanelView === "globals")
        // -------------------------------------------------------------
        if (currentPanelView === "globals" && !activeElementId) {
            menuBtn.innerHTML = `<i class="fa-solid fa-bars"></i>`;
            menuBtn.title = "Menu";
            titleEl.innerHTML = `<span class="e-logo">E</span> ELEMENTOR`;
            navEl.innerHTML = `
                <button class="elementor-panel-navigation-tab" data-tab="elements">Elements</button>
                <button class="elementor-panel-navigation-tab active" data-tab="globals">Globals</button>
            `;

            const sysColors = pageAst.global_styles.system_colors || [];
            const sysTypography = pageAst.global_styles.system_typography || [];

            let chipsHtml = "";
            sysColors.forEach(c => {
                chipsHtml += `
                    <div class="globals-color-chip" title="Click to edit ${c.title} in Site Settings">
                        <div class="globals-chip-swatch" style="background:${c.color};"></div>
                        <div class="globals-chip-text">
                            <span class="globals-chip-name">${c.title}</span>
                            <span class="globals-chip-hex">${c.color}</span>
                        </div>
                    </div>
                `;
            });

            let typeListHtml = "";
            sysTypography.forEach(t => {
                typeListHtml += `
                    <div class="globals-type-item">
                        <span class="globals-type-name">${t.title}</span>
                        <span class="globals-type-spec">${t.font_family.split(",")[0]} &bull; ${t.font_size || '16px'}</span>
                    </div>
                `;
            });

            bodyEl.innerHTML = `
                <div class="elementor-globals-panel">
                    <!-- Global Design Tokens -->
                    <div>
                        <div class="globals-section-header">
                            <span>Global Colors</span>
                            <button class="globals-manage-pill" id="btn-jump-colors">
                                <i class="fa-solid fa-pen"></i> Manage
                            </button>
                        </div>
                        <div class="globals-colors-grid">
                            ${chipsHtml}
                        </div>
                    </div>

                    <!-- Global Typography -->
                    <div>
                        <div class="globals-section-header">
                            <span>Global Typography</span>
                            <button class="globals-manage-pill" id="btn-jump-fonts">
                                <i class="fa-solid fa-pen"></i> Manage
                            </button>
                        </div>
                        <div class="globals-typography-list">
                            ${typeListHtml}
                        </div>
                    </div>

                    <!-- Reusable Global Blocks -->
                    <div>
                        <div class="globals-section-header">
                            <span>Global Blocks & Templates</span>
                            <span style="font-size:9px;color:#818a91;">Drag to canvas</span>
                        </div>
                        <div class="reusable-blocks-grid">
                            <div class="reusable-block-card" data-global-widget="GlobalHeader" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-compass"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Global Header Navbar</span>
                                    <span class="reusable-block-desc">Clean responsive branding & CTA bar</span>
                                </div>
                            </div>
                            <div class="reusable-block-card" data-global-widget="GlobalHero" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-wand-magic-sparkles"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Hero Feature Banner</span>
                                    <span class="reusable-block-desc">High-impact title, subtitle & button</span>
                                </div>
                            </div>
                            <div class="reusable-block-card" data-global-widget="GlobalPricing" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-tags"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Pricing Matrix Cards</span>
                                    <span class="reusable-block-desc">3-column plans with highlighted tier</span>
                                </div>
                            </div>
                            <div class="reusable-block-card" data-global-widget="GlobalFooter" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-shoe-prints"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Global Site Footer</span>
                                    <span class="reusable-block-desc">Dark footer with copyright & links</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Elementor Pro / Global Widget Nerd Box -->
                    <div class="elementor-nerd-box">
                        <div class="elementor-nerd-box-icon"><i class="fa-solid fa-globe"></i></div>
                        <div class="elementor-nerd-box-title">Meet Our Global System</div>
                        <div class="elementor-nerd-box-message">Save any widget or layout as a global template component. Edit once to sync across your entire site in native compiled Rust.</div>
                    </div>
                </div>
            `;

            // Wire Tab Switchers
            navEl.querySelector('[data-tab="elements"]').onclick = () => {
                currentPanelView = "elements";
                renderInspector();
            };

            document.getElementById("btn-jump-colors").onclick = () => {
                currentPanelView = "site-settings-colors";
                renderInspector();
            };
            document.getElementById("btn-jump-fonts").onclick = () => {
                currentPanelView = "site-settings-fonts";
                renderInspector();
            };

            bodyEl.querySelectorAll(".globals-color-chip").forEach(chip => {
                chip.onclick = () => {
                    currentPanelView = "site-settings-colors";
                    renderInspector();
                };
            });

            // Draggable Global Blocks
            bodyEl.querySelectorAll(".reusable-block-card").forEach(card => {
                const type = card.dataset.globalWidget;
                card.ondragstart = (e) => {
                    window._activeDragData = { type: "new_widget", widgetType: type };
                    try {
                        e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                        e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                        e.dataTransfer.effectAllowed = "copy";
                    } catch(err) {}
                    card.classList.add("is-dragging-card");
                };
                card.ondragend = () => {
                    card.classList.remove("is-dragging-card");
                    setTimeout(() => {
                        window._activeDragData = null;
                        hideDropIndicator();
                    }, 350);
                };
                card.onclick = () => {
                    insertWidgetAtRoot(type);
                };
            });

            return;
        }

"###;
