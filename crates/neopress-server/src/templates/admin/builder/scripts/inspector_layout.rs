//! Layout (Display, Flexbox, CSS Grid, child placement), Position and Size Panels
//! (`scripts/inspector_layout.rs`).

pub const JS_INSPECTOR_LAYOUT: &str = r###"
(function() {
    function renderLayoutPanels(container, node) {
        const N = window.NSInspector;
        const parent = window.findParentNode ? window.findParentNode(node.id) : null;
        const parentEl = parent ? N.canvasEl(parent) : (N.canvasEl(node) || {}).parentElement;
        const parentDisplay = parentEl && parentEl.nodeType === 1 ? getComputedStyle(parentEl).display : "";
        const inFlex = /flex/.test(parentDisplay);
        const inGrid = /grid/.test(parentDisplay);
        const FLEX = 'data-show-if="display=flex|inline-flex"';
        const GRID = 'data-show-if="display=grid|inline-grid"';
        const FLEX_OR_GRID = 'data-show-if="display=flex|inline-flex|grid|inline-grid"';

        // ---------------------------------------------------------
        // 1. LAYOUT
        // ---------------------------------------------------------
        const layoutBody = `
            ${N.row("Display", "display", N.seg("display", [
                ["block", "Block"], ["flex", "Flex"], ["grid", "Grid"], ["inline-block", "Inline", "inline-block"], ["none", '<i class="fa-solid fa-eye-slash"></i>', "none (hidden)"]
            ]))}
            ${N.row("Direction", "flex_direction", N.seg("flex_direction", [
                ["row", "&rarr;", "row"], ["column", "&darr;", "column"], ["row-reverse", "&larr;", "row-reverse"], ["column-reverse", "&uarr;", "column-reverse"]
            ]), FLEX)}
            ${N.row("Align", "align_items", N.seg("align_items", [
                ["flex-start", "Start"], ["center", "Center"], ["flex-end", "End"], ["stretch", "Stretch"], ["baseline", "Base", "baseline"]
            ]), FLEX_OR_GRID)}
            ${N.row("Justify", "justify_content", N.select("justify_content", [
                ["flex-start", "Start"], ["center", "Center"], ["flex-end", "End"], ["space-between", "Space between"],
                ["space-around", "Space around"], ["space-evenly", "Space evenly"], ["stretch", "Stretch"]
            ]), FLEX_OR_GRID)}
            ${N.row("Wrap", "flex_wrap", N.seg("flex_wrap", [["nowrap", "No wrap"], ["wrap", "Wrap"], ["wrap-reverse", "Reverse", "wrap-reverse"]]), FLEX)}
            ${N.row("Columns", "grid_template_columns", N.input("grid_template_columns", "e.g. repeat(3, 1fr)"), GRID)}
            <div class="cad-row ns-presets" ${GRID}>
                ${[1, 2, 3, 4, 6].map(n => `<button type="button" class="ns-chip" data-grid-cols="${n}">${n} col</button>`).join("")}
            </div>
            ${N.row("Rows", "grid_template_rows", N.input("grid_template_rows", "auto"), GRID)}
            ${N.row("Flow", "grid_auto_flow", N.select("grid_auto_flow", ["row", "column", "dense", "row dense", "column dense"]), GRID)}
            ${N.row("Items X", "justify_items", N.seg("justify_items", [["start", "Start"], ["center", "Center"], ["end", "End"], ["stretch", "Stretch"]]), GRID)}
            ${N.row("Gap", "gap", N.input("gap", "0px"), FLEX_OR_GRID)}
        `;
        const layoutPanel = N.panel(container, "layout", "fa-table-columns", "Layout", layoutBody);
        layoutPanel.querySelectorAll("[data-grid-cols]").forEach(b => {
            b.onclick = () => {
                const n = +b.dataset.gridCols;
                window.setLocalStyle("grid_template_columns", n === 1 ? "1fr" : `repeat(${n}, minmax(0, 1fr))`, { commit: true });
                N.refresh(layoutPanel, node);
            };
        });
        N.bind(layoutPanel, node);

        // ---------------------------------------------------------
        // 2. CHILD PLACEMENT (only when the parent is flex or grid)
        // ---------------------------------------------------------
        if (inFlex || inGrid) {
            const childBody = `
                ${N.row("Align self", "align_self", N.seg("align_self", [
                    ["auto", "Auto"], ["flex-start", "Start"], ["center", "Center"], ["flex-end", "End"], ["stretch", "Stretch"]
                ]))}
                ${inFlex ? `
                    ${N.row("Sizing", "flex", N.seg("flex", [
                        ["0 1 auto", "Shrink", "0 1 auto (don't grow)"], ["1 1 0%", "Grow", "1 1 0% (fill space)"], ["0 0 auto", "None", "0 0 auto (fixed)"]
                    ]))}
                    ${N.row("Flex", "flex", N.input("flex", "grow shrink basis"))}` : ""}
                ${inGrid ? N.pair("Col", "grid_column", N.input("grid_column", "span 2"), "Row", "grid_row", N.input("grid_row", "auto")) : ""}
            `;
            const childPanel = N.panel(container, "gridchild", "fa-object-ungroup", inGrid ? "Grid Child" : "Flex Child", childBody);
            N.bind(childPanel, node);
        }

        // ---------------------------------------------------------
        // 3. POSITION
        // ---------------------------------------------------------
        const OFFSETS = 'data-show-if="position!=static"';
        const posBody = `
            ${N.row("Position", "position", N.select("position", [
                ["static", "Static"], ["relative", "Relative"], ["absolute", "Absolute"], ["fixed", "Fixed"], ["sticky", "Sticky"]
            ]))}
            ${N.pair("Top", "top", N.input("top", "auto"), "Right", "right", N.input("right", "auto"), OFFSETS)}
            ${N.pair("Bottom", "bottom", N.input("bottom", "auto"), "Left", "left", N.input("left", "auto"), OFFSETS)}
            <div class="cad-row ns-presets" ${OFFSETS}>
                <button type="button" class="ns-chip" data-pin="fill" title="top/right/bottom/left: 0">Fill</button>
                <button type="button" class="ns-chip" data-pin="top" title="Pin to top edge">Top</button>
                <button type="button" class="ns-chip" data-pin="clear" title="Clear offsets">Clear</button>
            </div>
            ${N.pair("Z-Index", "z_index", N.input("z_index", "auto"), "Overflow", "overflow", N.select("overflow", [
                ["visible", "Visible"], ["hidden", "Hidden"], ["auto", "Auto"], ["scroll", "Scroll"], ["clip", "Clip"]
            ]))}
        `;
        const posPanel = N.panel(container, "position", "fa-crosshairs", "Position", posBody);
        posPanel.querySelectorAll("[data-pin]").forEach(b => {
            b.onclick = () => {
                const presets = {
                    fill: { top: "0px", right: "0px", bottom: "0px", left: "0px" },
                    top: { top: "0px", right: "0px", bottom: "", left: "0px" },
                    clear: { top: "", right: "", bottom: "", left: "" }
                };
                window.setLocalStyles(presets[b.dataset.pin], { commit: true });
                N.refresh(posPanel, node);
            };
        });
        N.bind(posPanel, node);

        // ---------------------------------------------------------
        // 4. SIZE
        // ---------------------------------------------------------
        const sizeBody = `
            ${N.pair("Width", "width", N.input("width", "auto"), "Height", "height", N.input("height", "auto"))}
            ${N.pair("Min W", "min_width", N.input("min_width", "0px"), "Min H", "min_height", N.input("min_height", "0px"))}
            ${N.pair("Max W", "max_width", N.input("max_width", "none"), "Max H", "max_height", N.input("max_height", "none"))}
            <div class="cad-row ns-presets">
                <button type="button" class="ns-chip" data-size="full" title="width: 100%">100% W</button>
                <button type="button" class="ns-chip" data-size="fit" title="width: fit-content">Fit</button>
                <button type="button" class="ns-chip" data-size="screen" title="min-height: 100vh">Full screen</button>
                <button type="button" class="ns-chip" data-size="boxed" title="max-width: 1200px; centered">Boxed</button>
            </div>
            ${N.pair("Aspect", "aspect_ratio", N.select("aspect_ratio", [
                ["auto", "Auto"], ["1 / 1", "1:1"], ["4 / 3", "4:3"], ["3 / 2", "3:2"], ["16 / 9", "16:9"], ["21 / 9", "21:9"], ["3 / 4", "3:4"], ["9 / 16", "9:16"]
            ]), "Fit", "object_fit", N.select("object_fit", [
                ["fill", "Fill"], ["cover", "Cover"], ["contain", "Contain"], ["none", "None"], ["scale-down", "Scale down"]
            ]))}
        `;
        const sizePanel = N.panel(container, "size", "fa-up-right-and-down-left-from-center", "Size", sizeBody);
        sizePanel.querySelectorAll("[data-size]").forEach(b => {
            b.onclick = () => {
                const presets = {
                    full: { width: "100%" },
                    fit: { width: "fit-content" },
                    screen: { min_height: "100vh" },
                    boxed: { max_width: "1200px", margin_left: "auto", margin_right: "auto", width: "100%" }
                };
                window.setLocalStyles(presets[b.dataset.size], { commit: true });
                N.refresh(sizePanel, node);
                if (window.repaintBoxModel) window.repaintBoxModel();
            };
        });
        N.bind(sizePanel, node);
    }
    window.renderLayoutPanels = renderLayoutPanels;
})();
"###;
