//! Effects (Opacity, Shadows, Transforms, Transitions, Filters, Cursor) and per-element
//! Custom CSS Panels (`scripts/inspector_effects.rs`).

pub const JS_INSPECTOR_EFFECTS: &str = r###"
(function() {
    const SHADOWS = [
        ["0 1px 2px rgba(0,0,0,0.06)", "XS"],
        ["0 4px 12px rgba(0,0,0,0.08)", "Card"],
        ["0 12px 32px rgba(0,0,0,0.18)", "Elevated"],
        ["0 24px 60px rgba(0,0,0,0.28)", "Modal"],
        ["0 0 20px rgba(37,99,235,0.35)", "Glow"],
        ["inset 0 2px 6px rgba(0,0,0,0.15)", "Inset"]
    ];

    function renderEffectsPanels(container, node) {
        const N = window.NSInspector;
        const layer = window.getEditLayerKey ? window.getEditLayerKey() : null;
        const isState = layer === "hover" || layer === "focus" || layer === "active";

        const fxBody = `
            ${N.row("Opacity", "opacity", N.range("opacity", 0, 1, 0.01))}
            ${N.row("Shadow", "box_shadow", N.input("box_shadow", "none"))}
            <div class="cad-row ns-presets">
                ${SHADOWS.map(([v, l]) => `<button type="button" class="ns-chip" data-shadow="${v}" title="${v}">${l}</button>`).join("")}
                <button type="button" class="ns-chip" data-shadow="" title="Remove shadow">None</button>
            </div>
            ${N.row("Transform", "transform", N.input("transform", "none"))}
            <div class="cad-row ns-presets">
                <button type="button" class="ns-chip" data-transform="translateY(-4px)">Lift</button>
                <button type="button" class="ns-chip" data-transform="scale(1.05)">Grow</button>
                <button type="button" class="ns-chip" data-transform="rotate(-2deg)">Tilt</button>
                <button type="button" class="ns-chip" data-transform="">None</button>
            </div>
            ${N.row("Transition", "transition", N.input("transition", "all 0.2s ease"))}
            ${N.row("Filter", "filter", N.input("filter", "none"))}
            ${N.row("Backdrop", "backdrop_filter", N.input("backdrop_filter", "blur(8px)"))}
            ${N.row("Cursor", "cursor", N.select("cursor", ["auto", "default", "pointer", "text", "move", "grab", "not-allowed", "zoom-in"]))}
            ${isState ? `<div class="ns-hint"><i class="fa-solid fa-circle-info"></i> Tip: set a <b>Transition</b> on the Default state so ${layer} changes animate smoothly.</div>` : ""}
        `;
        const fxPanel = N.panel(container, "effects", "fa-wand-magic-sparkles", "Effects", fxBody);
        fxPanel.querySelectorAll("[data-shadow]").forEach(b => {
            b.onclick = () => { window.setLocalStyle("box_shadow", b.dataset.shadow, { commit: true }); N.refresh(fxPanel, node); };
        });
        fxPanel.querySelectorAll("[data-transform]").forEach(b => {
            b.onclick = () => { window.setLocalStyle("transform", b.dataset.transform, { commit: true }); N.refresh(fxPanel, node); };
        });
        N.bind(fxPanel, node);

        const cssBody = `
            ${N.textarea("custom_css", "e.g.\nmix-blend-mode: multiply;\nclip-path: circle(50%);")}
            <div class="ns-hint">Declarations only (no selectors). Applied last on this layer, on the canvas and the live site.</div>
        `;
        const cssPanel = N.panel(container, "customcss", "fa-code", "Custom CSS", cssBody);
        N.bind(cssPanel, node);
    }
    window.renderEffectsPanels = renderEffectsPanels;
})();
"###;
