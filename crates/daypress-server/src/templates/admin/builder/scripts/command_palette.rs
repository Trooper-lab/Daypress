//! Command Palette Spotlight Finder (Cmd+K / Ctrl+K) (`scripts/command_palette.rs`).

pub const JS_COMMAND_PALETTE: &str = r###"
(function() {
    const COMMANDS = [
        // Studio Actions
        { title: "Save Layout to SQLite", cat: "Studio Action", icon: "fa-solid fa-cloud-arrow-up", run: () => window.StudioApi.saveLayout() },
        { title: "View Live Page in New Tab", cat: "Studio Action", icon: "fa-solid fa-arrow-up-right-from-square", run: () => { const b = document.getElementById("btn-view-live"); if (b && b.href) window.open(b.href, "_blank"); } },
        { title: "Toggle Structural X-Ray Mode", cat: "Studio Action", icon: "fa-solid fa-border-all", run: () => window.toggleXray() },
        { title: "Undo Mutation", cat: "Studio Action", icon: "fa-solid fa-arrow-rotate-left", run: () => window.undo() },
        { title: "Redo Mutation", cat: "Studio Action", icon: "fa-solid fa-arrow-rotate-right", run: () => window.redo() },
        { title: "View Canonical JSON AST", cat: "Studio Action", icon: "fa-solid fa-code", run: () => document.getElementById("ast-modal").classList.add("open") },
        { title: "Page & Document Settings", cat: "Studio Action", icon: "fa-solid fa-gear", run: () => document.getElementById("settings-modal").classList.add("open") },

        // Responsive Breakpoints
        { title: "Switch to Desktop (1200px+)", cat: "Responsive", icon: "fa-solid fa-desktop", run: () => window.setBreakpoint("desktop") },
        { title: "Switch to Tablet (768px)", cat: "Responsive", icon: "fa-solid fa-tablet-screen-button", run: () => window.setBreakpoint("tablet") },
        { title: "Switch to Mobile (375px)", cat: "Responsive", icon: "fa-solid fa-mobile-screen-button", run: () => window.setBreakpoint("mobile") },

        // Add Elements Quick Launch
        { title: "Insert Price Card", cat: "Add Element", icon: "fa-solid fa-credit-card", run: () => window.insertWidgetFromPalette("PriceCard") },
        { title: "Insert Call To Action", cat: "Add Element", icon: "fa-solid fa-bullhorn", run: () => window.insertWidgetFromPalette("CallToAction") },
        { title: "Insert Countdown Timer", cat: "Add Element", icon: "fa-solid fa-clock", run: () => window.insertWidgetFromPalette("Countdown") },
        { title: "Insert 3D Flip Box", cat: "Add Element", icon: "fa-solid fa-arrows-rotate", run: () => window.insertWidgetFromPalette("FlipBox") },
        { title: "Insert Animated Headline", cat: "Add Element", icon: "fa-solid fa-wand-magic-sparkles", run: () => window.insertWidgetFromPalette("AnimatedHeadline") },
        { title: "Insert Navigation Menu", cat: "Add Element", icon: "fa-solid fa-bars", run: () => window.insertWidgetFromPalette("NavMenu") },
        { title: "Insert Post Loop Grid", cat: "Add Element", icon: "fa-solid fa-table-cells", run: () => window.insertWidgetFromPalette("LoopGrid") },
        { title: "Insert Star Rating", cat: "Add Element", icon: "fa-solid fa-star", run: () => window.insertWidgetFromPalette("StarRating") },
        { title: "Insert Progress Bar", cat: "Add Element", icon: "fa-solid fa-bars-progress", run: () => window.insertWidgetFromPalette("ProgressBar") },
        { title: "Insert Alert Box", cat: "Add Element", icon: "fa-solid fa-triangle-exclamation", run: () => window.insertWidgetFromPalette("Alert") },
        { title: "Insert Benchmark HUD", cat: "Add Element", icon: "fa-solid fa-gauge-high", run: () => window.insertWidgetFromPalette("Benchmark") },
        { title: "Insert Social Share Buttons", cat: "Add Element", icon: "fa-solid fa-share-nodes", run: () => window.insertWidgetFromPalette("ShareButtons") }
    ];

    let activeIndex = 0;
    let filteredList = [];

    function initCommandPalette() {
        const modal = document.getElementById("modal-finder");
        const input = document.getElementById("finder-input");
        const results = document.getElementById("finder-results");
        const btnOpen = document.getElementById("btn-open-palette");

        if (!modal || !input || !results) return;

        function openPalette() {
            modal.classList.add("open");
            input.value = "";
            activeIndex = 0;
            renderResults("");
            input.focus();
        }

        function closePalette() {
            modal.classList.remove("open");
        }

        if (btnOpen) btnOpen.onclick = openPalette;
        modal.onclick = (e) => { if (e.target === modal) closePalette(); };

        // Global hotkey Cmd+K / Ctrl+K
        document.addEventListener("keydown", (e) => {
            if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "K")) {
                e.preventDefault();
                if (modal.classList.contains("open")) closePalette();
                else openPalette();
            } else if (e.key === "Escape" && modal.classList.contains("open")) {
                closePalette();
            }
        });

        input.addEventListener("input", () => {
            activeIndex = 0;
            renderResults(input.value.trim().toLowerCase());
        });

        input.addEventListener("keydown", (e) => {
            if (e.key === "ArrowDown") {
                e.preventDefault();
                if (filteredList.length > 0) {
                    activeIndex = (activeIndex + 1) % filteredList.length;
                    highlightItem();
                }
            } else if (e.key === "ArrowUp") {
                e.preventDefault();
                if (filteredList.length > 0) {
                    activeIndex = (activeIndex - 1 + filteredList.length) % filteredList.length;
                    highlightItem();
                }
            } else if (e.key === "Enter") {
                e.preventDefault();
                if (filteredList[activeIndex]) {
                    closePalette();
                    filteredList[activeIndex].run();
                }
            }
        });
    }

    function renderResults(q) {
        const results = document.getElementById("finder-results");
        if (!results) return;
        results.innerHTML = "";

        filteredList = COMMANDS.filter(cmd => {
            if (!q) return true;
            return cmd.title.toLowerCase().includes(q) || cmd.cat.toLowerCase().includes(q);
        });

        if (filteredList.length === 0) {
            results.innerHTML = `<div style="padding: 12px; color: var(--day-muted); font-size: 12px; text-align: center;">No matching studio commands</div>`;
            return;
        }

        filteredList.forEach((cmd, i) => {
            const item = document.createElement("div");
            item.className = "spotlight-item" + (i === activeIndex ? " active" : "");
            item.innerHTML = `
                <div class="spotlight-item-left">
                    <i class="${cmd.icon}" style="width: 16px; text-align: center; color: var(--day-signal);"></i>
                    <span>${cmd.title}</span>
                </div>
                <span class="spotlight-cat-tag">${cmd.cat}</span>
            `;
            item.onclick = () => {
                document.getElementById("modal-finder").classList.remove("open");
                cmd.run();
            };
            results.appendChild(item);
        });
    }

    function highlightItem() {
        const items = document.querySelectorAll(".spotlight-item");
        items.forEach((it, idx) => {
            it.classList.toggle("active", idx === activeIndex);
            if (idx === activeIndex) it.scrollIntoView({ block: "nearest" });
        });
    }

    document.addEventListener("DOMContentLoaded", () => {
        initCommandPalette();
    });
})();
"###;
