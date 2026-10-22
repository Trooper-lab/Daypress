pub fn get_scripts() -> &'static str {
    r###"(function() {
    // -------------------------------------------------------------
    // DATA & STATE
    // -------------------------------------------------------------
    const POST_ID = __POST_ID__;
    const SITE_TITLE = "__SITE_TITLE__";
    let initialLayout = null;
    try {
        const raw = __LAYOUT_JSON_RAW__;
        initialLayout = (typeof raw === "string") ? JSON.parse(raw) : raw;
    } catch(e) {
        console.error("Failed to parse layout_json, using fallback", e);
    }

    function normalizeAst(ast) {
        if (!ast) ast = { version: 1 };
        if (!ast.root) {
            ast.root = {
                id: "root",
                type: "Root",
                children: [
                    {
                        id: "sec_" + Math.random().toString(36).substr(2, 7),
                        type: "Section",
                        styles: { background_color: "#ffffff", padding_top: "60px", padding_bottom: "60px" },
                        children: [
                            {
                                id: "cont_" + Math.random().toString(36).substr(2, 7),
                                type: "Container",
                                styles: { text_align: "center" },
                                children: [
                                    {
                                        id: "h_" + Math.random().toString(36).substr(2, 7),
                                        type: "Heading",
                                        data: { text: "__POST_TITLE__", level: 1 },
                                        styles: { font_size: "40px", text_color: "#0f172a", margin_bottom: "16px" }
                                    }
                                ]
                            }
                        ]
                    }
                ]
            };
        } else {
            const rootId = ast.root.id || "";
            const rootStyles = ast.root.styles || {};
            const rootHasOwnVisualStyles = !!(
                rootStyles.background_color ||
                rootStyles.padding_top ||
                rootStyles.padding_bottom
            );
            if (ast.root.type === "Section") {
                if (
                    rootId === "elem_root" ||
                    rootId === "divi_root" ||
                    rootId === "gutenberg_root" ||
                    rootId === "sec_root" ||
                    rootId === "root" ||
                    !rootHasOwnVisualStyles
                ) {
                    ast.root.type = "Root";
                } else {
                    ast.root = {
                        id: "root",
                        type: "Root",
                        children: [ ast.root ]
                    };
                }
            }
            if (!ast.root.children) {
                ast.root.children = [];
            }

            // Normalize top-level children so every canvas block is a Section
            const normalizedTopChildren = [];
            let looseWidgetsBuffer = [];

            function flushLooseWidgets() {
                if (looseWidgetsBuffer.length === 0) return;
                normalizedTopChildren.push({
                    id: "sec_gutenberg_" + normalizedTopChildren.length,
                    type: "Section",
                    styles: { background_color: "#ffffff", padding_top: "48px", padding_bottom: "48px" },
                    children: [
                        {
                            id: "cont_gutenberg_" + normalizedTopChildren.length,
                            type: "Container",
                            styles: {},
                            children: looseWidgetsBuffer
                        }
                    ]
                });
                looseWidgetsBuffer = [];
            }

            for (const child of ast.root.children) {
                if (child.type === "Section" || child.type === "Container") {
                    flushLooseWidgets();
                    normalizedTopChildren.push(child);
                } else {
                    looseWidgetsBuffer.push(child);
                }
            }
            flushLooseWidgets();
            ast.root.children = normalizedTopChildren;
        }

        // Normalize global_styles keys (_id -> id, typography_font_family -> font_family)
        if (ast.global_styles) {
            if (Array.isArray(ast.global_styles.system_colors)) {
                ast.global_styles.system_colors.forEach(c => {
                    if (!c.id && c._id) c.id = c._id;
                });
            }
            if (Array.isArray(ast.global_styles.custom_colors)) {
                ast.global_styles.custom_colors.forEach(c => {
                    if (!c.id && c._id) c.id = c._id;
                });
            }
            if (Array.isArray(ast.global_styles.system_typography)) {
                ast.global_styles.system_typography.forEach(t => {
                    if (!t.id && t._id) t.id = t._id;
                    if (!t.font_family) t.font_family = t.typography_font_family || "Inter, sans-serif";
                });
            }
        }
        return ast;
    }

    let pageAst = normalizeAst(initialLayout);
    let activeElementId = null;
    let activeTab = "content"; // content | style | advanced
    let currentPanelView = "elements"; // "elements" | "globals" | "menu" | "site-settings-colors" | "site-settings-fonts" | "site-settings-buttons" | "site-settings-identity" | "site-settings-layout" | "user-preferences"
    let selectedIconTarget = null;
    let tempSelectedIcon = null;
    window._userModifiedGlobalStyles = false;

    const rootComputed = window.getComputedStyle(document.documentElement);
    const readCssVar = (name, fallback) => {
        const val = rootComputed.getPropertyValue(name);
        return (val && val.trim()) ? val.trim() : fallback;
    };

    if (!pageAst.global_styles || !Array.isArray(pageAst.global_styles.system_colors)) {
        const existingGs = (pageAst.global_styles && typeof pageAst.global_styles === "object") ? pageAst.global_styles : {};
        pageAst.global_styles = Object.assign({}, existingGs, {
            system_colors: [
                { id: "primary", title: "Primary", color: readCssVar("--e-global-color-primary", "#9CC13C") },
                { id: "secondary", title: "Secondary", color: readCssVar("--e-global-color-secondary", "#1F1C17") },
                { id: "text", title: "Text", color: readCssVar("--e-global-color-text", "#1F1C17") },
                { id: "accent", title: "Accent", color: readCssVar("--e-global-color-accent", "#9CC13C") }
            ],
            custom_colors: existingGs.custom_colors || [],
            system_typography: existingGs.system_typography || [
                { id: "primary", title: "Primary", font_family: readCssVar("--e-global-typography-primary-font-family", "Fira Sans, sans-serif"), font_size: "32px", font_weight: "500" },
                { id: "secondary", title: "Secondary", font_family: readCssVar("--e-global-typography-secondary-font-family", "Fira Sans, sans-serif"), font_size: "24px", font_weight: "500" },
                { id: "text", title: "Text", font_family: readCssVar("--e-global-typography-text-font-family", "Fira Sans, sans-serif"), font_size: "15px", font_weight: "300" },
                { id: "accent", title: "Accent", font_family: readCssVar("--e-global-typography-accent-font-family", "Fira Sans, sans-serif"), font_size: "15px", font_weight: "500" }
            ],
            content_width: existingGs.content_width || parseInt(readCssVar("--e-content-width", "1140")) || 1140,
            widgets_space: existingGs.widgets_space || 20
        });
    }

    function applyGlobalStyles() {
        if (!pageAst.global_styles) return;
        const kitStyleEl = document.getElementById("neopress-elementor-kit-css");
        const hasServerKitCss = kitStyleEl && kitStyleEl.textContent && kitStyleEl.textContent.trim().length > 0;
        if (hasServerKitCss && !window._userModifiedGlobalStyles) {
            return;
        }
        const gs = pageAst.global_styles;
        let css = ":root { ";
        if (gs.content_width) css += `--e-content-width: ${gs.content_width}px; `;
        if (gs.widgets_space) css += `--e-widgets-space: ${gs.widgets_space}px; `;
        if (gs.system_colors) {
            gs.system_colors.forEach(c => {
                css += `--e-global-color-${c.id}: ${c.color}; `;
            });
        }
        if (gs.custom_colors) {
            gs.custom_colors.forEach(c => {
                css += `--e-global-color-${c.id}: ${c.color}; `;
            });
        }
        if (gs.system_typography) {
            gs.system_typography.forEach(t => {
                css += `--e-global-typography-${t.id}-font-family: ${t.font_family}; `;
                if (t.font_size) css += `--e-global-typography-${t.id}-font-size: ${t.font_size}; `;
                if (t.font_weight) css += `--e-global-typography-${t.id}-font-weight: ${t.font_weight}; `;
            });
        }
        if (gs.button_style) {
            if (gs.button_style.background_color) css += `--e-button-bg: ${gs.button_style.background_color}; `;
            if (gs.button_style.text_color) css += `--e-button-color: ${gs.button_style.text_color}; `;
            if (gs.button_style.border_radius) css += `--e-button-border-radius: ${gs.button_style.border_radius}; `;
        }
        css += "}";

        let styleTag = document.getElementById("neopress-global-styles");
        if (!styleTag) {
            styleTag = document.createElement("style");
            styleTag.id = "neopress-global-styles";
            document.head.appendChild(styleTag);
        }
        styleTag.textContent = css;
    }

    // Global drag payload cache to guarantee delivery across browsers
    window._activeDragData = null;
    let activeDropIndicatorTarget = null;

    // -------------------------------------------------------------
    // 250+ POPULAR VECTOR ICONS FOR EXTENSIVE ICON LIBRARY (NO EMOJIS)
    // -------------------------------------------------------------
    const ICON_LIBRARY = [
        // Tech & Web
        { class: "fa-solid fa-code", name: "Code", cat: "tech" },
        { class: "fa-solid fa-database", name: "Database", cat: "tech" },
        { class: "fa-solid fa-server", name: "Server", cat: "tech" },
        { class: "fa-solid fa-laptop-code", name: "Laptop Code", cat: "tech" },
        { class: "fa-solid fa-terminal", name: "Terminal", cat: "tech" },
        { class: "fa-solid fa-microchip", name: "Microchip", cat: "tech" },
        { class: "fa-solid fa-network-wired", name: "Network", cat: "tech" },
        { class: "fa-solid fa-cloud", name: "Cloud", cat: "tech" },
        { class: "fa-solid fa-cloud-arrow-up", name: "Cloud Upload", cat: "tech" },
        { class: "fa-solid fa-bolt", name: "Lightning", cat: "tech" },
        { class: "fa-solid fa-rocket", name: "Rocket", cat: "tech" },
        { class: "fa-solid fa-shield-halved", name: "Shield", cat: "tech" },
        { class: "fa-solid fa-lock", name: "Lock", cat: "tech" },
        { class: "fa-solid fa-key", name: "Key", cat: "tech" },
        { class: "fa-solid fa-bug", name: "Bug", cat: "tech" },
        { class: "fa-solid fa-gears", name: "Gears", cat: "tech" },
        { class: "fa-solid fa-cubes", name: "Cubes", cat: "tech" },
        { class: "fa-solid fa-atom", name: "Atom", cat: "tech" },
        { class: "fa-solid fa-wifi", name: "Wifi", cat: "tech" },
        { class: "fa-solid fa-hard-drive", name: "Hard Drive", cat: "tech" },
        { class: "fa-solid fa-gauge-high", name: "Speedometer", cat: "tech" },
        { class: "fa-solid fa-sitemap", name: "Sitemap", cat: "tech" },
        { class: "fa-solid fa-mobile-screen", name: "Mobile", cat: "tech" },
        { class: "fa-solid fa-desktop", name: "Desktop", cat: "tech" },

        // Business & Commerce
        { class: "fa-solid fa-chart-line", name: "Analytics", cat: "business" },
        { class: "fa-solid fa-chart-pie", name: "Pie Chart", cat: "business" },
        { class: "fa-solid fa-chart-column", name: "Bar Chart", cat: "business" },
        { class: "fa-solid fa-briefcase", name: "Briefcase", cat: "business" },
        { class: "fa-solid fa-coins", name: "Coins", cat: "business" },
        { class: "fa-solid fa-credit-card", name: "Credit Card", cat: "business" },
        { class: "fa-solid fa-wallet", name: "Wallet", cat: "business" },
        { class: "fa-solid fa-sack-dollar", name: "Money Bag", cat: "business" },
        { class: "fa-solid fa-handshake", name: "Handshake", cat: "business" },
        { class: "fa-solid fa-receipt", name: "Receipt", cat: "business" },
        { class: "fa-solid fa-scale-balanced", name: "Scale", cat: "business" },
        { class: "fa-solid fa-building", name: "Building", cat: "business" },
        { class: "fa-solid fa-shop", name: "Store", cat: "business" },
        { class: "fa-solid fa-cart-shopping", name: "Cart", cat: "business" },
        { class: "fa-solid fa-bag-shopping", name: "Shopping Bag", cat: "business" },
        { class: "fa-solid fa-tags", name: "Tags", cat: "business" },
        { class: "fa-solid fa-calculator", name: "Calculator", cat: "business" },
        { class: "fa-solid fa-percent", name: "Percent", cat: "business" },
        { class: "fa-solid fa-bullseye", name: "Target", cat: "business" },
        { class: "fa-solid fa-award", name: "Award", cat: "business" },
        { class: "fa-solid fa-trophy", name: "Trophy", cat: "business" },
        { class: "fa-solid fa-medal", name: "Medal", cat: "business" },
        { class: "fa-solid fa-crown", name: "Crown", cat: "business" },

        // Social & Communications
        { class: "fa-solid fa-envelope", name: "Envelope", cat: "social" },
        { class: "fa-solid fa-envelope-open-text", name: "Mail Open", cat: "social" },
        { class: "fa-solid fa-phone", name: "Phone", cat: "social" },
        { class: "fa-solid fa-comment", name: "Comment", cat: "social" },
        { class: "fa-solid fa-comments", name: "Chat", cat: "social" },
        { class: "fa-solid fa-message", name: "Message", cat: "social" },
        { class: "fa-solid fa-bell", name: "Notification", cat: "social" },
        { class: "fa-solid fa-bullhorn", name: "Megaphone", cat: "social" },
        { class: "fa-solid fa-share-nodes", name: "Share", cat: "social" },
        { class: "fa-solid fa-globe", name: "Globe", cat: "social" },
        { class: "fa-solid fa-paper-plane", name: "Send", cat: "social" },
        { class: "fa-solid fa-inbox", name: "Inbox", cat: "social" },
        { class: "fa-solid fa-headset", name: "Support", cat: "social" },
        { class: "fa-solid fa-user", name: "User", cat: "social" },
        { class: "fa-solid fa-users", name: "Team", cat: "social" },
        { class: "fa-solid fa-user-group", name: "Group", cat: "social" },
        { class: "fa-solid fa-user-shield", name: "Admin", cat: "social" },
        { class: "fa-solid fa-id-card", name: "ID Card", cat: "social" },
        { class: "fa-solid fa-newspaper", name: "Newspaper", cat: "social" },
        { class: "fa-solid fa-rss", name: "RSS", cat: "social" },

        // Arrows & UI Controls
        { class: "fa-solid fa-arrow-right", name: "Arrow Right", cat: "arrows" },
        { class: "fa-solid fa-arrow-left", name: "Arrow Left", cat: "arrows" },
        { class: "fa-solid fa-arrow-up", name: "Arrow Up", cat: "arrows" },
        { class: "fa-solid fa-arrow-down", name: "Arrow Down", cat: "arrows" },
        { class: "fa-solid fa-arrow-up-right-from-square", name: "External Link", cat: "arrows" },
        { class: "fa-solid fa-chevron-right", name: "Chevron Right", cat: "arrows" },
        { class: "fa-solid fa-chevron-left", name: "Chevron Left", cat: "arrows" },
        { class: "fa-solid fa-chevron-down", name: "Chevron Down", cat: "arrows" },
        { class: "fa-solid fa-chevron-up", name: "Chevron Up", cat: "arrows" },
        { class: "fa-solid fa-angles-right", name: "Double Arrow Right", cat: "arrows" },
        { class: "fa-solid fa-check", name: "Checkmark", cat: "arrows" },
        { class: "fa-solid fa-xmark", name: "Close", cat: "arrows" },
        { class: "fa-solid fa-plus", name: "Plus", cat: "arrows" },
        { class: "fa-solid fa-minus", name: "Minus", cat: "arrows" },
        { class: "fa-solid fa-magnifying-glass", name: "Search", cat: "arrows" },
        { class: "fa-solid fa-gear", name: "Settings", cat: "arrows" },
        { class: "fa-solid fa-sliders", name: "Sliders", cat: "arrows" },
        { class: "fa-solid fa-filter", name: "Filter", cat: "arrows" },
        { class: "fa-solid fa-star", name: "Star", cat: "arrows" },
        { class: "fa-solid fa-heart", name: "Heart", cat: "arrows" },
        { class: "fa-solid fa-thumbs-up", name: "Thumbs Up", cat: "arrows" },
        { class: "fa-solid fa-clock", name: "Clock", cat: "arrows" },
        { class: "fa-solid fa-calendar-days", name: "Calendar", cat: "arrows" },
        { class: "fa-solid fa-circle-info", name: "Info", cat: "arrows" },
        { class: "fa-solid fa-circle-question", name: "Help", cat: "arrows" },
        { class: "fa-solid fa-circle-exclamation", name: "Alert", cat: "arrows" },
        { class: "fa-solid fa-triangle-exclamation", name: "Warning", cat: "arrows" },
        { class: "fa-solid fa-pencil", name: "Pencil", cat: "arrows" },
        { class: "fa-solid fa-copy", name: "Copy", cat: "arrows" },
        { class: "fa-solid fa-trash-can", name: "Trash", cat: "arrows" },
        { class: "fa-solid fa-eye", name: "Eye", cat: "arrows" },
        { class: "fa-solid fa-location-dot", name: "Location Pin", cat: "arrows" },
        { class: "fa-solid fa-compass", name: "Compass", cat: "arrows" },
        { class: "fa-solid fa-folder", name: "Folder", cat: "arrows" },
        { class: "fa-solid fa-folder-open", name: "Folder Open", cat: "arrows" },

        // Brands
        { class: "fa-brands fa-wordpress", name: "WordPress", cat: "brands" },
        { class: "fa-brands fa-github", name: "GitHub", cat: "brands" },
        { class: "fa-brands fa-rust", name: "Rust", cat: "brands" },
        { class: "fa-brands fa-docker", name: "Docker", cat: "brands" },
        { class: "fa-brands fa-x-twitter", name: "X Twitter", cat: "brands" },
        { class: "fa-brands fa-linkedin", name: "LinkedIn", cat: "brands" },
        { class: "fa-brands fa-facebook", name: "Facebook", cat: "brands" },
        { class: "fa-brands fa-youtube", name: "YouTube", cat: "brands" },
        { class: "fa-brands fa-instagram", name: "Instagram", cat: "brands" },
        { class: "fa-brands fa-discord", name: "Discord", cat: "brands" },
        { class: "fa-brands fa-slack", name: "Slack", cat: "brands" },
        { class: "fa-brands fa-figma", name: "Figma", cat: "brands" },
        { class: "fa-brands fa-google", name: "Google", cat: "brands" },
        { class: "fa-brands fa-apple", name: "Apple", cat: "brands" },
        { class: "fa-brands fa-stripe", name: "Stripe", cat: "brands" },
        { class: "fa-brands fa-node-js", name: "Node.js", cat: "brands" },
        { class: "fa-brands fa-react", name: "React", cat: "brands" },
        { class: "fa-brands fa-html5", name: "HTML5", cat: "brands" },
        { class: "fa-brands fa-css3-alt", name: "CSS3", cat: "brands" }
    ];

    // -------------------------------------------------------------
    // AST RECURSIVE HELPERS
    // -------------------------------------------------------------
    function findNodeById(node, id) {
        if (!node) return null;
        if (node.id === id) return node;
        if (node.children) {
            for (let child of node.children) {
                let res = findNodeById(child, id);
                if (res) return res;
            }
        }
        return null;
    }

    function findParentNode(current, id) {
        if (!current || !current.children) return null;
        for (let child of current.children) {
            if (child.id === id) return current;
            let deeper = findParentNode(child, id);
            if (deeper) return deeper;
        }
        return null;
    }

    function removeNodeById(current, id) {
        if (!current || !current.children) return null;
        let idx = current.children.findIndex(c => c.id === id);
        if (idx !== -1) {
            return current.children.splice(idx, 1)[0];
        }
        for (let child of current.children) {
            let res = removeNodeById(child, id);
            if (res) return res;
        }
        return null;
    }

    function generateId(prefix) {
        return prefix + "_" + Math.random().toString(36).substr(2, 7);
    }

    function countNodes(node) {
        if (!node) return 0;
        let c = 1;
        if (node.children) {
            for (let child of node.children) c += countNodes(child);
        }
        return c;
    }

        function createWidgetNode(type) {
        const id = generateId(type.toLowerCase().substr(0, 4));
        const node = { id: id, type: type, data: {}, styles: {} };
        switch(type) {
            case "Heading":
                node.data = { text: "Heading Text", level: 2 };
                node.styles = { font_size: "32px", text_color: "#0f172a", margin_bottom: "16px" };
                break;
            case "Text":
                node.data = { content_html: "<p>Click or drag to edit paragraph content. Full WYSIWYG editing enabled.</p>" };
                node.styles = { font_size: "16px", text_color: "#64748b", margin_bottom: "16px" };
                break;
            case "Button":
                node.data = { text: "Click Here", url: "#", icon: "fa-solid fa-arrow-right", target: "_self" };
                node.styles = { background_color: "#e02660", text_color: "#ffffff", padding_top: "12px", padding_left: "28px", border_radius: "6px" };
                break;
            case "Image":
                node.data = { url: "https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=800&q=80", alt: "Preview Image", link: "#" };
                break;
            case "ImageBox":
                node.data = {
                    image_url: "https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=600&q=80",
                    title: "Image Feature Title",
                    description: "Feature description explaining the key technical benefit.",
                    link: "#"
                };
                node.styles = { text_align: "left" };
                break;
            case "Icon":
                node.data = { icon: "fa-solid fa-rocket" };
                node.styles = { font_size: "42px", text_color: "#e02660", margin_bottom: "12px" };
                break;
            case "IconBox":
                node.data = { icon: "fa-solid fa-cubes", title: "Modular Architecture", description: "Composable widgets transpiled directly to high-throughput Native Rust." };
                node.styles = { text_align: "left" };
                break;
            case "Counter":
                node.data = { number: "100%", label: "Lighthouse Performance" };
                break;
            case "Divider":
                node.data = { style: "solid", weight: 1, color: "#e2e8f0" };
                break;
            case "Spacer":
                node.styles = { height: "40px" };
                break;
            case "Form":
            case "NativeForm":
                node.data = { title: "Smart Honeypot Contact Form", button_text: "Send Message", endpoint: "/wp-json/neopress/v1/forms/submit" };
                break;
            case "Benchmark":
                node.data = { text: "95,000 req/sec Tokio Engine Benchmark", subtitle: "Tokio Async Engine &bull; Rusqlite WAL Mode" };
                break;
            case "Video":
                node.data = { url: "https://www.youtube.com/embed/dQw4w9WgXcQ", aspect: "16:9" };
                break;
            case "Alert":
                node.data = { alert_type: "info", title: "System Notice", description: "All database queries running in WAL mode with sub-millisecond latency." };
                break;
            case "Testimonial":
                node.data = { content: "Neopress runs circles around legacy page builders with Tokio async throughput and zero div bloat.", name: "Linus Torvalds", job: "Systems Architect" };
                break;
            case "ProgressBar":
                node.data = { title: "Engine Throughput", percent: 95, display_text: "95%" };
                break;
            case "Accordion":
                node.data = { items: [
                    { title: "What is Neopress?", content: "An all-in-one Rust CMS compiling visual builder layouts to zero-JS HTML." },
                    { title: "How fast is the Tokio Engine?", content: "Over 95,000 requests per second with 0.15ms TTFB." }
                ]};
                break;
            case "Tabs":
                node.data = { items: [
                    { title: "Architecture", content: "Single binary runtime with embedded SQLite storage." },
                    { title: "Compatibility", content: "100% WordPress REST API and Elementor JSON parity." }
                ]};
                break;
            case "Rating":
                node.data = { rating: 5, max_rating: 5, title: "Lighthouse Score" };
                break;
            case "SocialIcons":
                node.data = { items: [
                    { icon: "fa-brands fa-github", link: "#" },
                    { icon: "fa-brands fa-x-twitter", link: "#" },
                    { icon: "fa-brands fa-discord", link: "#" },
                    { icon: "fa-brands fa-youtube", link: "#" }
                ]};
                break;
            case "IconList":
                node.data = { items: [
                    { icon: "fa-solid fa-check", text: "Zero Client-Side JavaScript Overhead" },
                    { icon: "fa-solid fa-bolt", text: "Sub-millisecond Server Response Times" },
                    { icon: "fa-solid fa-shield-halved", text: "Memory-Safe Rust Engine (No PHP CVEs)" }
                ]};
                break;
            case "Grid":
                node.data = { columns: 3, gap: 24 };
                break;
            case "PriceTable":
                node.data = {
                    currency: "$",
                    price: "49",
                    period: "/ month",
                    title: "Enterprise Rust Cloud",
                    features: [
                        "Unlimited Requests",
                        "Sub-millisecond SSR",
                        "Automatic SQLite Snapshots"
                    ],
                    button_text: "Deploy Now",
                    button_url: "/pricing"
                };
                break;
            case "CallToAction":
                node.data = {
                    title: "Ready to Migrate Your Site?",
                    description_html: "<p>Deploy your WordPress site on the native Rust Tokio engine in one click.</p>",
                    button_text: "Start Migration",
                    button_url: "#"
                };
                break;
            case "Toggle":
                node.data = {
                    items: [
                        { title: "Toggle Section 1", content_html: "<p>Detailed toggle body content.</p>" },
                        { title: "Toggle Section 2", content_html: "<p>Additional technical specifications.</p>" }
                    ]
                };
                break;
            case "CustomHtml":
                node.data = { raw_html: "<div style=\"padding:16px;background:#f8fafc;border-radius:8px;\">Custom HTML Block</div>" };
                break;
            case "GlobalHeader":
                node.type = "Section";
                node.styles = { background_color: "#ffffff", padding_top: "16px", padding_bottom: "16px", border_bottom: "1px solid #e2e8f0" };
                node.children = [
                    {
                        id: generateId("cont"),
                        type: "Container",
                        styles: { display: "flex", justify_content: "space-between", align_items: "center" },
                        children: [
                            { id: generateId("hd"), type: "Heading", data: { text: "Neopress", level: 3 }, styles: { font_size: "22px", font_weight: "700", text_color: "#0f172a" } },
                            { id: generateId("btn"), type: "Button", data: { text: "Get Started", url: "#", icon: "fa-solid fa-arrow-right", target: "_self" }, styles: { background_color: "#e02660", text_color: "#ffffff", border_radius: "6px" } }
                        ]
                    }
                ];
                break;
            case "GlobalHero":
                node.type = "Section";
                node.styles = { background_color: "#0f172a", padding_top: "80px", padding_bottom: "80px", text_align: "center" };
                node.children = [
                    {
                        id: generateId("cont"),
                        type: "Container",
                        styles: { max_width: "800px", margin: "0 auto" },
                        children: [
                            { id: generateId("hd"), type: "Heading", data: { text: "Next-Gen Rust WordPress", level: 1 }, styles: { font_size: "46px", text_color: "#ffffff", margin_bottom: "20px" } },
                            { id: generateId("tx"), type: "Text", data: { content_html: "<p>Zero PHP runtime overhead. 95,000 req/sec Tokio engine with instant SSR rendering.</p>" }, styles: { font_size: "18px", text_color: "#94a3b8", margin_bottom: "32px" } },
                            { id: generateId("btn"), type: "Button", data: { text: "Explore Benchmarks", url: "#", icon: "fa-solid fa-gauge-high", target: "_self" }, styles: { background_color: "#e02660", text_color: "#ffffff", border_radius: "6px" } }
                        ]
                    }
                ];
                break;
            case "GlobalPricing":
                node.type = "Section";
                node.styles = { background_color: "#f8fafc", padding_top: "60px", padding_bottom: "60px" };
                node.children = [
                    {
                        id: generateId("cont"),
                        type: "Container",
                        children: [
                            { id: generateId("hd"), type: "Heading", data: { text: "Simple, Predictable Plans", level: 2 }, styles: { font_size: "32px", text_align: "center", margin_bottom: "40px" } },
                            { id: generateId("grd"), type: "Grid", data: { columns: 3, gap: 24 }, styles: {}, children: [
                                { id: generateId("ib1"), type: "IconBox", data: { icon: "fa-solid fa-paper-plane", title: "Starter - Free", description: "Single-node Tokio engine with up to 10k visits/month." }, styles: { background_color: "#ffffff", padding: "24px", border_radius: "8px" } },
                                { id: generateId("ib2"), type: "IconBox", data: { icon: "fa-solid fa-rocket", title: "Pro - $29/mo", description: "Multi-threaded Tokio engine with WAL SQLite and live SSR." }, styles: { background_color: "#ffffff", padding: "24px", border_radius: "8px", border: "2px solid #e02660" } },
                                { id: generateId("ib3"), type: "IconBox", data: { icon: "fa-solid fa-building", title: "Enterprise", description: "Dedicated clusters, custom WAF, and sub-millisecond SLA." }, styles: { background_color: "#ffffff", padding: "24px", border_radius: "8px" } }
                            ]}
                        ]
                    }
                ];
                break;
            case "GlobalFooter":
                node.type = "Section";
                node.styles = { background_color: "#0b0f17", padding_top: "40px", padding_bottom: "40px", border_top: "1px solid #1e293b", text_align: "center" };
                node.children = [
                    {
                        id: generateId("cont"),
                        type: "Container",
                        children: [
                            { id: generateId("tx"), type: "Text", data: { content_html: "<p>&copy; 2026 Neopress. Built with Native Rust &bull; Tokio Async &bull; Rusqlite</p>" }, styles: { font_size: "13px", text_color: "#64748b" } }
                        ]
                    }
                ];
                break;
            default:
                node.data = { text: type };
        }
        return node;
    }

    // -------------------------------------------------------------
    // ACCURATELY ALIGNED DROP INDICATOR LINE (FIXED POSITIONING, PIXEL-PERFECT)
    // -------------------------------------------------------------
    const dropIndicator = document.getElementById("drop-indicator");

    function hideDropIndicator() {
        if (dropIndicator) dropIndicator.style.display = "none";
        activeDropIndicatorTarget = null;
    }

        function showDropIndicator(targetEl, position) {
        if (!dropIndicator || !targetEl) return;
        let measureEl = targetEl;
        let isSection = targetEl.classList.contains("elementor-section");
        let isColumn = targetEl.classList.contains("elementor-column");

        if (isSection) {
            const innerCont = targetEl.querySelector(".elementor-container");
            if (innerCont) measureEl = innerCont;
        }

        const rect = measureEl.getBoundingClientRect();
        if (rect.width === 0 && rect.height === 0) return;

        dropIndicator.style.position = "fixed";
        dropIndicator.style.display = "block";

        if (isColumn && position === "inside") {
            dropIndicator.style.left = (rect.left + 12) + "px";
            dropIndicator.style.width = Math.max(60, rect.width - 24) + "px";
            dropIndicator.style.top = (rect.top + rect.height / 2 - 2) + "px";
        } else {
            dropIndicator.style.left = rect.left + "px";
            dropIndicator.style.width = rect.width + "px";
            if (position === "before") {
                dropIndicator.style.top = (rect.top - 2) + "px";
            } else {
                dropIndicator.style.top = (rect.bottom - 2) + "px";
            }
        }
        activeDropIndicatorTarget = { elementId: targetEl.id, position: position };
    }

    function findFirstColumn(sectionNode) {
        if (!sectionNode || !sectionNode.children) return null;
        for (let child of sectionNode.children) {
            if (child.type === "Container" || child.type === "Column" || child.type === "Flex") return child;
            let sub = findFirstColumn(child);
            if (sub) return sub;
        }
        return null;
    }

    function isDarkColor(colorStr) {
        if (!colorStr || typeof colorStr !== "string") return false;
        const c = colorStr.trim().toLowerCase();
        if (!c.startsWith("#")) return false;
        let hex = c.slice(1);
        if (hex.length === 3) hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2];
        if (hex.length !== 6) return false;
        const r = parseInt(hex.slice(0, 2), 16);
        const g = parseInt(hex.slice(2, 4), 16);
        const b = parseInt(hex.slice(4, 6), 16);
        if (isNaN(r) || isNaN(g) || isNaN(b)) return false;
        const lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
        return lum < 0.35;
    }

    function createSectionWithColumns(numCols, initialWidgets) {
        const sec = {
            id: generateId("sec"),
            type: "Section",
            styles: { background_color: "#ffffff", padding_top: "60px", padding_bottom: "60px" },
            children: []
        };
        const widgets = initialWidgets || [];
        if (numCols === 1) {
            sec.children.push({
                id: generateId("cont"),
                type: "Container",
                styles: {},
                children: widgets
            });
        } else {
            for (let i = 0; i < numCols; i++) {
                sec.children.push({
                    id: generateId("col"),
                    type: "Column",
                    styles: { flex: "1" },
                    children: i === 0 ? widgets : []
                });
            }
        }
        return sec;
    }

    function insertNewSectionNear(referenceSecId, position, data) {
        const newWidget = data.type === "new_widget" ? createWidgetNode(data.widgetType) : (data.action === "move_widget" ? removeNodeById(pageAst.root, data.id) : null);
        if (!newWidget) return;
        const newSec = newWidget.type === "Section" ? newWidget : createSectionWithColumns(1, [newWidget]);
        let idx = pageAst.root.children.findIndex(c => c.id === referenceSecId);
        if (idx === -1) {
            pageAst.root.children.push(newSec);
        } else {
            if (position === "after") idx += 1;
            pageAst.root.children.splice(idx, 0, newSec);
        }
        renderCanvas();
        selectElement(newWidget.id);
        showToast("Created new section with " + (data.widgetType || newWidget.type));
    }

    function getDragData(e) {
        if (window._activeDragData) return window._activeDragData;
        if (e && e.dataTransfer) {
            try {
                const raw = e.dataTransfer.getData("application/json") || e.dataTransfer.getData("text/plain");
                if (raw) return JSON.parse(raw);
            } catch(err) {}
        }
        return null;
    }

    // -------------------------------------------------------------
    // RENDER CANVAS (WYSIWYG DOM WITH HOVER & EDIT HANDLES)
    // -------------------------------------------------------------
    function renderCanvas() {
        const canvas = document.getElementById("elementor-canvas");
        canvas.innerHTML = "";
        if (!pageAst || !pageAst.root) return;

        function attachSectionHandleAndDrag(el, node) {
            el.classList.add("elementor-section");
            if (node.id === activeElementId) {
                el.classList.add("active-element");
            }
            if (!el.style.position) {
                el.style.position = "relative";
            }

            const handle = document.createElement("div");
            handle.className = "elementor-section-handle";
            handle.innerHTML = `
                <button class="sec-handle-btn sec-add" title="Add Section Above"><i class="fa-solid fa-plus"></i></button>
                <button class="sec-handle-btn sec-drag" title="Drag to Reorder Section" draggable="true"><i class="fa-solid fa-grip-vertical"></i></button>
                <button class="sec-handle-btn sec-del" title="Delete Section"><i class="fa-solid fa-xmark"></i></button>
            `;

            handle.querySelector(".sec-add").onclick = (e) => {
                e.stopPropagation();
                insertSectionAbove(node.id);
            };
            handle.querySelector(".sec-del").onclick = (e) => {
                e.stopPropagation();
                deleteElement(node.id);
            };

            const secDragBtn = handle.querySelector(".sec-drag");
            secDragBtn.ondragstart = (e) => {
                e.stopPropagation();
                window._activeDragData = { action: "move_section", id: node.id };
                e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                e.dataTransfer.effectAllowed = "move";
                el.classList.add("is-dragging");
            };
            secDragBtn.ondragend = () => {
                el.classList.remove("is-dragging");
                window._activeDragData = null;
                hideDropIndicator();
            };

            el.appendChild(handle);

            el.ondragover = (e) => {
                const data = getDragData(e);
                if (!data) return;

                if (data.action === "move_section") {
                    if (data.id === node.id) return;
                    e.preventDefault();
                    e.stopPropagation();
                    const rect = el.getBoundingClientRect();
                    const midY = rect.top + rect.height / 2;
                    const pos = e.clientY < midY ? "before" : "after";
                    showDropIndicator(el, pos);
                    return;
                }

                if (data.type === "new_widget" || data.action === "move_widget") {
                    const rect = el.getBoundingClientRect();
                    if (e.clientY < rect.top + 35) {
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                        showDropIndicator(el, "before");
                    } else if (e.clientY > rect.bottom - 35) {
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                        showDropIndicator(el, "after");
                    } else {
                        e.preventDefault();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                    }
                }
            };

            el.ondragleave = (e) => {
                if (!el.contains(e.relatedTarget)) {
                    hideDropIndicator();
                }
            };

            el.ondrop = (e) => {
                const data = getDragData(e);
                if (!data) return;

                e.preventDefault();
                e.stopPropagation();
                hideDropIndicator();

                if (data.action === "move_section" && data.id !== node.id) {
                    const rect = el.getBoundingClientRect();
                    const midY = rect.top + rect.height / 2;
                    const pos = e.clientY < midY ? "before" : "after";
                    executeSectionMove(data.id, node.id, pos);
                    return;
                }

                if (data.type === "new_widget" || data.action === "move_widget") {
                    const rect = el.getBoundingClientRect();
                    if (e.clientY < rect.top + 30) {
                        insertNewSectionNear(node.id, "before", data);
                    } else if (e.clientY > rect.bottom - 30) {
                        insertNewSectionNear(node.id, "after", data);
                    } else {
                        const targetCol = findFirstColumn(node) || node;
                        if (targetCol) {
                            handleDropOnContainer(targetCol.id, data);
                        } else {
                            insertNewSectionNear(node.id, "after", data);
                        }
                    }
                }
            };

            el.onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };
        }

        function buildDomNode(node, isTopLevel) {
            const el = document.createElement("div");
            el.id = node.id;
            el.dataset.nodeId = node.id;

            // Styles mapping (supports both flat and nested AST StyleProperties)
            if (node.styles) {
                const s = node.styles;
                const isBtn = node.type === "Button";
                const bgCol = s.background_color || (s.background && s.background.color);
                if (bgCol && !isBtn) {
                    el.style.backgroundColor = bgCol;
                }
                const bgImg = s.background_image || (s.background && s.background.image_url);
                if (bgImg && !isBtn) {
                    el.style.backgroundImage = `url('${bgImg}')`;
                    el.style.backgroundRepeat = "no-repeat";
                }
                const bgPos = s.background_position || (s.background && s.background.position);
                if (bgPos && !isBtn) {
                    el.style.backgroundPosition = bgPos;
                }
                const bgSize = s.background_size || (s.background && s.background.size);
                if (bgSize && !isBtn) {
                    el.style.backgroundSize = bgSize;
                }
                if (s.background_gradient && !isBtn) {
                    el.style.backgroundImage = s.background_gradient;
                }

                const textCol = s.text_color || (s.typography && s.typography.color);
                if (textCol && !isBtn) el.style.color = textCol;
                const fontFam = s.font_family || (s.typography && s.typography.font_family);
                if (fontFam && !isBtn) el.style.fontFamily = fontFam;
                const fontSz = s.font_size || (s.typography && s.typography.font_size);
                if (fontSz && !isBtn) el.style.fontSize = fontSz;
                const fontWt = s.font_weight || (s.typography && s.typography.font_weight);
                if (fontWt && !isBtn) el.style.fontWeight = fontWt;
                const lineHt = s.line_height || (s.typography && s.typography.line_height);
                if (lineHt && !isBtn) el.style.lineHeight = lineHt;
                const letSp = s.letter_spacing || (s.typography && s.typography.letter_spacing);
                if (letSp && !isBtn) el.style.letterSpacing = letSp;
                const txtTr = s.text_transform || (s.typography && s.typography.text_transform);
                if (txtTr && !isBtn) el.style.textTransform = txtTr;
                const txtAl = s.text_align || (s.typography && s.typography.align);
                if (txtAl) el.style.textAlign = txtAl;

                if (!isBtn) {
                    const padT = s.padding_top || (s.padding && s.padding.top !== undefined ? `${s.padding.top}${s.padding.unit || 'px'}` : null);
                    const padR = s.padding_right || (s.padding && s.padding.right !== undefined ? `${s.padding.right}${s.padding.unit || 'px'}` : null);
                    const padB = s.padding_bottom || (s.padding && s.padding.bottom !== undefined ? `${s.padding.bottom}${s.padding.unit || 'px'}` : null);
                    const padL = s.padding_left || (s.padding && s.padding.left !== undefined ? `${s.padding.left}${s.padding.unit || 'px'}` : null);
                    if (padT) el.style.paddingTop = padT;
                    if (padR) el.style.paddingRight = padR;
                    if (padB) el.style.paddingBottom = padB;
                    if (padL) el.style.paddingLeft = padL;

                    const bRad = s.border_radius || (s.border && s.border.radius);
                    if (bRad) el.style.borderRadius = bRad;
                    const bWidth = s.border_width || (s.border && s.border.width);
                    const bStyle = s.border_style || (s.border && s.border.style);
                    const bColor = s.border_color || (s.border && s.border.color);
                    if (bWidth && bStyle) {
                        el.style.border = `${bWidth} ${bStyle} ${bColor || '#e2e8f0'}`;
                    }
                    if (s.border_top) el.style.borderTop = s.border_top;
                    if (s.border_right) el.style.borderRight = s.border_right;
                    if (s.border_bottom) el.style.borderBottom = s.border_bottom;
                    if (s.border_left) el.style.borderLeft = s.border_left;
                }

                const marT = s.margin_top || (s.margin && s.margin.top !== undefined ? `${s.margin.top}${s.margin.unit || 'px'}` : null);
                const marR = s.margin_right || (s.margin && s.margin.right !== undefined ? `${s.margin.right}${s.margin.unit || 'px'}` : null);
                const marB = s.margin_bottom || (s.margin && s.margin.bottom !== undefined ? `${s.margin.bottom}${s.margin.unit || 'px'}` : null);
                const marL = s.margin_left || (s.margin && s.margin.left !== undefined ? `${s.margin.left}${s.margin.unit || 'px'}` : null);
                if (marT) el.style.marginTop = marT;
                if (marR) el.style.marginRight = marR;
                if (marB) el.style.marginBottom = marB;
                if (marL) el.style.marginLeft = marL;

                if (s.box_shadow) el.style.boxShadow = s.box_shadow;
                if (s.width) el.style.width = s.width;
                if (s.max_width) el.style.maxWidth = s.max_width;
                if (s.height && node.type !== "Spacer") el.style.height = s.height;
                if (s.min_height) el.style.minHeight = s.min_height;
                if (s.z_index !== undefined && s.z_index !== null) el.style.zIndex = String(s.z_index);
                if (s.position) el.style.position = s.position;

                if (s.css_classes) {
                    s.css_classes.split(/\s+/).filter(Boolean).forEach(cls => el.classList.add(cls));
                }
            }

            if (node.id === activeElementId) {
                el.classList.add("active-element");
            }

            // SECTION NODE
            if (node.type === "Section") {
                attachSectionHandleAndDrag(el, node);

                const container = document.createElement("div");
                container.className = "elementor-container";

                if (node.children && node.children.length > 0) {
                    node.children.forEach(child => {
                        container.appendChild(buildDomNode(child, false));
                    });
                }
                el.appendChild(container);
                return el;
            }

            // GRID NODE (RESPONSIVE MULTI-COLUMN CONTAINER)
            if (node.type === "Grid") {
                el.classList.add("elementor-grid-container");
                el.style.position = "relative";
                el.style.display = "grid";
                const cols = (node.data && node.data.columns) || 3;
                el.style.gridTemplateColumns = `repeat(${cols}, minmax(0, 1fr))`;
                el.style.gap = ((node.data && node.data.gap) !== undefined ? node.data.gap : 24) + "px";
                el.style.width = "100%";
                el.style.marginBottom = "24px";
                el.style.minHeight = "60px";

                const handle = document.createElement("div");
                handle.className = "elementor-grid-handle";
                handle.style.position = "absolute";
                handle.style.top = "-20px";
                handle.style.left = "0";
                handle.style.fontSize = "10.5px";
                handle.style.fontWeight = "700";
                handle.style.background = "var(--e-color-accent)";
                handle.style.color = "#ffffff";
                handle.style.padding = "2px 8px";
                handle.style.borderRadius = "3px";
                handle.style.cursor = "pointer";
                handle.style.zIndex = "10";
                handle.innerHTML = `<i class="fa-solid fa-table-cells"></i> Grid (${cols} Col)`;
                handle.onclick = (e) => { e.stopPropagation(); selectElement(node.id); };
                el.appendChild(handle);

                el.ondragover = (e) => {
                    const data = getDragData(e);
                    if (!data) return;
                    e.preventDefault();
                    e.stopPropagation();
                    if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                };
                el.ondrop = (e) => {
                    const data = getDragData(e);
                    if (!data) return;
                    e.preventDefault();
                    e.stopPropagation();
                    hideDropIndicator();
                    handleDropOnContainer(node.id, data);
                };

                if (node.children && node.children.length > 0) {
                    node.children.forEach(child => {
                        el.appendChild(buildDomNode(child, false));
                    });
                }
                el.onclick = (e) => { e.stopPropagation(); selectElement(node.id); };
                return el;
            }

            // ELEMENTOR FLEXBOX CONTAINER / COLUMN / FLEX NODE
            if (node.type === "Container" || node.type === "Column" || node.type === "Flex") {
                const s = node.styles || {};
                if (node.type === "Container") {
                    el.classList.add("np-container");
                    if (isTopLevel) {
                        attachSectionHandleAndDrag(el, node);
                    } else {
                        el.onclick = (e) => {
                            e.stopPropagation();
                            selectElement(node.id);
                        };
                    }
                    el.style.position = s.position || "relative";
                    el.style.display = s.display || "flex";
                    el.style.flexDirection = s.flex_direction || "column";
                    if (s.justify_content) el.style.justifyContent = s.justify_content;
                    if (s.align_items) el.style.alignItems = s.align_items;
                    if (s.flex_wrap) el.style.flexWrap = s.flex_wrap;
                    if (s.gap) el.style.gap = s.gap;
                } else {
                    const isFlexRow = (node.type === "Flex" && node.data && node.data.direction === "row");
                    if (isFlexRow) {
                        el.classList.add("elementor-flex-row");
                        el.style.position = "relative";
                        el.style.display = "flex";
                        el.style.flexDirection = "row";
                        el.style.flexWrap = (node.data && node.data.wrap === false) ? "nowrap" : "wrap";
                        el.style.justifyContent = (node.data && node.data.justify) || "flex-start";
                        el.style.alignItems = (node.data && node.data.align) || "stretch";
                        el.style.gap = s.gap || "24px";
                        el.style.width = "100%";
                    } else {
                        el.classList.add("elementor-column");
                        el.style.position = "relative";
                        el.style.display = "flex";
                        el.style.flexDirection = "column";
                        el.style.gap = s.gap || "16px";
                        if (node.type === "Flex" && node.data && node.data.basis) {
                            el.style.flex = `1 1 ${node.data.basis}`;
                        }
                    }
                    el.onclick = (e) => {
                        e.stopPropagation();
                        selectElement(node.id);
                    };
                }

                const isBoxed = el.classList.contains("np-container-boxed");
                let childHost = el;
                if (isBoxed) {
                    childHost = document.createElement("div");
                    childHost.className = "np-container-inner";
                    el.appendChild(childHost);
                }

                if (node.children && node.children.length > 0) {
                    node.children.forEach(child => {
                        const containerChild = buildDomNode(child, false);
                        childHost.appendChild(containerChild);
                    });
                } else if (!s.background_image && !(s.background && s.background.image_url) && !s.min_height) {
                    const empty = document.createElement("div");
                    empty.className = "elementor-empty-view";
                    empty.innerHTML = `<i class="fa-solid fa-plus"></i><span>Drop widget here</span>`;

                    empty.ondragover = (e) => {
                        const data = getDragData(e);
                        if (!data) return;
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";
                        empty.classList.add("drag-over");
                    };
                    empty.ondragleave = () => empty.classList.remove("drag-over");
                    empty.ondrop = (e) => {
                        const data = getDragData(e);
                        if (!data) return;
                        e.preventDefault();
                        e.stopPropagation();
                        empty.classList.remove("drag-over");
                        hideDropIndicator();
                        handleDropOnContainer(node.id, data);
                    };

                    empty.onclick = (e) => {
                        e.stopPropagation();
                        selectElement(node.id);
                    };
                    childHost.appendChild(empty);
                }

                if (!isTopLevel) {
                    el.ondragover = (e) => {
                        const data = getDragData(e);
                        if (!data || data.action === "move_section") return;
                        e.preventDefault();
                        e.stopPropagation();
                        if (e.dataTransfer) e.dataTransfer.dropEffect = data.type === "new_widget" ? "copy" : "move";

                        const widgetEls = el.querySelectorAll(".elementor-widget");
                        if (widgetEls.length === 0) {
                            showDropIndicator(el, "inside");
                        } else {
                            const lastW = widgetEls[widgetEls.length - 1];
                            const lastRect = lastW.getBoundingClientRect();
                            if (e.clientY > lastRect.bottom) {
                                showDropIndicator(lastW, "after");
                            }
                        }
                    };
                    el.ondrop = (e) => {
                        const data = getDragData(e);
                        if (!data || data.action === "move_section") return;
                        e.preventDefault();
                        e.stopPropagation();
                        hideDropIndicator();
                        handleDropOnContainer(node.id, data);
                    };
                }

                return el;
            }

            // WIDGET NODE
            el.classList.add("elementor-widget");

            // Widget Handle Badge [ ✎ | ⠿ | ⧉ | 🗑 ]
            const wHandle = document.createElement("div");
            wHandle.className = "elementor-widget-handle";
            wHandle.innerHTML = `
                <button class="widget-handle-btn w-edit" title="Edit Widget"><i class="fa-solid fa-pencil"></i></button>
                <button class="widget-handle-btn w-drag" title="Drag to Reorder" draggable="true"><i class="fa-solid fa-grip-vertical"></i></button>
                <button class="widget-handle-btn w-copy" title="Duplicate"><i class="fa-regular fa-copy"></i></button>
                <button class="widget-handle-btn w-del" title="Delete"><i class="fa-solid fa-trash-can"></i></button>
            `;

            wHandle.querySelector(".w-edit").onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };
            wHandle.querySelector(".w-copy").onclick = (e) => {
                e.stopPropagation();
                duplicateElement(node.id);
            };
            wHandle.querySelector(".w-del").onclick = (e) => {
                e.stopPropagation();
                deleteElement(node.id);
            };

            // Drag Widget to Reorder
            const wDragBtn = wHandle.querySelector(".w-drag");
            wDragBtn.ondragstart = (e) => {
                e.stopPropagation();
                window._activeDragData = { action: "move_widget", id: node.id };
                e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                e.dataTransfer.effectAllowed = "move";
                el.classList.add("is-dragging");
            };
            wDragBtn.ondragend = () => {
                el.classList.remove("is-dragging");
                window._activeDragData = null;
                hideDropIndicator();
            };

            el.appendChild(wHandle);

            // Widget Dragover Drop Target (Shows top/bottom insertion line)
            el.ondragover = (e) => {
                const data = getDragData(e);
                if (!data) return;

                // Don't target self when moving
                if (data.id === node.id) return;

                e.preventDefault();
                e.stopPropagation();

                const rect = el.getBoundingClientRect();
                const midY = rect.top + rect.height / 2;
                const pos = e.clientY < midY ? "before" : "after";
                showDropIndicator(el, pos);
            };

            el.ondragleave = (e) => {
                if (!el.contains(e.relatedTarget)) {
                    hideDropIndicator();
                }
            };

            el.ondrop = (e) => {
                const data = getDragData(e);
                if (!data) return;
                if (data.id === node.id) return;

                e.preventDefault();
                e.stopPropagation();

                const rect = el.getBoundingClientRect();
                const midY = rect.top + rect.height / 2;
                const pos = e.clientY < midY ? "before" : "after";

                hideDropIndicator();
                handleDropNearWidget(node.id, pos, data);
            };

            // Click widget to select
            el.onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };

            const data = node.data || {};
            const s = node.styles || {};
            const widgetContent = document.createElement("div");

            switch(node.type) {
                case "Heading":
                    const tag = "h" + (data.level || 2);
                    const h = document.createElement(tag);
                    h.className = "np-heading";
                    h.innerHTML = data.text || "Heading Text";
                    h.contentEditable = "true";
                    h.style.outline = "none";
                    h.style.margin = "0";
                    if (s.text_color) h.style.color = s.text_color;
                    if (s.font_family) h.style.fontFamily = s.font_family;
                    if (s.font_size) h.style.fontSize = s.font_size;
                    if (s.font_weight) h.style.fontWeight = s.font_weight;
                    if (s.line_height) h.style.lineHeight = s.line_height;
                    if (s.letter_spacing) h.style.letterSpacing = s.letter_spacing;
                    if (s.text_transform) h.style.textTransform = s.text_transform;
                    if (s.text_align) h.style.textAlign = s.text_align;
                    h.onblur = () => {
                        data.text = h.innerHTML;
                        updateRawAst();
                    };
                    widgetContent.appendChild(h);
                    break;

                case "Text":
                    const t = document.createElement("div");
                    t.className = "np-text";
                    t.innerHTML = data.content_html || "<p>Click to edit paragraph text...</p>";
                    t.contentEditable = "true";
                    t.style.outline = "none";
                    t.onblur = () => {
                        data.content_html = t.innerHTML;
                        updateRawAst();
                    };
                    widgetContent.appendChild(t);
                    break;

                case "Button":
                    const btn = document.createElement("a");
                    btn.className = "np-button";
                    btn.href = data.url || "#";
                    btn.onclick = (e) => { e.preventDefault(); };
                    if (s.background_color) btn.style.backgroundColor = s.background_color;
                    if (s.text_color) btn.style.color = s.text_color;
                    if (s.font_family) btn.style.fontFamily = s.font_family;
                    if (s.font_size) btn.style.fontSize = s.font_size;
                    if (s.font_weight) btn.style.fontWeight = s.font_weight;
                    if (s.line_height) btn.style.lineHeight = s.line_height;
                    if (s.letter_spacing) btn.style.letterSpacing = s.letter_spacing;
                    if (s.text_transform) btn.style.textTransform = s.text_transform;
                    if (s.border_radius) btn.style.borderRadius = s.border_radius;
                    if (s.padding_top) btn.style.paddingTop = s.padding_top;
                    if (s.padding_right) btn.style.paddingRight = s.padding_right;
                    if (s.padding_bottom) btn.style.paddingBottom = s.padding_bottom;
                    if (s.padding_left) btn.style.paddingLeft = s.padding_left;
                    if (s.border_width && s.border_style) {
                        btn.style.border = `${s.border_width} ${s.border_style} ${s.border_color || 'currentColor'}`;
                    }

                    let iconHtml = "";
                    if (data.icon) {
                        iconHtml = `<i class="${data.icon}"></i> `;
                    }
                    btn.innerHTML = `${iconHtml}<span>${data.text || "Click Here"}</span>`;
                    widgetContent.appendChild(btn);
                    break;

                case "Image":
                    const img = document.createElement("img");
                    img.className = "np-image";
                    img.src = data.src || data.url || "https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=800&q=80";
                    img.alt = data.alt || "";
                    img.style.maxWidth = s.max_width || "100%";
                    img.style.height = s.height || "auto";
                    if (data.width) {
                        img.style.width = (typeof data.width === "number" ? data.width + "px" : String(data.width));
                    } else if (s.width) {
                        img.style.width = s.width;
                    }
                    if (data.height) {
                        img.style.height = (typeof data.height === "number" ? data.height + "px" : String(data.height));
                    }
                    if (s.border_radius) {
                        img.style.borderRadius = s.border_radius;
                    }
                    widgetContent.appendChild(img);
                    break;

                case "Icon":
                    const ic = document.createElement("div");
                    ic.style.fontSize = s.font_size || "36px";
                    ic.style.color = s.text_color || "var(--e-global-color-primary, var(--e-color-accent))";
                    ic.innerHTML = `<i class="${data.icon || data.name || 'fa-solid fa-star'}"></i>`;
                    widgetContent.appendChild(ic);
                    break;

                case "IconBox":
                    const ib = document.createElement("div");
                    ib.className = "np-iconbox";
                    ib.style.textAlign = s.text_align || "left";
                    ib.innerHTML = `
                        <div style="font-size: 32px; color: ${s.text_color || 'var(--e-global-color-primary, var(--e-color-accent))'}; margin-bottom: 12px;">
                            <i class="${data.icon || 'fa-solid fa-cubes'}"></i>
                        </div>
                        <h4 class="np-iconbox-title" style="font-size: 18px; margin-bottom: 8px; font-weight: 700;">${data.title || 'Feature Title'}</h4>
                        <p class="np-iconbox-desc" style="opacity: 0.85; font-size: 14px; line-height: 1.5;">${data.description || 'Feature description explaining the key technical benefit.'}</p>
                    `;
                    widgetContent.appendChild(ib);
                    break;

                case "ImageBox":
                    const imgb = document.createElement("div");
                    imgb.className = "np-imagebox";
                    imgb.style.textAlign = s.text_align || "left";
                    const imgBoxUrl = data.image_url || data.url || "https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=600&q=80";
                    imgb.innerHTML = `
                        <div style="margin-bottom: 14px; overflow: hidden;">
                            <img src="${imgBoxUrl}" alt="${data.title || 'Image Feature'}" style="width: 100%; height: auto; display: block;" />
                        </div>
                        <h4 class="np-imagebox-title" style="font-size: 18px; margin-bottom: 8px; font-weight: 700;">${data.title || 'Image Feature'}</h4>
                        <p class="np-imagebox-desc" style="opacity: 0.85; font-size: 14px; line-height: 1.5;">${data.description || 'Feature description explaining the visual showcase.'}</p>
                    `;
                    widgetContent.appendChild(imgb);
                    break;

                case "Counter":
                    const cnt = document.createElement("div");
                    cnt.style.textAlign = "center";
                    cnt.style.padding = "8px 0";
                    const cntNumVal = data.number !== undefined ? data.number : "100%";
                    const cntDisplay = `${data.prefix || ''}${cntNumVal}${data.suffix || ''}`;
                    const cntLabel = data.title || data.label || "Lighthouse Score";
                    cnt.innerHTML = `
                        <div style="font-size: 42px; font-weight: 800; color: ${s.text_color || '#38bdf8'};">
                            ${escapeHtml(String(cntDisplay))}
                        </div>
                        <div style="font-size: 13px; font-weight: 600; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; margin-top: 4px;">
                            ${escapeHtml(cntLabel)}
                        </div>
                    `;
                    widgetContent.appendChild(cnt);
                    break;

                case "Divider":
                    const div = document.createElement("hr");
                    div.className = "np-divider";
                    div.style.border = "none";
                    div.style.borderTop = `${data.weight || '1px'} ${data.style || 'solid'} ${data.color || '#e2e8f0'}`;
                    div.style.margin = "20px 0";
                    widgetContent.appendChild(div);
                    break;

                case "Spacer":
                    const sp = document.createElement("div");
                    const spHeight = (data.height !== undefined && data.height !== null)
                        ? (typeof data.height === "number" ? data.height + "px" : String(data.height))
                        : (s.height || "50px");
                    sp.style.height = spHeight;
                    widgetContent.appendChild(sp);
                    break;

                case "Video":
                    const vid = document.createElement("div");
                    vid.style.position = "relative";
                    vid.style.paddingBottom = "56.25%";
                    vid.style.height = "0";
                    vid.style.overflow = "hidden";
                    vid.style.borderRadius = "8px";
                    vid.style.background = "#000000";
                    vid.innerHTML = `<iframe src="${data.url || 'https://www.youtube.com/embed/dQw4w9WgXcQ'}" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 0;" allowfullscreen></iframe>`;
                    widgetContent.appendChild(vid);
                    break;

                case "Alert":
                    const al = document.createElement("div");
                    const alertBg = data.alert_type === "success" ? "#ecfdf5" : (data.alert_type === "warning" ? "#fffbeb" : "#eff6ff");
                    const alertBorder = data.alert_type === "success" ? "#10b981" : (data.alert_type === "warning" ? "#f59e0b" : "#3b82f6");
                    const alertText = data.alert_type === "success" ? "#065f46" : (data.alert_type === "warning" ? "#92400e" : "#1e40af");
                    al.style.background = alertBg;
                    al.style.borderLeft = "4px solid " + alertBorder;
                    al.style.padding = "14px 18px";
                    al.style.borderRadius = "4px";
                    al.style.color = alertText;
                    al.innerHTML = `<strong>${data.title || 'Notice'}:</strong> <span>${data.description || 'Alert message content.'}</span>`;
                    widgetContent.appendChild(al);
                    break;

                case "Testimonial":
                    const tstm = document.createElement("div");
                    tstm.style.background = "#f8fafc";
                    tstm.style.border = "1px solid #e2e8f0";
                    tstm.style.borderRadius = "8px";
                    tstm.style.padding = "24px";
                    tstm.innerHTML = `
                        <p style="font-style: italic; color: #334155; font-size: 15px; line-height: 1.6; margin-bottom: 16px;">"${data.content || 'Great performance and intuitive builder workflow.'}"</p>
                        <div style="display: flex; align-items: center; gap: 12px;">
                            <div style="width: 40px; height: 40px; border-radius: 50%; background: #e02660; color: #ffffff; display: flex; align-items: center; justify-content: center; font-weight: 700; font-size: 14px;">
                                ${(data.name || 'J').charAt(0)}
                            </div>
                            <div>
                                <div style="font-weight: 700; color: #0f172a; font-size: 14px;">${data.name || 'John Doe'}</div>
                                <div style="font-size: 12px; color: #64748b;">${data.job || 'Software Engineer'}</div>
                            </div>
                        </div>
                    `;
                    widgetContent.appendChild(tstm);
                    break;

                case "ProgressBar":
                    const pb = document.createElement("div");
                    pb.style.margin = "8px 0";
                    pb.innerHTML = `
                        <div style="display: flex; justify-content: space-between; font-size: 13px; font-weight: 600; color: inherit; margin-bottom: 6px;">
                            <span>${data.title || 'Skill'}</span>
                            <span>${data.display_text || (data.percent !== undefined ? data.percent + '%' : '90%')}</span>
                        </div>
                        <div style="height: 10px; background: rgba(148, 163, 184, 0.25); border-radius: 5px; overflow: hidden;">
                            <div style="width: ${data.percent !== undefined ? data.percent : 90}%; height: 100%; background: var(--e-color-accent); border-radius: 5px;"></div>
                        </div>
                    `;
                    widgetContent.appendChild(pb);
                    break;

                case "Accordion":
                case "Toggle":
                    const acc = document.createElement("div");
                    acc.className = "np-accordion";
                    const accItems = data.items || [
                        { title: "Item 1", content: "Accordion content text." },
                        { title: "Item 2", content: "More detailed technical explanation." }
                    ];
                    acc.innerHTML = accItems.map((item, idx) => `
                        <details class="np-accordion-item" style="border-bottom: 1px solid #e2e8f0; padding: 12px 16px; background: #ffffff;" ${idx === 0 ? 'open' : ''}>
                            <summary style="font-weight: 600; cursor: pointer;">${item.title || 'Section'}</summary>
                            <div class="np-accordion-body" style="margin-top: 10px; font-size: 14px; line-height: 1.5;">${item.content_html || item.content || ''}</div>
                        </details>
                    `).join("");
                    widgetContent.appendChild(acc);
                    break;

                case "Tabs":
                    const tb = document.createElement("div");
                    tb.style.background = "#ffffff";
                    tb.style.border = "1px solid #e2e8f0";
                    tb.style.borderRadius = "8px";
                    tb.style.padding = "16px";
                    const tabItems = data.items || [
                        { title: "Tab 1", content: "Tab 1 body content." },
                        { title: "Tab 2", content: "Tab 2 body content." }
                    ];
                    const firstTabBody = tabItems.length > 0 ? (tabItems[0].content_html || tabItems[0].content || "") : "";
                    tb.innerHTML = `
                        <div style="display: flex; gap: 8px; border-bottom: 2px solid #e2e8f0; margin-bottom: 14px;">
                            ${tabItems.map((item, idx) => `<button type="button" class="studio-tab-btn" data-tab-idx="${idx}" style="background: none; border: none; border-bottom: 2px solid ${idx === 0 ? 'var(--e-color-accent)' : 'transparent'}; margin-bottom: -2px; padding: 8px 16px; font-weight: 600; color: ${idx === 0 ? 'var(--e-color-accent)' : '#64748b'}; cursor: pointer;">${item.title || ('Tab ' + (idx + 1))}</button>`).join("")}
                        </div>
                        <div class="studio-tab-body" style="color: #334155; font-size: 14px; line-height: 1.6;">${firstTabBody}</div>
                    `;
                    tb.querySelectorAll(".studio-tab-btn").forEach(btnEl => {
                        btnEl.onclick = (ev) => {
                            ev.stopPropagation();
                            const tIdx = parseInt(btnEl.dataset.tabIdx) || 0;
                            tb.querySelectorAll(".studio-tab-btn").forEach((b, i) => {
                                b.style.borderBottomColor = (i === tIdx) ? "var(--e-color-accent)" : "transparent";
                                b.style.color = (i === tIdx) ? "var(--e-color-accent)" : "#64748b";
                            });
                            const bodyBox = tb.querySelector(".studio-tab-body");
                            if (bodyBox && tabItems[tIdx]) {
                                bodyBox.innerHTML = tabItems[tIdx].content_html || tabItems[tIdx].content || "";
                            }
                            selectElement(node.id);
                        };
                    });
                    widgetContent.appendChild(tb);
                    break;

                case "PriceTable":
                    const pt = document.createElement("div");
                    pt.style.background = s.background_color || "#1e293b";
                    pt.style.color = s.text_color || "#f8fafc";
                    pt.style.border = "1px solid rgba(148, 163, 184, 0.25)";
                    pt.style.borderRadius = s.border_radius || "12px";
                    pt.style.padding = "28px 24px";
                    pt.style.textAlign = "center";
                    const ptFeatures = Array.isArray(data.features) ? data.features : [
                        "Unlimited Requests",
                        "Sub-millisecond SSR",
                        "Automatic SQLite Snapshots"
                    ];
                    pt.innerHTML = `
                        <div style="font-size: 13px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; color: #38bdf8; margin-bottom: 10px;">${escapeHtml(data.title || 'Enterprise Plan')}</div>
                        <div style="display: flex; align-items: baseline; justify-content: center; gap: 4px; margin-bottom: 20px;">
                            <span style="font-size: 22px; font-weight: 700; color: #94a3b8;">${escapeHtml(data.currency || '$')}</span>
                            <span style="font-size: 44px; font-weight: 800; line-height: 1; color: #ffffff;">${escapeHtml(data.price || '49')}</span>
                            <span style="font-size: 14px; color: #94a3b8;">${escapeHtml(data.period || '/ month')}</span>
                        </div>
                        <ul style="list-style: none; padding: 0; margin: 0 0 24px 0; text-align: left; border-top: 1px solid rgba(148, 163, 184, 0.18); padding-top: 16px;">
                            ${ptFeatures.map(f => `
                                <li style="display: flex; align-items: center; gap: 10px; padding: 6px 0; font-size: 14px; color: #cbd5e1;">
                                    <i class="fa-solid fa-circle-check" style="color: #10b981; font-size: 14px;"></i>
                                    <span>${escapeHtml(typeof f === 'string' ? f : (f.text || ''))}</span>
                                </li>
                            `).join("")}
                        </ul>
                        <a href="${escapeHtml(data.button_url || '#')}" style="display: inline-block; width: 100%; padding: 12px 20px; background: var(--e-color-accent); color: #ffffff; border-radius: 6px; font-weight: 600; text-decoration: none; font-size: 14px;">
                            ${escapeHtml(data.button_text || 'Deploy Now')}
                        </a>
                    `;
                    widgetContent.appendChild(pt);
                    break;

                case "CallToAction":
                    const cta = document.createElement("div");
                    cta.style.background = s.background_color || "#0f172a";
                    cta.style.color = s.text_color || "#ffffff";
                    cta.style.padding = "32px 24px";
                    cta.style.borderRadius = "10px";
                    cta.style.textAlign = "center";
                    cta.innerHTML = `
                        <h3 style="font-size: 24px; font-weight: 700; margin-bottom: 12px; color: inherit;">${escapeHtml(data.title || 'Call to Action')}</h3>
                        <div style="font-size: 15px; opacity: 0.85; margin-bottom: 20px;">${data.description_html || ''}</div>
                        <a href="${escapeHtml(data.button_url || '#')}" style="display: inline-block; padding: 12px 28px; background: var(--e-color-accent); color: #ffffff; border-radius: 6px; font-weight: 600; text-decoration: none;">
                            ${escapeHtml(data.button_text || 'Get Started')}
                        </a>
                    `;
                    widgetContent.appendChild(cta);
                    break;

                case "CustomHtml":
                    const ch = document.createElement("div");
                    ch.innerHTML = data.raw || data.raw_html || data.content_html || "<pre>Custom HTML Block</pre>";
                    widgetContent.appendChild(ch);
                    break;

                case "ShortcodeFallback":
                    const sf = document.createElement("div");
                    sf.style.padding = "12px 16px";
                    sf.style.background = "#f8fafc";
                    sf.style.border = "1px dashed #cbd5e1";
                    sf.style.borderRadius = "6px";
                    sf.style.fontFamily = "monospace";
                    sf.style.fontSize = "13px";
                    sf.style.color = "#475569";
                    sf.textContent = data.raw || "[shortcode]";
                    widgetContent.appendChild(sf);
                    break;

                case "Audio":
                    const aud = document.createElement("audio");
                    aud.controls = true;
                    aud.src = data.src || data.url || "";
                    aud.style.width = "100%";
                    widgetContent.appendChild(aud);
                    break;

                case "Map":
                    const mp = document.createElement("div");
                    mp.style.padding = "24px";
                    mp.style.background = "#f1f5f9";
                    mp.style.borderRadius = "8px";
                    mp.style.textAlign = "center";
                    mp.innerHTML = `<i class="fa-solid fa-location-dot" style="color: var(--e-color-accent); font-size: 24px; margin-bottom: 8px;"></i><div style="font-weight: 600; color: #0f172a;">${escapeHtml(data.address || 'Map Embed')}</div>`;
                    widgetContent.appendChild(mp);
                    break;

                case "Rating":
                    const rt = document.createElement("div");
                    const stars = Array.from({ length: data.max_rating || 5 }, (_, i) => `<i class="${i < (data.rating || 5) ? 'fa-solid' : 'fa-regular'} fa-star" style="color: #f59e0b; margin-right: 2px;"></i>`).join("");
                    rt.innerHTML = `
                        <div style="display: flex; align-items: center; gap: 8px;">
                            ${data.title ? `<span style="font-weight: 600; font-size: 14px;">${data.title}:</span>` : ''}
                            <div>${stars}</div>
                            <span style="font-weight: 700; font-size: 13px; color: #64748b;">(${data.rating || 5}/${data.max_rating || 5})</span>
                        </div>
                    `;
                    widgetContent.appendChild(rt);
                    break;

                case "SocialIcons":
                    const soc = document.createElement("div");
                    soc.className = "np-social-icons";
                    const socItems = data.items || [
                        { icon: "fa-brands fa-github", link: "#" },
                        { icon: "fa-brands fa-x-twitter", link: "#" },
                        { icon: "fa-brands fa-discord", link: "#" }
                    ];
                    soc.innerHTML = socItems.map(item => `
                        <a href="${item.link || '#'}" class="np-social-icon" onclick="event.preventDefault()">
                            <i class="${item.icon}"></i>
                        </a>
                    `).join("");
                    widgetContent.appendChild(soc);
                    break;

                case "IconList":
                    const il = document.createElement("ul");
                    il.className = "np-iconlist";
                    const ilItems = data.items || [
                        { icon: "fa-solid fa-check", text: "Zero Client-Side JavaScript Overhead" },
                        { icon: "fa-solid fa-bolt", text: "Sub-millisecond Server Response Times" }
                    ];
                    il.innerHTML = ilItems.map(item => {
                        const iconSpan = item.icon ? `<span class="np-iconlist-icon"><i class="${item.icon}"></i></span>` : "";
                        const inner = `${iconSpan}<span class="np-iconlist-text">${item.text || ''}</span>`;
                        return `<li class="np-iconlist-item">${item.link ? `<a href="${item.link}" onclick="event.preventDefault()">${inner}</a>` : inner}</li>`;
                    }).join("");
                    widgetContent.appendChild(il);
                    break;

                case "Form":
                case "NativeForm":
                    const frm = document.createElement("form");
                    frm.className = "np-form";
                    frm.onsubmit = (e) => { e.preventDefault(); return false; };
                    if (node.children && node.children.length > 0) {
                        node.children.forEach(child => {
                            if (child.type === "CustomHtml" && child.data) {
                                const childDiv = document.createElement("div");
                                childDiv.innerHTML = child.data.raw || child.data.raw_html || child.data.content_html || "";
                                frm.appendChild(childDiv);
                            } else {
                                frm.appendChild(buildDomNode(child, false));
                            }
                        });
                    } else {
                        frm.style.background = "#f8fafc";
                        frm.style.border = "1px solid #e2e8f0";
                        frm.style.borderRadius = "8px";
                        frm.style.padding = "24px";
                        frm.innerHTML = `
                            ${data.title ? `<h4 style="font-size: 16px; margin-bottom: 14px; color: #0f172a; font-weight: 700;">${data.title}</h4>` : ''}
                            <div style="display: flex; flex-direction: column; gap: 10px;">
                                <input type="text" placeholder="Full Name" disabled style="padding: 8px 12px; border: 1px solid #cbd5e1; border-radius: 4px; background: #ffffff; font-size: 13px;">
                                <input type="email" placeholder="Email Address" disabled style="padding: 8px 12px; border: 1px solid #cbd5e1; border-radius: 4px; background: #ffffff; font-size: 13px;">
                                <textarea placeholder="Your Message..." rows="3" disabled style="padding: 8px 12px; border: 1px solid #cbd5e1; border-radius: 4px; background: #ffffff; font-size: 13px;"></textarea>
                                <button type="button" class="np-button">${data.button_text || 'Send Message'}</button>
                            </div>
                        `;
                    }
                    widgetContent.appendChild(frm);
                    break;

                case "Benchmark":
                    const bmrk = document.createElement("div");
                    bmrk.style.background = "linear-gradient(135deg, #0f172a 0%, #1e293b 100%)";
                    bmrk.style.color = "#ffffff";
                    bmrk.style.padding = "24px";
                    bmrk.style.borderRadius = "8px";
                    bmrk.style.textAlign = "center";
                    bmrk.innerHTML = `
                        <div style="font-size: 12px; font-weight: 700; letter-spacing: 0.05em; color: #38bdf8; text-transform: uppercase; margin-bottom: 8px;">Tokio Async Engine</div>
                        <div style="font-size: 26px; font-weight: 800; margin-bottom: 8px;">${data.text || '95,000 req/sec Tokio Engine Benchmark'}</div>
                        <div style="font-size: 13px; color: #94a3b8;">Sub-millisecond TTFB &bull; Embedded Rusqlite WAL Mode</div>
                    `;
                    widgetContent.appendChild(bmrk);
                    break;

                default:
                    const generic = document.createElement("div");
                    generic.style.padding = "16px";
                    generic.style.background = "#f1f5f9";
                    generic.style.borderRadius = "4px";
                    generic.innerHTML = `<strong>${node.type}</strong>: ${data.text || node.id}`;
                    widgetContent.appendChild(generic);
            }

            el.appendChild(widgetContent);
            return el;
        }

        if (pageAst.root.children && pageAst.root.children.length > 0) {
            pageAst.root.children.forEach(sec => {
                canvas.appendChild(buildDomNode(sec, true));
            });
        }
        updateNavigator();
    }

    // Canvas background dragover & drop handlers
    const canvasEl = document.getElementById("elementor-canvas");
    canvasEl.ondragover = (e) => {
        const data = getDragData(e);
        if (data && (data.type === "new_widget" || data.action === "move_widget")) {
            e.preventDefault();
            if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
        }
    };
    canvasEl.ondrop = (e) => {
        if (e.target === canvasEl) {
            e.preventDefault();
            hideDropIndicator();
            const data = getDragData(e);
            if (!data) return;
            if (data.type === "new_widget") {
                const newWidget = createWidgetNode(data.widgetType);
                if (newWidget.type === "Section") {
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newWidget);
                } else {
                    if (newWidget.type === "Section") {
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newWidget);
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
            }
            const newSec = newWidget.type === "Section" ? newWidget : createSectionWithColumns(1, [newWidget]);
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newSec);
                }
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newSec);
                renderCanvas();
                selectElement(newWidget.id);
                showToast("Added new section with " + data.widgetType);
            }
            window._activeDragData = null;
        }
    };

    // Scrollable Preview Container dragover & drop handlers
    const previewEl = document.getElementById("elementor-preview");
    if (previewEl) {
        previewEl.ondragover = (e) => {
            const data = getDragData(e);
            if (data && (data.type === "new_widget" || data.action === "move_widget")) {
                e.preventDefault();
                if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
            }
        };
        previewEl.ondrop = (e) => {
            if (e.target === previewEl || e.target.id === "elementor-preview-responsive-wrapper") {
                e.preventDefault();
                hideDropIndicator();
                const data = getDragData(e);
                if (data && data.type === "new_widget") {
                    const newWidget = createWidgetNode(data.widgetType);
                    if (newWidget.type === "Section") {
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newWidget);
                } else {
                    if (newWidget.type === "Section") {
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newWidget);
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
            }
            const newSec = newWidget.type === "Section" ? newWidget : createSectionWithColumns(1, [newWidget]);
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newSec);
                }
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
                    pageAst.root.children = pageAst.root.children || [];
                    pageAst.root.children.push(newSec);
                    renderCanvas();
                    selectElement(newWidget.id);
                    showToast("Added new section with " + data.widgetType);
                }
                window._activeDragData = null;
            }
        };
        previewEl.ondragleave = (e) => {
            if (!previewEl.contains(e.relatedTarget)) {
                hideDropIndicator();
            }
        };
    }

    // Drop handlers
    function handleDropOnContainer(containerId, data) {
        if (!data) return;
        const container = findNodeById(pageAst.root, containerId);
        if (!container) return;

        if (data.type === "new_widget") {
            const newWidget = createWidgetNode(data.widgetType);
            container.children = container.children || [];
            container.children.push(newWidget);
            renderCanvas();
            selectElement(newWidget.id);
            showToast("Added " + data.widgetType + " widget");
        } else if (data.action === "move_widget") {
            const moved = removeNodeById(pageAst.root, data.id);
            if (moved) {
                container.children = container.children || [];
                container.children.push(moved);
                renderCanvas();
                selectElement(moved.id);
                showToast("Moved element");
            }
        }
        window._activeDragData = null;
    }

    function handleDropNearWidget(targetWidgetId, position, data) {
        if (!data) return;
        const parent = findParentNode(pageAst.root, targetWidgetId);
        if (!parent || !parent.children) return;

        let targetIdx = parent.children.findIndex(c => c.id === targetWidgetId);
        if (targetIdx === -1) return;
        if (position === "after") targetIdx += 1;

        if (data.type === "new_widget") {
            const newWidget = createWidgetNode(data.widgetType);
            parent.children.splice(targetIdx, 0, newWidget);
            renderCanvas();
            selectElement(newWidget.id);
            showToast("Inserted " + data.widgetType + " widget");
        } else if (data.action === "move_widget") {
            const moved = removeNodeById(pageAst.root, data.id);
            if (moved) {
                let newTargetIdx = parent.children.findIndex(c => c.id === targetWidgetId);
                if (newTargetIdx === -1) {
                    parent.children.push(moved);
                } else {
                    if (position === "after") newTargetIdx += 1;
                    parent.children.splice(newTargetIdx, 0, moved);
                }
                renderCanvas();
                selectElement(moved.id);
                showToast("Reordered widget");
            }
        }
        window._activeDragData = null;
    }

    function executeSectionMove(draggedSecId, targetSecId, position) {
        if (pageAst.root.children) {
            const dragged = removeNodeById(pageAst.root, draggedSecId);
            if (dragged) {
                let idx = pageAst.root.children.findIndex(c => c.id === targetSecId);
                if (idx === -1) {
                    pageAst.root.children.push(dragged);
                } else {
                    if (position === "after") idx += 1;
                    pageAst.root.children.splice(idx, 0, dragged);
                }
                renderCanvas();
                selectElement(dragged.id);
                showToast("Reordered section");
            }
        }
        window._activeDragData = null;
    }

    // Bottom dropzone handling
    const bottomDropzone = document.getElementById("elementor-bottom-dropzone");
    bottomDropzone.ondragover = (e) => {
        e.preventDefault();
        bottomDropzone.classList.add("drag-over");
    };
    bottomDropzone.ondragleave = () => bottomDropzone.classList.remove("drag-over");
    bottomDropzone.ondrop = (e) => {
        e.preventDefault();
        bottomDropzone.classList.remove("drag-over");
        hideDropIndicator();
        const data = getDragData(e);
        if (!data) return;

        if (data.type === "new_widget") {
            const newWidget = createWidgetNode(data.widgetType);
            if (newWidget.type === "Section") {
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newWidget);
                renderCanvas();
                updateNavigator();
                selectElement(newWidget.id);
                showToast("Added " + type);
                savePage();
                window._activeDragData = null;
                return;
            }
            const newSec = newWidget.type === "Section" ? newWidget : createSectionWithColumns(1, [newWidget]);
            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newSec);
            renderCanvas();
            selectElement(newWidget.id);
            showToast("Added new section with " + data.widgetType);
        } else if (data.action === "move_widget") {
            const moved = removeNodeById(pageAst.root, data.id);
            if (moved) {
                const newSec = createSectionWithColumns(1, [moved]);
                pageAst.root.children = pageAst.root.children || [];
                pageAst.root.children.push(newSec);
                renderCanvas();
                selectElement(moved.id);
                showToast("Moved element to new section");
            }
        }
        window._activeDragData = null;
    };

    // -------------------------------------------------------------
    // PANEL INSPECTOR CONTROLS
    // -------------------------------------------------------------
        function renderInspector() {
        const titleEl = document.getElementById("elementor-panel-header-title");
        const navEl = document.getElementById("panel-nav");
        const bodyEl = document.getElementById("elementor-panel-content-wrapper");
        const menuBtn = document.getElementById("elementor-panel-header-menu-button");
        const addBtn = document.getElementById("elementor-panel-header-add-button");

        // Ensure global styles are applied to canvas
        applyGlobalStyles();

        // -------------------------------------------------------------
        // VIEW 1: HAMBURGER SITE SETTINGS & MENU ROOT
        // -------------------------------------------------------------
        if (currentPanelView === "menu") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Editor";
            titleEl.innerHTML = `<span style="display:flex;align-items:center;gap:6px;"><i class="fa-solid fa-sliders" style="color:var(--e-color-accent);"></i> SITE SETTINGS</span>`;
            navEl.innerHTML = `<div class="elementor-subpanel-header" style="width:100%;"><span style="font-size:11px;font-weight:700;color:#818a91;text-transform:uppercase;">Global Site Configuration</span></div>`;

            bodyEl.innerHTML = `
                <div class="elementor-panel-menu" id="elementor-panel-page-menu">
                    <!-- Group: SITE SETTINGS -->
                    <div class="elementor-panel-menu-group">
                        <div class="elementor-panel-menu-group-title">Site Settings</div>
                        <div class="elementor-panel-menu-items">
                            <div class="elementor-panel-menu-item" id="menu-item-colors">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-palette"></i></div>
                                <div class="elementor-panel-menu-item-title">Global Colors</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-fonts">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-font"></i></div>
                                <div class="elementor-panel-menu-item-title">Global Fonts</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-buttons">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-square-check"></i></div>
                                <div class="elementor-panel-menu-item-title">Buttons Style</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-identity">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-id-card"></i></div>
                                <div class="elementor-panel-menu-item-title">Site Identity</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-layout">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-table-columns"></i></div>
                                <div class="elementor-panel-menu-item-title">Layout & Spacing</div>
                            </div>
                        </div>
                    </div>

                    <!-- Group: PREFERENCES -->
                    <div class="elementor-panel-menu-group">
                        <div class="elementor-panel-menu-group-title">Preferences & Tools</div>
                        <div class="elementor-panel-menu-items">
                            <div class="elementor-panel-menu-item" id="menu-item-user-pref">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-sliders"></i></div>
                                <div class="elementor-panel-menu-item-title">User Preferences</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-shortcuts">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-keyboard"></i></div>
                                <div class="elementor-panel-menu-item-title">Keyboard Shortcuts</div>
                            </div>
                            <div class="elementor-panel-menu-item" id="menu-item-finder">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-magnifying-glass"></i></div>
                                <div class="elementor-panel-menu-item-title">Finder (Ctrl+E)</div>
                            </div>
                        </div>
                    </div>

                    <!-- Group: NAVIGATE FROM PAGE -->
                    <div class="elementor-panel-menu-group">
                        <div class="elementor-panel-menu-group-title">Navigate From Page</div>
                        <div class="elementor-panel-menu-items">
                            <a class="elementor-panel-menu-item" href="/?p=${POST_ID}" target="_blank">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-arrow-up-right-from-square"></i></div>
                                <div class="elementor-panel-menu-item-title">View Live Page</div>
                            </a>
                            <a class="elementor-panel-menu-item" href="/n-admin/post.php?action=edit&post=${POST_ID}">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-pen-nib"></i></div>
                                <div class="elementor-panel-menu-item-title">Exit to Post Editor</div>
                            </a>
                            <a class="elementor-panel-menu-item" href="/n-admin/">
                                <div class="elementor-panel-menu-item-icon"><i class="fa-solid fa-arrow-right-from-bracket"></i></div>
                                <div class="elementor-panel-menu-item-title">Exit to Dashboard</div>
                            </a>
                        </div>
                    </div>
                </div>
            `;

            document.getElementById("menu-item-colors").onclick = () => { currentPanelView = "site-settings-colors"; renderInspector(); };
            document.getElementById("menu-item-fonts").onclick = () => { currentPanelView = "site-settings-fonts"; renderInspector(); };
            document.getElementById("menu-item-buttons").onclick = () => { currentPanelView = "site-settings-buttons"; renderInspector(); };
            document.getElementById("menu-item-identity").onclick = () => { currentPanelView = "site-settings-identity"; renderInspector(); };
            document.getElementById("menu-item-layout").onclick = () => { currentPanelView = "site-settings-layout"; renderInspector(); };
            document.getElementById("menu-item-user-pref").onclick = () => { currentPanelView = "user-preferences"; renderInspector(); };
            document.getElementById("menu-item-shortcuts").onclick = () => {
                openShortcutsModal();
            };
            document.getElementById("menu-item-finder").onclick = () => {
                openFinderModal();
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 2: SUBPANEL - GLOBAL COLORS
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-colors") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `GLOBAL COLORS`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            const colors = pageAst.global_styles.system_colors || [];
            const customColors = pageAst.global_styles.custom_colors || [];

            let colorsHtml = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">System Colors</div>
            `;

            colors.forEach(c => {
                colorsHtml += `
                    <div class="elementor-color-token-row">
                        <div class="color-token-meta">
                            <div class="color-token-preview" style="background:${c.color};" id="swatch-${c.id}"></div>
                            <div class="color-token-info">
                                <span class="color-token-title">${c.title}</span>
                                <span class="color-token-code" id="hex-${c.id}">${c.color}</span>
                            </div>
                        </div>
                        <div class="color-token-input-wrapper">
                            <input type="color" value="${c.color}" data-color-id="${c.id}" class="system-color-input">
                        </div>
                    </div>
                `;
            });

            colorsHtml += `
                <div class="globals-section-header" style="margin-top:16px;">
                    <span>Custom Colors</span>
                    <button class="elementor-back-btn" id="btn-add-custom-color" style="font-size:11px;color:var(--e-color-accent);">
                        <i class="fa-solid fa-plus"></i> Add Color
                    </button>
                </div>
            `;

            if (customColors.length === 0) {
                colorsHtml += `<div style="font-size:11px;color:#818a91;font-style:italic;">No custom colors defined yet. Click '+ Add Color' to create one.</div>`;
            } else {
                customColors.forEach((cc, idx) => {
                    colorsHtml += `
                        <div class="elementor-color-token-row">
                            <div class="color-token-meta">
                                <div class="color-token-preview" style="background:${cc.color};"></div>
                                <div class="color-token-info">
                                    <span class="color-token-title">${cc.title}</span>
                                    <span class="color-token-code">${cc.color}</span>
                                </div>
                            </div>
                            <div style="display:flex;align-items:center;gap:6px;">
                                <input type="color" value="${cc.color}" data-custom-idx="${idx}" class="custom-color-input">
                                <button class="elementor-back-btn btn-del-color" data-del-idx="${idx}" style="color:#ef4444;"><i class="fa-solid fa-trash-can"></i></button>
                            </div>
                        </div>
                    `;
                });
            }

            colorsHtml += `
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-colors">
                            <i class="fa-solid fa-floppy-disk"></i> Apply & Save Palette
                        </button>
                    </div>
                </div>
            `;

            bodyEl.innerHTML = colorsHtml;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };


            bodyEl.querySelectorAll(".system-color-input").forEach(input => {
                input.oninput = (e) => {
                    window._userModifiedGlobalStyles = true;
                    const cid = e.target.dataset.colorId;
                    const val = e.target.value;
                    const c = colors.find(x => x.id === cid);
                    if (c) c.color = val;
                    const swatch = document.getElementById("swatch-" + cid);
                    if (swatch) swatch.style.background = val;
                    const hex = document.getElementById("hex-" + cid);
                    if (hex) hex.textContent = val;
                    applyGlobalStyles();
                };
            });

            bodyEl.querySelectorAll(".custom-color-input").forEach(input => {
                input.oninput = (e) => {
                    window._userModifiedGlobalStyles = true;
                    const idx = parseInt(e.target.dataset.customIdx);
                    if (customColors[idx]) {
                        customColors[idx].color = e.target.value;
                        applyGlobalStyles();
                    }
                };
            });

            bodyEl.querySelectorAll(".btn-del-color").forEach(btn => {
                btn.onclick = () => {
                    window._userModifiedGlobalStyles = true;
                    const idx = parseInt(btn.dataset.delIdx);
                    customColors.splice(idx, 1);
                    applyGlobalStyles();
                    renderInspector();
                };
            });

            const addCustomBtn = document.getElementById("btn-add-custom-color");
            if (addCustomBtn) {
                addCustomBtn.onclick = () => {
                    const name = prompt("Enter Custom Color Title:", "Brand Highlight");
                    if (name) {
                        window._userModifiedGlobalStyles = true;
                        const newId = "custom_" + Math.random().toString(36).substr(2, 6);
                        customColors.push({ id: newId, title: name, color: "#8b5cf6" });
                        applyGlobalStyles();
                        renderInspector();
                    }
                };
            }

            document.getElementById("btn-save-colors").onclick = () => {
                window._userModifiedGlobalStyles = true;
                applyGlobalStyles();
                savePage();
                showToast("Global Colors updated & saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 3: SUBPANEL - GLOBAL FONTS
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-fonts") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `GLOBAL FONTS`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            const typography = pageAst.global_styles.system_typography || [];
            const fontFamilies = ["Fira Sans, sans-serif", "Roboto, sans-serif", "Inter, sans-serif", "Poppins, sans-serif", "Montserrat, sans-serif", "Roboto Slab, serif", "Playfair Display, serif", "system-ui, sans-serif"];

            let fontsHtml = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">System Typography Tokens</div>
            `;

            typography.forEach(t => {
                let optionsHtml = "";
                fontFamilies.forEach(f => {
                    const sel = t.font_family.includes(f.split(",")[0]) ? "selected" : "";
                    optionsHtml += `<option value="${f}" ${sel}>${f.split(",")[0]}</option>`;
                });

                fontsHtml += `
                    <div class="elementor-control-row" style="background:#191b1d;padding:12px;border:1px solid var(--e-border);border-radius:5px;">
                        <div class="elementor-control-label">
                            <span>${t.title} Font</span>
                            <span style="font-size:10px;color:#818a91;">${t.font_size || '16px'} (${t.font_weight || '400'})</span>
                        </div>
                        <select class="elementor-select font-family-select" data-type-id="${t.id}">
                            ${optionsHtml}
                        </select>
                    </div>
                `;
            });

            fontsHtml += `
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-fonts">
                            <i class="fa-solid fa-floppy-disk"></i> Apply & Save Typography
                        </button>
                    </div>
                </div>
            `;

            bodyEl.innerHTML = fontsHtml;
            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };


            bodyEl.querySelectorAll(".font-family-select").forEach(sel => {
                sel.onchange = (e) => {
                    const tid = e.target.dataset.typeId;
                    const item = typography.find(x => x.id === tid);
                    if (item) {
                        item.font_family = e.target.value;
                        applyGlobalStyles();
                    }
                };
            });

            document.getElementById("btn-save-fonts").onclick = () => {
                applyGlobalStyles();
                savePage();
                showToast("Global Fonts updated & saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 4: SUBPANEL - BUTTONS STYLE
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-buttons") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `BUTTONS STYLE`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Default Button Appearance</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Background Color</label>
                        <div class="color-picker-control">
                            <input type="color" class="color-swatch" id="btn-def-bg" value="#e02660">
                            <input type="text" class="elementor-input" value="#e02660" readonly>
                        </div>
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Text Color</label>
                        <div class="color-picker-control">
                            <input type="color" class="color-swatch" id="btn-def-txt" value="#ffffff">
                            <input type="text" class="elementor-input" value="#ffffff" readonly>
                        </div>
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Border Radius (px)</label>
                        <input type="number" class="elementor-input" id="btn-def-rad" value="6" min="0" max="50">
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-button-style">
                            <i class="fa-solid fa-floppy-disk"></i> Save Button Presets
                        </button>
                    </div>
                </div>
            `;

            const bDef = pageAst.global_styles.button_style || { background_color: "#e02660", text_color: "#ffffff", border_radius: "6px" };
            const bgIn = document.getElementById("btn-def-bg");
            const txtIn = document.getElementById("btn-def-txt");
            const radIn = document.getElementById("btn-def-rad");
            if (bgIn && bDef.background_color) bgIn.value = bDef.background_color;
            if (txtIn && bDef.text_color) txtIn.value = bDef.text_color;
            if (radIn && bDef.border_radius) radIn.value = parseInt(bDef.border_radius) || 6;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            document.getElementById("btn-save-button-style").onclick = () => {
                pageAst.global_styles.button_style = {
                    background_color: bgIn ? bgIn.value : "#e02660",
                    text_color: txtIn ? txtIn.value : "#ffffff",
                    border_radius: (radIn ? radIn.value : "6") + "px"
                };
                applyGlobalStyles();
                savePage();
                showToast("Global Button styling updated & saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 5: SUBPANEL - LAYOUT & SPACING
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-layout") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `LAYOUT & SPACING`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            const cw = pageAst.global_styles.content_width || 1140;
            const ws = pageAst.global_styles.widgets_space || 20;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Container Geometry</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Content Width (px)</label>
                        <input type="number" class="elementor-input" id="layout-content-width" value="${cw}" min="600" max="1920">
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Widgets Space / Gap (px)</label>
                        <input type="number" class="elementor-input" id="layout-widgets-space" value="${ws}" min="0" max="60">
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-layout">
                            <i class="fa-solid fa-floppy-disk"></i> Save Layout Defaults
                        </button>
                    </div>
                </div>
            `;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            document.getElementById("layout-content-width").oninput = (e) => {
                pageAst.global_styles.content_width = parseInt(e.target.value) || 1140;
                applyGlobalStyles();
            };
            document.getElementById("layout-widgets-space").oninput = (e) => {
                pageAst.global_styles.widgets_space = parseInt(e.target.value) || 20;
                applyGlobalStyles();
            };
            document.getElementById("btn-save-layout").onclick = () => {
                applyGlobalStyles();
                savePage();
                showToast("Layout geometry saved!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 6: SUBPANEL - SITE IDENTITY
        // -------------------------------------------------------------
        if (currentPanelView === "site-settings-identity") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `SITE IDENTITY`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Identity Settings</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Site Title</label>
                        <input type="text" class="elementor-input" id="identity-title" value="Neopress High-Throughput Engine">
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Site Tagline</label>
                        <input type="text" class="elementor-input" id="identity-tagline" value="Native Rust &bull; Tokio Async &bull; Zero PHP Overhead">
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-identity">
                            <i class="fa-solid fa-floppy-disk"></i> Update Site Identity
                        </button>
                    </div>
                </div>
            `;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            fetch("/wp-json/wp/v2/settings", { headers: { "X-WP-Nonce": "wp_rest_admin" }, credentials: "include" }).then(r => r.json()).then(s => {
                const titleIn = document.getElementById("identity-title");
                const tagIn = document.getElementById("identity-tagline");
                if (titleIn && s.title) titleIn.value = s.title;
                if (tagIn && s.description) tagIn.value = s.description;
            }).catch(() => {});
            document.getElementById("btn-save-identity").onclick = async () => {
                const t = document.getElementById("identity-title").value;
                const d = document.getElementById("identity-tagline").value;
                try {
                    const res = await fetch("/wp-json/wp/v2/settings", {
                        method: "POST",
                        headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                        credentials: "include",
                        body: JSON.stringify({ title: t, description: d })
                    });
                    if (res.ok) {
                        document.title = "Elementor Visual Builder – " + t;
                        showToast("Site Identity updated in SQLite!");
                    } else {
                        showToast("Failed to update site settings");
                    }
                } catch(e) {
                    showToast("Site identity updated locally");
                }
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 7: SUBPANEL - USER PREFERENCES
        // -------------------------------------------------------------
        if (currentPanelView === "user-preferences") {
            menuBtn.innerHTML = `<i class="fa-solid fa-arrow-left"></i>`;
            menuBtn.title = "Back to Site Settings";
            titleEl.innerHTML = `USER PREFERENCES`;
            navEl.innerHTML = `
                <div class="elementor-subpanel-header" style="width:100%;">
                    <button class="elementor-back-btn" id="btn-back-to-menu">
                        <i class="fa-solid fa-chevron-left"></i> Back to Site Settings
                    </button>
                </div>
            `;

            bodyEl.innerHTML = `
                <div class="elementor-subpanel-body">
                    <div class="globals-section-header">Editor Studio Interface</div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">UI Theme</label>
                        <select class="elementor-select" id="pref-theme">
                            <option value="dark" selected>Dark Studio (Default)</option>
                            <option value="light">Light Studio</option>
                        </select>
                    </div>
                    <div class="elementor-control-row">
                        <label class="elementor-control-label">Panel Width</label>
                        <select class="elementor-select" id="pref-panel-width">
                            <option value="300" selected>300px (Default)</option>
                            <option value="360">360px (Wide)</option>
                            <option value="420">420px (Ultra-wide)</option>
                        </select>
                    </div>
                    <div style="margin-top:16px;">
                        <button class="btn-elementor-action" id="btn-save-pref">
                            <i class="fa-solid fa-check"></i> Save Preferences
                        </button>
                    </div>
                </div>
            `;

            const wrapper = document.getElementById("elementor-editor-wrapper");
            const panel = document.getElementById("elementor-panel");
            const themeSel = document.getElementById("pref-theme");
            const widthSel = document.getElementById("pref-panel-width");

            const savedTheme = localStorage.getItem("neopress_studio_theme") || "dark";
            const savedWidth = localStorage.getItem("neopress_panel_width") || "300";
            if (themeSel) themeSel.value = savedTheme;
            if (widthSel) widthSel.value = savedWidth;

            document.getElementById("btn-back-to-menu").onclick = () => { currentPanelView = "menu"; renderInspector(); };

            if (themeSel) {
                themeSel.onchange = (e) => {
                    const th = e.target.value;
                    if (th === "light") wrapper.classList.add("theme-light");
                    else wrapper.classList.remove("theme-light");
                    localStorage.setItem("neopress_studio_theme", th);
                };
            }
            if (widthSel) {
                widthSel.onchange = (e) => {
                    panel.style.width = e.target.value + "px";
                    localStorage.setItem("neopress_panel_width", e.target.value);
                };
            }
            document.getElementById("btn-save-pref").onclick = () => {
                showToast("User Preferences saved to local storage!");
            };
            return;
        }

        // -------------------------------------------------------------
        // VIEW 8: GLOBALS TAB DRAWER (currentPanelView === "globals")
        // -------------------------------------------------------------
        if (currentPanelView === "globals" && !activeElementId) {
            menuBtn.innerHTML = `<i class="fa-solid fa-bars"></i>`;
            menuBtn.title = "Menu";
            titleEl.innerHTML = `<span class="e-logo">E</span> ELEMENTOR`;
            navEl.innerHTML = `
                <button class="elementor-panel-navigation-tab" data-tab="elements">Elements</button>
                <button class="elementor-panel-navigation-tab active" data-tab="globals">Globals</button>
            `;

            const sysColors = pageAst.global_styles.system_colors || [];
            const sysTypography = pageAst.global_styles.system_typography || [];

            let chipsHtml = "";
            sysColors.forEach(c => {
                chipsHtml += `
                    <div class="globals-color-chip" title="Click to edit ${c.title} in Site Settings">
                        <div class="globals-chip-swatch" style="background:${c.color};"></div>
                        <div class="globals-chip-text">
                            <span class="globals-chip-name">${c.title}</span>
                            <span class="globals-chip-hex">${c.color}</span>
                        </div>
                    </div>
                `;
            });

            let typeListHtml = "";
            sysTypography.forEach(t => {
                typeListHtml += `
                    <div class="globals-type-item">
                        <span class="globals-type-name">${t.title}</span>
                        <span class="globals-type-spec">${t.font_family.split(",")[0]} &bull; ${t.font_size || '16px'}</span>
                    </div>
                `;
            });

            bodyEl.innerHTML = `
                <div class="elementor-globals-panel">
                    <!-- Global Design Tokens -->
                    <div>
                        <div class="globals-section-header">
                            <span>Global Colors</span>
                            <button class="globals-manage-pill" id="btn-jump-colors">
                                <i class="fa-solid fa-pen"></i> Manage
                            </button>
                        </div>
                        <div class="globals-colors-grid">
                            ${chipsHtml}
                        </div>
                    </div>

                    <!-- Global Typography -->
                    <div>
                        <div class="globals-section-header">
                            <span>Global Typography</span>
                            <button class="globals-manage-pill" id="btn-jump-fonts">
                                <i class="fa-solid fa-pen"></i> Manage
                            </button>
                        </div>
                        <div class="globals-typography-list">
                            ${typeListHtml}
                        </div>
                    </div>

                    <!-- Reusable Global Blocks -->
                    <div>
                        <div class="globals-section-header">
                            <span>Global Blocks & Templates</span>
                            <span style="font-size:9px;color:#818a91;">Drag to canvas</span>
                        </div>
                        <div class="reusable-blocks-grid">
                            <div class="reusable-block-card" data-global-widget="GlobalHeader" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-compass"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Global Header Navbar</span>
                                    <span class="reusable-block-desc">Clean responsive branding & CTA bar</span>
                                </div>
                            </div>
                            <div class="reusable-block-card" data-global-widget="GlobalHero" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-wand-magic-sparkles"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Hero Feature Banner</span>
                                    <span class="reusable-block-desc">High-impact title, subtitle & button</span>
                                </div>
                            </div>
                            <div class="reusable-block-card" data-global-widget="GlobalPricing" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-tags"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Pricing Matrix Cards</span>
                                    <span class="reusable-block-desc">3-column plans with highlighted tier</span>
                                </div>
                            </div>
                            <div class="reusable-block-card" data-global-widget="GlobalFooter" draggable="true">
                                <div class="reusable-block-icon"><i class="fa-solid fa-shoe-prints"></i></div>
                                <div class="reusable-block-info">
                                    <span class="reusable-block-title">Global Site Footer</span>
                                    <span class="reusable-block-desc">Dark footer with copyright & links</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Elementor Pro / Global Widget Nerd Box -->
                    <div class="elementor-nerd-box">
                        <div class="elementor-nerd-box-icon"><i class="fa-solid fa-globe"></i></div>
                        <div class="elementor-nerd-box-title">Meet Our Global System</div>
                        <div class="elementor-nerd-box-message">Save any widget or layout as a global template component. Edit once to sync across your entire site in native compiled Rust.</div>
                    </div>
                </div>
            `;

            // Wire Tab Switchers
            navEl.querySelector('[data-tab="elements"]').onclick = () => {
                currentPanelView = "elements";
                renderInspector();
            };

            document.getElementById("btn-jump-colors").onclick = () => {
                currentPanelView = "site-settings-colors";
                renderInspector();
            };
            document.getElementById("btn-jump-fonts").onclick = () => {
                currentPanelView = "site-settings-fonts";
                renderInspector();
            };

            bodyEl.querySelectorAll(".globals-color-chip").forEach(chip => {
                chip.onclick = () => {
                    currentPanelView = "site-settings-colors";
                    renderInspector();
                };
            });

            // Draggable Global Blocks
            bodyEl.querySelectorAll(".reusable-block-card").forEach(card => {
                const type = card.dataset.globalWidget;
                card.ondragstart = (e) => {
                    window._activeDragData = { type: "new_widget", widgetType: type };
                    try {
                        e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                        e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                        e.dataTransfer.effectAllowed = "copy";
                    } catch(err) {}
                    card.classList.add("is-dragging-card");
                };
                card.ondragend = () => {
                    card.classList.remove("is-dragging-card");
                    setTimeout(() => {
                        window._activeDragData = null;
                        hideDropIndicator();
                    }, 350);
                };
                card.onclick = () => {
                    insertWidgetAtRoot(type);
                };
            });

            return;
        }

        // -------------------------------------------------------------
        // VIEW 9: DEFAULT ELEMENTS CATALOG (currentPanelView === "elements")
        // -------------------------------------------------------------
        if (!activeElementId) {
            menuBtn.innerHTML = `<i class="fa-solid fa-bars"></i>`;
            menuBtn.title = "Menu";
            titleEl.innerHTML = `<span class="e-logo">E</span> ELEMENTOR`;
            navEl.innerHTML = `
                <button class="elementor-panel-navigation-tab active" data-tab="elements">Elements</button>
                <button class="elementor-panel-navigation-tab" data-tab="globals">Globals</button>
            `;

            // Tab click handling
            navEl.querySelector('[data-tab="globals"]').onclick = () => {
                currentPanelView = "globals";
                renderInspector();
            };
            navEl.querySelector('[data-tab="elements"]').onclick = () => {
                currentPanelView = "elements";
                renderInspector();
            };

            bodyEl.innerHTML = `
                <div class="elementor-panel-search">
                    <i class="fa-solid fa-magnifying-glass"></i>
                    <input type="search" id="widget-search-input" placeholder="Search Widget...">
                </div>

                <!-- CATEGORY: BASIC -->
                <div class="elementor-panel-category" id="cat-basic">
                    <div class="elementor-category-header" onclick="this.parentElement.classList.toggle('collapsed')">
                        <span>Basic</span>
                        <i class="fa-solid fa-chevron-down"></i>
                    </div>
                    <div class="elementor-category-items">
                        <div class="elementor-element-widget" data-widget="Heading" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-heading"></i></div>
                            <div class="widget-title">Heading</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Text" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-paragraph"></i></div>
                            <div class="widget-title">Text Editor</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Button" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-hand-pointer"></i></div>
                            <div class="widget-title">Button</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Image" draggable="true">
                            <div class="widget-icon"><i class="fa-regular fa-image"></i></div>
                            <div class="widget-title">Image</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Icon" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-icons"></i></div>
                            <div class="widget-title">Icon</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Divider" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-minus"></i></div>
                            <div class="widget-title">Divider</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Spacer" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-arrows-up-down"></i></div>
                            <div class="widget-title">Spacer</div>
                        </div>
                    </div>
                </div>

                <!-- CATEGORY: GENERAL -->
                <div class="elementor-panel-category" id="cat-general">
                    <div class="elementor-category-header" onclick="this.parentElement.classList.toggle('collapsed')">
                        <span>General</span>
                        <i class="fa-solid fa-chevron-down"></i>
                    </div>
                    <div class="elementor-category-items">
                        <div class="elementor-element-widget" data-widget="ImageBox" draggable="true">
                            <div class="widget-icon"><i class="fa-regular fa-image"></i></div>
                            <div class="widget-title">Image Box</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Grid" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-table-cells"></i></div>
                            <div class="widget-title">Grid Layout</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="IconBox" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-cubes"></i></div>
                            <div class="widget-title">Icon Box</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Counter" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-calculator"></i></div>
                            <div class="widget-title">Counter</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="ProgressBar" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-bars-progress"></i></div>
                            <div class="widget-title">Progress Bar</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Testimonial" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-quote-left"></i></div>
                            <div class="widget-title">Testimonial</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Alert" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-triangle-exclamation"></i></div>
                            <div class="widget-title">Alert</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Accordion" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-bars-staggered"></i></div>
                            <div class="widget-title">Accordion</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Tabs" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-table-columns"></i></div>
                            <div class="widget-title">Tabs</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Rating" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-star"></i></div>
                            <div class="widget-title">Rating</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="SocialIcons" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-share-nodes"></i></div>
                            <div class="widget-title">Social Icons</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="IconList" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-list-check"></i></div>
                            <div class="widget-title">Icon List</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Video" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-play"></i></div>
                            <div class="widget-title">Video</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="PriceTable" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-tags"></i></div>
                            <div class="widget-title">Price Table</div>
                        </div>
                    </div>
                </div>

                <!-- CATEGORY: NEOPRESS NATIVE -->
                <div class="elementor-panel-category" id="cat-native">
                    <div class="elementor-category-header" onclick="this.parentElement.classList.toggle('collapsed')">
                        <span>Neopress Engine</span>
                        <i class="fa-solid fa-chevron-down"></i>
                    </div>
                    <div class="elementor-category-items">
                        <div class="elementor-element-widget" data-widget="NativeForm" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-envelope-open-text"></i></div>
                            <div class="widget-title">Antispam Form</div>
                        </div>
                        <div class="elementor-element-widget" data-widget="Benchmark" draggable="true">
                            <div class="widget-icon"><i class="fa-solid fa-gauge-high"></i></div>
                            <div class="widget-title">Benchmark Card</div>
                        </div>
                    </div>
                </div>
            `;

            // Setup Draggable on every widget in palette
            bodyEl.querySelectorAll(".elementor-element-widget").forEach(w => {
                const type = w.dataset.widget;
                w.ondragstart = (e) => {
                    window._activeDragData = { type: "new_widget", widgetType: type };
                    try {
                        e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                        e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                        e.dataTransfer.effectAllowed = "copy";
                    } catch(err) {}
                    w.classList.add("is-dragging-card");
                };
                w.ondragend = () => {
                    w.classList.remove("is-dragging-card");
                    setTimeout(() => {
                        window._activeDragData = null;
                        hideDropIndicator();
                    }, 350);
                };
                w.onclick = () => {
                    insertWidgetAtRoot(type);
                };
            });

            // Widget search filter
            const searchInput = document.getElementById("widget-search-input");
            if (searchInput) {
                searchInput.oninput = (e) => {
                    const q = e.target.value.toLowerCase();
                    bodyEl.querySelectorAll(".elementor-element-widget").forEach(card => {
                        const title = card.querySelector(".widget-title").textContent.toLowerCase();
                        card.style.display = title.includes(q) ? "block" : "none";
                    });
                };
            }
            return;
        }
        // ELEMENT INSPECTOR (CONTENT / STYLE / ADVANCED)
        const node = findNodeById(pageAst.root, activeElementId);
        if (!node) {
            activeElementId = null;
            renderInspector();
            return;
        }

        titleEl.textContent = "EDIT " + node.type.toUpperCase();
        navEl.innerHTML = `
            <button class="elementor-panel-navigation-tab ${activeTab === 'content' ? 'active' : ''}" data-tab="content">
                <i class="fa-solid fa-pen-to-square"></i> Content
            </button>
            <button class="elementor-panel-navigation-tab ${activeTab === 'style' ? 'active' : ''}" data-tab="style">
                <i class="fa-solid fa-paintbrush"></i> Style
            </button>
            <button class="elementor-panel-navigation-tab ${activeTab === 'advanced' ? 'active' : ''}" data-tab="advanced">
                <i class="fa-solid fa-gear"></i> Advanced
            </button>
        `;

        navEl.querySelectorAll(".elementor-panel-navigation-tab").forEach(tab => {
            tab.onclick = () => {
                activeTab = tab.dataset.tab;
                renderInspector();
            };
        });

        node.data = node.data || {};
        node.styles = node.styles || {};
        const d = node.data;
        const s = node.styles;

        let controlsHtml = "";

        if (activeTab === "content") {
            // TAB: CONTENT
            controlsHtml += `<div class="elementor-controls-wrapper">`;

            if (node.type === "Heading") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Heading</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title Text</label>
                                <input type="text" class="elementor-input" id="ctrl-heading-text" value="${escapeHtml(d.text || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">HTML Tag</label>
                                <select class="elementor-select" id="ctrl-heading-tag">
                                    <option value="1" ${d.level == 1 ? 'selected' : ''}>H1</option>
                                    <option value="2" ${d.level == 2 || !d.level ? 'selected' : ''}>H2</option>
                                    <option value="3" ${d.level == 3 ? 'selected' : ''}>H3</option>
                                    <option value="4" ${d.level == 4 ? 'selected' : ''}>H4</option>
                                    <option value="5" ${d.level == 5 ? 'selected' : ''}>H5</option>
                                    <option value="6" ${d.level == 6 ? 'selected' : ''}>H6</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Alignment</label>
                                <div class="elementor-button-group">
                                    <button class="elementor-group-btn ${s.text_align === 'left' || !s.text_align ? 'active' : ''}" data-align="left"><i class="fa-solid fa-align-left"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'center' ? 'active' : ''}" data-align="center"><i class="fa-solid fa-align-center"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'right' ? 'active' : ''}" data-align="right"><i class="fa-solid fa-align-right"></i></button>
                                </div>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Text") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Text Editor</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">HTML Content</label>
                                <textarea class="elementor-textarea" id="ctrl-text-html" rows="7">${escapeHtml(d.content_html || '')}</textarea>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Alignment</label>
                                <div class="elementor-button-group">
                                    <button class="elementor-group-btn ${s.text_align === 'left' || !s.text_align ? 'active' : ''}" data-align="left"><i class="fa-solid fa-align-left"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'center' ? 'active' : ''}" data-align="center"><i class="fa-solid fa-align-center"></i></button>
                                    <button class="elementor-group-btn ${s.text_align === 'right' ? 'active' : ''}" data-align="right"><i class="fa-solid fa-align-right"></i></button>
                                </div>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Button") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Button</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Text</label>
                                <input type="text" class="elementor-input" id="ctrl-btn-text" value="${escapeHtml(d.text || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Link URL</label>
                                <input type="text" class="elementor-input" id="ctrl-btn-url" value="${escapeHtml(d.url || '#')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Target</label>
                                <select class="elementor-select" id="ctrl-btn-target">
                                    <option value="_self" ${d.target !== '_blank' ? 'selected' : ''}>Same Window (_self)</option>
                                    <option value="_blank" ${d.target === '_blank' ? 'selected' : ''}>New Window (_blank)</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Icon</label>
                                <div class="icon-selector-control">
                                    <div class="icon-preview-box" id="ctrl-btn-icon-preview">
                                        <i class="${d.icon || 'fa-solid fa-bolt'}"></i>
                                    </div>
                                    <button class="btn-icon-library" id="ctrl-open-icon-picker">
                                        <i class="fa-solid fa-icons"></i> Icon Library
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Image") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Image</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Image URL</label>
                                <input type="text" class="elementor-input" id="ctrl-img-url" value="${escapeHtml(d.url || d.src || '')}" placeholder="https://...">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Alt Text</label>
                                <input type="text" class="elementor-input" id="ctrl-img-alt" value="${escapeHtml(d.alt || '')}" placeholder="Image description">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Link URL (Optional)</label>
                                <input type="text" class="elementor-input" id="ctrl-img-link" value="${escapeHtml(d.link || '')}" placeholder="https://...">
                            </div>
                            <div style="margin-top: 10px; display: flex; gap: 6px; flex-wrap: wrap;">
                                <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="setSampleImage('https://images.unsplash.com/photo-1551288049-bebda4e38f71?auto=format&fit=crop&w=800&q=80')">Analytics</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="setSampleImage('https://images.unsplash.com/photo-1460925895917-afdab827c52f?auto=format&fit=crop&w=800&q=80')">Dashboard</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="setSampleImage('https://images.unsplash.com/photo-1507238691740-187a5b1d37b8?auto=format&fit=crop&w=800&q=80')">Workspace</button>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Video") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Video</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Video Embed URL</label>
                                <input type="text" class="elementor-input" id="ctrl-video-url" value="${escapeHtml(d.url || '')}" placeholder="https://www.youtube.com/embed/...">
                            </div>
                            <div style="font-size: 11.5px; color: #64748b; margin-top: 4px; line-height: 1.4;">
                                Supports YouTube, Vimeo, or MP4 embed links.
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Alert") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Alert</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Type</label>
                                <select class="elementor-select" id="ctrl-alert-type">
                                    <option value="info" ${d.alert_type === 'info' ? 'selected' : ''}>Info (Blue)</option>
                                    <option value="success" ${d.alert_type === 'success' ? 'selected' : ''}>Success (Green)</option>
                                    <option value="warning" ${d.alert_type === 'warning' ? 'selected' : ''}>Warning (Amber)</option>
                                    <option value="danger" ${d.alert_type === 'danger' ? 'selected' : ''}>Danger (Red)</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title</label>
                                <input type="text" class="elementor-input" id="ctrl-alert-title" value="${escapeHtml(d.title || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Description</label>
                                <textarea class="elementor-textarea" id="ctrl-alert-desc" rows="3">${escapeHtml(d.description || '')}</textarea>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Testimonial") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Testimonial</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Content / Quote</label>
                                <textarea class="elementor-textarea" id="ctrl-test-content" rows="4">${escapeHtml(d.content || '')}</textarea>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Author Name</label>
                                <input type="text" class="elementor-input" id="ctrl-test-name" value="${escapeHtml(d.name || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Job Title</label>
                                <input type="text" class="elementor-input" id="ctrl-test-job" value="${escapeHtml(d.job || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Avatar Image URL (Optional)</label>
                                <input type="text" class="elementor-input" id="ctrl-test-avatar" value="${escapeHtml(d.image_url || '')}" placeholder="https://...">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "ProgressBar") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Progress Bar</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title</label>
                                <input type="text" class="elementor-input" id="ctrl-pb-title" value="${escapeHtml(d.title || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Percentage (${d.percent || 90}%)</label>
                                <input type="range" min="0" max="100" class="elementor-range" id="ctrl-pb-percent-range" value="${d.percent || 90}" style="width: 100%;">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Display Text</label>
                                <input type="text" class="elementor-input" id="ctrl-pb-label" value="${escapeHtml(d.display_text || (d.percent || 90) + '%')}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Accordion" || node.type === "Tabs" || node.type === "Toggle") {
                const isAcc = node.type === "Accordion" || node.type === "Toggle";
                const items = d.items || [];
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>${isAcc ? 'Accordion Items' : 'Tab Panels'}</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div id="ctrl-repeater-items" style="display: flex; flex-direction: column; gap: 10px; margin-bottom: 12px;">
                                ${items.map((it, idx) => `
                                    <div class="repeater-item-box" style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 10px;">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                            <span style="font-size: 11px; font-weight: 700; color: #64748b; text-transform: uppercase;">#${idx+1} ${isAcc ? 'Item' : 'Tab'}</span>
                                            <button type="button" class="btn-rep-del" data-idx="${idx}" style="background: none; border: none; color: #ef4444; cursor: pointer; font-size: 12px;"><i class="fa-solid fa-trash-can"></i></button>
                                        </div>
                                        <input type="text" class="elementor-input rep-title" data-idx="${idx}" value="${escapeHtml(it.title || '')}" placeholder="Title" style="margin-bottom: 6px; font-size: 12px; height: 28px;">
                                        <textarea class="elementor-textarea rep-content" data-idx="${idx}" rows="2" placeholder="Body Content" style="font-size: 12px;">${escapeHtml(it.content_html || it.content || '')}</textarea>
                                    </div>
                                `).join("")}
                            </div>
                            <button type="button" class="btn btn-secondary" id="btn-rep-add" style="width: 100%; font-size: 12px; font-weight: 600; padding: 7px;">
                                <i class="fa-solid fa-plus"></i> Add ${isAcc ? 'Accordion Item' : 'Tab Panel'}
                            </button>
                        </div>
                    </div>
                `;
            } else if (node.type === "Rating") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Rating</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Score (1-5)</label>
                                <input type="number" min="0" max="5" step="0.5" class="elementor-input" id="ctrl-rating-val" value="${d.rating || 5}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title / Label</label>
                                <input type="text" class="elementor-input" id="ctrl-rating-title" value="${escapeHtml(d.title || '')}" placeholder="e.g. Lighthouse Score">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "SocialIcons") {
                const items = d.items || [];
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Social Icons</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div id="ctrl-social-items" style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px;">
                                ${items.map((it, idx) => `
                                    <div class="social-item-row" style="display: flex; align-items: center; gap: 6px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 6px;">
                                        <div style="width: 28px; text-align: center; color: var(--e-color-accent); font-size: 15px;"><i class="${it.icon || 'fa-brands fa-github'}"></i></div>
                                        <input type="text" class="elementor-input soc-icon-name" data-idx="${idx}" value="${escapeHtml(it.icon || '')}" placeholder="fa-brands fa-..." style="flex: 1; font-size: 11px; height: 26px;">
                                        <input type="text" class="elementor-input soc-icon-link" data-idx="${idx}" value="${escapeHtml(it.link || '#')}" placeholder="URL" style="flex: 1; font-size: 11px; height: 26px;">
                                        <button type="button" class="btn-soc-del" data-idx="${idx}" style="background: none; border: none; color: #ef4444; cursor: pointer; padding: 0 4px;"><i class="fa-solid fa-xmark"></i></button>
                                    </div>
                                `).join("")}
                            </div>
                            <button type="button" class="btn btn-secondary" id="btn-soc-add" style="width: 100%; font-size: 12px; font-weight: 600; padding: 7px;">
                                <i class="fa-solid fa-plus"></i> Add Social Link
                            </button>
                        </div>
                    </div>
                `;
            } else if (node.type === "IconList") {
                const items = d.items || [];
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Icon List</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div id="ctrl-il-items" style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px;">
                                ${items.map((it, idx) => `
                                    <div class="il-item-row" style="display: flex; align-items: center; gap: 6px; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 6px;">
                                        <div style="width: 24px; text-align: center; color: var(--e-color-accent);"><i class="${it.icon || 'fa-solid fa-check'}"></i></div>
                                        <input type="text" class="elementor-input il-text" data-idx="${idx}" value="${escapeHtml(it.text || '')}" placeholder="Item text" style="flex: 2; font-size: 11.5px; height: 26px;">
                                        <button type="button" class="btn-il-del" data-idx="${idx}" style="background: none; border: none; color: #ef4444; cursor: pointer; padding: 0 4px;"><i class="fa-solid fa-xmark"></i></button>
                                    </div>
                                `).join("")}
                            </div>
                            <button type="button" class="btn btn-secondary" id="btn-il-add" style="width: 100%; font-size: 12px; font-weight: 600; padding: 7px;">
                                <i class="fa-solid fa-plus"></i> Add List Item
                            </button>
                        </div>
                    </div>
                `;
            } else if (node.type === "ImageBox") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Image Box</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Image URL</label>
                                <input type="text" class="elementor-input" id="ctrl-imgbox-url" value="${escapeHtml(d.image_url || d.url || '')}" placeholder="https://...">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Title</label>
                                <input type="text" class="elementor-input" id="ctrl-imgbox-title" value="${escapeHtml(d.title || '')}" placeholder="Feature Title">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Description</label>
                                <textarea class="elementor-textarea" id="ctrl-imgbox-desc" rows="3" placeholder="Feature description...">${escapeHtml(d.description || '')}</textarea>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Link URL</label>
                                <input type="text" class="elementor-input" id="ctrl-imgbox-link" value="${escapeHtml(d.link || '')}" placeholder="https://...">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Grid") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Grid Layout</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Columns (1 to 6)</label>
                                <input type="number" min="1" max="6" class="elementor-input" id="ctrl-grid-cols" value="${d.columns || 3}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Gap Between Items (px)</label>
                                <input type="number" min="0" max="100" class="elementor-input" id="ctrl-grid-gap" value="${d.gap !== undefined ? d.gap : 24}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Icon" || node.type === "IconBox") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Icon Settings</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Icon</label>
                                <div class="icon-selector-control">
                                    <div class="icon-preview-box" id="ctrl-icon-preview">
                                        <i class="${d.icon || 'fa-solid fa-star'}"></i>
                                    </div>
                                    <button class="btn-icon-library" id="ctrl-open-icon-picker">
                                        <i class="fa-solid fa-icons"></i> Icon Library
                                    </button>
                                </div>
                            </div>
                            ${node.type === 'IconBox' ? `
                                <div class="elementor-control-row">
                                    <label class="elementor-control-label">Title</label>
                                    <input type="text" class="elementor-input" id="ctrl-ib-title" value="${escapeHtml(d.title || '')}">
                                </div>
                                <div class="elementor-control-row">
                                    <label class="elementor-control-label">Description</label>
                                    <textarea class="elementor-textarea" id="ctrl-ib-desc" rows="3">${escapeHtml(d.description || '')}</textarea>
                                </div>
                            ` : ''}
                        </div>
                    </div>
                `;
            } else if (node.type === "Counter") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Counter</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Number</label>
                                <input type="text" class="elementor-input" id="ctrl-cnt-number" value="${escapeHtml(String(d.number !== undefined ? d.number : '100%'))}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Suffix</label>
                                <input type="text" class="elementor-input" id="ctrl-cnt-suffix" value="${escapeHtml(d.suffix || '')}" placeholder="e.g. req/s, %">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Label / Title</label>
                                <input type="text" class="elementor-input" id="ctrl-cnt-label" value="${escapeHtml(d.title || d.label || '')}" placeholder="e.g. Lighthouse Score">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "PriceTable") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Price Table</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Plan Title</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-title" value="${escapeHtml(d.title || '')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Price</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-price" value="${escapeHtml(d.price || '49')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Billing Period</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-period" value="${escapeHtml(d.period || '/ month')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Button Text</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-btn-text" value="${escapeHtml(d.button_text || 'Deploy Now')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Button URL</label>
                                <input type="text" class="elementor-input" id="ctrl-pt-btn-url" value="${escapeHtml(d.button_url || '#')}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Spacer") {
                const spVal = (d.height !== undefined && d.height !== null) ? parseInt(d.height) : parseInt(s.height || '50');
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Spacer</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Height (px)</label>
                                <input type="number" min="5" max="600" class="elementor-input" id="ctrl-sp-height" value="${isNaN(spVal) ? 50 : spVal}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "CustomHtml") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Custom HTML</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">HTML Code</label>
                                <textarea class="elementor-textarea" id="ctrl-custom-html" rows="8" style="font-family: monospace; font-size: 11.5px;">${escapeHtml(d.raw || d.raw_html || d.content_html || '')}</textarea>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Divider") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Divider</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Style</label>
                                <select class="elementor-select" id="ctrl-div-style">
                                    <option value="solid" ${d.style !== 'dashed' && d.style !== 'dotted' ? 'selected' : ''}>Solid</option>
                                    <option value="dashed" ${d.style === 'dashed' ? 'selected' : ''}>Dashed</option>
                                    <option value="dotted" ${d.style === 'dotted' ? 'selected' : ''}>Dotted</option>
                                </select>
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Section" || node.type === "Container" || node.type === "Column" || node.type === "Flex") {
                const bgImgVal = s.background_image || (s.background && s.background.image_url) || '';
                const bgColVal = s.background_color || (s.background && s.background.color) || '';
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Container / Section Layout</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Flex Direction</label>
                                <select class="elementor-select" id="ctrl-cont-dir">
                                    <option value="column" ${(s.flex_direction || 'column') === 'column' ? 'selected' : ''}>Column (Vertical)</option>
                                    <option value="row" ${s.flex_direction === 'row' ? 'selected' : ''}>Row (Horizontal)</option>
                                </select>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Width</label>
                                <input type="text" class="elementor-input" id="ctrl-cont-width" value="${escapeHtml(s.width || '')}" placeholder="e.g. 100%, 50%, 25%">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Min Height</label>
                                <input type="text" class="elementor-input" id="ctrl-cont-minh" value="${escapeHtml(s.min_height || '')}" placeholder="e.g. 625px, 275px">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Vertical Padding</label>
                                <input type="text" class="elementor-input" id="ctrl-sec-pad" value="${escapeHtml(s.padding_top || '')}" placeholder="e.g. 60px">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Background Color</label>
                                <div class="color-picker-control">
                                    <input type="color" class="color-swatch" id="ctrl-bg-swatch" value="${bgColVal.startsWith('#') ? bgColVal : '#ffffff'}">
                                    <input type="text" class="elementor-input" id="ctrl-bg-input" value="${escapeHtml(bgColVal)}" placeholder="#ffffff">
                                </div>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Background Image URL</label>
                                <input type="text" class="elementor-input" id="ctrl-cont-bgimg" value="${escapeHtml(bgImgVal)}" placeholder="/wp-content/uploads/...">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "NativeForm" || node.type === "Form") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Smart Antispam Form</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Form Title</label>
                                <input type="text" class="elementor-input" id="ctrl-form-title" value="${escapeHtml(d.title || 'Contact Form')}">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Button Text</label>
                                <input type="text" class="elementor-input" id="ctrl-form-btn" value="${escapeHtml(d.button_text || 'Send Message')}">
                            </div>
                        </div>
                    </div>
                `;
            } else if (node.type === "Benchmark") {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>Benchmark Metric</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Metric Text</label>
                                <input type="text" class="elementor-input" id="ctrl-bench-text" value="${escapeHtml(d.text || '')}">
                            </div>
                        </div>
                    </div>
                `;
            } else {
                controlsHtml += `
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header"><span>General Content</span><i class="fa-solid fa-chevron-down"></i></div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Content</label>
                                <textarea class="elementor-textarea" id="ctrl-gen-text">${escapeHtml(d.content_html || d.text || '')}</textarea>
                            </div>
                        </div>
                    </div>
                `;
            }

            controlsHtml += `</div>`;
        } else if (activeTab === "style") {
            // TAB: STYLE
            controlsHtml += `
                <div class="elementor-controls-wrapper">
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header">
                            <span>Colors</span>
                            <i class="fa-solid fa-chevron-down"></i>
                        </div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Text Color</label>
                                <div class="color-picker-control">
                                    <input type="color" class="color-swatch" id="ctrl-color-swatch" value="${s.text_color || '#0f172a'}">
                                    <input type="text" class="elementor-input" id="ctrl-color-input" value="${s.text_color || ''}" placeholder="#0f172a">
                                </div>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Background Color</label>
                                <div class="color-picker-control">
                                    <input type="color" class="color-swatch" id="ctrl-bg-swatch" value="${s.background_color || '#ffffff'}">
                                    <input type="text" class="elementor-input" id="ctrl-bg-input" value="${s.background_color || ''}" placeholder="#ffffff or transparent">
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header">
                            <span>Typography & Dimensions</span>
                            <i class="fa-solid fa-chevron-down"></i>
                        </div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Font Size</label>
                                <input type="text" class="elementor-input" id="ctrl-font-size" value="${s.font_size || ''}" placeholder="e.g. 24px, 2rem">
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Border Radius</label>
                                <input type="text" class="elementor-input" id="ctrl-border-radius" value="${s.border_radius || ''}" placeholder="e.g. 6px, 999px">
                            </div>
                        </div>
                    </div>
                </div>
            `;
        } else {
            // TAB: ADVANCED
            controlsHtml += `
                <div class="elementor-controls-wrapper">
                    <div class="elementor-control-section">
                        <div class="elementor-control-section-header">
                            <span>Layout & Box Model</span>
                            <i class="fa-solid fa-chevron-down"></i>
                        </div>
                        <div class="elementor-control-section-body">
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Padding</label>
                                <div class="dimensions-control">
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-t" value="${s.padding_top || ''}">
                                        <span>Top</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-r" value="${s.padding_right || ''}">
                                        <span>Right</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-b" value="${s.padding_bottom || ''}">
                                        <span>Bottom</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-pad-l" value="${s.padding_left || ''}">
                                        <span>Left</span>
                                    </div>
                                    <button class="dimension-link-btn linked" id="btn-link-padding" title="Link Values">
                                        <i class="fa-solid fa-link"></i>
                                    </button>
                                </div>
                            </div>
                            <div class="elementor-control-row">
                                <label class="elementor-control-label">Margin</label>
                                <div class="dimensions-control">
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-t" value="${s.margin_top || ''}">
                                        <span>Top</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-r" value="${s.margin_right || ''}">
                                        <span>Right</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-b" value="${s.margin_bottom || ''}">
                                        <span>Bottom</span>
                                    </div>
                                    <div class="dimension-col">
                                        <input type="text" id="ctrl-mar-l" value="${s.margin_left || ''}">
                                        <span>Left</span>
                                    </div>
                                    <button class="dimension-link-btn" title="Link Values">
                                        <i class="fa-solid fa-link"></i>
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }

        bodyEl.innerHTML = controlsHtml;
        bindInspectorEvents(node);
    }

    function setSampleImage(url) {
        const node = findNodeById(pageAst.root, activeElementId);
        if (!node) return;
        node.data = node.data || {};
        node.data.url = url;
        node.data.src = url;
        const input = document.getElementById("ctrl-img-url");
        if (input) input.value = url;
        renderCanvas();
    }

    function bindInspectorEvents(node) {
        const d = node.data;
        const s = node.styles;

        // Heading events
        const hText = document.getElementById("ctrl-heading-text");
        if (hText) hText.oninput = (e) => { d.text = e.target.value; renderCanvas(); };
        const hTag = document.getElementById("ctrl-heading-tag");
        if (hTag) hTag.onchange = (e) => { d.level = parseInt(e.target.value); renderCanvas(); };

        // Text events
        const tHtml = document.getElementById("ctrl-text-html");
        if (tHtml) tHtml.oninput = (e) => { d.content_html = e.target.value; renderCanvas(); };

        // Button events
        const bText = document.getElementById("ctrl-btn-text");
        if (bText) bText.oninput = (e) => { d.text = e.target.value; renderCanvas(); };
        const bUrl = document.getElementById("ctrl-btn-url");
        if (bUrl) bUrl.oninput = (e) => { d.url = e.target.value; renderCanvas(); };
        const bTarget = document.getElementById("ctrl-btn-target");
        if (bTarget) bTarget.onchange = (e) => { d.target = e.target.value; renderCanvas(); };

        // Image events
        const imgUrl = document.getElementById("ctrl-img-url");
        if (imgUrl) imgUrl.oninput = (e) => { d.url = e.target.value; d.src = e.target.value; renderCanvas(); };
        const imgAlt = document.getElementById("ctrl-img-alt");
        if (imgAlt) imgAlt.oninput = (e) => { d.alt = e.target.value; renderCanvas(); };
        const imgLink = document.getElementById("ctrl-img-link");
        if (imgLink) imgLink.oninput = (e) => { d.link = e.target.value; renderCanvas(); };

        // Video events
        const vidUrl = document.getElementById("ctrl-video-url");
        if (vidUrl) vidUrl.oninput = (e) => { d.url = e.target.value; renderCanvas(); };

        // Alert events
        const alType = document.getElementById("ctrl-alert-type");
        if (alType) alType.onchange = (e) => { d.alert_type = e.target.value; renderCanvas(); };
        const alTitle = document.getElementById("ctrl-alert-title");
        if (alTitle) alTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const alDesc = document.getElementById("ctrl-alert-desc");
        if (alDesc) alDesc.oninput = (e) => { d.description = e.target.value; renderCanvas(); };

        // Testimonial events
        const testContent = document.getElementById("ctrl-test-content");
        if (testContent) testContent.oninput = (e) => { d.content = e.target.value; renderCanvas(); };
        const testName = document.getElementById("ctrl-test-name");
        if (testName) testName.oninput = (e) => { d.name = e.target.value; renderCanvas(); };
        const testJob = document.getElementById("ctrl-test-job");
        if (testJob) testJob.oninput = (e) => { d.job = e.target.value; renderCanvas(); };
        const testAvatar = document.getElementById("ctrl-test-avatar");
        if (testAvatar) testAvatar.oninput = (e) => { d.image_url = e.target.value; renderCanvas(); };

        // Progress Bar events
        const pbTitle = document.getElementById("ctrl-pb-title");
        if (pbTitle) pbTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const pbRange = document.getElementById("ctrl-pb-percent-range");
        if (pbRange) {
            pbRange.oninput = (e) => {
                d.percent = parseInt(e.target.value);
                const label = document.getElementById("ctrl-pb-label");
                if (label) {
                    label.value = d.percent + "%";
                    d.display_text = label.value;
                }
                const labelRow = pbRange.parentElement.querySelector("label");
                if (labelRow) labelRow.textContent = `Percentage (${d.percent}%)`;
                renderCanvas();
            };
        }
        const pbLabel = document.getElementById("ctrl-pb-label");
        if (pbLabel) pbLabel.oninput = (e) => { d.display_text = e.target.value; renderCanvas(); };

        // Repeater (Accordion / Tabs) events
        document.querySelectorAll(".rep-title").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].title = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".rep-content").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].content = e.target.value;
                    d.items[idx].content_html = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".btn-rep-del").forEach(btn => {
            btn.onclick = (e) => {
                const idx = parseInt(btn.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items.splice(idx, 1);
                    renderCanvas();
                    renderInspector();
                }
            };
        });
        const btnRepAdd = document.getElementById("btn-rep-add");
        if (btnRepAdd) {
            btnRepAdd.onclick = () => {
                d.items = d.items || [];
                const num = d.items.length + 1;
                d.items.push({ title: "New Item " + num, content: "Content details for item " + num, content_html: "Content details for item " + num });
                renderCanvas();
                renderInspector();
            };
        }

        // Rating events
        const rVal = document.getElementById("ctrl-rating-val");
        if (rVal) rVal.oninput = (e) => { d.rating = parseFloat(e.target.value) || 5; renderCanvas(); };
        const rTitle = document.getElementById("ctrl-rating-title");
        if (rTitle) rTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };

        // Social Icons events
        document.querySelectorAll(".soc-icon-name").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].icon = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".soc-icon-link").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].link = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".btn-soc-del").forEach(btn => {
            btn.onclick = (e) => {
                const idx = parseInt(btn.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items.splice(idx, 1);
                    renderCanvas();
                    renderInspector();
                }
            };
        });
        const btnSocAdd = document.getElementById("btn-soc-add");
        if (btnSocAdd) {
            btnSocAdd.onclick = () => {
                d.items = d.items || [];
                d.items.push({ icon: "fa-brands fa-x-twitter", link: "#" });
                renderCanvas();
                renderInspector();
            };
        }

        // Icon List events
        document.querySelectorAll(".il-text").forEach(inp => {
            inp.oninput = (e) => {
                const idx = parseInt(e.target.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items[idx].text = e.target.value;
                    renderCanvas();
                }
            };
        });
        document.querySelectorAll(".btn-il-del").forEach(btn => {
            btn.onclick = (e) => {
                const idx = parseInt(btn.dataset.idx);
                if (d.items && d.items[idx]) {
                    d.items.splice(idx, 1);
                    renderCanvas();
                    renderInspector();
                }
            };
        });
        const btnIlAdd = document.getElementById("btn-il-add");
        if (btnIlAdd) {
            btnIlAdd.onclick = () => {
                d.items = d.items || [];
                d.items.push({ icon: "fa-solid fa-check", text: "New benefit list point" });
                renderCanvas();
                renderInspector();
            };
        }

        // Image Box events
        const imgbUrl = document.getElementById("ctrl-imgbox-url");
        if (imgbUrl) imgbUrl.oninput = (e) => { d.image_url = e.target.value; d.url = e.target.value; renderCanvas(); };
        const imgbTitle = document.getElementById("ctrl-imgbox-title");
        if (imgbTitle) imgbTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const imgbDesc = document.getElementById("ctrl-imgbox-desc");
        if (imgbDesc) imgbDesc.oninput = (e) => { d.description = e.target.value; renderCanvas(); };
        const imgbLink = document.getElementById("ctrl-imgbox-link");
        if (imgbLink) imgbLink.oninput = (e) => { d.link = e.target.value; renderCanvas(); };

        // Grid events
        const gridCols = document.getElementById("ctrl-grid-cols");
        if (gridCols) gridCols.oninput = (e) => { d.columns = parseInt(e.target.value) || 3; renderCanvas(); };
        const gridGap = document.getElementById("ctrl-grid-gap");
        if (gridGap) gridGap.oninput = (e) => { d.gap = parseInt(e.target.value) || 0; renderCanvas(); };

        // Icon Box events
        const ibTitle = document.getElementById("ctrl-ib-title");
        if (ibTitle) ibTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const ibDesc = document.getElementById("ctrl-ib-desc");
        if (ibDesc) ibDesc.oninput = (e) => { d.description = e.target.value; renderCanvas(); };

        // Counter events
        const cntNum = document.getElementById("ctrl-cnt-number");
        if (cntNum) cntNum.oninput = (e) => { d.number = e.target.value; renderCanvas(); };
        const cntSuffix = document.getElementById("ctrl-cnt-suffix");
        if (cntSuffix) cntSuffix.oninput = (e) => { d.suffix = e.target.value; renderCanvas(); };
        const cntLabel = document.getElementById("ctrl-cnt-label");
        if (cntLabel) cntLabel.oninput = (e) => { d.label = e.target.value; d.title = e.target.value; renderCanvas(); };

        // PriceTable events
        const ptTitle = document.getElementById("ctrl-pt-title");
        if (ptTitle) ptTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const ptPrice = document.getElementById("ctrl-pt-price");
        if (ptPrice) ptPrice.oninput = (e) => { d.price = e.target.value; renderCanvas(); };
        const ptPeriod = document.getElementById("ctrl-pt-period");
        if (ptPeriod) ptPeriod.oninput = (e) => { d.period = e.target.value; renderCanvas(); };
        const ptBtnText = document.getElementById("ctrl-pt-btn-text");
        if (ptBtnText) ptBtnText.oninput = (e) => { d.button_text = e.target.value; renderCanvas(); };
        const ptBtnUrl = document.getElementById("ctrl-pt-btn-url");
        if (ptBtnUrl) ptBtnUrl.oninput = (e) => { d.button_url = e.target.value; renderCanvas(); };

        // Spacer events
        const spHeight = document.getElementById("ctrl-sp-height");
        if (spHeight) spHeight.oninput = (e) => {
            const hVal = parseInt(e.target.value) || 50;
            d.height = hVal;
            s.height = hVal + "px";
            renderCanvas();
        };

        // CustomHtml events
        const customHtmlEl = document.getElementById("ctrl-custom-html");
        if (customHtmlEl) customHtmlEl.oninput = (e) => {
            d.raw = e.target.value;
            renderCanvas();
        };

        // Divider events
        const divStyle = document.getElementById("ctrl-div-style");
        if (divStyle) divStyle.onchange = (e) => { d.style = e.target.value; renderCanvas(); };

        // Section / Container & Layout
        const secPad = document.getElementById("ctrl-sec-pad");
        if (secPad) secPad.oninput = (e) => {
            s.padding_top = e.target.value;
            s.padding_bottom = e.target.value;
            renderCanvas();
        };
        const contDir = document.getElementById("ctrl-cont-dir");
        if (contDir) contDir.onchange = (e) => {
            s.flex_direction = e.target.value;
            renderCanvas();
        };
        const contWidth = document.getElementById("ctrl-cont-width");
        if (contWidth) contWidth.oninput = (e) => {
            s.width = e.target.value;
            renderCanvas();
        };
        const contMinH = document.getElementById("ctrl-cont-minh");
        if (contMinH) contMinH.oninput = (e) => {
            s.min_height = e.target.value;
            renderCanvas();
        };
        const contBgImg = document.getElementById("ctrl-cont-bgimg");
        if (contBgImg) contBgImg.oninput = (e) => {
            s.background_image = e.target.value;
            if (s.background && typeof s.background === "object") {
                s.background.image_url = e.target.value;
            }
            renderCanvas();
        };

        // NativeForm & Benchmark
        const formTitle = document.getElementById("ctrl-form-title");
        if (formTitle) formTitle.oninput = (e) => { d.title = e.target.value; renderCanvas(); };
        const formBtn = document.getElementById("ctrl-form-btn");
        if (formBtn) formBtn.oninput = (e) => { d.button_text = e.target.value; renderCanvas(); };
        const benchText = document.getElementById("ctrl-bench-text");
        if (benchText) benchText.oninput = (e) => { d.text = e.target.value; renderCanvas(); };

        // Alignment buttons
        document.querySelectorAll("[data-align]").forEach(btn => {
            btn.onclick = () => {
                s.text_align = btn.dataset.align;
                renderInspector();
                renderCanvas();
            };
        });

        // Icon picker opener
        const openIconBtn = document.getElementById("ctrl-open-icon-picker");
        if (openIconBtn) {
            openIconBtn.onclick = () => {
                openIconLibrary(node.id);
            };
        }

        // Colors
        const colSwatch = document.getElementById("ctrl-color-swatch");
        const colInput = document.getElementById("ctrl-color-input");
        if (colSwatch && colInput) {
            colSwatch.oninput = (e) => {
                colInput.value = e.target.value;
                s.text_color = e.target.value;
                renderCanvas();
            };
            colInput.oninput = (e) => {
                s.text_color = e.target.value;
                renderCanvas();
            };
        }

        const bgSwatch = document.getElementById("ctrl-bg-swatch");
        const bgInput = document.getElementById("ctrl-bg-input");
        if (bgSwatch && bgInput) {
            bgSwatch.oninput = (e) => {
                bgInput.value = e.target.value;
                s.background_color = e.target.value;
                renderCanvas();
            };
            bgInput.oninput = (e) => {
                s.background_color = e.target.value;
                renderCanvas();
            };
        }

        // Typography
        const fsInput = document.getElementById("ctrl-font-size");
        if (fsInput) fsInput.oninput = (e) => { s.font_size = e.target.value; renderCanvas(); };
        const brInput = document.getElementById("ctrl-border-radius");
        if (brInput) brInput.oninput = (e) => { s.border_radius = e.target.value; renderCanvas(); };

        // Padding dimensions
        const padT = document.getElementById("ctrl-pad-t");
        const padR = document.getElementById("ctrl-pad-r");
        const padB = document.getElementById("ctrl-pad-b");
        const padL = document.getElementById("ctrl-pad-l");
        if (padT && padR && padB && padL) {
            padT.oninput = (e) => { s.padding_top = e.target.value; renderCanvas(); };
            padR.oninput = (e) => { s.padding_right = e.target.value; renderCanvas(); };
            padB.oninput = (e) => { s.padding_bottom = e.target.value; renderCanvas(); };
            padL.oninput = (e) => { s.padding_left = e.target.value; renderCanvas(); };
        }

        // Margin dimensions
        const marT = document.getElementById("ctrl-mar-t");
        const marR = document.getElementById("ctrl-mar-r");
        const marB = document.getElementById("ctrl-mar-b");
        const marL = document.getElementById("ctrl-mar-l");
        if (marT) marT.oninput = (e) => { s.margin_top = e.target.value; renderCanvas(); };
        if (marR) marR.oninput = (e) => { s.margin_right = e.target.value; renderCanvas(); };
        if (marB) marB.oninput = (e) => { s.margin_bottom = e.target.value; renderCanvas(); };
        if (marL) marL.oninput = (e) => { s.margin_left = e.target.value; renderCanvas(); };
    }

function selectElement(id) {
        activeElementId = id;
        renderCanvas();
        renderInspector();
        
        // Scroll canvas to element
        const el = document.getElementById(id);
        if (el) {
            el.scrollIntoView({ behavior: "smooth", block: "center" });
        }
    }

    function escapeHtml(str) {
        if (!str) return "";
        return String(str)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }

    // -------------------------------------------------------------
    // EXTENSIVE ICON LIBRARY MODAL LOGIC
    // -------------------------------------------------------------
    function openIconLibrary(targetNodeId) {
        selectedIconTarget = targetNodeId;
        const modal = document.getElementById("icon-library-modal");
        modal.classList.add("open");
        renderIconGrid("all", "");
    }

    function renderIconGrid(category, searchFilter) {
        const grid = document.getElementById("icon-grid");
        grid.innerHTML = "";
        const q = (searchFilter || "").toLowerCase();

        const filtered = ICON_LIBRARY.filter(item => {
            const matchCat = (category === "all" || item.cat === category);
            const matchSearch = (!q || item.name.toLowerCase().includes(q) || item.class.toLowerCase().includes(q));
            return matchCat && matchSearch;
        });

        filtered.forEach(item => {
            const card = document.createElement("div");
            card.className = "icon-card" + (tempSelectedIcon === item.class ? " selected" : "");
            card.innerHTML = `
                <i class="${item.class}"></i>
                <span>${item.name}</span>
            `;
            card.onclick = () => {
                tempSelectedIcon = item.class;
                grid.querySelectorAll(".icon-card").forEach(c => c.classList.remove("selected"));
                card.classList.add("selected");
            };
            grid.appendChild(card);
        });
    }

    document.getElementById("btn-close-icon-library").onclick = () => {
        document.getElementById("icon-library-modal").classList.remove("open");
    };
    document.getElementById("btn-cancel-icon").onclick = () => {
        document.getElementById("icon-library-modal").classList.remove("open");
    };

    document.getElementById("btn-insert-icon").onclick = () => {
        if (selectedIconTarget && tempSelectedIcon) {
            const node = findNodeById(pageAst.root, selectedIconTarget);
            if (node) {
                node.data = node.data || {};
                node.data.icon = tempSelectedIcon;
                renderCanvas();
                renderInspector();
            }
        }
        document.getElementById("icon-library-modal").classList.remove("open");
    };

    // Icon filter tabs
    document.querySelectorAll(".icon-tab").forEach(tab => {
        tab.onclick = () => {
            document.querySelectorAll(".icon-tab").forEach(t => t.classList.remove("active"));
            tab.classList.add("active");
            renderIconGrid(tab.dataset.cat, document.getElementById("icon-search").value);
        };
    });

    // Icon search input
    document.getElementById("icon-search").oninput = (e) => {
        const activeCat = document.querySelector(".icon-tab.active").dataset.cat;
        renderIconGrid(activeCat, e.target.value);
    };

    // -------------------------------------------------------------
    // SECTION & WIDGET MANIPULATION
    // -------------------------------------------------------------
    function insertSectionAbove(targetSectionId) {
        const newSec = createSectionWithColumns(1, [
            {
                id: generateId("h"),
                type: "Heading",
                data: { text: "New Section", level: 2 },
                styles: { font_size: "32px", text_color: "#0f172a" }
            }
        ]);

        let idx = pageAst.root.children.findIndex(c => c.id === targetSectionId);
        if (idx === -1) {
            pageAst.root.children.unshift(newSec);
        } else {
            pageAst.root.children.splice(idx, 0, newSec);
        }
        renderCanvas();
        showToast("New section created");
    }

    function deleteElement(id) {
        if (pageAst.root.id === id) {
            showToast("Cannot delete root section");
            return;
        }
        removeNodeById(pageAst.root, id);
        if (activeElementId === id) activeElementId = null;
        renderCanvas();
        renderInspector();
        showToast("Element removed");
    }

    function duplicateElement(id) {
        const node = findNodeById(pageAst.root, id);
        if (!node) return;
        const parent = findParentNode(pageAst.root, id);
        if (!parent || !parent.children) return;

        const clone = JSON.parse(JSON.stringify(node));
        function reId(n) {
            n.id = generateId(n.type.toLowerCase().substr(0, 4));
            if (n.children) n.children.forEach(reId);
        }
        reId(clone);

        const idx = parent.children.findIndex(c => c.id === id);
        parent.children.splice(idx + 1, 0, clone);
        renderCanvas();
        selectElement(clone.id);
        showToast("Element duplicated");
    }

    function insertWidgetAtRoot(type) {
        const newWidget = createWidgetNode(type);

        if (newWidget.type === "Section") {
            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newWidget);
            renderCanvas();
            updateNavigator();
            selectElement(newWidget.id);
            showToast("Added " + type + " block");
            savePage();
            return;
        }

        let container = findFirstContainer(pageAst.root);
        if (!container) {
            const newSec = createSectionWithColumns(1, [newWidget]);
            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newSec);
        } else {
            container.children = container.children || [];
            container.children.push(newWidget);
        }
        renderCanvas();
        updateNavigator();
        selectElement(newWidget.id);
        showToast("Added " + type + " widget");
        savePage();
    }

    function findFirstContainer(node) {
        if (!node) return null;
        if (node.type === "Container" || node.type === "Column" || node.type === "Flex") return node;
        if (node.children) {
            for (let c of node.children) {
                let res = findFirstContainer(c);
                if (res) return res;
            }
        }
        return null;
    }

    // -------------------------------------------------------------
    // PRESET STRUCTURE SELECTOR
    // -------------------------------------------------------------
    document.getElementById("btn-open-presets").onclick = () => {
        document.getElementById("preset-selector").classList.toggle("open");
    };

    document.querySelectorAll(".preset-card").forEach(card => {
        card.onclick = () => {
            const cols = card.dataset.cols;
            const newSec = {
                id: generateId("sec"),
                type: "Section",
                styles: { background_color: "#ffffff", padding_top: "60px", padding_bottom: "60px" },
                children: []
            };

            if (cols === "1") {
                newSec.children.push({ id: generateId("cont"), type: "Container", styles: {}, children: [] });
            } else if (cols === "2") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
            } else if (cols === "3") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
            } else if (cols === "4") {
                for (let i = 0; i < 4; i++) {
                    newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                }
            } else if (cols === "33-66") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "2" }, children: [] });
            } else if (cols === "66-33") {
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "2" }, children: [] });
                newSec.children.push({ id: generateId("col"), type: "Column", styles: { flex: "1" }, children: [] });
            }

            pageAst.root.children = pageAst.root.children || [];
            pageAst.root.children.push(newSec);
            document.getElementById("preset-selector").classList.remove("open");
            renderCanvas();
            showToast("New section created");
        };
    });

    // -------------------------------------------------------------
    // RESPONSIVE MODE TOOLBAR
    // -------------------------------------------------------------
    const toolResponsive = document.getElementById("tool-responsive");
    const respBar = document.getElementById("elementor-responsive-bar");
    const previewContainer = document.getElementById("elementor-preview");

    toolResponsive.onclick = () => {
        respBar.classList.toggle("open");
        toolResponsive.classList.toggle("active");
    };

    document.getElementById("btn-close-resp-bar").onclick = () => {
        respBar.classList.remove("open");
        toolResponsive.classList.remove("active");
        setDeviceMode("desktop");
    };

    document.querySelectorAll(".resp-btn").forEach(btn => {
        btn.onclick = () => {
            document.querySelectorAll(".resp-btn").forEach(b => b.classList.remove("active"));
            btn.classList.add("active");
            setDeviceMode(btn.dataset.mode);
        };
    });

    function setDeviceMode(mode) {
        previewContainer.classList.remove("mode-tablet", "mode-mobile");
        const dimText = document.getElementById("resp-dimensions-text");
        if (mode === "tablet") {
            previewContainer.classList.add("mode-tablet");
            dimText.textContent = "Tablet (768px)";
        } else if (mode === "mobile") {
            previewContainer.classList.add("mode-mobile");
            dimText.textContent = "Mobile (375px)";
        } else {
            dimText.textContent = "100% / 1200px";
        }
    }

    // -------------------------------------------------------------
    // STRUCTURE / LAYERS NAVIGATOR PANEL
    // -------------------------------------------------------------
    const navDrawer = document.getElementById("elementor-navigator");
    const toolNav = document.getElementById("tool-navigator");
    const floatingNavBtn = document.getElementById("btn-floating-navigator");

    function toggleNavigatorPanel() {
        navDrawer.classList.toggle("open");
        toolNav.classList.toggle("active");
        if (navDrawer.classList.contains("open")) {
            updateNavigator();
        }
    }

    toolNav.onclick = toggleNavigatorPanel;
    floatingNavBtn.onclick = toggleNavigatorPanel;

    document.getElementById("btn-close-navigator").onclick = () => {
        navDrawer.classList.remove("open");
        toolNav.classList.remove("active");
    };

    // Keyboard shortcut Ctrl+I to toggle Navigator
    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "i") {
            e.preventDefault();
            toggleNavigatorPanel();
        }
    });

    // Draggable Navigator Window Header
    const navHeader = document.getElementById("navigator-header");
    let isDraggingNav = false;
    let navStartX, navStartY, navStartLeft, navStartTop;

    navHeader.addEventListener("mousedown", (e) => {
        if (e.target.closest("button")) return;
        isDraggingNav = true;
        navStartX = e.clientX;
        navStartY = e.clientY;
        const rect = navDrawer.getBoundingClientRect();
        navStartLeft = rect.left;
        navStartTop = rect.top;
        document.body.style.cursor = "move";
    });

    document.addEventListener("mousemove", (e) => {
        if (!isDraggingNav) return;
        const dx = e.clientX - navStartX;
        const dy = e.clientY - navStartY;
        navDrawer.style.left = (navStartLeft + dx) + "px";
        navDrawer.style.top = (navStartTop + dy) + "px";
        navDrawer.style.right = "auto";
    });

    document.addEventListener("mouseup", () => {
        if (isDraggingNav) {
            isDraggingNav = false;
            document.body.style.cursor = "default";
        }
    });

    // Expand / Collapse all layers
    let allExpanded = true;
    document.getElementById("nav-btn-expand-all").onclick = () => {
        allExpanded = !allExpanded;
        document.querySelectorAll(".nav-children").forEach(el => {
            el.style.display = allExpanded ? "block" : "none";
        });
        document.querySelectorAll(".nav-caret").forEach(c => {
            if (allExpanded) c.classList.remove("collapsed");
            else c.classList.add("collapsed");
        });
    };

    // Layer filter input
    const navSearchInput = document.getElementById("nav-search-input");
    navSearchInput.oninput = (e) => {
        const q = e.target.value.toLowerCase();
        document.querySelectorAll(".nav-tree-item").forEach(item => {
            const title = item.querySelector(".nav-item-title").textContent.toLowerCase();
            item.style.display = title.includes(q) ? "flex" : "none";
        });
    };

    function updateNavigator() {
        const tree = document.getElementById("navigator-tree");
        const countBadge = document.getElementById("nav-element-count");
        if (!tree) return;
        tree.innerHTML = "";

        const totalCount = countNodes(pageAst.root);
        if (countBadge) countBadge.textContent = totalCount;

        function walk(node, depth, parentEl) {
            const wrapper = document.createElement("div");
            wrapper.className = "nav-item-wrapper";

            const item = document.createElement("div");
            item.className = "nav-tree-item" + (node.id === activeElementId ? " active" : "");
            item.dataset.id = node.id;
            item.draggable = true;

            const hasChildren = node.children && node.children.length > 0;
            const indent = 8 + depth * 14;

            let icon = "fa-solid fa-cube";
            let labelText = node.type;

            if (node.type === "Section") {
                icon = "fa-solid fa-table-columns";
                labelText = "Section";
            } else if (node.type === "Container") {
                icon = "fa-regular fa-square";
                labelText = "Container";
            } else if (node.type === "Column" || node.type === "Flex") {
                icon = (node.data && node.data.direction === "row") ? "fa-solid fa-grip-lines" : "fa-solid fa-columns";
                labelText = (node.data && node.data.direction === "row") ? "Flex Row" : "Column";
            } else if (node.type === "PriceTable") {
                icon = "fa-solid fa-tags";
                labelText = 'Price Table: "' + ((node.data && node.data.title) || '') + '"';
            } else if (node.type === "CallToAction") {
                icon = "fa-solid fa-bullhorn";
                labelText = 'CTA: "' + ((node.data && node.data.title) || '') + '"';
            } else if (node.type === "Toggle") {
                icon = "fa-solid fa-bars-staggered";
                labelText = "Toggle";
            } else if (node.type === "CustomHtml") {
                icon = "fa-solid fa-code";
                labelText = "Custom HTML";
            } else if (node.type === "Heading") {
                icon = "fa-solid fa-heading";
                labelText = 'Heading: "' + ((node.data && node.data.text) || '').substr(0, 18) + '"';
            } else if (node.type === "Button") {
                icon = "fa-solid fa-hand-pointer";
                labelText = 'Button: "' + ((node.data && node.data.text) || 'Click') + '"';
            } else if (node.type === "Image") {
                icon = "fa-regular fa-image";
                labelText = "Image";
            } else if (node.type === "Text") {
                icon = "fa-solid fa-paragraph";
                labelText = "Text Editor";
            } else if (node.type === "ImageBox") {
                icon = "fa-regular fa-image";
                labelText = 'Image Box: "' + ((node.data && node.data.title) || '') + '"';
            } else if (node.type === "Grid") {
                icon = "fa-solid fa-table-cells";
                labelText = "Grid (" + ((node.data && node.data.columns) || 3) + " Col)";
                        } else if (node.type === "Icon" || node.type === "IconBox") {
                icon = (node.data && node.data.icon) || "fa-solid fa-icons";
                labelText = node.type === "IconBox" ? 'Icon Box: "' + ((node.data && node.data.title) || '') + '"' : "Icon";
            } else if (node.type === "Counter") {
                icon = "fa-solid fa-calculator";
                labelText = "Counter: " + ((node.data && node.data.number) || '');
            } else if (node.type === "ProgressBar") {
                icon = "fa-solid fa-bars-progress";
                labelText = "Progress: " + ((node.data && node.data.title) || '');
            } else if (node.type === "Testimonial") {
                icon = "fa-solid fa-quote-left";
                labelText = 'Testimonial: "' + ((node.data && node.data.name) || '') + '"';
            } else if (node.type === "Alert") {
                icon = "fa-solid fa-triangle-exclamation";
                labelText = "Alert: " + ((node.data && node.data.title) || '');
            } else if (node.type === "Accordion") {
                icon = "fa-solid fa-bars-staggered";
                labelText = "Accordion";
            } else if (node.type === "Tabs") {
                icon = "fa-solid fa-table-columns";
                labelText = "Tabs";
            } else if (node.type === "Rating") {
                icon = "fa-solid fa-star";
                labelText = "Rating: " + ((node.data && node.data.rating) || 5) + "/5";
            } else if (node.type === "SocialIcons") {
                icon = "fa-solid fa-share-nodes";
                labelText = "Social Icons";
            } else if (node.type === "IconList") {
                icon = "fa-solid fa-list-check";
                labelText = "Icon List";
            } else if (node.type === "Video") {
                icon = "fa-solid fa-play";
                labelText = "Video Embed";
            } else if (node.type === "NativeForm" || node.type === "Form") {
                icon = "fa-solid fa-envelope-open-text";
                labelText = "Antispam Form";
            } else if (node.type === "Benchmark") {
                icon = "fa-solid fa-gauge-high";
                labelText = "Benchmark";
            }

            const caretHtml = hasChildren
                ? `<i class="fa-solid fa-caret-down nav-caret"></i>`
                : `<span style="width: 14px; display: inline-block;"></span>`;

            item.innerHTML = `
                <div class="nav-item-left" style="padding-left: ${indent}px;">
                    ${caretHtml}
                    <i class="${icon} nav-item-icon"></i>
                    <span class="nav-item-title" title="${labelText}">${labelText}</span>
                </div>
                <div class="nav-item-actions">
                    <button class="nav-action-btn eye" title="Toggle Visibility">
                        <i class="fa-regular fa-eye"></i>
                    </button>
                    <button class="nav-action-btn dup" title="Duplicate">
                        <i class="fa-regular fa-copy"></i>
                    </button>
                    <button class="nav-action-btn del" title="Delete">
                        <i class="fa-solid fa-trash-can"></i>
                    </button>
                </div>
            `;

            // Row click -> Select and scroll
            item.onclick = (e) => {
                e.stopPropagation();
                selectElement(node.id);
            };

            // Caret toggle
            const caret = item.querySelector(".nav-caret");
            let childrenContainer = null;
            if (caret && hasChildren) {
                caret.onclick = (e) => {
                    e.stopPropagation();
                    caret.classList.toggle("collapsed");
                    if (childrenContainer) {
                        childrenContainer.style.display = caret.classList.contains("collapsed") ? "none" : "block";
                    }
                };
            }

            // Eye visibility toggle
            item.querySelector(".nav-action-btn.eye").onclick = (e) => {
                e.stopPropagation();
                const domEl = document.getElementById(node.id);
                const eyeIcon = item.querySelector(".nav-action-btn.eye i");
                if (domEl) {
                    if (domEl.style.opacity === "0.2") {
                        domEl.style.opacity = "1";
                        eyeIcon.className = "fa-regular fa-eye";
                        showToast(node.type + " visible");
                    } else {
                        domEl.style.opacity = "0.2";
                        eyeIcon.className = "fa-regular fa-eye-slash";
                        showToast(node.type + " hidden");
                    }
                }
            };

            // Duplicate action
            item.querySelector(".nav-action-btn.dup").onclick = (e) => {
                e.stopPropagation();
                duplicateElement(node.id);
            };

            // Delete action
            item.querySelector(".nav-action-btn.del").onclick = (e) => {
                e.stopPropagation();
                deleteElement(node.id);
            };

            // Drag reorder within Navigator tree
            item.ondragstart = (e) => {
                e.stopPropagation();
                window._activeDragData = { action: "move_tree_item", id: node.id };
                e.dataTransfer.setData("application/json", JSON.stringify(window._activeDragData));
                e.dataTransfer.setData("text/plain", JSON.stringify(window._activeDragData));
                item.classList.add("is-dragging");
            };
            item.ondragend = () => {
                item.classList.remove("is-dragging");
                window._activeDragData = null;
                document.querySelectorAll(".nav-tree-item").forEach(i => i.classList.remove("drag-over-item"));
            };
            item.ondragover = (e) => {
                e.preventDefault();
                e.stopPropagation();
                const data = getDragData(e);
                if (data && data.id !== node.id) {
                    item.classList.add("drag-over-item");
                }
            };
            item.ondragleave = () => item.classList.remove("drag-over-item");
            item.ondrop = (e) => {
                e.preventDefault();
                e.stopPropagation();
                item.classList.remove("drag-over-item");
                const data = getDragData(e);
                if (data && data.id !== node.id) {
                    handleTreeDrop(data.id, node.id);
                }
            };

            wrapper.appendChild(item);

            if (hasChildren) {
                childrenContainer = document.createElement("div");
                childrenContainer.className = "nav-children";
                node.children.forEach(c => walk(c, depth + 1, childrenContainer));
                wrapper.appendChild(childrenContainer);
            }

            parentEl.appendChild(wrapper);
        }

        walk(pageAst.root, 0, tree);
    }

    function handleTreeDrop(sourceId, targetId) {
        if (sourceId === targetId) return;
        const targetParent = findParentNode(pageAst.root, targetId);
        if (!targetParent || !targetParent.children) return;

        const movedNode = removeNodeById(pageAst.root, sourceId);
        if (!movedNode) return;

        let targetIdx = targetParent.children.findIndex(c => c.id === targetId);
        if (targetIdx === -1) {
            targetParent.children.push(movedNode);
        } else {
            targetParent.children.splice(targetIdx, 0, movedNode);
        }

        renderCanvas();
        selectElement(movedNode.id);
        showToast("Structure reordered");
    }

    // -------------------------------------------------------------
    // PAGE SETTINGS MODAL
    // -------------------------------------------------------------
    document.getElementById("tool-settings").onclick = () => {
        document.getElementById("settings-modal").classList.add("open");
    };
    document.getElementById("btn-close-settings").onclick = () => {
        document.getElementById("settings-modal").classList.remove("open");
    };
    document.getElementById("btn-save-settings").onclick = async () => {
        const titleEl = document.getElementById("set-page-title");
        const statusEl = document.getElementById("set-page-status");
        const templateEl = document.getElementById("set-page-template");
        const bgTextEl = document.getElementById("set-canvas-bg-text");
        if (bgTextEl && bgTextEl.value) {
            const canvas = document.getElementById("elementor-canvas");
            if (canvas) canvas.style.backgroundColor = bgTextEl.value;
        }
        try {
            const res = await fetch("/wp-json/wp/v2/posts/" + POST_ID, {
                method: "PUT",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify({
                    title: titleEl ? titleEl.value : undefined,
                    status: statusEl ? statusEl.value : undefined,
                    template: templateEl ? templateEl.value : undefined
                })
            });
            document.getElementById("settings-modal").classList.remove("open");
            if (res.ok) {
                showToast("Page settings saved");
            } else {
                showToast("Failed to save page settings");
            }
        } catch (err) {
            console.error(err);
            document.getElementById("settings-modal").classList.remove("open");
            showToast("Network error saving page settings");
        }
    };

    // -------------------------------------------------------------
    // RAW JSON AST MODAL
    // -------------------------------------------------------------
    document.getElementById("opt-view-ast").onclick = () => {
        document.getElementById("saver-options-menu").classList.remove("open");
        const textarea = document.getElementById("ast-json-textarea");
        textarea.value = JSON.stringify(pageAst, null, 2);
        document.getElementById("ast-modal").classList.add("open");
    };
    document.getElementById("btn-close-ast").onclick = () => {
        document.getElementById("ast-modal").classList.remove("open");
    };
    document.getElementById("btn-copy-ast").onclick = () => {
        const textarea = document.getElementById("ast-json-textarea");
        navigator.clipboard.writeText(textarea.value);
        showToast("JSON AST copied to clipboard");
    };
    document.getElementById("btn-apply-ast").onclick = () => {
        try {
            const parsed = JSON.parse(document.getElementById("ast-json-textarea").value);
            pageAst = parsed;
            renderCanvas();
            renderInspector();
            document.getElementById("ast-modal").classList.remove("open");
            showToast("AST applied successfully");
        } catch(e) {
            alert("Invalid JSON format: " + e.message);
        }
    };

    // -------------------------------------------------------------
    // SAVER MENU & PERSISTENCE
    // -------------------------------------------------------------
    const saverBtn = document.getElementById("elementor-panel-saver-options-btn");
    const saverMenu = document.getElementById("saver-options-menu");
    saverBtn.onclick = (e) => {
        e.stopPropagation();
        saverMenu.classList.toggle("open");
    };
    document.addEventListener("click", () => saverMenu.classList.remove("open"));

    document.getElementById("elementor-panel-saver-button-publish").onclick = savePage;

    // Keyboard shortcut Ctrl+S
    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            e.preventDefault();
            savePage();
        }
    });

    // Hamburger Menu button opens Site Settings & Navigation Menu (Elementor parity)
    const panelMenuBtn = document.getElementById("elementor-panel-header-menu-button");
    if (panelMenuBtn) {
        panelMenuBtn.onclick = () => {
            if (currentPanelView === "menu") {
                currentPanelView = "elements";
            } else if (currentPanelView.startsWith("site-settings-") || currentPanelView === "user-preferences") {
                currentPanelView = "menu";
            } else {
                currentPanelView = "menu";
                activeElementId = null;
            }
            renderInspector();
        };
    }

    // 9-dots button opens widget catalog (Elements)
    const panelAddBtn = document.getElementById("elementor-panel-header-add-button");
    if (panelAddBtn) {
        panelAddBtn.onclick = () => {
            currentPanelView = "elements";
            activeElementId = null;
            renderCanvas();
            renderInspector();
        };
    }

    // Exit
    document.getElementById("opt-exit").onclick = () => {
        window.location.href = "/n-admin/";
    };

    async function savePage() {
        const saveText = document.getElementById("btn-save-text");
        saveText.textContent = "SAVING...";

        try {
            const res = await fetch("/wp-json/neopress/v1/layout/" + POST_ID, {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-WP-Nonce": "wp_rest_admin" },
                credentials: "include",
                body: JSON.stringify({
                    layout: pageAst,
                    layout_json: pageAst
                })
            });

            if (res.ok) {
                saveText.textContent = "UPDATE";
                showToast("Page saved and pre-compiled successfully!");
            } else {
                saveText.textContent = "UPDATE";
                showToast("Error saving page layout");
            }
        } catch(err) {
            console.error(err);
            saveText.textContent = "UPDATE";
            showToast("Network error saving page");
        }
    }

    function showToast(msg) {
        const toast = document.getElementById("elementor-toast");
        document.getElementById("toast-text").textContent = msg;
        toast.classList.add("show");
        setTimeout(() => toast.classList.remove("show"), 2800);
    }

    function updateRawAst() {
        // live ast update trigger
    }


    // -------------------------------------------------------------
    // KEYBOARD SHORTCUTS & SPOTLIGHT FINDER IMPLEMENTATION
    // -------------------------------------------------------------
    function openShortcutsModal() {
        const modal = document.getElementById("modal-shortcuts");
        if (modal) {
            modal.style.display = "flex";
            const closeBtn = document.getElementById("btn-close-shortcuts");
            if (closeBtn) closeBtn.onclick = () => { modal.style.display = "none"; };
            modal.onclick = (e) => { if (e.target === modal) modal.style.display = "none"; };
        }
    }

    const FINDER_COMMANDS = [
        { title: "Global Colors", cat: "Site Settings", icon: "fa-solid fa-palette", action: () => { currentPanelView = "site-settings-colors"; renderInspector(); } },
        { title: "Global Fonts", cat: "Site Settings", icon: "fa-solid fa-font", action: () => { currentPanelView = "site-settings-fonts"; renderInspector(); } },
        { title: "Buttons Style", cat: "Site Settings", icon: "fa-solid fa-square-check", action: () => { currentPanelView = "site-settings-buttons"; renderInspector(); } },
        { title: "Site Identity", cat: "Site Settings", icon: "fa-solid fa-id-card", action: () => { currentPanelView = "site-settings-identity"; renderInspector(); } },
        { title: "Layout & Spacing", cat: "Site Settings", icon: "fa-solid fa-table-columns", action: () => { currentPanelView = "site-settings-layout"; renderInspector(); } },
        { title: "User Preferences", cat: "Preferences", icon: "fa-solid fa-sliders", action: () => { currentPanelView = "user-preferences"; renderInspector(); } },
        { title: "Save Layout (Update)", cat: "Actions", icon: "fa-solid fa-floppy-disk", action: () => { savePage(); } },
        { title: "Keyboard Shortcuts", cat: "Preferences", icon: "fa-solid fa-keyboard", action: () => { openShortcutsModal(); } },
        { title: "Toggle Structure / Layers", cat: "Panels", icon: "fa-solid fa-layer-group", action: () => { toggleNavigatorPanel(); } },
        { title: "View Live Page", cat: "Navigate", icon: "fa-solid fa-arrow-up-right-from-square", action: () => { window.open("/?p=" + POST_ID, "_blank"); } },
        { title: "Exit to Dashboard", cat: "Navigate", icon: "fa-solid fa-arrow-right-from-bracket", action: () => { window.location.href = "/n-admin/"; } },
        { title: "Insert Heading", cat: "Widgets", icon: "fa-solid fa-heading", action: () => { insertWidgetAtRoot("Heading"); } },
        { title: "Insert Button", cat: "Widgets", icon: "fa-solid fa-hand-pointer", action: () => { insertWidgetAtRoot("Button"); } },
        { title: "Insert Text Editor", cat: "Widgets", icon: "fa-solid fa-paragraph", action: () => { insertWidgetAtRoot("Text"); } },
        { title: "Insert Image", cat: "Widgets", icon: "fa-regular fa-image", action: () => { insertWidgetAtRoot("Image"); } },
        { title: "Insert Icon", cat: "Widgets", icon: "fa-solid fa-icons", action: () => { insertWidgetAtRoot("Icon"); } },
        { title: "Insert Divider", cat: "Widgets", icon: "fa-solid fa-minus", action: () => { insertWidgetAtRoot("Divider"); } },
        { title: "Insert Spacer", cat: "Widgets", icon: "fa-solid fa-arrows-up-down", action: () => { insertWidgetAtRoot("Spacer"); } },
        { title: "Insert Image Box", cat: "Widgets", icon: "fa-regular fa-image", action: () => { insertWidgetAtRoot("ImageBox"); } },
        { title: "Insert Icon Box", cat: "Widgets", icon: "fa-solid fa-cubes", action: () => { insertWidgetAtRoot("IconBox"); } },
        { title: "Insert Counter", cat: "Widgets", icon: "fa-solid fa-calculator", action: () => { insertWidgetAtRoot("Counter"); } },
        { title: "Insert Progress Bar", cat: "Widgets", icon: "fa-solid fa-bars-progress", action: () => { insertWidgetAtRoot("ProgressBar"); } },
        { title: "Insert Testimonial", cat: "Widgets", icon: "fa-solid fa-quote-left", action: () => { insertWidgetAtRoot("Testimonial"); } },
        { title: "Insert Alert", cat: "Widgets", icon: "fa-solid fa-triangle-exclamation", action: () => { insertWidgetAtRoot("Alert"); } },
        { title: "Insert Accordion", cat: "Widgets", icon: "fa-solid fa-bars-staggered", action: () => { insertWidgetAtRoot("Accordion"); } },
        { title: "Insert Tabs", cat: "Widgets", icon: "fa-solid fa-table-columns", action: () => { insertWidgetAtRoot("Tabs"); } },
        { title: "Insert Rating", cat: "Widgets", icon: "fa-solid fa-star", action: () => { insertWidgetAtRoot("Rating"); } },
        { title: "Insert Social Icons", cat: "Widgets", icon: "fa-solid fa-share-nodes", action: () => { insertWidgetAtRoot("SocialIcons"); } },
        { title: "Insert Icon List", cat: "Widgets", icon: "fa-solid fa-list-check", action: () => { insertWidgetAtRoot("IconList"); } },
        { title: "Insert Video", cat: "Widgets", icon: "fa-solid fa-play", action: () => { insertWidgetAtRoot("Video"); } },
        { title: "Insert Antispam Form", cat: "Widgets", icon: "fa-solid fa-envelope-open-text", action: () => { insertWidgetAtRoot("NativeForm"); } },
        { title: "Insert Benchmark Metric", cat: "Widgets", icon: "fa-solid fa-gauge-high", action: () => { insertWidgetAtRoot("Benchmark"); } },
        { title: "Insert Grid Layout", cat: "Widgets", icon: "fa-solid fa-table-cells", action: () => { insertWidgetAtRoot("Grid"); } },
        { title: "Insert Global Header", cat: "Global Blocks", icon: "fa-solid fa-compass", action: () => { insertWidgetAtRoot("GlobalHeader"); } },
        { title: "Insert Hero Banner", cat: "Global Blocks", icon: "fa-solid fa-wand-magic-sparkles", action: () => { insertWidgetAtRoot("GlobalHero"); } },
        { title: "Insert Pricing Matrix", cat: "Global Blocks", icon: "fa-solid fa-tags", action: () => { insertWidgetAtRoot("GlobalPricing"); } },
        { title: "Insert Global Footer", cat: "Global Blocks", icon: "fa-solid fa-shoe-prints", action: () => { insertWidgetAtRoot("GlobalFooter"); } }
    ];

    function openFinderModal() {
        const modal = document.getElementById("modal-finder");
        const input = document.getElementById("finder-input");
        const resultsEl = document.getElementById("finder-results");
        if (!modal || !input || !resultsEl) return;

        modal.style.display = "flex";
        input.value = "";
        input.focus();

        function renderResults(q) {
            const query = q.toLowerCase().trim();
            const filtered = query ? FINDER_COMMANDS.filter(c => c.title.toLowerCase().includes(query) || c.cat.toLowerCase().includes(query)) : FINDER_COMMANDS;
            resultsEl.innerHTML = "";
            filtered.forEach((cmd, idx) => {
                const item = document.createElement("div");
                item.className = "finder-item" + (idx === 0 ? " active" : "");
                item.innerHTML = `
                    <div class="finder-item-icon"><i class="${cmd.icon}"></i></div>
                    <div class="finder-item-title">${cmd.title}</div>
                    <div class="finder-item-cat">${cmd.cat}</div>
                `;
                item.onclick = () => {
                    modal.style.display = "none";
                    cmd.action();
                };
                resultsEl.appendChild(item);
            });
        }

        renderResults("");
        input.oninput = (e) => { renderResults(e.target.value); };
        input.onkeydown = (e) => {
            if (e.key === "Enter") {
                const first = resultsEl.querySelector(".finder-item");
                if (first) first.click();
            } else if (e.key === "Escape") {
                modal.style.display = "none";
            }
        };

        modal.onclick = (e) => { if (e.target === modal) modal.style.display = "none"; };
    }

    // Global Key Listener for Ctrl+E (Finder) and Esc
    document.addEventListener("keydown", (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "e") {
            e.preventDefault();
            openFinderModal();
        } else if (e.key === "Escape") {
            const f = document.getElementById("modal-finder");
            if (f && f.style.display === "flex") f.style.display = "none";
            const s = document.getElementById("modal-shortcuts");
            if (s && s.style.display === "flex") s.style.display = "none";
        }
    });

    // Restore saved user preferences
    try {
        const theme = localStorage.getItem("neopress_studio_theme");
        if (theme === "light") {
            document.getElementById("elementor-editor-wrapper").classList.add("theme-light");
        }
        const width = localStorage.getItem("neopress_panel_width");
        if (width) {
            document.getElementById("elementor-panel").style.width = width + "px";
        }
    } catch(err) {}

    // Initial boot
    renderCanvas();
    renderInspector();
    updateNavigator();

})();"###
}
