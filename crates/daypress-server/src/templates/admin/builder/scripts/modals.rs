//! Icon Library Picker, Page Settings, JSON AST Modal and Toast Controllers (`scripts/modals.rs`).

pub const JS_MODALS: &str = r###"
(function() {
    let selectedIconClass = null;

    const ICONS_DATA = [
        // Tech
        { cls: "fa-solid fa-code", name: "Code", cat: "tech" },
        { cls: "fa-solid fa-server", name: "Server", cat: "tech" },
        { cls: "fa-solid fa-database", name: "Database", cat: "tech" },
        { cls: "fa-solid fa-cloud", name: "Cloud", cat: "tech" },
        { cls: "fa-solid fa-bolt", name: "Lightning", cat: "tech" },
        { cls: "fa-solid fa-terminal", name: "Terminal", cat: "tech" },
        { cls: "fa-solid fa-microchip", name: "Chip", cat: "tech" },
        { cls: "fa-solid fa-shield-halved", name: "Shield", cat: "tech" },
        { cls: "fa-solid fa-lock", name: "Lock", cat: "tech" },
        { cls: "fa-solid fa-gear", name: "Gear", cat: "tech" },
        { cls: "fa-solid fa-gauge-high", name: "Speed", cat: "tech" },
        // Business
        { cls: "fa-solid fa-chart-line", name: "Analytics", cat: "business" },
        { cls: "fa-solid fa-credit-card", name: "Credit Card", cat: "business" },
        { cls: "fa-solid fa-coins", name: "Coins", cat: "business" },
        { cls: "fa-solid fa-briefcase", name: "Briefcase", cat: "business" },
        { cls: "fa-solid fa-award", name: "Award", cat: "business" },
        { cls: "fa-solid fa-trophy", name: "Trophy", cat: "business" },
        { cls: "fa-solid fa-building", name: "Building", cat: "business" },
        { cls: "fa-solid fa-shop", name: "Store", cat: "business" },
        // Social
        { cls: "fa-solid fa-envelope", name: "Envelope", cat: "social" },
        { cls: "fa-solid fa-phone", name: "Phone", cat: "social" },
        { cls: "fa-solid fa-comment", name: "Comment", cat: "social" },
        { cls: "fa-solid fa-bell", name: "Bell", cat: "social" },
        { cls: "fa-solid fa-user", name: "User", cat: "social" },
        { cls: "fa-solid fa-users", name: "Users", cat: "social" },
        { cls: "fa-solid fa-globe", name: "Globe", cat: "social" },
        // Arrows
        { cls: "fa-solid fa-arrow-right", name: "Arrow Right", cat: "arrows" },
        { cls: "fa-solid fa-arrow-left", name: "Arrow Left", cat: "arrows" },
        { cls: "fa-solid fa-arrow-up", name: "Arrow Up", cat: "arrows" },
        { cls: "fa-solid fa-arrow-down", name: "Arrow Down", cat: "arrows" },
        { cls: "fa-solid fa-check", name: "Check", cat: "arrows" },
        { cls: "fa-solid fa-xmark", name: "Cross", cat: "arrows" },
        { cls: "fa-solid fa-chevron-right", name: "Chevron", cat: "arrows" },
        // Brands
        { cls: "fa-brands fa-github", name: "GitHub", cat: "brands" },
        { cls: "fa-brands fa-x-twitter", name: "X", cat: "brands" },
        { cls: "fa-brands fa-linkedin", name: "LinkedIn", cat: "brands" },
        { cls: "fa-brands fa-discord", name: "Discord", cat: "brands" }
    ];

    function initModals() {
        initIconModal();
        initSettingsModal();
        initAstModal();
        initShortcutsModal();
    }

    function initIconModal() {
        const modal = document.getElementById("icon-library-modal");
        const grid = document.getElementById("icon-grid");
        const search = document.getElementById("icon-search");
        const tabs = document.querySelectorAll(".icon-tab");
        const btnClose = document.getElementById("btn-close-icon-library");
        const btnCancel = document.getElementById("btn-cancel-icon");
        const btnInsert = document.getElementById("btn-insert-icon");

        if (!modal || !grid) return;

        function renderIcons(cat, q) {
            grid.innerHTML = "";
            const filtered = ICONS_DATA.filter(ic => {
                const matchCat = cat === "all" || ic.cat === cat;
                const matchQ = !q || ic.name.toLowerCase().includes(q) || ic.cls.includes(q);
                return matchCat && matchQ;
            });

            filtered.forEach(ic => {
                const card = document.createElement("div");
                card.className = "icon-card" + (selectedIconClass === ic.cls ? " selected" : "");
                card.innerHTML = `<i class="${ic.cls}"></i><span class="icon-card-name">${ic.name}</span>`;
                card.onclick = () => {
                    grid.querySelectorAll(".icon-card").forEach(c => c.classList.remove("selected"));
                    card.classList.add("selected");
                    selectedIconClass = ic.cls;
                };
                grid.appendChild(card);
            });
        }

        tabs.forEach(tab => {
            tab.onclick = () => {
                tabs.forEach(t => t.classList.remove("active"));
                tab.classList.add("active");
                renderIcons(tab.dataset.cat, search ? search.value.trim().toLowerCase() : "");
            };
        });

        if (search) {
            search.oninput = () => {
                const activeTab = document.querySelector(".icon-tab.active");
                renderIcons(activeTab ? activeTab.dataset.cat : "all", search.value.trim().toLowerCase());
            };
        }

        if (btnClose) btnClose.onclick = () => modal.classList.remove("open");
        if (btnCancel) btnCancel.onclick = () => modal.classList.remove("open");

        if (btnInsert) {
            btnInsert.onclick = () => {
                if (selectedIconClass) {
                    const id = window.DayStudioState.activeElementId;
                    const node = id ? window.findNodeById(id) : null;
                    if (node) {
                        if (!node.data) node.data = {};
                        node.data.name = selectedIconClass;
                        node.data.icon = selectedIconClass;
                        window.markDirty(true);
                        window.renderCanvas();
                        if (window.renderInspector) window.renderInspector();
                    }
                    modal.classList.remove("open");
                }
            };
        }

        renderIcons("all", "");
    }

    function initSettingsModal() {
        const modal = document.getElementById("settings-modal");
        const btnClose = document.getElementById("btn-close-settings");
        const btnCancel = document.getElementById("btn-cancel-settings");
        const btnSave = document.getElementById("btn-save-settings");

        if (!modal) return;
        if (btnClose) btnClose.onclick = () => modal.classList.remove("open");
        if (btnCancel) btnCancel.onclick = () => modal.classList.remove("open");

        if (btnSave) {
            btnSave.onclick = () => {
                const titleInput = document.getElementById("set-page-title");
                if (titleInput && titleInput.value.trim()) {
                    window.DayStudioState.currentContext.title = titleInput.value.trim();
                    const lblTitle = document.getElementById("lbl-template-title");
                    if (lblTitle) lblTitle.textContent = titleInput.value.trim();
                }
                const bgInput = document.getElementById("set-canvas-bg-text");
                const artboard = document.getElementById("daystudio-artboard");
                if (bgInput && artboard) {
                    artboard.style.backgroundColor = bgInput.value.trim();
                }
                modal.classList.remove("open");
                window.markDirty(true);
                if (window.showToast) window.showToast("Page settings updated");
            };
        }
    }

    function initAstModal() {
        const modal = document.getElementById("ast-modal");
        const btnOpen = document.getElementById("btn-open-ast-modal");
        const btnClose = document.getElementById("btn-close-ast");
        const textarea = document.getElementById("ast-json-textarea");
        const btnCopy = document.getElementById("btn-copy-ast");
        const btnApply = document.getElementById("btn-apply-ast");

        if (!modal || !textarea) return;

        if (btnOpen) {
            btnOpen.onclick = () => {
                textarea.value = JSON.stringify(window.DayStudioState.pageAst, null, 2);
                modal.classList.add("open");
            };
        }

        if (btnClose) btnClose.onclick = () => modal.classList.remove("open");

        if (btnCopy) {
            btnCopy.onclick = () => {
                textarea.select();
                navigator.clipboard.writeText(textarea.value);
                if (window.showToast) window.showToast("Copied JSON to clipboard");
            };
        }

        if (btnApply) {
            btnApply.onclick = () => {
                try {
                    const parsed = JSON.parse(textarea.value);
                    window.DayStudioState.pageAst = window.normalizeAst(parsed);
                    window.pushHistorySnapshot();
                    window.renderCanvas();
                    if (window.renderInspector) window.renderInspector();
                    if (window.updateNavigator) window.updateNavigator();
                    modal.classList.remove("open");
                    if (window.showToast) window.showToast("AST recompiled successfully");
                } catch(e) {
                    alert("Invalid JSON: " + e.message);
                }
            };
        }
    }

    function initShortcutsModal() {
        const modal = document.getElementById("modal-shortcuts");
        const btnClose = document.getElementById("btn-close-shortcuts");
        if (modal && btnClose) {
            btnClose.onclick = () => modal.classList.remove("open");
            modal.onclick = (e) => { if (e.target === modal) modal.classList.remove("open"); };
        }
    }

    function showToast(msg) {
        const toast = document.getElementById("elementor-toast");
        const text = document.getElementById("toast-text");
        if (!toast || !text) return;
        text.textContent = msg;
        toast.classList.add("show");
        setTimeout(() => toast.classList.remove("show"), 2600);
    }
    window.showToast = showToast;

    document.addEventListener("DOMContentLoaded", () => {
        initModals();
        // Initial render
        setTimeout(() => {
            if (window.renderCanvas) window.renderCanvas();
            if (window.renderInspector) window.renderInspector();
        }, 80);
    });
})();
"###;
