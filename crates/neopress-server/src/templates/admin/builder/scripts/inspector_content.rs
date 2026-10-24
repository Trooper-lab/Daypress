//! Block-Specific Content Settings and Universal Element Properties (`scripts/inspector_content.rs`).

pub const JS_INSPECTOR_CONTENT: &str = r###"
(function() {
    function renderContentSettings(container, node) {
        const type = node.type;
        const data = node.data || {};

        if (node.data && node.data._locked) {
            const notice = document.createElement("div");
            notice.className = "cad-locked-notice";
            notice.style.cssText = "padding:8px 12px;margin-bottom:8px;background:rgba(245,158,11,0.1);border:1px solid var(--neo-token);border-radius:var(--neo-radius-sm);color:var(--neo-token);font-size:11px;display:flex;align-items:center;gap:6px;";
            notice.innerHTML = `<i class="fa-solid fa-lock"></i> <span>Locked element. Unlock in Navigator to edit.</span>`;
            container.appendChild(notice);
        }

        // 1. Block-Specific Settings Panel
        const contentPanel = document.createElement("div");
        contentPanel.className = "cad-panel open";
        contentPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-sliders"></i> ${type} Content</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body" id="widget-specific-fields"></div>
        `;
        const fields = contentPanel.querySelector("#widget-specific-fields");
        renderSpecificFields(fields, node);
        attachToggle(contentPanel);
        container.appendChild(contentPanel);

        // 2. Universal Element Settings Panel
        const universalPanel = document.createElement("div");
        universalPanel.className = "cad-panel open";
        universalPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-code"></i> Universal Settings</span>
                <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row">
                    <span class="cad-label">Custom ID</span>
                    <input type="text" class="cad-input font-mono" style="width: 120px;" id="univ-node-id" value="${node.id}">
                </div>
                <div class="cad-row">
                    <span class="cad-label">HTML Tag</span>
                    <select class="cad-select" style="width: 100px;" id="univ-html-tag">
                        <option value="div" ${(!node.data || node.data.html_tag === 'div') ? 'selected' : ''}>div</option>
                        <option value="section" ${node.data && node.data.html_tag === 'section' ? 'selected' : ''}>section</option>
                        <option value="article" ${node.data && node.data.html_tag === 'article' ? 'selected' : ''}>article</option>
                        <option value="header" ${node.data && node.data.html_tag === 'header' ? 'selected' : ''}>header</option>
                        <option value="footer" ${node.data && node.data.html_tag === 'footer' ? 'selected' : ''}>footer</option>
                        <option value="nav" ${node.data && node.data.html_tag === 'nav' ? 'selected' : ''}>nav</option>
                    </select>
                </div>
                <div class="cad-row"><span class="cad-label">Hide Desktop</span><input type="checkbox" id="univ-hide-desktop" ${(node.styles && node.styles.hide_desktop) ? 'checked' : ''}></div>
                <div class="cad-row"><span class="cad-label">Hide Tablet</span><input type="checkbox" id="univ-hide-tablet" ${(node.styles && node.styles.hide_tablet) ? 'checked' : ''}></div>
                <div class="cad-row"><span class="cad-label">Hide Mobile</span><input type="checkbox" id="univ-hide-mobile" ${(node.styles && node.styles.hide_mobile) ? 'checked' : ''}></div>
            </div>
        `;
        attachToggle(universalPanel);
        attachUniversalListeners(universalPanel, node);
        container.appendChild(universalPanel);

        if (node.data && node.data._locked) {
            container.querySelectorAll("input, select, textarea, button").forEach(ctrl => {
                ctrl.disabled = true;
            });
        }
    }
    window.renderContentSettings = renderContentSettings;

    function renderSpecificFields(container, node) {
        const type = node.type;
        const data = node.data || {};

        switch(type) {
            case "Heading": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Text</span><input type="text" class="cad-input" style="flex:1;" data-key="text" value="${data.text || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Level</span><select class="cad-select" data-key="level"><option value="1" ${data.level === 1 ? 'selected' : ''}>H1</option><option value="2" ${(!data.level || data.level === 2) ? 'selected' : ''}>H2</option><option value="3" ${data.level === 3 ? 'selected' : ''}>H3</option></select></div>
                `;
                break;
            }
            case "Button": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Label</span><input type="text" class="cad-input" style="flex:1;" data-key="text" value="${data.text || 'Action'}"></div>
                    <div class="cad-row"><span class="cad-label">URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="url" value="${data.url || '#'}"></div>
                    <div class="cad-row"><span class="cad-label">Icon</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="icon" value="${data.icon || ''}" placeholder="fa-solid fa-arrow-right"></div>
                `;
                break;
            }
            case "PriceCard": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${data.title || 'Pro Cloud'}"></div>
                    <div class="cad-row"><span class="cad-label">Subtitle</span><input type="text" class="cad-input" style="flex:1;" data-key="subtitle" value="${data.subtitle || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Currency</span><input type="text" class="cad-input" style="width:60px;" data-key="currency" value="${data.currency || '$'}"><span class="cad-label">Price</span><input type="text" class="cad-input font-mono" style="width:70px;" data-key="price" value="${data.price || '49'}"></div>
                    <div class="cad-row"><span class="cad-label">Original Price</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="original_price" value="${data.original_price || ''}" placeholder="$79"></div>
                    <div class="cad-row"><span class="cad-label">Period</span><input type="text" class="cad-input" style="flex:1;" data-key="period" value="${data.period || '/mo'}"></div>
                    <div class="cad-row"><span class="cad-label">Ribbon Badge</span><input type="text" class="cad-input" style="flex:1;" data-key="badge" value="${data.badge || ''}" placeholder="POPULAR"></div>
                    <div class="cad-row"><span class="cad-label">Button Text</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${data.button_text || 'Deploy'}"></div>
                    <div class="cad-row"><span class="cad-label">Button URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="button_url" value="${data.button_url || '/signup'}"></div>
                `;
                break;
            }
            case "CallToAction": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${data.title || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Description</span><textarea class="cad-textarea" style="flex:1;" data-key="description">${data.description || ''}</textarea></div>
                    <div class="cad-row"><span class="cad-label">Button Text</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${data.button_text || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Button URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="button_url" value="${data.button_url || ''}"></div>
                `;
                break;
            }
            case "Countdown": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Target Date</span><input type="datetime-local" class="cad-input" style="flex:1;" data-key="target_date" value="${data.target_date || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Expiry Note</span><input type="text" class="cad-input" style="flex:1;" data-key="expiry_message" value="${data.expiry_message || 'Expired'}"></div>
                `;
                break;
            }
            case "FlipBox": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Front Title</span><input type="text" class="cad-input" style="flex:1;" data-key="front_title" value="${data.front_title || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Front Icon</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="front_icon" value="${data.front_icon || 'fa-solid fa-server'}"></div>
                    <div class="cad-row"><span class="cad-label">Back Title</span><input type="text" class="cad-input" style="flex:1;" data-key="back_title" value="${data.back_title || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Back Button</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${data.button_text || 'Action'}"></div>
                `;
                break;
            }
            case "Alert": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Type</span><select class="cad-select" style="flex:1;" data-key="alert_type"><option value="info" ${data.alert_type === 'info' ? 'selected' : ''}>Info</option><option value="success" ${data.alert_type === 'success' ? 'selected' : ''}>Success</option><option value="warning" ${data.alert_type === 'warning' ? 'selected' : ''}>Warning</option><option value="danger" ${data.alert_type === 'danger' ? 'selected' : ''}>Danger</option></select></div>
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${data.title || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Description</span><input type="text" class="cad-input" style="flex:1;" data-key="description" value="${data.description || ''}"></div>
                `;
                break;
            }
            case "PriceTable": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${data.title || 'Business Tier'}"></div>
                    <div class="cad-row"><span class="cad-label">Price</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="price" value="${data.price || '$79'}"></div>
                    <div class="cad-row"><span class="cad-label">Period</span><input type="text" class="cad-input" style="flex:1;" data-key="period" value="${data.period || '/month'}"></div>
                    <div class="cad-row"><span class="cad-label">Button Text</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${data.button_text || 'Choose Plan'}"></div>
                    <div class="cad-row"><span class="cad-label">Button URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="button_url" value="${data.button_url || '#'}"></div>
                `;
                break;
            }
            case "Testimonial": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Quote</span><textarea class="cad-textarea" style="flex:1;" data-key="content">${data.content || ''}</textarea></div>
                    <div class="cad-row"><span class="cad-label">Author</span><input type="text" class="cad-input" style="flex:1;" data-key="name" value="${data.name || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Job Title</span><input type="text" class="cad-input" style="flex:1;" data-key="job" value="${data.job || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Avatar URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="image_url" value="${data.image_url || ''}"></div>
                `;
                break;
            }
            case "Counter": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Prefix</span><input type="text" class="cad-input" style="width:60px;" data-key="prefix" value="${data.prefix || ''}"><span class="cad-label">Number</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="number" value="${data.number !== undefined ? data.number : '99.9'}"><span class="cad-label">Suffix</span><input type="text" class="cad-input" style="width:60px;" data-key="suffix" value="${data.suffix || '%'}"></div>
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${data.title || ''}"></div>
                `;
                break;
            }
            case "Benchmark": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Metric Label</span><input type="text" class="cad-input" style="flex:1;" data-key="label" value="${data.label || 'Throughput (Req/Sec)'}"></div>
                    <div class="cad-row"><span class="cad-label">Value</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="value" value="${data.value || '128,450 req/s'}"></div>
                    <div class="cad-row"><span class="cad-label">Comparison</span><input type="text" class="cad-input" style="flex:1;" data-key="comparison" value="${data.comparison || ''}"></div>
                `;
                break;
            }
            case "SearchForm": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Placeholder</span><input type="text" class="cad-input" style="flex:1;" data-key="placeholder" value="${data.placeholder || ''}"></div>
                    <div class="cad-row"><span class="cad-label">Button</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${data.button_text || 'Search'}"></div>
                `;
                break;
            }
            case "LoginForm": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">User Label</span><input type="text" class="cad-input" style="flex:1;" data-key="label_username" value="${data.label_username || 'Username or Email'}"></div>
                    <div class="cad-row"><span class="cad-label">Pass Label</span><input type="text" class="cad-input" style="flex:1;" data-key="label_password" value="${data.label_password || 'Password'}"></div>
                `;
                break;
            }
            case "CustomHtml": {
                const raw = data.raw || "";
                const parser = new DOMParser();
                const doc = parser.parseFromString("<div id='insp-root'>" + raw + "</div>", "text/html");
                const secEl = doc.getElementById("insp-root").firstElementChild;
                const cls = secEl ? (secEl.className || "") : "";
                const isPageheader = cls.includes("pageheader");
                const isSection = cls.includes("section");
                let bg = "white";
                if (cls.includes("dblue") || cls.includes("black")) bg = "dblue";
                else if (cls.includes("lblue") || cls.includes("blue")) bg = "lblue";
                else if (cls.includes("green")) bg = "green";
                const hasAngle = cls.includes("angle");
                const heroFull = cls.includes("full");

                const headings = secEl ? Array.from(secEl.querySelectorAll("h1, h2, h3, h4")) : [];
                const buttons = secEl ? Array.from(secEl.querySelectorAll("a.btn, a.wp-block-button__link")) : [];

                let html = "";
                if (isSection) {
                    html += `
                        <div class="cad-row">
                            <span class="cad-label">ACF Theme</span>
                            <select class="cad-select" style="flex:1;" id="acf-insp-bg">
                                <option value="dblue" ${bg === 'dblue' ? 'selected' : ''}>Dark Navy (.section.dblue)</option>
                                <option value="lblue" ${bg === 'lblue' ? 'selected' : ''}>Primary Blue (.section.lblue)</option>
                                <option value="white" ${bg === 'white' ? 'selected' : ''}>White (.section.white)</option>
                                <option value="green" ${bg === 'green' ? 'selected' : ''}>Neon Green (.section.green)</option>
                            </select>
                        </div>
                        <div class="cad-row"><span class="cad-label">Diagonal Angle</span><input type="checkbox" id="acf-insp-angle" ${hasAngle ? 'checked' : ''}></div>
                    `;
                } else if (isPageheader) {
                    html += `
                        <div class="cad-row"><span class="cad-label">Hero Height</span><select class="cad-select" style="flex:1;" id="acf-insp-hero"><option value="full" ${heroFull ? 'selected' : ''}>Full Height (.pageheader.full)</option><option value="compact" ${!heroFull ? 'selected' : ''}>Compact (.pageheader)</option></select></div>
                    `;
                }

                headings.slice(0, 3).forEach((h, idx) => {
                    const safeVal = (h.textContent || "").trim().replace(/"/g, "&quot;");
                    html += `
                        <div class="cad-row">
                            <span class="cad-label">${h.tagName} #${idx + 1}</span>
                            <input type="text" class="cad-input acf-insp-heading" data-h-idx="${idx}" style="flex:1;" value="${safeVal}">
                        </div>
                    `;
                });

                buttons.slice(0, 3).forEach((b, idx) => {
                    const safeTxt = (b.textContent || "").trim().replace(/"/g, "&quot;");
                    const safeHref = (b.getAttribute("href") || "#").replace(/"/g, "&quot;");
                    html += `
                        <div class="cad-row">
                            <span class="cad-label">CTA #${idx + 1}</span>
                            <input type="text" class="cad-input acf-insp-btn-txt" data-b-idx="${idx}" style="flex:1;" value="${safeTxt}">
                        </div>
                        <div class="cad-row">
                            <span class="cad-label">CTA #${idx + 1} URL</span>
                            <input type="text" class="cad-input font-mono acf-insp-btn-url" data-b-idx="${idx}" style="flex:1;" value="${safeHref}">
                        </div>
                    `;
                });

                const safeRaw = raw.replace(/</g, "&lt;").replace(/>/g, "&gt;");
                html += `
                    <div class="cad-row" style="flex-direction:column;align-items:stretch;gap:6px;margin-top:8px;">
                        <span class="cad-label">ACF Section HTML Source</span>
                        <textarea class="cad-textarea font-mono" rows="6" style="width:100%;font-size:11px;" id="acf-insp-raw">${safeRaw}</textarea>
                    </div>
                `;
                container.innerHTML = html;

                function mutateSec(fn) {
                    const p = new DOMParser();
                    const d = p.parseFromString("<div id='m-root'>" + (node.data.raw || "") + "</div>", "text/html");
                    const el = d.getElementById("m-root").firstElementChild;
                    if (!el) return;
                    fn(el);
                    node.data.raw = el.outerHTML;
                    const rawArea = container.querySelector("#acf-insp-raw");
                    if (rawArea) rawArea.value = node.data.raw;
                    window.markDirty(true);
                    window.renderCanvas();
                    if (window.updateNavigator) window.updateNavigator();
                }

                const bgSel = container.querySelector("#acf-insp-bg");
                if (bgSel) {
                    bgSel.onchange = () => mutateSec(el => {
                        el.classList.remove("white", "black", "blue", "lblue", "dblue", "green");
                        el.classList.add(bgSel.value);
                    });
                }
                const angleChk = container.querySelector("#acf-insp-angle");
                if (angleChk) {
                    angleChk.onchange = () => mutateSec(el => {
                        if (angleChk.checked) el.classList.add("angle");
                        else el.classList.remove("angle");
                    });
                }
                const heroSel = container.querySelector("#acf-insp-hero");
                if (heroSel) {
                    heroSel.onchange = () => mutateSec(el => {
                        if (heroSel.value === "full") el.classList.add("full");
                        else el.classList.remove("full");
                    });
                }
                container.querySelectorAll(".acf-insp-heading").forEach(inp => {
                    inp.oninput = () => {
                        const i = parseInt(inp.dataset.hIdx, 10);
                        mutateSec(el => {
                            const hs = el.querySelectorAll("h1, h2, h3, h4");
                            if (hs[i]) hs[i].textContent = inp.value;
                        });
                    };
                });
                container.querySelectorAll(".acf-insp-btn-txt").forEach(inp => {
                    inp.oninput = () => {
                        const i = parseInt(inp.dataset.bIdx, 10);
                        mutateSec(el => {
                            const bs = el.querySelectorAll("a.btn, a.wp-block-button__link");
                            if (bs[i]) bs[i].textContent = inp.value;
                        });
                    };
                });
                container.querySelectorAll(".acf-insp-btn-url").forEach(inp => {
                    inp.oninput = () => {
                        const i = parseInt(inp.dataset.bIdx, 10);
                        mutateSec(el => {
                            const bs = el.querySelectorAll("a.btn, a.wp-block-button__link");
                            if (bs[i]) bs[i].setAttribute("href", inp.value);
                        });
                    };
                });
                const rawArea = container.querySelector("#acf-insp-raw");
                if (rawArea) {
                    rawArea.oninput = () => {
                        node.data.raw = rawArea.value;
                        window.markDirty(true);
                        window.renderCanvas();
                    };
                }
                break;
            }
            default: {
                container.innerHTML = `
                    <p style="font-size: 11.5px; color: var(--neo-muted);">No specialized settings for this block type.</p>
                `;
            }
        }

        // Attach listeners for data keys
        container.querySelectorAll("input[data-key], textarea[data-key], select[data-key]").forEach(ctrl => {
            const k = ctrl.dataset.key;
            ctrl.oninput = () => {
                if (!node.data) node.data = {};
                node.data[k] = ctrl.value;
                window.markDirty(true);
                window.renderCanvas();
            };
        });
    }

    function attachUniversalListeners(panel, node) {
        const inputId = panel.querySelector("#univ-node-id");
        if (inputId) {
            inputId.onchange = () => {
                const newId = inputId.value.trim();
                if (newId && newId !== node.id) {
                    node.id = newId;
                    window.NeoStudioState.activeElementId = newId;
                    window.markDirty(true);
                    window.renderCanvas();
                    if (window.selectElement) window.selectElement(newId);
                }
            };
        }
        const selectTag = panel.querySelector("#univ-html-tag");
        if (selectTag) {
            selectTag.onchange = () => {
                if (!node.data) node.data = {};
                node.data.html_tag = selectTag.value;
                window.markDirty(true);
                window.renderCanvas();
            };
        }
        ["desktop", "tablet", "mobile"].forEach(bp => {
            const chk = panel.querySelector("#univ-hide-" + bp);
            if (chk) {
                chk.onchange = () => {
                    if (!node.styles) node.styles = {};
                    if (chk.checked) node.styles["hide_" + bp] = true;
                    else delete node.styles["hide_" + bp];
                    if (window.refreshNodeStyles) window.refreshNodeStyles(node);
                    window.pushHistorySnapshot();
                };
            }
        });
    }

    function attachToggle(panel) {
        const h = panel.querySelector(".cad-panel-header");
        if (h) h.onclick = () => panel.classList.toggle("open");
    }
})();
"###;
