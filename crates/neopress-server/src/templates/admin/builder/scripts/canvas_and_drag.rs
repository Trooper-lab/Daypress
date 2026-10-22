//! Interactive Visual Builder canvas DOM renderer and drag-and-drop handlers.

pub const SCRIPT_CANVAS_AND_DRAG: &str = r###"    // -------------------------------------------------------------
    // RENDER CANVAS (WYSIWYG DOM WITH HOVER & EDIT HANDLES)
    // -------------------------------------------------------------
    function renderCanvas() {
        const canvas = document.getElementById("elementor-canvas");
        canvas.innerHTML = "";
        if (!pageAst || !pageAst.root) return;

        function attachSectionHandleAndDrag(el, node) {
            el.classList.add("elementor-section");
            if (node.id === activeElementId) {
                el.classList.add("active-element");
            }
            if (!el.style.position) {
                el.style.position = "relative";
            }

            const handle = document.createElement("div");
            handle.className = "elementor-section-handle";
            handle.innerHTML = `
                <button class="sec-handle-btn sec-add" title="Add Section Above"><i class="fa-solid fa-plus"></i></button>
                <button class="sec-handle-btn sec-drag" title="Drag to Reorder Section" draggable="true"><i class="fa-solid fa-grip-vertical"></i></button>
                <button class="sec-handle-btn sec-del" title="Delete Section"><i class="fa-solid fa-xmark"></i></button>
            `;

            handle.querySelector(".sec-add").onclick = (e) => {
                e.stopPropagation();
                insertSectionAbove(node.id);
            };
            handle.querySelector(".sec-del").onclick = (e) => {
                e.stopPropagation();
                deleteElement(node.id);
            };

            const secDragBtn = handle.querySelector(".sec-drag");
            secDragBtn.ondragstart = (e) => {
                e.stopPropagation();
                window._activeDragData = { action: "move_section", id: node.id };
                e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                e.dataTransfer.effectAllowed = "move";
                el.classList.add("is-dragging");
            };
            secDragBtn.ondragend = () => {
                el.classList.remove("is-dragging");
                window._activeDragData = null;
                hideDropIndicator();
            };

            el.appendChild(handle);

            el.ondragover = (e) => {
                const data = getDragData(e);
                if (!data) return;

                if (data.action === "move_section") {
                    if (data.id === node.id) return;
                    e.preventDefault();
                    e.stopPropagation();
                    const rect = el.getBoundingClientRect();
                    const midY = rect.top + rect.height / 2;
                    const pos = e.clientY < midY ? "before" : "after";
                    showDropIndicator(el, pos);
                    return;
                }

                if (data.type === "new_widget" || data.action === "move_widget") {
                    const rect = el.getBoundingClientRect();
                    if (e.clientY < rect.top + 35) {
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                        showDropIndicator(el, "before");
                    } else if (e.clientY > rect.bottom - 35) {
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                        showDropIndicator(el, "after");
                    } else {
                        e.preventDefault();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                    }
                }
            };

            el.ondragleave = (e) => {
                if (!el.contains(e.relatedTarget)) {
                    hideDropIndicator();
                }
            };

            el.ondrop = (e) => {
                const data = getDragData(e);
                if (!data) return;

                e.preventDefault();
                e.stopPropagation();
                hideDropIndicator();

                if (data.action === "move_section" && data.id !== node.id) {
                    const rect = el.getBoundingClientRect();
                    const midY = rect.top + rect.height / 2;
                    const pos = e.clientY < midY ? "before" : "after";
                    executeSectionMove(data.id, node.id, pos);
                    return;
                }

                if (data.type === "new_widget" || data.action === "move_widget") {
                    const rect = el.getBoundingClientRect();
                    if (e.clientY < rect.top + 30) {
                        insertNewSectionNear(node.id, "before", data);
                    } else if (e.clientY > rect.bottom - 30) {
                        insertNewSectionNear(node.id, "after", data);
                    } else {
                        const targetCol = findFirstColumn(node) || node;
                        if (targetCol) {
                            handleDropOnContainer(targetCol.id, data);
                        } else {
                            insertNewSectionNear(node.id, "after", data);
                        }
                    }
                }
            };

            el.onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };
        }

        function buildDomNode(node, isTopLevel) {
            const el = document.createElement("div");
            el.id = node.id;
            el.dataset.nodeId = node.id;

            // Styles mapping (supports both flat and nested AST StyleProperties)
            if (node.styles) {
                const s = node.styles;
                const isBtn = node.type === "Button";
                const bgCol = s.background_color || (s.background && s.background.color);
                if (bgCol && !isBtn) {
                    el.style.backgroundColor = bgCol;
                }
                const bgImg = s.background_image || (s.background && s.background.image_url);
                if (bgImg && !isBtn) {
                    el.style.backgroundImage = `url('${bgImg}')`;
                    el.style.backgroundRepeat = "no-repeat";
                }
                const bgPos = s.background_position || (s.background && s.background.position);
                if (bgPos && !isBtn) {
                    el.style.backgroundPosition = bgPos;
                }
                const bgSize = s.background_size || (s.background && s.background.size);
                if (bgSize && !isBtn) {
                    el.style.backgroundSize = bgSize;
                }
                if (s.background_gradient && !isBtn) {
                    el.style.backgroundImage = s.background_gradient;
                }

                const textCol = s.text_color || (s.typography && s.typography.color);
                if (textCol && !isBtn) el.style.color = textCol;
                const fontFam = s.font_family || (s.typography && s.typography.font_family);
                if (fontFam && !isBtn) el.style.fontFamily = fontFam;
                const fontSz = s.font_size || (s.typography && s.typography.font_size);
                if (fontSz && !isBtn) el.style.fontSize = fontSz;
                const fontWt = s.font_weight || (s.typography && s.typography.font_weight);
                if (fontWt && !isBtn) el.style.fontWeight = fontWt;
                const lineHt = s.line_height || (s.typography && s.typography.line_height);
                if (lineHt && !isBtn) el.style.lineHeight = lineHt;
                const letSp = s.letter_spacing || (s.typography && s.typography.letter_spacing);
                if (letSp && !isBtn) el.style.letterSpacing = letSp;
                const txtTr = s.text_transform || (s.typography && s.typography.text_transform);
                if (txtTr && !isBtn) el.style.textTransform = txtTr;
                const txtAl = s.text_align || (s.typography && s.typography.align);
                if (txtAl) el.style.textAlign = txtAl;

                if (!isBtn) {
                    const padT = s.padding_top || (s.padding && s.padding.top !== undefined ? `${s.padding.top}${s.padding.unit || 'px'}` : null);
                    const padR = s.padding_right || (s.padding && s.padding.right !== undefined ? `${s.padding.right}${s.padding.unit || 'px'}` : null);
                    const padB = s.padding_bottom || (s.padding && s.padding.bottom !== undefined ? `${s.padding.bottom}${s.padding.unit || 'px'}` : null);
                    const padL = s.padding_left || (s.padding && s.padding.left !== undefined ? `${s.padding.left}${s.padding.unit || 'px'}` : null);
                    if (padT) el.style.paddingTop = padT;
                    if (padR) el.style.paddingRight = padR;
                    if (padB) el.style.paddingBottom = padB;
                    if (padL) el.style.paddingLeft = padL;

                    const bRad = s.border_radius || (s.border && s.border.radius);
                    if (bRad) el.style.borderRadius = bRad;
                    const bWidth = s.border_width || (s.border && s.border.width);
                    const bStyle = s.border_style || (s.border && s.border.style);
                    const bColor = s.border_color || (s.border && s.border.color);
                    if (bWidth && bStyle) {
                        el.style.border = `${bWidth} ${bStyle} ${bColor || '#e2e8f0'}`;
                    }
                    if (s.border_top) el.style.borderTop = s.border_top;
                    if (s.border_right) el.style.borderRight = s.border_right;
                    if (s.border_bottom) el.style.borderBottom = s.border_bottom;
                    if (s.border_left) el.style.borderLeft = s.border_left;
                }

                const marT = s.margin_top || (s.margin && s.margin.top !== undefined ? `${s.margin.top}${s.margin.unit || 'px'}` : null);
                const marR = s.margin_right || (s.margin && s.margin.right !== undefined ? `${s.margin.right}${s.margin.unit || 'px'}` : null);
                const marB = s.margin_bottom || (s.margin && s.margin.bottom !== undefined ? `${s.margin.bottom}${s.margin.unit || 'px'}` : null);
                const marL = s.margin_left || (s.margin && s.margin.left !== undefined ? `${s.margin.left}${s.margin.unit || 'px'}` : null);
                if (marT) el.style.marginTop = marT;
                if (marR) el.style.marginRight = marR;
                if (marB) el.style.marginBottom = marB;
                if (marL) el.style.marginLeft = marL;

                if (s.box_shadow) el.style.boxShadow = s.box_shadow;
                if (s.width) el.style.width = s.width;
                if (s.max_width) el.style.maxWidth = s.max_width;
                if (s.height && node.type !== "Spacer") el.style.height = s.height;
                if (s.min_height) el.style.minHeight = s.min_height;
                if (s.z_index !== undefined && s.z_index !== null) el.style.zIndex = String(s.z_index);
                if (s.position) el.style.position = s.position;

                if (s.css_classes) {
                    s.css_classes.split(/\s+/).filter(Boolean).forEach(cls => el.classList.add(cls));
                }
            }

            if (node.id === activeElementId) {
                el.classList.add("active-element");
            }

            // SECTION NODE
            if (node.type === "Section") {
                attachSectionHandleAndDrag(el, node);

                const container = document.createElement("div");
                container.className = "elementor-container";

                if (node.children && node.children.length > 0) {
                    node.children.forEach(child => {
                        container.appendChild(buildDomNode(child, false));
                    });
                }
                el.appendChild(container);
                return el;
            }

            // GRID NODE (RESPONSIVE MULTI-COLUMN CONTAINER)
            if (node.type === "Grid") {
                el.classList.add("elementor-grid-container");
                el.style.position = "relative";
                el.style.display = "grid";
                const cols = (node.data && node.data.columns) || 3;
                el.style.gridTemplateColumns = `repeat(${cols}, minmax(0, 1fr))`;
                el.style.gap = ((node.data && node.data.gap) !== undefined ? node.data.gap : 24) + "px";
                el.style.width = "100%";
                el.style.marginBottom = "24px";
                el.style.minHeight = "60px";

                const handle = document.createElement("div");
                handle.className = "elementor-grid-handle";
                handle.style.position = "absolute";
                handle.style.top = "-20px";
                handle.style.left = "0";
                handle.style.fontSize = "10.5px";
                handle.style.fontWeight = "700";
                handle.style.background = "var(--e-color-accent)";
                handle.style.color = "#ffffff";
                handle.style.padding = "2px 8px";
                handle.style.borderRadius = "3px";
                handle.style.cursor = "pointer";
                handle.style.zIndex = "10";
                handle.innerHTML = `<i class="fa-solid fa-table-cells"></i> Grid (${cols} Col)`;
                handle.onclick = (e) => { e.stopPropagation(); selectElement(node.id); };
                el.appendChild(handle);

                el.ondragover = (e) => {
                    const data = getDragData(e);
                    if (!data) return;
                    e.preventDefault();
                    e.stopPropagation();
                    if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                };
                el.ondrop = (e) => {
                    const data = getDragData(e);
                    if (!data) return;
                    e.preventDefault();
                    e.stopPropagation();
                    hideDropIndicator();
                    handleDropOnContainer(node.id, data);
                };

                if (node.children && node.children.length > 0) {
                    node.children.forEach(child => {
                        el.appendChild(buildDomNode(child, false));
                    });
                }
                el.onclick = (e) => { e.stopPropagation(); selectElement(node.id); };
                return el;
            }

            // ELEMENTOR FLEXBOX CONTAINER / COLUMN / FLEX NODE
            if (node.type === "Container" || node.type === "Column" || node.type === "Flex") {
                const s = node.styles || {};
                if (node.type === "Container") {
                    el.classList.add("np-container");
                    if (isTopLevel) {
                        attachSectionHandleAndDrag(el, node);
                    } else {
                        el.onclick = (e) => {
                            e.stopPropagation();
                            selectElement(node.id);
                        };
                    }
                    el.style.position = s.position || "relative";
                    el.style.display = s.display || "flex";
                    el.style.flexDirection = s.flex_direction || "column";
                    if (s.justify_content) el.style.justifyContent = s.justify_content;
                    if (s.align_items) el.style.alignItems = s.align_items;
                    if (s.flex_wrap) el.style.flexWrap = s.flex_wrap;
                    if (s.gap) el.style.gap = s.gap;
                } else {
                    const isFlexRow = (node.type === "Flex" && node.data && node.data.direction === "row");
                    if (isFlexRow) {
                        el.classList.add("elementor-flex-row");
                        el.style.position = "relative";
                        el.style.display = "flex";
                        el.style.flexDirection = "row";
                        el.style.flexWrap = (node.data && node.data.wrap === false) ? "nowrap" : "wrap";
                        el.style.justifyContent = (node.data && node.data.justify) || "flex-start";
                        el.style.alignItems = (node.data && node.data.align) || "stretch";
                        el.style.gap = s.gap || "24px";
                        el.style.width = "100%";
                    } else {
                        el.classList.add("elementor-column");
                        el.style.position = "relative";
                        el.style.display = "flex";
                        el.style.flexDirection = "column";
                        el.style.gap = s.gap || "16px";
                        if (node.type === "Flex" && node.data && node.data.basis) {
                            el.style.flex = `1 1 ${node.data.basis}`;
                        }
                    }
                    el.onclick = (e) => {
                        e.stopPropagation();
                        selectElement(node.id);
                    };
                }

                const isBoxed = el.classList.contains("np-container-boxed");
                let childHost = el;
                if (isBoxed) {
                    childHost = document.createElement("div");
                    childHost.className = "np-container-inner";
                    el.appendChild(childHost);
                }

                if (node.children && node.children.length > 0) {
                    node.children.forEach(child => {
                        const containerChild = buildDomNode(child, false);
                        childHost.appendChild(containerChild);
                    });
                } else if (!s.background_image && !(s.background && s.background.image_url) && !s.min_height) {
                    const empty = document.createElement("div");
                    empty.className = "elementor-empty-view";
                    empty.innerHTML = `<i class="fa-solid fa-plus"></i><span>Drop widget here</span>`;

                    empty.ondragover = (e) => {
                        const data = getDragData(e);
                        if (!data) return;
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                        empty.classList.add("drag-over");
                    };
                    empty.ondragleave = () => empty.classList.remove("drag-over");
                    empty.ondrop = (e) => {
                        const data = getDragData(e);
                        if (!data) return;
                        e.preventDefault();
                        e.stopPropagation();
                        empty.classList.remove("drag-over");
                        hideDropIndicator();
                        handleDropOnContainer(node.id, data);
                    };

                    empty.onclick = (e) => {
                        e.stopPropagation();
                        selectElement(node.id);
                    };
                    childHost.appendChild(empty);
                }

                if (!isTopLevel) {
                    el.ondragover = (e) => {
                        const data = getDragData(e);
                        if (!data || data.action === "move_section") return;
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";

                        const widgetEls = el.querySelectorAll(".elementor-widget");
                        if (widgetEls.length === 0) {
                            showDropIndicator(el, "inside");
                        } else {
                            const lastW = widgetEls[widgetEls.length - 1];
                            const lastRect = lastW.getBoundingClientRect();
                            if (e.clientY > lastRect.bottom) {
                                showDropIndicator(lastW, "after");
                            }
                        }
                    };
                    el.ondrop = (e) => {
                        const data = getDragData(e);
                        if (!data || data.action === "move_section") return;
                        e.preventDefault();
                        e.stopPropagation();
                        hideDropIndicator();
                        handleDropOnContainer(node.id, data);
                    };
                }

                return el;
            }

            // WIDGET NODE
            el.classList.add("elementor-widget");

            // Widget Handle Badge [ ✎ | ⠿ | ⧉ | 🗑 ]
            const wHandle = document.createElement("div");
            wHandle.className = "elementor-widget-handle";
            wHandle.innerHTML = `
                <button class="widget-handle-btn w-edit" title="Edit Widget"><i class="fa-solid fa-pencil"></i></button>
                <button class="widget-handle-btn w-drag" title="Drag to Reorder" draggable="true"><i class="fa-solid fa-grip-vertical"></i></button>
                <button class="widget-handle-btn w-copy" title="Duplicate"><i class="fa-regular fa-copy"></i></button>
                <button class="widget-handle-btn w-del" title="Delete"><i class="fa-solid fa-trash-can"></i></button>
            `;

            wHandle.querySelector(".w-edit").onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };
            wHandle.querySelector(".w-copy").onclick = (e) => {
                e.stopPropagation();
                duplicateElement(node.id);
            };
            wHandle.querySelector(".w-del").onclick = (e) => {
                e.stopPropagation();
                deleteElement(node.id);
            };

            // Drag Widget to Reorder
            const wDragBtn = wHandle.querySelector(".w-drag");
            wDragBtn.ondragstart = (e) => {
                e.stopPropagation();
                window._activeDragData = { action: "move_widget", id: node.id };
                e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                e.dataTransfer.effectAllowed = "move";
                el.classList.add("is-dragging");
            };
            wDragBtn.ondragend = () => {
                el.classList.remove("is-dragging");
                window._activeDragData = null;
                hideDropIndicator();
            };

            el.appendChild(wHandle);

            // Widget Dragover Drop Target (Shows top/bottom insertion line)
            el.ondragover = (e) => {
                const data = getDragData(e);
                if (!data) return;

                // Don't target self when moving
                if (data.id === node.id) return;

                e.preventDefault();
                e.stopPropagation();

                const rect = el.getBoundingClientRect();
                const midY = rect.top + rect.height / 2;
                const pos = e.clientY < midY ? "before" : "after";
                showDropIndicator(el, pos);
            };

            el.ondragleave = (e) => {
                if (!el.contains(e.relatedTarget)) {
                    hideDropIndicator();
                }
            };

            el.ondrop = (e) => {
                const data = getDragData(e);
                if (!data) return;
                if (data.id === node.id) return;

                e.preventDefault();
                e.stopPropagation();

                const rect = el.getBoundingClientRect();
                const midY = rect.top + rect.height / 2;
                const pos = e.clientY < midY ? "before" : "after";

                hideDropIndicator();
                handleDropNearWidget(node.id, pos, data);
            };

            // Click widget to select
            el.onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };

            const data = node.data || {};
            const s = node.styles || {};
            const widgetContent = document.createElement("div");

            switch(node.type) {
                case "Heading":
                    const tag = "h" + (data.level || 2);
                    const h = document.createElement(tag);
                    h.className = "np-heading";
                    h.innerHTML = data.text || "Heading Text";
                    h.contentEditable = "true";
                    h.style.outline = "none";
                    h.style.margin = "0";
                    if (s.text_color) h.style.color = s.text_color;
                    if (s.font_family) h.style.fontFamily = s.font_family;
                    if (s.font_size) h.style.fontSize = s.font_size;
                    if (s.font_weight) h.style.fontWeight = s.font_weight;
                    if (s.line_height) h.style.lineHeight = s.line_height;
                    if (s.letter_spacing) h.style.letterSpacing = s.letter_spacing;
                    if (s.text_transform) h.style.textTransform = s.text_transform;
                    if (s.text_align) h.style.textAlign = s.text_align;
                    h.onblur = () => {
                        data.text = h.innerHTML;
                        updateRawAst();
                    };
                    widgetContent.appendChild(h);
                    break;

                case "Text":
                    const t = document.createElement("div");
                    t.className = "np-text";
                    t.innerHTML = data.content_html || "<p>Click to edit paragraph text...</p>";
                    t.contentEditable = "true";
                    t.style.outline = "none";
                    t.onblur = () => {
                        data.content_html = t.innerHTML;
                        updateRawAst();
                    };
                    widgetContent.appendChild(t);
                    break;

                case "Button":
                    const btn = document.createElement("a");
                    btn.className = "np-button";
                    btn.href = data.url || "#";
                    btn.onclick = (e) => { e.preventDefault(); };
                    if (s.background_color) btn.style.backgroundColor = s.background_color;
                    if (s.text_color) btn.style.color = s.text_color;
                    if (s.font_family) btn.style.fontFamily = s.font_family;
                    if (s.font_size) btn.style.fontSize = s.font_size;
                    if (s.font_weight) btn.style.fontWeight = s.font_weight;
                    if (s.line_height) btn.style.lineHeight = s.line_height;
                    if (s.letter_spacing) btn.style.letterSpacing = s.letter_spacing;
                    if (s.text_transform) btn.style.textTransform = s.text_transform;
                    if (s.border_radius) btn.style.borderRadius = s.border_radius;
                    if (s.padding_top) btn.style.paddingTop = s.padding_top;
                    if (s.padding_right) btn.style.paddingRight = s.padding_right;
                    if (s.padding_bottom) btn.style.paddingBottom = s.padding_bottom;
                    if (s.padding_left) btn.style.paddingLeft = s.padding_left;
                    if (s.border_width && s.border_style) {
                        btn.style.border = `${s.border_width} ${s.border_style} ${s.border_color || 'currentColor'}`;
                    }

                    let iconHtml = "";
                    if (data.icon) {
                        iconHtml = `<i class="${data.icon}"></i> `;
                    }
                    btn.innerHTML = `${iconHtml}<span>${data.text || "Click Here"}</span>`;
                    widgetContent.appendChild(btn);
                    break;

                case "Image":
                    const img = document.createElement("img");
                    img.className = "np-image";
                    img.src = data.src || data.url || "https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=800&q=80";
                    img.alt = data.alt || "";
                    img.style.maxWidth = s.max_width || "100%";
                    img.style.height = s.height || "auto";
                    if (data.width) {
                        img.style.width = (typeof data.width === "number" ? data.width + "px" : String(data.width));
                    } else if (s.width) {
                        img.style.width = s.width;
                    }
                    if (data.height) {
                        img.style.height = (typeof data.height === "number" ? data.height + "px" : String(data.height));
                    }
                    if (s.border_radius) {
                        img.style.borderRadius = s.border_radius;
                    }
                    widgetContent.appendChild(img);
                    break;

                case "Icon":
                    const ic = document.createElement("div");
                    ic.style.fontSize = s.font_size || "36px";
                    ic.style.color = s.text_color || "var(--e-global-color-primary, var(--e-color-accent))";
                    ic.innerHTML = `<i class="${data.icon || data.name || 'fa-solid fa-star'}"></i>`;
                    widgetContent.appendChild(ic);
                    break;

                case "IconBox":
                    const ib = document.createElement("div");
                    ib.className = "np-iconbox";
                    ib.style.textAlign = s.text_align || "left";
                    ib.innerHTML = `
                        <div style="font-size: 32px; color: ${s.text_color || 'var(--e-global-color-primary, var(--e-color-accent))'}; margin-bottom: 12px;">
                            <i class="${data.icon || 'fa-solid fa-cubes'}"></i>
                        </div>
                        <h4 class="np-iconbox-title" style="font-size: 18px; margin-bottom: 8px; font-weight: 700;">${data.title || 'Feature Title'}</h4>
                        <p class="np-iconbox-desc" style="opacity: 0.85; font-size: 14px; line-height: 1.5;">${data.description || 'Feature description explaining the key technical benefit.'}</p>
                    `;
                    widgetContent.appendChild(ib);
                    break;

                case "ImageBox":
                    const imgb = document.createElement("div");
                    imgb.className = "np-imagebox";
                    imgb.style.textAlign = s.text_align || "left";
                    const imgBoxUrl = data.image_url || data.url || "https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=600&q=80";
                    imgb.innerHTML = `
                        <div style="margin-bottom: 14px; overflow: hidden;">
                            <img src="${imgBoxUrl}" alt="${data.title || 'Image Feature'}" style="width: 100%; height: auto; display: block;" />
                        </div>
                        <h4 class="np-imagebox-title" style="font-size: 18px; margin-bottom: 8px; font-weight: 700;">${data.title || 'Image Feature'}</h4>
                        <p class="np-imagebox-desc" style="opacity: 0.85; font-size: 14px; line-height: 1.5;">${data.description || 'Feature description explaining the visual showcase.'}</p>
                    `;
                    widgetContent.appendChild(imgb);
                    break;

                case "Counter":
                    const cnt = document.createElement("div");
                    cnt.style.textAlign = "center";
                    cnt.style.padding = "8px 0";
                    const cntNumVal = data.number !== undefined ? data.number : "100%";
                    const cntDisplay = `${data.prefix || ''}${cntNumVal}${data.suffix || ''}`;
                    const cntLabel = data.title || data.label || "Lighthouse Score";
                    cnt.innerHTML = `
                        <div style="font-size: 42px; font-weight: 800; color: ${s.text_color || '#38bdf8'};">
                            ${escapeHtml(String(cntDisplay))}
                        </div>
                        <div style="font-size: 13px; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; margin-top: 4px;">
                            ${escapeHtml(cntLabel)}
                        </div>
                    `;
                    widgetContent.appendChild(cnt);
                    break;

                case "Divider":
                    const div = document.createElement("hr");
                    div.className = "np-divider";
                    div.style.border = "none";
                    div.style.borderTop = `${data.weight || '1px'} ${data.style || 'solid'} ${data.color || '#e2e8f0'}`;
                    div.style.margin = "20px 0";
                    widgetContent.appendChild(div);
                    break;

                case "Spacer":
                    const sp = document.createElement("div");
                    const spHeight = (data.height !== undefined && data.height !== null)
                        ? (typeof data.height === "number" ? data.height + "px" : String(data.height))
                        : (s.height || "50px");
                    sp.style.height = spHeight;
                    widgetContent.appendChild(sp);
                    break;

                case "Video":
                    const vid = document.createElement("div");
                    vid.style.position = "relative";
                    vid.style.paddingBottom = "56.25%";
                    vid.style.height = "0";
                    vid.style.overflow = "hidden";
                    vid.style.borderRadius = "8px";
                    vid.style.background = "#000000";
                    vid.innerHTML = `<iframe src="${data.url || 'https://www.youtube.com/embed/dQw4w9WgXcQ'}" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0;" allowfullscreen></iframe>`;
                    widgetContent.appendChild(vid);
                    break;

                case "Alert":
                    const al = document.createElement("div");
                    const alertBg = data.alert_type === "success" ? "#ecfdf5" : (data.alert_type === "warning" ? "#fffbeb" : "#eff6ff");
                    const alertBorder = data.alert_type === "success" ? "#10b981" : (data.alert_type === "warning" ? "#f59e0b" : "#3b82f6");
                    const alertText = data.alert_type === "success" ? "#065f46" : (data.alert_type === "warning" ? "#92400e" : "#1e40af");
                    al.style.background = alertBg;
                    al.style.borderLeft = "4px solid " + alertBorder;
                    al.style.padding = "14px 18px";
                    al.style.borderRadius = "4px";
                    al.style.color = alertText;
                    al.innerHTML = `<strong>${data.title || 'Notice'}:</strong> <span>${data.description || 'Alert message content.'}</span>`;
                    widgetContent.appendChild(al);
                    break;

                case "Testimonial":
                    const tstm = document.createElement("div");
                    tstm.style.background = "#f8fafc";
                    tstm.style.border = "1px solid #e2e8f0";
                    tstm.style.borderRadius = "8px";
                    tstm.style.padding = "24px";
                    tstm.innerHTML = `
                        <p style="font-style: italic; color: #334155; font-size: 15px; line-height: 1.6; margin-bottom: 16px;">"${data.content || 'Great performance and intuitive builder workflow.'}"</p>
                        <div style="display: flex; align-items: center; gap: 12px;">
                            <div style="width: 40px; height: 40px; border-radius: 50%; background: #e02660; color: #ffffff; display: flex; align-items: center; justify-content: center; font-weight: 700; font-size: 14px;">
                                ${(data.name || 'J').charAt(0)}
                            </div>
                            <div>
                                <div style="font-weight: 700; color: #0f172a; font-size: 14px;">${data.name || 'John Doe'}</div>
                                <div style="font-size: 12px; color: #64748b;">${data.job || 'Software Engineer'}</div>
                            </div>
                        </div>
                    `;
                    widgetContent.appendChild(tstm);
                    break;

                case "ProgressBar":
                    const pb = document.createElement("div");
                    pb.style.margin = "8px 0";
                    pb.innerHTML = `
                        <div style="display: flex; justify-content: space-between; font-size: 13px; font-weight: 600; color: inherit; margin-bottom: 6px;">
                            <span>${data.title || 'Skill'}</span>
                            <span>${data.display_text || (data.percent !== undefined ? data.percent + '%' : '90%')}</span>
                        </div>
                        <div style="height: 10px; background: rgba(148, 163, 184, 0.25); border-radius: 5px; overflow: hidden;">
                            <div style="width: ${data.percent !== undefined ? data.percent : 90}%; height: 100%; background: var(--e-color-accent); border-radius: 5px;"></div>
                        </div>
                    `;
                    widgetContent.appendChild(pb);
                    break;

                case "Accordion":
                case "Toggle":
                    const acc = document.createElement("div");
                    acc.className = "np-accordion";
                    const accItems = data.items || [
                        { title: "Item 1", content: "Accordion content text." },
                        { title: "Item 2", content: "More detailed technical explanation." }
                    ];
                    acc.innerHTML = accItems.map((item, idx) => `
                        <details class="np-accordion-item" style="border-bottom: 1px solid #e2e8f0; padding: 12px 16px; background: #ffffff;" ${idx === 0 ? 'open' : ''}>
                            <summary style="font-weight: 600; cursor: pointer;">${item.title || 'Section'}</summary>
                            <div class="np-accordion-body" style="margin-top: 10px; font-size: 14px; line-height: 1.5;">${item.content_html || item.content || ''}</div>
                        </details>
                    `).join("");
                    widgetContent.appendChild(acc);
                    break;

                case "Tabs":
                    const tb = document.createElement("div");
                    tb.style.background = "#ffffff";
                    tb.style.border = "1px solid #e2e8f0";
                    tb.style.borderRadius = "8px";
                    tb.style.padding = "16px";
                    const tabItems = data.items || [
                        { title: "Tab 1", content: "Tab 1 body content." },
                        { title: "Tab 2", content: "Tab 2 body content." }
                    ];
                    const firstTabBody = tabItems.length > 0 ? (tabItems[0].content_html || tabItems[0].content || "") : "";
                    tb.innerHTML = `
                        <div style="display: flex; gap: 8px; border-bottom: 2px solid #e2e8f0; margin-bottom: 14px;">
                            ${tabItems.map((item, idx) => `<button type="button" class="studio-tab-btn" data-tab-idx="${idx}" style="background: none; border: none; border-bottom: 2px solid ${idx === 0 ? 'var(--e-color-accent)' : 'transparent'}; margin-bottom: -2px; padding: 8px 16px; font-weight: 600; color: ${idx === 0 ? 'var(--e-color-accent)' : '#64748b'}; cursor: pointer;">${item.title || ('Tab ' + (idx + 1))}</button>`).join("")}
                        </div>
                        <div class="studio-tab-body" style="color: #334155; font-size: 14px; line-height: 1.6;">${firstTabBody}</div>
                    `;
                    tb.querySelectorAll(".studio-tab-btn").forEach(btnEl => {
                        btnEl.onclick = (ev) => {
                            ev.stopPropagation();
                            const tIdx = parseInt(btnEl.dataset.tabIdx) || 0;
                            tb.querySelectorAll(".studio-tab-btn").forEach((b, i) => {
                                b.style.borderBottomColor = (i === tIdx) ? "var(--e-color-accent)" : "transparent";
                                b.style.color = (i === tIdx) ? "var(--e-color-accent)" : "#64748b";
                            });
                            const bodyBox = tb.querySelector(".studio-tab-body");
                            if (bodyBox && tabItems[tIdx]) {
                                bodyBox.innerHTML = tabItems[tIdx].content_html || tabItems[tIdx].content || "";
                            }
                            selectElement(node.id);
                        };
                    });
                    widgetContent.appendChild(tb);
                    break;

                case "PriceTable":
                    const pt = document.createElement("div");
                    pt.style.background = s.background_color || "#ffffff";
                    pt.style.color = s.text_color || "#0f172a";
                    pt.style.border = "1px solid #e2e8f0";
                    pt.style.borderRadius = s.border_radius || "12px";
                    pt.style.padding = "28px 24px";
                    pt.style.textAlign = "center";
                    const ptFeatures = Array.isArray(data.features) ? data.features : [
                        "Unlimited Requests",
                        "Sub-millisecond SSR",
                        "Automatic SQLite Snapshots"
                    ];
                    const ptPeriodHtml = (data.period && String(data.period).trim() !== "")
                        ? `<span style="font-size: 14px; color: #64748b;">${escapeHtml(data.period)}</span>`
                        : "";
                    const ptBtnHtml = (data.button_text && String(data.button_text).trim() !== "")
                        ? `<a href="${escapeHtml(data.button_url || '#')}" onclick="event.preventDefault()" style="display: inline-block; width: 100%; padding: 12px 20px; background: var(--e-global-color-primary, var(--e-color-accent)); color: #ffffff; border-radius: 6px; font-weight: 600; text-decoration: none; font-size: 14px;">${escapeHtml(data.button_text)}</a>`
                        : "";
                    pt.innerHTML = `
                        <div style="font-size: 15px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: var(--e-global-color-primary, var(--e-color-accent)); margin-bottom: 10px;">${escapeHtml(data.title || 'Plan')}</div>
                        <div style="display: flex; align-items: baseline; justify-content: center; gap: 4px; margin-bottom: 20px;">
                            <span style="font-size: 22px; font-weight: 700; color: #64748b;">${escapeHtml(data.currency || '')}</span>
                            <span style="font-size: 40px; font-weight: 800; line-height: 1; color: inherit;">${escapeHtml(data.price || '')}</span>
                            ${ptPeriodHtml}
                        </div>
                        <ul style="list-style: none; padding: 16px 0 0 0; margin: 0 0 ${ptBtnHtml ? '24px' : '0'} 0; text-align: left; border-top: 1px solid #e2e8f0;">
                            ${ptFeatures.map(f => `
                                <li style="display: flex; align-items: flex-start; gap: 10px; padding: 6px 0; font-size: 14px; color: inherit;">
                                    <i class="fa-solid fa-circle-check" style="color: var(--e-global-color-primary, #10b981); font-size: 14px; margin-top: 3px;"></i>
                                    <span>${escapeHtml(typeof f === 'string' ? f : (f.text || ''))}</span>
                                </li>
                            `).join("")}
                        </ul>
                        ${ptBtnHtml}
                    `;
                    widgetContent.appendChild(pt);
                    break;

                case "CallToAction":
                    const cta = document.createElement("div");
                    cta.style.background = s.background_color || "#0f172a";
                    cta.style.color = s.text_color || "#ffffff";
                    cta.style.padding = "32px 24px";
                    cta.style.borderRadius = "10px";
                    cta.style.textAlign = "center";
                    cta.innerHTML = `
                        <h3 style="font-size: 24px; font-weight: 700; margin-bottom: 12px; color: inherit;">${escapeHtml(data.title || 'Call to Action')}</h3>
                        <div style="font-size: 15px; opacity: 0.85; margin-bottom: 20px;">${data.description_html || ''}</div>
                        <a href="${escapeHtml(data.button_url || '#')}" style="display: inline-block; padding: 12px 28px; background: var(--e-color-accent); color: #ffffff; border-radius: 6px; font-weight: 600; text-decoration: none;">
                            ${escapeHtml(data.button_text || 'Get Started')}
                        </a>
                    `;
                    widgetContent.appendChild(cta);
                    break;

                case "CustomHtml":
                    const ch = document.createElement("div");
                    ch.innerHTML = data.raw || data.raw_html || data.content_html || "<pre>Custom HTML Block</pre>";
                    ch.querySelectorAll("script").forEach(oldScript => {
                        const newScript = document.createElement("script");
                        Array.from(oldScript.attributes).forEach(attr => newScript.setAttribute(attr.name, attr.value));
                        newScript.textContent = oldScript.textContent;
                        oldScript.parentNode.replaceChild(newScript, oldScript);
                    });
                    widgetContent.appendChild(ch);
                    break;

                case "ShortcodeFallback":
                    const sf = document.createElement("div");
                    sf.style.padding = "12px 16px";
                    sf.style.background = "#f8fafc";
                    sf.style.border = "1px dashed #cbd5e1";
                    sf.style.borderRadius = "6px";
                    sf.style.fontFamily = "monospace";
                    sf.style.fontSize = "13px";
                    sf.style.color = "#475569";
                    sf.textContent = data.raw || "[shortcode]";
                    widgetContent.appendChild(sf);
                    break;

                case "Audio":
                    const aud = document.createElement("audio");
                    aud.controls = true;
                    aud.src = data.src || data.url || "";
                    aud.style.width = "100%";
                    widgetContent.appendChild(aud);
                    break;

                case "Map":
                    const mp = document.createElement("div");
                    mp.style.padding = "24px";
                    mp.style.background = "#f1f5f9";
                    mp.style.borderRadius = "8px";
                    mp.style.textAlign = "center";
                    mp.innerHTML = `<i class="fa-solid fa-location-dot" style="color: var(--e-color-accent); font-size: 24px; margin-bottom: 8px;"></i><div style="font-weight: 600; color: #0f172a;">${escapeHtml(data.address || 'Map Embed')}</div>`;
                    widgetContent.appendChild(mp);
                    break;

                case "Rating":
                    const rt = document.createElement("div");
                    const stars = Array.from({ length: data.max_rating || 5 }, (_, i) => `<i class="${i < (data.rating || 5) ? 'fa-solid' : 'fa-regular'} fa-star" style="color: #f59e0b; margin-right: 2px;"></i>`).join("");
                    rt.innerHTML = `
                        <div style="display: flex; align-items: center; gap: 8px;">
                            ${data.title ? `<span style="font-weight: 600; font-size: 14px;">${data.title}:</span>` : ''}
                            <div>${stars}</div>
                            <span style="font-weight: 700; font-size: 13px; color: #64748b;">(${data.rating || 5}/${data.max_rating || 5})</span>
                        </div>
                    `;
                    widgetContent.appendChild(rt);
                    break;

                case "SocialIcons":
                    const soc = document.createElement("div");
                    soc.className = "np-social-icons";
                    const socItems = data.items || [
                        { icon: "fa-brands fa-github", link: "#" },
                        { icon: "fa-brands fa-x-twitter", link: "#" },
                        { icon: "fa-brands fa-discord", link: "#" }
                    ];
                    soc.innerHTML = socItems.map(item => `
                        <a href="${item.link || '#'}" class="np-social-icon" onclick="event.preventDefault()">
                            <i class="${item.icon}"></i>
                        </a>
                    `).join("");
                    widgetContent.appendChild(soc);
                    break;

                case "IconList":
                    const il = document.createElement("ul");
                    il.className = "np-iconlist";
                    const ilItems = data.items || [
                        { icon: "fa-solid fa-check", text: "Zero Client-Side JavaScript Overhead" },
                        { icon: "fa-solid fa-bolt", text: "Sub-millisecond Server Response Times" }
                    ];
                    il.innerHTML = ilItems.map(item => {
                        const iconSpan = item.icon ? `<span class="np-iconlist-icon"><i class="${item.icon}"></i></span>` : "";
                        const inner = `${iconSpan}<span class="np-iconlist-text">${item.text || ''}</span>`;
                        return `<li class="np-iconlist-item">${item.link ? `<a href="${item.link}" onclick="event.preventDefault()">${inner}</a>` : inner}</li>`;
                    }).join("");
                    widgetContent.appendChild(il);
                    break;

                case "Form":
                case "NativeForm":
                    const frm = document.createElement("form");
                    frm.className = "np-form";
                    frm.onsubmit = (e) => { e.preventDefault(); return false; };
                    if (node.children && node.children.length > 0) {
                        node.children.forEach(child => {
                            if (child.type === "CustomHtml" && child.data) {
                                const childDiv = document.createElement("div");
                                childDiv.innerHTML = child.data.raw || child.data.raw_html || child.data.content_html || "";
                                frm.appendChild(childDiv);
                            } else {
                                frm.appendChild(buildDomNode(child, false));
                            }
                        });
                    } else {
                        frm.style.background = "#f8fafc";
                        frm.style.border = "1px solid #e2e8f0";
                        frm.style.borderRadius = "8px";
                        frm.style.padding = "24px";
                        frm.innerHTML = `
                            ${data.title ? `<h4 style="font-size: 16px; margin-bottom: 14px; color: #0f172a; font-weight: 700;">${data.title}</h4>` : ''}
                            <div style="display: flex; flex-direction: column; gap: 10px;">
                                <input type="text" placeholder="Full Name" disabled style="padding: 8px 12px; border: 1px solid #cbd5e1; border-radius: 4px; background: #ffffff; font-size: 13px;">
                                <input type="email" placeholder="Email Address" disabled style="padding: 8px 12px; border: 1px solid #cbd5e1; border-radius: 4px; background: #ffffff; font-size: 13px;">
                                <textarea placeholder="Your Message..." rows="3" disabled style="padding: 8px 12px; border: 1px solid #cbd5e1; border-radius: 4px; background: #ffffff; font-size: 13px;"></textarea>
                                <button type="button" class="np-button">${data.button_text || 'Send Message'}</button>
                            </div>
                        `;
                    }
                    widgetContent.appendChild(frm);
                    break;

                case "Benchmark":
                    const bmrk = document.createElement("div");
                    bmrk.style.background = "linear-gradient(135deg, #0f172a 0%, #1e293b 100%)";
                    bmrk.style.color = "#ffffff";
                    bmrk.style.padding = "24px";
                    bmrk.style.borderRadius = "8px";
                    bmrk.style.textAlign = "center";
                    bmrk.innerHTML = `
                        <div style="font-size: 12px; font-weight: 700; letter-spacing: 0.05em; color: #38bdf8; text-transform: uppercase; margin-bottom: 8px;">Tokio Async Engine</div>
                        <div style="font-size: 26px; font-weight: 800; margin-bottom: 8px;">${data.text || '95,000 req/sec Tokio Engine Benchmark'}</div>
                        <div style="font-size: 13px; color: #94a3b8;">Sub-millisecond TTFB &bull; Embedded Rusqlite WAL Mode</div>
                    `;
                    widgetContent.appendChild(bmrk);
                    break;

                default:
                    const generic = document.createElement("div");
                    generic.style.padding = "16px";
                    generic.style.background = "#f1f5f9";
                    generic.style.borderRadius = "4px";
                    generic.innerHTML = `<strong>${node.type}</strong>: ${data.text || node.id}`;
                    widgetContent.appendChild(generic);
            }

            el.appendChild(widgetContent);
            return el;
        }

        if (pageAst.root.children && pageAst.root.children.length > 0) {
            pageAst.root.children.forEach(sec => {
                canvas.appendChild(buildDomNode(sec, true));
            });
        }
        updateNavigator();
    }

    // Canvas background dragover & drop handlers
    const canvasEl = document.getElementById("elementor-canvas");
    canvasEl.ondragover = (e) => {
        const data = getDragData(e);
        if (data && (data.type === "new_widget" || data.action === "move_widget")) {
            e.preventDefault();
            if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
        }
    };
    canvasEl.ondrop = (e) => {
        if (e.target === canvasEl) {
            e.preventDefault();
            hideDropIndicator();
            const data = getDragData(e);
            if (!data) return;
            if (data.type === "new_widget") {
                const newWidget = createWidgetNode(data.widgetType);
                if (newWidget.type === "Section") {
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newWidget);
                } else {
                    if (newWidget.type === "Section") {
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newWidget);
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
            }
            const newSec = newWidget.type === "Section" ? newWidget : createSectionWithColumns(1, [newWidget]);
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newSec);
                }
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newSec);
                renderCanvas();
                selectElement(newWidget.id);
                showToast("Added new section with " + data.widgetType);
            }
            window._activeDragData = null;
        }
    };

    // Scrollable Preview Container dragover & drop handlers
    const previewEl = document.getElementById("elementor-preview");
    if (previewEl) {
        previewEl.ondragover = (e) => {
            const data = getDragData(e);
            if (data && (data.type === "new_widget" || data.action === "move_widget")) {
                e.preventDefault();
                if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
            }
        };
        previewEl.ondrop = (e) => {
            if (e.target === previewEl || e.target.id === "elementor-preview-responsive-wrapper") {
                e.preventDefault();
                hideDropIndicator();
                const data = getDragData(e);
                if (data && data.type === "new_widget") {
                    const newWidget = createWidgetNode(data.widgetType);
                    if (newWidget.type === "Section") {
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newWidget);
                } else {
                    if (newWidget.type === "Section") {
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newWidget);
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
            }
            const newSec = newWidget.type === "Section" ? newWidget : createSectionWithColumns(1, [newWidget]);
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newSec);
                }
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newSec);
                    renderCanvas();
                    selectElement(newWidget.id);
                    showToast("Added new section with " + data.widgetType);
                }
                window._activeDragData = null;
            }
        };
        previewEl.ondragleave = (e) => {
            if (!previewEl.contains(e.relatedTarget)) {
                hideDropIndicator();
            }
        };
    }

    // Drop handlers
    function handleDropOnContainer(containerId, data) {
        if (!data) return;
        const container = findNodeById(pageAst.root, containerId);
        if (!container) return;

        if (data.type === "new_widget") {
            const newWidget = createWidgetNode(data.widgetType);
            container.children = container.children || [];
            container.children.push(newWidget);
            renderCanvas();
            selectElement(newWidget.id);
            showToast("Added " + data.widgetType + " widget");
        } else if (data.action === "move_widget") {
            const moved = removeNodeById(pageAst.root, data.id);
            if (moved) {
                container.children = container.children || [];
                container.children.push(moved);
                renderCanvas();
                selectElement(moved.id);
                showToast("Moved element");
            }
        }
        window._activeDragData = null;
    }

    function handleDropNearWidget(targetWidgetId, position, data) {
        if (!data) return;
        const parent = findParentNode(pageAst.root, targetWidgetId);
        if (!parent || !parent.children) return;

        let targetIdx = parent.children.findIndex(c => c.id === targetWidgetId);
        if (targetIdx === -1) return;
        if (position === "after") targetIdx += 1;

        if (data.type === "new_widget") {
            const newWidget = createWidgetNode(data.widgetType);
            parent.children.splice(targetIdx, 0, newWidget);
            renderCanvas();
            selectElement(newWidget.id);
            showToast("Inserted " + data.widgetType + " widget");
        } else if (data.action === "move_widget") {
            const moved = removeNodeById(pageAst.root, data.id);
            if (moved) {
                let newTargetIdx = parent.children.findIndex(c => c.id === targetWidgetId);
                if (newTargetIdx === -1) {
                    parent.children.push(moved);
                } else {
                    if (position === "after") newTargetIdx += 1;
                    parent.children.splice(newTargetIdx, 0, moved);
                }
                renderCanvas();
                selectElement(moved.id);
                showToast("Reordered widget");
            }
        }
        window._activeDragData = null;
    }

    function executeSectionMove(draggedSecId, targetSecId, position) {
        if (pageAst.root.children) {
            const dragged = removeNodeById(pageAst.root, draggedSecId);
            if (dragged) {
                let idx = pageAst.root.children.findIndex(c => c.id === targetSecId);
                if (idx === -1) {
                    pageAst.root.children.push(dragged);
                } else {
                    if (position === "after") idx += 1;
                    pageAst.root.children.splice(idx, 0, dragged);
                }
                renderCanvas();
                selectElement(dragged.id);
                showToast("Reordered section");
            }
        }
        window._activeDragData = null;
    }

    // Bottom dropzone handling
    const bottomDropzone = document.getElementById("elementor-bottom-dropzone");
    bottomDropzone.ondragover = (e) => {
        e.preventDefault();
        bottomDropzone.classList.add("drag-over");
    };
    bottomDropzone.ondragleave = () => bottomDropzone.classList.remove("drag-over");
    bottomDropzone.ondrop = (e) => {
        e.preventDefault();
        bottomDropzone.classList.remove("drag-over");
        hideDropIndicator();
        const data = getDragData(e);
        if (!data) return;

        if (data.type === "new_widget") {
            const newWidget = createWidgetNode(data.widgetType);
            if (newWidget.type === "Section") {
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newWidget);
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
            }
            const newSec = newWidget.type === "Section" ? newWidget : createSectionWithColumns(1, [newWidget]);
            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newSec);
            renderCanvas();
            selectElement(newWidget.id);
            showToast("Added new section with " + data.widgetType);
        } else if (data.action === "move_widget") {
            const moved = removeNodeById(pageAst.root, data.id);
            if (moved) {
                const newSec = createSectionWithColumns(1, [moved]);
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newSec);
                renderCanvas();
                selectElement(moved.id);
                showToast("Moved element to new section");
            }
        }
        window._activeDragData = null;
    };

"###;
