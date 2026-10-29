//! Typography, Backgrounds and Borders CAD Style Panels (`scripts/inspector_styles.rs`).

pub const JS_INSPECTOR_STYLES: &str = r###"
(function() {
    const SANS_FONTS = [
        "Inter", "Schibsted Grotesk", "Plus Jakarta Sans", "Space Grotesk", "DM Sans",
        "Manrope", "Outfit", "Work Sans", "Roboto", "Open Sans", "Lato",
        "Montserrat", "Poppins", "Raleway", "Nunito", "Oswald"
    ];
    const SERIF_FONTS = [
        "Newsreader", "Playfair Display", "Fraunces", "Instrument Serif",
        "Merriweather", "Lora", "DM Serif Display", "Libre Baskerville"
    ];
    const MONO_FONTS = [
        "IBM Plex Mono", "JetBrains Mono", "Fira Code"
    ];
    const SYSTEM_FONTS = [
        ["system-ui, -apple-system, 'Segoe UI', sans-serif", "System UI Sans"],
        ["Georgia, 'Times New Roman', serif", "System Serif (Georgia)"],
        ["'Helvetica Neue', Arial, sans-serif", "Helvetica / Arial"],
        ["ui-monospace, Menlo, Consolas, monospace", "System Monospace"]
    ];

    function cleanFontLabel(v) {
        const s = String(v || "").trim();
        if (!s) return "";
        if (s.startsWith("var(")) return s;
        return s.split(",")[0].replace(/["']/g, "").trim() || s;
    }

    function renderFontControl() {
        const esc = window.NSInspector.esc;
        const gs = window.DayStudioState && window.DayStudioState.pageAst && window.DayStudioState.pageAst.global_styles;
        const tokenOpts = [
            ["var(--serif)", "Token: Display / Serif (--serif)"],
            ["var(--sans)", "Token: Body / Sans (--sans)"],
            ["var(--mono)", "Token: Monospace (--mono)"]
        ];
        if (gs) {
            [...(gs.system_typography || []), ...(gs.custom_typography || [])].forEach(t => {
                if (t && t.font_family) {
                    const label = `${t.title || t.id}: ${cleanFontLabel(t.font_family)}`;
                    if (!tokenOpts.some(([v]) => v === t.font_family)) {
                        tokenOpts.push([t.font_family, label]);
                    }
                }
            });
        }

        const optGroup = (label, items) =>
            `<optgroup label="${esc(label)}">` +
            items.map(it => {
                const [v, l] = Array.isArray(it) ? it : [it, it];
                return `<option value="${esc(v)}">${esc(l)}</option>`;
            }).join("") +
            `</optgroup>`;

        return `
            <div class="ns-font-control" style="display:flex;flex-direction:column;gap:4px;width:100%;">
                <div style="display:flex;gap:4px;width:100%;align-items:center;">
                    <select class="cad-select ns-font-picker" data-font-select="font_family" style="flex:1;min-width:0;">
                        <option value="">— Inherit</option>
                        ${optGroup("Design System Tokens", tokenOpts)}
                        ${optGroup("Sans-Serif Fonts", SANS_FONTS)}
                        ${optGroup("Serif & Editorial Fonts", SERIF_FONTS)}
                        ${optGroup("Monospace Fonts", MONO_FONTS)}
                        ${optGroup("System Font Stacks", SYSTEM_FONTS)}
                        <option value="__custom__">Custom font / CSS var…</option>
                    </select>
                    <button type="button" class="ns-clear" data-clear="font_family" title="Reset font family">&times;</button>
                </div>
                <input type="text" class="cad-input font-mono ns-font-custom" placeholder="e.g. 'Newsreader', serif or var(--serif)" spellcheck="false" autocomplete="off" style="display:none;width:100%;">
            </div>
        `;
    }

    function refreshFontControls(root, node) {
        const esc = window.NSInspector.esc;
        root.querySelectorAll("select.ns-font-picker[data-font-select]").forEach(sel => {
            const prop = sel.dataset.fontSelect;
            const val = window.getLayerValue(node, prop);
            const inh = window.getInheritedValue(node, prop) || window.NSInspector.computedOf(node, prop);
            const wrap = sel.closest(".ns-font-control");
            const customInput = wrap ? wrap.querySelector(".ns-font-custom") : null;

            sel.options[0].textContent = "— " + window.NSInspector.short(cleanFontLabel(inh) || "Inherit", 16);
            sel.classList.toggle("is-unset", !val);

            if (val) {
                let matched = Array.from(sel.options).find(o => o.value === val);
                if (!matched) {
                    const primary = cleanFontLabel(val).toLowerCase();
                    matched = Array.from(sel.options).find(o => o.value && o.value !== "__custom__" && cleanFontLabel(o.value).toLowerCase() === primary);
                }
                if (!matched) {
                    let dyn = sel.querySelector('option[data-dyn-font="1"]');
                    if (!dyn) {
                        dyn = document.createElement("option");
                        dyn.dataset.dynFont = "1";
                        sel.insertBefore(dyn, sel.options[1]);
                    }
                    dyn.value = val;
                    dyn.textContent = cleanFontLabel(val) + " (Custom)";
                    matched = dyn;
                }
                if (document.activeElement !== sel && sel.dataset.customOpen !== "1") {
                    sel.value = matched.value;
                }
                if (customInput && document.activeElement !== customInput) {
                    customInput.value = val;
                }
            } else {
                if (document.activeElement !== sel && sel.dataset.customOpen !== "1") {
                    sel.value = "";
                }
                if (customInput && document.activeElement !== customInput && sel.dataset.customOpen !== "1") {
                    customInput.value = "";
                    customInput.style.display = "none";
                }
            }
        });
    }
    window.refreshFontControls = refreshFontControls;

    function bindFontControl(panel, node) {
        const sel = panel.querySelector("select.ns-font-picker[data-font-select]");
        const wrap = sel ? sel.closest(".ns-font-control") : null;
        const customInput = wrap ? wrap.querySelector(".ns-font-custom") : null;
        if (!sel) return;

        sel.addEventListener("change", () => {
            if (sel.value === "__custom__") {
                sel.dataset.customOpen = "1";
                if (customInput) {
                    customInput.style.display = "block";
                    customInput.value = window.getLayerValue(node, "font_family") || "";
                    customInput.focus();
                    customInput.select();
                }
                return;
            }
            delete sel.dataset.customOpen;
            if (customInput) customInput.style.display = "none";
            if (sel.value && window.DayStyle && window.DayStyle.ensureFontLoaded) {
                window.DayStyle.ensureFontLoaded(sel.value);
            }
            window.setLocalStyle("font_family", sel.value, { commit: true });
            window.NSInspector.refresh(panel, node);
        });

        if (customInput) {
            customInput.addEventListener("input", () => {
                const v = customInput.value.trim();
                if (v && window.DayStyle && window.DayStyle.ensureFontLoaded) {
                    window.DayStyle.ensureFontLoaded(v);
                }
                window.setLocalStyle("font_family", v, false);
                window.NSInspector.refresh(panel, node);
            });
            customInput.addEventListener("change", () => {
                delete sel.dataset.customOpen;
                const v = customInput.value.trim();
                window.setLocalStyle("font_family", v, true);
                window.NSInspector.refresh(panel, node);
            });
            customInput.addEventListener("keydown", (e) => {
                if (e.key === "Enter") { e.preventDefault(); customInput.blur(); }
                if (e.key === "Escape") {
                    delete sel.dataset.customOpen;
                    customInput.style.display = "none";
                    window.NSInspector.refresh(panel, node);
                }
            });
        }
    }

    function renderTokenColorChips(propName) {
        const gs = window.DayStudioState && window.DayStudioState.pageAst && window.DayStudioState.pageAst.global_styles;
        if (!gs) return "";
        const tokens = [...(gs.system_colors || []), ...(gs.custom_colors || [])].slice(0, 8);
        if (!tokens.length) return "";
        return `<div class="cad-row ns-presets" style="gap:4px;flex-wrap:wrap;">` +
            tokens.map(c => {
                const cssVar = `var(--${c.id}, ${c.color})`;
                return `<button type="button" class="ns-chip" data-token-prop="${propName}" data-token-val="${window.NSInspector.esc(cssVar)}" title="${window.NSInspector.esc(c.title || c.id)}: ${window.NSInspector.esc(c.color)}" style="width:18px;height:18px;padding:0;background:${window.NSInspector.esc(c.color)};border:1px solid var(--day-border);"></button>`;
            }).join("") +
            `</div>`;
    }

    function bindTokenChips(panel, node) {
        panel.querySelectorAll("[data-token-prop]").forEach(btn => {
            btn.onclick = () => {
                window.setLocalStyle(btn.dataset.tokenProp, btn.dataset.tokenVal, { commit: true });
                window.NSInspector.refresh(panel, node);
            };
        });
    }

    function renderVisualPanels(container, node) {
        const N = window.NSInspector;

        // ---------------------------------------------------------
        // 1. TYPOGRAPHY
        // ---------------------------------------------------------
        const typoBody = `
            ${N.row("Family", "font_family", renderFontControl())}
            ${N.pair("Size", "font_size", N.input("font_size", "16px"), "Weight", "font_weight", N.select("font_weight", [
                ["100", "100 Thin"], ["200", "200 Extra Light"], ["300", "300 Light"], ["400", "400 Normal"], ["500", "500 Medium"],
                ["600", "600 Semi Bold"], ["700", "700 Bold"], ["800", "800 Extra Bold"], ["900", "900 Black"]
            ]))}
            ${N.pair("Height", "line_height", N.input("line_height", "1.5"), "Spacing", "letter_spacing", N.input("letter_spacing", "0px"))}
            ${N.row("Color", "text_color", N.color("text_color"))}
            ${renderTokenColorChips("text_color")}
            ${N.row("Align", "text_align", N.seg("text_align", [
                ["left", '<i class="fa-solid fa-align-left"></i>'], ["center", '<i class="fa-solid fa-align-center"></i>'],
                ["right", '<i class="fa-solid fa-align-right"></i>'], ["justify", '<i class="fa-solid fa-align-justify"></i>']
            ]))}
            ${N.row("Case", "text_transform", N.seg("text_transform", [
                ["none", '<i class="fa-solid fa-minus"></i>', "none"], ["uppercase", "AA", "uppercase"], ["capitalize", "Aa", "capitalize"], ["lowercase", "aa", "lowercase"]
            ]))}
            ${N.row("Style", "font_style,text_decoration", `
                ${N.seg("font_style", [["normal", '<i class="fa-solid fa-font"></i>', "normal"], ["italic", '<i class="fa-solid fa-italic"></i>', "italic"]])}
                ${N.seg("text_decoration", [["none", '<i class="fa-solid fa-minus"></i>', "none"], ["underline", '<i class="fa-solid fa-underline"></i>', "underline"], ["line-through", '<i class="fa-solid fa-strikethrough"></i>', "line-through"]])}
            `)}
            ${N.row("Shadow", "text_shadow", N.input("text_shadow", "0 1px 2px rgba(0,0,0,.3)"))}
        `;
        const typoPanel = N.panel(container, "typography", "fa-font", "Typography", typoBody);
        bindFontControl(typoPanel, node);
        bindTokenChips(typoPanel, node);
        N.bind(typoPanel, node);

        // ---------------------------------------------------------
        // 2. BACKGROUNDS
        // ---------------------------------------------------------
        const IMG = 'data-show-if="background_image!=none|"';
        const bgBody = `
            ${N.row("Color", "background_color", N.color("background_color"))}
            ${renderTokenColorChips("background_color")}
            ${N.row("Image", "background_image", N.input("background_image", "https://… or /uploads/…"))}
            ${N.pair("Size", "background_size", N.select("background_size", [["cover", "Cover"], ["contain", "Contain"], ["auto", "Auto"], ["100% 100%", "Stretch"]]),
                     "Repeat", "background_repeat", N.select("background_repeat", [["no-repeat", "None"], ["repeat", "Tile"], ["repeat-x", "X"], ["repeat-y", "Y"]]), IMG)}
            ${N.pair("Position", "background_position", N.select("background_position", [
                ["center center", "Center"], ["top center", "Top"], ["bottom center", "Bottom"], ["center left", "Left"], ["center right", "Right"], ["top left", "Top left"], ["top right", "Top right"]
            ]), "Fixed", "background_attachment", N.select("background_attachment", [["scroll", "Scroll"], ["fixed", "Fixed (parallax)"], ["local", "Local"]]), IMG)}
            ${N.row("Gradient", "background_gradient", N.input("background_gradient", "linear-gradient(…)"))}
            <div class="cad-row ns-presets">
                <button type="button" class="ns-chip" data-grad="linear-gradient(135deg, #2563eb 0%, #7c3aed 100%)" style="background:linear-gradient(135deg,#2563eb,#7c3aed)">&nbsp;</button>
                <button type="button" class="ns-chip" data-grad="linear-gradient(180deg, #0f172a 0%, #1e293b 100%)" style="background:linear-gradient(180deg,#0f172a,#1e293b)">&nbsp;</button>
                <button type="button" class="ns-chip" data-grad="linear-gradient(135deg, #f59e0b 0%, #ef4444 100%)" style="background:linear-gradient(135deg,#f59e0b,#ef4444)">&nbsp;</button>
                <button type="button" class="ns-chip" data-grad="linear-gradient(180deg, rgba(0,0,0,0) 0%, rgba(0,0,0,0.6) 100%)" title="Dark overlay (combine with an image)" style="background:linear-gradient(180deg,#555,#000)">&nbsp;</button>
                <button type="button" class="ns-chip" data-grad="" title="Remove gradient">None</button>
            </div>
        `;
        const bgPanel = N.panel(container, "background", "fa-fill-drip", "Backgrounds", bgBody);
        bgPanel.querySelectorAll("[data-grad]").forEach(b => {
            b.onclick = () => {
                window.setLocalStyle("background_gradient", b.dataset.grad, { commit: true });
                N.refresh(bgPanel, node);
            };
        });
        bindTokenChips(bgPanel, node);
        N.bind(bgPanel, node);

        // ---------------------------------------------------------
        // 3. BORDERS
        // ---------------------------------------------------------
        const borderBody = `
            ${N.pair("Radius", "border_radius", N.input("border_radius", "0px"), "Width", "border_width", N.input("border_width", "0px"))}
            ${N.row("Style", "border_style", N.seg("border_style", [
                ["none", '<i class="fa-solid fa-ban"></i>', "none"], ["solid", "Solid"], ["dashed", "Dashed"], ["dotted", "Dotted"], ["double", "Double"]
            ]))}
            ${N.row("Color", "border_color", N.color("border_color"))}
            ${renderTokenColorChips("border_color")}
            <div class="cad-row ns-presets">
                ${["0px", "4px", "8px", "16px", "50%", "9999px"].map(r => `<button type="button" class="ns-chip" data-radius="${r}" title="border-radius: ${r}">${r === "9999px" ? "Pill" : r}</button>`).join("")}
            </div>
        `;
        const borderPanel = N.panel(container, "border", "fa-vector-square", "Borders", borderBody);
        borderPanel.querySelectorAll("[data-radius]").forEach(b => {
            b.onclick = () => {
                window.setLocalStyle("border_radius", b.dataset.radius, { commit: true });
                N.refresh(borderPanel, node);
            };
        });
        bindTokenChips(borderPanel, node);
        N.bind(borderPanel, node);
    }
    window.renderVisualPanels = renderVisualPanels;
})();
"###;
