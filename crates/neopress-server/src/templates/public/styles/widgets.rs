//! Canvas & SSR Styles for Core and Pro Blocks (`templates/public/styles/widgets.rs`).
//!
//! Provides single-source-of-truth styles for all 26 commercial, dynamic,
//! and marketing Pro widgets with normalized class selectors for full visual parity
//! across both NeoStudio editor artboards and public SSR pages.

pub const CSS_WIDGETS: &str = r#"
/* Container & Layout Compatibility */
.np-section { width: 100%; position: relative; box-sizing: border-box; }
.np-container { width: 100%; max-width: 1200px; margin: 0 auto; position: relative; box-sizing: border-box; }
.elementor-column { display: flex; flex-direction: column; box-sizing: border-box; }

/* Commercial Pro Blocks: Price Card & Tables */
.np-price-card {
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    padding: 32px 28px;
    display: flex;
    flex-direction: column;
    position: relative;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.05);
}
.np-price-table {
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    padding: 32px 28px;
    display: flex;
    flex-direction: column;
    position: relative;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.05);
}
.np-price-card.featured, .np-price-card-featured {
    border-color: #2563eb;
    box-shadow: 0 8px 30px rgba(37, 99, 235, 0.18);
}
.price-card-badge, .np-price-badge {
    position: absolute;
    top: -12px;
    right: 20px;
    background: #2563eb;
    color: #ffffff;
    font-size: 10px;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: 20px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
}
.price-card-title, .np-price-title {
    font-size: 20px;
    font-weight: 700;
    color: #0f172a;
    margin-bottom: 6px;
}
.price-card-subtitle, .np-price-subtitle {
    font-size: 13px;
    color: #64748b;
    margin-bottom: 20px;
}
.price-card-pricing, .np-price-amount {
    display: flex;
    align-items: baseline;
    gap: 4px;
    margin-bottom: 24px;
}
.price-currency, .np-price-currency { font-size: 22px; font-weight: 600; color: #0f172a; }
.price-amount, .np-price-val { font-size: 44px; font-weight: 800; color: #0f172a; line-height: 1; }
.price-original, .np-price-original { font-size: 16px; color: #94a3b8; text-decoration: line-through; margin-left: 6px; }
.price-period, .np-price-period { font-size: 13px; color: #64748b; }
.price-features-list, .np-price-features {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 28px;
    flex: 1;
}
.price-feature-item, .np-price-feature {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    color: #334155;
}
.feature-icon, .np-price-feature i { color: #2563eb; font-size: 12px; }
.np-price-feature::before { content: "✓"; color: #10b981; font-weight: 800; margin-right: 4px; }
.price-feature-item.excluded { color: #94a3b8; text-decoration: line-through; }
.price-feature-item.excluded .feature-icon { color: #cbd5e1; }
.np-button.btn-primary, .np-button.np-price-btn, .np-price-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: #2563eb;
    color: #ffffff;
    font-size: 14px;
    font-weight: 600;
    padding: 12px 24px;
    border-radius: 8px;
    text-decoration: none;
    transition: all 0.15s ease;
    border: none;
    cursor: pointer;
}
.np-price-btn:hover { background: #1d4ed8; transform: translateY(-1px); }

/* Call to Action & Testimonials */
.np-cta {
    position: relative;
    border-radius: 12px;
    overflow: hidden;
    padding: 48px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: #111827;
    color: #ffffff;
    border: 1px solid #1e293b;
}
.np-cta.skin-cover { background-size: cover; background-position: center; color: #ffffff; }
.np-cta-title { font-size: 28px; font-weight: 800; color: #ffffff; line-height: 1.25; }
.np-cta-desc { font-size: 16px; color: #94a3b8; line-height: 1.6; }
.np-cta-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: #2563eb;
    color: #ffffff;
    font-weight: 600;
    padding: 12px 24px;
    border-radius: 8px;
    text-decoration: none;
    width: fit-content;
    transition: background 0.15s ease;
}
.np-cta-btn:hover { background: #1d4ed8; }
.np-testimonial, .np-testimonial-bubble {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    padding: 24px;
    position: relative;
    color: #1e293b;
}
.testimonial-author-row, .np-testimonial-meta { display: flex; align-items: center; gap: 12px; margin-top: 16px; }
.testimonial-avatar, .np-testimonial-avatar { width: 44px; height: 44px; border-radius: 50%; object-fit: cover; }
.np-testimonial-quote { font-size: 15px; line-height: 1.65; color: #334155; font-style: italic; }
.np-testimonial-author { font-weight: 700; font-size: 14px; color: #0f172a; }
.np-testimonial-role { font-size: 12.5px; color: #64748b; }

/* Countdown Timer */
.np-countdown { display: flex; gap: 16px; justify-content: center; flex-wrap: wrap; }
.np-countdown-wrapper { display: flex; gap: 16px; justify-content: center; flex-wrap: wrap; }
.countdown-box, .np-countdown-item {
    background: #0f172a;
    color: #ffffff;
    border-radius: 8px;
    padding: 16px 20px;
    min-width: 76px;
    display: flex;
    flex-direction: column;
    align-items: center;
    border: 1px solid #1e293b;
}
.countdown-digits, .np-countdown-digit {
    font-family: 'JetBrains Mono', monospace;
    font-size: 32px;
    font-weight: 700;
    line-height: 1;
    color: #38bdf8;
}
.countdown-unit, .np-countdown-label {
    font-size: 11px;
    color: #94a3b8;
    text-transform: uppercase;
    margin-top: 4px;
    letter-spacing: 0.05em;
}

/* Flip Box & Animated Headline */
.np-flipbox { perspective: 1000px; min-height: 240px; }
.flipbox-inner, .np-flipbox-inner {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 240px;
    transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1);
    transform-style: preserve-3d;
}
.np-flipbox:hover .flipbox-inner, .np-flipbox:hover .np-flipbox-inner { transform: rotateY(180deg); }
.flipbox-front, .flipbox-back, .np-flipbox-front, .np-flipbox-back {
    position: absolute;
    inset: 0;
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;
    border-radius: 12px;
    padding: 32px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    border: 1px solid #1e293b;
}
.flipbox-front, .np-flipbox-front { background: #111827; color: #ffffff; }
.flipbox-back, .np-flipbox-back { transform: rotateY(180deg); background: #15181e; color: #ffffff; }
.flipbox-title, .np-flipbox-title { font-size: 18px; font-weight: 700; margin-bottom: 8px; }
.flipbox-desc, .np-flipbox-desc { font-size: 13.5px; color: #94a3b8; line-height: 1.5; }

/* Progress Bar, Ratings & Alerts */
.np-progress-bar, .np-progressbar { width: 100%; }
.progress-track, .np-progress-track {
    width: 100%;
    height: 12px;
    background: #1e293b;
    border-radius: 6px;
    overflow: hidden;
}
.progress-fill, .np-progress-fill {
    height: 100%;
    background: #2563eb;
    border-radius: 6px;
    transition: width 0.4s ease;
}
.np-progress-title {
    font-size: 13.5px;
    font-weight: 600;
    color: #ffffff;
    margin-bottom: 8px;
    display: flex;
    justify-content: space-between;
}
.np-star-rating, .np-rating { display: flex; align-items: center; gap: 8px; }
.np-star-rating i, .np-rating i, .np-rating-stars i { color: #f59e0b; font-size: 18px; }
.np-rating-score { font-size: 13px; color: #94a3b8; font-weight: 600; }
.np-alert { padding: 14px 18px; border-radius: 8px; display: flex; align-items: flex-start; gap: 12px; }
.np-alert.info    { background: #eff6ff; border: 1px solid #bfdbfe; color: #1e40af; }
.np-alert.success { background: #f0fdf4; border: 1px solid #bbf7d0; color: #166534; }
.np-alert.warning { background: #fffbeb; border: 1px solid #fef3c7; color: #92400e; }
.np-alert.danger  { background: #fef2f2; border: 1px solid #fecaca; color: #991b1b; }

/* Dynamic & Theme Builder: Nav Menu */
.np-nav-menu { position: relative; }
.np-nav-menu, .np-nav-menu-list {
    display: flex;
    align-items: center;
    gap: 24px;
    list-style: none;
    margin: 0;
    padding: 0;
}
.np-menu-toggle { display: none; }
@media (max-width: 768px) {
    .np-menu-toggle {
        display: flex;
        flex-direction: column;
        gap: 4px;
        background: transparent;
        border: 1px solid #1e293b;
        border-radius: 4px;
        padding: 6px 8px;
        cursor: pointer;
    }
    .np-menu-toggle span {
        display: block;
        width: 18px;
        height: 2px;
        background: #ffffff;
    }
    .np-nav-menu-list {
        display: none;
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        background: #0b0f19;
        flex-direction: column;
        padding: 12px;
        gap: 12px;
        border: 1px solid #1e293b;
        border-radius: 6px;
        z-index: 999;
    }
    .np-nav-menu-list.is-open { display: flex; }
}
.np-nav-menu a, .np-nav-menu .np-nav-link, .np-nav-menu .np-nav-menu-link {
    text-decoration: none;
    color: inherit;
    font-weight: 500;
    font-size: 14px;
    transition: color 0.15s ease;
}
.np-nav-menu a:hover, .np-nav-menu .np-nav-link:hover, .np-nav-menu .np-nav-menu-link:hover {
    color: #2563eb;
}
"#;
