//! Artboard WYSIWYG DOM Builder for Core Containers and Basic Widgets (`scripts/canvas_render.rs`).

pub const JS_CANVAS_RENDER: &str = r###"
(function() {
    function renderCanvas() {
        const canvas = document.getElementById("neostudio-canvas");
        if (!canvas) return;
        canvas.innerHTML = "";
        if (window.resetStateRules) window.resetStateRules();

        const state = window.NeoStudioState;
        if (!state.pageAst || !state.pageAst.root) return;

        // Render recursive tree
        for (const child of (state.pageAst.root.children || [])) {
            const childEl = buildDomNode(child);
            if (childEl) canvas.appendChild(childEl);
        }

        // Re-synchronize on-canvas selection chrome and forced pseudo-state preview
        if (window.applyForcedState) window.applyForcedState();
        if (window.updateSelectionChrome) {
            window.updateSelectionChrome();
        }
    }
    window.renderCanvas = renderCanvas;

    function buildDomNode(node) {
        if (!node) return null;
        let el = null;

        // Delegate to Pro Block builder if applicable
        if (window.isProWidget && window.isProWidget(node.type)) {
            el = window.renderProWidget(node);
        } else {
            el = buildStandardNode(node);
        }

        if (!el) return null;

        el.dataset.nodeId = node.id;
        el.classList.add("neostudio-node", "node-" + (node.type || "unknown").toLowerCase());

        // Apply styles (base + responsive overrides + pseudo-state rules) via the Style Engine
        if (window.applyNodeStyles) window.applyNodeStyles(el, node);

        // Make canvas node draggable & drop target
        const isLocked = node.data && node.data._locked;
        if (node.id !== "root" && !isLocked) {
            el.draggable = true;
            el.setAttribute("draggable", "true");
        }

        el.addEventListener("dragstart", (e) => {
            if (isLocked || node.id === "root") {
                e.preventDefault();
                return;
            }
            e.stopPropagation();
            window._draggedNavNodeId = node.id;
            window._draggedPaletteType = null;
            e.dataTransfer.setData("text/plain", node.id);
            e.dataTransfer.setData("application/x-neostudio-node", node.id);
            e.dataTransfer.effectAllowed = "move";
            el.classList.add("is-dragging");
        });

        el.addEventListener("dragend", (e) => {
            e.stopPropagation();
            el.classList.remove("is-dragging");
            window._draggedNavNodeId = null;
            if (window.hideDropIndicator) window.hideDropIndicator();
        });

        el.addEventListener("dragover", (e) => {
            e.preventDefault();
            e.stopPropagation();
            const draggedNodeId = window._draggedNavNodeId;
            if (draggedNodeId && (draggedNodeId === node.id || (window.isDescendant && window.isDescendant(draggedNodeId, node.id)))) {
                e.dataTransfer.dropEffect = "none";
                return;
            }
            e.dataTransfer.dropEffect = window._draggedPaletteType ? "copy" : "move";
            const rect = el.getBoundingClientRect();
            const relY = e.clientY - rect.top;
            const isContainer = ["Section", "Container", "Column", "Grid", "Flex"].includes(node.type);
            let pos = relY < rect.height * 0.5 ? "before" : "after";
            if (isContainer && relY >= rect.height * 0.25 && relY <= rect.height * 0.75) {
                pos = "inside";
            }
            el.dataset.dropPos = pos;
            if (window.showDropIndicator) window.showDropIndicator(el, pos);
        });

        el.addEventListener("dragleave", (e) => {
            e.stopPropagation();
            delete el.dataset.dropPos;
        });

        el.addEventListener("drop", (e) => {
            e.preventDefault();
            e.stopPropagation();
            const pos = el.dataset.dropPos || "after";
            delete el.dataset.dropPos;
            if (window.hideDropIndicator) window.hideDropIndicator();

            const rawText = e.dataTransfer ? e.dataTransfer.getData("text/plain") : "";
            const paletteType = window._draggedPaletteType
                || (e.dataTransfer && e.dataTransfer.getData("application/x-neostudio-palette"))
                || (rawText.startsWith("palette:") ? rawText.slice(8) : null);

            if (paletteType) {
                window._draggedPaletteType = null;
                if (window.insertWidgetRelativeTo) {
                    window.insertWidgetRelativeTo(paletteType, node.id, pos);
                } else if (window.insertWidgetFromPalette) {
                    window.insertWidgetFromPalette(paletteType);
                }
                return;
            }

            const draggedNodeId = window._draggedNavNodeId || rawText;
            if (draggedNodeId && draggedNodeId !== node.id && window.moveNode) {
                window._draggedNavNodeId = null;
                window.moveNode(draggedNodeId, node.id, pos);
            }
        });

        // Click-to-select (and prevent anchor navigation inside canvas)
        el.addEventListener("click", (e) => {
            if (el.tagName === "A" || e.target.closest("a")) {
                e.preventDefault();
            }
            e.stopPropagation();
            if (window.selectElement) window.selectElement(node.id);
        });

        // Double-click inline text editing for Heading, Text, Button, and CustomHtml inner elements
        if (["Heading", "Text", "Button"].includes(node.type) && !isLocked) {
            el.addEventListener("dblclick", (e) => {
                e.stopPropagation();
                el.contentEditable = "true";
                el.focus();
                const onBlur = () => {
                    el.removeAttribute("contenteditable");
                    el.removeEventListener("blur", onBlur);
                    if (!node.data) node.data = {};
                    if (node.type === "Text") {
                        node.data.content_html = el.innerHTML;
                    } else {
                        node.data.text = el.innerText.trim();
                    }
                    window.pushHistorySnapshot();
                    if (window.renderInspector) window.renderInspector();
                };
                el.addEventListener("blur", onBlur);
            });
        } else if (node.type === "CustomHtml" && !isLocked) {
            el.addEventListener("dblclick", (e) => {
                const target = e.target.closest("h1, h2, h3, h4, h5, h6, p, li, a.btn, a.wp-block-button__link, blockquote");
                if (!target || !el.contains(target)) return;
                e.stopPropagation();
                target.contentEditable = "true";
                target.style.outline = "2px dashed #3b82f6";
                target.focus();
                const onBlur = () => {
                    target.removeAttribute("contenteditable");
                    target.style.outline = "";
                    target.removeEventListener("blur", onBlur);
                    if (!node.data) node.data = {};
                    const clone = el.cloneNode(true);
                    clone.classList.remove("neostudio-node", "node-customhtml", "is-selected", "is-hovered", "is-dragging",
                        "neostudio-hidden-bp", "neostudio-force-hover", "neostudio-force-focus", "neostudio-force-active");
                    // Strip builder-applied classes & inline styles: they live in node.styles, not in the raw markup
                    (el.dataset.neostudioClasses || "").split(/\s+/).filter(Boolean).forEach(c => clone.classList.remove(c));
                    const baseStyle = el.dataset.neostudioBaseStyle || "";
                    if (baseStyle) clone.setAttribute("style", baseStyle); else clone.removeAttribute("style");
                    clone.removeAttribute("data-neostudio-base-style");
                    clone.removeAttribute("data-neostudio-classes");
                    if (!clone.className) clone.removeAttribute("class");
                    clone.removeAttribute("data-node-id");
                    clone.removeAttribute("draggable");
                    node.data.raw = clone.outerHTML;
                    window.markDirty(true);
                    window.pushHistorySnapshot();
                    if (window.renderInspector) window.renderInspector();
                    if (window.updateNavigator) window.updateNavigator();
                };
                target.addEventListener("blur", onBlur);
            });
        }

        // Hover overlay feedback
        el.addEventListener("mouseenter", (e) => {
            e.stopPropagation();
            if (window.onNodeHover) window.onNodeHover(node.id);
        });

        return el;
    }
    window.buildDomNode = buildDomNode;

    function buildStandardNode(node) {
        let el = null;
        const type = node.type || "Container";

        switch(type) {
            case "Section": {
                el = document.createElement("section");
                el.className = "np-section";
                renderChildren(el, node);
                break;
            }
            case "Container": {
                el = document.createElement("div");
                el.className = "np-container";
                renderChildren(el, node);
                break;
            }
            case "Column": {
                el = document.createElement("div");
                el.className = "elementor-column";
                renderChildren(el, node);
                break;
            }
            case "Grid": {
                el = document.createElement("div");
                el.className = "elementor-grid";
                renderChildren(el, node);
                break;
            }
            case "Flex": {
                el = document.createElement("div");
                el.className = "elementor-flex";
                renderChildren(el, node);
                break;
            }
            case "Heading": {
                const level = (node.data && node.data.level) || 2;
                el = document.createElement("h" + level);
                el.className = "np-heading";
                el.textContent = (node.data && node.data.text) || "Heading Text";
                break;
            }
            case "Text": {
                el = document.createElement("div");
                el.className = "np-text";
                el.innerHTML = (node.data && node.data.content_html) || "<p>Text content block...</p>";
                break;
            }
            case "Button": {
                el = document.createElement("a");
                el.className = "np-button";
                el.href = (node.data && node.data.url) || "#";
                const icon = node.data && node.data.icon ? `<i class="${node.data.icon}"></i> ` : "";
                el.innerHTML = `${icon}<span>${(node.data && node.data.text) || "Button Action"}</span>`;
                break;
            }
            case "Image": {
                // Rendered as a bare <img> exactly like public SSR, so size/radius/object-fit match 1:1
                el = document.createElement("img");
                el.className = "np-image";
                el.src = (node.data && node.data.src) || "/static/neopress-icon.svg";
                el.alt = (node.data && node.data.alt) || "Image";
                break;
            }
            case "Video": {
                el = document.createElement("div");
                el.className = "np-video";
                const url = (node.data && node.data.url) || "https://www.youtube.com/embed/dQw4w9WgXcQ";
                el.innerHTML = `<iframe src="${url}" style="width:100%; aspect-ratio: 16/9; border:none; border-radius:8px;" allowfullscreen></iframe>`;
                break;
            }
            case "Spacer": {
                el = document.createElement("div");
                el.className = "np-spacer";
                const h = (node.data && node.data.height) || 48;
                el.style.height = (typeof h === "number" ? h + "px" : h);
                break;
            }
            case "Divider": {
                el = document.createElement("div");
                el.className = "np-divider";
                el.style.borderTop = "1px solid #e2e8f0";
                el.style.margin = "20px 0";
                break;
            }
            case "Icon": {
                el = document.createElement("div");
                el.className = "np-icon";
                const iconClass = (node.data && node.data.name) || "fa-solid fa-star";
                el.innerHTML = `<i class="${iconClass}"></i>`;
                break;
            }
            case "CustomHtml": {
                const rawHtml = (node.data && node.data.raw) || "<section class=\"section white\"><div class=\"container\"><h2>ACF Section</h2></div></section>";
                const tpl = document.createElement("template");
                tpl.innerHTML = rawHtml.trim();
                if (tpl.content.children.length === 1) {
                    el = tpl.content.firstElementChild;
                } else {
                    el = document.createElement("div");
                    el.innerHTML = rawHtml;
                }
                el.querySelectorAll("[data-aos]").forEach(a => a.classList.add("aos-animate"));
                break;
            }
            default: {
                el = document.createElement("div");
                renderChildren(el, node);
            }
        }
        return el;
    }

    function renderChildren(el, node) {
        if (!node.children || !Array.isArray(node.children)) return;
        for (const child of node.children) {
            const childDom = buildDomNode(child);
            if (childDom) el.appendChild(childDom);
        }
    }
})();
"###;
