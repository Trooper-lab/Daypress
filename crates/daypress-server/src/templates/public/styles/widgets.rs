//! Canvas & SSR Styles for Core and Pro Blocks (`templates/public/styles/widgets.rs`).
//!
//! Provides single-source-of-truth styles for all 26 commercial, dynamic,
//! and marketing Pro widgets with normalized class selectors for full visual parity
//! across both DayStudio editor artboards and public SSR pages.

pub const CSS_WIDGETS: &str = r#"
/* Container & Layout Compatibility (imported designs, `.dp-design`, lay themselves out) */
.dp-section:not(:where(.dp-design, .dp-design *)) { width: 100%; position: relative; box-sizing: border-box; }
.dp-container:not(:where(.dp-design, .dp-design *)) { width: 100%; max-width: 1200px; margin: 0 auto; position: relative; box-sizing: border-box; }
.daystudio-column { display: flex; flex-direction: column; box-sizing: border-box; }

/* Commercial Pro Blocks: Price Card & Tables */
.dp-price-card {
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    padding: 32px 28px;
    display: flex;
    flex-direction: column;
    position: relative;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.05);
}
.dp-price-table {
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    padding: 32px 28px;
    display: flex;
    flex-direction: column;
    position: relative;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.05);
}
.dp-price-card.featured, .dp-price-card-featured {
    border-color: #2563eb;
    box-shadow: 0 8px 30px rgba(37, 99, 235, 0.18);
}
.price-card-badge, .dp-price-badge {
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
.price-card-title, .dp-price-title {
    font-size: 20px;
    font-weight: 700;
    color: #0f172a;
    margin-bottom: 6px;
}
.price-card-subtitle, .dp-price-subtitle {
    font-size: 13px;
    color: #64748b;
    margin-bottom: 20px;
}
.price-card-pricing, .dp-price-amount {
    display: flex;
    align-items: baseline;
    gap: 4px;
    margin-bottom: 24px;
}
.price-currency, .dp-price-currency { font-size: 22px; font-weight: 600; color: #0f172a; }
.price-amount, .dp-price-val { font-size: 44px; font-weight: 800; color: #0f172a; line-height: 1; }
.price-original, .dp-price-original { font-size: 16px; color: #94a3b8; text-decoration: line-through; margin-left: 6px; }
.price-period, .dp-price-period { font-size: 13px; color: #64748b; }
.price-features-list, .dp-price-features {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 28px;
    flex: 1;
}
.price-feature-item, .dp-price-feature {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    color: #334155;
}
.feature-icon, .dp-price-feature i { color: #2563eb; font-size: 12px; }
.dp-price-feature::before { content: "✓"; color: #10b981; font-weight: 800; margin-right: 4px; }
.price-feature-item.excluded { color: #94a3b8; text-decoration: line-through; }
.price-feature-item.excluded .feature-icon { color: #cbd5e1; }
.dp-button.btn-primary, .dp-button.dp-price-btn, .dp-price-btn {
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
.dp-price-btn:hover { background: #1d4ed8; transform: translateY(-1px); }

/* Call to Action & Testimonials */
.dp-cta {
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
.dp-cta.skin-cover { background-size: cover; background-position: center; color: #ffffff; }
.dp-cta-title { font-size: 28px; font-weight: 800; color: #ffffff; line-height: 1.25; }
.dp-cta-desc { font-size: 16px; color: #94a3b8; line-height: 1.6; }
.dp-cta-btn {
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
.dp-cta-btn:hover { background: #1d4ed8; }
.dp-testimonial, .dp-testimonial-bubble {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    padding: 24px;
    position: relative;
    color: #1e293b;
}
.testimonial-author-row, .dp-testimonial-meta { display: flex; align-items: center; gap: 12px; margin-top: 16px; }
.testimonial-avatar, .dp-testimonial-avatar { width: 44px; height: 44px; border-radius: 50%; object-fit: cover; }
.dp-testimonial-quote { font-size: 15px; line-height: 1.65; color: #334155; font-style: italic; }
.dp-testimonial-author { font-weight: 700; font-size: 14px; color: #0f172a; }
.dp-testimonial-role { font-size: 12.5px; color: #64748b; }

/* Countdown Timer */
.dp-countdown { display: flex; gap: 16px; justify-content: center; flex-wrap: wrap; }
.dp-countdown-wrapper { display: flex; gap: 16px; justify-content: center; flex-wrap: wrap; }
.countdown-box, .dp-countdown-item {
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
.countdown-digits, .dp-countdown-digit {
    font-family: 'JetBrains Mono', monospace;
    font-size: 32px;
    font-weight: 700;
    line-height: 1;
    color: #38bdf8;
}
.countdown-unit, .dp-countdown-label {
    font-size: 11px;
    color: #94a3b8;
    text-transform: uppercase;
    margin-top: 4px;
    letter-spacing: 0.05em;
}

/* Flip Box & Animated Headline */
.dp-flipbox { perspective: 1000px; min-height: 240px; }
.flipbox-inner, .dp-flipbox-inner {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 240px;
    transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1);
    transform-style: preserve-3d;
}
.dp-flipbox:hover .flipbox-inner, .dp-flipbox:hover .dp-flipbox-inner { transform: rotateY(180deg); }
.flipbox-front, .flipbox-back, .dp-flipbox-front, .dp-flipbox-back {
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
.flipbox-front, .dp-flipbox-front { background: #111827; color: #ffffff; }
.flipbox-back, .dp-flipbox-back { transform: rotateY(180deg); background: #15181e; color: #ffffff; }
.flipbox-title, .dp-flipbox-title { font-size: 18px; font-weight: 700; margin-bottom: 8px; }
.flipbox-desc, .dp-flipbox-desc { font-size: 13.5px; color: #94a3b8; line-height: 1.5; }

/* Progress Bar, Ratings & Alerts */
.dp-progress-bar, .dp-progressbar { width: 100%; }
.progress-track, .dp-progress-track {
    width: 100%;
    height: 12px;
    background: #1e293b;
    border-radius: 6px;
    overflow: hidden;
}
.progress-fill, .dp-progress-fill {
    height: 100%;
    background: #2563eb;
    border-radius: 6px;
    transition: width 0.4s ease;
}
.dp-progress-title {
    font-size: 13.5px;
    font-weight: 600;
    color: #ffffff;
    margin-bottom: 8px;
    display: flex;
    justify-content: space-between;
}
.dp-star-rating, .dp-rating { display: flex; align-items: center; gap: 8px; }
.dp-star-rating i, .dp-rating i, .dp-rating-stars i { color: #f59e0b; font-size: 18px; }
.dp-rating-score { font-size: 13px; color: #94a3b8; font-weight: 600; }
.dp-alert { padding: 14px 18px; border-radius: 8px; display: flex; align-items: flex-start; gap: 12px; }
.dp-alert.info    { background: #eff6ff; border: 1px solid #bfdbfe; color: #1e40af; }
.dp-alert.success { background: #f0fdf4; border: 1px solid #bbf7d0; color: #166534; }
.dp-alert.warning { background: #fffbeb; border: 1px solid #fef3c7; color: #92400e; }
.dp-alert.danger  { background: #fef2f2; border: 1px solid #fecaca; color: #991b1b; }

/* Dynamic & Theme Builder: Nav Menu & 2026 Mega Menu Dropdowns */
.dp-nav-menu { position: relative; }
.dp-nav-menu, .dp-nav-menu-list {
    display: flex;
    align-items: center;
    gap: 24px;
    list-style: none;
    margin: 0;
    padding: 0;
}
.dp-nav-item { position: relative; list-style: none; }
.dp-nav-item.menu-item-has-megamenu { position: static; }
.dp-nav-menu a, .dp-nav-menu .dp-nav-link, .dp-nav-menu .dp-nav-menu-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    text-decoration: none;
    color: inherit;
    font-weight: 500;
    font-size: 14px;
    padding: 6px 2px;
    transition: color 0.15s ease;
}
.dp-nav-menu a:hover, .dp-nav-menu .dp-nav-link:hover, .dp-nav-menu .dp-nav-menu-link:hover {
    color: #2563eb;
}
.dp-nav-icon { font-size: 13px; opacity: 0.85; }
.dp-nav-caret { font-size: 10px; opacity: 0.65; margin-left: 2px; transition: transform 0.2s ease; }
.dp-nav-item:hover > .dp-nav-link .dp-nav-caret,
.dp-nav-item:focus-within > .dp-nav-link .dp-nav-caret { transform: rotate(180deg); }
.dp-nav-badge {
    display: inline-flex;
    align-items: center;
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 2px 6px;
    border-radius: 999px;
    background: rgba(37, 99, 235, 0.14);
    color: #3b82f6;
    border: 1px solid rgba(59, 130, 246, 0.28);
    line-height: 1.2;
}
.dp-nav-desc {
    display: block;
    font-size: 11.5px;
    font-weight: 400;
    opacity: 0.65;
    margin-top: 2px;
}
.dp-megamenu-dropdown {
    position: absolute;
    top: calc(100% + 10px);
    left: 50%;
    transform: translateX(-50%) translateY(8px);
    width: min(980px, 92vw);
    background: #0f172a;
    color: #f8fafc;
    border: 1px solid rgba(148, 163, 184, 0.18);
    border-radius: 14px;
    box-shadow: 0 24px 60px rgba(2, 6, 23, 0.45), 0 0 0 1px rgba(255, 255, 255, 0.04) inset;
    padding: 24px;
    z-index: 9995;
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition: opacity 0.18s cubic-bezier(0.16, 1, 0.3, 1), transform 0.18s cubic-bezier(0.16, 1, 0.3, 1), visibility 0.18s;
    box-sizing: border-box;
    text-align: left;
}
.dp-megamenu-dropdown::before {
    content: "";
    position: absolute;
    top: -14px;
    left: 0;
    right: 0;
    height: 14px;
}
.dp-nav-item.menu-item-has-megamenu:hover > .dp-megamenu-dropdown,
.dp-nav-item.menu-item-has-megamenu:focus-within > .dp-megamenu-dropdown,
.dp-megamenu-dropdown.is-open {
    opacity: 1;
    visibility: visible;
    pointer-events: auto;
    transform: translateX(-50%) translateY(0);
}
.dp-menu-toggle { display: none; }
#daystudio-ssr-header .dp-menu-toggle { pointer-events: auto; }
#daystudio-artboard.artboard-mobile .dp-menu-toggle {
    display: inline-flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 4px;
    background: var(--dp-hamburger-bg, rgba(148, 163, 184, 0.12));
    color: var(--dp-hamburger-color, currentColor);
    border: 1px solid rgba(148, 163, 184, 0.28);
    border-radius: 6px;
    padding: 7px 9px;
    cursor: pointer;
}
#daystudio-artboard.artboard-mobile .dp-menu-toggle span {
    display: block;
    width: 18px;
    height: 2px;
    background: currentColor;
    border-radius: 2px;
}
#daystudio-artboard.artboard-mobile .dp-nav-menu-list {
    display: none;
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    background: var(--dp-mobile-menu-bg, #0f172a);
    color: var(--dp-mobile-menu-text, #f8fafc);
    flex-direction: column;
    align-items: stretch;
    padding: 12px;
    gap: 10px;
    border: 1px solid rgba(148, 163, 184, 0.22);
    border-radius: 8px;
    z-index: 999;
}
#daystudio-artboard.artboard-mobile .dp-nav-menu-list.is-open { display: flex; }
#daystudio-artboard.artboard-mobile header.day-header .header-container {
    flex-wrap: wrap;
    padding: 10px 14px;
}
#daystudio-artboard.artboard-mobile header.day-header nav.day-nav,
#daystudio-artboard.artboard-mobile header.day-header .header-actions {
    display: none;
    width: 100%;
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border, rgba(148, 163, 184, 0.2));
}
#daystudio-artboard.artboard-mobile header.day-header .header-container.is-mobile-open nav.day-nav,
#daystudio-artboard.artboard-mobile header.day-header .header-container.is-mobile-open .header-actions {
    display: flex;
}
@media (max-width: 768px) {
    .dp-menu-toggle {
        display: inline-flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        gap: 4px;
        background: var(--dp-hamburger-bg, rgba(148, 163, 184, 0.12));
        color: var(--dp-hamburger-color, currentColor);
        border: 1px solid rgba(148, 163, 184, 0.28);
        border-radius: 6px;
        padding: 7px 9px;
        cursor: pointer;
    }
    .dp-menu-toggle span {
        display: block;
        width: 18px;
        height: 2px;
        background: currentColor;
        border-radius: 2px;
    }
    header.day-header .header-container { flex-wrap: wrap; }
    header.day-header nav.day-nav,
    header.day-header .header-actions {
        display: none;
        width: 100%;
        flex-direction: column;
        align-items: stretch;
        gap: 10px;
        padding-top: 10px;
        border-top: 1px solid var(--border, rgba(148, 163, 184, 0.2));
    }
    header.day-header .header-container.is-mobile-open nav.day-nav,
    header.day-header .header-container.is-mobile-open .header-actions {
        display: flex;
    }
    .dp-nav-menu-list {
        display: none;
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        background: var(--dp-mobile-menu-bg, #0b0f19);
        color: var(--dp-mobile-menu-text, #f8fafc);
        flex-direction: column;
        align-items: stretch;
        padding: 12px;
        gap: 12px;
        border: 1px solid #1e293b;
        border-radius: 6px;
        z-index: 999;
    }
    .dp-nav-menu-list.is-open { display: flex; }
    .dp-megamenu-dropdown {
        position: static;
        transform: none !important;
        width: 100% !important;
        max-width: 100% !important;
        opacity: 1;
        visibility: visible;
        pointer-events: auto;
        margin-top: 8px;
        padding: 16px;
        box-shadow: none;
    }
}
"#;
