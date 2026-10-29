//! Interactive Design Tokens Drawer (`scripts/tokens_drawer.rs`).
//!
//! Live editor bound to `window.DayStudioState.pageAst.global_styles` (`ElementorKit`)
//! supporting System & Custom Colors, Typography Families & Scales, Layout Measure,
//! Button Defaults, and Custom CSS with real-time artboard injection and SQLite persistence.

pub const JS_TOKENS_DRAWER: &str = r###"
(function() {
    function ensureGlobalStyles() {
        const state = window.DayStudioState;
        if (!state.pageAst) {
            state.pageAst = { version: "1.0", root: { id: "root", type: "Root", styles: {}, children: [] } };
        }
        if (!state.pageAst.global_styles || typeof state.pageAst.global_styles !== "object") {
            state.pageAst.global_styles = {
                system_colors: [
                    { id: "primary", title: "Primary Brand", color: "#C8371A" },
                    { id: "secondary", title: "Secondary / Ink", color: "#13120F" },
                    { id: "text", title: "Body Text", color: "#26241F" },
                    { id: "accent", title: "Accent", color: "#1F5E3B" }
                ],
                custom_colors: [
                    { id: "paper", title: "Paper Surface", color: "#F5F1E8" },
                    { id: "cream", title: "Cream Card", color: "#ECE6D8" },
                    { id: "rule", title: "Hairline Rule", color: "#D6CEBC" }
                ],
                system_typography: [
                    { id: "primary", title: "Display Heading", font_family: "'Newsreader', Georgia, serif", font_size: "48px", font_weight: "400", line_height: "1.05" },
                    { id: "secondary", title: "Section Heading", font_family: "'Schibsted Grotesk', sans-serif", font_size: "28px", font_weight: "600", line_height: "1.15" },
                    { id: "text", title: "Body Copy", font_family: "'Schibsted Grotesk', sans-serif", font_size: "16px", font_weight: "400", line_height: "1.55" },
                    { id: "accent", title: "Monospace / Spec", font_family: "'IBM Plex Mono', monospace", font_size: "12px", font_weight: "500", line_height: "1.4" }
                ],
                custom_typography: [],
                content_width: 1320,
                widgets_space: 24,
                button_style: {
                    background_color: "#13120F",
                    text_color: "#F5F1E8",
                    border_radius: "0px"
                },
                custom_css: ""
            };
        }
        const gs = state.pageAst.global_styles;
        if (!Array.isArray(gs.system_colors)) gs.system_colors = [];
        if (!Array.isArray(gs.custom_colors)) gs.custom_colors = [];
        if (!Array.isArray(gs.system_typography)) gs.system_typography = [];
        if (!Array.isArray(gs.custom_typography)) gs.custom_typography = [];
        if (!gs.content_width) gs.content_width = 1320;
        if (!gs.widgets_space) gs.widgets_space = 24;
        if (!gs.button_style || typeof gs.button_style !== "object") {
            gs.button_style = { background_color: "#13120F", text_color: "#F5F1E8", border_radius: "0px" };
        }
        return gs;
    }
    window.ensureGlobalStyles = ensureGlobalStyles;

    function syncGlobalTokensCss() {
        const state = window.DayStudioState;
        if (!state || !state.pageAst || !state.pageAst.global_styles) return;
        const gs = state.pageAst.global_styles;
        let styleEl = document.getElementById("daystudio-live-tokens-css");
        if (!styleEl) {
            styleEl = document.createElement("style");
            styleEl.id = "daystudio-live-tokens-css";
            document.head.appendChild(styleEl);
        }

        const lines = [];
        lines.push("#daystudio-site-preview, #daystudio-canvas {");
        lines.push(`  --e-content-width: ${gs.content_width || 1320}px;`);
        lines.push(`  --dp-container-max-width: ${gs.content_width || 1320}px;`);
        lines.push(`  --wide: ${gs.content_width || 1320}px;`);
        lines.push(`  --e-widgets-space: ${gs.widgets_space || 24}px;`);

        const allColors = [...(gs.system_colors || []), ...(gs.custom_colors || [])];
        allColors.forEach(c => {
            if (!c || !c.id || !c.color) return;
            lines.push(`  --e-global-color-${c.id}: ${c.color};`);
            lines.push(`  --dp-color-${c.id}: ${c.color};`);
            lines.push(`  --${c.id}: ${c.color};`);
            if (c.id === "secondary") lines.push(`  --dp-color-heading: ${c.color};`);
        });

        const allTypo = [...(gs.system_typography || []), ...(gs.custom_typography || [])];
        allTypo.forEach(t => {
            if (!t || !t.id || !t.font_family) return;
            lines.push(`  --e-global-typography-${t.id}-font-family: ${t.font_family};`);
            lines.push(`  --${t.id}: ${t.font_family};`);
            if (t.id === "primary") lines.push(`  --dp-font-heading: ${t.font_family}; --serif: ${t.font_family};`);
            if (t.id === "secondary") lines.push(`  --dp-font-subheading: ${t.font_family};`);
            if (t.id === "text") lines.push(`  --dp-font-body: ${t.font_family}; --sans: ${t.font_family};`);
            if (t.id === "accent") lines.push(`  --dp-font-accent: ${t.font_family}; --mono: ${t.font_family};`);
        });

        if (gs.button_style) {
            if (gs.button_style.background_color) {
                lines.push(`  --e-button-bg: ${gs.button_style.background_color};`);
                lines.push(`  --dp-button-bg: ${gs.button_style.background_color};`);
            }
            if (gs.button_style.text_color) {
                lines.push(`  --e-button-color: ${gs.button_style.text_color};`);
                lines.push(`  --dp-button-color: ${gs.button_style.text_color};`);
            }
            if (gs.button_style.border_radius) {
                lines.push(`  --e-button-border-radius: ${gs.button_style.border_radius};`);
                lines.push(`  --dp-button-radius: ${gs.button_style.border_radius};`);
            }
        }
        lines.push("}");
        if (gs.custom_css && String(gs.custom_css).trim()) {
            lines.push(String(gs.custom_css));
        }

        styleEl.textContent = lines.join("\n");
    }
    window.syncGlobalTokensCss = syncGlobalTokensCss;

    function toHexPickerValue(val) {
        const s = String(val || "").trim();
        if (/^#[0-9a-fA-F]{6}$/.test(s)) return s;
        if (/^#[0-9a-fA-F]{3}$/.test(s)) {
            return "#" + s[1] + s[1] + s[2] + s[2] + s[3] + s[3];
        }
        return "#C8371A";
    }

    function escAttr(s) {
        return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
    }

    function renderTokensDrawer(container) {
        if (!container) return;
        const gs = ensureGlobalStyles();
        syncGlobalTokensCss();

        const renderColorRow = (c, idx, isCustom) => `
            <div class="cad-row" style="gap:6px;margin-bottom:6px;" data-color-group="${isCustom ? 'custom' : 'system'}" data-idx="${idx}">
                <input type="color" class="tok-color-picker" value="${toHexPickerValue(c.color)}" style="width:24px;height:24px;padding:0;border:1px solid var(--day-border);background:none;cursor:pointer;flex-shrink:0;">
                <input type="text" class="cad-input tok-color-title" value="${escAttr(c.title || c.id)}" placeholder="Token Name" style="flex:1;min-width:0;font-size:11px;">
                <input type="text" class="cad-input font-mono tok-color-val" value="${escAttr(c.color)}" style="width:82px;font-size:11px;">
                ${isCustom ? `<button type="button" class="cad-btn secondary tok-color-del" title="Delete Token" style="padding:2px 6px;font-size:10px;">✕</button>` : ""}
            </div>
        `;

        const renderTypoBlock = (t, idx, isCustom) => `
            <div style="border:1px solid var(--day-border);border-radius:4px;padding:8px;margin-bottom:8px;background:rgba(255,255,255,0.02);" data-typo-group="${isCustom ? 'custom' : 'system'}" data-idx="${idx}">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px;">
                    <span style="font-size:11px;font-weight:600;color:var(--day-ink);">${escAttr(t.title || t.id)} <code style="font-size:10px;color:var(--day-muted);">(--${escAttr(t.id)})</code></span>
                </div>
                <div class="cad-row" style="margin-bottom:4px;">
                    <span class="cad-label" style="width:52px;">Family</span>
                    <input type="text" class="cad-input tok-typo-family" value="${escAttr(t.font_family || '')}" style="flex:1;font-size:11px;">
                </div>
                <div class="cad-row" style="gap:4px;">
                    <span class="cad-label" style="width:36px;">Size</span>
                    <input type="text" class="cad-input font-mono tok-typo-size" value="${escAttr(t.font_size || '')}" placeholder="16px" style="width:68px;font-size:11px;">
                    <span class="cad-label" style="width:32px;">Wght</span>
                    <input type="text" class="cad-input font-mono tok-typo-weight" value="${escAttr(t.font_weight || '')}" placeholder="400" style="width:46px;font-size:11px;">
                    <span class="cad-label" style="width:24px;">LH</span>
                    <input type="text" class="cad-input font-mono tok-typo-lh" value="${escAttr(t.line_height || '')}" placeholder="1.5" style="width:46px;font-size:11px;">
                </div>
            </div>
        `;

        container.innerHTML = `
            <div class="drawer-header" style="display:flex;justify-content:space-between;align-items:center;">
                <span class="drawer-title">Design Tokens (ElementorKit)</span>
                <button type="button" class="cad-btn primary" id="tok-save-top-btn" style="padding:4px 10px;font-size:11px;"><i class="fa-solid fa-floppy-disk"></i> Save</button>
            </div>
            <div class="drawer-scroll-body" style="padding:12px;">
                <div style="margin-bottom:16px;">
                    <div style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.08em;color:var(--day-muted);margin-bottom:8px;font-weight:700;">System Color Tokens</div>
                    <div id="tok-sys-colors">${gs.system_colors.map((c, i) => renderColorRow(c, i, false)).join("")}</div>
                </div>

                <div style="margin-bottom:16px;">
                    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">
                        <span style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.08em;color:var(--day-muted);font-weight:700;">Custom Palette Tokens (${gs.custom_colors.length})</span>
                        <button type="button" class="cad-btn secondary" id="tok-add-color-btn" style="padding:2px 8px;font-size:10.5px;">+ Add Color</button>
                    </div>
                    <div id="tok-cust-colors">${gs.custom_colors.map((c, i) => renderColorRow(c, i, true)).join("")}</div>
                </div>

                <div style="margin-bottom:16px;">
                    <div style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.08em;color:var(--day-muted);margin-bottom:8px;font-weight:700;">Typography Scale & Fonts</div>
                    <div id="tok-sys-typo">${gs.system_typography.map((t, i) => renderTypoBlock(t, i, false)).join("")}</div>
                    <div id="tok-cust-typo">${gs.custom_typography.map((t, i) => renderTypoBlock(t, i, true)).join("")}</div>
                </div>

                <div style="margin-bottom:16px;border-top:1px solid var(--day-border);padding-top:12px;">
                    <div style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.08em;color:var(--day-muted);margin-bottom:8px;font-weight:700;">Layout Measure & Buttons</div>
                    <div class="cad-row" style="margin-bottom:6px;">
                        <span class="cad-label">Max Measure (px)</span>
                        <input type="number" class="cad-input font-mono" id="tok-content-width" value="${gs.content_width || 1320}" style="width:90px;">
                    </div>
                    <div class="cad-row" style="margin-bottom:6px;">
                        <span class="cad-label">Block Gap (px)</span>
                        <input type="number" class="cad-input font-mono" id="tok-widgets-space" value="${gs.widgets_space || 24}" style="width:90px;">
                    </div>
                    <div class="cad-row" style="margin-bottom:6px;">
                        <span class="cad-label">Button BG</span>
                        <input type="text" class="cad-input font-mono" id="tok-btn-bg" value="${escAttr((gs.button_style && gs.button_style.background_color) || '#13120F')}" style="width:110px;">
                    </div>
                    <div class="cad-row" style="margin-bottom:6px;">
                        <span class="cad-label">Button Text</span>
                        <input type="text" class="cad-input font-mono" id="tok-btn-color" value="${escAttr((gs.button_style && gs.button_style.text_color) || '#F5F1E8')}" style="width:110px;">
                    </div>
                    <div class="cad-row">
                        <span class="cad-label">Button Radius</span>
                        <input type="text" class="cad-input font-mono" id="tok-btn-radius" value="${escAttr((gs.button_style && gs.button_style.border_radius) || '0px')}" style="width:110px;">
                    </div>
                </div>

                <div style="margin-bottom:14px;border-top:1px solid var(--day-border);padding-top:12px;">
                    <div style="font-size:10.5px;text-transform:uppercase;letter-spacing:0.08em;color:var(--day-muted);margin-bottom:6px;font-weight:700;">Global Stylesheet / Custom CSS</div>
                    <textarea class="cad-textarea font-mono" id="tok-custom-css" rows="5" style="width:100%;font-size:11px;">${String(gs.custom_css || "").replace(/</g, "&lt;")}</textarea>
                </div>

                <button type="button" class="cad-btn primary" id="tok-save-bottom-btn" style="width:100%;justify-content:center;padding:8px;"><i class="fa-solid fa-floppy-disk"></i> Save Design Tokens to SQLite</button>
            </div>
        `;

        const onMutate = () => {
            syncGlobalTokensCss();
            if (window.markDirty) window.markDirty(true);
            if (window.renderInspector && window.DayStudioState.activeElementId) window.renderInspector();
        };

        container.querySelectorAll("[data-color-group]").forEach(row => {
            const isCust = row.dataset.colorGroup === "custom";
            const idx = parseInt(row.dataset.idx, 10);
            const targetArr = isCust ? gs.custom_colors : gs.system_colors;
            const item = targetArr[idx];
            if (!item) return;

            const picker = row.querySelector(".tok-color-picker");
            const titleInp = row.querySelector(".tok-color-title");
            const valInp = row.querySelector(".tok-color-val");
            const delBtn = row.querySelector(".tok-color-del");

            if (picker) {
                picker.oninput = () => {
                    item.color = picker.value;
                    if (valInp) valInp.value = picker.value;
                    onMutate();
                };
            }
            if (valInp) {
                valInp.oninput = () => {
                    item.color = valInp.value.trim();
                    if (picker && /^#[0-9a-fA-F]{6}$/.test(item.color)) picker.value = item.color;
                    onMutate();
                };
            }
            if (titleInp) {
                titleInp.oninput = () => {
                    item.title = titleInp.value;
                    if (window.markDirty) window.markDirty(true);
                };
            }
            if (delBtn && isCust) {
                delBtn.onclick = () => {
                    gs.custom_colors.splice(idx, 1);
                    onMutate();
                    renderTokensDrawer(container);
                };
            }
        });

        const addColorBtn = container.querySelector("#tok-add-color-btn");
        if (addColorBtn) {
            addColorBtn.onclick = () => {
                const n = gs.custom_colors.length + 1;
                gs.custom_colors.push({ id: "token-" + n, title: "Token " + n, color: "#C8371A" });
                onMutate();
                renderTokensDrawer(container);
            };
        }

        container.querySelectorAll("[data-typo-group]").forEach(block => {
            const isCust = block.dataset.typoGroup === "custom";
            const idx = parseInt(block.dataset.idx, 10);
            const targetArr = isCust ? gs.custom_typography : gs.system_typography;
            const item = targetArr[idx];
            if (!item) return;

            const fam = block.querySelector(".tok-typo-family");
            const sz = block.querySelector(".tok-typo-size");
            const wt = block.querySelector(".tok-typo-weight");
            const lh = block.querySelector(".tok-typo-lh");

            if (fam) fam.oninput = () => { item.font_family = fam.value; onMutate(); };
            if (sz) sz.oninput = () => { item.font_size = sz.value || null; onMutate(); };
            if (wt) wt.oninput = () => { item.font_weight = wt.value || null; onMutate(); };
            if (lh) lh.oninput = () => { item.line_height = lh.value || null; onMutate(); };
        });

        const cw = container.querySelector("#tok-content-width");
        if (cw) cw.oninput = () => { gs.content_width = parseInt(cw.value, 10) || 1320; onMutate(); };
        const ws = container.querySelector("#tok-widgets-space");
        if (ws) ws.oninput = () => { gs.widgets_space = parseInt(ws.value, 10) || 24; onMutate(); };
        const bBg = container.querySelector("#tok-btn-bg");
        if (bBg) bBg.oninput = () => { gs.button_style.background_color = bBg.value; onMutate(); };
        const bCol = container.querySelector("#tok-btn-color");
        if (bCol) bCol.oninput = () => { gs.button_style.text_color = bCol.value; onMutate(); };
        const bRad = container.querySelector("#tok-btn-radius");
        if (bRad) bRad.oninput = () => { gs.button_style.border_radius = bRad.value; onMutate(); };
        const cCss = container.querySelector("#tok-custom-css");
        if (cCss) cCss.oninput = () => { gs.custom_css = cCss.value; onMutate(); };

        const saveHandler = () => {
            if (window.pushHistorySnapshot) window.pushHistorySnapshot();
            if (window.saveCurrentLayout) window.saveCurrentLayout();
        };
        const saveTop = container.querySelector("#tok-save-top-btn");
        if (saveTop) saveTop.onclick = saveHandler;
        const saveBot = container.querySelector("#tok-save-bottom-btn");
        if (saveBot) saveBot.onclick = saveHandler;
    }
    window.renderTokensDrawer = renderTokensDrawer;

    document.addEventListener("DOMContentLoaded", () => {
        setTimeout(syncGlobalTokensCss, 50);
    });
})();
"###;
