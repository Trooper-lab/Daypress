use super::layout::render_admin_shell;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::utils::escape_html;

// 1. Themes (/d-admin/themes)
pub fn render_themes_view(settings: &DaypressSettingsResponse) -> String {
    let active_theme = if settings.active_theme.is_empty() {
        "Daypress Daylight"
    } else {
        &settings.active_theme
    };

    let themes = [
        (
            "Daypress Daylight",
            "Daypress Daylight",
            "Warm Editorial Paper & Ink (Default)",
            "1.0.0",
            "linear-gradient(135deg, #fbf8f2 0%, #eae3d5 100%)",
            "#a8401b",
        ),
        (
            "Daypress Modern",
            "Daypress Modern",
            "Clean Studio Alabaster & Indigo",
            "1.0.0",
            "linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%)",
            "#2563eb",
        ),
        (
            "Daypress Night",
            "Daypress Night",
            "High-Contrast Obsidian Dark Mode",
            "1.0.0",
            "linear-gradient(135deg, #090d16 0%, #1e293b 100%)",
            "#38bdf8",
        ),
    ];

    let mut cards_html = String::new();
    for (name, banner_title, subtitle, version, bg, sub_color) in themes {
        let is_active = active_theme.eq_ignore_ascii_case(name);
        let card_border = if is_active {
            "border: 2px solid var(--press); overflow: hidden;"
        } else {
            "overflow: hidden;"
        };
        let status_line = if is_active {
            format!(
                r#"<div style="font-size: 11.5px; color: var(--moss); font-weight: 600; margin-top: 2px;">Active Theme &bull; Version {}</div>"#,
                version
            )
        } else {
            format!(
                r#"<div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 2px;">Version {}</div>"#,
                version
            )
        };
        let action_btn = if is_active {
            r#"<a href="/d-admin/customize" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">Customize</a>"#.to_string()
        } else {
            format!(
                r#"<button class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="switchTheme('{}')">Activate</button>"#,
                name
            )
        };

        cards_html.push_str(&format!(
            r##"
            <div class="day-card" style="{card_border}">
                <div style="height: 180px; background: {bg}; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--paper-raised); padding: 20px; text-align: center;">
                    <div style="font-size: 22px; font-weight: 700; letter-spacing: -0.5px;">{banner_title}</div>
                    <div style="font-size: 12px; color: {sub_color}; font-family: var(--font-mono); margin-top: 4px;">{subtitle}</div>
                </div>
                <div style="padding: 16px; display: flex; justify-content: space-between; align-items: center;">
                    <div>
                        <strong style="font-size: 15px; color: var(--ink);">{name}</strong>
                        {status_line}
                    </div>
                    {action_btn}
                </div>
            </div>
            "##,
            card_border = card_border,
            bg = bg,
            banner_title = banner_title,
            sub_color = sub_color,
            subtitle = subtitle,
            name = name,
            status_line = status_line,
            action_btn = action_btn
        ));
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Themes</h1>
                <p class="page-subtitle">Manage, customize, and activate visual presentation skins for your publication</p>
            </div>
            <div>
                <a href="/d-admin/customize" class="btn btn-primary">Customize Active Theme</a>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px;">
            {cards_html}
        </div>

        <script>
            async function switchTheme(theme) {{
                try {{
                    const res = await fetch('/d-admin/api/theme', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ theme }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Theme activated in SQLite: ' + theme);
                        setTimeout(() => window.location.reload(), 400);
                    }} else {{
                        showToast('Failed to switch theme', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        cards_html = cards_html
    );

    render_admin_shell("themes", "Themes", &body, settings)
}

// 2. Customize (/d-admin/customize)
pub fn render_customize_view(settings: &DaypressSettingsResponse) -> String {
    let brand_color = if settings.brand_color.is_empty() {
        "#a8401b"
    } else {
        &settings.brand_color
    };
    let active_theme = if settings.active_theme.is_empty() {
        "Daypress Daylight"
    } else {
        &settings.active_theme
    };
    let sel_theme_daylight = if active_theme.eq_ignore_ascii_case("Daypress Daylight") {
        "selected"
    } else {
        ""
    };
    let sel_theme_modern = if active_theme.eq_ignore_ascii_case("Daypress Modern") {
        "selected"
    } else {
        ""
    };
    let sel_theme_night = if active_theme.eq_ignore_ascii_case("Daypress Night") {
        "selected"
    } else {
        ""
    };
    let scale = settings.typography_scale.as_str();
    let sel_system = if scale == "system" { "selected" } else { "" };
    let sel_serif = if scale.is_empty() || scale == "serif" {
        "selected"
    } else {
        ""
    };
    let sel_mono = if scale == "mono" { "selected" } else { "" };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Theme Customizer</h1>
                <p class="page-subtitle">Real-time live styling controls and preview</p>
            </div>
            <div style="display: flex; gap: 8px;">
                <a href="/" target="_blank" class="btn btn-secondary">Open Site &nearr;</a>
                <button type="button" class="btn btn-primary" onclick="saveCustomizer()">Publish Changes</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 340px 1fr; gap: 20px; height: calc(100vh - 180px); min-height: 540px;">
            <!-- Left Controls Panel -->
            <div class="day-card" style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 18px;">
                <h3 style="font-size: 15px; font-weight: 700; color: var(--ink); padding-bottom: 8px; border-bottom: 1px solid var(--border-subtle);">Active Theme: {active_theme}</h3>

                <div>
                    <label class="form-label">Daypress Theme Preset</label>
                    <select id="custTheme" class="form-select" onchange="updateLivePreview()">
                        <option value="Daypress Daylight" {sel_theme_daylight}>Daypress Daylight (Warm Editorial Paper &amp; Ink)</option>
                        <option value="Daypress Modern" {sel_theme_modern}>Daypress Modern (Clean Studio Alabaster &amp; Indigo)</option>
                        <option value="Daypress Night" {sel_theme_night}>Daypress Night (Obsidian Dark Slate)</option>
                    </select>
                </div>

                <div>
                    <label class="form-label">Site Title</label>
                    <input type="text" id="custTitle" value="{title}" class="form-input" oninput="updateLivePreview()" />
                </div>

                <div>
                    <label class="form-label">Site Tagline</label>
                    <input type="text" id="custTagline" value="{desc}" class="form-input" oninput="updateLivePreview()" />
                </div>

                <div>
                    <label class="form-label">Primary Brand Color</label>
                    <div style="display: flex; gap: 10px; align-items: center;">
                        <input type="color" id="custColor" value="{brand_color}" style="width: 44px; height: 36px; padding: 0; border: 1px solid var(--border-subtle); border-radius: 6px; cursor: pointer;" oninput="updateLivePreview()" />
                        <span id="custColorVal" style="font-family: var(--font-mono); font-size: 13px;">{brand_color}</span>
                    </div>
                </div>

                <div>
                    <label class="form-label">Converged Daypress Theme Tokens (DayStudio Design Kit)</label>
                    <div style="padding: 10px 12px; background: var(--paper-raised); border: 1px solid var(--border-subtle); border-radius: 6px; font-size: 12px; color: var(--ink-muted);">
                        <div style="font-weight: 600; color: var(--ink); margin-bottom: 4px;">✓ Unified CSS Variable System Active</div>
                        <div>Global Colors (<code style="font-size:11px;">--dp-color-*</code> / <code style="font-size:11px;">--e-global-color-*</code>) &amp; Typography (<code style="font-size:11px;">--dp-font-*</code>) synced with Visual Studio Builder.</div>
                    </div>
                </div>

                <div>
                    <label class="form-label">Typography Scale</label>
                    <select id="custFont" class="form-select" onchange="updateLivePreview()">
                        <option value="serif" {sel_serif}>Editorial Serif (Newsreader + Schibsted Grotesk)</option>
                        <option value="system" {sel_system}>Modern System (Schibsted Grotesk / System UI)</option>
                        <option value="mono" {sel_mono}>Technical Monospace (IBM Plex Mono)</option>
                    </select>
                </div>

                <div style="flex: 1; display: flex; flex-direction: column;">
                    <label class="form-label">Additional Custom CSS</label>
                    <textarea id="custCss" class="form-textarea" style="flex: 1; min-height: 120px; font-family: var(--font-mono); font-size: 12px;" placeholder="/* Custom CSS overrides */&#10;body {{&#10;  letter-spacing: -0.01em;&#10;}}" oninput="updateLivePreview()">{custom_css}</textarea>
                </div>
            </div>

            <!-- Right Live Preview Iframe -->
            <div class="day-card" style="overflow: hidden; display: flex; flex-direction: column; border-radius: var(--radius-sm);">
                <div style="padding: 10px 16px; background: var(--ink); color: var(--paper); font-size: 12px; display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-family: var(--font-mono);">Live Preview: {url}/</span>
                    <button type="button" onclick="document.getElementById('previewIframe').contentWindow.location.reload()" style="background: none; border: none; color: var(--ink-muted); cursor: pointer; font-size: 11px;">&#8635; Refresh</button>
                </div>
                <iframe id="previewIframe" src="/" style="flex: 1; width: 100%; border: none; background: var(--paper-raised);"></iframe>
            </div>
        </div>

        <script>
            function updateLivePreview() {{
                const color = document.getElementById('custColor').value;
                document.getElementById('custColorVal').textContent = color;
                const iframe = document.getElementById('previewIframe');
                try {{
                    const titleEl = iframe.contentWindow.document.querySelector('.site-title, h1, header a');
                    if (titleEl) titleEl.textContent = document.getElementById('custTitle').value;
                }} catch (e) {{}}
            }}
            async function saveCustomizer() {{
                const active_theme = document.getElementById('custTheme').value;
                const title = document.getElementById('custTitle').value;
                const description = document.getElementById('custTagline').value;
                const brand_color = document.getElementById('custColor').value;
                const typography_scale = document.getElementById('custFont').value;
                const custom_css = document.getElementById('custCss').value;
                try {{
                    const res = await fetch('/d-admin/api/customizer', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ active_theme, title, description, brand_color, typography_scale, custom_css }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Customizer changes published to SQLite!');
                        const iframe = document.getElementById('previewIframe');
                        if (iframe && iframe.contentWindow) iframe.contentWindow.location.reload();
                    }} else {{
                        showToast('Failed to save customizer', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        active_theme = escape_html(active_theme),
        title = escape_html(&settings.title),
        desc = escape_html(&settings.description),
        brand_color = escape_html(brand_color),
        sel_system = sel_system,
        sel_serif = sel_serif,
        sel_mono = sel_mono,
        custom_css = escape_html(&settings.custom_css),
        url = escape_html(&settings.url)
    );

    render_admin_shell("customize", "Customize", &body, settings)
}

// 3. Widgets (/d-admin/widgets)
pub fn render_widgets_view(settings: &DaypressSettingsResponse) -> String {
    let default_val = serde_json::json!({
        "sidebar": ["Search Bar", "Recent Posts"],
        "footer": ["Categories List"]
    });
    let val: serde_json::Value =
        serde_json::from_str(&settings.widgets_json).unwrap_or(default_val);
    let sidebar_items = val
        .get("sidebar")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["Search Bar", "Recent Posts"]);
    let footer_items = val
        .get("footer")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["Categories List"]);

    let mut sidebar_html = String::new();
    for item in sidebar_items {
        sidebar_html.push_str(&format!(
            r#"<div class="day-card widget-item" style="padding: 10px 14px; background: var(--paper-raised); border: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center;">
                <span class="widget-title" style="font-size: 13px; font-weight: 600;">{title}</span>
                <button onclick="this.parentElement.remove()" style="color: var(--oxide); background: none; border: none; cursor: pointer; font-size: 12px;">Remove</button>
            </div>"#,
            title = escape_html(item)
        ));
    }

    let mut footer_html = String::new();
    for item in footer_items {
        footer_html.push_str(&format!(
            r#"<div class="day-card widget-item" style="padding: 10px 14px; background: var(--paper-raised); border: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center;">
                <span class="widget-title" style="font-size: 13px; font-weight: 600;">{title}</span>
                <button onclick="this.parentElement.remove()" style="color: var(--oxide); background: none; border: none; cursor: pointer; font-size: 12px;">Remove</button>
            </div>"#,
            title = escape_html(item)
        ));
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Widgets</h1>
                <p class="page-subtitle">Add and configure sidebar blocks, footer columns, and dynamic widgets</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="saveWidgets()">Save Widget Areas</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 24px;">
            <!-- Left: Available Widgets -->
            <div class="day-card" style="padding: 20px;">
                <h3 style="font-size: 15px; font-weight: 700; color: var(--ink); margin-bottom: 6px;">Available Widgets</h3>
                <p style="font-size: 13px; color: var(--ink-muted); margin-bottom: 16px;">Click on any widget to add it to your primary sidebar or footer areas.</p>

                <div style="display: flex; flex-direction: column; gap: 10px;">
                    <div class="day-card" style="padding: 12px; display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised); border: 1px solid var(--border-subtle);">
                        <div>
                            <strong style="font-size: 13.5px; color: var(--ink);">Search Bar</strong>
                            <div style="font-size: 11.5px; color: var(--ink-muted);">A search form for your site</div>
                        </div>
                        <div style="display: flex; gap: 6px;">
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Search Bar', 'sidebar')">+ Sidebar</button>
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Search Bar', 'footer')">+ Footer</button>
                        </div>
                    </div>

                    <div class="day-card" style="padding: 12px; display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised); border: 1px solid var(--border-subtle);">
                        <div>
                            <strong style="font-size: 13.5px; color: var(--ink);">Recent Posts</strong>
                            <div style="font-size: 11.5px; color: var(--ink-muted);">Your site's most recent articles</div>
                        </div>
                        <div style="display: flex; gap: 6px;">
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Recent Posts', 'sidebar')">+ Sidebar</button>
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Recent Posts', 'footer')">+ Footer</button>
                        </div>
                    </div>

                    <div class="day-card" style="padding: 12px; display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised); border: 1px solid var(--border-subtle);">
                        <div>
                            <strong style="font-size: 13.5px; color: var(--ink);">Categories List</strong>
                            <div style="font-size: 11.5px; color: var(--ink-muted);">A list or dropdown of categories</div>
                        </div>
                        <div style="display: flex; gap: 6px;">
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Categories List', 'sidebar')">+ Sidebar</button>
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Categories List', 'footer')">+ Footer</button>
                        </div>
                    </div>

                    <div class="day-card" style="padding: 12px; display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised); border: 1px solid var(--border-subtle);">
                        <div>
                            <strong style="font-size: 13.5px; color: var(--ink);">Custom HTML</strong>
                            <div style="font-size: 11.5px; color: var(--ink-muted);">Arbitrary HTML or embedded snippets</div>
                        </div>
                        <div style="display: flex; gap: 6px;">
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Custom HTML', 'sidebar')">+ Sidebar</button>
                            <button class="btn btn-secondary" style="font-size: 11px; padding: 4px 8px;" onclick="addWidget('Custom HTML', 'footer')">+ Footer</button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Right: Active Widget Zones -->
            <div style="display: flex; flex-direction: column; gap: 20px;">
                <div class="day-card" style="padding: 20px;">
                    <h3 style="font-size: 15px; font-weight: 700; color: var(--ink); margin-bottom: 4px;">Primary Sidebar</h3>
                    <p style="font-size: 12.5px; color: var(--ink-muted); margin-bottom: 14px;">Appears on the right side of editorial articles and single posts.</p>
                    <div id="sidebarWidgetArea" style="display: flex; flex-direction: column; gap: 8px;">
                        {sidebar_html}
                    </div>
                </div>

                <div class="day-card" style="padding: 20px;">
                    <h3 style="font-size: 15px; font-weight: 700; color: var(--ink); margin-bottom: 4px;">Footer Columns</h3>
                    <p style="font-size: 12.5px; color: var(--ink-muted); margin-bottom: 14px;">Appears across the bottom of every page.</p>
                    <div id="footerWidgetArea" style="display: flex; flex-direction: column; gap: 8px;">
                        {footer_html}
                    </div>
                </div>
            </div>
        </div>

        <script>
            function addWidget(name, zone) {{
                const areaId = zone === 'footer' ? 'footerWidgetArea' : 'sidebarWidgetArea';
                const area = document.getElementById(areaId);
                if (!area) return;
                const div = document.createElement('div');
                div.className = 'day-card widget-item';
                div.style = 'padding: 10px 14px; background: var(--paper-raised); border: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center;';
                div.innerHTML = '<span class="widget-title" style="font-size: 13px; font-weight: 600;">' + name + '</span><button onclick="this.parentElement.remove()" style="color: var(--oxide); background: none; border: none; cursor: pointer; font-size: 12px;">Remove</button>';
                area.appendChild(div);
                showToast('✓ Added ' + name + ' to ' + (zone === 'footer' ? 'Footer Columns' : 'Primary Sidebar'));
            }}
            async function saveWidgets() {{
                const sidebar = Array.from(document.querySelectorAll('#sidebarWidgetArea .widget-title')).map(el => el.textContent.trim());
                const footer = Array.from(document.querySelectorAll('#footerWidgetArea .widget-title')).map(el => el.textContent.trim());
                try {{
                    const res = await fetch('/d-admin/api/widgets', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ widgets: {{ sidebar, footer }} }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Widget zones updated in SQLite options!');
                    }} else {{
                        showToast('Failed to save widgets', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
    "##,
        sidebar_html = sidebar_html,
        footer_html = footer_html
    );

    render_admin_shell("widgets", "Widgets", &body, settings)
}

// 4. Menus (/d-admin/nav-menus)
pub fn render_nav_menus_view(
    settings: &DaypressSettingsResponse,
    pages: &[DaypressPostResponse],
) -> String {
    let mut page_items = String::new();
    for p in pages {
        page_items.push_str(&format!(
            r#"<label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                <input type="checkbox" class="page-menu-chk" data-title="{title}" data-url="/{slug}/">
                <span>{title}</span>
            </label>"#,
            title = escape_html(&p.title.rendered),
            slug = p.slug
        ));
    }

    let mut existing_menu_cards = String::new();
    if settings.nav_menu_items.is_empty() {
        existing_menu_cards.push_str(
            r#"<div class="day-card menu-item-card" data-title="Home" data-url="/" style="padding: 12px 16px; background: var(--paper-raised); border: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center;">
                <span class="menu-item-title" style="font-weight: 600; font-size: 13px;">Home</span>
                <div style="display: flex; gap: 8px; align-items: center;">
                    <span class="menu-item-url" style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono);">/</span>
                    <button onclick="this.closest('.day-card').remove()" style="color: var(--oxide); background: none; border: none; cursor: pointer; font-size: 12px;">Remove</button>
                </div>
            </div>"#,
        );
    } else {
        for item in &settings.nav_menu_items {
            existing_menu_cards.push_str(&format!(
                r#"<div class="day-card menu-item-card" data-title="{title}" data-url="{url}" style="padding: 12px 16px; background: var(--paper-raised); border: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center;">
                    <span class="menu-item-title" style="font-weight: 600; font-size: 13px;">{title}</span>
                    <div style="display: flex; gap: 8px; align-items: center;">
                        <span class="menu-item-url" style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono);">{url}</span>
                        <button onclick="this.closest('.day-card').remove()" style="color: var(--oxide); background: none; border: none; cursor: pointer; font-size: 12px;">Remove</button>
                    </div>
                </div>"#,
                title = escape_html(&item.title),
                url = escape_html(&item.url),
            ));
        }
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Menus</h1>
                <p class="page-subtitle">Configure site navigation structures and link hierarchies</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="saveNavMenu()">Save Menu</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 340px 1fr; gap: 24px;">
            <!-- Left: Add Items -->
            <div style="display: flex; flex-direction: column; gap: 20px;">
                <div class="day-card" style="padding: 20px;">
                    <h3 style="font-size: 14px; font-weight: 700; color: var(--ink); margin-bottom: 12px;">Pages</h3>
                    <div style="max-height: 200px; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; padding-bottom: 12px;">
                        {page_items}
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px; margin-top: 8px;" onclick="addSelectedPages()">Add to Menu</button>
                </div>

                <div class="day-card" style="padding: 20px;">
                    <h3 style="font-size: 14px; font-weight: 700; color: var(--ink); margin-bottom: 12px;">Custom Links</h3>
                    <div class="form-group" style="margin-bottom: 10px;">
                        <label class="form-label" style="font-size: 12px;">URL</label>
                        <input type="text" id="customLinkUrl" placeholder="https://" class="form-input" value="https://">
                    </div>
                    <div class="form-group" style="margin-bottom: 12px;">
                        <label class="form-label" style="font-size: 12px;">Link Text</label>
                        <input type="text" id="customLinkText" placeholder="Menu Item" class="form-input">
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="addCustomLink()">Add to Menu</button>
                </div>
            </div>

            <!-- Right: Menu Structure -->
            <div class="day-card" style="padding: 24px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px; padding-bottom: 14px; border-bottom: 1px solid var(--border-subtle);">
                    <div style="display: flex; align-items: center; gap: 10px;">
                        <label style="font-size: 13px; font-weight: 600;">Menu Name:</label>
                        <input type="text" id="menuName" value="Primary Navigation" class="form-input" style="width: 220px;">
                    </div>
                    <span style="font-size: 12px; color: var(--ink-muted);">Header Location: ✓ Active</span>
                </div>

                <h4 style="font-size: 14px; font-weight: 600; color: var(--ink); margin-bottom: 6px;">Menu Structure</h4>
                <p style="font-size: 12.5px; color: var(--ink-muted); margin-bottom: 16px;">Reorder items by dragging or clicking to organize your site navigation.</p>

                <div id="menuItemsList" style="display: flex; flex-direction: column; gap: 8px;">
                    {existing_menu_cards}
                </div>
            </div>
        </div>

        <script>
            function addSelectedPages() {{
                const list = document.getElementById('menuItemsList');
                document.querySelectorAll('.page-menu-chk:checked').forEach(chk => {{
                    const title = chk.getAttribute('data-title');
                    const url = chk.getAttribute('data-url');
                    const item = document.createElement('div');
                    item.className = 'day-card menu-item-card';
                    item.setAttribute('data-title', title);
                    item.setAttribute('data-url', url);
                    item.style = 'padding: 12px 16px; background: var(--paper-raised); border: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center;';
                    item.innerHTML = '<span class="menu-item-title" style="font-weight: 600; font-size: 13px;">' + title + '</span><div style="display: flex; gap: 8px; align-items: center;"><span class="menu-item-url" style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono);">' + url + '</span><button onclick="this.closest(\'.day-card\').remove()" style="color: var(--oxide); background: none; border: none; cursor: pointer; font-size: 12px;">Remove</button></div>';
                    list.appendChild(item);
                    chk.checked = false;
                }});
                showToast('✓ Pages added to menu');
            }}
            function addCustomLink() {{
                const url = document.getElementById('customLinkUrl').value;
                const text = document.getElementById('customLinkText').value;
                if (!text || !url) return;
                const list = document.getElementById('menuItemsList');
                const item = document.createElement('div');
                item.className = 'day-card menu-item-card';
                item.setAttribute('data-title', text);
                item.setAttribute('data-url', url);
                item.style = 'padding: 12px 16px; background: var(--paper-raised); border: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center;';
                item.innerHTML = '<span class="menu-item-title" style="font-weight: 600; font-size: 13px;">' + text + '</span><div style="display: flex; gap: 8px; align-items: center;"><span class="menu-item-url" style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono);">' + url + '</span><button onclick="this.closest(\'.day-card\').remove()" style="color: var(--oxide); background: none; border: none; cursor: pointer; font-size: 12px;">Remove</button></div>';
                list.appendChild(item);
                document.getElementById('customLinkText').value = '';
                showToast('✓ Custom link added to menu');
            }}
            async function saveNavMenu() {{
                const items = Array.from(document.querySelectorAll('#menuItemsList .menu-item-card')).map((card, idx) => ({{
                    id: idx + 1,
                    title: card.getAttribute('data-title') || card.querySelector('.menu-item-title').textContent.trim(),
                    url: card.getAttribute('data-url') || card.querySelector('.menu-item-url').textContent.trim(),
                    target: '',
                    parent: 0,
                    menu_order: idx + 1
                }}));
                try {{
                    const res = await fetch('/d-admin/api/menus', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ items }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Navigation menu updated and saved in SQLite!');
                    }} else {{
                        showToast('Failed to save navigation menu', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        page_items = page_items,
        existing_menu_cards = existing_menu_cards
    );

    render_admin_shell("menus", "Menus", &body, settings)
}

// 5. Theme File Editor (/d-admin/theme-editor)
pub fn render_theme_editor_view(settings: &DaypressSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Theme File Editor</h1>
                <p class="page-subtitle">Inspect and safely edit template stylesheets and frontend components</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="saveThemeFile()">Update File</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 260px; gap: 20px;">
            <!-- Code Area -->
            <div class="day-card" style="padding: 20px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                    <span id="activeFileName" style="font-family: var(--font-mono); font-size: 13px; font-weight: 600; color: var(--ink);">style.css (Daypress Core Styles)</span>
                    <span style="font-size: 12px; color: var(--moss); font-weight: 600;">✓ Sandboxed Path</span>
                </div>
                <textarea id="themeFileContent" class="day-html-textarea" style="min-height: 500px;">/*
Theme Name: Daypress Modern 2026
Theme URI: https://github.com/Trooper-lab/daypress
Description: Ultra-fast native Rust editorial template with embedded Rusqlite
Author: Daypress Core Team
Version: 1.0.0
*/

:root {
  --primary-color: #2563eb;
  --text-main: #0f172a;
  --bg-color: #ffffff;
}

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Inter", sans-serif;
  color: var(--text-main);
  background: var(--bg-color);
  line-height: 1.7;
}

.site-header {
  border-bottom: 1px solid #e2e8f0;
  padding: 24px 0;
}
</textarea>
            </div>

            <!-- File List -->
            <div class="day-card" style="padding: 20px; height: fit-content;">
                <h4 style="font-size: 13.5px; font-weight: 700; color: var(--ink); margin-bottom: 12px;">Theme Files</h4>
                <div style="display: flex; flex-direction: column; gap: 6px;">
                    <a href="javascript:void(0)" onclick="loadFile('style.css')" style="padding: 6px 10px; border-radius: 4px; background: var(--press-wash); color: var(--press); font-weight: 600; font-size: 12.5px; text-decoration: none;">Stylesheet (style.css)</a>
                    <a href="javascript:void(0)" onclick="loadFile('public.rs')" style="padding: 6px 10px; border-radius: 4px; color: var(--ink-muted); font-size: 12.5px; text-decoration: none;">Public Template (public.rs)</a>
                    <a href="javascript:void(0)" onclick="loadFile('single.rs')" style="padding: 6px 10px; border-radius: 4px; color: var(--ink-muted); font-size: 12.5px; text-decoration: none;">Single Article (single.rs)</a>
                </div>
                <div style="margin-top: 20px; padding-top: 14px; border-top: 1px solid var(--border-subtle); font-size: 11.5px; color: var(--ink-muted);">
                    <strong>Notice:</strong> Edits are validated before persisting to prevent syntax corruption.
                </div>
            </div>
        </div>

        <script>
            let currentThemeFile = 'style.css';
            async function loadFile(name) {
                currentThemeFile = name;
                document.getElementById('activeFileName').textContent = name + ' (Daypress Template)';
                try {
                    const res = await fetch('/d-admin/api/theme-file?file=' + encodeURIComponent(name));
                    if (res.ok) {
                        const data = await res.json();
                        if (data.content && data.content.trim().length > 0) {
                            document.getElementById('themeFileContent').value = data.content;
                        }
                        showToast('Loaded ' + name);
                    }
                } catch (e) {
                    showToast('Failed to load ' + name, true);
                }
            }
            async function saveThemeFile() {
                const content = document.getElementById('themeFileContent').value;
                try {
                    const res = await fetch('/d-admin/api/theme-file', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ file: currentThemeFile, content })
                    });
                    if (res.ok) {
                        showToast('✓ File saved to SQLite with zero-panic validation!');
                    } else {
                        showToast('Failed to save theme file', true);
                    }
                } catch (e) {
                    showToast('Network error: ' + e, true);
                }
            }
        </script>
    "##;

    render_admin_shell("theme-editor", "Theme File Editor", body, settings)
}
