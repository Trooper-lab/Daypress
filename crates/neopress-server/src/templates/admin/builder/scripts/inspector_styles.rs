//! Inspector sidebar Style and Advanced tab controls (Typography, Background, Spacing, Border).

pub const SCRIPT_INSPECTOR_STYLES: &str = r###"        // -------------------------------------------------------------
        // VIEW 9: DEFAULT ELEMENTS CATALOG (currentPanelView === "elements")
        // -------------------------------------------------------------
        if (!activeElementId) {
            menuBtn.innerHTML = `<i class="fa-solid fa-bars"></i>`;
            menuBtn.title = "Menu";
            titleEl.innerHTML = `<span class="e-logo">E</span> ELEMENTOR`;
            navEl.innerHTML = `
                <button class="elementor-panel-navigation-tab active" data-tab="elements">Elements</button>
                <button class="elementor-panel-navigation-tab" data-tab="globals">Globals</button>
            `;

            // Tab click handling
            navEl.querySelector('[data-tab="globals"]').onclick = () => {
                currentPanelView = "globals";
                renderInspector();
            };
            navEl.querySelector('[data-tab="elements"]').onclick = () => {
                currentPanelView = "elements";
                renderInspector();
            };

            bodyEl.innerHTML = `
                <div class="elementor-panel-search">
                    <i class="fa-solid fa-magnifying-glass"></i>
                    <input type="search" id="widget-search-input" placeholder="Search Widget...">
                </div>

                <!-- CATEGORY: BASIC -->
                <div class="elementor-panel-category" id="cat-basic">
                    <div class="elementor-category-header" onclick="this.parentElement.classList.toggle('collapsed')">
                        <span>Basic</span>
                        <i class="fa-solid fa-chevron-down"></i>
                    </div>
                    <div class="elementor-category-items">
                        <div class="elementor-element-widget" data-widget="Heading" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-heading"></i></div>
                            <div class="widget-title">Heading</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Text" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-paragraph"></i></div>
                            <div class="widget-title">Text Editor</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Button" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-hand-pointer"></i></div>
                            <div class="widget-title">Button</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Image" draggable="true">
                            <div class="widget-icon"><i class="fa-regular fa-image"></i></div>
                            <div class="widget-title">Image</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Icon" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-icons"></i></div>
                            <div class="widget-title">Icon</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Divider" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-minus"></i></div>
                            <div class="widget-title">Divider</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Spacer" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-arrows-up-down"></i></div>
                            <div class="widget-title">Spacer</div>
                        </div>
                    </div>
                </div>

                <!-- CATEGORY: GENERAL -->
                <div class="elementor-panel-category" id="cat-general">
                    <div class="elementor-category-header" onclick="this.parentElement.classList.toggle('collapsed')">
                        <span>General</span>
                        <i class="fa-solid fa-chevron-down"></i>
                    </div>
                    <div class="elementor-category-items">
                        <div class="elementor-element-widget" data-widget="ImageBox" draggable="true">
                            <div class="widget-icon"><i class="fa-regular fa-image"></i></div>
                            <div class="widget-title">Image Box</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Grid" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-table-cells"></i></div>
                            <div class="widget-title">Grid Layout</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="IconBox" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-cubes"></i></div>
                            <div class="widget-title">Icon Box</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Counter" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-calculator"></i></div>
                            <div class="widget-title">Counter</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="ProgressBar" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-bars-progress"></i></div>
                            <div class="widget-title">Progress Bar</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Testimonial" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-quote-left"></i></div>
                            <div class="widget-title">Testimonial</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Alert" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-triangle-exclamation"></i></div>
                            <div class="widget-title">Alert</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Accordion" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-bars-staggered"></i></div>
                            <div class="widget-title">Accordion</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Tabs" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-table-columns"></i></div>
                            <div class="widget-title">Tabs</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Rating" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-star"></i></div>
                            <div class="widget-title">Rating</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="SocialIcons" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-share-nodes"></i></div>
                            <div class="widget-title">Social Icons</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="IconList" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-list-check"></i></div>
                            <div class="widget-title">Icon List</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Video" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-play"></i></div>
                            <div class="widget-title">Video</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="PriceTable" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-tags"></i></div>
                            <div class="widget-title">Price Table</div>
                        </div>
                    </div>
                </div>

                <!-- CATEGORY: NEOPRESS NATIVE -->
                <div class="elementor-panel-category" id="cat-native">
                    <div class="elementor-category-header" onclick="this.parentElement.classList.toggle('collapsed')">
                        <span>Neopress Engine</span>
                        <i class="fa-solid fa-chevron-down"></i>
                    </div>
                    <div class="elementor-category-items">
                        <div class="elementor-element-widget" data-widget="NativeForm" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-envelope-open-text"></i></div>
                            <div class="widget-title">Antispam Form</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Benchmark" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-gauge-high"></i></div>
                            <div class="widget-title">Benchmark Card</div>
                        </div>
                    </div>
                </div>
            `;

            // Setup Draggable on every widget in palette
            bodyEl.querySelectorAll(".elementor-element-widget").forEach(w => {
                const type = w.dataset.widget;
                w.ondragstart = (e) => {
                    window._activeDragData = { type: "new_widget", widgetType: type };
                    try {
                        e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                        e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                        e.dataTransfer.effectAllowed = "copy";
                    } catch(err) {}
                    w.classList.add("is-dragging-card");
                };
                w.ondragend = () => {
                    w.classList.remove("is-dragging-card");
                    setTimeout(() => {
                        window._activeDragData = null;
                        hideDropIndicator();
                    }, 350);
                };
                w.onclick = () => {
                    insertWidgetAtRoot(type);
                };
            });

            // Widget search filter
            const searchInput = document.getElementById("widget-search-input");
            if (searchInput) {
                searchInput.oninput = (e) => {
                    const q = e.target.value.toLowerCase();
                    bodyEl.querySelectorAll(".elementor-element-widget").forEach(card => {
                        const title = card.querySelector(".widget-title").textContent.toLowerCase();
                        card.style.display = title.includes(q) ? "block" : "none";
                    });
                };
            }
            return;
        }
        // ELEMENT INSPECTOR (CONTENT / STYLE / ADVANCED)
        const node = findNodeById(pageAst.root, activeElementId);
        if (!node) {
            activeElementId = null;
            renderInspector();
            return;
        }

        titleEl.textContent = "EDIT " + node.type.toUpperCase();
        navEl.innerHTML = `
            <button class="elementor-panel-navigation-tab ${activeTab === 'content' ? 'active' : ''}" data-tab="content">
                <i class="fa-solid fa-pen-to-square"></i> Content
            </button>
            <button class="elementor-panel-navigation-tab ${activeTab === 'style' ? 'active' : ''}" data-tab="style">
                <i class="fa-solid fa-paintbrush"></i> Style
            </button>
            <button class="elementor-panel-navigation-tab ${activeTab === 'advanced' ? 'active' : ''}" data-tab="advanced">
                <i class="fa-solid fa-gear"></i> Advanced
            </button>
        `;

        navEl.querySelectorAll(".elementor-panel-navigation-tab").forEach(tab => {
            tab.onclick = () => {
                activeTab = tab.dataset.tab;
                renderInspector();
            };
        });

        node.data = node.data || {};
        node.styles = node.styles || {};
        const d = node.data;
        const s = node.styles;

        let controlsHtml = "";

        if (activeTab === "content") {
            // TAB: CONTENT
            controlsHtml += `<div class="elementor-controls-wrapper">`;

            if (node.type === "Heading") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Heading</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title Text</label>
                                <input type="text" class="elementor-input" id="ctrl-heading-text" value="${escapeHtml(d.text || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">HTML Tag</label>
                                <select class="elementor-select" id="ctrl-heading-tag">
                                    <option value="1" ${d.level == 1 ? 'selected' : ''}>H1</option>
                                    <option value="2" ${d.level == 2 || !d.level ? 'selected' : ''}>H2</option>
                                    <option value="3" ${d.level == 3 ? 'selected' : ''}>H3</option>
                                    <option value="4" ${d.level == 4 ? 'selected' : ''}>H4</option>
                                    <option value="5" ${d.level == 5 ? 'selected' : ''}>H5</option>
                                    <option value="6" ${d.level == 6 ? 'selected' : ''}>H6</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Alignment</label>
                                <div class="elementor-button-group">
                                    <button class="elementor-group-btn ${s.text_align === 'left' || !s.text_align ? 'active' : ''}" data-align="left"><i class="fa-solid fa-align-left"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'center' ? 'active' : ''}" data-align="center"><i class="fa-solid fa-align-center"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'right' ? 'active' : ''}" data-align="right"><i class="fa-solid fa-align-right"></i></button>
                                </div>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Text") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Text Editor</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">HTML Content</label>
                                <textarea class="elementor-textarea" id="ctrl-text-html" rows="7">${escapeHtml(d.content_html || '')}</textarea>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Alignment</label>
                                <div class="elementor-button-group">
                                    <button class="elementor-group-btn ${s.text_align === 'left' || !s.text_align ? 'active' : ''}" data-align="left"><i class="fa-solid fa-align-left"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'center' ? 'active' : ''}" data-align="center"><i class="fa-solid fa-align-center"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'right' ? 'active' : ''}" data-align="right"><i class="fa-solid fa-align-right"></i></button>
                                </div>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Button") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Button</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Text</label>
                                <input type="text" class="elementor-input" id="ctrl-btn-text" value="${escapeHtml(d.text || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Link URL</label>
                                <input type="text" class="elementor-input" id="ctrl-btn-url" value="${escapeHtml(d.url || '#')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Target</label>
                                <select class="elementor-select" id="ctrl-btn-target">
                                    <option value="_self" ${d.target !== '_blank' ? 'selected' : ''}>Same Window (_self)</option>
                                    <option value="_blank" ${d.target === '_blank' ? 'selected' : ''}>New Window (_blank)</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Icon</label>
                                <div class="icon-selector-control">
                                    <div class="icon-preview-box" id="ctrl-btn-icon-preview">
                                        <i class="${d.icon || 'fa-solid fa-bolt'}"></i>
                                    </div>
                                    <button class="btn-icon-library" id="ctrl-open-icon-picker">
                                        <i class="fa-solid fa-icons"></i> Icon Library
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Image") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Image</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Image URL</label>
                                <input type="text" class="elementor-input" id="ctrl-img-url" value="${escapeHtml(d.url || d.src || '')}" placeholder="https://...">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Alt Text</label>
                                <input type="text" class="elementor-input" id="ctrl-img-alt" value="${escapeHtml(d.alt || '')}" placeholder="Image description">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Link URL (Optional)</label>
                                <input type="text" class="elementor-input" id="ctrl-img-link" value="${escapeHtml(d.link || '')}" placeholder="https://...">
                            </div>
                            <div style="margin-top: 10px; display: flex; gap: 6px; flex-wrap: wrap;">
                                <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="setSampleImage('https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=800&q=80')">Analytics</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="setSampleImage('https://images.unsplash.com/photo-1460925895917-afdab827c52f?auto=format&fit=crop&w=800&q=80')">Dashboard</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="setSampleImage('https://images.unsplash.com/photo-1507238691740-187a5b1d37b8?auto=format&fit=crop&w=800&q=80')">Workspace</button>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Video") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Video</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Video Embed URL</label>
                                <input type="text" class="elementor-input" id="ctrl-video-url" value="${escapeHtml(d.url || '')}" placeholder="https://www.youtube.com/embed/...">
                            </div>
                            <div style="font-size: 11.5px; color: #64748b; margin-top: 4px; line-height: 1.4;">
                                Supports YouTube, Vimeo, or MP4 embed links.
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Alert") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Alert</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Type</label>
                                <select class="elementor-select" id="ctrl-alert-type">
                                    <option value="info" ${d.alert_type === 'info' ? 'selected' : ''}>Info (Blue)</option>
                                    <option value="success" ${d.alert_type === 'success' ? 'selected' : ''}>Success (Green)</option>
                                    <option value="warning" ${d.alert_type === 'warning' ? 'selected' : ''}>Warning (Amber)</option>
                                    <option value="danger" ${d.alert_type === 'danger' ? 'selected' : ''}>Danger (Red)</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title</label>
                                <input type="text" class="elementor-input" id="ctrl-alert-title" value="${escapeHtml(d.title || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Description</label>
                                <textarea class="elementor-textarea" id="ctrl-alert-desc" rows="3">${escapeHtml(d.description || '')}</textarea>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Testimonial") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Testimonial</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Content / Quote</label>
                                <textarea class="elementor-textarea" id="ctrl-test-content" rows="4">${escapeHtml(d.content || '')}</textarea>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Author Name</label>
                                <input type="text" class="elementor-input" id="ctrl-test-name" value="${escapeHtml(d.name || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Job Title</label>
                                <input type="text" class="elementor-input" id="ctrl-test-job" value="${escapeHtml(d.job || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Avatar Image URL (Optional)</label>
                                <input type="text" class="elementor-input" id="ctrl-test-avatar" value="${escapeHtml(d.image_url || '')}" placeholder="https://...">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "ProgressBar") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Progress Bar</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title</label>
                                <input type="text" class="elementor-input" id="ctrl-pb-title" value="${escapeHtml(d.title || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Percentage (${d.percent || 90}%)</label>
                                <input type="range" min="0" max="100" class="elementor-range" id="ctrl-pb-percent-range" value="${d.percent || 90}" style="width: 100%;">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Display Text</label>
                                <input type="text" class="elementor-input" id="ctrl-pb-label" value="${escapeHtml(d.display_text || (d.percent || 90) + '%')}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Accordion" || node.type === "Tabs" || node.type === "Toggle") {
                const isAcc = node.type === "Accordion" || node.type === "Toggle";
                const items = d.items || [];
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>${isAcc ? 'Accordion Items' : 'Tab Panels'}</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div id="ctrl-repeater-items" style="display: flex; flex-direction: column; gap: 10px; margin-bottom: 12px;">
                                ${items.map((it, idx) => `
                                    <div class="repeater-item-box" style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 10px;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                            <span style="font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase;">#${idx+1} ${isAcc ? 'Item' : 'Tab'}</span>
                                            <button type="button" class="btn-rep-del" data-idx="${idx}" style="background: none; border: none; color: #ef4444; cursor: pointer; font-size: 12px;"><i class="fa-solid fa-trash-can"></i></button>
                                        </div>
                                        <input type="text" class="elementor-input rep-title" data-idx="${idx}" value="${escapeHtml(it.title || '')}" placeholder="Title" style="margin-bottom: 6px; font-size: 12px; height: 28px;">
                                        <textarea class="elementor-textarea rep-content" data-idx="${idx}" rows="2" placeholder="Body Content" style="font-size: 12px;">${escapeHtml(it.content_html || it.content || '')}</textarea>
                                    </div>
                                `).join("")}
                            </div>
                            <button type="button" class="btn btn-secondary" id="btn-rep-add" style="width: 100%; font-size: 12px; font-weight: 600; padding: 7px;">
                                <i class="fa-solid fa-plus"></i> Add ${isAcc ? 'Accordion Item' : 'Tab Panel'}
                            </button>
                        </div>
                    </div>
                `;
            } else if (node.type === "Rating") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Rating</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Score (1-5)</label>
                                <input type="number" min="0" max="5" step="0.5" class="elementor-input" id="ctrl-rating-val" value="${d.rating || 5}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title / Label</label>
                                <input type="text" class="elementor-input" id="ctrl-rating-title" value="${escapeHtml(d.title || '')}" placeholder="e.g. Lighthouse Score">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "SocialIcons") {
                const items = d.items || [];
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Social Icons</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div id="ctrl-social-items" style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px;">
                                ${items.map((it, idx) => `
                                    <div class="social-item-row" style="display: flex; align-items: center; gap: 6px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 6px;">
                                        <div style="width: 28px; text-align: center; color: var(--e-color-accent); font-size: 15px;"><i class="${it.icon || 'fa-brands fa-github'}"></i></div>
                                        <input type="text" class="elementor-input soc-icon-name" data-idx="${idx}" value="${escapeHtml(it.icon || '')}" placeholder="fa-brands fa-..." style="flex: 1; font-size: 11px; height: 26px;">
                                        <input type="text" class="elementor-input soc-icon-link" data-idx="${idx}" value="${escapeHtml(it.link || '#')}" placeholder="URL" style="flex: 1; font-size: 11px; height: 26px;">
                                        <button type="button" class="btn-soc-del" data-idx="${idx}" style="background: none; border: none; color: #ef4444; cursor: pointer; padding: 0 4px;"><i class="fa-solid fa-xmark"></i></button>
                                    </div>
                                `).join("")}
                            </div>
                            <button type="button" class="btn btn-secondary" id="btn-soc-add" style="width: 100%; font-size: 12px; font-weight: 600; padding: 7px;">
                                <i class="fa-solid fa-plus"></i> Add Social Link
                            </button>
                        </div>
                    </div>
                `;
            } else if (node.type === "IconList") {
                const items = d.items || [];
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Icon List</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div id="ctrl-il-items" style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px;">
                                ${items.map((it, idx) => `
                                    <div class="il-item-row" style="display: flex; align-items: center; gap: 6px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 6px;">
                                        <div style="width: 24px; text-align: center; color: var(--e-color-accent);"><i class="${it.icon || 'fa-solid fa-check'}"></i></div>
                                        <input type="text" class="elementor-input il-text" data-idx="${idx}" value="${escapeHtml(it.text || '')}" placeholder="Item text" style="flex: 2; font-size: 11.5px; height: 26px;">
                                        <button type="button" class="btn-il-del" data-idx="${idx}" style="background: none; border: none; color: #ef4444; cursor: pointer; padding: 0 4px;"><i class="fa-solid fa-xmark"></i></button>
                                    </div>
                                `).join("")}
                            </div>
                            <button type="button" class="btn btn-secondary" id="btn-il-add" style="width: 100%; font-size: 12px; font-weight: 600; padding: 7px;">
                                <i class="fa-solid fa-plus"></i> Add List Item
                            </button>
                        </div>
                    </div>
                `;
            } else if (node.type === "ImageBox") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Image Box</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Image URL</label>
                                <input type="text" class="elementor-input" id="ctrl-imgbox-url" value="${escapeHtml(d.image_url || d.url || '')}" placeholder="https://...">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title</label>
                                <input type="text" class="elementor-input" id="ctrl-imgbox-title" value="${escapeHtml(d.title || '')}" placeholder="Feature Title">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Description</label>
                                <textarea class="elementor-textarea" id="ctrl-imgbox-desc" rows="3" placeholder="Feature description...">${escapeHtml(d.description || '')}</textarea>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Link URL</label>
                                <input type="text" class="elementor-input" id="ctrl-imgbox-link" value="${escapeHtml(d.link || '')}" placeholder="https://...">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Grid") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Grid Layout</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Columns (1 to 6)</label>
                                <input type="number" min="1" max="6" class="elementor-input" id="ctrl-grid-cols" value="${d.columns || 3}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Gap Between Items (px)</label>
                                <input type="number" min="0" max="100" class="elementor-input" id="ctrl-grid-gap" value="${d.gap !== undefined ? d.gap : 24}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Icon" || node.type === "IconBox") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Icon Settings</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Icon</label>
                                <div class="icon-selector-control">
                                    <div class="icon-preview-box" id="ctrl-icon-preview">
                                        <i class="${d.icon || 'fa-solid fa-star'}"></i>
                                    </div>
                                    <button class="btn-icon-library" id="ctrl-open-icon-picker">
                                        <i class="fa-solid fa-icons"></i> Icon Library
                                    </button>
                                </div>
                            </div>
                            ${node.type === 'IconBox' ? `
                                <div class="elementor-control-row">
                                    <label class="elementor-control-label">Title</label>
                                    <input type="text" class="elementor-input" id="ctrl-ib-title" value="${escapeHtml(d.title || '')}">
                                </div>
                                <div class="elementor-control-row">
                                    <label class="elementor-control-label">Description</label>
                                    <textarea class="elementor-textarea" id="ctrl-ib-desc" rows="3">${escapeHtml(d.description || '')}</textarea>
                                </div>
                            ` : ''}
                        </div>
                    </div>
                `;
            } else if (node.type === "Counter") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Counter</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Number</label>
                                <input type="text" class="elementor-input" id="ctrl-cnt-number" value="${escapeHtml(String(d.number !== undefined ? d.number : '100%'))}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Suffix</label>
                                <input type="text" class="elementor-input" id="ctrl-cnt-suffix" value="${escapeHtml(d.suffix || '')}" placeholder="e.g. req/s, %">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Label / Title</label>
                                <input type="text" class="elementor-input" id="ctrl-cnt-label" value="${escapeHtml(d.title || d.label || '')}" placeholder="e.g. Lighthouse Score">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "PriceTable") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Price Table</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Plan Title</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-title" value="${escapeHtml(d.title || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Price</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-price" value="${escapeHtml(d.price || '49')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Billing Period</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-period" value="${escapeHtml(d.period || '/ month')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Button Text</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-btn-text" value="${escapeHtml(d.button_text || 'Deploy Now')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Button URL</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-btn-url" value="${escapeHtml(d.button_url || '#')}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Spacer") {
                const spVal = (d.height !== undefined && d.height !== null) ? parseInt(d.height) : parseInt(s.height || '50');
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Spacer</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Height (px)</label>
                                <input type="number" min="5" max="600" class="elementor-input" id="ctrl-sp-height" value="${isNaN(spVal) ? 50 : spVal}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "CustomHtml") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Custom HTML</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">HTML Code</label>
                                <textarea class="elementor-textarea" id="ctrl-custom-html" rows="8" style="font-family: monospace; font-size: 11.5px;">${escapeHtml(d.raw || d.raw_html || d.content_html || '')}</textarea>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Divider") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Divider</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Style</label>
                                <select class="elementor-select" id="ctrl-div-style">
                                    <option value="solid" ${d.style !== 'dashed' && d.style !== 'dotted' ? 'selected' : ''}>Solid</option>
                                    <option value="dashed" ${d.style === 'dashed' ? 'selected' : ''}>Dashed</option>
                                    <option value="dotted" ${d.style === 'dotted' ? 'selected' : ''}>Dotted</option>
                                </select>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Section" || node.type === "Container" || node.type === "Column" || node.type === "Flex") {
                const bgImgVal = s.background_image || (s.background && s.background.image_url) || '';
                const bgColVal = s.background_color || (s.background && s.background.color) || '';
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Container / Section Layout</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Flex Direction</label>
                                <select class="elementor-select" id="ctrl-cont-dir">
                                    <option value="column" ${(s.flex_direction || 'column') === 'column' ? 'selected' : ''}>Column (Vertical)</option>
                                    <option value="row" ${s.flex_direction === 'row' ? 'selected' : ''}>Row (Horizontal)</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Width</label>
                                <input type="text" class="elementor-input" id="ctrl-cont-width" value="${escapeHtml(s.width || '')}" placeholder="e.g. 100%, 50%, 25%">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Min Height</label>
                                <input type="text" class="elementor-input" id="ctrl-cont-minh" value="${escapeHtml(s.min_height || '')}" placeholder="e.g. 625px, 275px">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Vertical Padding</label>
                                <input type="text" class="elementor-input" id="ctrl-sec-pad" value="${escapeHtml(s.padding_top || '')}" placeholder="e.g. 60px">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Background Color</label>
                                <div class="color-picker-control">
                                    <input type="color" class="color-swatch" id="ctrl-bg-swatch" value="${bgColVal.startsWith('#') ? bgColVal : '#ffffff'}">
                                    <input type="text" class="elementor-input" id="ctrl-bg-input" value="${escapeHtml(bgColVal)}" placeholder="#ffffff">
                                </div>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Background Image URL</label>
                                <input type="text" class="elementor-input" id="ctrl-cont-bgimg" value="${escapeHtml(bgImgVal)}" placeholder="/wp-content/uploads/...">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "NativeForm" || node.type === "Form") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Smart Antispam Form</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Form Title</label>
                                <input type="text" class="elementor-input" id="ctrl-form-title" value="${escapeHtml(d.title || 'Contact Form')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Button Text</label>
                                <input type="text" class="elementor-input" id="ctrl-form-btn" value="${escapeHtml(d.button_text || 'Send Message')}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Benchmark") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Benchmark Metric</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Metric Text</label>
                                <input type="text" class="elementor-input" id="ctrl-bench-text" value="${escapeHtml(d.text || '')}">
                            </div>
                        </div>
                    </div>
                `;
            } else {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>General Content</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Content</label>
                                <textarea class="elementor-textarea" id="ctrl-gen-text">${escapeHtml(d.content_html || d.text || '')}</textarea>
                            </div>
                        </div>
                    </div>
                `;
            }

            controlsHtml += `</div>`;
        } else if (activeTab === "style") {
            // TAB: STYLE
            controlsHtml += `
                <div class="elementor-controls-wrapper">
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header">
                            <span>Colors</span>
                            <i class="fa-solid fa-chevron-down"></i>
                        </div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Text Color</label>
                                <div class="color-picker-control">
                                    <input type="color" class="color-swatch" id="ctrl-color-swatch" value="${s.text_color || '#0f172a'}">
                                    <input type="text" class="elementor-input" id="ctrl-color-input" value="${s.text_color || ''}" placeholder="#0f172a">
                                </div>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Background Color</label>
                                <div class="color-picker-control">
                                    <input type="color" class="color-swatch" id="ctrl-bg-swatch" value="${s.background_color || '#ffffff'}">
                                    <input type="text" class="elementor-input" id="ctrl-bg-input" value="${s.background_color || ''}" placeholder="#ffffff or transparent">
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header">
                            <span>Typography & Dimensions</span>
                            <i class="fa-solid fa-chevron-down"></i>
                        </div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Font Size</label>
                                <input type="text" class="elementor-input" id="ctrl-font-size" value="${s.font_size || ''}" placeholder="e.g. 24px, 2rem">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Border Radius</label>
                                <input type="text" class="elementor-input" id="ctrl-border-radius" value="${s.border_radius || ''}" placeholder="e.g. 6px, 999px">
                            </div>
                        </div>
                    </div>
                </div>
            `;
        } else {
            // TAB: ADVANCED
            controlsHtml += `
                <div class="elementor-controls-wrapper">
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header">
                            <span>Layout & Box Model</span>
                            <i class="fa-solid fa-chevron-down"></i>
                        </div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Padding</label>
                                <div class="dimensions-control">
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-t" value="${s.padding_top || ''}">
                                        <span>Top</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-r" value="${s.padding_right || ''}">
                                        <span>Right</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-b" value="${s.padding_bottom || ''}">
                                        <span>Bottom</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-l" value="${s.padding_left || ''}">
                                        <span>Left</span>
                                    </div>
                                    <button class="dimension-link-btn linked" id="btn-link-padding" title="Link Values">
                                        <i class="fa-solid fa-link"></i>
                                    </button>
                                </div>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Margin</label>
                                <div class="dimensions-control">
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-t" value="${s.margin_top || ''}">
                                        <span>Top</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-r" value="${s.margin_right || ''}">
                                        <span>Right</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-b" value="${s.margin_bottom || ''}">
                                        <span>Bottom</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-l" value="${s.margin_left || ''}">
                                        <span>Left</span>
                                    </div>
                                    <button class="dimension-link-btn" title="Link Values">
                                        <i class="fa-solid fa-link"></i>
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }

        bodyEl.innerHTML = controlsHtml;
        bindInspectorEvents(node);
    }

"###;
