//! Block-Specific Content Settings and Universal Element Properties (`scripts/inspector_content.rs`).

pub const JS_INSPECTOR_CONTENT: &str = r###"
(function() {
    const SEMANTIC_TAGS = [
        "div", "section", "article", "figure", "figcaption", "blockquote",
        "dl", "dt", "dd", "ol", "ul", "li", "p", "span", "cite", "code", "pre",
        "header", "footer", "nav", "aside", "main"
    ];

    function renderContentSettings(container, node) {
        const type = node.type;
        const currentTag = ((node.data && node.data.html_tag) || (node.styles && node.styles.html_tag) || (type === "Section" ? "section" : "div")).toLowerCase();

        if (node.data && node.data._locked) {
            const notice = document.createElement("div");
            notice.className = "cad-locked-notice";
            notice.style.cssText = "padding:8px 12px;margin-bottom:8px;background:rgba(245,158,11,0.1);border:1px solid var(--day-token);border-radius:var(--day-radius-sm);color:var(--day-token);font-size:11px;display:flex;align-items:center;gap:6px;";
            notice.innerHTML = `<i class="fa-solid fa-lock"></i> <span>Locked element. Unlock in Navigator to edit.</span>`;
            container.appendChild(notice);
        }

        const hasStarterDefaults = !!(window.getWidgetStarterDefaults && Object.keys(window.getWidgetStarterDefaults(type) || {}).length);
        const contentPanel = document.createElement("div");
        contentPanel.className = "cad-panel open";
        contentPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-sliders"></i> ${type} Content</span>
                <div style="display:flex;align-items:center;gap:6px;">
                    ${hasStarterDefaults ? `<button type="button" class="panel-reset-btn" id="btn-reset-content-defaults" style="display:inline-flex;" title="Reset ${type} content fields to defaults"><i class="fa-solid fa-rotate-left"></i> Reset</button>` : ""}
                    <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
                </div>
            </div>
            <div class="cad-panel-body" id="widget-specific-fields"></div>
        `;
        const fields = contentPanel.querySelector("#widget-specific-fields");
        renderSpecificFields(fields, node);
        attachToggle(contentPanel);
        const rstContentBtn = contentPanel.querySelector("#btn-reset-content-defaults");
        if (rstContentBtn) {
            rstContentBtn.onclick = (e) => {
                e.stopPropagation();
                const dflt = window.getWidgetStarterDefaults ? window.getWidgetStarterDefaults(type) : {};
                const keepName = node.data && node.data._name;
                node.data = Object.assign({}, dflt);
                if (keepName) node.data._name = keepName;
                window.markDirty(true);
                window.pushHistorySnapshot();
                window.renderCanvas();
                if (window.renderInspector) window.renderInspector();
                if (window.showToast) window.showToast(`Reset ${type} content to defaults`);
            };
        }
        container.appendChild(contentPanel);

        const tagOptions = SEMANTIC_TAGS.map(t => `<option value="${t}" ${currentTag === t ? 'selected' : ''}>${t}</option>`).join("");
        const universalPanel = document.createElement("div");
        universalPanel.className = "cad-panel open";
        universalPanel.innerHTML = `
            <div class="cad-panel-header">
                <span class="cad-panel-title"><i class="fa-solid fa-code"></i> Universal Settings</span>
                <div style="display:flex;align-items:center;gap:6px;">
                    <button type="button" class="panel-reset-btn" id="btn-reset-universal" style="display:inline-flex;" title="Reset HTML tag and responsive visibility"><i class="fa-solid fa-rotate-left"></i> Reset</button>
                    <i class="fa-solid fa-chevron-right cad-panel-chevron"></i>
                </div>
            </div>
            <div class="cad-panel-body">
                <div class="cad-row">
                    <span class="cad-label">Custom ID</span>
                    <input type="text" class="cad-input font-mono" style="width: 120px;" id="univ-node-id" value="${node.id}">
                </div>
                <div class="cad-row">
                    <span class="cad-label">HTML Tag</span>
                    <select class="cad-select" style="width: 110px;" id="univ-html-tag">${tagOptions}</select>
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
        const esc = (v) => String(v == null ? "" : v).replace(/&/g, "&amp;").replace(/"/g, "&quot;");

        switch(type) {
            case "Heading": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Text (HTML)</span><input type="text" class="cad-input" style="flex:1;" data-key="text" value="${esc(data.text)}"></div>
                    <div class="cad-row"><span class="cad-label">Level</span><select class="cad-select" data-key="level"><option value="1" ${data.level == 1 ? 'selected' : ''}>H1</option><option value="2" ${(!data.level || data.level == 2) ? 'selected' : ''}>H2</option><option value="3" ${data.level == 3 ? 'selected' : ''}>H3</option><option value="4" ${data.level == 4 ? 'selected' : ''}>H4</option><option value="5" ${data.level == 5 ? 'selected' : ''}>H5</option><option value="6" ${data.level == 6 ? 'selected' : ''}>H6</option></select></div>
                `;
                break;
            }
            case "Text": {
                const safeHtml = String(data.content_html || "").replace(/</g, "&lt;").replace(/>/g, "&gt;");
                container.innerHTML = `
                    <div class="cad-row" style="flex-direction:column;align-items:stretch;gap:4px;"><span class="cad-label">Rich Text / HTML</span><textarea class="cad-textarea font-mono" rows="4" style="width:100%;font-size:11px;" data-key="content_html">${safeHtml}</textarea></div>
                `;
                break;
            }
            case "Button": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Label</span><input type="text" class="cad-input" style="flex:1;" data-key="text" value="${esc(data.text || 'Action')}"></div>
                    <div class="cad-row"><span class="cad-label">URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="url" value="${esc(data.url || '#')}"></div>
                    <div class="cad-row"><span class="cad-label">Icon</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="icon" data-icon-field="1" value="${esc(data.icon)}" placeholder="fa-solid fa-arrow-right"></div>
                `;
                break;
            }
            case "Icon": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Icon</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="name" data-icon-field="1" value="${esc(data.name || data.icon || 'fa-solid fa-star')}"></div>
                    <div class="cad-row"><span class="cad-label">Link URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="url" value="${esc(data.url)}" placeholder="https://..."></div>
                `;
                break;
            }
            case "Image": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Source URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="src" value="${esc(data.src)}"></div>
                    <div class="cad-row"><span class="cad-label">Alt Text</span><input type="text" class="cad-input" style="flex:1;" data-key="alt" value="${esc(data.alt)}"></div>
                `;
                break;
            }
            case "Video": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Embed URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="url" value="${esc(data.url || 'https://www.youtube.com/embed/dQw4w9WgXcQ')}"></div>
                    <div class="cad-row"><span class="cad-label">Aspect</span><select class="cad-select" style="flex:1;" data-key="aspect_ratio"><option value="16:9" ${(!data.aspect_ratio || data.aspect_ratio === '16:9') ? 'selected' : ''}>16:9 Widescreen</option><option value="4:3" ${data.aspect_ratio === '4:3' ? 'selected' : ''}>4:3 Standard</option><option value="1:1" ${data.aspect_ratio === '1:1' ? 'selected' : ''}>1:1 Square</option></select></div>
                `;
                break;
            }
            case "Spacer": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Height (px)</span><input type="number" class="cad-input font-mono" style="flex:1;" data-key="height" value="${parseInt(data.height, 10) || 48}" min="0" max="600"></div>
                `;
                break;
            }
            case "Divider": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Line Style</span><select class="cad-select" style="flex:1;" data-key="style"><option value="solid" ${(!data.style || data.style === 'solid') ? 'selected' : ''}>Solid</option><option value="dashed" ${data.style === 'dashed' ? 'selected' : ''}>Dashed</option><option value="dotted" ${data.style === 'dotted' ? 'selected' : ''}>Dotted</option></select></div>
                `;
                break;
            }
            case "PriceCard": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title || 'Pro Cloud')}"></div>
                    <div class="cad-row"><span class="cad-label">Subtitle</span><input type="text" class="cad-input" style="flex:1;" data-key="subtitle" value="${esc(data.subtitle)}"></div>
                    <div class="cad-row"><span class="cad-label">Currency</span><input type="text" class="cad-input" style="width:60px;" data-key="currency" value="${esc(data.currency || '$')}"><span class="cad-label">Price</span><input type="text" class="cad-input font-mono" style="width:70px;" data-key="price" value="${esc(data.price || '49')}"></div>
                    <div class="cad-row"><span class="cad-label">Original Price</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="original_price" value="${esc(data.original_price)}" placeholder="$79"></div>
                    <div class="cad-row"><span class="cad-label">Period</span><input type="text" class="cad-input" style="flex:1;" data-key="period" value="${esc(data.period || '/mo')}"></div>
                    <div class="cad-row"><span class="cad-label">Ribbon Badge</span><input type="text" class="cad-input" style="flex:1;" data-key="badge" value="${esc(data.badge)}" placeholder="POPULAR"></div>
                    <div class="cad-row"><span class="cad-label">Button Text</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${esc(data.button_text || 'Deploy')}"></div>
                    <div class="cad-row"><span class="cad-label">Button URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="button_url" value="${esc(data.button_url || '/signup')}"></div>
                `;
                break;
            }
            case "CallToAction": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title)}"></div>
                    <div class="cad-row"><span class="cad-label">Description</span><textarea class="cad-textarea" style="flex:1;" data-key="description">${data.description || ''}</textarea></div>
                    <div class="cad-row"><span class="cad-label">Button Text</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${esc(data.button_text)}"></div>
                    <div class="cad-row"><span class="cad-label">Button URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="button_url" value="${esc(data.button_url)}"></div>
                `;
                break;
            }
            case "Countdown": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Target Date</span><input type="datetime-local" class="cad-input" style="flex:1;" data-key="target_date" value="${esc(data.target_date)}"></div>
                    <div class="cad-row"><span class="cad-label">Expiry Note</span><input type="text" class="cad-input" style="flex:1;" data-key="expiry_message" value="${esc(data.expiry_message || 'Expired')}"></div>
                `;
                break;
            }
            case "FlipBox": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Front Title</span><input type="text" class="cad-input" style="flex:1;" data-key="front_title" value="${esc(data.front_title)}"></div>
                    <div class="cad-row"><span class="cad-label">Front Icon</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="front_icon" data-icon-field="1" value="${esc(data.front_icon || 'fa-solid fa-server')}"></div>
                    <div class="cad-row"><span class="cad-label">Back Title</span><input type="text" class="cad-input" style="flex:1;" data-key="back_title" value="${esc(data.back_title)}"></div>
                    <div class="cad-row"><span class="cad-label">Back Button</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${esc(data.button_text || 'Action')}"></div>
                `;
                break;
            }
            case "AnimatedHeadline": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Before Text</span><input type="text" class="cad-input" style="flex:1;" data-key="before_text" value="${esc(data.before_text || 'Build')}"></div>
                    <div class="cad-row"><span class="cad-label">After Text</span><input type="text" class="cad-input" style="flex:1;" data-key="after_text" value="${esc(data.after_text || 'Websites')}"></div>
                `;
                break;
            }
            case "StarRating":
            case "Rating": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title || 'Customer Rating')}"></div>
                    <div class="cad-row"><span class="cad-label">Score</span><input type="number" step="0.1" min="0" max="5" class="cad-input font-mono" style="width:70px;" data-key="rating" value="${data.rating ?? 5}"><span class="cad-label">Max</span><input type="number" min="1" max="10" class="cad-input font-mono" style="width:60px;" data-key="max_rating" value="${data.max_rating ?? 5}"></div>
                `;
                break;
            }
            case "ProgressBar": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title || 'Progress')}"></div>
                    <div class="cad-row"><span class="cad-label">Percent (%)</span><input type="number" min="0" max="100" class="cad-input font-mono" style="width:70px;" data-key="percent" value="${data.percent ?? 98}"><span class="cad-label">Badge</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="display_text" value="${esc(data.display_text || '98%')}"></div>
                `;
                break;
            }
            case "Alert": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Type</span><select class="cad-select" style="flex:1;" data-key="alert_type"><option value="info" ${data.alert_type === 'info' ? 'selected' : ''}>Info</option><option value="success" ${data.alert_type === 'success' ? 'selected' : ''}>Success</option><option value="warning" ${data.alert_type === 'warning' ? 'selected' : ''}>Warning</option><option value="danger" ${data.alert_type === 'danger' ? 'selected' : ''}>Danger</option></select></div>
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title)}"></div>
                    <div class="cad-row"><span class="cad-label">Description</span><input type="text" class="cad-input" style="flex:1;" data-key="description" value="${esc(data.description)}"></div>
                `;
                break;
            }
            case "PriceTable": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title || 'Business Tier')}"></div>
                    <div class="cad-row"><span class="cad-label">Price</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="price" value="${esc(data.price || '$79')}"></div>
                    <div class="cad-row"><span class="cad-label">Period</span><input type="text" class="cad-input" style="flex:1;" data-key="period" value="${esc(data.period || '/month')}"></div>
                    <div class="cad-row"><span class="cad-label">Button Text</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${esc(data.button_text || 'Choose Plan')}"></div>
                    <div class="cad-row"><span class="cad-label">Button URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="button_url" value="${esc(data.button_url || '#')}"></div>
                `;
                break;
            }
            case "Testimonial": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Quote</span><textarea class="cad-textarea" style="flex:1;" data-key="content">${data.content || ''}</textarea></div>
                    <div class="cad-row"><span class="cad-label">Author</span><input type="text" class="cad-input" style="flex:1;" data-key="name" value="${esc(data.name)}"></div>
                    <div class="cad-row"><span class="cad-label">Job Title</span><input type="text" class="cad-input" style="flex:1;" data-key="job" value="${esc(data.job)}"></div>
                    <div class="cad-row"><span class="cad-label">Avatar URL</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="image_url" value="${esc(data.image_url)}"></div>
                `;
                break;
            }
            case "Counter": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Prefix</span><input type="text" class="cad-input" style="width:60px;" data-key="prefix" value="${esc(data.prefix)}"><span class="cad-label">Number</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="number" value="${data.number !== undefined ? data.number : '99.9'}"><span class="cad-label">Suffix</span><input type="text" class="cad-input" style="width:60px;" data-key="suffix" value="${esc(data.suffix || '%')}"></div>
                    <div class="cad-row"><span class="cad-label">Title</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title)}"></div>
                `;
                break;
            }
            case "Benchmark": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Metric Label</span><input type="text" class="cad-input" style="flex:1;" data-key="label" value="${esc(data.label || 'Throughput (Req/Sec)')}"></div>
                    <div class="cad-row"><span class="cad-label">Value</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="value" value="${esc(data.value || '128,450 req/s')}"></div>
                    <div class="cad-row"><span class="cad-label">Comparison</span><input type="text" class="cad-input" style="flex:1;" data-key="comparison" value="${esc(data.comparison)}"></div>
                `;
                break;
            }
            case "LoopGrid": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Post Type</span><select class="cad-select" style="flex:1;" data-key="post_type"><option value="post" ${(!data.post_type || data.post_type === 'post') ? 'selected' : ''}>Posts</option><option value="page" ${data.post_type === 'page' ? 'selected' : ''}>Pages</option></select></div>
                    <div class="cad-row"><span class="cad-label">Columns</span><input type="number" min="1" max="6" class="cad-input font-mono" style="width:64px;" data-key="columns" value="${data.columns || 3}"><span class="cad-label">Count</span><input type="number" min="1" max="24" class="cad-input font-mono" style="width:64px;" data-key="posts_per_page" value="${data.posts_per_page || 3}"></div>
                `;
                break;
            }
            case "TableOfContents": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Heading</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title || 'Table of Contents')}"></div>
                    <div class="cad-row"><span class="cad-label">Min H-Level</span><input type="number" min="1" max="6" class="cad-input font-mono" style="width:60px;" data-key="min_depth" value="${data.min_depth || 2}"><span class="cad-label">Max</span><input type="number" min="1" max="6" class="cad-input font-mono" style="width:60px;" data-key="max_depth" value="${data.max_depth || 4}"></div>
                `;
                break;
            }
            case "Breadcrumbs": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Home Label</span><input type="text" class="cad-input" style="flex:1;" data-key="home_text" value="${esc(data.home_text || 'Home')}"></div>
                    <div class="cad-row"><span class="cad-label">Separator</span><input type="text" class="cad-input font-mono" style="width:64px;" data-key="separator" value="${esc(data.separator || '/')}"></div>
                `;
                break;
            }
            case "Form":
            case "NativeForm": {
                const fid = data.form_id !== undefined ? data.form_id : 1;
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">FormStudio ID</span><input type="number" class="cad-input font-mono" style="flex:1;" data-key="form_id" value="${fid}" min="1"></div>
                    <div class="cad-row"><span class="cad-label">Form Heading</span><input type="text" class="cad-input" style="flex:1;" data-key="title" value="${esc(data.title)}"></div>
                    <div class="cad-row"><span class="cad-label">Submit Button</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${esc(data.button_text || 'Send Message')}"></div>
                    <div class="cad-row"><span class="cad-label">Custom Endpoint</span><input type="text" class="cad-input font-mono" style="flex:1;" data-key="endpoint" value="${esc(data.endpoint)}" placeholder="/wp-json/daypress/v1/forms/${fid}/submit"></div>
                    <div class="cad-row" style="margin-top:6px;"><a href="/d-admin/forms?form_id=${fid}" target="_blank" style="font-size:11px;color:var(--day-accent, #3b82f6);text-decoration:none;font-weight:600;">&#8599; Open Form #${fid} in FormStudio Builder</a></div>
                `;
                break;
            }
            case "SearchForm": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">Placeholder</span><input type="text" class="cad-input" style="flex:1;" data-key="placeholder" value="${esc(data.placeholder)}"></div>
                    <div class="cad-row"><span class="cad-label">Button</span><input type="text" class="cad-input" style="flex:1;" data-key="button_text" value="${esc(data.button_text || 'Search')}"></div>
                `;
                break;
            }
            case "LoginForm": {
                container.innerHTML = `
                    <div class="cad-row"><span class="cad-label">User Label</span><input type="text" class="cad-input" style="flex:1;" data-key="label_username" value="${esc(data.label_username || 'Username or Email')}"></div>
                    <div class="cad-row"><span class="cad-label">Pass Label</span><input type="text" class="cad-input" style="flex:1;" data-key="label_password" value="${esc(data.label_password || 'Password')}"></div>
                `;
                break;
            }
            default: {
                container.innerHTML = `<p style="font-size: 11.5px; color: var(--day-muted);">Select child blocks or use Style tab to customize this ${type}.</p>`;
            }
        }

        const numKeys = new Set(["level", "form_id", "height", "columns", "posts_per_page", "min_depth", "max_depth", "percent", "max_rating"]);
        container.querySelectorAll("input[data-key], textarea[data-key], select[data-key]").forEach(ctrl => {
            const k = ctrl.dataset.key;
            ctrl.oninput = () => {
                if (!node.data) node.data = {};
                node.data[k] = numKeys.has(k) ? (parseFloat(ctrl.value) || 0) : (k === "rating" ? (parseFloat(ctrl.value) || 0) : ctrl.value);
                if (node.type === "Icon" && k === "name") node.data.icon = ctrl.value;
                window.markDirty(true);
                window.renderCanvas();
            };
            ctrl.onchange = () => { if (window.pushHistorySnapshot) window.pushHistorySnapshot(); };
        });

        if (window.enhanceIconInput) container.querySelectorAll("input[data-icon-field='1']").forEach(inp => window.enhanceIconInput(inp));
        if (window.enhanceMediaInput) container.querySelectorAll("input[data-key='src'], input[data-key='image_url']").forEach(inp => window.enhanceMediaInput(inp));
    }

    function attachUniversalListeners(panel, node) {
        const inputId = panel.querySelector("#univ-node-id");
        if (inputId) {
            inputId.onchange = () => {
                const newId = inputId.value.trim();
                if (newId && newId !== node.id) {
                    node.id = newId;
                    window.DayStudioState.activeElementId = newId;
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
                if (!node.styles) node.styles = {};
                node.data.html_tag = selectTag.value;
                node.styles.html_tag = selectTag.value;
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
        const rstUniv = panel.querySelector("#btn-reset-universal");
        if (rstUniv) {
            rstUniv.onclick = (e) => {
                e.stopPropagation();
                if (node.data) delete node.data.html_tag;
                if (node.styles) {
                    delete node.styles.html_tag;
                    delete node.styles.hide_desktop;
                    delete node.styles.hide_tablet;
                    delete node.styles.hide_mobile;
                }
                window.markDirty(true);
                window.pushHistorySnapshot();
                window.renderCanvas();
                if (window.renderInspector) window.renderInspector();
            };
        }
    }

    function attachToggle(panel) {
        const h = panel.querySelector(".cad-panel-header");
        if (h) h.onclick = (e) => { if (!e.target.closest("button")) panel.classList.toggle("open"); };
    }
})();
"###;
