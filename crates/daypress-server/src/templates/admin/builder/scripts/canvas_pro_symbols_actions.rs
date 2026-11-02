//! Dynamic & Theme/Form Pro Blocks Canvas DOM Builders — Part 2: Symbol Clone, Save, Insert, Detach & Dynamic Binding (`scripts/canvas_pro_symbols_actions.rs`).

pub const JS_CANVAS_PRO_SYMBOLS_ACTIONS: &str = r###"
    function applyOverridesToClonedTree(node, overridesMap, regenerateIds) {
        if (!node || typeof node !== "object") return node;
        const origId = String(node.id || "");
        const ov = (overridesMap && origId && overridesMap[origId]) ? overridesMap[origId] : null;
        if (!node.data) node.data = {};

        if (ov && typeof ov === "object") {
            if (ov.text != null && ov.text !== "") {
                if ("text" in node) node.text = ov.text;
                if ("content" in node) node.content = ov.text;
                node.data.text = ov.text;
                node.data.content = ov.text;
                if (node.type === "Button" || "label" in node.data) node.data.label = ov.text;
            }
            if (ov.src != null && ov.src !== "") {
                node.src = ov.src;
                node.data.src = ov.src;
            }
            if (ov.alt != null && ov.alt !== "") {
                node.alt = ov.alt;
                node.data.alt = ov.alt;
            }
            if (ov.url != null && ov.url !== "") {
                node.url = ov.url;
                node.href = ov.url;
                node.data.url = ov.url;
                node.data.href = ov.url;
            }
        }

        if (regenerateIds && node.id && node.id !== "root") {
            node.id = node.id + "-det-" + Math.random().toString(36).slice(2, 6);
        }

        if (Array.isArray(node.children)) {
            node.children.forEach(function(child) {
                applyOverridesToClonedTree(child, overridesMap, regenerateIds);
            });
        }
        return node;
    }

    function renderSymbolAstPreviewHtml(node, overridesMap) {
        if (!node || typeof node !== "object") return "";
        const t = node.type || node.kind || "Container";
        const data = node.data || {};
        const ov = (overridesMap && node.id && overridesMap[node.id]) ? overridesMap[node.id] : {};

        const textVal = (ov.text != null && ov.text !== "")
            ? ov.text
            : (node.text ?? node.content ?? data.text ?? data.content ?? data.label ?? data.title ?? "");
        const srcVal = (ov.src != null && ov.src !== "")
            ? ov.src
            : (node.src ?? data.src ?? "");
        const altVal = (ov.alt != null && ov.alt !== "")
            ? ov.alt
            : (node.alt ?? data.alt ?? "");
        const urlVal = (ov.url != null && ov.url !== "")
            ? ov.url
            : (node.url ?? node.href ?? data.url ?? data.href ?? "#");

        const childrenHtml = Array.isArray(node.children)
            ? node.children.map(function(c) { return renderSymbolAstPreviewHtml(c, overridesMap); }).join("")
            : "";

        if (t === "Root") return childrenHtml;
        if (t === "Heading") {
            const tag = data.tag || node.tag || "h3";
            return `<${tag} data-symbol-child-id="${escHtml(node.id)}" style="margin:0 0 8px 0;font-weight:700;color:#0f172a;">${escHtml(textVal || "Symbol Heading")}</${tag}>`;
        }
        if (t === "Text" || t === "Paragraph" || t === "RichText") {
            return `<p data-symbol-child-id="${escHtml(node.id)}" style="margin:0 0 8px 0;font-size:14px;color:#475569;line-height:1.6;">${escHtml(textVal || "Symbol paragraph text")}</p>`;
        }
        if (t === "Button") {
            return `<a href="${escHtml(urlVal)}" data-symbol-child-id="${escHtml(node.id)}" style="display:inline-block;padding:8px 16px;background:var(--day-signal,#2563eb);color:#fff;border-radius:6px;font-size:13px;font-weight:600;text-decoration:none;">${escHtml(textVal || "Action")}</a>`;
        }
        if (t === "Image") {
            return srcVal
                ? `<img src="${escHtml(srcVal)}" alt="${escHtml(altVal)}" data-symbol-child-id="${escHtml(node.id)}" style="max-width:100%;height:auto;border-radius:6px;display:block;" />`
                : `<div data-symbol-child-id="${escHtml(node.id)}" style="height:100px;background:#f1f5f9;border-radius:6px;display:flex;align-items:center;justify-content:center;color:#64748b;font-size:12px;"><i class="fa-regular fa-image" style="margin-right:6px;"></i>${escHtml(altVal || "Symbol Image")}</div>`;
        }
        return `<div data-symbol-child-id="${escHtml(node.id)}" style="padding:8px;border:1px dashed rgba(124,58,237,0.2);border-radius:6px;margin-bottom:6px;">${textVal ? `<div style="font-size:13px;margin-bottom:4px;">${escHtml(textVal)}</div>` : ""}${childrenHtml}</div>`;
    }

    async function saveSelectedSectionAsGlobalSymbol() {
        const selNode = getSelectedNode();
        if (!selNode || selNode.id === "root" || selNode.type === "Root") {
            notifyUser("Select a Section, Container, or Element on the canvas first", true);
            return;
        }
        const defaultName = (selNode.data && (selNode.data.label || selNode.data.title))
            || (selNode.type ? selNode.type + " Symbol (" + selNode.id + ")" : "Global Section Symbol");
        const symbolTitle = window.prompt("Enter a name for this Synced Global Symbol:", defaultName);
        if (!symbolTitle || !symbolTitle.trim()) return;

        const clonedNode = JSON.parse(JSON.stringify(selNode));
        const symbolLayout = {
            version: 1,
            root: {
                id: "root",
                type: "Root",
                children: [clonedNode]
            }
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
            if (!res.ok) {
                throw new Error("Server returned " + res.status);
            }
            const created = await res.json();
            symbolsCache[created.id] = created;
            await fetchSectionSymbols(true);

            const symInstanceNode = {
                id: "sym-" + Date.now().toString(36),
                type: "SymbolInstance",
                symbol_id: Number(created.id),
                overrides: {},
                data: {
                    symbol_id: Number(created.id),
                    symbol_title: created.title || symbolTitle.trim(),
                    overrides: {}
                },
                styles: {},
                children: []
            };
            knownNodesMap.set(symInstanceNode.id, symInstanceNode);

            const root = getStudioAstRoot();
            const loc = root ? findParentAndIndex(root, selNode.id) : null;
            if (loc && loc.parent && Array.isArray(loc.parent.children)) {
                loc.parent.children.splice(loc.index, 1, symInstanceNode);
            } else {
                Object.keys(selNode).forEach(function(k) { delete selNode[k]; });
                Object.assign(selNode, symInstanceNode);
            }

            notifyStudioMutation(symInstanceNode.id);
            renderSymbolsDrawerContent();
            notifyUser('✓ Saved Global Symbol "' + (created.title || symbolTitle.trim()) + '" (#' + created.id + ')');
        } catch (err) {
            notifyUser("Failed to save global symbol: " + err.message, true);
        }
    }

    async function insertSymbolInstanceOntoCanvas(symbolId) {
        await fetchSectionSymbols(false);
        const numId = Number(symbolId);
        const symTpl = symbolsCache[numId] || symbolsList.find(function(s) { return Number(s.id) === numId; });
        if (!symTpl && symbolsList.length > 0 && !numId) {
            return insertSymbolInstanceOntoCanvas(symbolsList[0].id);
        }

        const newInstance = {
            id: "sym-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 5),
            type: "SymbolInstance",
            symbol_id: numId || 0,
            overrides: {},
            data: {
                symbol_id: numId || 0,
                symbol_title: symTpl ? symTpl.title : "Global Symbol",
                overrides: {}
            },
            styles: {},
            children: []
        };
        knownNodesMap.set(newInstance.id, newInstance);

        const root = getStudioAstRoot();
        const selNode = getSelectedNode();
        if (selNode && Array.isArray(selNode.children) && ["Root", "Section", "Container", "Column", "Div"].includes(selNode.type)) {
            selNode.children.push(newInstance);
        } else if (root && Array.isArray(root.children)) {
            root.children.push(newInstance);
        }

        notifyStudioMutation(newInstance.id);
        notifyUser("✓ Inserted Synced Symbol #" + (numId || "") + " onto canvas");
    }

    async function detachSymbolInstance(instanceNodeId) {
        const instNode = findAstNodeById(instanceNodeId) || getSelectedNode();
        if (!instNode || instNode.type !== "SymbolInstance") {
            notifyUser("Select a SymbolInstance node to detach", true);
            return;
        }
        await fetchSectionSymbols(false);
        const symbolId = Number(instNode.symbol_id ?? (instNode.data && instNode.data.symbol_id) ?? 0);
        const overrides = instNode.overrides || (instNode.data && instNode.data.overrides) || {};
        const symTpl = symbolsCache[symbolId] || symbolsList.find(function(s) { return Number(s.id) === symbolId; });

        let masterChildren = [];
        if (symTpl && symTpl.layout) {
            const layoutObj = typeof symTpl.layout === "string" ? JSON.parse(symTpl.layout) : symTpl.layout;
            if (layoutObj && layoutObj.root && Array.isArray(layoutObj.root.children)) {
                masterChildren = JSON.parse(JSON.stringify(layoutObj.root.children));
            }
        }
        if (masterChildren.length === 0) {
            masterChildren = [{
                id: "sec-" + Date.now().toString(36),
                type: "Section",
                data: { label: "Detached Symbol Section" },
                styles: { padding_top: "32px", padding_bottom: "32px" },
                children: []
            }];
        }

        const bakedChildren = masterChildren.map(function(child) {
            const baked = applyOverridesToClonedTree(child, overrides, true);
            indexAstSubtree(baked);
            return baked;
        });

        const root = getStudioAstRoot();
        const loc = root ? findParentAndIndex(root, instNode.id) : null;
        if (loc && loc.parent && Array.isArray(loc.parent.children)) {
            loc.parent.children.splice(loc.index, 1, ...bakedChildren);
            notifyStudioMutation(bakedChildren[0].id);
        } else {
            const first = bakedChildren[0];
            Object.keys(instNode).forEach(function(k) { delete instNode[k]; });
            Object.assign(instNode, first);
            notifyStudioMutation(instNode.id);
        }
        notifyUser("✓ Detached SymbolInstance into independent AST nodes with overrides baked in");
    }

    function applyDynamicTokenToSelection(token) {
        const selNode = getSelectedNode();
        if (!token) return;

        const inspector = document.getElementById("daystudio-inspector");
        let targetInput = lastFocusedInspectorInput;
        if (!targetInput || !inspector || !inspector.contains(targetInput) || targetInput.classList.contains("dp-ignore-dynamic-target")) {
            if (inspector) {
                targetInput = inspector.querySelector(
                    'input[type="text"]:not(.dp-ignore-dynamic-target), textarea:not(.dp-ignore-dynamic-target), input[type="url"]:not(.dp-ignore-dynamic-target)'
                );
            }
        }

        if (targetInput) {
            targetInput.value = token;
            targetInput.dispatchEvent(new Event("input", { bubbles: true }));
            targetInput.dispatchEvent(new Event("change", { bubbles: true }));
        }

        if (selNode) {
            if (!selNode.data) selNode.data = {};
            if (!selNode.attributes) selNode.attributes = {};
            selNode.data["data-dynamic"] = token;
            selNode.data.dynamic_binding = token;
            selNode.attributes["data-dynamic"] = token;

            const nodeType = selNode.type || selNode.kind || "";
            if (nodeType === "Image" || token === "{{post.featured_image}}") {
                selNode.src = token;
                selNode.data.src = token;
            } else if (["Heading", "Text", "Paragraph", "RichText", "Button"].includes(nodeType)) {
                if ("text" in selNode) selNode.text = token;
                if ("content" in selNode) selNode.content = token;
                selNode.data.text = token;
                selNode.data.content = token;
                if (nodeType === "Button") selNode.data.label = token;
            }
        }

        const selId = getSelectedNodeId();
        if (selId) {
            const canvasNodeEl = document.querySelector(
                `#daystudio-canvas [data-id="${CSS.escape(selId)}"], #daystudio-canvas [data-node-id="${CSS.escape(selId)}"]`
            );
            if (canvasNodeEl) {
                canvasNodeEl.setAttribute("data-dynamic", token);
            }
        }

        notifyStudioMutation(selId);
        notifyUser("✓ Bound field to " + token);
    }
"###;
