//! Dynamic & Theme/Form Pro Blocks Canvas DOM Builders (`scripts/canvas_pro_dynamic.rs`).

pub const JS_CANVAS_PRO_DYNAMIC: &str = r###"
(function() {
    function renderProDynamicWidget(node) {
        const type = node.type;
        const data = node.data || {};
        let el = document.createElement("div");

        switch(type) {
            case "NavMenu": {
                el.className = "np-nav-menu-wrapper";
                el.innerHTML = `
                    <nav class="np-nav-menu">
                        <a href="/" class="active">Home</a>
                        <a href="/pricing">Pricing</a>
                        <a href="/docs">Documentation</a>
                        <a href="/blog">Blog</a>
                        <a href="/contact">Contact</a>
                    </nav>
                `;
                return el;
            }

            case "LoopGrid": {
                el.className = "np-loop-grid";
                el.style.display = "grid";
                el.style.gridTemplateColumns = `repeat(${data.columns || 3}, 1fr)`;
                el.style.gap = "20px";
                for (let i = 1; i <= (data.posts_per_page || 3); i++) {
                    const postCard = document.createElement("div");
                    postCard.style.cssText = "background:#fff; border:1px solid #e2e8f0; border-radius:8px; overflow:hidden;";
                    postCard.innerHTML = `
                        <div style="height: 140px; background: #e2e8f0; display:flex; align-items:center; justify-content:center; color:#94a3b8;"><i class="fa-solid fa-image fa-2x"></i></div>
                        <div style="padding: 16px;">
                            <span style="font-size: 11px; color: var(--neo-signal); font-weight:600;">ARTICLE ${i}</span>
                            <h4 style="margin: 4px 0 8px 0; font-size: 16px;">Fastest CMS Architecture in 2026</h4>
                            <p style="font-size: 12px; color: #64748b; margin: 0;">Learn how Rust replaces legacy PHP monoliths.</p>
                        </div>
                    `;
                    el.appendChild(postCard);
                }
                return el;
            }

            case "Form":
            case "NativeForm": {
                el.className = "np-form";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 32px; max-width: 600px;";
                const title = data.title || "Get in Touch";
                const btn = data.button_text || "Submit Inquiry";
                el.innerHTML = `
                    <h3 class="np-form-title" style="margin: 0 0 20px 0; font-size: 20px; font-weight: 700; color: #0f172a;">${title}</h3>
                    <div class="np-form-fields" style="display: flex; flex-direction: column; gap: 14px;">
                        <div>
                            <label style="display: block; font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 5px;">Your Name</label>
                            <input type="text" class="np-input" placeholder="Alex Morgan" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        <div>
                            <label style="display: block; font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 5px;">Work Email</label>
                            <input type="email" class="np-input" placeholder="alex@company.com" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        <div>
                            <label style="display: block; font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 5px;">Message</label>
                            <textarea class="np-textarea" rows="3" placeholder="Tell us about your project requirements..." disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc; resize: none;"></textarea>
                        </div>
                        <button type="button" class="np-button np-form-submit" style="background: var(--neo-signal); color: #fff; padding: 12px 24px; border: none; border-radius: 6px; font-weight: 600; font-size: 14px; cursor: default; align-self: flex-start; margin-top: 4px;">${btn}</button>
                    </div>
                `;
                return el;
            }

            case "LoginForm": {
                el.className = "np-login-form";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 32px; max-width: 420px; box-shadow: 0 4px 16px rgba(0,0,0,0.04);";
                const labelUser = data.label_username || "Username or Email";
                const labelPass = data.label_password || "Password";
                const showRemember = data.remember_me !== false;
                el.innerHTML = `
                    <div style="text-align: center; margin-bottom: 24px;">
                        <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(37,99,235,0.1); color: var(--neo-signal); display: inline-flex; align-items: center; justify-content: center; font-size: 20px; margin-bottom: 12px;">
                            <i class="fa-solid fa-lock"></i>
                        </div>
                        <h3 style="margin: 0; font-size: 20px; font-weight: 700; color: #0f172a;">Sign In</h3>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 14px;">
                        <div class="np-form-field">
                            <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">${labelUser}</label>
                            <input type="text" class="np-input" placeholder="admin" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        <div class="np-form-field">
                            <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">${labelPass}</label>
                            <input type="password" class="np-input" value="••••••••" disabled style="width: 100%; box-sizing: border-box; padding: 10px 14px; border: 1px solid #cbd5e1; border-radius: 6px; font-size: 14px; background: #f8fafc;">
                        </div>
                        ${showRemember ? `
                            <label class="np-login-remember" style="display: flex; align-items: center; gap: 8px; font-size: 13px; color: #64748b; cursor: default;">
                                <input type="checkbox" checked disabled> Remember Me
                            </label>
                        ` : ""}
                        <button type="button" class="np-button np-login-btn" style="background: var(--neo-signal); color: #fff; padding: 11px; border: none; border-radius: 6px; font-weight: 600; font-size: 14px; cursor: default; margin-top: 6px;">Log In</button>
                    </div>
                `;
                return el;
            }

            case "SearchForm": {
                el.className = "np-search-form";
                el.style.cssText = "display: flex; width: 100%; max-width: 500px;";
                const placeholder = data.placeholder || "Search articles, guides, docs...";
                const btn = data.button_text || "Search";
                el.innerHTML = `
                    <div class="np-search-input-wrap" style="display: flex; width: 100%; position: relative;">
                        <i class="fa-solid fa-magnifying-glass" style="position: absolute; left: 14px; top: 50%; transform: translateY(-50%); color: #94a3b8; font-size: 14px;"></i>
                        <input type="search" class="np-search-input" placeholder="${placeholder}" disabled style="flex: 1; padding: 11px 16px 11px 40px; border: 1px solid #cbd5e1; border-right: none; border-radius: 6px 0 0 6px; font-size: 14px; background: #ffffff;">
                        <button type="button" class="np-search-btn" style="background: var(--neo-signal); color: #fff; padding: 0 20px; border: none; border-radius: 0 6px 6px 0; font-weight: 600; font-size: 13.5px; cursor: default;">${btn}</button>
                    </div>
                `;
                return el;
            }

            case "Breadcrumbs": {
                el.className = "np-breadcrumbs";
                el.style.cssText = "padding: 10px 0;";
                const home = data.home_text || "Home";
                const sep = data.separator || ">";
                el.innerHTML = `
                    <nav aria-label="Breadcrumbs">
                        <ol class="np-breadcrumb-list" style="display: flex; align-items: center; gap: 8px; list-style: none; padding: 0; margin: 0; font-size: 13px;">
                            <li class="np-breadcrumb-item"><a href="#" style="color: var(--neo-signal); text-decoration: none; font-weight: 500;"><i class="fa-solid fa-house" style="font-size: 12px; margin-right: 4px;"></i>${home}</a></li>
                            <li class="np-breadcrumb-separator" style="color: #94a3b8;">${sep}</li>
                            <li class="np-breadcrumb-item"><a href="#" style="color: var(--neo-signal); text-decoration: none; font-weight: 500;">Documentation</a></li>
                            <li class="np-breadcrumb-separator" style="color: #94a3b8;">${sep}</li>
                            <li class="np-breadcrumb-item active" style="color: #64748b; font-weight: 500;"><span>NeoStudio Architecture</span></li>
                        </ol>
                    </nav>
                `;
                return el;
            }

            case "TableOfContents": {
                el.className = "np-toc";
                el.style.cssText = "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 20px; max-width: 380px;";
                const title = data.title || "Table of Contents";
                el.innerHTML = `
                    <div class="np-toc-header" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; border-bottom: 1px solid #e2e8f0; padding-bottom: 10px;">
                        <h4 class="np-toc-title" style="margin: 0; font-size: 14px; font-weight: 700; color: #0f172a; text-transform: uppercase; letter-spacing: 0.05em;"><i class="fa-solid fa-list-ul" style="margin-right: 6px; color: var(--neo-signal);"></i>${title}</h4>
                        <i class="fa-solid fa-chevron-down" style="font-size: 11px; color: #64748b;"></i>
                    </div>
                    <div class="np-toc-body">
                        <ul class="np-toc-list" style="list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; font-size: 13px;">
                            <li><a href="#" style="color: var(--neo-signal); text-decoration: none;">1. Introduction & Overview</a></li>
                            <li style="padding-left: 14px;"><a href="#" style="color: #475569; text-decoration: none;">1.1 Tokio Async Runtime</a></li>
                            <li style="padding-left: 14px;"><a href="#" style="color: #475569; text-decoration: none;">1.2 SQLite Dual-Write Engine</a></li>
                            <li><a href="#" style="color: var(--neo-signal); text-decoration: none;">2. Performance & Benchmarks</a></li>
                            <li><a href="#" style="color: var(--neo-signal); text-decoration: none;">3. Theme Builder Engine</a></li>
                        </ul>
                    </div>
                `;
                return el;
            }

            case "MegaMenuContainer": {
                el.className = "np-megamenu-container";
                el.style.cssText = "background: #ffffff; border: 1px solid #e2e8f0; border-radius: 12px; padding: 28px; width: 100%; box-shadow: 0 10px 30px rgba(0,0,0,0.08);";
                const trigger = data.trigger || "Products Menu";
                el.innerHTML = `
                    <div style="font-size: 11px; text-transform: uppercase; color: var(--neo-signal); font-weight: 700; margin-bottom: 16px; letter-spacing: 0.05em;"><i class="fa-solid fa-layer-group" style="margin-right: 6px;"></i>MegaMenu Preview (Trigger: ${trigger})</div>
                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px;">
                        <div>
                            <h5 style="margin: 0 0 10px 0; font-size: 13.5px; font-weight: 700; color: #0f172a;">Core Engine</h5>
                            <p style="font-size: 12px; color: #64748b; line-height: 1.5; margin: 0 0 10px 0;">Ultra-low latency Rust web server replacing PHP-FPM.</p>
                            <a href="#" style="font-size: 12px; color: var(--neo-signal); font-weight: 600; text-decoration: none;">Learn more &rarr;</a>
                        </div>
                        <div>
                            <h5 style="margin: 0 0 10px 0; font-size: 13.5px; font-weight: 700; color: #0f172a;">NeoStudio CAD</h5>
                            <p style="font-size: 12px; color: #64748b; line-height: 1.5; margin: 0 0 10px 0;">Webflow-class visual builder with live box-model overlays.</p>
                            <a href="#" style="font-size: 12px; color: var(--neo-signal); font-weight: 600; text-decoration: none;">Explore studio &rarr;</a>
                        </div>
                        <div>
                            <h5 style="margin: 0 0 10px 0; font-size: 13.5px; font-weight: 700; color: #0f172a;">Theme Builder</h5>
                            <p style="font-size: 12px; color: #64748b; line-height: 1.5; margin: 0 0 10px 0;">Dynamic header, footer, and template condition engine.</p>
                            <a href="#" style="font-size: 12px; color: var(--neo-signal); font-weight: 600; text-decoration: none;">View templates &rarr;</a>
                        </div>
                    </div>
                `;
                return el;
            }

            case "PopupModal": {
                el.className = "np-popup-modal";
                el.style.cssText = "border: 1px dashed var(--neo-signal); border-radius: 12px; padding: 24px; background: rgba(37,99,235,0.03);";
                const modalId = data.modal_id || "promo-popup";
                el.innerHTML = `
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <span style="font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--neo-signal);"><i class="fa-solid fa-window-restore" style="margin-right: 6px;"></i>Popup Modal [${modalId}]</span>
                        <button type="button" style="border: none; background: #e2e8f0; width: 24px; height: 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; cursor: default; font-size: 12px;">&times;</button>
                    </div>
                    <div style="background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; padding: 24px; text-align: center; box-shadow: 0 4px 12px rgba(0,0,0,0.05);">
                        <h4 style="margin: 0 0 8px 0; font-size: 18px; font-weight: 700; color: #0f172a;">Exclusive Early Access</h4>
                        <p style="margin: 0 0 16px 0; font-size: 13px; color: #64748b;">Join 1,000+ engineers migrating from legacy PHP monoliths.</p>
                        <a href="#" class="np-button" style="background: var(--neo-signal); color: #fff; padding: 8px 18px; border-radius: 6px; font-weight: 600; font-size: 13px; text-decoration: none; display: inline-block;">Reserve Your Spot</a>
                    </div>
                `;
                return el;
            }

            default:
                return null;
        }
    }
    window.renderProDynamicWidget = renderProDynamicWidget;
})();
"###;
