//! SQLite dual-write persistence and Theme Builder REST API client (`scripts/api_client.rs`).

pub const JS_API_CLIENT: &str = r###"
(function() {
    window.StudioApi = {
        // -------------------------------------------------------------
        // LAYOUT PERSISTENCE (ZERO MOCK / SQLITE DUAL-WRITE)
        // -------------------------------------------------------------
        async saveLayout() {
            const state = window.NeoStudioState;
            if (state.isSaving) return;
            state.isSaving = true;

            const pill = document.getElementById("status-save-pill");
            const lbl = document.getElementById("lbl-save-status");
            const btnSaveText = document.getElementById("btn-save-text");

            if (pill) pill.className = "save-status-pill saving";
            if (lbl) lbl.textContent = "Saving...";
            if (btnSaveText) btnSaveText.textContent = "Saving...";

            try {
                const targetEndpoint = state.currentContext.type === "template"
                    ? `/wp-json/neopress/v1/templates/${state.currentContext.id}`
                    : `/wp-json/neopress/v1/layout/${state.postId}`;

                const targetMethod = state.currentContext.type === "template" ? "PUT" : "POST";

                const bodyPayload = state.currentContext.type === "template"
                    ? {
                        title: state.currentContext.title,
                        layout: state.pageAst,
                        layout_json: state.pageAst,
                        status: "publish"
                    }
                    : {
                        layout: state.pageAst,
                        layout_json: state.pageAst
                    };

                const res = await fetch(targetEndpoint, {
                    method: targetMethod,
                    headers: {
                        "Content-Type": "application/json",
                        "X-WP-Nonce": "wp_rest_admin"
                    },
                    credentials: "include",
                    body: JSON.stringify(bodyPayload)
                });

                if (res.ok) {
                    window.markDirty(false);
                    if (btnSaveText) btnSaveText.textContent = "Publish";
                    if (window.showToast) window.showToast("Layout saved to SQLite successfully");
                } else {
                    const errJson = await res.json().catch(() => ({}));
                    console.error("Save error response:", errJson);
                    if (pill) pill.className = "save-status-pill unsaved";
                    if (lbl) lbl.textContent = "Failed";
                    if (btnSaveText) btnSaveText.textContent = "Retry";
                    if (window.showToast) window.showToast("Save failed: " + (errJson.message || res.statusText));
                }
            } catch(err) {
                console.error("Network exception saving layout:", err);
                if (pill) pill.className = "save-status-pill unsaved";
                if (lbl) lbl.textContent = "Error";
                if (btnSaveText) btnSaveText.textContent = "Retry";
                if (window.showToast) window.showToast("Network error saving layout");
            } finally {
                state.isSaving = false;
            }
        },

        // -------------------------------------------------------------
        // THEME BUILDER REST API METHODS
        // -------------------------------------------------------------
        async getTemplates(templateType) {
            const url = templateType
                ? `/wp-json/neopress/v1/templates?template_type=${templateType}`
                : `/wp-json/neopress/v1/templates`;
            const res = await fetch(url, { credentials: "include" });
            if (!res.ok) throw new Error("Failed to fetch templates");
            return await res.json();
        },

        async getTemplate(id) {
            const res = await fetch(`/wp-json/neopress/v1/templates/${id}`, { credentials: "include" });
            if (!res.ok) throw new Error("Failed to fetch template " + id);
            return await res.json();
        },

        async createTemplate(payload) {
            const res = await fetch("/wp-json/neopress/v1/templates", {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify(payload)
            });
            if (!res.ok) throw new Error("Failed to create template");
            return await res.json();
        },

        async updateTemplate(id, payload) {
            const res = await fetch(`/wp-json/neopress/v1/templates/${id}`, {
                method: "PUT",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify(payload)
            });
            if (!res.ok) throw new Error("Failed to update template");
            return await res.json();
        },

        async deleteTemplate(id) {
            const res = await fetch(`/wp-json/neopress/v1/templates/${id}?force=true`, {
                method: "DELETE",
                headers: { "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include"
            });
            if (!res.ok) throw new Error("Failed to delete template");
            return await res.json();
        },

        async getConditions(id) {
            const res = await fetch(`/wp-json/neopress/v1/templates/${id}/conditions`, { credentials: "include" });
            if (!res.ok) throw new Error("Failed to fetch conditions");
            return await res.json();
        },

        async setConditions(id, conditions) {
            const res = await fetch(`/wp-json/neopress/v1/templates/${id}/conditions`, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify({ conditions })
            });
            if (!res.ok) throw new Error("Failed to save conditions");
            return await res.json();
        },

        async getStarters() {
            const res = await fetch("/wp-json/neopress/v1/templates/starter", { credentials: "include" });
            if (!res.ok) throw new Error("Failed to fetch starters");
            return await res.json();
        },

        async installStarter(starterId, title) {
            const res = await fetch("/wp-json/neopress/v1/templates/starter", {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify({ starter_id: starterId, title: title || "Starter Template" })
            });
            if (!res.ok) throw new Error("Failed to install starter");
            return await res.json();
        },

        async getPagesAndPosts(search) {
            const q = search ? `&search=${encodeURIComponent(search)}` : "";
            const [pagesRes, postsRes] = await Promise.all([
                fetch(`/wp-json/wp/v2/pages?per_page=100${q}`, { credentials: "include" }),
                fetch(`/wp-json/wp/v2/posts?per_page=100${q}`, { credentials: "include" })
            ]);
            const pages = pagesRes.ok ? await pagesRes.json() : [];
            const posts = postsRes.ok ? await postsRes.json() : [];
            return {
                pages: Array.isArray(pages) ? pages : [],
                posts: Array.isArray(posts) ? posts : []
            };
        },

        async getPostLayout(postId) {
            const res = await fetch(`/wp-json/neopress/v1/layout/${postId}`, { credentials: "include" });
            if (!res.ok) throw new Error("Failed to load layout for post " + postId);
            return await res.json();
        },

        async createQuickPage(title) {
            const res = await fetch("/wp-json/wp/v2/pages", {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify({ title: title || "New Page", status: "publish" })
            });
            if (!res.ok) throw new Error("Failed to create page");
            return await res.json();
        }
    };
})();
"###;
