//! Inspector input binding, live DOM mutation, icon picker modal, and element CRUD helpers.

pub const SCRIPT_INSPECTOR_EVENTS: &str = r###"    function setSampleImage(url) {
        const node = findNodeById(pageAst.root, activeElementId);
        if (!node) return;
        node.data = node.data || {};
        node.data.url = url;
        node.data.src = url;
        const input = document.getElementById("ctrl-img-url");
        if (input) input.value = url;
        renderCanvas();
    }

    function bindInspectorEvents(node) {
        const d = node.data;
        const s = node.styles;

        // Heading events
        const hText = document.getElementById("ctrl-heading-text");
        if (hText) hText.oninput = (e) => { d.text = e.target.value; renderCanvas(); };
        const hTag = document.getElementById("ctrl-heading-tag");
        if (hTag) hTag.onchange = (e) => { d.level = parseInt(e.target.value); renderCanvas(); };

        // Text events
        const tHtml = document.getElementById("ctrl-text-html");
        if (tHtml) tHtml.oninput = (e) => { d.content_html = e.target.value; renderCanvas(); };

        // Button events
        const bText = document.getElementById("ctrl-btn-text");
        if (bText) bText.oninput = (e) => { d.text = e.target.value; renderCanvas(); };
        const bUrl = document.getElementById("ctrl-btn-url");
        if (bUrl) bUrl.oninput = (e) => { d.url = e.target.value; renderCanvas(); };
        const bTarget = document.getElementById("ctrl-btn-target");
        if (bTarget) bTarget.onchange = (e) => { d.target = e.target.value; renderCanvas(); };

        // Image events
        const imgUrl = document.getElementById("ctrl-img-url");
        if (imgUrl) imgUrl.oninput = (e) => { d.url = e.target.value; d.src = e.target.value; renderCanvas(); };
        const imgAlt = document.getElementById("ctrl-img-alt");
        if (imgAlt) imgAlt.oninput = (e) => { d.alt = e.target.value; renderCanvas(); };
        const imgLink = document.getElementById("ctrl-img-link");
        if (imgLink) imgLink.oninput = (e) => { d.link = e.target.value; renderCanvas(); };

        // Video events
        const vidUrl = document.getElementById("ctrl-video-url");
        if (vidUrl) vidUrl.oninput = (e) => { d.url = e.target.value; renderCanvas(); };

        // Alert events
        const alType = document.getElementById("ctrl-alert-type");
        if (alType) alType.onchange = (e) => { d.alert_type = e.target.value; renderCanvas(); };
        const alTitle = document.getElementById("ctrl-alert-title");
        if (alTitle) alTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const alDesc = document.getElementById("ctrl-alert-desc");
        if (alDesc) alDesc.oninput = (e) => { d.description = e.target.value; renderCanvas(); };

        // Testimonial events
        const testContent = document.getElementById("ctrl-test-content");
        if (testContent) testContent.oninput = (e) => { d.content = e.target.value; renderCanvas(); };
        const testName = document.getElementById("ctrl-test-name");
        if (testName) testName.oninput = (e) => { d.name = e.target.value; renderCanvas(); };
        const testJob = document.getElementById("ctrl-test-job");
        if (testJob) testJob.oninput = (e) => { d.job = e.target.value; renderCanvas(); };
        const testAvatar = document.getElementById("ctrl-test-avatar");
        if (testAvatar) testAvatar.oninput = (e) => { d.image_url = e.target.value; renderCanvas(); };

        // Progress Bar events
        const pbTitle = document.getElementById("ctrl-pb-title");
        if (pbTitle) pbTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const pbRange = document.getElementById("ctrl-pb-percent-range");
        if (pbRange) {
            pbRange.oninput = (e) => {
                d.percent = parseInt(e.target.value);
                const label = document.getElementById("ctrl-pb-label");
                if (label) {
                    label.value = d.percent + "%";
                    d.display_text = label.value;
                }
                const labelRow = pbRange.parentElement.querySelector("label");
                if (labelRow) labelRow.textContent = `Percentage (${d.percent}%)`;
                renderCanvas();
            };
        }
        const pbLabel = document.getElementById("ctrl-pb-label");
        if (pbLabel) pbLabel.oninput = (e) => { d.display_text = e.target.value; renderCanvas(); };

        // Repeater (Accordion / Tabs) events
        document.querySelectorAll(".rep-title").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].title = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".rep-content").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].content = e.target.value;
                    d.items[idx].content_html = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".btn-rep-del").forEach(btn => {
            btn.onclick = (e) => {
                const idx = parseInt(btn.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items.splice(idx, 1);
                    renderCanvas();
                    renderInspector();
                }
            };
        });
        const btnRepAdd = document.getElementById("btn-rep-add");
        if (btnRepAdd) {
            btnRepAdd.onclick = () => {
                d.items = d.items || [];
                const num = d.items.length + 1;
                d.items.push({ title: "New Item " + num, content: "Content details for item " + num, content_html: "Content details for item " + num });
                renderCanvas();
                renderInspector();
            };
        }

        // Rating events
        const rVal = document.getElementById("ctrl-rating-val");
        if (rVal) rVal.oninput = (e) => { d.rating = parseFloat(e.target.value) || 5; renderCanvas(); };
        const rTitle = document.getElementById("ctrl-rating-title");
        if (rTitle) rTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };

        // Social Icons events
        document.querySelectorAll(".soc-icon-name").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].icon = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".soc-icon-link").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].link = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".btn-soc-del").forEach(btn => {
            btn.onclick = (e) => {
                const idx = parseInt(btn.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items.splice(idx, 1);
                    renderCanvas();
                    renderInspector();
                }
            };
        });
        const btnSocAdd = document.getElementById("btn-soc-add");
        if (btnSocAdd) {
            btnSocAdd.onclick = () => {
                d.items = d.items || [];
                d.items.push({ icon: "fa-brands fa-x-twitter", link: "#" });
                renderCanvas();
                renderInspector();
            };
        }

        // Icon List events
        document.querySelectorAll(".il-text").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].text = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".btn-il-del").forEach(btn => {
            btn.onclick = (e) => {
                const idx = parseInt(btn.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items.splice(idx, 1);
                    renderCanvas();
                    renderInspector();
                }
            };
        });
        const btnIlAdd = document.getElementById("btn-il-add");
        if (btnIlAdd) {
            btnIlAdd.onclick = () => {
                d.items = d.items || [];
                d.items.push({ icon: "fa-solid fa-check", text: "New benefit list point" });
                renderCanvas();
                renderInspector();
            };
        }

        // Image Box events
        const imgbUrl = document.getElementById("ctrl-imgbox-url");
        if (imgbUrl) imgbUrl.oninput = (e) => { d.image_url = e.target.value; d.url = e.target.value; renderCanvas(); };
        const imgbTitle = document.getElementById("ctrl-imgbox-title");
        if (imgbTitle) imgbTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const imgbDesc = document.getElementById("ctrl-imgbox-desc");
        if (imgbDesc) imgbDesc.oninput = (e) => { d.description = e.target.value; renderCanvas(); };
        const imgbLink = document.getElementById("ctrl-imgbox-link");
        if (imgbLink) imgbLink.oninput = (e) => { d.link = e.target.value; renderCanvas(); };

        // Grid events
        const gridCols = document.getElementById("ctrl-grid-cols");
        if (gridCols) gridCols.oninput = (e) => { d.columns = parseInt(e.target.value) || 3; renderCanvas(); };
        const gridGap = document.getElementById("ctrl-grid-gap");
        if (gridGap) gridGap.oninput = (e) => { d.gap = parseInt(e.target.value) || 0; renderCanvas(); };

        // Icon Box events
        const ibTitle = document.getElementById("ctrl-ib-title");
        if (ibTitle) ibTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const ibDesc = document.getElementById("ctrl-ib-desc");
        if (ibDesc) ibDesc.oninput = (e) => { d.description = e.target.value; renderCanvas(); };

        // Counter events
        const cntNum = document.getElementById("ctrl-cnt-number");
        if (cntNum) cntNum.oninput = (e) => { d.number = e.target.value; renderCanvas(); };
        const cntSuffix = document.getElementById("ctrl-cnt-suffix");
        if (cntSuffix) cntSuffix.oninput = (e) => { d.suffix = e.target.value; renderCanvas(); };
        const cntLabel = document.getElementById("ctrl-cnt-label");
        if (cntLabel) cntLabel.oninput = (e) => { d.label = e.target.value; d.title = e.target.value; renderCanvas(); };

        // PriceTable events
        const ptTitle = document.getElementById("ctrl-pt-title");
        if (ptTitle) ptTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const ptPrice = document.getElementById("ctrl-pt-price");
        if (ptPrice) ptPrice.oninput = (e) => { d.price = e.target.value; renderCanvas(); };
        const ptPeriod = document.getElementById("ctrl-pt-period");
        if (ptPeriod) ptPeriod.oninput = (e) => { d.period = e.target.value; renderCanvas(); };
        const ptBtnText = document.getElementById("ctrl-pt-btn-text");
        if (ptBtnText) ptBtnText.oninput = (e) => { d.button_text = e.target.value; renderCanvas(); };
        const ptBtnUrl = document.getElementById("ctrl-pt-btn-url");
        if (ptBtnUrl) ptBtnUrl.oninput = (e) => { d.button_url = e.target.value; renderCanvas(); };

        // Spacer events
        const spHeight = document.getElementById("ctrl-sp-height");
        if (spHeight) spHeight.oninput = (e) => {
            const hVal = parseInt(e.target.value) || 50;
            d.height = hVal;
            s.height = hVal + "px";
            renderCanvas();
        };

        // CustomHtml events
        const customHtmlEl = document.getElementById("ctrl-custom-html");
        if (customHtmlEl) customHtmlEl.oninput = (e) => {
            d.raw = e.target.value;
            renderCanvas();
        };

        // Divider events
        const divStyle = document.getElementById("ctrl-div-style");
        if (divStyle) divStyle.onchange = (e) => { d.style = e.target.value; renderCanvas(); };

        // Section / Container & Layout
        const secPad = document.getElementById("ctrl-sec-pad");
        if (secPad) secPad.oninput = (e) => {
            s.padding_top = e.target.value;
            s.padding_bottom = e.target.value;
            renderCanvas();
        };
        const contDir = document.getElementById("ctrl-cont-dir");
        if (contDir) contDir.onchange = (e) => {
            s.flex_direction = e.target.value;
            renderCanvas();
        };
        const contWidth = document.getElementById("ctrl-cont-width");
        if (contWidth) contWidth.oninput = (e) => {
            s.width = e.target.value;
            renderCanvas();
        };
        const contMinH = document.getElementById("ctrl-cont-minh");
        if (contMinH) contMinH.oninput = (e) => {
            s.min_height = e.target.value;
            renderCanvas();
        };
        const contBgImg = document.getElementById("ctrl-cont-bgimg");
        if (contBgImg) contBgImg.oninput = (e) => {
            s.background_image = e.target.value;
            if (s.background && typeof s.background === "object") {
                s.background.image_url = e.target.value;
            }
            renderCanvas();
        };

        // NativeForm & Benchmark
        const formTitle = document.getElementById("ctrl-form-title");
        if (formTitle) formTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const formBtn = document.getElementById("ctrl-form-btn");
        if (formBtn) formBtn.oninput = (e) => { d.button_text = e.target.value; renderCanvas(); };
        const benchText = document.getElementById("ctrl-bench-text");
        if (benchText) benchText.oninput = (e) => { d.text = e.target.value; renderCanvas(); };

        // Alignment buttons
        document.querySelectorAll("[data-align]").forEach(btn => {
            btn.onclick = () => {
                s.text_align = btn.dataset.align;
                renderInspector();
                renderCanvas();
            };
        });

        // Icon picker opener
        const openIconBtn = document.getElementById("ctrl-open-icon-picker");
        if (openIconBtn) {
            openIconBtn.onclick = () => {
                openIconLibrary(node.id);
            };
        }

        // Colors
        const colSwatch = document.getElementById("ctrl-color-swatch");
        const colInput = document.getElementById("ctrl-color-input");
        if (colSwatch && colInput) {
            colSwatch.oninput = (e) => {
                colInput.value = e.target.value;
                s.text_color = e.target.value;
                renderCanvas();
            };
            colInput.oninput = (e) => {
                s.text_color = e.target.value;
                renderCanvas();
            };
        }

        const bgSwatch = document.getElementById("ctrl-bg-swatch");
        const bgInput = document.getElementById("ctrl-bg-input");
        if (bgSwatch && bgInput) {
            bgSwatch.oninput = (e) => {
                bgInput.value = e.target.value;
                s.background_color = e.target.value;
                renderCanvas();
            };
            bgInput.oninput = (e) => {
                s.background_color = e.target.value;
                renderCanvas();
            };
        }

        // Typography
        const fsInput = document.getElementById("ctrl-font-size");
        if (fsInput) fsInput.oninput = (e) => { s.font_size = e.target.value; renderCanvas(); };
        const brInput = document.getElementById("ctrl-border-radius");
        if (brInput) brInput.oninput = (e) => { s.border_radius = e.target.value; renderCanvas(); };

        // Padding dimensions
        const padT = document.getElementById("ctrl-pad-t");
        const padR = document.getElementById("ctrl-pad-r");
        const padB = document.getElementById("ctrl-pad-b");
        const padL = document.getElementById("ctrl-pad-l");
        if (padT && padR && padB && padL) {
            padT.oninput = (e) => { s.padding_top = e.target.value; renderCanvas(); };
            padR.oninput = (e) => { s.padding_right = e.target.value; renderCanvas(); };
            padB.oninput = (e) => { s.padding_bottom = e.target.value; renderCanvas(); };
            padL.oninput = (e) => { s.padding_left = e.target.value; renderCanvas(); };
        }

        // Margin dimensions
        const marT = document.getElementById("ctrl-mar-t");
        const marR = document.getElementById("ctrl-mar-r");
        const marB = document.getElementById("ctrl-mar-b");
        const marL = document.getElementById("ctrl-mar-l");
        if (marT) marT.oninput = (e) => { s.margin_top = e.target.value; renderCanvas(); };
        if (marR) marR.oninput = (e) => { s.margin_right = e.target.value; renderCanvas(); };
        if (marB) marB.oninput = (e) => { s.margin_bottom = e.target.value; renderCanvas(); };
        if (marL) marL.oninput = (e) => { s.margin_left = e.target.value; renderCanvas(); };
    }

function selectElement(id) {
        activeElementId = id;
        renderCanvas();
        renderInspector();
        
        // Scroll canvas to element
        const el = document.getElementById(id);
        if (el) {
            el.scrollIntoView({ behavior: "smooth", block: "center" });
        }
    }

    function escapeHtml(str) {
        if (!str) return "";
        return String(str)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }

    // -------------------------------------------------------------
    // EXTENSIVE ICON LIBRARY MODAL LOGIC
    // -------------------------------------------------------------
    function openIconLibrary(targetNodeId) {
        selectedIconTarget = targetNodeId;
        const modal = document.getElementById("icon-library-modal");
        modal.classList.add("open");
        renderIconGrid("all", "");
    }

    function renderIconGrid(category, searchFilter) {
        const grid = document.getElementById("icon-grid");
        grid.innerHTML = "";
        const q = (searchFilter || "").toLowerCase();

        const filtered = ICON_LIBRARY.filter(item => {
            const matchCat = (category === "all" || item.cat === category);
            const matchSearch = (!q || item.name.toLowerCase().includes(q) || item.class.toLowerCase().includes(q));
            return matchCat && matchSearch;
        });

        filtered.forEach(item => {
            const card = document.createElement("div");
            card.className = "icon-card" + (tempSelectedIcon === item.class ? " selected" : "");
            card.innerHTML = `
                <i class="${item.class}"></i>
                <span>${item.name}</span>
            `;
            card.onclick = () => {
                tempSelectedIcon = item.class;
                grid.querySelectorAll(".icon-card").forEach(c => c.classList.remove("selected"));
                card.classList.add("selected");
            };
            grid.appendChild(card);
        });
    }

    document.getElementById("btn-close-icon-library").onclick = () => {
        document.getElementById("icon-library-modal").classList.remove("open");
    };
    document.getElementById("btn-cancel-icon").onclick = () => {
        document.getElementById("icon-library-modal").classList.remove("open");
    };

    document.getElementById("btn-insert-icon").onclick = () => {
        if (selectedIconTarget && tempSelectedIcon) {
            const node = findNodeById(pageAst.root, selectedIconTarget);
            if (node) {
                node.data = node.data || {};
                node.data.icon = tempSelectedIcon;
                renderCanvas();
                renderInspector();
            }
        }
        document.getElementById("icon-library-modal").classList.remove("open");
    };

    // Icon filter tabs
    document.querySelectorAll(".icon-tab").forEach(tab => {
        tab.onclick = () => {
            document.querySelectorAll(".icon-tab").forEach(t => t.classList.remove("active"));
            tab.classList.add("active");
            renderIconGrid(tab.dataset.cat, document.getElementById("icon-search").value);
        };
    });

    // Icon search input
    document.getElementById("icon-search").oninput = (e) => {
        const activeCat = document.querySelector(".icon-tab.active").dataset.cat;
        renderIconGrid(activeCat, e.target.value);
    };

    // -------------------------------------------------------------
    // SECTION & WIDGET MANIPULATION
    // -------------------------------------------------------------
    function insertSectionAbove(targetSectionId) {
        const newSec = createSectionWithColumns(1, [
            {
                id: generateId("h"),
                type: "Heading",
                data: { text: "New Section", level: 2 },
                styles: { font_size: "32px", text_color: "#0f172a" }
            }
        ]);

        let idx = pageAst.root.children.findIndex(c => c.id === targetSectionId);
        if (idx === -1) {
            pageAst.root.children.unshift(newSec);
        } else {
            pageAst.root.children.splice(idx, 0, newSec);
        }
        renderCanvas();
        showToast("New section created");
    }

    function deleteElement(id) {
        if (pageAst.root.id === id) {
            showToast("Cannot delete root section");
            return;
        }
        removeNodeById(pageAst.root, id);
        if (activeElementId === id) activeElementId = null;
        renderCanvas();
        renderInspector();
        showToast("Element removed");
    }

    function duplicateElement(id) {
        const node = findNodeById(pageAst.root, id);
        if (!node) return;
        const parent = findParentNode(pageAst.root, id);
        if (!parent || !parent.children) return;

        const clone = JSON.parse(JSON.stringify(node));
        function reId(n) {
            n.id = generateId(n.type.toLowerCase().substr(0, 4));
            if (n.children) n.children.forEach(reId);
        }
        reId(clone);

        const idx = parent.children.findIndex(c => c.id === id);
        parent.children.splice(idx + 1, 0, clone);
        renderCanvas();
        selectElement(clone.id);
        showToast("Element duplicated");
    }

    function insertWidgetAtRoot(type) {
        const newWidget = createWidgetNode(type);

        if (newWidget.type === "Section") {
            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newWidget);
            renderCanvas();
            updateNavigator();
            selectElement(newWidget.id);
            showToast("Added " + type + " block");
            savePage();
            return;
        }

        let container = findFirstContainer(pageAst.root);
        if (!container) {
            const newSec = createSectionWithColumns(1, [newWidget]);
            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newSec);
        } else {
            container.children = container.children || [];
            container.children.push(newWidget);
        }
        renderCanvas();
        updateNavigator();
        selectElement(newWidget.id);
        showToast("Added " + type + " widget");
        savePage();
    }

    function findFirstContainer(node) {
        if (!node) return null;
        if (node.type === "Container" || node.type === "Column" || node.type === "Flex") return node;
        if (node.children) {
            for (let c of node.children) {
                let res = findFirstContainer(c);
                if (res) return res;
            }
        }
        return null;
    }

"###;
