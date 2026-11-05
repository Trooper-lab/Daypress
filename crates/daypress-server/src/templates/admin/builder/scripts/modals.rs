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
        initRevisionDiffModal();
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

    function initRevisionDiffModal() {
        const modal = document.getElementById("modal-revision-diff");
        const btnClose = document.getElementById("btn-close-revision-diff");
        const btnCancel = document.getElementById("btn-cancel-revision-diff");
        const btnRestoreFull = document.getElementById("btn-restore-full-revision");
        const revSelect = document.getElementById("diff-rev-select");
        if (!modal) return;

        if (btnClose) btnClose.onclick = () => modal.classList.remove("open");
        if (btnCancel) btnCancel.onclick = () => modal.classList.remove("open");
        if (revSelect) {
            revSelect.onchange = () => {
                const revId = parseInt(revSelect.value, 10);
                if (revId) loadRevisionDiffData(revId);
            };
        }
        if (btnRestoreFull) {
            btnRestoreFull.onclick = async () => {
                const pid = window.DayStudioState && window.DayStudioState.currentContext ? window.DayStudioState.currentContext.id : 0;
                const revId = revSelect ? parseInt(revSelect.value, 10) : 0;
                if (!pid || !revId) return;
                const res = await fetch(`/wp-json/wp/v2/posts/${pid}/revisions/${revId}`, { method: "POST" });
                if (res.ok) {
                    showToast(`Restored Revision #${revId} — reloading canvas...`);
                    setTimeout(() => window.location.reload(), 400);
                }
            };
        }

        const params = new URLSearchParams(window.location.search);
        const diffRevParam = parseInt(params.get("diff_rev") || "0", 10);
        if (diffRevParam > 0) {
            setTimeout(() => openRevisionDiffModal(diffRevParam), 250);
        }
    }

    async function openRevisionDiffModal(initialRevId) {
        const modal = document.getElementById("modal-revision-diff");
        const revSelect = document.getElementById("diff-rev-select");
        const container = document.getElementById("diff-nodes-container");
        if (!modal || !revSelect || !container) return;
        modal.classList.add("open");

        const pid = window.DayStudioState && window.DayStudioState.currentContext ? window.DayStudioState.currentContext.id : 0;
        if (!pid) return;
        try {
            const res = await fetch(`/wp-json/wp/v2/posts/${pid}/revisions`);
            const revs = res.ok ? await res.json() : [];
            if (!Array.isArray(revs) || revs.length === 0) {
                revSelect.innerHTML = `<option value="">No revisions recorded yet</option>`;
                container.innerHTML = `<div style="padding:24px;text-align:center;color:var(--day-muted);">Save or publish changes to record a revision snapshot first.</div>`;
                return;
            }
            const targetRev = initialRevId || revs[0].id;
            revSelect.innerHTML = revs.map(r => {
                const dt = (r.date || "").replace("T", " ");
                const ttl = (r.title && r.title.rendered) ? r.title.rendered : `Revision #${r.id}`;
                return `<option value="${r.id}" ${r.id === targetRev ? "selected" : ""}>#${r.id} — ${ttl} (${dt})</option>`;
            }).join("");
            await loadRevisionDiffData(targetRev);
        } catch (e) {
            container.innerHTML = `<div style="padding:24px;color:var(--day-crimson);">Error loading revisions: ${e}</div>`;
        }
    }
    window.openRevisionDiffModal = openRevisionDiffModal;

    async function loadRevisionDiffData(revId) {
        const pid = window.DayStudioState && window.DayStudioState.currentContext ? window.DayStudioState.currentContext.id : 0;
        const badges = document.getElementById("diff-summary-badges");
        const container = document.getElementById("diff-nodes-container");
        if (!pid || !revId || !container) return;
        container.innerHTML = `<div style="padding:18px;text-align:center;color:var(--day-muted);">Computing structural AST diff against Revision #${revId}...</div>`;
        try {
            const res = await fetch(`/wp-json/daypress/v1/posts/${pid}/revisions/diff?rev=${revId}`);
            if (!res.ok) {
                container.innerHTML = `<div style="padding:18px;color:var(--day-crimson);">Failed to compute revision diff.</div>`;
                return;
            }
            const report = await res.json();
            if (badges) {
                badges.innerHTML = `
                    <span style="color:#4ade80;background:rgba(74,222,128,0.12);padding:3px 8px;border-radius:4px;">+${report.added_count || 0} Added</span>
                    <span style="color:#fbbf24;background:rgba(251,191,36,0.12);padding:3px 8px;border-radius:4px;">~${report.modified_count || 0} Modified</span>
                    <span style="color:#f87171;background:rgba(248,113,113,0.12);padding:3px 8px;border-radius:4px;">-${report.removed_count || 0} Removed</span>
                    <span style="color:var(--day-muted);padding:3px 8px;">=${report.unchanged_count || 0} Unchanged</span>
                `;
            }
            const nodes = report.nodes || [];
            if (nodes.length === 0) {
                container.innerHTML = `<div style="padding:24px;text-align:center;color:var(--day-emerald,#4ade80);font-weight:600;">✓ Live layout is 100% identical to Revision #${revId}.</div>`;
                return;
            }
            container.innerHTML = nodes.map(n => {
                const statusColor = n.status === "added" ? "#4ade80" : (n.status === "removed" ? "#f87171" : "#fbbf24");
                const changesHtml = (n.changes || []).map(c => `
                    <div style="display:grid;grid-template-columns:130px 1fr 1fr;gap:8px;padding:6px 8px;border-top:1px solid rgba(255,255,255,0.06);font-family:var(--day-font-mono);font-size:11px;">
                        <span style="color:var(--day-muted);">${c.field}</span>
                        <span style="color:#f87171;word-break:break-all;">- ${(c.old_value || "∅").slice(0, 120)}</span>
                        <span style="color:#4ade80;word-break:break-all;">+ ${(c.new_value || "∅").slice(0, 120)}</span>
                    </div>
                `).join("");
                const canCherryPick = n.status !== "added";
                return `<div style="border:1px solid var(--day-rule,#393836);border-radius:8px;background:var(--day-surface-elevated,#1b1d21);overflow:hidden;">
                    <div style="display:flex;justify-content:space-between;align-items:center;padding:8px 12px;">
                        <div style="display:flex;align-items:center;gap:8px;">
                            <span style="font-size:10px;font-weight:700;text-transform:uppercase;padding:2px 7px;border-radius:4px;color:${statusColor};border:1px solid ${statusColor};">${n.status}</span>
                            <strong style="font-size:12.5px;color:var(--day-ink);">${n.kind}</strong>
                            <code style="font-size:11px;color:var(--day-muted);">#${n.node_id}</code>
                            <span style="font-size:11.5px;color:var(--day-muted);">${(n.label || "").slice(0, 48)}</span>
                        </div>
                        ${canCherryPick ? `<button type="button" class="cad-btn secondary" style="height:24px;font-size:11px;" onclick="window.cherryPickRevisionNode(${revId}, '${n.node_id}')"><i class="fa-solid fa-wand-magic-sparkles"></i> Cherry-Pick Node</button>` : ""}
                    </div>
                    ${changesHtml}
                </div>`;
            }).join("");
        } catch (e) {
            container.innerHTML = `<div style="padding:18px;color:var(--day-crimson);">Diff error: ${e}</div>`;
        }
    }

    async function cherryPickRevisionNode(revId, nodeId) {
        const pid = window.DayStudioState && window.DayStudioState.currentContext ? window.DayStudioState.currentContext.id : 0;
        if (!pid || !revId || !nodeId) return;
        const res = await fetch(`/wp-json/daypress/v1/posts/${pid}/revisions/${revId}/cherry-pick`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ node_id: nodeId })
        });
        if (res.ok) {
            showToast(`✓ Cherry-picked node #${nodeId} from Revision #${revId}! Reloading...`);
            setTimeout(() => window.location.reload(), 400);
        } else {
            showToast("Failed to cherry-pick node from revision");
        }
    }
    window.cherryPickRevisionNode = cherryPickRevisionNode;

    function showToast(msg) {
        const toast = document.getElementById("daystudio-toast");
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
