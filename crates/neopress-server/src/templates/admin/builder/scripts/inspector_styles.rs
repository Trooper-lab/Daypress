//! Typography, Backgrounds, Borders & Effects CAD Style Panels (`scripts/inspector_styles.rs`).

pub const JS_INSPECTOR_STYLES: &str = r###"
(function() {
    function renderVisualPanels(container, node) {
        const styles = node.styles || {};

        // ---------------------------------------------------------
        // 1. TYPOGRAPHY PANEL
        // ---------------------------------------------------------
        const typoPanel = document.createElement("div");
        typoPanel.className = "cad-panel open";
        typoPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-font"></i> Typography</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row">
                    <span class="cad-label">Family</span>
                    <select class="cad-select" style="flex: 1;" data-prop="font_family">
                        <option value="Inter, sans-serif" ${styles.font_family && styles.font_family.includes('Inter') ? 'selected' : ''}>Inter (CAD)</option>
                        <option value="'JetBrains Mono', monospace" ${styles.font_family && styles.font_family.includes('Mono') ? 'selected' : ''}>JetBrains Mono</option>
                        <option value="'Plus Jakarta Sans', sans-serif" ${styles.font_family && styles.font_family.includes('Jakarta') ? 'selected' : ''}>Plus Jakarta</option>
                        <option value="system-ui, sans-serif">System UI</option>
                    </select>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Size</span>
                    <input type="text" class="cad-input font-mono" style="width: 70px;" data-prop="font_size" value="${styles.font_size || '15px'}">
                    <span class="cad-label">Weight</span>
                    <select class="cad-select" style="width: 80px;" data-prop="font_weight">
                        <option value="400" ${styles.font_weight === '400' ? 'selected' : ''}>400</option>
                        <option value="500" ${styles.font_weight === '500' ? 'selected' : ''}>500</option>
                        <option value="600" ${styles.font_weight === '600' ? 'selected' : ''}>600</option>
                        <option value="700" ${styles.font_weight === '700' ? 'selected' : ''}>700</option>
                    </select>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Color</span>
                    <div class="color-picker-control">
                        <input type="color" class="color-swatch-box" data-color-sync="text_color" value="${styles.text_color || '#F2F4F8'}">
                        <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="text_color" value="${styles.text_color || '#F2F4F8'}">
                    </div>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Align</span>
                    <div class="cad-segmented" data-prop="text_align">
                        <button class="seg-btn ${styles.text_align === 'left' ? 'active' : ''}" data-val="left"><i class="fa-solid fa-align-left"></i></button>
                        <button class="seg-btn ${styles.text_align === 'center' ? 'active' : ''}" data-val="center"><i class="fa-solid fa-align-center"></i></button>
                        <button class="seg-btn ${styles.text_align === 'right' ? 'active' : ''}" data-val="right"><i class="fa-solid fa-align-right"></i></button>
                    </div>
                </div>
            </div>
        `;
        attachToggle(typoPanel);
        attachListeners(typoPanel);
        container.appendChild(typoPanel);

        // ---------------------------------------------------------
        // 2. BACKGROUNDS & BORDERS PANEL
        // ---------------------------------------------------------
        const bgPanel = document.createElement("div");
        bgPanel.className = "cad-panel open";
        bgPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-brush"></i> Backgrounds & Borders</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row">
                    <span class="cad-label">Fill Color</span>
                    <div class="color-picker-control">
                        <input type="color" class="color-swatch-box" data-color-sync="background_color" value="${styles.background_color || '#ffffff'}">
                        <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="background_color" value="${styles.background_color || '#ffffff'}">
                    </div>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Radius</span>
                    <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="border_radius" value="${styles.border_radius || '0px'}">
                    <span class="cad-label">Border W</span>
                    <input type="text" class="cad-input font-mono" style="width: 60px;" data-prop="border_width" value="${styles.border_width || '0px'}">
                </div>
                <div class="cad-row">
                    <span class="cad-label">Border Color</span>
                    <div class="color-picker-control">
                        <input type="color" class="color-swatch-box" data-color-sync="border_color" value="${styles.border_color || '#242933'}">
                        <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="border_color" value="${styles.border_color || '#242933'}">
                    </div>
                </div>
            </div>
        `;
        attachToggle(bgPanel);
        attachListeners(bgPanel);
        container.appendChild(bgPanel);

        // ---------------------------------------------------------
        // 3. EFFECTS PANEL
        // ---------------------------------------------------------
        const fxPanel = document.createElement("div");
        fxPanel.className = "cad-panel open";
        fxPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-wand-magic"></i> Effects</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row">
                    <span class="cad-label">Opacity</span>
                    <input type="range" min="0" max="1" step="0.05" style="flex:1;" data-prop="opacity" value="${styles.opacity || 1}">
                    <span class="font-mono" style="font-size:11px; width:36px; text-align:right;">${Math.round((styles.opacity || 1) * 100)}%</span>
                </div>
                <div class="cad-row">
                    <span class="cad-label">Shadow</span>
                    <select class="cad-select" style="flex:1;" data-prop="box_shadow">
                        <option value="" ${!styles.box_shadow ? 'selected' : ''}>None</option>
                        <option value="0 4px 12px rgba(0,0,0,0.08)" ${styles.box_shadow && styles.box_shadow.includes('4px') ? 'selected' : ''}>Subtle Card</option>
                        <option value="0 12px 32px rgba(0,0,0,0.18)" ${styles.box_shadow && styles.box_shadow.includes('12px') ? 'selected' : ''}>Elevated Modal</option>
                        <option value="0 0 20px rgba(37,99,235,0.3)" ${styles.box_shadow && styles.box_shadow.includes('99,235') ? 'selected' : ''}>Cobalt Glow</option>
                    </select>
                </div>
            </div>
        `;
        attachToggle(fxPanel);
        attachListeners(fxPanel);
        container.appendChild(fxPanel);
    }
    window.renderVisualPanels = renderVisualPanels;

    function attachToggle(panel) {
        const h = panel.querySelector(".cad-panel-header");
        if (h) h.onclick = () => panel.classList.toggle("open");
    }

    function attachListeners(panel) {
        panel.querySelectorAll(".cad-segmented").forEach(group => {
            const prop = group.dataset.prop;
            group.querySelectorAll(".seg-btn").forEach(btn => {
                btn.onclick = () => {
                    group.querySelectorAll(".seg-btn").forEach(b => b.classList.remove("active"));
                    btn.classList.add("active");
                    window.setLocalStyle(prop, btn.dataset.val);
                };
            });
        });

        panel.querySelectorAll("input[data-prop], select[data-prop]").forEach(input => {
            const prop = input.dataset.prop;
            input.onchange = () => window.setLocalStyle(prop, input.value.trim());
        });

        // Color sync swatch with text input
        panel.querySelectorAll("input[data-color-sync]").forEach(swatch => {
            const prop = swatch.dataset.colorSync;
            swatch.oninput = () => {
                const textInput = panel.querySelector(`input[data-prop="${prop}"]`);
                if (textInput) textInput.value = swatch.value;
                window.setLocalStyle(prop, swatch.value);
            };
        });
    }
})();
"###;
