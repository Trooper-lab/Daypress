//! DayStudio Media Library Selector & Direct Image Uploader (`scripts/media_picker.rs`).
//!
//! Provides `window.openDayStudioMediaPicker(currentUrl, onSelect)` and
//! `window.enhanceMediaInput(inputEl, opts)` so any image or background-image
//! field in the Styling or Content Inspector can select from the SQLite Media
//! Library (`/wp-json/wp/v2/media`) or upload a new image directly.

pub const JS_MEDIA_PICKER: &str = r###"
(function() {
    function unwrapCssUrl(val) {
        if (!val) return "";
        const s = String(val).trim();
        const m = s.match(/^url\(['"]?(.*?)['"]?\)$/i);
        return m ? m[1] : s;
    }

    async function uploadFileToMediaLibrary(file) {
        const formData = new FormData();
        formData.append("file", file);
        const headers = {};
        if (window.DAYPRESS_NONCE) headers["X-WP-Nonce"] = window.DAYPRESS_NONCE;
        const res = await fetch("/wp-json/wp/v2/media", {
            method: "POST",
            headers,
            body: formData
        });
        if (!res.ok) throw new Error("Upload failed (" + res.status + ")");
        return await res.json();
    }
    window.uploadFileToMediaLibrary = uploadFileToMediaLibrary;

    function openDayStudioMediaPicker(currentUrl, onSelect) {
        const existing = document.getElementById("dp-media-picker-backdrop");
        if (existing) existing.remove();

        const cleanCurrent = unwrapCssUrl(currentUrl);
        let mediaItems = [];
        let searchQuery = "";

        const backdrop = document.createElement("div");
        backdrop.id = "dp-media-picker-backdrop";
        backdrop.style.cssText = "position:fixed;inset:0;z-index:99999;background:rgba(9,10,13,0.76);backdrop-filter:blur(6px);display:flex;align-items:center;justify-content:center;padding:24px;";

        const modal = document.createElement("div");
        modal.style.cssText = "width:760px;max-width:96vw;max-height:86vh;background:var(--day-canvas,#14161B);border:1px solid var(--day-edge-bright,#2C313C);border-radius:10px;box-shadow:0 24px 64px rgba(0,0,0,0.65);display:flex;flex-direction:column;overflow:hidden;color:var(--day-ink,#F3F4F6);font-family:var(--font-sans,sans-serif);";

        modal.innerHTML = `
            <div style="display:flex;align-items:center;justify-content:space-between;padding:12px 16px;border-bottom:1px solid var(--day-edge,#22262F);background:var(--day-panel,#1B1E24);">
                <div style="display:flex;align-items:center;gap:8px;">
                    <i class="fa-solid fa-photo-film" style="color:var(--day-signal,#93A1FF);"></i>
                    <span style="font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:0.06em;">Media Library & Image Uploader</span>
                </div>
                <button type="button" id="dp-mp-close" style="background:transparent;border:none;color:var(--day-ink-muted,#9CA3AF);cursor:pointer;font-size:14px;padding:4px 8px;"><i class="fa-solid fa-xmark"></i></button>
            </div>
            <div style="padding:12px 16px;border-bottom:1px solid var(--day-edge,#22262F);display:flex;gap:10px;align-items:center;flex-wrap:wrap;background:var(--day-well,#0E1014);">
                <div style="position:relative;flex:1;min-width:200px;">
                    <i class="fa-solid fa-magnifying-glass" style="position:absolute;left:10px;top:50%;transform:translateY(-50%);font-size:11px;color:var(--day-ink-muted);"></i>
                    <input type="search" id="dp-mp-search" class="insp-input" placeholder="Search media library by title or filename..." style="width:100%;padding-left:28px;height:30px;font-size:12px;">
                </div>
                <input type="file" id="dp-mp-file-input" accept="image/*,.svg,.webp,.avif" multiple style="display:none;">
                <button type="button" id="dp-mp-upload-btn" class="topbar-btn btn-publish" style="height:30px;padding:0 12px;font-size:11px;display:inline-flex;align-items:center;gap:6px;">
                    <i class="fa-solid fa-cloud-arrow-up"></i> Upload New Image
                </button>
            </div>
            <div id="dp-mp-dropzone" style="padding:16px;overflow-y:auto;flex:1;min-height:300px;">
                <div id="dp-mp-status" style="font-size:12px;color:var(--day-ink-muted);text-align:center;padding:32px 0;">Loading media library...</div>
                <div id="dp-mp-grid" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(132px,1fr));gap:10px;"></div>
            </div>
            <div style="padding:10px 16px;border-top:1px solid var(--day-edge,#22262F);background:var(--day-panel,#1B1E24);display:flex;align-items:center;gap:8px;">
                <span style="font-size:10px;font-family:var(--font-mono);color:var(--day-ink-muted);text-transform:uppercase;">Direct URL</span>
                <input type="text" id="dp-mp-custom-url" class="insp-input" style="flex:1;height:28px;font-size:11px;font-family:var(--font-mono);" placeholder="https://... or /wp-content/uploads/..." value="${cleanCurrent.replace(/"/g, '&quot;')}">
                <button type="button" id="dp-mp-apply-url" class="topbar-btn" style="height:28px;padding:0 12px;font-size:11px;background:var(--day-signal-subtle);color:var(--day-signal);border:1px solid var(--day-signal-edge);">Use URL</button>
            </div>
        `;

        backdrop.appendChild(modal);
        document.body.appendChild(backdrop);

        const close = () => backdrop.remove();
        backdrop.onclick = (e) => { if (e.target === backdrop) close(); };
        modal.querySelector("#dp-mp-close").onclick = close;

        const gridEl = modal.querySelector("#dp-mp-grid");
        const statusEl = modal.querySelector("#dp-mp-status");
        const searchEl = modal.querySelector("#dp-mp-search");
        const fileInput = modal.querySelector("#dp-mp-file-input");
        const uploadBtn = modal.querySelector("#dp-mp-upload-btn");
        const customUrlInput = modal.querySelector("#dp-mp-custom-url");

        modal.querySelector("#dp-mp-apply-url").onclick = () => {
            const val = customUrlInput.value.trim();
            if (onSelect) onSelect(val);
            close();
        };

        function renderGrid() {
            gridEl.innerHTML = "";
            const filtered = mediaItems.filter(item => {
                const url = item.source_url || "";
                const title = (item.title && item.title.rendered) ? item.title.rendered : (item.title || "");
                const hay = (title + " " + url + " " + (item.alt_text || "")).toLowerCase();
                return !searchQuery || hay.includes(searchQuery);
            });

            if (filtered.length === 0) {
                statusEl.style.display = "block";
                statusEl.innerHTML = mediaItems.length === 0
                    ? `<i class="fa-regular fa-images" style="font-size:24px;display:block;margin-bottom:8px;opacity:0.5;"></i>No images in Media Library yet. Click <b>Upload New Image</b> or drop files here.`
                    : `No media items match "${searchQuery}".`;
                return;
            }
            statusEl.style.display = "none";

            filtered.forEach(item => {
                const url = item.source_url || "";
                const title = (item.title && item.title.rendered) ? item.title.rendered : (item.title || url.split("/").pop() || "Image");
                const isSelected = url === cleanCurrent;

                const card = document.createElement("div");
                card.style.cssText = `border:1px solid ${isSelected ? "var(--day-signal,#93A1FF)" : "var(--day-edge,#22262F)"};border-radius:6px;background:var(--day-well,#0E1014);overflow:hidden;cursor:pointer;display:flex;flex-direction:column;transition:border-color 0.12s,transform 0.12s;${isSelected ? "box-shadow:0 0 0 2px var(--day-signal-subtle);" : ""}`;
                card.onmouseenter = () => { card.style.borderColor = "var(--day-signal,#93A1FF)"; };
                card.onmouseleave = () => { if (!isSelected) card.style.borderColor = "var(--day-edge,#22262F)"; };

                card.innerHTML = `
                    <div style="height:96px;background:repeating-conic-gradient(#181B22 0% 25%, #12141A 0% 50%) 50% / 12px 12px;display:flex;align-items:center;justify-content:center;overflow:hidden;position:relative;">
                        <img src="${url}" alt="" style="max-width:100%;max-height:100%;object-fit:contain;" loading="lazy">
                    </div>
                    <div style="padding:6px 8px;font-size:10.5px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;color:var(--day-ink);font-weight:500;" title="${title.replace(/"/g, '&quot;')}">${title}</div>
                `;
                card.onclick = () => {
                    if (onSelect) onSelect(url, item);
                    close();
                };
                gridEl.appendChild(card);
            });
        }

        async function fetchLibrary() {
            try {
                const res = await fetch("/wp-json/wp/v2/media?per_page=100");
                if (!res.ok) throw new Error("HTTP " + res.status);
                const data = await res.json();
                mediaItems = Array.isArray(data) ? data.filter(m => !m.mime_type || m.mime_type.startsWith("image/")) : [];
                renderGrid();
            } catch (err) {
                statusEl.style.display = "block";
                statusEl.textContent = "Could not load media library: " + err.message;
            }
        }

        searchEl.oninput = () => {
            searchQuery = searchEl.value.trim().toLowerCase();
            renderGrid();
        };

        uploadBtn.onclick = () => fileInput.click();
        fileInput.onchange = async () => {
            const files = Array.from(fileInput.files || []);
            if (files.length === 0) return;
            uploadBtn.disabled = true;
            uploadBtn.innerHTML = `<i class="fa-solid fa-spinner fa-spin"></i> Uploading...`;
            let lastUploaded = null;
            for (const f of files) {
                try {
                    lastUploaded = await uploadFileToMediaLibrary(f);
                } catch (e) {
                    if (window.showToast) window.showToast("Upload failed: " + e.message, "error");
                }
            }
            uploadBtn.disabled = false;
            uploadBtn.innerHTML = `<i class="fa-solid fa-cloud-arrow-up"></i> Upload New Image`;
            if (lastUploaded && lastUploaded.source_url) {
                if (window.showToast) window.showToast("Image uploaded to Media Library", "success");
                if (files.length === 1 && onSelect) {
                    onSelect(lastUploaded.source_url, lastUploaded);
                    close();
                    return;
                }
                await fetchLibrary();
            }
        };

        fetchLibrary();
    }
    window.openDayStudioMediaPicker = openDayStudioMediaPicker;

    function enhanceMediaInput(inputEl, opts) {
        if (!inputEl || inputEl.dataset.mediaEnhanced === "1") return;
        inputEl.dataset.mediaEnhanced = "1";
        const useCssUrl = !!(opts && opts.cssUrlWrap);

        const wrapper = document.createElement("div");
        wrapper.style.cssText = "display:flex;flex-direction:column;gap:6px;width:100%;";
        inputEl.parentNode.insertBefore(wrapper, inputEl);

        const topRow = document.createElement("div");
        topRow.style.cssText = "display:flex;align-items:center;gap:6px;width:100%;";

        const thumb = document.createElement("div");
        thumb.style.cssText = "width:28px;height:28px;border-radius:4px;border:1px solid var(--day-edge-bright);background:var(--day-well);display:flex;align-items:center;justify-content:center;overflow:hidden;flex-shrink:0;cursor:pointer;";
        thumb.title = "Click to browse Media Library";

        function syncThumb() {
            const raw = unwrapCssUrl(inputEl.value);
            if (raw && (raw.startsWith("http") || raw.startsWith("/") || raw.startsWith("data:"))) {
                thumb.innerHTML = `<img src="${raw.replace(/"/g, '&quot;')}" style="width:100%;height:100%;object-fit:cover;">`;
            } else {
                thumb.innerHTML = `<i class="fa-regular fa-image" style="font-size:11px;color:var(--day-ink-muted);"></i>`;
            }
        }
        syncThumb();

        inputEl.style.flex = "1";
        topRow.appendChild(thumb);
        topRow.appendChild(inputEl);

        const btnRow = document.createElement("div");
        btnRow.style.cssText = "display:flex;gap:6px;";

        const btnLib = document.createElement("button");
        btnLib.type = "button";
        btnLib.className = "insp-btn-block";
        btnLib.style.cssText = "flex:1;margin:0;padding:5px 8px;font-size:10.5px;display:inline-flex;align-items:center;justify-content:center;gap:5px;background:var(--day-raised);border:1px solid var(--day-edge-bright);color:var(--day-ink);border-radius:4px;cursor:pointer;";
        btnLib.innerHTML = `<i class="fa-solid fa-photo-film" style="color:var(--day-signal);"></i> Media Library`;

        const quickFile = document.createElement("input");
        quickFile.type = "file";
        quickFile.accept = "image/*,.svg,.webp,.avif";
        quickFile.style.display = "none";

        const btnUp = document.createElement("button");
        btnUp.type = "button";
        btnUp.className = "insp-btn-block";
        btnUp.style.cssText = "margin:0;padding:5px 9px;font-size:10.5px;display:inline-flex;align-items:center;justify-content:center;gap:5px;background:var(--day-signal-subtle);border:1px solid var(--day-signal-edge);color:var(--day-signal);border-radius:4px;cursor:pointer;";
        btnUp.innerHTML = `<i class="fa-solid fa-cloud-arrow-up"></i> Upload`;

        function applyUrl(url) {
            const formatted = (useCssUrl && url && !url.startsWith("url(")) ? `url('${url}')` : url;
            inputEl.value = formatted;
            syncThumb();
            inputEl.dispatchEvent(new Event("input", { bubbles: true }));
            inputEl.dispatchEvent(new Event("change", { bubbles: true }));
        }

        const openPicker = () => openDayStudioMediaPicker(inputEl.value, (url) => applyUrl(url));
        thumb.onclick = openPicker;
        btnLib.onclick = openPicker;
        btnUp.onclick = () => quickFile.click();
        quickFile.onchange = async () => {
            const file = quickFile.files && quickFile.files[0];
            if (!file) return;
            btnUp.disabled = true;
            btnUp.innerHTML = `<i class="fa-solid fa-spinner fa-spin"></i>`;
            try {
                const uploaded = await uploadFileToMediaLibrary(file);
                if (uploaded && uploaded.source_url) {
                    applyUrl(uploaded.source_url);
                    if (window.showToast) window.showToast("Uploaded & applied image", "success");
                }
            } catch (e) {
                if (window.showToast) window.showToast("Upload failed: " + e.message, "error");
            } finally {
                btnUp.disabled = false;
                btnUp.innerHTML = `<i class="fa-solid fa-cloud-arrow-up"></i> Upload`;
            }
        };

        inputEl.addEventListener("input", syncThumb);
        btnRow.appendChild(btnLib);
        btnRow.appendChild(btnUp);
        btnRow.appendChild(quickFile);
        wrapper.appendChild(topRow);
        wrapper.appendChild(btnRow);
    }
    window.enhanceMediaInput = enhanceMediaInput;
})();
"###;
