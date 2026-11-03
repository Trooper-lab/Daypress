//! Body (Page Root), Document Template, Canvas Background & SEO Settings Inspector (`scripts/inspector_page_settings.rs`).

pub const JS_INSPECTOR_PAGE_SETTINGS: &str = r###"
(function() {
    function ensureRootDataAndStyles() {
        const state = window.DayStudioState;
        if (!state.pageAst) state.pageAst = { version: 1 };
        if (!state.pageAst.root) state.pageAst.root = { id: "root", type: "Root", children: [] };
        if (!state.pageAst.root.styles) state.pageAst.root.styles = {};
        if (!state.pageAst.root.data) state.pageAst.root.data = {};
        return { root: state.pageAst.root, styles: state.pageAst.root.styles, data: state.pageAst.root.data };
    }

    function applyRootBodyStylesToCanvas() {
        const state = window.DayStudioState;
        if (!state || !state.pageAst || !state.pageAst.root) return;
        const { styles, data } = ensureRootDataAndStyles();
        const artboard = document.getElementById("daystudio-artboard");
        const canvas = document.getElementById("daystudio-canvas");
        const ssrHeader = document.getElementById("daystudio-ssr-header");
        const ssrFooter = document.getElementById("daystudio-ssr-footer");

        if (artboard) {
            artboard.style.backgroundColor = styles.background_color || "";
            artboard.style.color = styles.text_color || "";
            if (styles.font_family) {
                if (window.DayStyle && window.DayStyle.ensureFontLoaded) window.DayStyle.ensureFontLoaded(styles.font_family);
                artboard.style.fontFamily = `'${styles.font_family.replace(/^['"]|['"]$/g, "")}', sans-serif`;
            } else {
                artboard.style.fontFamily = "";
            }
            if (styles.background_image) {
                const bgUrl = String(styles.background_image).trim();
                artboard.style.backgroundImage = /^(url\(|linear-gradient|radial-gradient)/i.test(bgUrl) ? bgUrl : `url("${bgUrl}")`;
                artboard.style.backgroundSize = styles.background_size || "cover";
                artboard.style.backgroundPosition = styles.background_position || "center";
            } else {
                artboard.style.backgroundImage = "";
            }
        }

        if (canvas) {
            canvas.style.paddingTop = styles.padding_top || "";
            canvas.style.paddingBottom = styles.padding_bottom || "";
            canvas.style.paddingLeft = styles.padding_left || "";
            canvas.style.paddingRight = styles.padding_right || "";
            canvas.style.maxWidth = data.page_template === "boxed" ? "860px" : (styles.max_width || "");
            canvas.style.margin = (data.page_template === "boxed" || styles.max_width) ? "0 auto" : "";
            canvas.style.width = "100%";
        }

        const ctx = state.currentContext || {};
        const isHeaderOrFooterTpl = (ctx.type === "template" && ["header", "footer", "section_symbol", "popup"].includes(ctx.templateType))
            || ["template:header", "template:footer", "template:section_symbol", "template:popup"].includes(state.postType);
        const hideChrome = isHeaderOrFooterTpl || data.page_template === "canvas";
        if (ssrHeader) ssrHeader.style.display = hideChrome ? "none" : "";
        if (ssrFooter) ssrFooter.style.display = hideChrome ? "none" : "";

        if (state.pageAst.custom_css !== undefined) {
            let styleTag = document.getElementById("daystudio-page-custom-css");
            if (!styleTag) {
                styleTag = document.createElement("style");
                styleTag.id = "daystudio-page-custom-css";
                document.head.appendChild(styleTag);
            }
            styleTag.textContent = state.pageAst.custom_css || "";
        }
    }
    window.applyRootBodyStylesToCanvas = applyRootBodyStylesToCanvas;

    function attachPanelToggle(panel) {
        const header = panel.querySelector(".cad-panel-header");
        if (header) header.onclick = (e) => { if (!e.target.closest("button")) panel.classList.toggle("open"); };
    }

    function renderBodyPageInspector(inspector) {
        if (!inspector) return;
        const state = window.DayStudioState;
        const ctx = state.currentContext || {};
        const { styles, data } = ensureRootDataAndStyles();
        const isTemplate = ctx.type === "template" || String(state.postType || "").startsWith("template:");

        const prevScroll = document.getElementById("inspector-panels-scroll");
        const keepScroll = prevScroll && inspector.dataset.nodeId === "root" ? prevScroll.scrollTop : 0;
        inspector.dataset.nodeId = "root";

        const titleVal = (ctx.title || data.title || "").replace(/"/g, "&quot;");
        const slugVal = (ctx.slug || data.slug || "").replace(/"/g, "&quot;");
        const statusVal = data.status || "publish";
        const tplMode = data.page_template || "default";
        const bgCol = styles.background_color || "";
        const txtCol = styles.text_color || "";
        const fontFam = styles.font_family || "";
        const bgImg = (styles.background_image || "").replace(/"/g, "&quot;");
        const seoTitle = (data.seo_title || ctx.title || "").replace(/"/g, "&quot;");
        const seoDesc = (data.seo_description || "").replace(/</g, "&lt;").replace(/>/g, "&gt;");
        const ogImage = (data.og_image || "").replace(/"/g, "&quot;");
        const canonicalUrl = (data.canonical_url || "").replace(/"/g, "&quot;");
        const focusKw = (data.focus_keyword || "").replace(/"/g, "&quot;");
        const noindex = !!data.noindex;

        const fonts = (window.DayStyle && window.DayStyle.GOOGLE_FONTS) || ["Inter", "Plus Jakarta Sans", "Space Grotesk", "Newsreader", "Roboto"];
        const fontOpts = `<option value="" ${!fontFam ? "selected" : ""}>Theme Default (System / Inherit)</option>` +
            fonts.map(f => `<option value="${f}" ${fontFam === f ? "selected" : ""}>${f}</option>`).join("");

        inspector.innerHTML = `
            <div class="inspector-selector-bar">
                <div class="selector-target-row" style="justify-content:space-between;">
                    <div class="target-badge">
                        <i class="fa-solid fa-window-maximize" style="color:var(--day-token);"></i>
                        <span>Body &bull; ${isTemplate ? "Template Settings" : "Page & SEO Settings"}</span>
                        <span class="target-id">#root</span>
                    </div>
                </div>
                <div style="font-size:10.5px;color:var(--day-muted);margin-top:4px;">
                    Configure page-wide canvas background, layout template, typography, and SEO metadata.
                </div>
            </div>
            <div id="inspector-panels-scroll" style="flex: 1; overflow-y: auto;"></div>
        `;

        const scrollArea = document.getElementById("inspector-panels-scroll");

        // 1. Document & Template Panel
        const docPanel = document.createElement("div");
        docPanel.className = "cad-panel open";
        docPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-file-lines" style="color:var(--day-signal);"></i> Document & Layout Template</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" id="body-doc-title" value="${titleVal}" placeholder="Page or Template Title"></div>
                <div class="cad-row"><span class="cad-label">URL Slug</span><input type="text" class="cad-input font-mono" style="flex:1;" id="body-doc-slug" value="${slugVal}" placeholder="my-page-slug"></div>
                <div class="cad-row">
                    <span class="cad-label">Status</span>
                    <select class="cad-select" style="flex:1;" id="body-doc-status">
                        <option value="publish" ${statusVal === "publish" ? "selected" : ""}>Published (Live)</option>
                        <option value="draft" ${statusVal === "draft" ? "selected" : ""}>Draft</option>
                        <option value="private" ${statusVal === "private" ? "selected" : ""}>Private</option>
                    </select>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Template</span>
                    <select class="cad-select" style="flex:1;" id="body-doc-template">
                        <option value="default" ${tplMode === "default" ? "selected" : ""}>Default Theme (Header + Footer)</option>
                        <option value="canvas" ${tplMode === "canvas" ? "selected" : ""}>Blank Canvas (No Header / Footer)</option>
                        <option value="boxed" ${tplMode === "boxed" ? "selected" : ""}>Boxed Editorial (860px Centered)</option>
                    </select>
                </div>
                ${isTemplate ? `<div style="margin-top:6px;"><button type="button" class="cad-btn secondary" id="body-btn-conditions" style="width:100%;"><i class="fa-solid fa-sliders" style="color:var(--day-token);"></i> Configure Display Conditions</button></div>` : ""}
            </div>
        `;
        attachPanelToggle(docPanel);
        scrollArea.appendChild(docPanel);

        // 2. Body Background & Typography Panel
        const stylePanel = document.createElement("div");
        stylePanel.className = "cad-panel open";
        stylePanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-palette" style="color:var(--day-token);"></i> Body Background & Typography</span>
                <div style="display:flex;align-items:center;gap:6px;">
                    <button type="button" class="panel-reset-btn" id="body-reset-all-styles" style="display:inline-flex;" title="Reset all Body background, font & padding styles"><i class="fa-solid fa-rotate-left"></i> Reset</button>
                    <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
                </div>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row">
                    <span class="cad-label">Background</span>
                    <div class="color-picker-control" style="flex:1;">
                        <input type="color" class="color-swatch-box" id="body-bg-swatch" value="${bgCol.startsWith("#") && bgCol.length === 7 ? bgCol : "#ffffff"}">
                        <input type="text" class="cad-input font-mono" style="flex:1;" id="body-bg-text" value="${bgCol}" placeholder="#ffffff (Default)">
                        <button type="button" class="ns-clear" id="body-bg-clear" title="Reset background color">&#x21ba;</button>
                    </div>
                </div>
                <div style="display:flex;gap:5px;margin:2px 0 8px 76px;">
                    ${[["#ffffff","White"],["#f8fafc","Slate 50"],["#f4efe6","Warm Paper"],["#0f172a","Slate 900"],["#0b0f19","Night Desk"]].map(([c, lbl]) =>
                        `<button type="button" class="body-bg-preset" data-color="${c}" title="${lbl} (${c})" style="width:20px;height:20px;border-radius:4px;border:1px solid var(--day-rule);background:${c};cursor:pointer;"></button>`
                    ).join("")}
                </div>
                <div class="cad-row">
                    <span class="cad-label">Text Color</span>
                    <div class="color-picker-control" style="flex:1;">
                        <input type="color" class="color-swatch-box" id="body-txt-swatch" value="${txtCol.startsWith("#") && txtCol.length === 7 ? txtCol : "#111827"}">
                        <input type="text" class="cad-input font-mono" style="flex:1;" id="body-txt-text" value="${txtCol}" placeholder="#111827 (Default)">
                        <button type="button" class="ns-clear" id="body-txt-clear" title="Reset text color">&#x21ba;</button>
                    </div>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Body Font</span>
                    <div style="display:flex;gap:4px;flex:1;align-items:center;">
                        <select class="cad-select" style="flex:1;" id="body-font-family">${fontOpts}</select>
                        <button type="button" class="ns-clear" id="body-font-clear" title="Reset body font">&#x21ba;</button>
                    </div>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Bg Image</span>
                    <div style="display:flex;gap:4px;flex:1;align-items:center;">
                        <input type="text" class="cad-input font-mono" style="flex:1;" id="body-bg-image" value="${bgImg}" placeholder="/uploads/... or https://...">
                        <button type="button" class="ns-clear" id="body-bg-img-clear" title="Clear background image">&#x21ba;</button>
                    </div>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Pad Top/Bot</span>
                    <input type="text" class="cad-input font-mono" style="width:62px;" id="body-pad-top" value="${styles.padding_top || ""}" placeholder="0px">
                    <input type="text" class="cad-input font-mono" style="width:62px;" id="body-pad-bottom" value="${styles.padding_bottom || ""}" placeholder="0px">
                    <button type="button" class="ns-clear" id="body-pad-tb-clear" title="Reset top/bottom padding">&#x21ba;</button>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Pad L / R</span>
                    <input type="text" class="cad-input font-mono" style="width:62px;" id="body-pad-left" value="${styles.padding_left || ""}" placeholder="0px">
                    <input type="text" class="cad-input font-mono" style="width:62px;" id="body-pad-right" value="${styles.padding_right || ""}" placeholder="0px">
                    <button type="button" class="ns-clear" id="body-pad-lr-clear" title="Reset left/right padding">&#x21ba;</button>
                </div>
            </div>
        `;
        attachPanelToggle(stylePanel);
        scrollArea.appendChild(stylePanel);

        // 3. SEO, Meta Tags & Social Share Panel
        const seoPanel = document.createElement("div");
        seoPanel.className = "cad-panel open";
        const serpTitle = seoTitle || titleVal || "Untitled Page";
        const serpSlug = slugVal ? `/${slugVal.replace(/^\/+|\/+$/g, "")}/` : "/";
        const serpDesc = seoDesc || "Add a meta description below to control how this page appears in search engines and social shares.";
        seoPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-magnifying-glass-chart" style="color:var(--day-emerald);"></i> SEO & Social Sharing</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <div style="padding:9px 10px;border-radius:6px;background:var(--day-surface-inset);border:1px solid var(--day-rule);margin-bottom:8px;">
                    <div style="font-size:9.5px;font-family:var(--day-font-mono);color:var(--day-muted);text-transform:uppercase;margin-bottom:3px;">Search Engine Preview</div>
                    <div id="serp-prev-title" style="font-size:12.5px;font-weight:700;color:#60a5fa;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${serpTitle} &mdash; ${state.siteTitle || "Daypress"}</div>
                    <div id="serp-prev-url" style="font-size:10px;color:#34d399;font-family:var(--day-font-mono);margin:1px 0 3px;">${window.location.origin}${serpSlug}</div>
                    <div id="serp-prev-desc" style="font-size:11px;color:var(--day-muted);line-height:1.35;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;">${serpDesc}</div>
                </div>
                <div class="cad-row"><span class="cad-label">SEO Title</span><input type="text" class="cad-input" style="flex:1;" id="body-seo-title" value="${seoTitle}" placeholder="Custom SEO Title (max 60 chars)"></div>
                <div class="cad-row" style="flex-direction:column;align-items:stretch;gap:4px;">
                    <div style="display:flex;justify-content:space-between;"><span class="cad-label">Meta Description</span><span id="body-seo-desc-count" style="font-size:9.5px;font-family:var(--day-font-mono);color:var(--day-muted);">${seoDesc.length}/160</span></div>
                    <textarea class="cad-textarea" rows="3" style="width:100%;font-size:11px;" id="body-seo-desc" placeholder="Summary for search engines and OpenGraph cards...">${seoDesc}</textarea>
                </div>
                <div class="cad-row"><span class="cad-label">Focus KW</span><input type="text" class="cad-input" style="flex:1;" id="body-seo-kw" value="${focusKw}" placeholder="e.g. rust cms"></div>
                <div class="cad-row"><span class="cad-label">OG Image</span><input type="text" class="cad-input font-mono" style="flex:1;" id="body-seo-og-img" value="${ogImage}" placeholder="/uploads/social-card.png"></div>
                <div class="cad-row"><span class="cad-label">Canonical</span><input type="text" class="cad-input font-mono" style="flex:1;" id="body-seo-canonical" value="${canonicalUrl}" placeholder="https://..."></div>
                <div class="cad-row"><span class="cad-label">Noindex</span><label style="display:flex;align-items:center;gap:6px;font-size:11px;color:var(--day-ink);cursor:pointer;"><input type="checkbox" id="body-seo-noindex" ${noindex ? "checked" : ""}> Hide page from search engines</label></div>
            </div>
        `;
        attachPanelToggle(seoPanel);
        scrollArea.appendChild(seoPanel);

        // 4. Page Custom CSS Panel
        const cssPanel = document.createElement("div");
        cssPanel.className = "cad-panel";
        cssPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-code" style="color:var(--day-violet);"></i> Page Custom CSS</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <textarea class="cad-textarea font-mono" rows="5" style="width:100%;font-size:11px;" id="body-custom-css" placeholder="/* Custom CSS scoped to this page */">${(state.pageAst.custom_css || "").replace(/</g, "&lt;")}</textarea>
            </div>
        `;
        attachPanelToggle(cssPanel);
        scrollArea.appendChild(cssPanel);

        if (window.enhanceMediaInput) {
            const bgImgInp = stylePanel.querySelector("#body-bg-image");
            const ogImgInp = seoPanel.querySelector("#body-seo-og-img");
            if (bgImgInp) window.enhanceMediaInput(bgImgInp);
            if (ogImgInp) window.enhanceMediaInput(ogImgInp);
        }

        const markChanged = (metaChanged) => {
            if (metaChanged) state.pageMetaDirty = true;
            window.markDirty(true);
            applyRootBodyStylesToCanvas();
        };

        const titleInp = docPanel.querySelector("#body-doc-title");
        if (titleInp) {
            titleInp.oninput = () => {
                ctx.title = titleInp.value; data.title = titleInp.value;
                const lblTitle = document.getElementById("lbl-template-title");
                if (lblTitle) lblTitle.textContent = titleInp.value || "Untitled";
                const serpT = seoPanel.querySelector("#serp-prev-title");
                if (serpT && !data.seo_title) serpT.textContent = `${titleInp.value || "Untitled"} — ${state.siteTitle || "Daypress"}`;
                markChanged(true);
            };
        }
        const slugInp = docPanel.querySelector("#body-doc-slug");
        if (slugInp) {
            slugInp.oninput = () => {
                ctx.slug = slugInp.value.trim(); data.slug = ctx.slug;
                if (window.updateViewLiveLink) window.updateViewLiveLink();
                const serpU = seoPanel.querySelector("#serp-prev-url");
                if (serpU) serpU.textContent = `${window.location.origin}/${ctx.slug.replace(/^\/+|\/+$/g, "")}/`;
                markChanged(true);
            };
        }
        const statusSel = docPanel.querySelector("#body-doc-status");
        if (statusSel) statusSel.onchange = () => { data.status = statusSel.value; markChanged(true); };
        const tplSel = docPanel.querySelector("#body-doc-template");
        if (tplSel) tplSel.onchange = () => { data.page_template = tplSel.value; markChanged(true); };
        const condBtn = docPanel.querySelector("#body-btn-conditions");
        if (condBtn) {
            condBtn.onclick = () => {
                const tid = ctx.id || state.postId;
                if (window.openConditionsModal && tid) {
                    window.openConditionsModal(tid, ctx.title || "Template");
                }
            };
        }

        const bgSwatch = stylePanel.querySelector("#body-bg-swatch"), bgText = stylePanel.querySelector("#body-bg-text");
        const setBg = (val) => {
            styles.background_color = val;
            if (bgText && bgText.value !== val) bgText.value = val;
            if (bgSwatch) bgSwatch.value = /^#[0-9a-fA-F]{6}$/.test(val) ? val : "#ffffff";
            markChanged(false);
        };
        if (bgSwatch) bgSwatch.oninput = () => setBg(bgSwatch.value);
        if (bgText) bgText.oninput = () => setBg(bgText.value.trim());
        stylePanel.querySelector("#body-bg-clear").onclick = () => setBg("");
        stylePanel.querySelectorAll(".body-bg-preset").forEach(btn => { btn.onclick = () => setBg(btn.dataset.color); });

        const txtSwatch = stylePanel.querySelector("#body-txt-swatch"), txtText = stylePanel.querySelector("#body-txt-text");
        const setTxt = (val) => {
            styles.text_color = val;
            if (txtText && txtText.value !== val) txtText.value = val;
            if (txtSwatch) txtSwatch.value = /^#[0-9a-fA-F]{6}$/.test(val) ? val : "#111827";
            markChanged(false);
        };
        if (txtSwatch) txtSwatch.oninput = () => setTxt(txtSwatch.value);
        if (txtText) txtText.oninput = () => setTxt(txtText.value.trim());
        stylePanel.querySelector("#body-txt-clear").onclick = () => setTxt("");

        const fontSel = stylePanel.querySelector("#body-font-family");
        if (fontSel) fontSel.onchange = () => { styles.font_family = fontSel.value; markChanged(false); };
        stylePanel.querySelector("#body-font-clear").onclick = () => { styles.font_family = ""; if (fontSel) fontSel.value = ""; markChanged(false); };

        const bindStyleInput = (sel, key) => {
            const el = stylePanel.querySelector(sel);
            if (el) el.oninput = () => { styles[key] = el.value.trim(); markChanged(false); };
        };
        bindStyleInput("#body-bg-image", "background_image");
        bindStyleInput("#body-pad-top", "padding_top");
        bindStyleInput("#body-pad-bottom", "padding_bottom");
        bindStyleInput("#body-pad-left", "padding_left");
        bindStyleInput("#body-pad-right", "padding_right");

        stylePanel.querySelector("#body-bg-img-clear").onclick = () => { styles.background_image = ""; const el = stylePanel.querySelector("#body-bg-image"); if (el) el.value = ""; markChanged(false); };
        stylePanel.querySelector("#body-pad-tb-clear").onclick = () => { styles.padding_top = ""; styles.padding_bottom = ""; stylePanel.querySelector("#body-pad-top").value = ""; stylePanel.querySelector("#body-pad-bottom").value = ""; markChanged(false); };
        stylePanel.querySelector("#body-pad-lr-clear").onclick = () => { styles.padding_left = ""; styles.padding_right = ""; stylePanel.querySelector("#body-pad-left").value = ""; stylePanel.querySelector("#body-pad-right").value = ""; markChanged(false); };
        stylePanel.querySelector("#body-reset-all-styles").onclick = () => {
            ["background_color", "text_color", "font_family", "background_image", "padding_top", "padding_bottom", "padding_left", "padding_right"].forEach(k => delete styles[k]);
            markChanged(false);
            renderBodyPageInspector(inspector);
            if (window.showToast) window.showToast("Reset Body styles to theme defaults");
        };

        const seoTitleInp = seoPanel.querySelector("#body-seo-title");
        if (seoTitleInp) {
            seoTitleInp.oninput = () => {
                data.seo_title = seoTitleInp.value;
                const serpT = seoPanel.querySelector("#serp-prev-title");
                if (serpT) serpT.textContent = `${seoTitleInp.value || ctx.title || "Untitled"} — ${state.siteTitle || "Daypress"}`;
                markChanged(true);
            };
        }
        const seoDescInp = seoPanel.querySelector("#body-seo-desc");
        if (seoDescInp) {
            seoDescInp.oninput = () => {
                data.seo_description = seoDescInp.value;
                const cnt = seoPanel.querySelector("#body-seo-desc-count");
                if (cnt) cnt.textContent = `${seoDescInp.value.length}/160`;
                const serpD = seoPanel.querySelector("#serp-prev-desc");
                if (serpD) serpD.textContent = seoDescInp.value || "Add a meta description below...";
                markChanged(true);
            };
        }
        const bindSeoData = (sel, key, isCheck) => {
            const el = seoPanel.querySelector(sel);
            if (!el) return;
            el[isCheck ? "onchange" : "oninput"] = () => { data[key] = isCheck ? el.checked : el.value.trim(); markChanged(true); };
        };
        bindSeoData("#body-seo-kw", "focus_keyword", false);
        bindSeoData("#body-seo-og-img", "og_image", false);
        bindSeoData("#body-seo-canonical", "canonical_url", false);
        bindSeoData("#body-seo-noindex", "noindex", true);

        const cssInp = cssPanel.querySelector("#body-custom-css");
        if (cssInp) cssInp.oninput = () => { state.pageAst.custom_css = cssInp.value; markChanged(false); };

        scrollArea.scrollTop = keepScroll;
    }
    window.renderBodyPageInspector = renderBodyPageInspector;
})();
"###;
