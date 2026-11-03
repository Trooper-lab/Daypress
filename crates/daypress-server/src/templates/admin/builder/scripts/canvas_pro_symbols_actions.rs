//! DayStudio 0.5 Global Symbol Actions: Convert to Symbol, Collect Overrideable Nodes & Detach (`scripts/canvas_pro_symbols_actions.rs`).

pub const JS_CANVAS_PRO_SYMBOLS_ACTIONS: &str = r###"
(function() {
    function collectSymbolOverrideableNodes(children, out) {
        if (!Array.isArray(children)) return out || [];
        const list = out || [];
        children.forEach(node => {
            if (!node || typeof node !== "object") return;
            const d = node.data || {};
            if (["Heading", "Text", "Button", "Image"].includes(node.type)) {
                list.push({
                    id: node.id,
                    type: node.type,
                    text: node.type === "Text" ? (d.content_html || "") : (d.text || ""),
                    src: d.src || "",
                    alt: d.alt || "",
                    url: d.url || ""
                });
            }
            if (Array.isArray(node.children)) {
                collectSymbolOverrideableNodes(node.children, list);
            }
        });
        return list;
    }
    window.collectSymbolOverrideableNodes = collectSymbolOverrideableNodes;

    function bakeOverridesIntoTree(node, overrides) {
        if (!node || typeof node !== "object") return node;
        const origId = String(node.id || "");
        const ov = (overrides && origId && overrides[origId]) ? overrides[origId] : null;
        if (!node.data) node.data = {};
        if (ov && typeof ov === "object") {
            if (ov.text) {
                if (node.type === "Text") node.data.content_html = ov.text;
                else node.data.text = ov.text;
            }
            if (ov.src && node.type === "Image") node.data.src = ov.src;
            if (ov.alt && node.type === "Image") node.data.alt = ov.alt;
            if (ov.url) node.data.url = ov.url;
        }
        if (node.id && node.id !== "root") {
            node.id = window.generateId ? window.generateId(String(node.type || "el").toLowerCase()) : (node.id + "-det-" + Math.random().toString(36).slice(2, 6));
        }
        if (Array.isArray(node.children)) {
            node.children.forEach(child => bakeOverridesIntoTree(child, overrides));
        }
        return node;
    }

    async function convertActiveNodeToSymbol(nodeOrId) {
        const state = window.DayStudioState;
        const targetNode = (typeof nodeOrId === "string" ? (window.findNodeById && window.findNodeById(nodeOrId)) : nodeOrId)
            || (state && state.activeElementId && window.findNodeById ? window.findNodeById(state.activeElementId) : null);

        if (!targetNode || targetNode.id === "root" || targetNode.type === "Root") {
            if (window.showToast) window.showToast("Select a Section, Container, or Block on the canvas first", "warning");
            return;
        }
        const defaultName = (targetNode.data && (targetNode.data._name || targetNode.data.label || targetNode.data.title))
            || `${targetNode.type} Symbol (${targetNode.id})`;
        const symbolTitle = window.prompt("Name for this Synced Global Symbol:", defaultName);
        if (!symbolTitle || !symbolTitle.trim()) return;

        const clonedNode = JSON.parse(JSON.stringify(targetNode));
        const symbolLayout = {
            version: 1,
            root: { id: "root", type: "Root", children: [clonedNode] }
        };

        try {
            const res = await fetch("/wp-json/daypress/v1/templates", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                credentials: "same-origin",
                body: JSON.stringify({
                    title: symbolTitle.trim(),
                    template_type: "section_symbol",
                    status: "publish",
                    is_active: true,
                    priority: 10,
                    conditions: [],
                    layout: symbolLayout
                })
            });
            if (!res.ok) throw new Error("Server returned " + res.status);
            const created = await res.json();
            if (window.loadSymbolTemplatesCache) await window.loadSymbolTemplatesCache(true);

            const symInstanceNode = {
                id: window.generateId ? window.generateId("symbol") : ("sym-" + Date.now().toString(36)),
                type: "SymbolInstance",
                data: {
                    symbol_id: Number(created.id),
                    label: created.title || symbolTitle.trim(),
                    overrides: {}
                },
                styles: {},
                children: [JSON.parse(JSON.stringify(clonedNode))]
            };

            const parent = window.findParentNode ? window.findParentNode(targetNode.id) : null;
            if (parent && Array.isArray(parent.children)) {
                const idx = parent.children.findIndex(c => c.id === targetNode.id);
                if (idx !== -1) parent.children.splice(idx, 1, symInstanceNode);
            }

            if (window.markDirty) window.markDirty(true);
            if (window.pushHistorySnapshot) window.pushHistorySnapshot();
            if (window.renderCanvas) window.renderCanvas();
            if (window.updateNavigator) window.updateNavigator();
            if (window.selectElement) window.selectElement(symInstanceNode.id);
            if (window.showToast) window.showToast(`Saved Global Symbol "${created.title || symbolTitle.trim()}" (#${created.id})`);
        } catch (err) {
            if (window.showToast) window.showToast("Failed to save global symbol: " + err.message, "error");
        }
    }
    window.convertActiveNodeToSymbol = convertActiveNodeToSymbol;
    window.convertToGlobalSymbol = (nodeId) => convertActiveNodeToSymbol(nodeId);

    async function detachSymbolInstance(nodeOrId) {
        const state = window.DayStudioState;
        const instNode = (typeof nodeOrId === "string" ? (window.findNodeById && window.findNodeById(nodeOrId)) : nodeOrId)
            || (state && state.activeElementId && window.findNodeById ? window.findNodeById(state.activeElementId) : null);

        if (!instNode || instNode.type !== "SymbolInstance") {
            if (window.showToast) window.showToast("Select a SymbolInstance node to detach", "warning");
            return;
        }

        const list = window.loadSymbolTemplatesCache ? await window.loadSymbolTemplatesCache(false) : [];
        const symbolId = Number((instNode.data && instNode.data.symbol_id) || instNode.symbol_id || 0);
        const overrides = (instNode.data && instNode.data.overrides) || instNode.overrides || {};
        const symTpl = list.find(s => Number(s.id) === symbolId);

        let masterChildren = [];
        if (Array.isArray(instNode.children) && instNode.children.length > 0) {
            masterChildren = JSON.parse(JSON.stringify(instNode.children));
        } else if (symTpl && symTpl.layout && symTpl.layout.root && Array.isArray(symTpl.layout.root.children)) {
            masterChildren = JSON.parse(JSON.stringify(symTpl.layout.root.children));
        }
        if (masterChildren.length === 0) {
            masterChildren = [{
                id: window.generateId ? window.generateId("sec") : ("sec-" + Date.now().toString(36)),
                type: "Section",
                data: { _name: "Detached Symbol Section" },
                styles: { padding_top: "32px", padding_bottom: "32px" },
                children: []
            }];
        }

        const bakedChildren = masterChildren.map(child => bakeOverridesIntoTree(child, overrides));
        const parent = window.findParentNode ? window.findParentNode(instNode.id) : null;
        if (parent && Array.isArray(parent.children)) {
            const idx = parent.children.findIndex(c => c.id === instNode.id);
            if (idx !== -1) parent.children.splice(idx, 1, ...bakedChildren);
        }

        if (window.markDirty) window.markDirty(true);
        if (window.pushHistorySnapshot) window.pushHistorySnapshot();
        if (window.renderCanvas) window.renderCanvas();
        if (window.updateNavigator) window.updateNavigator();
        if (window.selectElement && bakedChildren[0]) window.selectElement(bakedChildren[0].id);
        if (window.showToast) window.showToast("Detached SymbolInstance into independent blocks");
    }
    window.detachSymbolInstance = detachSymbolInstance;

    function uid(prefix) {
        return window.generateId
            ? window.generateId(prefix)
            : (prefix + "_" + Math.random().toString(36).slice(2, 8));
    }

    function makeFeatureCard(icon, title, desc, url) {
        return {
            id: uid("cont"),
            type: "Container",
            data: { html_tag: "a" },
            styles: {
                display: "flex", flex_direction: "row", align_items: "flex-start", gap: "12px",
                padding_top: "10px", padding_right: "12px", padding_bottom: "10px", padding_left: "12px",
                border_radius: "8px", background_color: "rgba(255,255,255,0.03)"
            },
            children: [
                {
                    id: uid("icon"), type: "Icon", data: { name: icon, icon: icon, url: url || "#" },
                    styles: { text_color: "#60a5fa", font_size: "16px", margin_top: "2px" }, children: []
                },
                {
                    id: uid("col"), type: "Column", data: {}, styles: { gap: "3px" },
                    children: [
                        {
                            id: uid("h"), type: "Heading", data: { text: title, level: 5 },
                            styles: { text_color: "#f8fafc", font_size: "13.5px", font_weight: "700", margin_bottom: "0px" }, children: []
                        },
                        {
                            id: uid("t"), type: "Text", data: { content_html: `<p>${desc}</p>` },
                            styles: { text_color: "#94a3b8", font_size: "12px", line_height: "1.45" }, children: []
                        }
                    ]
                }
            ]
        };
    }
    window.makeMegaMenuFeatureCard = makeFeatureCard;

    function buildMegaMenuPresetChildren(presetKey, itemTitle) {
        const title = itemTitle || "Platform";
        if (presetKey === "enterprise_4col") {
            const cols = [
                { head: "CORE PLATFORM", items: [["fa-solid fa-bolt", "Edge Runtime", "Sub-millisecond Rust SSR"], ["fa-solid fa-database", "SQLite Engine", "Zero-dependency persistence"]] },
                { head: "SOLUTIONS", items: [["fa-solid fa-shield-halved", "Zero-Trust Security", "TOTP 2FA & RBAC policies"], ["fa-solid fa-cloud-arrow-up", "Edge Publisher", "1-click R2 & CDN sync"]] },
                { head: "DEVELOPERS", items: [["fa-solid fa-code", "REST & MCP API", "WP v2 + AI agent tools"], ["fa-solid fa-terminal", "CLI & Docker", "Single-binary containers"]] },
                { head: "RESOURCES", items: [["fa-solid fa-book-open", "Documentation", "Full architecture guides"], ["fa-solid fa-gauge-high", "Benchmarks", "Live latency telemetry"]] }
            ].map(group => ({
                id: uid("col"), type: "Column", data: {}, styles: { gap: "10px" },
                children: [
                    {
                        id: uid("h"), type: "Heading", data: { text: group.head, level: 6 },
                        styles: { text_color: "#93A1FF", font_size: "11px", font_weight: "700", letter_spacing: "0.08em" }, children: []
                    },
                    ...group.items.map(it => makeFeatureCard(it[0], it[1], it[2], "#"))
                ]
            }));
            return [
                {
                    id: uid("grid"), type: "Grid", data: {},
                    styles: { display: "grid", grid_template_columns: "repeat(4, 1fr)", gap: "16px" }, children: cols
                },
                {
                    id: uid("bar"), type: "Container", data: {},
                    styles: {
                        display: "flex", flex_direction: "row", align_items: "center", justify_content: "space-between",
                        margin_top: "16px", padding_top: "12px", padding_right: "16px", padding_bottom: "12px", padding_left: "16px",
                        background_color: "rgba(59,130,246,0.1)", border_radius: "8px"
                    },
                    children: [
                        {
                            id: uid("t"), type: "Text",
                            data: { content_html: `<p><strong>Ready to migrate your fleet?</strong> Import any WordPress site in 1 click.</p>` },
                            styles: { text_color: "#e2e8f0", font_size: "12.5px" }, children: []
                        },
                        {
                            id: uid("btn"), type: "Button",
                            data: { text: "Start Free Migration", url: "/migrate", icon: "fa-solid fa-arrow-right" },
                            styles: { background_color: "#2563eb", text_color: "#ffffff", font_size: "12px", border_radius: "6px", padding_top: "8px", padding_bottom: "8px", padding_left: "14px", padding_right: "14px" },
                            children: []
                        }
                    ]
                }
            ];
        }

        if (presetKey === "blank_row") {
            return [{
                id: uid("grid"), type: "Grid", data: {},
                styles: { display: "grid", grid_template_columns: "repeat(3, 1fr)", gap: "18px" },
                children: [1, 2, 3].map(n => ({
                    id: uid("col"), type: "Column", data: {},
                    styles: {
                        gap: "10px", padding_top: "14px", padding_right: "14px", padding_bottom: "14px", padding_left: "14px",
                        border_radius: "8px", background_color: "rgba(255,255,255,0.03)"
                    },
                    children: [
                        { id: uid("h"), type: "Heading", data: { text: `${title} Column ${n}`, level: 5 }, styles: { text_color: "#f8fafc", font_size: "14px", font_weight: "700" }, children: [] },
                        { id: uid("t"), type: "Text", data: { content_html: "<p>Drop buttons, icons, links, or cards into this column.</p>" }, styles: { text_color: "#94a3b8", font_size: "12px" }, children: [] }
                    ]
                }))
            }];
        }

        return [{
            id: uid("grid"), type: "Grid", data: {},
            styles: { display: "grid", grid_template_columns: "1fr 1fr 1.1fr", gap: "18px" },
            children: [
                {
                    id: uid("col"), type: "Column", data: {}, styles: { gap: "10px" },
                    children: [
                        { id: uid("h"), type: "Heading", data: { text: `${title.toUpperCase()} SUITE`, level: 6 }, styles: { text_color: "#93A1FF", font_size: "11px", font_weight: "700", letter_spacing: "0.08em" }, children: [] },
                        makeFeatureCard("fa-solid fa-layer-group", "Visual AST Studio", "Real-time CAD artboard & responsive breakpoints", "#"),
                        makeFeatureCard("fa-solid fa-bolt", "Sub-50µs Hot Cache", "DashMap memory cache with instant invalidation", "#"),
                        makeFeatureCard("fa-solid fa-clone", "Synced Global Symbols", "Figma-style master components with overrides", "#")
                    ]
                },
                {
                    id: uid("col"), type: "Column", data: {}, styles: { gap: "10px" },
                    children: [
                        { id: uid("h"), type: "Heading", data: { text: "WORKFLOWS & SCALE", level: 6 }, styles: { text_color: "#93A1FF", font_size: "11px", font_weight: "700", letter_spacing: "0.08em" }, children: [] },
                        makeFeatureCard("fa-solid fa-wand-magic-sparkles", "1-Click WP Migration", "Elementor, Divi & Gutenberg AST transpiler", "#"),
                        makeFeatureCard("fa-solid fa-shield-halved", "Enterprise Security", "TOTP 2FA, RBAC capabilities & audit trails", "#"),
                        makeFeatureCard("fa-solid fa-cloud-arrow-up", "Edge R2 & Auto-TLS", "Zero-config ACME certificates & object storage", "#")
                    ]
                },
                {
                    id: uid("col"), type: "Column", data: {},
                    styles: {
                        gap: "12px", padding_top: "18px", padding_right: "18px", padding_bottom: "18px", padding_left: "18px",
                        border_radius: "10px", background_color: "#1e293b", border_width: "1px", border_style: "solid", border_color: "#334155"
                    },
                    children: [
                        { id: uid("h"), type: "Heading", data: { text: "Daypress 0.5 Release", level: 4 }, styles: { text_color: "#ffffff", font_size: "16px", font_weight: "800" }, children: [] },
                        { id: uid("t"), type: "Text", data: { content_html: "<p>Design full-bleed 2026 Mega Menus, Theme Layouts, Synced Symbols, and Pop-ups directly on the visual canvas.</p>" }, styles: { text_color: "#cbd5e1", font_size: "12.5px", line_height: "1.55" }, children: [] },
                        { id: uid("btn"), type: "Button", data: { text: "Explore What's New", url: "/changelog", icon: "fa-solid fa-rocket" }, styles: { background_color: "#2563eb", text_color: "#ffffff", font_size: "12.5px", border_radius: "6px", padding_top: "9px", padding_bottom: "9px", padding_left: "14px", padding_right: "14px" }, children: [] }
                    ]
                }
            ]
        }];
    }
    window.buildMegaMenuPresetChildren = buildMegaMenuPresetChildren;
})();
"###;
