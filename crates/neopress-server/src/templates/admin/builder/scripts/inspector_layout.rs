//! Layout (Flexbox & CSS Grid, Position, Offsets) and Size Panels (`scripts/inspector_layout.rs`).

pub const JS_INSPECTOR_LAYOUT: &str = r###"
(function() {
    function renderLayoutPanels(container, node) {
        const styles = node.styles || {};

        // ---------------------------------------------------------
        // 1. LAYOUT PANEL (FLEXBOX / GRID / POSITION)
        // ---------------------------------------------------------
        const layoutPanel = document.createElement("div");
        layoutPanel.className = "cad-panel open";
        layoutPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-table-columns"></i> Layout</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <!-- Display -->
                <div class="cad-row">
                    <span class="cad-label">Display</span>
                    <div class="cad-segmented" data-prop="display">
                        <button class="seg-btn ${styles.display === 'block' ? 'active' : ''}" data-val="block">Block</button>
                        <button class="seg-btn ${(!styles.display || styles.display === 'flex') ? 'active' : ''}" data-val="flex">Flex</button>
                        <button class="seg-btn ${styles.display === 'grid' ? 'active' : ''}" data-val="grid">Grid</button>
                        <button class="seg-btn ${styles.display === 'none' ? 'active' : ''}" data-val="none">None</button>
                    </div>
                </div>

                <!-- Flex Direction -->
                <div class="cad-row flex-only">
                    <span class="cad-label">Direction</span>
                    <div class="cad-segmented" data-prop="flex_direction">
                        <button class="seg-btn ${(!styles.flex_direction || styles.flex_direction === 'row') ? 'active' : ''}" data-val="row" title="Row">&rarr;</button>
                        <button class="seg-btn ${styles.flex_direction === 'column' ? 'active' : ''}" data-val="column" title="Column">&darr;</button>
                        <button class="seg-btn ${styles.flex_direction === 'row-reverse' ? 'active' : ''}" data-val="row-reverse" title="Row Reverse">&larr;</button>
                        <button class="seg-btn ${styles.flex_direction === 'column-reverse' ? 'active' : ''}" data-val="column-reverse" title="Column Reverse">&uarr;</button>
                    </div>
                </div>

                <!-- Align Items -->
                <div class="cad-row flex-only">
                    <span class="cad-label">Align</span>
                    <div class="cad-segmented" data-prop="align_items">
                        <button class="seg-btn ${styles.align_items === 'flex-start' ? 'active' : ''}" data-val="flex-start">Start</button>
                        <button class="seg-btn ${(!styles.align_items || styles.align_items === 'center') ? 'active' : ''}" data-val="center">Center</button>
                        <button class="seg-btn ${styles.align_items === 'flex-end' ? 'active' : ''}" data-val="flex-end">End</button>
                        <button class="seg-btn ${styles.align_items === 'stretch' ? 'active' : ''}" data-val="stretch">Stretch</button>
                    </div>
                </div>

                <!-- Justify Content -->
                <div class="cad-row flex-only">
                    <span class="cad-label">Justify</span>
                    <div class="cad-segmented" data-prop="justify_content">
                        <button class="seg-btn ${styles.justify_content === 'flex-start' ? 'active' : ''}" data-val="flex-start">Start</button>
                        <button class="seg-btn ${styles.justify_content === 'center' ? 'active' : ''}" data-val="center">Center</button>
                        <button class="seg-btn ${styles.justify_content === 'space-between' ? 'active' : ''}" data-val="space-between">Between</button>
                    </div>
                </div>

                <!-- Gap -->
                <div class="cad-row">
                    <span class="cad-label">Gap</span>
                    <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="gap" value="${styles.gap || '16px'}">
                </div>

                <!-- Position -->
                <div class="cad-row">
                    <span class="cad-label">Position</span>
                    <select class="cad-select" style="width: 110px;" data-prop="position">
                        <option value="static" ${styles.position === 'static' ? 'selected' : ''}>Static</option>
                        <option value="relative" ${(!styles.position || styles.position === 'relative') ? 'selected' : ''}>Relative</option>
                        <option value="absolute" ${styles.position === 'absolute' ? 'selected' : ''}>Absolute</option>
                        <option value="sticky" ${styles.position === 'sticky' ? 'selected' : ''}>Sticky</option>
                    </select>
                </div>

                <!-- Z-Index & Overflow -->
                <div class="cad-row">
                    <span class="cad-label">Z-Index</span>
                    <input type="number" class="cad-input font-mono" style="width: 60px;" data-prop="z_index" value="${styles.z_index || 1}">
                    <span class="cad-label">Overflow</span>
                    <select class="cad-select" style="width: 80px;" data-prop="overflow">
                        <option value="visible" ${styles.overflow === 'visible' ? 'selected' : ''}>Visible</option>
                        <option value="hidden" ${styles.overflow === 'hidden' ? 'selected' : ''}>Hidden</option>
                        <option value="auto" ${styles.overflow === 'auto' ? 'selected' : ''}>Auto</option>
                    </select>
                </div>
            </div>
        `;
        attachPanelToggle(layoutPanel);
        attachPropertyListeners(layoutPanel);
        container.appendChild(layoutPanel);

        // ---------------------------------------------------------
        // 2. SIZE PANEL
        // ---------------------------------------------------------
        const sizePanel = document.createElement("div");
        sizePanel.className = "cad-panel open";
        sizePanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-up-right-and-down-left-from-center"></i> Size</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row">
                    <span class="cad-label">Width</span>
                    <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="width" value="${styles.width || '100%'}">
                    <span class="cad-label">Height</span>
                    <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="height" value="${styles.height || 'auto'}">
                </div>
                <div class="cad-row">
                    <span class="cad-label">Min-W</span>
                    <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="min_width" value="${styles.min_width || ''}" placeholder="0px">
                    <span class="cad-label">Max-W</span>
                    <input type="text" class="cad-input font-mono" style="width: 80px;" data-prop="max_width" value="${styles.max_width || ''}" placeholder="none">
                </div>
                <div class="cad-row">
                    <span class="cad-label">Aspect</span>
                    <select class="cad-select" style="width: 90px;" data-prop="aspect_ratio">
                        <option value="" ${!styles.aspect_ratio ? 'selected' : ''}>Auto</option>
                        <option value="1/1" ${styles.aspect_ratio === '1/1' ? 'selected' : ''}>1:1</option>
                        <option value="16/9" ${styles.aspect_ratio === '16/9' ? 'selected' : ''}>16:9</option>
                        <option value="4/3" ${styles.aspect_ratio === '4/3' ? 'selected' : ''}>4:3</option>
                    </select>
                    <span class="cad-label">Fit</span>
                    <select class="cad-select" style="width: 80px;" data-prop="object_fit">
                        <option value="cover" ${styles.object_fit === 'cover' ? 'selected' : ''}>Cover</option>
                        <option value="contain" ${styles.object_fit === 'contain' ? 'selected' : ''}>Contain</option>
                    </select>
                </div>
            </div>
        `;
        attachPanelToggle(sizePanel);
        attachPropertyListeners(sizePanel);
        container.appendChild(sizePanel);
    }
    window.renderLayoutPanels = renderLayoutPanels;

    function attachPanelToggle(panel) {
        const header = panel.querySelector(".cad-panel-header");
        if (header) {
            header.onclick = () => panel.classList.toggle("open");
        }
    }

    function attachPropertyListeners(panel) {
        // Segmented buttons
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

        // Inputs & Selects
        panel.querySelectorAll("input[data-prop], select[data-prop]").forEach(input => {
            const prop = input.dataset.prop;
            input.onchange = () => {
                window.setLocalStyle(prop, input.value.trim());
            };
        });
    }
})();
"###;
