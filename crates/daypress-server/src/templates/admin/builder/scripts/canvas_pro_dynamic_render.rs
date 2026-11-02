//! Dynamic & Theme/Form Pro Blocks Canvas DOM Builders — Part 1: State & Symbol API (`scripts/canvas_pro_dynamic_render.rs`).

pub const JS_CANVAS_PRO_DYNAMIC_RENDER: &str = r###"
(function() {
    const DYNAMIC_TOKENS = [
        { token: "{{post.title}}", label: "Post Title", category: "Post" },
        { token: "{{post.date}}", label: "Post Date", category: "Post" },
        { token: "{{post.excerpt}}", label: "Post Excerpt", category: "Post" },
        { token: "{{post.featured_image}}", label: "Featured Image", category: "Post" },
        { token: "{{post.content}}", label: "Post Content", category: "Post" },
        { token: "{{author.name}}", label: "Author Name", category: "Author" },
        { token: "{{archive.title}}", label: "Archive Title", category: "Archive" },
        { token: "{{archive.description}}", label: "Archive Description", category: "Archive" },
        { token: "{{site.title}}", label: "Site Title", category: "Site" }
    ];

    let symbolsCache = {};
    let symbolsList = [];
    let megaMenuTemplatesList = [];
    let symbolsLoaded = false;
    let symbolsLoadingPromise = null;
    let knownNodesMap = new Map();
    let capturedAstRoot = null;
    let lastFocusedInspectorInput = null;
    let isEnhancingInspector = false;
    let isEnhancingDrawer = false;

    function escHtml(s) {
        return String(s == null ? "" : s).replace(/[&<>"']/g, function(c) {
            return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c];
        });
    }

    function notifyUser(msg, isError) {
        if (typeof showToast === "function") {
            showToast(msg, !!isError);
            return;
        }
        if (typeof window.showToast === "function") {
            window.showToast(msg, !!isError);
            return;
        }
        const statusEl = document.getElementById("lbl-save-status");
        if (statusEl) {
            statusEl.textContent = msg;
        }
    }

    function indexAstSubtree(node) {
        if (!node || typeof node !== "object") return;
        if (node.id) {
            knownNodesMap.set(String(node.id), node);
        }
        if (Array.isArray(node.children)) {
            node.children.forEach(indexAstSubtree);
        }
    }

    function getStudioAstRoot() {
        try {
            if (typeof state !== "undefined" && state) {
                if (state.ast && state.ast.root) return state.ast.root;
                if (state.layout && state.layout.root) return state.layout.root;
                if (state.root && Array.isArray(state.root.children)) return state.root;
            }
        } catch (_) {}
        try {
            if (typeof studioState !== "undefined" && studioState) {
                if (studioState.ast && studioState.ast.root) return studioState.ast.root;
                if (studioState.layout && studioState.layout.root) return studioState.layout.root;
                if (studioState.root) return studioState.root;
            }
        } catch (_) {}
        try {
            if (typeof ast !== "undefined" && ast) {
                if (ast.root) return ast.root;
                if (Array.isArray(ast.children)) return ast;
            }
        } catch (_) {}
        if (window.state) {
            if (window.state.ast && window.state.ast.root) return window.state.ast.root;
            if (window.state.layout && window.state.layout.root) return window.state.layout.root;
            if (window.state.root) return window.state.root;
        }
        if (window.DayStudio && window.DayStudio.ast && window.DayStudio.ast.root) {
            return window.DayStudio.ast.root;
        }
        if (capturedAstRoot && Array.isArray(capturedAstRoot.children)) {
            return capturedAstRoot;
        }
        if (knownNodesMap.has("root")) {
            return knownNodesMap.get("root");
        }
        return null;
    }

    function getSelectedNodeId() {
        try {
            if (typeof state !== "undefined" && state) {
                const sid = state.selectedId || state.selectedNodeId || (state.selected && state.selected.id);
                if (sid) return String(sid);
            }
        } catch (_) {}
        try {
            if (typeof selectedNodeId !== "undefined" && selectedNodeId) return String(selectedNodeId);
        } catch (_) {}
        try {
            if (typeof selectedId !== "undefined" && selectedId) return String(selectedId);
        } catch (_) {}
        if (window.selectedNodeId) return String(window.selectedNodeId);
        if (window.selectedId) return String(window.selectedId);

        const badgeIdEl = document.getElementById("sel-badge-id");
        const chromeEl = document.getElementById("daystudio-selection-chrome");
        if (badgeIdEl && (!chromeEl || chromeEl.style.display !== "none")) {
            const raw = (badgeIdEl.textContent || "").trim().replace(/^#/, "");
            if (raw && raw !== "node") return raw;
        }

        const bcItems = document.querySelectorAll("#daystudio-breadcrumbs .bc-item[data-id]");
        if (bcItems.length > 0) {
            const lastBc = bcItems[bcItems.length - 1];
            const bcId = lastBc.getAttribute("data-id");
            if (bcId) return bcId;
        }

        const selCanvasEl = document.querySelector("#daystudio-canvas .selected[data-id], #daystudio-canvas .is-selected[data-id], #daystudio-canvas [data-node-id].selected");
        if (selCanvasEl) {
            return selCanvasEl.getAttribute("data-id") || selCanvasEl.getAttribute("data-node-id");
        }
        return null;
    }

    function findNodeInTree(root, targetId) {
        if (!root || !targetId) return null;
        if (String(root.id) === String(targetId)) return root;
        if (Array.isArray(root.children)) {
            for (let i = 0; i < root.children.length; i++) {
                const found = findNodeInTree(root.children[i], targetId);
                if (found) return found;
            }
        }
        return null;
    }

    function findParentAndIndex(root, targetId) {
        if (!root || !targetId || !Array.isArray(root.children)) return null;
        for (let i = 0; i < root.children.length; i++) {
            const child = root.children[i];
            if (child && String(child.id) === String(targetId)) {
                return { parent: root, index: i, node: child };
            }
            const nested = findParentAndIndex(child, targetId);
            if (nested) return nested;
        }
        return null;
    }

    function findAstNodeById(targetId) {
        if (!targetId) return null;
        try {
            if (typeof findNodeById === "function") {
                const n = findNodeById(targetId);
                if (n) return n;
            }
        } catch (_) {}
        try {
            if (typeof findNode === "function") {
                const root = getStudioAstRoot();
                const n = root ? findNode(root, targetId) : findNode(targetId);
                if (n) return n;
            }
        } catch (_) {}
        const root = getStudioAstRoot();
        if (root) {
            const found = findNodeInTree(root, targetId);
            if (found) return found;
        }
        if (knownNodesMap.has(String(targetId))) {
            return knownNodesMap.get(String(targetId));
        }
        return null;
    }

    function getSelectedNode() {
        const id = getSelectedNodeId();
        return id ? findAstNodeById(id) : null;
    }

    function notifyStudioMutation(selectIdAfter) {
        try {
            if (typeof pushHistory === "function") pushHistory();
            else if (typeof recordHistory === "function") recordHistory();
        } catch (_) {}
        try {
            if (typeof markDirty === "function") markDirty();
        } catch (_) {}

        const pill = document.getElementById("status-save-pill");
        const lbl = document.getElementById("lbl-save-status");
        if (pill) {
            pill.classList.remove("saved");
            pill.classList.add("unsaved");
        }
        if (lbl) lbl.textContent = "Unsaved";

        try {
            if (typeof renderCanvas === "function") renderCanvas();
            else if (typeof window.renderCanvas === "function") window.renderCanvas();
        } catch (_) {}
        try {
            if (typeof renderNavigator === "function") renderNavigator();
            else if (typeof window.renderNavigator === "function") window.renderNavigator();
        } catch (_) {}
        if (selectIdAfter) {
            try {
                if (typeof selectNode === "function") selectNode(selectIdAfter);
                else if (typeof window.selectNode === "function") window.selectNode(selectIdAfter);
            } catch (_) {}
        }
        try {
            if (typeof renderInspector === "function") renderInspector();
            else if (typeof window.renderInspector === "function") window.renderInspector();
        } catch (_) {}
        setTimeout(enhanceInspectorPanel, 20);
    }

    async function fetchSectionSymbols(forceRefresh) {
        if (symbolsLoaded && !forceRefresh) return symbolsList;
        if (symbolsLoadingPromise && !forceRefresh) return symbolsLoadingPromise;

        symbolsLoadingPromise = (async function() {
            try {
                const res = await fetch("/wp-json/daypress/v1/templates?type=section_symbol", {
                    credentials: "same-origin"
                });
                if (res.ok) {
                    const data = await res.json();
                    const rawList = Array.isArray(data) ? data : (data.templates || []);
                    symbolsList = rawList.filter(function(t) {
                        return !t.template_type || t.template_type === "section_symbol";
                    });
                    symbolsList.forEach(function(s) {
                        symbolsCache[s.id] = s;
                    });
                }

                const allRes = await fetch("/wp-json/daypress/v1/templates", {
                    credentials: "same-origin"
                });
                if (allRes.ok) {
                    const allData = await allRes.json();
                    const allRaw = Array.isArray(allData) ? allData : (allData.templates || []);
                    megaMenuTemplatesList = allRaw.filter(function(t) {
                        return t.template_type === "section_symbol" || t.template_type === "header";
                    });
                    allRaw.forEach(function(t) {
                        if (t.template_type === "section_symbol") {
                            symbolsCache[t.id] = t;
                            if (!symbolsList.some(function(existing) { return existing.id === t.id; })) {
                                symbolsList.push(t);
                            }
                        }
                    });
                }
                symbolsLoaded = true;
            } catch (err) {
                console.error("Failed to fetch section_symbol templates:", err);
            } finally {
                symbolsLoadingPromise = null;
            }
            return symbolsList;
        })();

        return symbolsLoadingPromise;
    }

    function collectOverridableSymbolChildren(node, outList) {
        outList = outList || [];
        if (!node || typeof node !== "object") return outList;
        const t = node.type || node.kind || "";
        const data = node.data || {};
        if (node.id && node.id !== "root" && t !== "Root") {
            const defaultText = node.text ?? node.content ?? data.text ?? data.content ?? data.label ?? data.title ?? "";
            const defaultSrc = node.src ?? data.src ?? data.url ?? "";
            const defaultAlt = node.alt ?? data.alt ?? "";
            const defaultUrl = node.url ?? node.href ?? data.url ?? data.href ?? "";
            const isLeafOrContent = ["Heading", "Text", "Paragraph", "RichText", "Button", "Image", "Icon", "Counter", "Badge"].includes(t)
                || defaultText !== "" || defaultSrc !== "" || defaultUrl !== "";
            if (isLeafOrContent || !Array.isArray(node.children) || node.children.length === 0) {
                outList.push({
                    id: String(node.id),
                    type: t || "Element",
                    text: String(defaultText || ""),
                    src: String(defaultSrc || ""),
                    alt: String(defaultAlt || ""),
                    url: String(defaultUrl || "")
                });
            }
        }
        if (Array.isArray(node.children)) {
            node.children.forEach(function(c) {
                collectOverridableSymbolChildren(c, outList);
            });
        }
        return outList;
    }
"###;
