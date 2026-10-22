//! Responsive mode switcher, Navigator tree drawer, layout persistence API, and Cmd+K Finder.

pub const SCRIPT_NAVIGATOR_AND_MODALS: &str = r###"    // -------------------------------------------------------------
    // PRESET STRUCTURE SELECTOR
    // -------------------------------------------------------------
    document.getElementById("btn-open-presets").onclick = () => {
        document.getElementById("preset-selector").classList.toggle("open");
    };

    document.querySelectorAll(".preset-card").forEach(card => {
        card.onclick = () => {
            const cols = card.dataset.cols;
            const newSec = {
                id: generateId("sec"),
                type: "Section",
                styles: { background_color: "#ffffff", padding_top: "60px", padding_bottom: "60px" },
                children: []
            };

            if (cols === "1") {
                newSec.children.push({ id: generateId("cont"), type: "Container", styles: {}, children: [] });
            } else if (cols === "2") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
            } else if (cols === "3") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
            } else if (cols === "4") {
                for (let i = 0; i < 4; i++) {
                    newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                }
            } else if (cols === "33-66") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "2" }, children: [] });
            } else if (cols === "66-33") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "2" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
            }

            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newSec);
            document.getElementById("preset-selector").classList.remove("open");
            renderCanvas();
            showToast("New section created");
        };
    });

    // -------------------------------------------------------------
    // RESPONSIVE MODE TOOLBAR
    // -------------------------------------------------------------
    const toolResponsive = document.getElementById("tool-responsive");
    const respBar = document.getElementById("elementor-responsive-bar");
    const previewContainer = document.getElementById("elementor-preview");

    toolResponsive.onclick = () => {
        respBar.classList.toggle("open");
        toolResponsive.classList.toggle("active");
    };

    document.getElementById("btn-close-resp-bar").onclick = () => {
        respBar.classList.remove("open");
        toolResponsive.classList.remove("active");
        setDeviceMode("desktop");
    };

    document.querySelectorAll(".resp-btn").forEach(btn => {
        btn.onclick = () => {
            document.querySelectorAll(".resp-btn").forEach(b => b.classList.remove("active"));
            btn.classList.add("active");
            setDeviceMode(btn.dataset.mode);
        };
    });

    function setDeviceMode(mode) {
        previewContainer.classList.remove("mode-tablet", "mode-mobile");
        const dimText = document.getElementById("resp-dimensions-text");
        if (mode === "tablet") {
            previewContainer.classList.add("mode-tablet");
            dimText.textContent = "Tablet (768px)";
        } else if (mode === "mobile") {
            previewContainer.classList.add("mode-mobile");
            dimText.textContent = "Mobile (375px)";
        } else {
            dimText.textContent = "100% / 1200px";
        }
    }

    // -------------------------------------------------------------
    // STRUCTURE / LAYERS NAVIGATOR PANEL
    // -------------------------------------------------------------
    const navDrawer = document.getElementById("elementor-navigator");
    const toolNav = document.getElementById("tool-navigator");
    const floatingNavBtn = document.getElementById("btn-floating-navigator");

    function toggleNavigatorPanel() {
        navDrawer.classList.toggle("open");
        toolNav.classList.toggle("active");
        if (navDrawer.classList.contains("open")) {
            updateNavigator();
        }
    }

    toolNav.onclick = toggleNavigatorPanel;
    floatingNavBtn.onclick = toggleNavigatorPanel;

    document.getElementById("btn-close-navigator").onclick = () => {
        navDrawer.classList.remove("open");
        toolNav.classList.remove("active");
    };

    // Keyboard shortcut Ctrl+I to toggle Navigator
    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "i") {
            e.preventDefault();
            toggleNavigatorPanel();
        }
    });

    // Draggable Navigator Window Header
    const navHeader = document.getElementById("navigator-header");
    let isDraggingNav = false;
    let navStartX, navStartY, navStartLeft, navStartTop;

    navHeader.addEventListener("mousedown", (e) => {
        if (e.target.closest("button")) return;
        isDraggingNav = true;
        navStartX = e.clientX;
        navStartY = e.clientY;
        const rect = navDrawer.getBoundingClientRect();
        navStartLeft = rect.left;
        navStartTop = rect.top;
        document.body.style.cursor = "move";
    });

    document.addEventListener("mousemove", (e) => {
        if (!isDraggingNav) return;
        const dx = e.clientX - navStartX;
        const dy = e.clientY - navStartY;
        navDrawer.style.left = (navStartLeft + dx) + "px";
        navDrawer.style.top = (navStartTop + dy) + "px";
        navDrawer.style.right = "auto";
    });

    document.addEventListener("mouseup", () => {
        if (isDraggingNav) {
            isDraggingNav = false;
            document.body.style.cursor = "default";
        }
    });

    // Expand / Collapse all layers
    let allExpanded = true;
    document.getElementById("nav-btn-expand-all").onclick = () => {
        allExpanded = !allExpanded;
        document.querySelectorAll(".nav-children").forEach(el => {
            el.style.display = allExpanded ? "block" : "none";
        });
        document.querySelectorAll(".nav-caret").forEach(c => {
            if (allExpanded) c.classList.remove("collapsed");
            else c.classList.add("collapsed");
        });
    };

    // Layer filter input
    const navSearchInput = document.getElementById("nav-search-input");
    navSearchInput.oninput = (e) => {
        const q = e.target.value.toLowerCase();
        document.querySelectorAll(".nav-tree-item").forEach(item => {
            const title = item.querySelector(".nav-item-title").textContent.toLowerCase();
            item.style.display = title.includes(q) ? "flex" : "none";
        });
    };

    function updateNavigator() {
        const tree = document.getElementById("navigator-tree");
        const countBadge = document.getElementById("nav-element-count");
        if (!tree) return;
        tree.innerHTML = "";

        const totalCount = countNodes(pageAst.root);
        if (countBadge) countBadge.textContent = totalCount;

        function walk(node, depth, parentEl) {
            const wrapper = document.createElement("div");
            wrapper.className = "nav-item-wrapper";

            const item = document.createElement("div");
            item.className = "nav-tree-item" + (node.id === activeElementId ? " active" : "");
            item.dataset.id = node.id;
            item.draggable = true;

            const hasChildren = node.children && node.children.length > 0;
            const indent = 8 + depth * 14;

            let icon = "fa-solid fa-cube";
            let labelText = node.type;

            if (node.type === "Section") {
                icon = "fa-solid fa-table-columns";
                labelText = "Section";
            } else if (node.type === "Container") {
                icon = "fa-regular fa-square";
                labelText = "Container";
            } else if (node.type === "Column" || node.type === "Flex") {
                icon = (node.data && node.data.direction === "row") ? "fa-solid fa-grip-lines" : "fa-solid fa-columns";
                labelText = (node.data && node.data.direction === "row") ? "Flex Row" : "Column";
            } else if (node.type === "PriceTable") {
                icon = "fa-solid fa-tags";
                labelText = 'Price Table: "' + ((node.data && node.data.title) || '') + '"';
            } else if (node.type === "CallToAction") {
                icon = "fa-solid fa-bullhorn";
                labelText = 'CTA: "' + ((node.data && node.data.title) || '') + '"';
            } else if (node.type === "Toggle") {
                icon = "fa-solid fa-bars-staggered";
                labelText = "Toggle";
            } else if (node.type === "CustomHtml") {
                icon = "fa-solid fa-code";
                labelText = "Custom HTML";
            } else if (node.type === "Heading") {
                icon = "fa-solid fa-heading";
                labelText = 'Heading: "' + ((node.data && node.data.text) || '').substr(0, 18) + '"';
            } else if (node.type === "Button") {
                icon = "fa-solid fa-hand-pointer";
                labelText = 'Button: "' + ((node.data && node.data.text) || 'Click') + '"';
            } else if (node.type === "Image") {
                icon = "fa-regular fa-image";
                labelText = "Image";
            } else if (node.type === "Text") {
                icon = "fa-solid fa-paragraph";
                labelText = "Text Editor";
            } else if (node.type === "ImageBox") {
                icon = "fa-regular fa-image";
                labelText = 'Image Box: "' + ((node.data && node.data.title) || '') + '"';
            } else if (node.type === "Grid") {
                icon = "fa-solid fa-table-cells";
                labelText = "Grid (" + ((node.data && node.data.columns) || 3) + " Col)";
                        } else if (node.type === "Icon" || node.type === "IconBox") {
                icon = (node.data && node.data.icon) || "fa-solid fa-icons";
                labelText = node.type === "IconBox" ? 'Icon Box: "' + ((node.data && node.data.title) || '') + '"' : "Icon";
            } else if (node.type === "Counter") {
                icon = "fa-solid fa-calculator";
                labelText = "Counter: " + ((node.data && node.data.number) || '');
            } else if (node.type === "ProgressBar") {
                icon = "fa-solid fa-bars-progress";
                labelText = "Progress: " + ((node.data && node.data.title) || '');
            } else if (node.type === "Testimonial") {
                icon = "fa-solid fa-quote-left";
                labelText = 'Testimonial: "' + ((node.data && node.data.name) || '') + '"';
            } else if (node.type === "Alert") {
                icon = "fa-solid fa-triangle-exclamation";
                labelText = "Alert: " + ((node.data && node.data.title) || '');
            } else if (node.type === "Accordion") {
                icon = "fa-solid fa-bars-staggered";
                labelText = "Accordion";
            } else if (node.type === "Tabs") {
                icon = "fa-solid fa-table-columns";
                labelText = "Tabs";
            } else if (node.type === "Rating") {
                icon = "fa-solid fa-star";
                labelText = "Rating: " + ((node.data && node.data.rating) || 5) + "/5";
            } else if (node.type === "SocialIcons") {
                icon = "fa-solid fa-share-nodes";
                labelText = "Social Icons";
            } else if (node.type === "IconList") {
                icon = "fa-solid fa-list-check";
                labelText = "Icon List";
            } else if (node.type === "Video") {
                icon = "fa-solid fa-play";
                labelText = "Video Embed";
            } else if (node.type === "NativeForm" || node.type === "Form") {
                icon = "fa-solid fa-envelope-open-text";
                labelText = "Antispam Form";
            } else if (node.type === "Benchmark") {
                icon = "fa-solid fa-gauge-high";
                labelText = "Benchmark";
            }

            const caretHtml = hasChildren
                ? `<i class="fa-solid fa-caret-down nav-caret"></i>`
                : `<span style="width: 14px; display: inline-block;"></span>`;

            item.innerHTML = `
                <div class="nav-item-left" style="padding-left: ${indent}px;">
                    ${caretHtml}
                    <i class="${icon} nav-item-icon"></i>
                    <span class="nav-item-title" title="${labelText}">${labelText}</span>
                </div>
                <div class="nav-item-actions">
                    <button class="nav-action-btn eye" title="Toggle Visibility">
                        <i class="fa-regular fa-eye"></i>
                    </button>
                    <button class="nav-action-btn dup" title="Duplicate">
                        <i class="fa-regular fa-copy"></i>
                    </button>
                    <button class="nav-action-btn del" title="Delete">
                        <i class="fa-solid fa-trash-can"></i>
                    </button>
                </div>
            `;

            // Row click -> Select and scroll
            item.onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };

            // Caret toggle
            const caret = item.querySelector(".nav-caret");
            let childrenContainer = null;
            if (caret && hasChildren) {
                caret.onclick = (e) => {
                    e.stopPropagation();
                    caret.classList.toggle("collapsed");
                    if (childrenContainer) {
                        childrenContainer.style.display = caret.classList.contains("collapsed") ? "none" : "block";
                    }
                };
            }

            // Eye visibility toggle
            item.querySelector(".nav-action-btn.eye").onclick = (e) => {
                e.stopPropagation();
                const domEl = document.getElementById(node.id);
                const eyeIcon = item.querySelector(".nav-action-btn.eye i");
                if (domEl) {
                    if (domEl.style.opacity === "0.2") {
                        domEl.style.opacity = "1";
                        eyeIcon.className = "fa-regular fa-eye";
                        showToast(node.type + " visible");
                    } else {
                        domEl.style.opacity = "0.2";
                        eyeIcon.className = "fa-regular fa-eye-slash";
                        showToast(node.type + " hidden");
                    }
                }
            };

            // Duplicate action
            item.querySelector(".nav-action-btn.dup").onclick = (e) => {
                e.stopPropagation();
                duplicateElement(node.id);
            };

            // Delete action
            item.querySelector(".nav-action-btn.del").onclick = (e) => {
                e.stopPropagation();
                deleteElement(node.id);
            };

            // Drag reorder within Navigator tree
            item.ondragstart = (e) => {
                e.stopPropagation();
                window._activeDragData = { action: "move_tree_item", id: node.id };
                e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                item.classList.add("is-dragging");
            };
            item.ondragend = () => {
                item.classList.remove("is-dragging");
                window._activeDragData = null;
                document.querySelectorAll(".nav-tree-item").forEach(i => i.classList.remove("drag-over-item"));
            };
            item.ondragover = (e) => {
                e.preventDefault();
                e.stopPropagation();
                const data = getDragData(e);
                if (data && data.id !== node.id) {
                    item.classList.add("drag-over-item");
                }
            };
            item.ondragleave = () => item.classList.remove("drag-over-item");
            item.ondrop = (e) => {
                e.preventDefault();
                e.stopPropagation();
                item.classList.remove("drag-over-item");
                const data = getDragData(e);
                if (data && data.id !== node.id) {
                    handleTreeDrop(data.id, node.id);
                }
            };

            wrapper.appendChild(item);

            if (hasChildren) {
                childrenContainer = document.createElement("div");
                childrenContainer.className = "nav-children";
                node.children.forEach(c => walk(c, depth + 1, childrenContainer));
                wrapper.appendChild(childrenContainer);
            }

            parentEl.appendChild(wrapper);
        }

        walk(pageAst.root, 0, tree);
    }

    function handleTreeDrop(sourceId, targetId) {
        if (sourceId === targetId) return;
        const targetParent = findParentNode(pageAst.root, targetId);
        if (!targetParent || !targetParent.children) return;

        const movedNode = removeNodeById(pageAst.root, sourceId);
        if (!movedNode) return;

        let targetIdx = targetParent.children.findIndex(c => c.id === targetId);
        if (targetIdx === -1) {
            targetParent.children.push(movedNode);
        } else {
            targetParent.children.splice(targetIdx, 0, movedNode);
        }

        renderCanvas();
        selectElement(movedNode.id);
        showToast("Structure reordered");
    }

    // -------------------------------------------------------------
    // PAGE SETTINGS MODAL
    // -------------------------------------------------------------
    document.getElementById("tool-settings").onclick = () => {
        document.getElementById("settings-modal").classList.add("open");
    };
    document.getElementById("btn-close-settings").onclick = () => {
        document.getElementById("settings-modal").classList.remove("open");
    };
    document.getElementById("btn-save-settings").onclick = async () => {
        const titleEl = document.getElementById("set-page-title");
        const statusEl = document.getElementById("set-page-status");
        const templateEl = document.getElementById("set-page-template");
        const bgTextEl = document.getElementById("set-canvas-bg-text");
        if (bgTextEl && bgTextEl.value) {
            const canvas = document.getElementById("elementor-canvas");
            if (canvas) canvas.style.backgroundColor = bgTextEl.value;
        }
        try {
            const res = await fetch("/wp-json/wp/v2/posts/" + POST_ID, {
                method: "PUT",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify({
                    title: titleEl ? titleEl.value : undefined,
                    status: statusEl ? statusEl.value : undefined,
                    template: templateEl ? templateEl.value : undefined
                })
            });
            document.getElementById("settings-modal").classList.remove("open");
            if (res.ok) {
                showToast("Page settings saved");
            } else {
                showToast("Failed to save page settings");
            }
        } catch (err) {
            console.error(err);
            document.getElementById("settings-modal").classList.remove("open");
            showToast("Network error saving page settings");
        }
    };

    // -------------------------------------------------------------
    // RAW JSON AST MODAL
    // -------------------------------------------------------------
    document.getElementById("opt-view-ast").onclick = () => {
        document.getElementById("saver-options-menu").classList.remove("open");
        const textarea = document.getElementById("ast-json-textarea");
        textarea.value = JSON.stringify(pageAst, null, 2);
        document.getElementById("ast-modal").classList.add("open");
    };
    document.getElementById("btn-close-ast").onclick = () => {
        document.getElementById("ast-modal").classList.remove("open");
    };
    document.getElementById("btn-copy-ast").onclick = () => {
        const textarea = document.getElementById("ast-json-textarea");
        navigator.clipboard.writeText(textarea.value);
        showToast("JSON AST copied to clipboard");
    };
    document.getElementById("btn-apply-ast").onclick = () => {
        try {
            const parsed = JSON.parse(document.getElementById("ast-json-textarea").value);
            pageAst = parsed;
            renderCanvas();
            renderInspector();
            document.getElementById("ast-modal").classList.remove("open");
            showToast("AST applied successfully");
        } catch(e) {
            alert("Invalid JSON format: " + e.message);
        }
    };

    // -------------------------------------------------------------
    // SAVER MENU & PERSISTENCE
    // -------------------------------------------------------------
    const saverBtn = document.getElementById("elementor-panel-saver-options-btn");
    const saverMenu = document.getElementById("saver-options-menu");
    saverBtn.onclick = (e) => {
        e.stopPropagation();
        saverMenu.classList.toggle("open");
    };
    document.addEventListener("click", () => saverMenu.classList.remove("open"));

    document.getElementById("elementor-panel-saver-button-publish").onclick = savePage;

    // Keyboard shortcut Ctrl+S
    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            e.preventDefault();
            savePage();
        }
    });

    // Hamburger Menu button opens Site Settings & Navigation Menu (Elementor parity)
    const panelMenuBtn = document.getElementById("elementor-panel-header-menu-button");
    if (panelMenuBtn) {
        panelMenuBtn.onclick = () => {
            if (currentPanelView === "menu") {
                currentPanelView = "elements";
            } else if (currentPanelView.startsWith("site-settings-") || currentPanelView === "user-preferences") {
                currentPanelView = "menu";
            } else {
                currentPanelView = "menu";
                activeElementId = null;
            }
            renderInspector();
        };
    }

    // 9-dots button opens widget catalog (Elements)
    const panelAddBtn = document.getElementById("elementor-panel-header-add-button");
    if (panelAddBtn) {
        panelAddBtn.onclick = () => {
            currentPanelView = "elements";
            activeElementId = null;
            renderCanvas();
            renderInspector();
        };
    }

    // Exit
    document.getElementById("opt-exit").onclick = () => {
        window.location.href = "/n-admin/";
    };

    async function savePage() {
        const saveText = document.getElementById("btn-save-text");
        saveText.textContent = "SAVING...";

        try {
            const res = await fetch("/wp-json/neopress/v1/layout/" + POST_ID, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify({
                    layout: pageAst,
                    layout_json: pageAst
                })
            });

            if (res.ok) {
                saveText.textContent = "UPDATE";
                showToast("Page saved and pre-compiled successfully!");
            } else {
                saveText.textContent = "UPDATE";
                showToast("Error saving page layout");
            }
        } catch(err) {
            console.error(err);
            saveText.textContent = "UPDATE";
            showToast("Network error saving page");
        }
    }

    function showToast(msg) {
        const toast = document.getElementById("elementor-toast");
        document.getElementById("toast-text").textContent = msg;
        toast.classList.add("show");
        setTimeout(() => toast.classList.remove("show"), 2800);
    }

    function updateRawAst() {
        // live ast update trigger
    }


    // -------------------------------------------------------------
    // KEYBOARD SHORTCUTS & SPOTLIGHT FINDER IMPLEMENTATION
    // -------------------------------------------------------------
    function openShortcutsModal() {
        const modal = document.getElementById("modal-shortcuts");
        if (modal) {
            modal.style.display = "flex";
            const closeBtn = document.getElementById("btn-close-shortcuts");
            if (closeBtn) closeBtn.onclick = () => { modal.style.display = "none"; };
            modal.onclick = (e) => { if (e.target === modal) modal.style.display = "none"; };
        }
    }

    const FINDER_COMMANDS = [
        { title: "Global Colors", cat: "Site Settings", icon: "fa-solid fa-palette", action: () => { currentPanelView = "site-settings-colors"; renderInspector(); } },
        { title: "Global Fonts", cat: "Site Settings", icon: "fa-solid fa-font", action: () => { currentPanelView = "site-settings-fonts"; renderInspector(); } },
        { title: "Buttons Style", cat: "Site Settings", icon: "fa-solid fa-square-check", action: () => { currentPanelView = "site-settings-buttons"; renderInspector(); } },
        { title: "Site Identity", cat: "Site Settings", icon: "fa-solid fa-id-card", action: () => { currentPanelView = "site-settings-identity"; renderInspector(); } },
        { title: "Layout & Spacing", cat: "Site Settings", icon: "fa-solid fa-table-columns", action: () => { currentPanelView = "site-settings-layout"; renderInspector(); } },
        { title: "User Preferences", cat: "Preferences", icon: "fa-solid fa-sliders", action: () => { currentPanelView = "user-preferences"; renderInspector(); } },
        { title: "Save Layout (Update)", cat: "Actions", icon: "fa-solid fa-floppy-disk", action: () => { savePage(); } },
        { title: "Keyboard Shortcuts", cat: "Preferences", icon: "fa-solid fa-keyboard", action: () => { openShortcutsModal(); } },
        { title: "Toggle Structure / Layers", cat: "Panels", icon: "fa-solid fa-layer-group", action: () => { toggleNavigatorPanel(); } },
        { title: "View Live Page", cat: "Navigate", icon: "fa-solid fa-arrow-up-right-from-square", action: () => { window.open("/?p=" + POST_ID, "_blank"); } },
        { title: "Exit to Dashboard", cat: "Navigate", icon: "fa-solid fa-arrow-right-from-bracket", action: () => { window.location.href = "/n-admin/"; } },
        { title: "Insert Heading", cat: "Widgets", icon: "fa-solid fa-heading", action: () => { insertWidgetAtRoot("Heading"); } },
        { title: "Insert Button", cat: "Widgets", icon: "fa-solid fa-hand-pointer", action: () => { insertWidgetAtRoot("Button"); } },
        { title: "Insert Text Editor", cat: "Widgets", icon: "fa-solid fa-paragraph", action: () => { insertWidgetAtRoot("Text"); } },
        { title: "Insert Image", cat: "Widgets", icon: "fa-regular fa-image", action: () => { insertWidgetAtRoot("Image"); } },
        { title: "Insert Icon", cat: "Widgets", icon: "fa-solid fa-icons", action: () => { insertWidgetAtRoot("Icon"); } },
        { title: "Insert Divider", cat: "Widgets", icon: "fa-solid fa-minus", action: () => { insertWidgetAtRoot("Divider"); } },
        { title: "Insert Spacer", cat: "Widgets", icon: "fa-solid fa-arrows-up-down", action: () => { insertWidgetAtRoot("Spacer"); } },
        { title: "Insert Image Box", cat: "Widgets", icon: "fa-regular fa-image", action: () => { insertWidgetAtRoot("ImageBox"); } },
        { title: "Insert Icon Box", cat: "Widgets", icon: "fa-solid fa-cubes", action: () => { insertWidgetAtRoot("IconBox"); } },
        { title: "Insert Counter", cat: "Widgets", icon: "fa-solid fa-calculator", action: () => { insertWidgetAtRoot("Counter"); } },
        { title: "Insert Progress Bar", cat: "Widgets", icon: "fa-solid fa-bars-progress", action: () => { insertWidgetAtRoot("ProgressBar"); } },
        { title: "Insert Testimonial", cat: "Widgets", icon: "fa-solid fa-quote-left", action: () => { insertWidgetAtRoot("Testimonial"); } },
        { title: "Insert Alert", cat: "Widgets", icon: "fa-solid fa-triangle-exclamation", action: () => { insertWidgetAtRoot("Alert"); } },
        { title: "Insert Accordion", cat: "Widgets", icon: "fa-solid fa-bars-staggered", action: () => { insertWidgetAtRoot("Accordion"); } },
        { title: "Insert Tabs", cat: "Widgets", icon: "fa-solid fa-table-columns", action: () => { insertWidgetAtRoot("Tabs"); } },
        { title: "Insert Rating", cat: "Widgets", icon: "fa-solid fa-star", action: () => { insertWidgetAtRoot("Rating"); } },
        { title: "Insert Social Icons", cat: "Widgets", icon: "fa-solid fa-share-nodes", action: () => { insertWidgetAtRoot("SocialIcons"); } },
        { title: "Insert Icon List", cat: "Widgets", icon: "fa-solid fa-list-check", action: () => { insertWidgetAtRoot("IconList"); } },
        { title: "Insert Video", cat: "Widgets", icon: "fa-solid fa-play", action: () => { insertWidgetAtRoot("Video"); } },
        { title: "Insert Antispam Form", cat: "Widgets", icon: "fa-solid fa-envelope-open-text", action: () => { insertWidgetAtRoot("NativeForm"); } },
        { title: "Insert Benchmark Metric", cat: "Widgets", icon: "fa-solid fa-gauge-high", action: () => { insertWidgetAtRoot("Benchmark"); } },
        { title: "Insert Grid Layout", cat: "Widgets", icon: "fa-solid fa-table-cells", action: () => { insertWidgetAtRoot("Grid"); } },
        { title: "Insert Global Header", cat: "Global Blocks", icon: "fa-solid fa-compass", action: () => { insertWidgetAtRoot("GlobalHeader"); } },
        { title: "Insert Hero Banner", cat: "Global Blocks", icon: "fa-solid fa-wand-magic-sparkles", action: () => { insertWidgetAtRoot("GlobalHero"); } },
        { title: "Insert Pricing Matrix", cat: "Global Blocks", icon: "fa-solid fa-tags", action: () => { insertWidgetAtRoot("GlobalPricing"); } },
        { title: "Insert Global Footer", cat: "Global Blocks", icon: "fa-solid fa-shoe-prints", action: () => { insertWidgetAtRoot("GlobalFooter"); } }
    ];

    function openFinderModal() {
        const modal = document.getElementById("modal-finder");
        const input = document.getElementById("finder-input");
        const resultsEl = document.getElementById("finder-results");
        if (!modal || !input || !resultsEl) return;

        modal.style.display = "flex";
        input.value = "";
        input.focus();

        function renderResults(q) {
            const query = q.toLowerCase().trim();
            const filtered = query ? FINDER_COMMANDS.filter(c => c.title.toLowerCase().includes(query) || c.cat.toLowerCase().includes(query)) : FINDER_COMMANDS;
            resultsEl.innerHTML = "";
            filtered.forEach((cmd, idx) => {
                const item = document.createElement("div");
                item.className = "finder-item" + (idx === 0 ? " active" : "");
                item.innerHTML = `
                    <div class="finder-item-icon"><i class="${cmd.icon}"></i></div>
                    <div class="finder-item-title">${cmd.title}</div>
                    <div class="finder-item-cat">${cmd.cat}</div>
                `;
                item.onclick = () => {
                    modal.style.display = "none";
                    cmd.action();
                };
                resultsEl.appendChild(item);
            });
        }

        renderResults("");
        input.oninput = (e) => { renderResults(e.target.value); };
        input.onkeydown = (e) => {
            if (e.key === "Enter") {
                const first = resultsEl.querySelector(".finder-item");
                if (first) first.click();
            } else if (e.key === "Escape") {
                modal.style.display = "none";
            }
        };

        modal.onclick = (e) => { if (e.target === modal) modal.style.display = "none"; };
    }

    // Global Key Listener for Ctrl+E (Finder) and Esc
    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "e") {
            e.preventDefault();
            openFinderModal();
        } else if (e.key === "Escape") {
            const f = document.getElementById("modal-finder");
            if (f && f.style.display === "flex") f.style.display = "none";
            const s = document.getElementById("modal-shortcuts");
            if (s && s.style.display === "flex") s.style.display = "none";
        }
    });

    // Restore saved user preferences
    try {
        const theme = localStorage.getItem("neopress_studio_theme");
        if (theme === "light") {
            document.getElementById("elementor-editor-wrapper").classList.add("theme-light");
        }
        const width = localStorage.getItem("neopress_panel_width");
        if (width) {
            document.getElementById("elementor-panel").style.width = width + "px";
        }
    } catch(err) {}

    // Initial boot
    renderCanvas();
    renderInspector();
    updateNavigator();

})();"###;
