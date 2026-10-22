//! AST state normalization, ElementorKit global styles, and widget/icon catalog.

pub const SCRIPT_AST_AND_GLOBALS: &str = r###"(function() {
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

"###;
