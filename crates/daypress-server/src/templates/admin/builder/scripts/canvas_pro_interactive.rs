//! Interactive & Media Pro Blocks Canvas DOM Builders (`scripts/canvas_pro_interactive.rs`).

pub const JS_CANVAS_PRO_INTERACTIVE: &str = r###"
(function() {
    function renderProInteractiveWidget(node) {
        const type = node.type;
        const data = node.data || {};
        let el = document.createElement("div");

        switch(type) {
            case "Testimonial": {
                el.className = "dp-testimonial";
                el.style.cssText = "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 12px; padding: 28px; margin: 0; position: relative;";
                const quote = data.content || "Daypress dropped our TTFB from 350ms to 0.4ms overnight while cutting cloud hosting costs by 80%.";
                const name = data.name || "Sarah Chen";
                const job = data.job || "VP of Infrastructure, Acme Cloud";
                const img = data.image_url;
                const avatarHtml = img
                    ? `<img src="${img}" alt="${name}" class="dp-testimonial-avatar" style="width: 44px; height: 44px; border-radius: 50%; object-fit: cover;">`
                    : `<div style="width: 44px; height: 44px; border-radius: 50%; background: #e2e8f0; display: flex; align-items: center; justify-content: center; color: #64748b; font-weight: 700; font-size: 16px;">${name.charAt(0)}</div>`;
                el.innerHTML = `
                    <i class="fa-solid fa-quote-left" style="font-size: 24px; color: #cbd5e1; margin-bottom: 12px; display: block;"></i>
                    <p class="dp-testimonial-content" style="font-size: 15px; line-height: 1.6; color: #1e293b; margin: 0 0 18px 0; font-style: italic;">"${quote}"</p>
                    <div class="dp-testimonial-meta" style="display: flex; align-items: center; gap: 14px;">
                        ${avatarHtml}
                        <div>
                            <cite class="dp-testimonial-name" style="font-style: normal; font-weight: 700; font-size: 14px; color: #0f172a; display: block;">${name}</cite>
                            <span class="dp-testimonial-job" style="font-size: 12px; color: #64748b;">${job}</span>
                        </div>
                    </div>
                `;
                return el;
            }

            case "TestimonialCarousel": {
                el.className = "dp-testimonial-carousel dp-carousel-skin-" + (data.skin || "bubble");
                el.style.cssText = "position: relative; padding: 32px; background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px;";
                const slides = Array.isArray(data.slides) && data.slides.length > 0 ? data.slides : [
                    { quote: "Migrating our 20,000-page WooCommerce store took less than 2 minutes.", author: "Alex Rivera", role: "CTO", company: "FinTech Global" },
                    { quote: "Sub-millisecond page loads with zero PHP vulnerabilities. Simply unmatched in 2026.", author: "Elena Rostova", role: "Principal Architect", company: "CloudScale" }
                ];
                const activeSlide = slides[0];
                el.innerHTML = `
                    <div class="dp-carousel-track" style="margin-bottom: 20px;">
                        <blockquote class="dp-testimonial" style="margin: 0;">
                            <p class="dp-testimonial-content" style="font-size: 16px; line-height: 1.6; color: #1e293b; font-style: italic; margin-bottom: 16px;">"${activeSlide.quote}"</p>
                            <div class="dp-testimonial-meta" style="display: flex; align-items: center; gap: 12px;">
                                <div style="width: 40px; height: 40px; border-radius: 50%; background: #2563EB; color: #fff; display: flex; align-items: center; justify-content: center; font-weight: 700;">${(activeSlide.author || 'A').charAt(0)}</div>
                                <div>
                                    <cite class="dp-testimonial-name" style="font-style: normal; font-weight: 700; font-size: 14px; color: #0f172a; display: block;">${activeSlide.author}</cite>
                                    <span class="dp-testimonial-job" style="font-size: 12px; color: #64748b;">${activeSlide.role || ''}${activeSlide.company ? ' at ' + activeSlide.company : ''}</span>
                                </div>
                            </div>
                        </blockquote>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid #f1f5f9; padding-top: 16px;">
                        <div style="display: flex; gap: 6px;">
                            ${slides.map((_, i) => `<span style="width: 8px; height: 8px; border-radius: 50%; background: ${i === 0 ? 'var(--day-signal)' : '#cbd5e1'}; display: inline-block;"></span>`).join("")}
                        </div>
                        <div class="dp-carousel-arrows" style="display: flex; gap: 8px;">
                            <button type="button" style="width: 30px; height: 30px; border-radius: 50%; border: 1px solid #e2e8f0; background: #fff; cursor: pointer; display: flex; align-items: center; justify-content: center;"><i class="fa-solid fa-chevron-left" style="font-size: 11px;"></i></button>
                            <button type="button" style="width: 30px; height: 30px; border-radius: 50%; border: 1px solid #e2e8f0; background: #fff; cursor: pointer; display: flex; align-items: center; justify-content: center;"><i class="fa-solid fa-chevron-right" style="font-size: 11px;"></i></button>
                        </div>
                    </div>
                `;
                return el;
            }

            case "SocialIcons": {
                el.className = "dp-social-icons dp-social-skin-" + (data.skin || "default") + " dp-social-" + (data.color_type || "official");
                el.style.cssText = "display: flex; gap: 12px; align-items: center; padding: 12px 0;";
                const items = Array.isArray(data.items) && data.items.length > 0 ? data.items : [
                    { icon: "fa-brands fa-x-twitter", link: "https://x.com", color: "#000000" },
                    { icon: "fa-brands fa-github", link: "https://github.com", color: "#24292e" },
                    { icon: "fa-brands fa-linkedin-in", link: "https://linkedin.com", color: "#0a66c2" },
                    { icon: "fa-brands fa-discord", link: "https://discord.com", color: "#5865F2" }
                ];
                el.innerHTML = items.map(item => `
                    <a href="${item.link || '#'}" class="dp-social-icon" style="width: 38px; height: 38px; border-radius: 50%; background: ${item.color || 'var(--day-chassis)'}; color: #ffffff; display: flex; align-items: center; justify-content: center; text-decoration: none;">
                        <i class="${item.icon}" style="font-size: 16px;"></i>
                    </a>
                `).join("");
                return el;
            }

            case "ShareButtons": {
                el.className = "dp-share-buttons";
                el.style.cssText = "display: flex; gap: 10px; flex-wrap: wrap; padding: 12px 0;";
                const platforms = Array.isArray(data.platforms) && data.platforms.length > 0 ? data.platforms : [
                    { platform: "x", label: "Share on X", icon: "fa-brands fa-x-twitter", bg: "#000" },
                    { platform: "linkedin", label: "LinkedIn", icon: "fa-brands fa-linkedin-in", bg: "#0a66c2" },
                    { platform: "facebook", label: "Facebook", icon: "fa-brands fa-facebook-f", bg: "#1877f2" }
                ];
                el.innerHTML = platforms.map(p => `
                    <a href="#" class="dp-share-btn dp-share-network-${p.platform}" style="display: inline-flex; align-items: center; gap: 8px; background: ${p.bg || '#1e293b'}; color: #fff; padding: 8px 14px; border-radius: 6px; font-size: 12.5px; font-weight: 500; text-decoration: none;">
                        <i class="${p.icon || 'fa-solid fa-share-nodes'}"></i>
                        <span>${p.label || p.platform}</span>
                    </a>
                `).join("");
                return el;
            }

            case "Accordion": {
                el.className = "dp-accordion";
                el.style.cssText = "display: flex; flex-direction: column; gap: 10px; width: 100%;";
                const items = Array.isArray(data.items) && data.items.length > 0 ? data.items : [
                    { title: "What makes Daypress faster than standard WordPress?", content: "Daypress replaces the PHP-FPM runtime with Axum + Tokio, achieving sub-millisecond SSR directly from SQLite." },
                    { title: "Can I migrate existing Elementor & Divi themes?", content: "Yes! Daypress transpiles Elementor v2/v3/Pro and Divi 4/5 layouts into native AST with 100% style parity." },
                    { title: "Does DayStudio support custom CSS Grid & Flexbox?", content: "Yes, DayStudio provides a 2026 CAD visual interface with full control over CSS Grid tracks, flex layout, and box-model metrics." }
                ];
                el.innerHTML = items.map((item, idx) => `
                    <details class="dp-accordion-item" ${idx === 0 ? "open" : ""} style="background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden;">
                        <summary style="padding: 16px 20px; font-weight: 600; font-size: 14.5px; color: #0f172a; cursor: pointer; display: flex; justify-content: space-between; align-items: center; user-select: none;">
                            <span>${item.title}</span>
                            <i class="fa-solid fa-chevron-down" style="font-size: 12px; color: #64748b;"></i>
                        </summary>
                        <div class="dp-accordion-body" style="padding: 0 20px 18px 20px; font-size: 13.5px; line-height: 1.6; color: #475569;">
                            ${item.content || item.content_html || ""}
                        </div>
                    </details>
                `).join("");
                return el;
            }

            case "Tabs": {
                el.className = "dp-tabs";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 10px; overflow: hidden;";
                const items = Array.isArray(data.items) && data.items.length > 0 ? data.items : [
                    { title: "Overview", content: "High-performance CMS core built for sub-millisecond edge response times and zero maintenance overhead." },
                    { title: "Architecture", content: "Asynchronous Tokio task worker pool coupled with Rusqlite transactional dual-write storage." },
                    { title: "Benchmarks", content: "Over 120,000 requests per second with 0.38ms TTFB on standard commodity hardware." }
                ];
                el.innerHTML = `
                    <div class="dp-tabs-nav" style="display: flex; border-bottom: 1px solid #e2e8f0; background: #f8fafc;">
                        ${items.map((tab, idx) => `
                            <button type="button" class="dp-tab-btn ${idx === 0 ? 'active' : ''}" style="padding: 12px 20px; border: none; background: ${idx === 0 ? '#fff' : 'transparent'}; font-weight: 600; font-size: 13.5px; color: ${idx === 0 ? 'var(--day-signal)' : '#64748b'}; border-bottom: 2px solid ${idx === 0 ? 'var(--day-signal)' : 'transparent'}; cursor: pointer;">${tab.title}</button>
                        `).join("")}
                    </div>
                    <div class="dp-tabs-content" style="padding: 24px;">
                        ${items.map((tab, idx) => `
                            <div class="dp-tab-pane" style="display: ${idx === 0 ? 'block' : 'none'}; font-size: 14px; line-height: 1.6; color: #334155;">${tab.content || tab.content_html || ""}</div>
                        `).join("")}
                    </div>
                `;
                return el;
            }

            default:
                return null;
        }
    }
    window.renderProInteractiveWidget = renderProInteractiveWidget;
})();
"###;
