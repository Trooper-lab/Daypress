//! WYSIWYG DOM Dispatcher for 26 Elementor Pro and Divi Pro Blocks (`scripts/canvas_pro_widgets.rs`).

pub const JS_CANVAS_PRO_WIDGETS: &str = r###"
(function() {
    const PRO_WIDGET_TYPES = new Set([
        "PriceCard", "PriceTable", "CallToAction", "Testimonial", "TestimonialCarousel",
        "Countdown", "FlipBox", "AnimatedHeadline", "StarRating", "Rating", "ProgressBar",
        "Alert", "Counter", "Benchmark", "SocialIcons", "ShareButtons",
        "NavMenu", "MegaMenuContainer", "PopupModal", "LoopGrid", "TableOfContents",
        "Breadcrumbs", "SearchForm", "LoginForm", "NativeForm", "Form", "Accordion", "Tabs"
    ]);

    function isProWidget(type) {
        return PRO_WIDGET_TYPES.has(type);
    }
    window.isProWidget = isProWidget;

    function renderProWidget(node) {
        const type = node.type;

        // 1. Try Commercial & Marketing builder
        if (window.renderProCommercialWidget) {
            const el = window.renderProCommercialWidget(node);
            if (el) return el;
        }

        // 2. Try Interactive & Media builder
        if (window.renderProInteractiveWidget) {
            const el = window.renderProInteractiveWidget(node);
            if (el) return el;
        }

        // 3. Try Dynamic & Theme/Form builder
        if (window.renderProDynamicWidget) {
            const el = window.renderProDynamicWidget(node);
            if (el) return el;
        }

        // 4. Absolute Fallback (Only hit for genuinely unknown custom types)
        let el = document.createElement("div");
        el.className = "dp-pro-block";
        el.style.padding = "24px";
        el.style.border = "1px dashed var(--day-rule)";
        el.innerHTML = `<i class="fa-solid fa-cube"></i> <span>${type}</span>`;
        return el;
    }
    window.renderProWidget = renderProWidget;
})();
"###;
