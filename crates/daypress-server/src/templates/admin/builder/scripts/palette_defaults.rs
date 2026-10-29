//! Starter content defaults dictionary for Palette & Command Palette block insertion (`scripts/palette_defaults.rs`).
//!
//! Provides rich, production-grade starter content defaults for all 26 Pro block types
//! and core primitives when inserted onto the canvas, ensuring both builder canvas fidelity
//! and public SSR rendering without unrendered blanks or broken layouts.

pub const JS_PALETTE_DEFAULTS: &str = r###"
(function() {
    function getFutureDateIso(days) {
        const d = new Date(Date.now() + (days || 30) * 24 * 60 * 60 * 1000);
        const pad = n => String(n).padStart(2, "0");
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
    }

    const STARTER_DEFAULTS = {
        // Layout Primitives
        Heading: { text: "New Section Heading", level: 2 },
        Text: { content_html: "<p>Start writing high-performance web copy here...</p>" },
        Button: { text: "Get Started", url: "#", target: "_self" },
        Image: { src: "https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=800&auto=format&fit=crop", alt: "Modern high-speed infrastructure" },
        Video: { url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ", aspect_ratio: "16:9", autoplay: false, muted: true, loop_video: false },
        Spacer: { height: 40 },
        Divider: { style: "solid" },
        Icon: { name: "fa-solid fa-bolt", size: 24, color: "#2563EB" },

        // Commercial & Marketing Pro Blocks
        PriceCard: {
            title: "Pro Cloud",
            subtitle: "High-throughput Rust runtime for mission-critical web applications",
            currency: "$",
            price: "49",
            original_price: "79",
            period: "month",
            badge: "MOST POPULAR",
            featured: true,
            features: [
                { text: "Sub-millisecond Rust SSR", included: true, icon: "fa-check" },
                { text: "Unlimited Team Seats", included: true, icon: "fa-check" },
                { text: "Priority Support & Zero CVEs", included: true, icon: "fa-check" },
                { text: "Transactional SQLite Dual-Write", included: true, icon: "fa-check" }
            ],
            button_text: "Get Started",
            button_url: "#signup"
        },

        PriceTable: {
            title: "Enterprise Dedicated",
            price: "$99",
            period: "/month",
            features: [
                "High-throughput Tokio runtime",
                "Transactional SQLite engine",
                "24/7 dedicated engineering support",
                "Zero PHP CVE vulnerability surface"
            ],
            button_text: "Choose Enterprise",
            button_url: "#contact"
        },

        CallToAction: {
            title: "Ready to Modernize Your Stack?",
            description: "Deploy with native Tokio async speed and sub-millisecond Rust SSR.",
            button_text: "Start Free Trial",
            button_url: "#signup",
            secondary_button_text: "Read Documentation",
            secondary_button_url: "/docs",
            skin: "cover",
            ribbon: "NEW RELEASE"
        },

        Countdown: {
            target_date: "DYNAMIC",
            mode: "due_date",
            show_days: true,
            show_hours: true,
            show_minutes: true,
            show_seconds: true,
            expiry_message: "Offer Expired"
        },

        FlipBox: {
            front_icon: "fa-solid fa-cube",
            front_title: "Interactive 3D",
            front_description: "Card hover transitions powered by hardware-accelerated CSS transforms.",
            back_title: "Turnkey Architecture",
            back_description: "Engineered with native Axum and Rusqlite for zero-latency execution.",
            button_text: "Explore Architecture",
            button_url: "#features",
            direction: "flip_left"
        },

        AnimatedHeadline: {
            before_text: "Build",
            words: ["Ultra-Fast", "Zero-CVE", "Modern"],
            after_text: "Websites",
            animation_style: "rotate"
        },

        Testimonial: {
            content: "Daypress dropped our TTFB from 350ms to 0.4ms overnight while cutting cloud hosting costs by 80%.",
            name: "Sarah Chen",
            job: "VP of Infrastructure, Acme Cloud",
            image_url: ""
        },

        TestimonialCarousel: {
            skin: "bubble",
            autoplay: true,
            loop_slides: true,
            slides: [
                { quote: "Migrating our 20,000-page WooCommerce store took less than 2 minutes.", author: "Alex Rivera", role: "CTO", company: "FinTech Global" },
                { quote: "Sub-millisecond page loads with zero PHP vulnerabilities. Simply unmatched in 2026.", author: "Elena Rostova", role: "Principal Architect", company: "CloudScale" }
            ]
        },

        StarRating: {
            title: "Customer Rating",
            rating: 5.0,
            max_rating: 5,
            star_color: "#F59E0B",
            star_size: 18
        },

        Rating: {
            title: "Performance Score",
            rating: 5.0,
            max_rating: 5
        },

        ProgressBar: {
            title: "Compilation Progress",
            percent: 98,
            display_text: "98%",
            bar_color: "#2563EB",
            track_color: "#242933",
            height: 12
        },

        Alert: {
            alert_type: "info",
            title: "System Performance Notice",
            description: "Native Axum worker pool operating at peak efficiency with sub-millisecond response times.",
            dismissible: true,
            icon: "fa-solid fa-circle-info"
        },

        Benchmark: {
            label: "Throughput (Req/Sec)",
            value: "128,450 req/s",
            comparison: "46x faster than PHP 8.3 FPM (2,780 req/s)"
        },

        Counter: {
            number: 99,
            prefix: "",
            suffix: ".9%",
            title: "Uptime SLA Guarantee"
        },

        SocialIcons: {
            skin: "default",
            color_type: "official",
            items: [
                { icon: "fa-brands fa-x-twitter", link: "https://x.com" },
                { icon: "fa-brands fa-github", link: "https://github.com" },
                { icon: "fa-brands fa-linkedin-in", link: "https://linkedin.com" },
                { icon: "fa-brands fa-discord", link: "https://discord.com" }
            ]
        },

        ShareButtons: {
            skin: "solid",
            official_colors: true,
            platforms: [
                { platform: "x", label: "Share on X" },
                { platform: "linkedin", label: "LinkedIn" },
                { platform: "facebook", label: "Facebook" }
            ]
        },

        // Dynamic & Theme Pro Blocks
        NavMenu: {
            menu_slug: "main-menu",
            layout: "horizontal",
            align: "left",
            mobile_hamburger: true,
            items: [
                { title: "Home", url: "/", order: 1 },
                { title: "Solutions", url: "/solutions", order: 2 },
                { title: "Pricing", url: "/pricing", order: 3 },
                { title: "About", url: "/about", order: 4 }
            ]
        },

        LoopGrid: {
            post_type: "post",
            posts_per_page: 3,
            columns: 3,
            show_thumbnail: true,
            show_excerpt: true,
            show_meta: true,
            show_pagination: false
        },

        TableOfContents: {
            title: "Table of Contents",
            min_depth: 2,
            max_depth: 4,
            collapsible: true
        },

        Breadcrumbs: {
            separator: ">",
            home_text: "Home"
        },

        SearchForm: {
            placeholder: "Search articles, guides, docs...",
            button_text: "Search",
            endpoint: "/"
        },

        LoginForm: {
            label_username: "Username or Email",
            label_password: "Password",
            redirect_url: "/wp-admin/",
            remember_me: true
        },

        NativeForm: {
            title: "Contact Engineering",
            button_text: "Send Message",
            endpoint: "/api/contact"
        },

        Form: {
            title: "Get in Touch",
            button_text: "Submit Inquiry",
            endpoint: "/api/contact"
        },

        MegaMenuContainer: {
            trigger: "Products Menu",
            width: "100%"
        },

        PopupModal: {
            modal_id: "promo-modal",
            trigger_selector: ".open-promo-btn",
            overlay_bg: "rgba(0,0,0,0.6)",
            close_button: true
        },

        Accordion: {
            items: [
                {
                    title: "What makes Daypress faster than standard WordPress?",
                    content_html: "Daypress replaces the PHP-FPM runtime with Axum + Tokio, achieving sub-millisecond SSR directly from SQLite."
                },
                {
                    title: "Can I migrate existing Elementor & Divi themes?",
                    content_html: "Yes! Daypress transpiles Elementor v2/v3/Pro and Divi 4/5 layouts into native AST with 100% style parity."
                },
                {
                    title: "Does DayStudio support custom CSS Grid & Flexbox?",
                    content_html: "Yes, DayStudio provides a 2026 CAD visual interface with full control over CSS Grid tracks, flex layout, and box-model metrics."
                }
            ]
        },

        Tabs: {
            items: [
                {
                    title: "Overview",
                    content_html: "High-performance CMS core built for sub-millisecond edge response times and zero maintenance overhead."
                },
                {
                    title: "Architecture",
                    content_html: "Asynchronous Tokio task worker pool coupled with Rusqlite transactional dual-write storage."
                },
                {
                    title: "Benchmarks",
                    content_html: "Over 120,000 requests per second with 0.38ms TTFB on standard commodity hardware."
                }
            ]
        }
    };

    function getWidgetStarterDefaults(type) {
        if (!type || !STARTER_DEFAULTS[type]) return {};
        const clone = JSON.parse(JSON.stringify(STARTER_DEFAULTS[type]));
        if (type === "Countdown" && clone.target_date === "DYNAMIC") {
            clone.target_date = getFutureDateIso(30);
        }
        return clone;
    }

    window.getWidgetStarterDefaults = getWidgetStarterDefaults;
})();
"###;
