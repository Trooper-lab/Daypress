//! Typography, Backgrounds and Borders CAD Style Panels (`scripts/inspector_styles.rs`).

pub const JS_INSPECTOR_STYLES: &str = r###"
(function() {
    const SYSTEM_FONTS = [
        "system-ui, -apple-system, 'Segoe UI', sans-serif",
        "Georgia, 'Times New Roman', serif",
        "'Helvetica Neue', Arial, sans-serif",
        "ui-monospace, Menlo, Consolas, monospace"
    ];

    function fontOptions() {
        const theme = new Set();
        try {
            document.fonts.forEach(f => {
                const fam = String(f.family || "").replace(/["']/g, "").trim();
                if (fam && !/font awesome|fontawesome/i.test(fam)) theme.add(fam);
            });
        } catch (e) {}
        const all = new Set([...(window.NeoStyle.GOOGLE_FONTS || []), ...theme, ...SYSTEM_FONTS]);
        return Array.from(all).map(f => `<option value="${window.NSInspector.esc(f)}"></option>`).join("");
    }

    function renderVisualPanels(container, node) {
        const N = window.NSInspector;

        // ---------------------------------------------------------
        // 1. TYPOGRAPHY
        // ---------------------------------------------------------
        const typoBody = `
            <datalist id="ns-font-families">${fontOptions()}</datalist>
            ${N.row("Family", "font_family", N.input("font_family", "inherit", "ns-font-families"))}
            ${N.pair("Size", "font_size", N.input("font_size", "16px"), "Weight", "font_weight", N.select("font_weight", [
                ["100", "100 Thin"], ["200", "200 Extra Light"], ["300", "300 Light"], ["400", "400 Normal"], ["500", "500 Medium"],
                ["600", "600 Semi Bold"], ["700", "700 Bold"], ["800", "800 Extra Bold"], ["900", "900 Black"]
            ]))}
            ${N.pair("Height", "line_height", N.input("line_height", "1.5"), "Spacing", "letter_spacing", N.input("letter_spacing", "0px"))}
            ${N.row("Color", "text_color", N.color("text_color"))}
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
        N.bind(typoPanel, node);

        // ---------------------------------------------------------
        // 2. BACKGROUNDS
        // ---------------------------------------------------------
        const IMG = 'data-show-if="background_image!=none|"';
        const bgBody = `
            ${N.row("Color", "background_color", N.color("background_color"))}
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
            <div class="cad-row ns-presets">
                ${["0px", "4px", "8px", "16px", "9999px"].map(r => `<button type="button" class="ns-chip" data-radius="${r}" title="border-radius: ${r}">${r === "9999px" ? "Pill" : r}</button>`).join("")}
            </div>
        `;
        const borderPanel = N.panel(container, "border", "fa-vector-square", "Borders", borderBody);
        borderPanel.querySelectorAll("[data-radius]").forEach(b => {
            b.onclick = () => {
                window.setLocalStyle("border_radius", b.dataset.radius, { commit: true });
                N.refresh(borderPanel, node);
            };
        });
        N.bind(borderPanel, node);
    }
    window.renderVisualPanels = renderVisualPanels;
})();
"###;
