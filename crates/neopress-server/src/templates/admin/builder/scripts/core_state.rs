//! Client-side unified state container, AST normalization and history stack (`scripts/core_state.rs`).

pub const JS_CORE_STATE: &str = r###"
(function() {
    window.NeoStudioState = {
        // Canonical AST & Post Context
        postId: __POST_ID__,
        postSlug: "__POST_SLUG__",
        postType: "__POST_TYPE__",
        siteTitle: "__SITE_TITLE__",
        pageAst: null,
        currentContext: { type: "post", id: __POST_ID__, title: "__POST_TITLE__", slug: "__POST_SLUG__", postType: "__POST_TYPE__" },

        // Active Selection & Modifiers
        activeElementId: null,
        currentBreakpoint: "desktop", // "desktop" | "tablet" | "mobile"
        pseudoState: "default",       // "default" | "hover" | "focus" | "active"
        activeClassSelector: null,
        zoomLevel: 1.0,
        xrayMode: false,
        activeDrawer: "elements",     // "elements" | "navigator" | "templates" | "tokens" | "history" | "settings"
        drawerOpen: true,
        currentInspectorTab: "style", // "style" | "settings"

        // Mutation History Stack
        historyStack: [],
        historyIndex: -1,
        maxHistory: 50,

        // Persistence State
        isDirty: false,
        isSaving: false
    };

    // Parse Initial AST
    try {
        const raw = __LAYOUT_JSON_RAW__;
        window.NeoStudioState.pageAst = normalizeAst(typeof raw === "string" ? JSON.parse(raw) : raw);
    } catch(e) {
        console.error("Failed to parse initial layout AST, initializing blank", e);
        window.NeoStudioState.pageAst = normalizeAst(null);
    }

    // -------------------------------------------------------------
    // AST NORMALIZATION
    // -------------------------------------------------------------
    function normalizeAst(ast) {
        if (!ast) ast = { version: 1 };
        if (!ast.root) {
            ast.root = {
                id: "root",
                type: "Root",
                children: [
                    {
                        id: generateId("sec"),
                        type: "Section",
                        styles: { background_color: "#ffffff", padding_top: "60px", padding_bottom: "60px" },
                        children: [
                            {
                                id: generateId("cont"),
                                type: "Container",
                                styles: { text_align: "center" },
                                children: [
                                    {
                                        id: generateId("h"),
                                        type: "Heading",
                                        data: { text: "__POST_TITLE__", level: 1 },
                                        styles: { font_size: "36px", text_color: "#0f172a", margin_bottom: "16px" }
                                    }
                                ]
                            }
                        ]
                    }
                ]
            };
        } else {
            if (ast.root.type === "Section" && (ast.root.id === "root" || !ast.root.styles || !ast.root.styles.background_color)) {
                ast.root.type = "Root";
            }
            if (!ast.root.children) ast.root.children = [];
        }
        // Flatten nested import style objects (padding/typography/background/border) into the flat
        // keys the inspector edits, so edits are never shadowed by stale nested values.
        if (window.NeoStyle) window.NeoStyle.normalizeTreeStyles(ast.root);
        return ast;
    }
    window.normalizeAst = normalizeAst;

    // ID Generator
    function generateId(prefix) {
        return (prefix || "node") + "_" + Math.random().toString(36).substr(2, 7);
    }
    window.generateId = generateId;

    // -------------------------------------------------------------
    // MUTATION HISTORY ENGINE (UNDO / REDO)
    // -------------------------------------------------------------
    function pushHistorySnapshot() {
        const state = window.NeoStudioState;
        if (!state.pageAst) return;
        const snapshot = JSON.stringify(state.pageAst);

        // Discard any redos beyond current index
        if (state.historyIndex < state.historyStack.length - 1) {
            state.historyStack = state.historyStack.slice(0, state.historyIndex + 1);
        }

        state.historyStack.push(snapshot);
        if (state.historyStack.length > state.maxHistory) {
            state.historyStack.shift();
        } else {
            state.historyIndex++;
        }

        markDirty(true);
        updateHistoryButtons();
    }
    window.pushHistorySnapshot = pushHistorySnapshot;

    function undo() {
        const state = window.NeoStudioState;
        if (state.historyIndex > 0) {
            state.historyIndex--;
            state.pageAst = JSON.parse(state.historyStack[state.historyIndex]);
            markDirty(true);
            updateHistoryButtons();
            if (window.renderCanvas) window.renderCanvas();
            if (window.renderInspector) window.renderInspector();
            if (window.updateNavigator) window.updateNavigator();
        }
    }
    window.undo = undo;

    function redo() {
        const state = window.NeoStudioState;
        if (state.historyIndex < state.historyStack.length - 1) {
            state.historyIndex++;
            state.pageAst = JSON.parse(state.historyStack[state.historyIndex]);
            markDirty(true);
            updateHistoryButtons();
            if (window.renderCanvas) window.renderCanvas();
            if (window.renderInspector) window.renderInspector();
            if (window.updateNavigator) window.updateNavigator();
        }
    }
    window.redo = redo;

    function updateHistoryButtons() {
        const state = window.NeoStudioState;
        const btnUndo = document.getElementById("btn-undo");
        const btnRedo = document.getElementById("btn-redo");
        if (btnUndo) {
            if (state.historyIndex > 0) btnUndo.classList.remove("disabled");
            else btnUndo.classList.add("disabled");
        }
        if (btnRedo) {
            if (state.historyIndex < state.historyStack.length - 1) btnRedo.classList.remove("disabled");
            else btnRedo.classList.add("disabled");
        }
    }

    function markDirty(dirty) {
        const state = window.NeoStudioState;
        state.isDirty = dirty;
        const pill = document.getElementById("status-save-pill");
        const lbl = document.getElementById("lbl-save-status");
        if (pill && lbl) {
            if (dirty) {
                pill.className = "save-status-pill unsaved";
                lbl.textContent = "Unsaved";
            } else {
                pill.className = "save-status-pill saved";
                lbl.textContent = "Saved";
            }
        }
    }
    window.markDirty = markDirty;

    function updateViewLiveLink() {
        const state = window.NeoStudioState;
        const btnLive = document.getElementById("btn-view-live");
        const badge = document.getElementById("badge-template-type");
        const lblTitle = document.getElementById("lbl-template-title");
        const ctx = state.currentContext || {};

        if (badge) badge.textContent = ctx.type === "template" ? (ctx.templateType || "Template") : (ctx.postType || "Page");
        if (lblTitle && ctx.title) lblTitle.textContent = ctx.title;

        if (btnLive) {
            if (ctx.type === "template") {
                btnLive.href = ctx.templateType === "404_page" ? "/404-preview-check" : "/";
            } else if (ctx.slug && ctx.slug !== "__POST_SLUG__") {
                btnLive.href = "/" + ctx.slug.replace(/^\/+|\/+$/g, "") + "/";
            } else if (state.postId) {
                btnLive.href = "/?p=" + state.postId;
            } else {
                btnLive.href = "/";
            }
        }
    }
    window.updateViewLiveLink = updateViewLiveLink;

    // Push initial snapshot
    setTimeout(() => {
        pushHistorySnapshot();
        markDirty(false);
    }, 50);

    // -------------------------------------------------------------
    // AST RECURSIVE NODE TRAVERSAL
    // -------------------------------------------------------------
    function findNodeById(id, node) {
        if (!node) node = window.NeoStudioState.pageAst ? window.NeoStudioState.pageAst.root : null;
        if (!node) return null;
        if (node.id === id) return node;
        if (node.children) {
            for (const child of node.children) {
                const found = findNodeById(id, child);
                if (found) return found;
            }
        }
        return null;
    }
    window.findNodeById = findNodeById;

    function findParentNode(childId, node) {
        if (!node) node = window.NeoStudioState.pageAst ? window.NeoStudioState.pageAst.root : null;
        if (!node || !node.children) return null;
        for (const child of node.children) {
            if (child.id === childId) return node;
            const parent = findParentNode(childId, child);
            if (parent) return parent;
        }
        return null;
    }
    window.findParentNode = findParentNode;

    function isDescendant(ancestorId, testId) {
        const ancestor = findNodeById(ancestorId);
        if (!ancestor || !ancestor.children) return false;
        for (const child of ancestor.children) {
            if (child.id === testId) return true;
            if (isDescendant(child.id, testId)) return true;
        }
        return false;
    }
    window.isDescendant = isDescendant;
})();
"###;
