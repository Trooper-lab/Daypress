//! Commercial & Marketing Pro Blocks Canvas DOM Builders (`scripts/canvas_pro_commercial.rs`).

pub const JS_CANVAS_PRO_COMMERCIAL: &str = r###"
(function() {
    function renderProCommercialWidget(node) {
        const type = node.type;
        const data = node.data || {};
        let el = document.createElement("div");

        switch(type) {
            case "PriceCard": {
                el.className = "np-price-card" + (data.featured ? " featured" : "");
                if (data.badge) {
                    const badge = document.createElement("div");
                    badge.className = "price-card-badge";
                    badge.textContent = data.badge;
                    el.appendChild(badge);
                }
                const title = document.createElement("h3");
                title.className = "price-card-title";
                title.textContent = data.title || "Pro Cloud";
                el.appendChild(title);

                if (data.subtitle) {
                    const sub = document.createElement("p");
                    sub.className = "price-card-subtitle";
                    sub.textContent = data.subtitle;
                    el.appendChild(sub);
                }

                const priceRow = document.createElement("div");
                priceRow.className = "price-card-pricing";
                priceRow.innerHTML = `
                    <span class="price-currency">${data.currency || "$"}</span>
                    <span class="price-amount">${data.price || "49"}</span>
                    ${data.original_price ? `<span class="price-original">${data.original_price}</span>` : ""}
                    <span class="price-period">${data.period || "/mo"}</span>
                `;
                el.appendChild(priceRow);

                const list = document.createElement("ul");
                list.className = "price-features-list";
                const features = Array.isArray(data.features) && data.features.length > 0 ? data.features : [
                    { text: "Unlimited requests", included: true },
                    { text: "Sub-millisecond SSR", included: true },
                    { text: "Zero PHP CVEs", included: true }
                ];
                for (const f of features) {
                    const li = document.createElement("li");
                    const isInc = f.included !== false;
                    li.className = "price-feature-item" + (isInc ? "" : " excluded");
                    const iconClass = f.icon || (isInc ? "fa-solid fa-check" : "fa-solid fa-xmark");
                    li.innerHTML = `<i class="${iconClass} feature-icon"></i> <span>${f.text || f}</span>`;
                    list.appendChild(li);
                }
                el.appendChild(list);

                const btn = document.createElement("a");
                btn.className = "np-button btn-primary";
                btn.style.width = "100%";
                btn.style.backgroundColor = "var(--neo-signal)";
                btn.style.color = "#ffffff";
                btn.style.borderRadius = "6px";
                btn.style.padding = "10px 20px";
                btn.href = data.button_url || "#";
                btn.textContent = data.button_text || "Get Started";
                el.appendChild(btn);
                return el;
            }

            case "PriceTable": {
                el.className = "np-price-table";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 32px 24px; display: flex; flex-direction: column; text-align: center; box-shadow: 0 4px 16px rgba(0,0,0,0.04);";
                const title = data.title || "Business Tier";
                const price = data.price || "$79";
                const period = data.period || "/month";
                const btnText = data.button_text || "Choose Plan";
                const btnUrl = data.button_url || "#";
                const features = Array.isArray(data.features) && data.features.length > 0 ? data.features : [
                    "High-throughput Rust runtime",
                    "Multi-region replication",
                    "24/7 dedicated engineering support",
                    "Zero PHP maintenance overhead"
                ];
                el.innerHTML = `
                    <div class="np-price-header" style="margin-bottom: 16px;">
                        <h3 class="np-price-title" style="margin: 0; font-size: 20px; font-weight: 700; color: #0f172a;">${title}</h3>
                    </div>
                    <div class="np-price-amount" style="margin-bottom: 24px;">
                        <span class="np-price-val" style="font-size: 38px; font-weight: 800; color: #0f172a; font-family: var(--neo-font-ui);">${price}</span>
                        <span class="np-price-period" style="font-size: 13px; color: #64748b;">${period}</span>
                    </div>
                    <ul class="np-price-features" style="list-style: none; padding: 0; margin: 0 0 24px 0; display: flex; flex-direction: column; gap: 10px; text-align: left;">
                        ${features.map(f => {
                            const text = typeof f === "object" ? (f.text || "") : f;
                            return `<li class="np-price-feature" style="font-size: 13.5px; color: #334155; display: flex; align-items: center; gap: 8px;">
                                <i class="fa-solid fa-check" style="color: var(--neo-signal); font-size: 12px;"></i>
                                <span>${text}</span>
                            </li>`;
                        }).join("")}
                    </ul>
                    <a href="${btnUrl}" class="np-button np-price-btn" style="background: var(--neo-signal); color: #fff; padding: 10px 20px; border-radius: 6px; font-weight: 600; text-decoration: none; display: block; margin-top: auto;">${btnText}</a>
                `;
                return el;
            }

            case "CallToAction": {
                el.className = "np-cta " + (data.skin === "cover" ? "skin-cover" : "skin-classic");
                el.style.backgroundColor = data.skin === "cover" ? "#1e293b" : "#f1f5f9";
                if (data.image_url) el.style.backgroundImage = `url(${data.image_url})`;
                el.innerHTML = `
                    <h2 style="font-size: 28px; font-weight: 700; margin: 0;">${data.title || "Ready to Accelerate?"}</h2>
                    <p style="font-size: 15px; margin: 0; opacity: 0.85;">${data.description || "Deploy with native Tokio async speed."}</p>
                    <div style="display: flex; gap: 12px; margin-top: 8px;">
                        <a href="${data.button_url || '#'}" class="np-button" style="background: var(--neo-signal); color: #fff; padding: 10px 20px; border-radius: 6px;">${data.button_text || "Start Now"}</a>
                        ${data.secondary_button_text ? `<a href="${data.secondary_button_url || '#'}" class="np-button" style="background: transparent; border: 1px solid currentColor; padding: 10px 20px; border-radius: 6px;">${data.secondary_button_text}</a>` : ""}
                    </div>
                `;
                return el;
            }

            case "Countdown": {
                el.className = "np-countdown";
                el.innerHTML = `
                    <div class="countdown-box"><span class="countdown-digits">04</span><span class="countdown-unit">Days</span></div>
                    <div class="countdown-box"><span class="countdown-digits">18</span><span class="countdown-unit">Hours</span></div>
                    <div class="countdown-box"><span class="countdown-digits">32</span><span class="countdown-unit">Mins</span></div>
                    <div class="countdown-box"><span class="countdown-digits">45</span><span class="countdown-unit">Secs</span></div>
                `;
                return el;
            }

            case "FlipBox": {
                el.className = "np-flipbox";
                el.innerHTML = `
                    <div class="flipbox-inner">
                        <div class="flipbox-front" style="background: #f8fafc; border: 1px solid #e2e8f0;">
                            <i class="${data.front_icon || 'fa-solid fa-server'}" style="font-size: 28px; color: var(--neo-signal); margin-bottom: 12px;"></i>
                            <h3 style="margin: 0 0 6px 0;">${data.front_title || "Front Face"}</h3>
                            <p style="font-size: 13px; color: #64748b; margin: 0;">${data.front_description || "Hover to inspect details."}</p>
                        </div>
                        <div class="flipbox-back">
                            <h3 style="margin: 0 0 6px 0;">${data.back_title || "Back Face"}</h3>
                            <p style="font-size: 13px; opacity: 0.85; margin: 0 0 16px 0;">${data.back_description || "High-performance architecture."}</p>
                            <a href="${data.button_url || '#'}" class="np-button" style="background: #fff; color: var(--neo-chassis); padding: 8px 16px; border-radius: 4px;">${data.button_text || "Learn More"}</a>
                        </div>
                    </div>
                `;
                return el;
            }

            case "AnimatedHeadline": {
                el.className = "np-animated-headline";
                const words = Array.isArray(data.words) && data.words.length > 0 ? data.words : ["Fastest", "Safest", "Modern"];
                el.innerHTML = `
                    <h2 style="font-size: 32px; font-weight: 700; margin: 0; text-align: center;">
                        <span>${data.before_text || "Build "}</span>
                        <span style="color: var(--neo-signal); text-decoration: underline;">${words[0]}</span>
                        <span>${data.after_text || " Websites"}</span>
                    </h2>
                `;
                return el;
            }

            case "StarRating":
            case "Rating": {
                el.className = "np-star-rating";
                const rating = parseFloat(data.rating) || 4.9;
                el.style.display = "flex";
                el.style.alignItems = "center";
                el.style.gap = "8px";
                let stars = "";
                for (let i = 0; i < 5; i++) {
                    stars += `<i class="fa-solid fa-star" style="color: ${data.star_color || '#F59E0B'}; font-size: ${data.star_size || 16}px;"></i>`;
                }
                el.innerHTML = `<div>${stars}</div><span style="font-weight: 600; font-size: 13px;">${rating} / 5</span>`;
                return el;
            }

            case "ProgressBar": {
                el.className = "np-progress-bar";
                const pct = data.percent || 95;
                el.innerHTML = `
                    <div style="display: flex; justify-content: space-between; margin-bottom: 6px; font-size: 13px; font-weight: 600;">
                        <span>${data.title || "Engine Performance"}</span>
                        <span>${pct}%</span>
                    </div>
                    <div class="progress-track" style="height: ${data.height || 12}px;">
                        <div class="progress-fill" style="width: ${pct}%; background: ${data.bar_color || 'var(--neo-signal)'};"></div>
                    </div>
                `;
                return el;
            }

            case "Alert": {
                const aType = data.alert_type || "info";
                el.className = `np-alert ${aType}`;
                el.innerHTML = `
                    <i class="${data.icon || 'fa-solid fa-circle-info'}" style="font-size: 18px; margin-top: 2px;"></i>
                    <div style="flex: 1;">
                        <strong style="display: block; font-size: 14px; margin-bottom: 2px;">${data.title || "Notice"}</strong>
                        <span style="font-size: 13px;">${data.description || "Neopress native transaction completed."}</span>
                    </div>
                `;
                return el;
            }

            case "Counter": {
                el.className = "np-counter";
                el.style.cssText = "padding: 24px; text-align: center; background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px;";
                const pfx = data.prefix || "";
                const sfx = data.suffix || "%";
                const num = data.number !== undefined ? data.number : "99.9";
                const title = data.title || "Uptime SLA";
                el.innerHTML = `
                    <div class="np-counter-val" style="font-size: 42px; font-weight: 800; color: #0f172a; font-family: var(--neo-font-mono); line-height: 1.1;">
                        <span class="counter-prefix" style="color: var(--neo-signal);">${pfx}</span><span class="counter-number">${num}</span><span class="counter-suffix" style="color: var(--neo-signal);">${sfx}</span>
                    </div>
                    <div class="np-counter-title" style="font-size: 13px; font-weight: 600; color: #64748b; margin-top: 8px; text-transform: uppercase; letter-spacing: 0.05em;">${title}</div>
                `;
                return el;
            }

            case "Benchmark": {
                el.className = "np-benchmark";
                el.style.cssText = "padding: 24px; background: #0D0F12; color: #F2F4F8; border: 1px solid #242933; border-radius: 10px; font-family: var(--neo-font-ui); box-shadow: 0 4px 20px rgba(0,0,0,0.25);";
                const label = data.label || "Throughput (Req/Sec)";
                const value = data.value || "128,450 req/s";
                const comp = data.comparison || "46x faster than PHP 8.3 FPM (2,780 req/s)";
                el.innerHTML = `
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                        <div class="np-benchmark-label" style="font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: var(--neo-muted);">${label}</div>
                        <span style="font-size: 10px; font-weight: 700; background: rgba(37,99,235,0.2); color: #60a5fa; border: 1px solid #2563eb; padding: 2px 8px; border-radius: 4px; font-family: var(--neo-font-mono);">0.38ms TTFB</span>
                    </div>
                    <div class="np-benchmark-value" style="font-size: 32px; font-weight: 800; color: #F2F4F8; font-family: var(--neo-font-mono); margin-bottom: 12px; letter-spacing: -0.02em;">${value}</div>
                    <div style="height: 6px; background: #242933; border-radius: 3px; overflow: hidden; margin-bottom: 12px; display: flex;">
                        <div style="width: 96%; background: var(--neo-signal); height: 100%; border-radius: 3px;"></div>
                        <div style="width: 4%; background: #475569; height: 100%;"></div>
                    </div>
                    <div class="np-benchmark-comparison" style="font-size: 12px; color: #94a3b8; display: flex; align-items: center; gap: 6px;">
                        <i class="fa-solid fa-bolt" style="color: #F59E0B;"></i>
                        <span>${comp}</span>
                    </div>
                `;
                return el;
            }

            default:
                return null;
        }
    }
    window.renderProCommercialWidget = renderProCommercialWidget;
})();
"###;
