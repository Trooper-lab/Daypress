//! Interactive Visual Icon Selector Modal & Field Enhancer (`scripts/icon_picker.rs`).

pub const JS_ICON_PICKER: &str = r###"
(function() {
    const ICON_CATALOG = [
        // Interface & Navigation
        { cls: "fa-solid fa-bolt", label: "Bolt", cat: "interface", kw: "fast lightning speed power flash" },
        { cls: "fa-solid fa-compass", label: "Compass", cat: "interface", kw: "navigation explore guide direction" },
        { cls: "fa-solid fa-layer-group", label: "Layers", cat: "interface", kw: "stack architecture hierarchy" },
        { cls: "fa-solid fa-sliders", label: "Sliders", cat: "interface", kw: "settings controls config" },
        { cls: "fa-solid fa-gear", label: "Gear", cat: "interface", kw: "settings cog preferences" },
        { cls: "fa-solid fa-magnifying-glass", label: "Search", cat: "interface", kw: "find zoom lookup" },
        { cls: "fa-solid fa-bell", label: "Bell", cat: "interface", kw: "alert notification" },
        { cls: "fa-solid fa-bookmark", label: "Bookmark", cat: "interface", kw: "save favorite tag" },
        { cls: "fa-solid fa-check", label: "Check", cat: "interface", kw: "done success tick" },
        { cls: "fa-solid fa-circle-check", label: "Check Circle", cat: "interface", kw: "verified approved complete" },
        { cls: "fa-solid fa-wand-magic-sparkles", label: "Sparkles", cat: "interface", kw: "magic ai generate polish" },
        { cls: "fa-solid fa-arrow-right", label: "Arrow Right", cat: "interface", kw: "next forward link cta" },
        { cls: "fa-solid fa-arrow-up-right-from-square", label: "External", cat: "interface", kw: "open window link" },
        { cls: "fa-solid fa-chevron-right", label: "Chevron", cat: "interface", kw: "caret arrow" },
        { cls: "fa-solid fa-bars", label: "Menu Bars", cat: "interface", kw: "hamburger nav list" },
        { cls: "fa-solid fa-table-columns", label: "Columns", cat: "interface", kw: "layout mega menu grid" },
        { cls: "fa-solid fa-grip", label: "Grid Grip", cat: "interface", kw: "apps matrix bento" },
        { cls: "fa-solid fa-cube", label: "Cube", cat: "interface", kw: "block 3d module" },
        { cls: "fa-solid fa-cubes", label: "Cubes", cat: "interface", kw: "modules packages ecosystem" },
        { cls: "fa-solid fa-puzzle-piece", label: "Plugin", cat: "interface", kw: "extension addon integration" },
        { cls: "fa-solid fa-filter", label: "Filter", cat: "interface", kw: "funnel sort refine" },
        { cls: "fa-solid fa-star", label: "Star", cat: "interface", kw: "rating featured favorite" },
        { cls: "fa-solid fa-heart", label: "Heart", cat: "interface", kw: "like love health" },
        { cls: "fa-solid fa-fire", label: "Fire", cat: "interface", kw: "hot trending popular" },

        // Product, AI & Cloud
        { cls: "fa-solid fa-rocket", label: "Rocket", cat: "product", kw: "launch startup deploy boost" },
        { cls: "fa-solid fa-microchip", label: "Microchip", cat: "product", kw: "cpu hardware silicon compute ai" },
        { cls: "fa-solid fa-brain", label: "AI Brain", cat: "product", kw: "intelligence neural machine learning" },
        { cls: "fa-solid fa-robot", label: "Agent Bot", cat: "product", kw: "automation ai assistant" },
        { cls: "fa-solid fa-gauge-high", label: "Speedometer", cat: "product", kw: "performance benchmark latency fast" },
        { cls: "fa-solid fa-chart-line", label: "Line Chart", cat: "product", kw: "analytics growth metrics" },
        { cls: "fa-solid fa-chart-pie", label: "Pie Chart", cat: "product", kw: "analytics report share" },
        { cls: "fa-solid fa-chart-simple", label: "Bar Chart", cat: "product", kw: "stats telemetry dashboard" },
        { cls: "fa-solid fa-database", label: "Database", cat: "product", kw: "sql sqlite storage data" },
        { cls: "fa-solid fa-server", label: "Server", cat: "product", kw: "rack hosting backend infra" },
        { cls: "fa-solid fa-cloud", label: "Cloud", cat: "product", kw: "saas hosting edge cdn" },
        { cls: "fa-solid fa-cloud-arrow-up", label: "Cloud Deploy", cat: "product", kw: "upload publish backup sync" },
        { cls: "fa-solid fa-network-wired", label: "Network", cat: "product", kw: "cluster mesh nodes routing" },
        { cls: "fa-solid fa-diagram-project", label: "Workflow", cat: "product", kw: "graph pipeline nodes" },
        { cls: "fa-solid fa-code", label: "Code", cat: "product", kw: "developer html api sdk" },
        { cls: "fa-solid fa-terminal", label: "Terminal", cat: "product", kw: "cli shell command console" },
        { cls: "fa-solid fa-laptop-code", label: "Dev Studio", cat: "product", kw: "coding ide software" },
        { cls: "fa-solid fa-mobile-screen", label: "Mobile App", cat: "product", kw: "phone responsive ios android" },
        { cls: "fa-solid fa-globe", label: "Global Edge", cat: "product", kw: "world international domain web" },
        { cls: "fa-solid fa-arrows-rotate", label: "Sync Loop", cat: "product", kw: "refresh update ci cd" },

        // Commerce, Security & Enterprise
        { cls: "fa-solid fa-shield-halved", label: "Shield", cat: "commerce", kw: "security protection waf zero trust" },
        { cls: "fa-solid fa-lock", label: "Lock", cat: "commerce", kw: "encrypted ssl tls private auth" },
        { cls: "fa-solid fa-key", label: "API Key", cat: "commerce", kw: "token auth passkey" },
        { cls: "fa-solid fa-fingerprint", label: "Biometric", cat: "commerce", kw: "identity auth security" },
        { cls: "fa-solid fa-user-shield", label: "Admin Role", cat: "commerce", kw: "rbac permissions privacy" },
        { cls: "fa-solid fa-award", label: "Award", cat: "commerce", kw: "certified badge quality" },
        { cls: "fa-solid fa-trophy", label: "Trophy", cat: "commerce", kw: "winner leader best" },
        { cls: "fa-solid fa-crown", label: "Enterprise", cat: "commerce", kw: "vip pro premium tier" },
        { cls: "fa-solid fa-gem", label: "Gem", cat: "commerce", kw: "diamond value luxury" },
        { cls: "fa-solid fa-credit-card", label: "Billing", cat: "commerce", kw: "payment checkout stripe" },
        { cls: "fa-solid fa-wallet", label: "Wallet", cat: "commerce", kw: "finance money balance" },
        { cls: "fa-solid fa-tags", label: "Pricing Tags", cat: "commerce", kw: "discount sale offer" },
        { cls: "fa-solid fa-cart-shopping", label: "Cart", cat: "commerce", kw: "ecommerce shop buy" },
        { cls: "fa-solid fa-store", label: "Marketplace", cat: "commerce", kw: "shop storefront" },
        { cls: "fa-solid fa-building", label: "Company", cat: "commerce", kw: "organization enterprise office" },
        { cls: "fa-solid fa-briefcase", label: "Portfolio", cat: "commerce", kw: "business work careers" },
        { cls: "fa-solid fa-handshake", label: "Partners", cat: "commerce", kw: "b2b deal trust" },
        { cls: "fa-solid fa-users", label: "Team", cat: "commerce", kw: "community customers people" },

        // Content, Media & Support
        { cls: "fa-solid fa-book-open", label: "Docs", cat: "media", kw: "documentation guide manual knowledge" },
        { cls: "fa-solid fa-newspaper", label: "Blog / News", cat: "media", kw: "articles press editorial" },
        { cls: "fa-solid fa-file-lines", label: "Changelog", cat: "media", kw: "document page notes" },
        { cls: "fa-solid fa-folder-open", label: "Resources", cat: "media", kw: "library dam files assets" },
        { cls: "fa-solid fa-pen-nib", label: "Design", cat: "media", kw: "author vector creative" },
        { cls: "fa-solid fa-palette", label: "Theme Palette", cat: "media", kw: "color style tokens" },
        { cls: "fa-solid fa-image", label: "Image", cat: "media", kw: "photo gallery media" },
        { cls: "fa-solid fa-video", label: "Video", cat: "media", kw: "stream webinar tutorial" },
        { cls: "fa-solid fa-circle-play", label: "Play Demo", cat: "media", kw: "watch video start" },
        { cls: "fa-solid fa-bullhorn", label: "Announcement", cat: "media", kw: "marketing campaign promo" },
        { cls: "fa-solid fa-envelope", label: "Email", cat: "media", kw: "contact newsletter inbox" },
        { cls: "fa-solid fa-paper-plane", label: "Send", cat: "media", kw: "submit message launch" },
        { cls: "fa-solid fa-comments", label: "Forum / Chat", cat: "media", kw: "discussion support feedback" },
        { cls: "fa-solid fa-headset", label: "Support", cat: "media", kw: "helpdesk customer care" },
        { cls: "fa-solid fa-calendar-days", label: "Events", cat: "media", kw: "schedule booking date" },
        { cls: "fa-solid fa-clock", label: "Uptime / Time", cat: "media", kw: "history schedule 24/7" },
        { cls: "fa-solid fa-location-dot", label: "Location", cat: "media", kw: "map address pin" },
        { cls: "fa-solid fa-circle-question", label: "FAQ / Help", cat: "media", kw: "question support info" },

        // Brands & Ecosystem
        { cls: "fa-brands fa-github", label: "GitHub", cat: "brands", kw: "git repo open source" },
        { cls: "fa-brands fa-x-twitter", label: "X / Twitter", cat: "brands", kw: "social tweet" },
        { cls: "fa-brands fa-linkedin", label: "LinkedIn", cat: "brands", kw: "social business network" },
        { cls: "fa-brands fa-discord", label: "Discord", cat: "brands", kw: "community chat" },
        { cls: "fa-brands fa-slack", label: "Slack", cat: "brands", kw: "chat workspace" },
        { cls: "fa-brands fa-youtube", label: "YouTube", cat: "brands", kw: "video channel" },
        { cls: "fa-brands fa-stripe", label: "Stripe", cat: "brands", kw: "payments billing" },
        { cls: "fa-brands fa-aws", label: "AWS", cat: "brands", kw: "amazon cloud s3" },
        { cls: "fa-brands fa-cloudflare", label: "Cloudflare", cat: "brands", kw: "edge r2 cdn" },
        { cls: "fa-brands fa-docker", label: "Docker", cat: "brands", kw: "container deploy" },
        { cls: "fa-brands fa-rust", label: "Rust", cat: "brands", kw: "native fast language" },
        { cls: "fa-brands fa-wordpress", label: "WordPress", cat: "brands", kw: "cms migration" }
    ];
    window.DAYSTUDIO_ICON_CATALOG = ICON_CATALOG;

    function openDayStudioIconPicker(currentValue, onSelect) {
        const existing = document.getElementById("dp-icon-modal-backdrop");
        if (existing) existing.remove();

        let activeCat = "all";
        let searchQuery = "";
        let selectedCls = (currentValue || "").trim();

        const backdrop = document.createElement("div");
        backdrop.id = "dp-icon-modal-backdrop";
        backdrop.className = "dp-icon-modal-backdrop";

        backdrop.innerHTML = `
            <div class="dp-icon-modal" role="dialog" aria-label="Visual Icon Selector">
                <div class="cad-panel-header" style="padding:12px 14px;background:var(--day-chassis);border-bottom:1px solid var(--day-rule);">
                    <span class="cad-panel-title" style="font-size:11.5px;color:var(--day-ink);">
                        <i class="fa-solid fa-icons" style="color:var(--day-signal);"></i> Visual Icon Selector
                    </span>
                    <button type="button" class="cad-btn sm secondary" id="dp-icon-modal-close" title="Close (Esc)">
                        <i class="fa-solid fa-xmark"></i>
                    </button>
                </div>
                <div style="padding:10px 14px;background:var(--day-surface);border-bottom:1px solid var(--day-rule-subtle);display:flex;gap:8px;align-items:center;">
                    <i class="fa-solid fa-magnifying-glass" style="color:var(--day-muted);font-size:11px;"></i>
                    <input type="search" id="dp-icon-search-input" class="cad-input" style="flex:1;height:28px;" placeholder="Search 90+ icons (e.g. bolt, rocket, shield, chart, docs, github)...">
                    <button type="button" class="cad-btn sm danger" id="dp-icon-clear-btn" title="Remove Icon">
                        <i class="fa-solid fa-ban"></i> None
                    </button>
                </div>
                <div class="dp-icon-modal-cats" id="dp-icon-cat-bar">
                    <button type="button" class="dp-icon-cat-pill active" data-cat="all">All Icons</button>
                    <button type="button" class="dp-icon-cat-pill" data-cat="interface">Interface & Nav</button>
                    <button type="button" class="dp-icon-cat-pill" data-cat="product">Product, AI & Cloud</button>
                    <button type="button" class="dp-icon-cat-pill" data-cat="commerce">Commerce & Security</button>
                    <button type="button" class="dp-icon-cat-pill" data-cat="media">Docs & Media</button>
                    <button type="button" class="dp-icon-cat-pill" data-cat="brands">Brands</button>
                </div>
                <div class="dp-icon-grid" id="dp-icon-grid-body"></div>
                <div style="padding:10px 14px;background:var(--day-chassis);border-top:1px solid var(--day-rule);display:flex;align-items:center;gap:8px;">
                    <span class="cad-label" style="width:auto;">Custom Class:</span>
                    <input type="text" id="dp-icon-custom-input" class="cad-input font-mono" style="flex:1;" value="${selectedCls.replace(/"/g, '&quot;')}" placeholder="fa-solid fa-bolt">
                    <button type="button" class="cad-btn primary" id="dp-icon-apply-btn">
                        <i class="fa-solid fa-check"></i> Use Icon
                    </button>
                </div>
            </div>
        `;

        document.body.appendChild(backdrop);

        const gridEl = backdrop.querySelector("#dp-icon-grid-body");
        const searchEl = backdrop.querySelector("#dp-icon-search-input");
        const customEl = backdrop.querySelector("#dp-icon-custom-input");

        function close() {
            backdrop.remove();
        }

        function commit(val) {
            if (typeof onSelect === "function") onSelect(val);
            close();
        }

        function renderGrid() {
            const q = searchQuery.toLowerCase().trim();
            const filtered = ICON_CATALOG.filter(item => {
                if (activeCat !== "all" && item.cat !== activeCat) return false;
                if (!q) return true;
                return item.label.toLowerCase().includes(q)
                    || item.cls.toLowerCase().includes(q)
                    || item.kw.toLowerCase().includes(q);
            });

            if (filtered.length === 0) {
                gridEl.innerHTML = `<div style="grid-column:1/-1;padding:28px;text-align:center;color:var(--day-muted);font-size:11.5px;">No matching icons in catalog. You can still type any FontAwesome class below and click <strong>Use Icon</strong>.</div>`;
                return;
            }

            gridEl.innerHTML = filtered.map(item => {
                const isAct = item.cls === selectedCls ? "active" : "";
                return `<button type="button" class="dp-icon-tile ${isAct}" data-icon-cls="${item.cls}" title="${item.cls}">
                    <i class="${item.cls}"></i>
                    <span>${item.label}</span>
                </button>`;
            }).join("");

            gridEl.querySelectorAll(".dp-icon-tile").forEach(tile => {
                tile.onclick = () => {
                    const chosen = tile.getAttribute("data-icon-cls") || "";
                    selectedCls = chosen;
                    customEl.value = chosen;
                    commit(chosen);
                };
            });
        }

        backdrop.querySelectorAll(".dp-icon-cat-pill").forEach(pill => {
            pill.onclick = () => {
                backdrop.querySelectorAll(".dp-icon-cat-pill").forEach(p => p.classList.remove("active"));
                pill.classList.add("active");
                activeCat = pill.getAttribute("data-cat") || "all";
                renderGrid();
            };
        });

        searchEl.oninput = () => {
            searchQuery = searchEl.value || "";
            renderGrid();
        };

        backdrop.querySelector("#dp-icon-clear-btn").onclick = () => commit("");
        backdrop.querySelector("#dp-icon-apply-btn").onclick = () => commit(customEl.value.trim());
        backdrop.querySelector("#dp-icon-modal-close").onclick = close;
        backdrop.onclick = (e) => { if (e.target === backdrop) close(); };

        renderGrid();
        setTimeout(() => searchEl.focus(), 20);
    }
    window.openDayStudioIconPicker = openDayStudioIconPicker;

    // Enhances any existing <input> element into a visual icon selector row with live preview button
    function enhanceIconInput(inputEl, onPicked) {
        if (!inputEl || inputEl.dataset.iconEnhanced === "1") return;
        inputEl.dataset.iconEnhanced = "1";

        const wrap = document.createElement("div");
        wrap.className = "cad-icon-field-wrap";
        inputEl.parentNode.insertBefore(wrap, inputEl);

        const previewBtn = document.createElement("button");
        previewBtn.type = "button";
        previewBtn.className = "cad-icon-preview-btn";
        previewBtn.title = "Open Visual Icon Selector";

        function syncPreview() {
            const v = (inputEl.value || "").trim();
            previewBtn.innerHTML = v ? `<i class="${v}"></i>` : `<i class="fa-solid fa-icons" style="opacity:0.55;"></i>`;
        }
        syncPreview();

        const browseBtn = document.createElement("button");
        browseBtn.type = "button";
        browseBtn.className = "cad-btn sm secondary";
        browseBtn.innerHTML = `<i class="fa-solid fa-grip"></i> Pick`;
        browseBtn.title = "Browse Visual Icon Library";

        const openPicker = () => {
            openDayStudioIconPicker(inputEl.value, (chosen) => {
                inputEl.value = chosen;
                syncPreview();
                inputEl.dispatchEvent(new Event("input", { bubbles: true }));
                inputEl.dispatchEvent(new Event("change", { bubbles: true }));
                if (typeof onPicked === "function") onPicked(chosen);
            });
        };

        previewBtn.onclick = openPicker;
        browseBtn.onclick = openPicker;
        inputEl.addEventListener("input", syncPreview);

        wrap.appendChild(previewBtn);
        wrap.appendChild(inputEl);
        wrap.appendChild(browseBtn);
    }
    window.enhanceIconInput = enhanceIconInput;
})();
"###;
