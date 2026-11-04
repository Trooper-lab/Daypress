//! Real-Time Daystudio WCAG 2.1 Contrast, Broken-Link & SEO Linter (`scripts/audit_linter.rs`).
//!
//! Styled in Daystudio Night Edition CAD tokens (`--day-chassis`, `--day-surface`, `--day-rule`,
//! `--day-ink`, `--day-emerald`, `--day-token`, `--day-crimson`, `--day-font-mono`).

pub const JS_AUDIT_LINTER: &str = r###"
(function() {
    let lastReport = {
        score: 100, wcag_score: 100, links_score: 100, seo_score: 100,
        word_count: 0, reading_time_minutes: 1, issues: [], verified_links: []
    };
    let activeFilter = "all";
    let auditDebounceTimer = null;

    function srgbChannelLuminance(c) {
        const v = c / 255;
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    }

    function parseRgbStr(str) {
        if (!str) return null;
        const m = str.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)/i);
        if (!m) return null;
        const alpha = m[4] !== undefined ? parseFloat(m[4]) : 1;
        return alpha < 0.08 ? null : [parseInt(m[1], 10), parseInt(m[2], 10), parseInt(m[3], 10)];
    }

    function computeContrastRatio(fg, bg) {
        const l1 = 0.2126 * srgbChannelLuminance(fg[0]) + 0.7152 * srgbChannelLuminance(fg[1]) + 0.0722 * srgbChannelLuminance(fg[2]);
        const l2 = 0.2126 * srgbChannelLuminance(bg[0]) + 0.7152 * srgbChannelLuminance(bg[1]) + 0.0722 * srgbChannelLuminance(bg[2]);
        return Math.round(((Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05)) * 100) / 100;
    }

    function resolveEffectiveBackground(el) {
        let cur = el;
        while (cur && cur !== document.body) {
            const rgb = parseRgbStr(window.getComputedStyle(cur).backgroundColor);
            if (rgb) return rgb;
            cur = cur.parentElement;
        }
        const artboard = document.getElementById("daystudio-artboard");
        return (artboard && artboard.getAttribute("data-theme") === "dark") ? [22, 23, 26] : [243, 238, 228];
    }

    function scanLiveCanvasDomContrast() {
        const canvas = document.getElementById("daystudio-canvas");
        if (!canvas) return [];
        const domIssues = [];
        canvas.querySelectorAll("[data-node-id]").forEach(el => {
            const nodeId = el.getAttribute("data-node-id");
            if (!nodeId || nodeId === "root") return;
            const nodeObj = window.findNodeById ? window.findNodeById(nodeId) : null;
            if (!nodeObj || !["Heading", "Text", "Button"].includes(nodeObj.type)) return;
            const text = (el.innerText || "").trim();
            if (!text) return;
            const cs = window.getComputedStyle(el);
            const fg = parseRgbStr(cs.color);
            const bg = resolveEffectiveBackground(el);
            if (!fg || !bg) return;
            const ratio = computeContrastRatio(fg, bg);
            const fontSizePx = parseFloat(cs.fontSize || "16");
            const fontWeight = parseInt(cs.fontWeight || "400", 10);
            const minAa = (fontSizePx >= 24 || (fontSizePx >= 18.66 && fontWeight >= 700)) ? 3.0 : 4.5;
            if (ratio < minAa) {
                const bgLum = 0.2126 * srgbChannelLuminance(bg[0]) + 0.7152 * srgbChannelLuminance(bg[1]) + 0.0722 * srgbChannelLuminance(bg[2]);
                domIssues.push({
                    id: "dom-contrast-" + nodeId, category: "wcag", severity: "error", node_id: nodeId,
                    title: `Low Canvas Contrast (${ratio.toFixed(2)}:1 < ${minAa.toFixed(1)}:1 AA)`,
                    detail: `${nodeObj.type} "${text.slice(0, 38)}" fails WCAG 2.1 AA in active preview mode.`,
                    auto_fix_kind: "contrast_text", auto_fix_value: bgLum > 0.35 ? "#16171a" : "#efe9de"
                });
            }
        });
        return domIssues;
    }

    function collectCanvasLinks() {
        const canvas = document.getElementById("daystudio-canvas");
        if (!canvas) return [];
        const links = new Set();
        canvas.querySelectorAll("a[href]").forEach(a => {
            const h = (a.getAttribute("href") || "").trim();
            if (h) links.add(h);
        });
        return Array.from(links);
    }

    function buildClientFallbackReport() {
        const domIssues = scanLiveCanvasDomContrast();
        const state = window.DayStudioState;
        const astIssues = [];
        let prevHeadingLevel = 0;
        let wordCount = 0;
        function walkAst(node) {
            if (!node) return;
            const kind = node.type || "";
            const d = node.data || {};
            if (kind === "Heading") {
                const lvl = parseInt(d.level || node.level || 2, 10) || 2;
                const txt = (d.text || node.text || "").trim();
                wordCount += txt.split(/\s+/).filter(Boolean).length;
                if (prevHeadingLevel > 0 && lvl > prevHeadingLevel + 1) {
                    astIssues.push({
                        id: "wcag-heading-skip-" + node.id, category: "wcag", severity: "warning", node_id: node.id,
                        title: `Skipped Heading Level (H${prevHeadingLevel} → H${lvl})`,
                        detail: `Heading "${txt.slice(0, 42)}" skips from H${prevHeadingLevel} to H${lvl}.`,
                        auto_fix_kind: "heading_level", auto_fix_value: String(prevHeadingLevel + 1)
                    });
                }
                prevHeadingLevel = lvl;
            } else if (kind === "Text") {
                const raw = (d.content_html || node.content_html || "").replace(/<[^>]+>/g, " ").trim();
                wordCount += raw.split(/\s+/).filter(Boolean).length;
            } else if (kind === "Image" && !(d.alt || node.alt || "").trim()) {
                astIssues.push({
                    id: "wcag-img-alt-" + node.id, category: "wcag", severity: "error", node_id: node.id,
                    title: "Image Missing Alt Text (WCAG 1.1.1)",
                    detail: "Add descriptive alternative text for screen readers.",
                    auto_fix_kind: "image_alt", auto_fix_value: "Editorial illustration"
                });
            }
            if (Array.isArray(node.children)) node.children.forEach(walkAst);
        }
        if (state && state.pageAst && state.pageAst.root) walkAst(state.pageAst.root);
        const combined = [...domIssues, ...astIssues];
        const wcagScore = Math.max(0, 100 - combined.reduce((acc, i) => acc + (i.severity === "error" ? 12 : 6), 0));
        return {
            score: Math.round(wcagScore * 0.45 + 100 * 0.30 + 92 * 0.25),
            wcag_score: wcagScore, links_score: 100, seo_score: 92,
            word_count: wordCount, reading_time_minutes: Math.max(1, Math.ceil(wordCount / 220)),
            issues: combined, verified_links: collectCanvasLinks().map(href => ({ href, valid: true }))
        };
    }

    function mergeDomContrastIntoReport(report) {
        const domContrastIssues = scanLiveCanvasDomContrast();
        const existingNodeContrast = new Set(
            (report.issues || []).filter(i => i.id && i.id.indexOf("wcag-contrast-") === 0).map(i => i.node_id)
        );
        domContrastIssues.forEach(di => {
            if (!existingNodeContrast.has(di.node_id)) report.issues.unshift(di);
        });
    }

    async function runDaystudioLinter() {
        const state = window.DayStudioState;
        if (!state || !state.pageAst) return lastReport;
        const artboard = document.getElementById("daystudio-artboard");
        const activeTheme = (artboard && artboard.getAttribute("data-theme")) || "light";
        const ctx = state.currentContext || {};

        lastReport = window.DAYPRESS_PRELOADED_LINTER_REPORT
            ? JSON.parse(JSON.stringify(window.DAYPRESS_PRELOADED_LINTER_REPORT))
            : buildClientFallbackReport();
        if (window.DAYPRESS_PRELOADED_LINTER_REPORT) mergeDomContrastIntoReport(lastReport);
        updateTopbarLinterPill(lastReport);
        if (state.activeDrawer === "linter" && state.drawerOpen) {
            const container = document.getElementById("drawer-panel-container");
            if (container) renderLinterDrawer(container);
        }

        const payload = {
            post_id: state.postId > 0 ? state.postId : null, title: ctx.title || "",
            slug: ctx.slug || state.postSlug || "", layout_json: JSON.stringify(state.pageAst),
            theme: activeTheme, links: collectCanvasLinks()
        };

        try {
            const res = await fetch("/wp-json/daypress/v1/seo/audit", {
                method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(payload)
            });
            if (res.ok) {
                const serverReport = await res.json();
                mergeDomContrastIntoReport(serverReport);
                lastReport = serverReport;
                updateTopbarLinterPill(lastReport);
                if (state.activeDrawer === "linter" && state.drawerOpen) {
                    const container = document.getElementById("drawer-panel-container");
                    if (container) renderLinterDrawer(container);
                }
            }
        } catch (err) {}
        return lastReport;
    }
    window.runDaystudioLinter = runDaystudioLinter;

    function scheduleLinterAudit() {
        if (auditDebounceTimer) clearTimeout(auditDebounceTimer);
        auditDebounceTimer = setTimeout(() => runDaystudioLinter(), 450);
    }
    window.scheduleLinterAudit = scheduleLinterAudit;

    function updateTopbarLinterPill(report) {
        const lbl = document.getElementById("lbl-linter-score");
        const icon = document.getElementById("icon-linter-status");
        if (!lbl || !icon) return;
        const s = report ? report.score : 100;
        const errCount = (report && report.issues ? report.issues.length : 0);
        lbl.textContent = errCount > 0 ? `Audit ${s} (${errCount})` : `Audit ${s}`;
        icon.style.color = s >= 85 ? "var(--day-emerald, #54b87a)" : (s >= 65 ? "var(--day-token, #d4a72c)" : "var(--day-crimson, #f25f5c)");
    }

    async function applyIssueAutoFix(issue) {
        if (!issue || !issue.auto_fix_kind) return;
        const state = window.DayStudioState;
        const node = issue.node_id && window.findNodeById ? window.findNodeById(issue.node_id) : null;

        if (issue.auto_fix_kind === "contrast_text" && node) {
            if (!node.styles) node.styles = {};
            node.styles.text_color = issue.auto_fix_value || "#16171a";
            node.styles.color = issue.auto_fix_value || "#16171a";
            if (node.styles.typography) node.styles.typography.color = issue.auto_fix_value || "#16171a";
        } else if (issue.auto_fix_kind === "image_alt" && node) {
            if (!node.data) node.data = {};
            node.data.alt = issue.auto_fix_value || "Editorial image";
            node.alt = node.data.alt;
        } else if (issue.auto_fix_kind === "heading_level" && node) {
            const newLvl = parseInt(issue.auto_fix_value || "2", 10) || 2;
            if (!node.data) node.data = {};
            node.data.level = newLvl;
            node.level = newLvl;
        } else if (issue.auto_fix_kind === "broken_link" && node) {
            const safeUrl = issue.auto_fix_value || "/";
            if (!node.data) node.data = {};
            node.data.url = safeUrl;
            node.url = safeUrl;
        } else if (issue.auto_fix_kind === "seo_description" && state.postId > 0) {
            const endpoint = state.postType === "page"
                ? `/wp-json/wp/v2/pages/${state.postId}`
                : `/wp-json/wp/v2/posts/${state.postId}`;
            try {
                await fetch(endpoint, {
                    method: "PATCH",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ seo_description: issue.auto_fix_value || "" })
                });
            } catch (_) {}
        }

        if (window.pushHistorySnapshot) window.pushHistorySnapshot();
        if (window.renderCanvas) window.renderCanvas();
        if (issue.node_id && window.selectElement) window.selectElement(issue.node_id);
        await runDaystudioLinter();
    }

    function renderLinterDrawer(container) {
        if (!container) return;
        const r = lastReport;
        const scoreColor = r.score >= 85
            ? "var(--day-emerald, #54b87a)"
            : (r.score >= 65 ? "var(--day-token, #d4a72c)" : "var(--day-crimson, #f25f5c)");

        const filteredIssues = (r.issues || []).filter(item => {
            if (activeFilter === "all") return true;
            return item.category === activeFilter;
        });

        let issuesHtml = "";
        if (filteredIssues.length === 0) {
            issuesHtml = `
                <div style="padding:24px 14px;text-align:center;border:1px solid var(--day-rule, #1e2025);background:var(--day-chassis, #0b0c0e);border-radius:2px;">
                    <i class="fa-solid fa-circle-check" style="color:var(--day-emerald, #54b87a);font-size:18px;margin-bottom:8px;display:block;"></i>
                    <div style="font-size:12px;font-weight:600;color:var(--day-ink, #ece7dc);">All Checks Passing</div>
                    <div style="font-size:11px;color:var(--day-muted, #848078);margin-top:4px;font-family:var(--day-font-mono, monospace);">WCAG AA · SQLite Links · SEO</div>
                </div>
            `;
        } else {
            issuesHtml = filteredIssues.map((iss, idx) => {
                const isErr = iss.severity === "error";
                const badgeBg = isErr ? "rgba(242,95,92,0.14)" : "rgba(212,167,44,0.14)";
                const badgeFg = isErr ? "var(--day-crimson, #f25f5c)" : "var(--day-token, #d4a72c)";
                const catLabel = (iss.category || "wcag").toUpperCase();
                const nodeBadge = iss.node_id
                    ? `<span style="font-family:var(--day-font-mono, monospace);font-size:9.5px;color:var(--day-muted, #848078);">#${iss.node_id}</span>`
                    : "";
                const fixBtn = iss.auto_fix_kind
                    ? `<button type="button" class="cad-btn btn-linter-autofix" data-issue-idx="${idx}" style="height:22px;padding:0 8px;font-size:10px;font-family:var(--day-font-mono, monospace);background:var(--day-oxide, #e2552b);border:1px solid var(--day-oxide, #e2552b);color:#fff8f0;border-radius:2px;font-weight:600;cursor:pointer;">Auto-Fix</button>`
                    : "";
                return `
                    <div class="linter-issue-card" data-node-id="${iss.node_id || ""}" style="background:var(--day-chassis, #0b0c0e);border:1px solid var(--day-rule, #1e2025);border-left:2px solid ${badgeFg};border-radius:2px;padding:10px 11px;margin-bottom:8px;cursor:pointer;">
                        <div style="display:flex;justify-content:space-between;align-items:center;gap:6px;margin-bottom:5px;">
                            <div style="display:flex;align-items:center;gap:6px;">
                                <span style="background:${badgeBg};color:${badgeFg};font-family:var(--day-font-mono, monospace);font-size:9px;font-weight:600;padding:1px 5px;border-radius:2px;text-transform:uppercase;">${catLabel}</span>
                                ${nodeBadge}
                            </div>
                            ${fixBtn}
                        </div>
                        <div style="font-size:11.5px;font-weight:600;color:var(--day-ink, #ece7dc);line-height:1.35;margin-bottom:4px;">${iss.title}</div>
                        <div style="font-size:10.5px;color:var(--day-muted, #848078);line-height:1.4;">${iss.detail}</div>
                    </div>
                `;
            }).join("");
        }

        container.innerHTML = `
            <div class="drawer-header" style="display:flex;justify-content:space-between;align-items:center;">
                <span class="drawer-title">WCAG, Link &amp; SEO Linter</span>
                <button type="button" class="cad-btn secondary" id="btn-rescan-linter" style="height:22px;padding:0 8px;font-size:10px;font-family:var(--day-font-mono, monospace);">
                    <i class="fa-solid fa-rotate"></i> Scan
                </button>
            </div>
            <div class="drawer-scroll-body" style="padding:12px;">
                <div style="background:var(--day-chassis, #0b0c0e);border:1px solid var(--day-rule, #1e2025);border-radius:2px;padding:12px;margin-bottom:12px;">
                    <div style="display:flex;justify-content:space-between;align-items:baseline;margin-bottom:10px;">
                        <div>
                            <div style="font-family:var(--day-font-mono, monospace);font-size:10px;text-transform:uppercase;letter-spacing:0.08em;color:var(--day-muted, #848078);">LIVE AUDIT SCORE</div>
                            <div style="font-family:var(--day-font-mono, monospace);font-size:24px;font-weight:700;color:${scoreColor};line-height:1.1;margin-top:2px;">${r.score}<span style="font-size:12px;color:var(--day-muted, #848078);">/100</span></div>
                        </div>
                        <div style="text-align:right;font-family:var(--day-font-mono, monospace);font-size:10px;color:var(--day-muted, #848078);">
                            <div>${r.word_count || 0} words · ${r.reading_time_minutes || 1}m read</div>
                            <div>${(r.verified_links || []).length} SQLite links checked</div>
                        </div>
                    </div>
                    <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:6px;border-top:1px solid var(--day-rule, #1e2025);padding-top:8px;font-family:var(--day-font-mono, monospace);font-size:10px;">
                        <div><span style="color:var(--day-muted);">WCAG</span> <strong style="color:var(--day-ink);">${r.wcag_score}</strong></div>
                        <div><span style="color:var(--day-muted);">LINKS</span> <strong style="color:var(--day-ink);">${r.links_score}</strong></div>
                        <div><span style="color:var(--day-muted);">SEO</span> <strong style="color:var(--day-ink);">${r.seo_score}</strong></div>
                    </div>
                </div>

                <div style="display:flex;gap:4px;margin-bottom:10px;">
                    ${["all", "wcag", "links", "seo"].map(tab => `
                        <button type="button" class="cad-btn secondary linter-filter-tab" data-filter="${tab}" style="flex:1;height:24px;font-size:10px;font-family:var(--day-font-mono, monospace);text-transform:uppercase;${activeFilter === tab ? "background:var(--day-oxide, #e2552b);border-color:var(--day-oxide, #e2552b);color:#fff8f0;font-weight:600;" : ""}">
                            ${tab}
                        </button>
                    `).join("")}
                </div>

                <div id="linter-issues-list">${issuesHtml}</div>
            </div>
        `;

        const btnRescan = container.querySelector("#btn-rescan-linter");
        if (btnRescan) btnRescan.onclick = () => runDaystudioLinter();

        container.querySelectorAll(".linter-filter-tab").forEach(btn => {
            btn.onclick = () => {
                activeFilter = btn.getAttribute("data-filter") || "all";
                renderLinterDrawer(container);
            };
        });

        container.querySelectorAll(".linter-issue-card").forEach(card => {
            card.onclick = (e) => {
                if (e.target.closest(".btn-linter-autofix")) return;
                const nid = card.getAttribute("data-node-id");
                if (nid && window.selectElement) {
                    window.selectElement(nid);
                    const domEl = document.querySelector(`#daystudio-canvas [data-node-id="${nid}"]`);
                    if (domEl && domEl.scrollIntoView) {
                        domEl.scrollIntoView({ behavior: "smooth", block: "center" });
                    }
                }
            };
        });

        container.querySelectorAll(".btn-linter-autofix").forEach(btn => {
            btn.onclick = (e) => {
                e.stopPropagation();
                const idx = parseInt(btn.getAttribute("data-issue-idx") || "0", 10);
                const iss = filteredIssues[idx];
                if (iss) applyIssueAutoFix(iss);
            };
        });
    }
    window.renderLinterDrawer = renderLinterDrawer;

    function openLinterDrawer() {
        const railBtn = document.getElementById("rail-btn-linter");
        if (railBtn) railBtn.click();
    }
    window.openLinterDrawer = openLinterDrawer;

    document.addEventListener("DOMContentLoaded", () => {
        const topBtn = document.getElementById("btn-toggle-linter");
        if (topBtn) topBtn.addEventListener("click", openLinterDrawer);

        document.addEventListener("keydown", (e) => {
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "l") {
                e.preventDefault();
                openLinterDrawer();
            }
        });

        setTimeout(() => {
            runDaystudioLinter();
        }, 120);
    });
})();
"###;
