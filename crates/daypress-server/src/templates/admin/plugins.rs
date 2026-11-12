use super::layout::render_admin_shell;
use crate::models::{
    BackupSnapshotItem, DaypressForm, DaypressSettingsResponse, FormEntryItem, FormFieldSchema,
    FormSettings, GlobalSeoSettings, InstalledPluginRecord, NotFoundLogEntry, RedirectRule,
};
use crate::templates::utils::escape_html;

// 1. Installed Plugins (/d-admin/plugins)
pub fn render_plugins_view(
    settings: &DaypressSettingsResponse,
    installed: &[InstalledPluginRecord],
) -> String {
    let total_count = installed.len();
    let active_count = installed.iter().filter(|p| p.active).count();
    let inactive_count = total_count.saturating_sub(active_count);
    let builtin_count = installed.iter().filter(|p| p.manifest.is_builtin).count();
    let custom_count = total_count.saturating_sub(builtin_count);

    let mut rows_html = String::new();
    for record in installed {
        let slug = escape_html(&record.manifest.slug);
        let name = escape_html(&record.manifest.name);
        let desc = escape_html(&record.manifest.description);
        let version = escape_html(&record.manifest.version);
        let author = escape_html(&record.manifest.author);
        let is_active = record.active;

        let category_label = if record.manifest.is_builtin {
            "Official Extension"
        } else {
            "Community Extension"
        };

        let badge_html = if is_active {
            r#"<span class="badge badge-published" style="font-size:11px;">Active</span>"#
        } else {
            r#"<span class="badge badge-draft" style="font-size:11px;">Inactive</span>"#
        };

        let origin_badge = if record.manifest.is_builtin {
            ""
        } else {
            r#"<span style="font-size:10.5px; background:var(--press-wash); color:var(--press); border:1px solid var(--press); padding:1px 6px; border-radius:4px; margin-left:6px; font-weight:600;">Community Extension</span>"#
        };

        let toggle_action = if is_active {
            format!(
                r#"<a href="javascript:void(0)" onclick="togglePlugin(this, '{slug}', false)" style="color:var(--ink); font-weight:600;">Deactivate</a>"#,
                slug = slug
            )
        } else {
            format!(
                r#"<a href="javascript:void(0)" onclick="togglePlugin(this, '{slug}', true)" style="color:var(--moss); font-weight:600;">Activate</a>"#,
                slug = slug
            )
        };

        let (settings_url, settings_label) = match record.manifest.slug.as_str() {
            "daypress-forms" => ("/d-admin/forms", "FormStudio & Vault"),
            "daypress-newsletters" => ("/d-admin/newsletters", "Newsletters"),
            "daypress-webhooks" => ("/d-admin/webhooks", "Shortlinks & Webhooks"),
            "daypress-ab-testing" => ("/d-admin/ab-testing", "A/B Testing & Vitals"),
            "whatsapp-chat" => (
                "/d-admin/plugin-settings?plugin=whatsapp-chat",
                "Configure Chat",
            ),
            "code-snippets" => (
                "/d-admin/plugin-settings?plugin=code-snippets",
                "Configure Snippets",
            ),
            _ => {
                if let Some(ref custom_url) = record.manifest.settings_url {
                    if !custom_url.is_empty() {
                        (custom_url.as_str(), "Settings")
                    } else {
                        ("/d-admin/plugins", "Settings")
                    }
                } else {
                    ("/d-admin/plugins", "Settings")
                }
            }
        };

        let custom_actions = if !record.manifest.is_builtin {
            format!(
                r#"| <a href="/d-admin/plugin-editor?plugin={slug}">Edit Code</a> | <a href="javascript:void(0)" onclick="uninstallCustomPlugin('{slug}')" class="trash-action">Delete</a>"#,
                slug = slug
            )
        } else {
            String::new()
        };

        let shortcode_hint = if !record.manifest.hooks.shortcode.is_empty() {
            format!(
                r#"<span style="font-family:var(--font-mono);font-size:11px;background:var(--paper-sunk);color:var(--ink);padding:1px 6px;border-radius:4px;margin-left:6px;">[{}]</span>"#,
                escape_html(&record.manifest.hooks.shortcode)
            )
        } else {
            String::new()
        };

        let row_status = if is_active { "active" } else { "inactive" };
        let row_kind = if record.manifest.is_builtin {
            "builtin"
        } else {
            "community"
        };
        let row_bg = if is_active {
            "background: var(--oxide-wash);"
        } else {
            ""
        };
        let active_left_border = if is_active {
            "border-left: 4px solid var(--oxide);"
        } else {
            "border-left: 4px solid transparent;"
        };

        rows_html.push_str(&format!(
            r##"
            <tr id="plugin-row-{slug}" data-status="{row_status}" data-kind="{row_kind}" style="{row_bg}">
                <td style="width: 32px; {active_left_border}"><input type="checkbox" name="checked_plugins[]" value="{slug}" onchange="updateBulkSelection()"></td>
                <td style="width: 300px;">
                    <div style="display:flex;align-items:center;flex-wrap:wrap;gap:2px;">
                        <strong style="color: var(--ink); font-size: 14px;">{name}</strong>
                        {origin_badge}
                    </div>
                    <div class="table-actions" style="margin-top:6px;">
                        {toggle_action} | <a href="{settings_url}">{settings_label}</a> {custom_actions}
                    </div>
                </td>
                <td>
                    <div style="color:var(--ink);line-height:1.5;">{desc}{shortcode_hint}</div>
                    <div style="font-size: 12px; color: var(--ink-muted); margin-top: 6px;">
                        Version {version} | By {author} | {category_label}
                    </div>
                </td>
                <td style="width: 120px;">{badge_html}</td>
            </tr>
            "##,
            slug = slug,
            name = name,
            origin_badge = origin_badge,
            toggle_action = toggle_action,
            settings_url = settings_url,
            settings_label = settings_label,
            custom_actions = custom_actions,
            desc = desc,
            shortcode_hint = shortcode_hint,
            version = version,
            author = author,
            category_label = category_label,
            badge_html = badge_html,
            row_status = row_status,
            row_kind = row_kind,
            row_bg = row_bg,
            active_left_border = active_left_border,
        ));
    }

    if rows_html.is_empty() {
        rows_html = r#"<tr><td colspan="4" style="text-align:center; padding:36px; color:var(--ink-muted);">No plugins found matching this view.</td></tr>"#.to_string();
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Plugins</h1>
                <a href="/d-admin/plugin-install" class="page-title-action">+ Add New / Install Extension</a>
                <a href="/d-admin/api/plugin-sdk-template" class="page-title-action">Download Starter SDK (.json)</a>
            </div>
        </div>

        <div class="wp-subsubsub-row">
            <div class="subsubsub" id="pluginFilterTabs">
                <button type="button" class="plug-tab status-tab active" data-filter="all" onclick="filterPluginsByTab('all', this)">All ({total_count})</button>
                <span class="subsubsub-sep">|</span>
                <button type="button" class="plug-tab status-tab" data-filter="active" onclick="filterPluginsByTab('active', this)">Active ({active_count})</button>
                <span class="subsubsub-sep">|</span>
                <button type="button" class="plug-tab status-tab" data-filter="inactive" onclick="filterPluginsByTab('inactive', this)">Inactive ({inactive_count})</button>
                <span class="subsubsub-sep">|</span>
                <button type="button" class="plug-tab status-tab" data-filter="builtin" onclick="filterPluginsByTab('builtin', this)">Official ({builtin_count})</button>
                <span class="subsubsub-sep">|</span>
                <button type="button" class="plug-tab status-tab" data-filter="community" onclick="filterPluginsByTab('community', this)">Community ({custom_count})</button>
            </div>
            <div style="display: flex; gap: 6px; align-items: center;">
                <input type="search" id="pluginSearchInput" placeholder="Search installed plugins..." class="form-input" style="width: 220px; padding: 4px 10px; min-height: 30px; font-size: 13px;" oninput="filterPluginsList(this.value)" />
            </div>
        </div>

        <div class="tablenav top">
            <div class="tablenav-actions">
                <select id="bulkPluginAction" class="form-input" style="width: auto; padding: 4px 28px 4px 8px; min-height: 30px; font-size: 13px;">
                    <option value="">Bulk actions</option>
                    <option value="activate">Activate</option>
                    <option value="deactivate">Deactivate</option>
                </select>
                <button type="button" class="btn btn-secondary" style="padding: 4px 10px; min-height: 30px; font-size: 13px;" onclick="applyBulkPluginAction()">Apply</button>
            </div>
            <div class="tablenav-pages">
                <span>{total_count} items</span>
            </div>
        </div>

        <div class="day-table-wrapper">
            <table class="day-table" id="pluginsTable">
                <thead>
                    <tr>
                        <th style="width: 32px;"><input type="checkbox" id="selectAllPlugins" onchange="toggleSelectAllPlugins(this)"></th>
                        <th style="width: 300px;">Plugin</th>
                        <th>Description</th>
                        <th style="width: 120px;">Status</th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
                <tfoot>
                    <tr>
                        <th style="width: 32px;"><input type="checkbox" onchange="toggleSelectAllPlugins(this)"></th>
                        <th style="width: 300px;">Plugin</th>
                        <th>Description</th>
                        <th style="width: 120px;">Status</th>
                    </tr>
                </tfoot>
            </table>
        </div>

        <script>
            let activePluginFilter = 'all';

            function toggleSelectAllPlugins(master) {{
                document.querySelectorAll('#pluginsTable tbody input[name="checked_plugins[]"]').forEach(cb => {{
                    const row = cb.closest('tr');
                    if (row && row.style.display !== 'none') {{
                        cb.checked = master.checked;
                    }}
                }});
            }}

            function updateBulkSelection() {{
                // helper if needed
            }}

            function filterPluginsByTab(filter, btn) {{
                activePluginFilter = filter;
                document.querySelectorAll('#pluginFilterTabs .plug-tab').forEach(b => {{
                    b.classList.remove('active');
                }});
                if (btn) {{
                    btn.classList.add('active');
                }}
                const master = document.getElementById('selectAllPlugins');
                if (master) master.checked = false;
                const q = document.getElementById('pluginSearchInput') ? document.getElementById('pluginSearchInput').value : '';
                filterPluginsList(q);
            }}

            function filterPluginsList(q) {{
                const filter = (q || '').toLowerCase();
                const rows = document.querySelectorAll('#pluginsTable tbody tr[data-status]');
                rows.forEach(r => {{
                    const text = r.textContent.toLowerCase();
                    const rowStatus = r.getAttribute('data-status');
                    const rowKind = r.getAttribute('data-kind');
                    let tabMatch = true;
                    if (activePluginFilter === 'active') tabMatch = (rowStatus === 'active');
                    else if (activePluginFilter === 'inactive') tabMatch = (rowStatus === 'inactive');
                    else if (activePluginFilter === 'builtin') tabMatch = (rowKind === 'builtin');
                    else if (activePluginFilter === 'community') tabMatch = (rowKind === 'community');

                    const searchMatch = !filter || text.includes(filter);
                    r.style.display = (tabMatch && searchMatch) ? '' : 'none';
                }});
            }}

            async function togglePlugin(btn, plugin, activate) {{
                const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || '';
                try {{
                    const res = await fetch('/d-admin/api/plugins', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json', 'X-WP-Nonce': nonce }},
                        body: JSON.stringify({{ plugin, active: activate }})
                    }});
                    if (res.ok) {{
                        showToast('✓ ' + plugin + (activate ? ' activated!' : ' deactivated!'));
                        setTimeout(() => window.location.reload(), 450);
                    }} else {{
                        showToast('Failed to update plugin state', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function applyBulkPluginAction() {{
                const action = document.getElementById('bulkPluginAction').value;
                if (!action) {{
                    showToast('Please select a bulk action', true);
                    return;
                }}
                const checked = Array.from(document.querySelectorAll('#pluginsTable tbody input[name="checked_plugins[]"]:checked'));
                if (!checked.length) {{
                    showToast('No plugins selected', true);
                    return;
                }}
                const activate = (action === 'activate');
                const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || '';
                showToast('Updating ' + checked.length + ' plugin(s)...');
                for (const cb of checked) {{
                    const plugin = cb.value;
                    try {{
                        await fetch('/d-admin/api/plugins', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json', 'X-WP-Nonce': nonce }},
                            body: JSON.stringify({{ plugin, active: activate }})
                        }});
                    }} catch (e) {{}}
                }}
                showToast('✓ Bulk update completed');
                setTimeout(() => window.location.reload(), 500);
            }}

            async function uninstallCustomPlugin(slug) {{
                if (!confirm('Uninstall custom plugin "' + slug + '" from SQLite?')) return;
                const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || '';
                try {{
                    const res = await fetch('/d-admin/api/plugin-install', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json', 'X-WP-Nonce': nonce }},
                        body: JSON.stringify({{ action: 'uninstall', slug }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Uninstalled ' + slug);
                        const row = document.getElementById('plugin-row-' + slug);
                        if (row) row.remove();
                    }} else {{
                        showToast('Failed to uninstall plugin', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        total_count = total_count,
        builtin_count = builtin_count,
        custom_count = custom_count,
        active_count = active_count,
        inactive_count = inactive_count,
        rows_html = rows_html
    );

    render_admin_shell("plugins", "Plugins", &body, settings)
}

// 2. Add New Plugin & Community SDK Installer (/d-admin/plugin-install)
pub fn render_plugin_install_view(settings: &DaypressSettingsResponse) -> String {
    let sdk_json = serde_json::to_string_pretty(&crate::plugins::starter_plugin_sdk_template())
        .unwrap_or_else(|_| "{}".to_string());
    let sdk_sample = escape_html(&sdk_json);
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Add Plugins &amp; Community Extensions</h1>
                <p class="page-subtitle">Install curated extensions, upload manifest bundles, or develop custom plugins using declarative hooks and schemas</p>
            </div>
            <div style="display: flex; gap: 10px;">
                <a href="/d-admin/api/plugin-sdk-template" class="btn btn-secondary">⬇ Download Starter SDK (.json)</a>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('pluginManifestFile').click()">Upload Manifest (.json)</button>
                <input type="file" id="pluginManifestFile" accept=".json,application/json" style="display: none;" onchange="handleManifestFileUpload(this)">
            </div>
        </div>

        <!-- Featured Community Extensions -->
        <div style="margin-bottom: 32px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
                <h3 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;">Featured 1-Click Community Extensions</h3>
                <span style="font-size: 12px; color: var(--ink-muted);">Ready to install directly into SQLite</span>
            </div>
            <div id="pluginCatalogGrid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px;">
                <div class="day-card" style="padding: 22px; display: flex; flex-direction: column; justify-content: space-between;">
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                            <h3 style="font-size: 15px; font-weight: 700; color: var(--ink);">Top Promo Announcement Bar</h3>
                            <span style="font-size: 11px; background: var(--moss-wash); color: var(--moss); padding: 2px 6px; border-radius: 4px; font-weight: 600;">Community SDK</span>
                        </div>
                        <p style="font-size: 13px; color: var(--ink-muted); margin: 8px 0 14px 0; line-height: 1.5;">
                            Sticky top banner with customizable colors, link button, and <code>[promo_cta]</code> shortcode for marketing campaigns.
                        </p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 14px; border-top: 1px solid var(--border-subtle);">
                        <span style="font-size: 12px; color: var(--ink-muted);">Hooks: body_top + shortcode</span>
                        <button class="btn btn-primary" style="font-size: 12px;" onclick="quickInstallPlugin('promo-announcement-bar')">Install Now</button>
                    </div>
                </div>

                <div class="day-card" style="padding: 22px; display: flex; flex-direction: column; justify-content: space-between;">
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                            <h3 style="font-size: 15px; font-weight: 700; color: var(--ink);">Back-to-Top Floating Button</h3>
                            <span style="font-size: 11px; background: var(--press-wash); color: var(--press); padding: 2px 6px; border-radius: 4px; font-weight: 600;">UX Utility</span>
                        </div>
                        <p style="font-size: 13px; color: var(--ink-muted); margin: 8px 0 14px 0; line-height: 1.5;">
                            Smooth-scroll floating button that fades in after scrolling down long pages, with customizable accent color and label.
                        </p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 14px; border-top: 1px solid var(--border-subtle);">
                        <span style="font-size: 12px; color: var(--ink-muted);">Hooks: footer</span>
                        <button class="btn btn-primary" style="font-size: 12px;" onclick="quickInstallPlugin('back-to-top-button')">Install Now</button>
                    </div>
                </div>

                <div class="day-card" style="padding: 22px; display: flex; flex-direction: column; justify-content: space-between;">
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                            <h3 style="font-size: 15px; font-weight: 700; color: var(--ink);">Scroll Reading Progress Bar</h3>
                            <span style="font-size: 11px; background: var(--press-wash); color: var(--press); padding: 2px 6px; border-radius: 4px; font-weight: 600;">Editorial</span>
                        </div>
                        <p style="font-size: 13px; color: var(--ink-muted); margin: 8px 0 14px 0; line-height: 1.5;">
                            Renders a smooth progress bar at the very top of articles that dynamically fills as the reader scrolls down.
                        </p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 14px; border-top: 1px solid var(--border-subtle);">
                        <span style="font-size: 12px; color: var(--ink-muted);">Hooks: body_top</span>
                        <button class="btn btn-primary" style="font-size: 12px;" onclick="quickInstallPlugin('reading-progress-bar')">Install Now</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Community Plugin Manifest Installer -->
        <div class="day-card" style="padding: 24px; margin-bottom: 24px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                <div>
                    <h3 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;">Community Plugin Manifest Developer (<code>daypress-plugin.json</code>)</h3>
                    <p style="font-size: 13px; color: var(--ink-muted); margin-top: 4px;">
                        Paste or edit custom JSON schemas (<code>settings_schema</code>) and HTML/CSS/JS hook templates (<code>head_html</code>, <code>body_top_html</code>, <code>content_before_html</code>, <code>footer_html</code>, <code>shortcode</code>). Daypress will validate and register the plugin.
                    </p>
                </div>
            </div>
            <textarea id="manifestJsonInput" rows="10" class="day-html-textarea" style="min-height: 200px; font-size: 12.5px; margin-bottom: 14px;">{sdk_sample}</textarea>
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <button type="button" class="btn btn-primary" onclick="installManifestFromTextarea()">Validate &amp; Install Plugin &rarr;</button>
                <a href="/d-admin/plugins" style="font-size: 13px; color: var(--press); text-decoration: none; font-weight: 600;">&larr; Return to Installed Plugins</a>
            </div>
        </div>

        <script>
            const CURATED_EXTENSIONS = {{
                "promo-announcement-bar": {{
                    "slug": "promo-announcement-bar",
                    "name": "Top Promo Announcement Bar",
                    "version": "1.0.0",
                    "author": "Community Developer",
                    "description": "Displays a customizable sticky announcement banner at the top of every public page and provides a [promo_cta] shortcode.",
                    "settings_url": "/d-admin/plugin-settings?plugin=promo-announcement-bar",
                    "is_builtin": false,
                    "settings_schema": [
                        {{ "key": "banner_text", "label": "Announcement Text", "field_type": "text", "default_value": "Fast 1-Click Migrations with Daypress CMS!", "description": "Banner message displayed across the top.", "options": [] }},
                        {{ "key": "bg_color", "label": "Background Color", "field_type": "color", "default_value": "#1e293b", "description": "Hex color code.", "options": [] }},
                        {{ "key": "text_color", "label": "Text Color", "field_type": "color", "default_value": "#f8fafc", "description": "Hex text color.", "options": [] }},
                        {{ "key": "cta_link", "label": "Call-to-Action Link URL", "field_type": "text", "default_value": "/contact/", "description": "Optional link URL for the banner button.", "options": [] }},
                        {{ "key": "cta_label", "label": "Button Label", "field_type": "text", "default_value": "Learn More →", "description": "Button text.", "options": [] }}
                    ],
                    "hooks": {{
                        "head_html": "<style>#dp-promo-bar {{ position: relative; z-index: 9998; padding: 10px 16px; text-align: center; font-size: 13px; font-weight: 500; display: flex; align-items: center; justify-content: center; gap: 12px; }} #dp-promo-bar a {{ text-decoration: underline; font-weight: 600; color: inherit; }}</style>",
                        "body_top_html": "<div id=\"dp-promo-bar\" style=\"background: {{{{bg_color}}}}; color: {{{{text_color}}}};\"><span>{{{{banner_text}}}}</span><a href=\"{{{{cta_link}}}}\">{{{{cta_label}}}}</a></div>",
                        "shortcode": "promo_cta",
                        "shortcode_html": "<div style=\"padding: 16px 20px; background: {{{{bg_color}}}}; color: {{{{text_color}}}}; border-radius: 8px; text-align: center; margin: 16px 0;\"><p style=\"margin: 0 0 10px 0; font-size: 15px; font-weight: 600;\">{{{{banner_text}}}}</p><a href=\"{{{{cta_link}}}}\" style=\"display: inline-block; padding: 6px 16px; background: #ffffff; color: #0f172a; border-radius: 4px; font-weight: 600; text-decoration: none;\">{{{{cta_label}}}}</a></div>"
                    }}
                }},
                "back-to-top-button": {{
                    "slug": "back-to-top-button",
                    "name": "Back-to-Top Floating Button",
                    "version": "1.0.0",
                    "author": "UX Studio",
                    "description": "Smooth-scroll floating button that fades in after scrolling down long pages, with customizable accent color and label.",
                    "settings_url": "/d-admin/plugin-settings?plugin=back-to-top-button",
                    "is_builtin": false,
                    "settings_schema": [
                        {{ "key": "button_label", "label": "Button Label", "field_type": "text", "default_value": "↑ Top", "description": "Button text.", "options": [] }},
                        {{ "key": "accent_color", "label": "Accent Color", "field_type": "color", "default_value": "#2563eb", "description": "Floating button background color.", "options": [] }}
                    ],
                    "hooks": {{
                        "footer_html": "<button id=\"dp-back-to-top\" type=\"button\" onclick=\"window.scrollTo({{top:0,behavior:'smooth'}})\" style=\"position:fixed;bottom:24px;right:24px;padding:10px 14px;background:{{{{accent_color}}}};color:#fff;border:none;border-radius:999px;font-weight:600;font-size:13px;cursor:pointer;box-shadow:0 8px 20px rgba(0,0,0,0.2);opacity:0;pointer-events:none;transition:opacity 0.2s ease;z-index:9997;\">{{{{button_label}}}}</button><script>window.addEventListener('scroll',()=>{{const b=document.getElementById('dp-back-to-top');if(!b)return;const show=window.scrollY>320;b.style.opacity=show?'1':'0';b.style.pointerEvents=show?'auto':'none';}});</script>"
                    }}
                }},
                "reading-progress-bar": {{
                    "slug": "reading-progress-bar",
                    "name": "Scroll Reading Progress Bar",
                    "version": "1.0.0",
                    "author": "Editorial Studio",
                    "description": "Renders a smooth progress bar at the very top of articles that fills as the reader scrolls down.",
                    "settings_url": "/d-admin/plugin-settings?plugin=reading-progress-bar",
                    "is_builtin": false,
                    "settings_schema": [
                        {{ "key": "bar_color", "label": "Progress Bar Color", "field_type": "color", "default_value": "#2563eb", "description": "Color of the reading bar.", "options": [] }},
                        {{ "key": "bar_height", "label": "Height (px)", "field_type": "text", "default_value": "4", "description": "Thickness of the progress bar.", "options": [] }}
                    ],
                    "hooks": {{
                        "body_top_html": "<div id=\"dp-reading-bar\" style=\"position: fixed; top: 0; left: 0; width: 0%; height: {{{{bar_height}}}}px; background: {{{{bar_color}}}}; z-index: 99999; transition: width 0.1s linear;\"></div><script>window.addEventListener('scroll', () => {{ const h = document.documentElement.scrollHeight - window.innerHeight; const pct = h > 0 ? (window.scrollY / h) * 100 : 0; const el = document.getElementById('dp-reading-bar'); if (el) el.style.width = Math.min(100, Math.max(0, pct)) + '%'; }});</script>"
                    }}
                }}
            }};

            async function quickInstallPlugin(slug) {{
                const manifest = CURATED_EXTENSIONS[slug];
                if (!manifest) return;
                showToast('Installing ' + manifest.name + '...');
                try {{
                    const res = await fetch('/d-admin/api/plugin-install', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ manifest }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.success) {{
                        showToast('✓ ' + manifest.name + ' installed & activated!');
                        setTimeout(() => {{
                            window.location.href = '/d-admin/plugin-settings?plugin=' + encodeURIComponent(manifest.slug);
                        }}, 600);
                    }} else {{
                        showToast('Install error: ' + (data.error || 'Unknown error'), true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function handleManifestFileUpload(input) {{
                if (!input.files || !input.files[0]) return;
                try {{
                    const text = await input.files[0].text();
                    document.getElementById('manifestJsonInput').value = text;
                    await installManifestFromTextarea();
                }} catch (e) {{
                    showToast('Failed to read manifest file: ' + e, true);
                }}
            }}

            async function installManifestFromTextarea() {{
                const raw = document.getElementById('manifestJsonInput').value.trim();
                if (!raw) {{
                    showToast('Please paste a plugin manifest JSON first', true);
                    return;
                }}
                let manifest;
                try {{
                    manifest = JSON.parse(raw);
                }} catch (e) {{
                    showToast('Invalid JSON syntax: ' + e.message, true);
                    return;
                }}
                try {{
                    const res = await fetch('/d-admin/api/plugin-install', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ manifest }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.success) {{
                        showToast('✓ Plugin "' + manifest.name + '" installed & activated!');
                        setTimeout(() => {{
                            window.location.href = '/d-admin/plugin-settings?plugin=' + encodeURIComponent(manifest.slug);
                        }}, 600);
                    }} else {{
                        showToast('Install error: ' + (data.error || 'Unknown error'), true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        sdk_sample = sdk_sample
    );

    render_admin_shell("plugin-install", "Add Plugins", &body, settings)
}

// 3. Plugin File Editor (/d-admin/plugin-editor)
pub fn render_plugin_editor_view(settings: &DaypressSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Plugin File Editor</h1>
                <p class="page-subtitle">View and customize plugin module logic with strict memory-safety isolation</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="savePluginFile()">Update File</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 260px; gap: 20px;">
            <div class="day-card" style="padding: 20px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                    <span id="activePluginFileName" style="font-family: var(--font-mono); font-size: 13px; font-weight: 600; color: var(--ink);">daypress-optimizer.rs (Active Plugin)</span>
                    <span style="font-size: 12px; color: var(--moss); font-weight: 600;">✓ Safe Sandbox</span>
                </div>
                <textarea id="pluginFileContent" class="day-html-textarea" style="min-height: 500px;">// Daypress WAL Optimizer Plugin
// Version: 1.0.0
// Description: Automated Write-Ahead-Log checkpointing

pub struct WalOptimizer {
    interval_seconds: u64,
}

impl WalOptimizer {
    pub fn new() -> Self {
        Self {
            interval_seconds: 300,
        }
    }

    pub fn run_checkpoint(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }
}
</textarea>
            </div>

            <div class="day-card" style="padding: 20px; height: fit-content;">
                <h4 style="font-size: 13.5px; font-weight: 700; color: var(--ink); margin-bottom: 12px;">Plugin Files</h4>
                <div style="display: flex; flex-direction: column; gap: 6px;">
                    <a href="javascript:void(0)" onclick="loadPluginFile('daypress-optimizer.rs')" style="padding: 6px 10px; border-radius: 4px; background: var(--press-wash); color: var(--press); font-weight: 600; font-size: 12.5px; text-decoration: none;">daypress-optimizer.rs</a>
                    <a href="javascript:void(0)" onclick="loadPluginFile('webp-engine.rs')" style="padding: 6px 10px; border-radius: 4px; color: var(--ink-muted); font-size: 12.5px; text-decoration: none;">webp-engine.rs</a>
                    <a href="javascript:void(0)" onclick="loadPluginFile('wxr-stream.rs')" style="padding: 6px 10px; border-radius: 4px; color: var(--ink-muted); font-size: 12.5px; text-decoration: none;">wxr-stream.rs</a>
                </div>
                <div style="margin-top: 20px; padding-top: 14px; border-top: 1px solid var(--border-subtle); font-size: 11.5px; color: var(--ink-muted);">
                    <strong>Security:</strong> Direct execution is sandboxed against root filesystem writes.
                </div>
            </div>
        </div>

        <script>
            let currentPluginFile = 'daypress-optimizer.rs';
            async function loadPluginFile(name) {
                currentPluginFile = name;
                document.getElementById('activePluginFileName').textContent = name + ' (Plugin File)';
                try {
                    const res = await fetch('/d-admin/api/theme-file?file=' + encodeURIComponent(name));
                    if (res.ok) {
                        const data = await res.json();
                        if (data.content && data.content.trim().length > 0) {
                            document.getElementById('pluginFileContent').value = data.content;
                        }
                        showToast('Loaded ' + name);
                    }
                } catch (e) {
                    showToast('Failed to load ' + name, true);
                }
            }
            async function savePluginFile() {
                const content = document.getElementById('pluginFileContent').value;
                try {
                    const res = await fetch('/d-admin/api/theme-file', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ file: currentPluginFile, content })
                    });
                    if (res.ok) {
                        showToast('✓ Plugin module validated and saved in SQLite!');
                    } else {
                        showToast('Failed to save plugin file', true);
                    }
                } catch (e) {
                    showToast('Network error: ' + e, true);
                }
            }
        </script>
    "##;

    render_admin_shell("plugin-editor", "Plugin File Editor", body, settings)
}

// 4. Dynamic Plugin Settings Page (/d-admin/plugin-settings?plugin=<slug>)
pub fn render_plugin_settings_view(
    settings: &DaypressSettingsResponse,
    plugin: Option<&InstalledPluginRecord>,
) -> String {
    let Some(record) = plugin else {
        let not_found = r#"<div class="day-card" style="padding: 32px; text-align: center;">
            <h2>Plugin Not Found</h2>
            <p style="color:var(--ink-muted);margin:8px 0 16px 0;">The requested plugin is not installed in SQLite.</p>
            <a href="/d-admin/plugins" class="btn btn-primary">&larr; Back to Plugins</a>
        </div>"#;
        return render_admin_shell("plugin-settings", "Plugin Settings", not_found, settings);
    };

    let slug = escape_html(&record.manifest.slug);
    let name = escape_html(&record.manifest.name);
    let desc = escape_html(&record.manifest.description);
    let active_badge = if record.active {
        r#"<span class="badge badge-published">Active</span>"#
    } else {
        r#"<span class="badge badge-draft">Inactive</span>"#
    };
    let toggle_btn = if record.active {
        format!(
            r#"<button type="button" class="btn btn-secondary" onclick="togglePluginActiveState('{slug}', false)">Deactivate Module</button>"#,
            slug = slug
        )
    } else {
        format!(
            r#"<button type="button" class="btn btn-secondary" style="border-color:var(--moss);color:var(--moss);font-weight:700;" onclick="togglePluginActiveState('{slug}', true)">Activate Module</button>"#,
            slug = slug
        )
    };
    let nav_tab = match record.manifest.slug.as_str() {
        "whatsapp-chat" => "whatsapp-chat",
        "code-snippets" => "code-snippets",
        _ => "plugin-settings",
    };

    let mut fields_html = String::new();
    for field in &record.manifest.settings_schema {
        let key = escape_html(&field.key);
        let label = escape_html(&field.label);
        let current_val = record
            .config
            .get(&field.key)
            .cloned()
            .unwrap_or_else(|| field.default_value.clone());
        let val_esc = escape_html(&current_val);

        let input_control = match field.field_type.as_str() {
            "textarea" => format!(
                r#"<textarea class="form-textarea plugin-cfg-field" data-key="{key}" rows="5" style="font-family:var(--font-mono);font-size:12.5px;">{val_esc}</textarea>"#
            ),
            "color" => format!(
                r#"<input type="color" class="form-input plugin-cfg-field" data-key="{key}" value="{val_esc}" style="width:90px;height:38px;padding:3px;" />"#
            ),
            "select" => {
                let mut opts = String::new();
                for opt in &field.options {
                    let opt_esc = escape_html(opt);
                    let sel = if opt == &current_val { "selected" } else { "" };
                    opts.push_str(&format!(
                        r#"<option value="{opt_esc}" {sel}>{opt_esc}</option>"#
                    ));
                }
                format!(
                    r#"<select class="form-select plugin-cfg-field" data-key="{key}">{opts}</select>"#
                )
            }
            "boolean" => {
                let checked = if current_val == "true" || current_val == "1" {
                    "checked"
                } else {
                    ""
                };
                format!(
                    r#"<label style="display:flex;align-items:center;gap:8px;cursor:pointer;">
                        <input type="checkbox" class="plugin-cfg-field" data-key="{key}" data-type="boolean" {checked} />
                        <span>Enable {label}</span>
                    </label>"#
                )
            }
            _ => format!(
                r#"<input type="text" class="form-input plugin-cfg-field" data-key="{key}" value="{val_esc}" />"#
            ),
        };

        fields_html.push_str(&format!(
            r#"<div class="form-group">
                <label class="form-label">{label} <span style="font-family:var(--font-mono);font-size:11px;color:var(--ink-muted);font-weight:400;">({{{{{key}}}}})</span></label>
                {input_control}
            </div>"#
        ));
    }

    if fields_html.is_empty() {
        fields_html = r#"<p style="color:var(--ink-muted);">This plugin does not declare any custom settings fields.</p>"#.to_string();
    }

    let shortcode_info = if !record.manifest.hooks.shortcode.is_empty() {
        format!(
            r#"<div class="day-card" style="padding:18px 24px;margin-bottom:20px;background:var(--paper-raised);">
                <strong>Shortcode Available:</strong> Embed <code style="background:var(--paper-sunk);padding:2px 8px;border-radius:4px;font-family:var(--font-mono);">[{}]</code> inside any post or page.
            </div>"#,
            escape_html(&record.manifest.hooks.shortcode)
        )
    } else {
        String::new()
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <div style="display:flex;align-items:center;gap:10px;">
                    <h1 class="page-title">{name}</h1>
                    {active_badge}
                </div>
                <p class="page-subtitle">{desc}</p>
            </div>
            <div style="display:flex;gap:10px;">
                <a href="/d-admin/plugins" class="btn btn-secondary">&larr; All Plugins</a>
                {toggle_btn}
                <button type="button" class="btn btn-primary" onclick="savePluginSettings('{slug}')">Save Settings to SQLite</button>
            </div>
        </div>

        {shortcode_info}

        <div class="day-card" style="padding: 24px; max-width: 780px;">
            {fields_html}
            <div style="padding-top:16px;border-top:1px solid var(--border-subtle);display:flex;justify-content:flex-end;">
                <button type="button" class="btn btn-primary" onclick="savePluginSettings('{slug}')">Save Settings</button>
            </div>
        </div>

        <script>
            async function togglePluginActiveState(plugin, activate) {{
                const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || '';
                try {{
                    const res = await fetch('/d-admin/api/plugins', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json', 'X-WP-Nonce': nonce }},
                        body: JSON.stringify({{ plugin, active: activate }})
                    }});
                    if (res.ok) {{
                        showToast('✓ ' + plugin + (activate ? ' activated!' : ' deactivated!'));
                        setTimeout(() => window.location.reload(), 350);
                    }} else {{
                        showToast('Failed to update module state', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function savePluginSettings(slug) {{
                const nonce = (window.wpApiSettings && window.wpApiSettings.nonce) || '';
                const config = {{}};
                document.querySelectorAll('.plugin-cfg-field').forEach(el => {{
                    const key = el.getAttribute('data-key');
                    if (!key) return;
                    if (el.getAttribute('data-type') === 'boolean') {{
                        config[key] = el.checked ? 'true' : 'false';
                    }} else {{
                        config[key] = el.value;
                    }}
                }});
                try {{
                    const res = await fetch('/d-admin/api/plugin-settings', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json', 'X-WP-Nonce': nonce }},
                        body: JSON.stringify({{ slug, config }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Settings persisted to SQLite!');
                    }} else {{
                        showToast('Failed to save plugin settings', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        name = name,
        active_badge = active_badge,
        toggle_btn = toggle_btn,
        desc = desc,
        slug = slug,
        shortcode_info = shortcode_info,
        fields_html = fields_html
    );

    render_admin_shell(nav_tab, &name, &body, settings)
}

// 5. Full SEO Toolkit & Redirection Suite (/d-admin/seo)
pub fn render_seo_view(
    settings: &DaypressSettingsResponse,
    seo: &GlobalSeoSettings,
    redirects: &[RedirectRule],
    logs_404: &[NotFoundLogEntry],
) -> String {
    let breadcrumbs_checked = if seo.enable_breadcrumbs {
        "checked"
    } else {
        ""
    };
    let auto_redir_checked = if seo.auto_redirect_slug_changes {
        "checked"
    } else {
        ""
    };

    let mut redirect_rows = String::new();
    for r in redirects {
        redirect_rows.push_str(&format!(
            r#"<tr id="redir-row-{id}">
                <td style="font-family:var(--font-mono);font-weight:600;color:var(--ink);">{src}</td>
                <td style="font-family:var(--font-mono);color:var(--press);">{dst}</td>
                <td><span class="badge badge-published">{code}</span></td>
                <td style="font-family:var(--font-mono);">{hits}</td>
                <td>
                    <button type="button" class="btn btn-danger" style="padding:3px 8px;font-size:11.5px;" onclick="deleteRedirect({id})">Delete</button>
                </td>
            </tr>"#,
            id = r.id,
            src = escape_html(&r.source_path),
            dst = escape_html(&r.target_url),
            code = r.status_code,
            hits = r.hits
        ));
    }
    if redirects.is_empty() {
        redirect_rows = r#"<tr><td colspan="5" style="text-align:center;color:var(--ink-muted);padding:20px;">No 301/302 redirect rules configured yet. Changing a published post's slug automatically registers a 301 redirect here.</td></tr>"#.to_string();
    }

    let mut log_404_rows = String::new();
    for entry in logs_404 {
        let path_esc = escape_html(&entry.path);
        log_404_rows.push_str(&format!(
            r#"<tr>
                <td style="font-family:var(--font-mono);color:var(--oxide);font-weight:600;">{path}</td>
                <td style="font-weight:700;">{hits}</td>
                <td style="font-size:12px;color:var(--ink-muted);">{last_seen}</td>
                <td>
                    <button type="button" class="btn btn-secondary" style="padding:3px 8px;font-size:11.5px;" onclick="prefillRedirect('{path}')">+ 301 Redirect</button>
                </td>
            </tr>"#,
            path = path_esc,
            hits = entry.hits,
            last_seen = escape_html(&entry.last_seen)
        ));
    }
    if logs_404.is_empty() {
        log_404_rows = r#"<tr><td colspan="4" style="text-align:center;color:var(--ink-muted);padding:20px;">Zero 404 Not Found hits recorded.</td></tr>"#.to_string();
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <div style="display:flex;align-items:center;gap:10px;">
                    <h1 class="page-title">SEO Toolkit &amp; Redirection Suite</h1>
                    <span class="badge badge-published">Yoast + RankMath + Redirection Parity</span>
                </div>
                <p class="page-subtitle">Global Schema.org JSON-LD, OpenGraph, XML Sitemaps, 301/302 Redirect Manager, and 404 Telemetry</p>
            </div>
            <div style="display:flex;gap:10px;">
                <a href="/sitemap.xml" target="_blank" class="btn btn-secondary">XML Sitemap ↗</a>
                <a href="/robots.txt" target="_blank" class="btn btn-secondary">robots.txt ↗</a>
                <button type="button" class="btn btn-primary" onclick="saveGlobalSeo()">Save Global SEO</button>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:1fr 1fr;gap:24px;">
            <!-- Left Column: Global SEO & Schema.org -->
            <div class="day-card" style="padding:24px;">
                <h3 style="font-size:16px;font-weight:700;color:var(--ink);margin-bottom:16px;">Global SEO &amp; Schema.org Knowledge Graph</h3>

                <div style="display:grid;grid-template-columns:120px 1fr;gap:14px;">
                    <div class="form-group">
                        <label class="form-label">Title Separator</label>
                        <input type="text" id="seo_sep" class="form-input" value="{sep}" />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Twitter / X Handle</label>
                        <input type="text" id="seo_twitter" class="form-input" value="{twitter}" placeholder="@daypress" />
                    </div>
                </div>

                <div class="form-group">
                    <label class="form-label">Organization / Publisher Name</label>
                    <input type="text" id="seo_org_name" class="form-input" value="{org_name}" placeholder="Daypress Inc." />
                </div>

                <div class="form-group">
                    <label class="form-label">Organization Logo URL</label>
                    <input type="text" id="seo_org_logo" class="form-input" value="{org_logo}" placeholder="/uploads/logo.png" />
                </div>

                <div class="form-group">
                    <label class="form-label">Default OpenGraph Social Image URL</label>
                    <input type="text" id="seo_default_og" class="form-input" value="{default_og}" placeholder="/uploads/default-og.webp" />
                </div>

                <div class="form-group">
                    <label class="form-label">Custom <code>robots.txt</code> Rules <span style="font-weight:400;color:var(--ink-muted);">(Leave blank for dynamic default)</span></label>
                    <textarea id="seo_robots" rows="4" class="form-textarea" style="font-family:var(--font-mono);font-size:12px;" placeholder="User-agent: *&#10;Allow: /">{robots}</textarea>
                </div>

                <label style="display:flex;align-items:center;gap:8px;font-size:13px;cursor:pointer;margin-bottom:10px;">
                    <input type="checkbox" id="seo_breadcrumbs" {breadcrumbs_checked} />
                    <span>Enable Schema.org <code>BreadcrumbList</code> structured data</span>
                </label>

                <label style="display:flex;align-items:center;gap:8px;font-size:13px;cursor:pointer;margin-bottom:16px;">
                    <input type="checkbox" id="seo_auto_redir" {auto_redir_checked} />
                    <span>Automatically create a 301 redirect when a published post's slug changes</span>
                </label>

                <button type="button" class="btn btn-primary" onclick="saveGlobalSeo()">Save Global SEO Settings</button>
            </div>

            <!-- Right Column: 301/302 Redirect Manager & 404 Monitor -->
            <div style="display:flex;flex-direction:column;gap:24px;">
                <div class="day-card" style="padding:24px;">
                    <h3 style="font-size:16px;font-weight:700;color:var(--ink);margin-bottom:6px;">301 / 302 Redirect Manager</h3>
                    <p style="font-size:12.5px;color:var(--ink-muted);margin-bottom:14px;">Zero-latency SQLite redirect lookup executed before route resolution.</p>

                    <div style="display:grid;grid-template-columns:1fr 1fr 90px auto;gap:8px;margin-bottom:16px;">
                        <input type="text" id="newRedirSource" class="form-input" placeholder="/old-url-slug" />
                        <input type="text" id="newRedirTarget" class="form-input" placeholder="/new-url-slug/" />
                        <select id="newRedirCode" class="form-select">
                            <option value="301">301</option>
                            <option value="302">302</option>
                        </select>
                        <button type="button" class="btn btn-primary" onclick="addRedirectRule()">+ Add</button>
                    </div>

                    <div class="day-table-wrapper">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Source Path</th>
                                    <th>Target URL</th>
                                    <th>Type</th>
                                    <th>Hits</th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                                {redirect_rows}
                            </tbody>
                        </table>
                    </div>
                </div>

                <!-- 404 Not Found Monitor -->
                <div class="day-card" style="padding:24px;">
                    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
                        <div>
                            <h3 style="font-size:16px;font-weight:700;color:var(--ink);margin:0;">404 Not Found Monitor</h3>
                            <p style="font-size:12.5px;color:var(--ink-muted);margin-top:2px;">Broken inbound links captured automatically in SQLite.</p>
                        </div>
                        <button type="button" class="btn btn-secondary" style="font-size:12px;" onclick="clear404Logs()">Clear Log</button>
                    </div>

                    <div class="day-table-wrapper">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Missing Path</th>
                                    <th>Hits</th>
                                    <th>Last Seen</th>
                                    <th>Action</th>
                                </tr>
                            </thead>
                            <tbody>
                                {log_404_rows}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>

        <script>
            async function saveGlobalSeo() {{
                const payload = {{
                    title_separator: document.getElementById('seo_sep').value || '–',
                    org_name: document.getElementById('seo_org_name').value.trim(),
                    org_logo_url: document.getElementById('seo_org_logo').value.trim(),
                    default_og_image: document.getElementById('seo_default_og').value.trim(),
                    twitter_site: document.getElementById('seo_twitter').value.trim(),
                    enable_breadcrumbs: document.getElementById('seo_breadcrumbs').checked,
                    auto_redirect_slug_changes: document.getElementById('seo_auto_redir').checked,
                    custom_robots_txt: document.getElementById('seo_robots').value
                }};
                try {{
                    const res = await fetch('/d-admin/api/seo', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});
                    if (res.ok) {{
                        showToast('✓ Global SEO settings saved to SQLite!');
                    }} else {{
                        showToast('Failed to save SEO settings', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function addRedirectRule() {{
                const source_path = document.getElementById('newRedirSource').value.trim();
                const target_url = document.getElementById('newRedirTarget').value.trim();
                const status_code = parseInt(document.getElementById('newRedirCode').value) || 301;
                if (!source_path || !target_url) {{
                    showToast('Enter both source path and target URL', true);
                    return;
                }}
                const res = await fetch('/d-admin/api/redirects', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'create', source_path, target_url, status_code }})
                }});
                if (res.ok) {{
                    showToast('✓ Redirect rule saved!');
                    setTimeout(() => location.reload(), 400);
                }} else {{
                    showToast('Failed to create redirect rule', true);
                }}
            }}

            async function deleteRedirect(id) {{
                const res = await fetch('/d-admin/api/redirects', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'delete', id }})
                }});
                if (res.ok) {{
                    const row = document.getElementById('redir-row-' + id);
                    if (row) row.remove();
                    showToast('✓ Redirect deleted');
                }}
            }}

            function prefillRedirect(path) {{
                document.getElementById('newRedirSource').value = path;
                document.getElementById('newRedirTarget').focus();
                showToast('Enter destination URL for ' + path);
            }}

            async function clear404Logs() {{
                const res = await fetch('/d-admin/api/redirects', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'clear_404' }})
                }});
                if (res.ok) {{
                    showToast('✓ 404 log cleared');
                    setTimeout(() => location.reload(), 400);
                }}
            }}
        </script>
        "##,
        sep = escape_html(&seo.title_separator),
        twitter = escape_html(&seo.twitter_site),
        org_name = escape_html(&seo.org_name),
        org_logo = escape_html(&seo.org_logo_url),
        default_og = escape_html(&seo.default_og_image),
        robots = escape_html(&seo.custom_robots_txt),
        breadcrumbs_checked = breadcrumbs_checked,
        auto_redir_checked = auto_redir_checked,
        redirect_rows = redirect_rows,
        log_404_rows = log_404_rows
    );

    render_admin_shell("seo", "SEO & Redirects", &body, settings)
}

// 6. Daypress FormStudio & Submissions Vault (/d-admin/forms)
pub fn render_forms_view(
    settings: &DaypressSettingsResponse,
    forms: &[(DaypressForm, i64)],
    entries: &[FormEntryItem],
    selected_form_id: Option<i64>,
) -> String {
    render_forms_view_with_filters(settings, forms, entries, selected_form_id, "all", "")
}

pub fn render_forms_view_with_filters(
    settings: &DaypressSettingsResponse,
    forms: &[(DaypressForm, i64)],
    entries: &[FormEntryItem],
    selected_form_id: Option<i64>,
    status_filter: &str,
    search_query: &str,
) -> String {
    let active_form = selected_form_id
        .and_then(|id| forms.iter().find(|(f, _)| f.id == id).map(|(f, _)| f))
        .or_else(|| forms.first().map(|(f, _)| f));

    let mut forms_list_html = String::new();
    for (f, entry_count) in forms {
        let is_sel = active_form.map(|af| af.id) == Some(f.id);
        let row_bg = if is_sel {
            "background:var(--press-wash);"
        } else {
            ""
        };
        let parsed_fields: Vec<FormFieldSchema> =
            serde_json::from_str(&f.schema_json).unwrap_or_default();
        forms_list_html.push_str(&format!(
            r#"<tr style="{row_bg}">
                <td>
                    <strong><a href="/d-admin/forms?form_id={id}" style="color:var(--ink);text-decoration:none;">{title}</a></strong>
                </td>
                <td>
                    <code style="background:var(--paper-sunk);padding:2px 6px;border-radius:4px;font-family:var(--font-mono);font-size:12px;">[daypress_form id="{id}"]</code>
                </td>
                <td>{field_count} fields &bull; {entry_count} entries</td>
                <td style="display:flex;gap:6px;">
                    <a href="/d-admin/forms?form_id={id}" class="btn btn-secondary" style="padding:3px 8px;font-size:11.5px;">Edit</a>
                    <button type="button" class="btn btn-danger" style="padding:3px 8px;font-size:11.5px;" onclick="deleteForm({id})">&times;</button>
                </td>
            </tr>"#,
            row_bg = row_bg,
            id = f.id,
            title = escape_html(&f.title),
            field_count = parsed_fields.len(),
            entry_count = entry_count
        ));
    }

    let (edit_id, edit_title, parsed_settings, edit_fields_json) = if let Some(f) = active_form {
        let parsed_fields: Vec<FormFieldSchema> =
            serde_json::from_str(&f.schema_json).unwrap_or_default();
        let ps: FormSettings = serde_json::from_str(&f.settings_json).unwrap_or_default();
        (
            f.id,
            escape_html(&f.title),
            ps,
            serde_json::to_string_pretty(&parsed_fields).unwrap_or_else(|_| "[]".to_string()),
        )
    } else {
        (
            0,
            "Contact Form".to_string(),
            FormSettings::default(),
            "[]".to_string(),
        )
    };

    let auto_resp_checked = if parsed_settings.auto_responder_enabled {
        "checked"
    } else {
        ""
    };
    let step_titles_csv = escape_html(&parsed_settings.step_titles.join(", "));
    let cond_routes_json = serde_json::to_string_pretty(&parsed_settings.conditional_routes)
        .unwrap_or_else(|_| "[]".to_string());

    let mut entries_rows = String::new();
    for e in entries {
        let mut pairs = Vec::new();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&e.data_json) {
            if let Some(obj) = val.as_object() {
                for (k, v) in obj {
                    let val_str = v
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| v.to_string());
                    pairs.push(format!(
                        "<strong>{}:</strong> {}",
                        escape_html(k),
                        escape_html(&val_str)
                    ));
                }
            }
        }
        let mut summary_html = if pairs.is_empty() {
            escape_html(&e.data_json)
        } else {
            pairs.join(" &bull; ")
        };

        if !e.attachments.is_empty() {
            let mut att_links = Vec::new();
            for att in &e.attachments {
                let fn_esc = escape_html(&att.filename);
                att_links.push(format!(
                    r#"<a href="/d-admin/forms/attachment/{eid}/{fn_esc}" target="_blank" style="display:inline-flex;align-items:center;gap:4px;background:var(--press-wash);color:var(--press);padding:2px 8px;border-radius:4px;font-size:11.5px;font-weight:600;text-decoration:none;">&#128206; {fn_esc}</a>"#,
                    eid = e.id,
                    fn_esc = fn_esc
                ));
            }
            summary_html.push_str(&format!(
                r#"<div style="margin-top:6px;display:flex;gap:6px;flex-wrap:wrap;">{}</div>"#,
                att_links.join("")
            ));
        }

        let star_char = if e.is_starred { "★" } else { "☆" };
        let star_color = if e.is_starred {
            "#f59e0b"
        } else {
            "var(--ink-muted)"
        };

        let status_badge = match e.status.as_str() {
            "unread" => {
                r#"<span class="badge" style="background:var(--press-wash);color:var(--press);font-size:11px;">Unread</span>"#
            }
            "spam" => {
                r#"<span class="badge" style="background:#fee2e2;color:#b91c1c;font-size:11px;">Spam</span>"#
            }
            "trash" => r#"<span class="badge badge-draft" style="font-size:11px;">Trash</span>"#,
            _ => r#"<span class="badge badge-published" style="font-size:11px;">Read</span>"#,
        };

        let email_badge = match e.email_status.as_str() {
            "sent" => {
                r#"<span class="badge badge-published" style="font-size:11px;">✓ Sent</span>"#
                    .to_string()
            }
            "failed" => format!(
                r#"<span class="badge" style="background:#fee2e2;color:#b91c1c;font-size:11px;" title="{}">✕ Failed</span>"#,
                escape_html(e.email_error.as_deref().unwrap_or(""))
            ),
            "queued" => r#"<span class="badge badge-draft" style="font-size:11px;">Queued</span>"#
                .to_string(),
            _ => r#"<span style="font-size:11.5px;color:var(--ink-muted);">Disabled</span>"#
                .to_string(),
        };

        entries_rows.push_str(&format!(
            r#"<tr id="entry-row-{id}">
                <td style="width:36px;text-align:center;">
                    <button type="button" onclick="toggleEntryStar({id})" style="background:none;border:none;cursor:pointer;font-size:16px;color:{star_color};" title="Star entry">{star_char}</button>
                </td>
                <td style="font-family:var(--font-mono);font-size:12px;">#{id}<div style="font-size:11px;color:var(--ink-muted);">Form #{form_id}</div></td>
                <td>{status_badge}</td>
                <td style="font-size:13px;line-height:1.5;">{summary_html}</td>
                <td>{email_badge}</td>
                <td style="font-size:12px;color:var(--ink-muted);">{created_at}</td>
                <td style="display:flex;gap:6px;flex-wrap:wrap;">
                    <button type="button" class="btn btn-secondary" style="padding:3px 8px;font-size:11.5px;" onclick="retryEntryEmail({id})" title="Re-send Admin &amp; Auto-Responder Email">↻ Retry Email</button>
                    <select onchange="updateEntryStatus({id}, this.value)" class="form-select" style="width:auto;padding:2px 6px;font-size:11.5px;">
                        <option value="">Move to...</option>
                        <option value="read">Mark Read</option>
                        <option value="unread">Mark Unread</option>
                        <option value="spam">Spam</option>
                        <option value="trash">Trash</option>
                    </select>
                    <button type="button" class="btn btn-danger" style="padding:3px 8px;font-size:11.5px;" onclick="deleteFormEntry({id})">Delete</button>
                </td>
            </tr>"#,
            id = e.id,
            form_id = e.form_id,
            star_color = star_color,
            star_char = star_char,
            status_badge = status_badge,
            summary_html = summary_html,
            email_badge = email_badge,
            created_at = escape_html(&e.created_at)
        ));
    }
    if entries.is_empty() {
        entries_rows = r#"<tr><td colspan="7" style="text-align:center;color:var(--ink-muted);padding:24px;">No form submissions found in this view.</td></tr>"#.to_string();
    }

    let filter_tab = |st: &str, label: &str| -> String {
        let active = status_filter == st;
        let style = if active {
            "background:var(--ink);color:var(--paper-raised);"
        } else {
            ""
        };
        let fid_q = selected_form_id
            .map(|id| format!("form_id={}&", id))
            .unwrap_or_default();
        format!(
            r#"<a href="/d-admin/forms?{fid_q}status={st}" class="btn btn-secondary" style="padding:5px 11px;font-size:12px;text-decoration:none;{style}">{label}</a>"#
        )
    };
    let status_tabs = format!(
        "{}{}{}{}{}{}",
        filter_tab("all", "Inbox (Active)"),
        filter_tab("unread", "Unread"),
        filter_tab("starred", "★ Starred"),
        filter_tab("failed_email", "⚠ Failed Email"),
        filter_tab("spam", "Spam Quarantine"),
        filter_tab("trash", "Trash"),
    );

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <div style="display:flex;align-items:center;gap:10px;">
                    <h1 class="page-title">Daypress FormStudio &amp; Submissions Vault</h1>
                    <span class="badge badge-published">WPForms / Gravity Forms / CF7 Parity</span>
                </div>
                <p class="page-subtitle">Visual multi-step form builder, conditional logic, visitor file uploads, anti-spam quarantine, and fail-safe SQLite Submissions Vault</p>
            </div>
            <div style="display:flex;gap:10px;">
                <a href="/d-admin/api/forms/export-csv" class="btn btn-secondary">⬇ Export CSV</a>
                <a href="/d-admin/api/forms/export-json" class="btn btn-secondary">⬇ Export JSON</a>
                <button type="button" class="btn btn-primary" onclick="createNewForm()">+ New Form</button>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:360px 1fr;gap:24px;margin-bottom:24px;align-items:start;">
            <!-- Left: Configured Forms List -->
            <div class="day-card" style="padding:20px;">
                <h3 style="font-size:15px;font-weight:700;color:var(--ink);margin:0 0 12px 0;">Configured Forms</h3>
                <div class="day-table-wrapper">
                    <table class="day-table">
                        <thead>
                            <tr>
                                <th>Form</th>
                                <th>Shortcode</th>
                                <th>Stats</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {forms_list_html}
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- Right: Visual Field & Step Builder + Notifications & Anti-Spam -->
            <div class="day-card" style="padding:24px;">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px;flex-wrap:wrap;gap:10px;">
                    <div>
                        <h3 style="font-size:16px;font-weight:700;color:var(--ink);margin:0;">FormStudio Builder <span id="formEditBadge">#{edit_id}</span></h3>
                        <span style="font-size:12px;color:var(--ink-muted);">Embed anywhere via shortcode <code style="background:var(--press-wash);color:var(--press);padding:2px 6px;border-radius:4px;">[daypress_form id="{edit_id}"]</code> or the Daystudio Form block</span>
                    </div>
                    <button type="button" class="btn btn-primary" onclick="saveCurrentForm()">Save Form to SQLite</button>
                </div>
                <input type="hidden" id="form_id" value="{edit_id}" />

                <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:12px;margin-bottom:14px;">
                    <div class="form-group" style="margin:0;">
                        <label class="form-label">Form Title</label>
                        <input type="text" id="form_title" class="form-input" value="{edit_title}" />
                    </div>
                    <div class="form-group" style="margin:0;">
                        <label class="form-label">Submit Button Label</label>
                        <input type="text" id="form_submit_label" class="form-input" value="{edit_submit}" />
                    </div>
                    <div class="form-group" style="margin:0;">
                        <label class="form-label">Multi-Step Titles (comma-separated)</label>
                        <input type="text" id="form_step_titles" class="form-input" value="{step_titles_csv}" placeholder="Step 1: Contact, Step 2: Project Details" />
                    </div>
                </div>

                <!-- Visual Field Palette & Canvas -->
                <div style="border:1px solid var(--border);border-radius:8px;padding:16px;margin-bottom:16px;background:var(--paper-raised);">
                    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;flex-wrap:wrap;gap:8px;">
                        <strong style="font-size:13.5px;color:var(--ink);">Visual Field &amp; Step Designer</strong>
                        <div style="display:flex;gap:6px;flex-wrap:wrap;">
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('text')">+ Text</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('email')">+ Email</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('tel')">+ Phone</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('textarea')">+ Textarea</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('select')">+ Select</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('radio')">+ Radio</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('checkbox_group')">+ Checkboxes</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('file')">+ File Upload</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('calculated')">+ Calculated</button>
                            <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="addVisualField('consent')">+ GDPR Consent</button>
                        </div>
                    </div>
                    <div id="visualFieldsList" style="display:flex;flex-direction:column;gap:8px;"></div>
                </div>

                <!-- Advanced Settings Accordion / Grid -->
                <div style="display:grid;grid-template-columns:1fr 1fr;gap:14px;margin-bottom:14px;">
                    <div style="border:1px solid var(--border);border-radius:8px;padding:14px;">
                        <h4 style="font-size:13.5px;font-weight:700;margin:0 0 10px 0;">Admin Notification &amp; Confirmation</h4>
                        <div class="form-group">
                            <label class="form-label">Admin Recipient Email(s)</label>
                            <input type="text" id="form_recipient_email" class="form-input" value="{edit_email}" placeholder="sales@example.com, support@example.com" />
                        </div>
                        <div class="form-group">
                            <label class="form-label">Email Subject Template (Supports <code>{{{{field}}}}</code>, <code>{{{{form_title}}}}</code>)</label>
                            <input type="text" id="form_email_subject" class="form-input" value="{email_subj}" />
                        </div>
                        <div class="form-group">
                            <label class="form-label">Success Message</label>
                            <input type="text" id="form_success_msg" class="form-input" value="{edit_success}" />
                        </div>
                        <div class="form-group">
                            <label class="form-label">Optional Redirect URL after Submit</label>
                            <input type="text" id="form_redirect_url" class="form-input" value="{redirect_url}" placeholder="/thank-you/" />
                        </div>
                        <div class="form-group" style="margin:0;">
                            <label class="form-label">Outbound Webhook URL (JSON POST)</label>
                            <input type="text" id="form_webhook_url" class="form-input" value="{webhook_url}" placeholder="https://hooks.zapier.com/..." />
                        </div>
                    </div>

                    <div style="border:1px solid var(--border);border-radius:8px;padding:14px;">
                        <h4 style="font-size:13.5px;font-weight:700;margin:0 0 10px 0;">User Auto-Responder &amp; Anti-Spam</h4>
                        <label style="display:flex;align-items:center;gap:8px;font-size:13px;font-weight:600;margin-bottom:10px;cursor:pointer;">
                            <input type="checkbox" id="form_auto_resp_enabled" {auto_resp_checked} />
                            <span>Send Confirmation Email to Visitor</span>
                        </label>
                        <div style="display:grid;grid-template-columns:130px 1fr;gap:8px;">
                            <div class="form-group">
                                <label class="form-label">Email Field Key</label>
                                <input type="text" id="form_auto_resp_field" class="form-input" value="{auto_resp_field}" placeholder="email" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">Auto-Responder Subject</label>
                                <input type="text" id="form_auto_resp_subj" class="form-input" value="{auto_resp_subj}" />
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Auto-Responder Body</label>
                            <textarea id="form_auto_resp_body" rows="2" class="form-textarea" style="font-size:12px;">{auto_resp_body}</textarea>
                        </div>
                        <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:8px;">
                            <div class="form-group" style="margin:0;">
                                <label class="form-label">Min Submit (sec)</label>
                                <input type="number" id="form_min_sec" class="form-input" value="{min_sec}" min="0" max="60" />
                            </div>
                            <div class="form-group" style="margin:0;">
                                <label class="form-label">Rate Limit / min</label>
                                <input type="number" id="form_rate_limit" class="form-input" value="{rate_limit}" min="1" max="500" />
                            </div>
                            <div class="form-group" style="margin:0;">
                                <label class="form-label">Captcha</label>
                                <select id="form_captcha_provider" class="form-select">
                                    <option value="none" {cap_none}>None (Honeypot+Trap)</option>
                                    <option value="turnstile" {cap_turnstile}>Cloudflare Turnstile</option>
                                    <option value="hcaptcha" {cap_hcaptcha}>hCaptcha</option>
                                </select>
                            </div>
                        </div>
                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-top:8px;">
                            <input type="text" id="form_captcha_site" class="form-input" value="{cap_site}" placeholder="Captcha Site Key" style="font-size:12px;" />
                            <input type="password" id="form_captcha_secret" class="form-input" value="{cap_secret}" placeholder="Captcha Secret Key" style="font-size:12px;" />
                        </div>
                    </div>
                </div>

                <details style="margin-bottom:14px;">
                    <summary style="cursor:pointer;font-size:12.5px;font-weight:600;color:var(--ink-muted);">Raw Schema JSON &amp; Conditional Email Routing Rules</summary>
                    <div style="display:grid;grid-template-columns:1.3fr 1fr;gap:12px;margin-top:10px;">
                        <div>
                            <label class="form-label">Fields Schema JSON</label>
                            <textarea id="form_fields_json" rows="7" class="form-textarea" style="font-family:var(--font-mono);font-size:12px;" oninput="syncVisualFromTextarea()">{edit_fields_json}</textarea>
                        </div>
                        <div>
                            <label class="form-label">Conditional Email Routes JSON (<code>[{{"field","operator","value","recipient_email"}}]</code>)</label>
                            <textarea id="form_cond_routes_json" rows="7" class="form-textarea" style="font-family:var(--font-mono);font-size:12px;">{cond_routes_json}</textarea>
                        </div>
                    </div>
                </details>

                <div style="display:flex;justify-content:flex-end;gap:10px;">
                    <button type="button" class="btn btn-primary" onclick="saveCurrentForm()">Save Form to SQLite</button>
                </div>
            </div>
        </div>

        <!-- Submissions Vault Inbox -->
        <div class="day-card" style="padding:24px;">
            <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:14px;flex-wrap:wrap;gap:12px;">
                <div>
                    <h3 style="font-size:16px;font-weight:700;color:var(--ink);margin:0;">Fail-Safe SQLite Submissions Vault ({entry_count})</h3>
                    <p style="font-size:12.5px;color:var(--ink-muted);margin:2px 0 0 0;">Every submission is committed to SQLite before SMTP dispatch so zero leads are ever lost.</p>
                </div>
                <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;">
                    {status_tabs}
                    <form method="GET" action="/d-admin/forms" style="display:flex;gap:6px;">
                        <input type="hidden" name="status" value="{status_filter}" />
                        <input type="text" name="s" value="{search_query}" placeholder="Search entries..." class="form-input" style="width:180px;padding:5px 10px;font-size:12.5px;" />
                        <button type="submit" class="btn btn-secondary" style="padding:5px 10px;font-size:12px;">Search</button>
                    </form>
                </div>
            </div>
            <div class="day-table-wrapper">
                <table class="day-table">
                    <thead>
                        <tr>
                            <th style="width:36px;">★</th>
                            <th style="width:90px;">Entry</th>
                            <th style="width:85px;">Status</th>
                            <th>Submitted Payload &amp; Attachments</th>
                            <th style="width:95px;">Email</th>
                            <th style="width:155px;">Timestamp</th>
                            <th style="width:230px;">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {entries_rows}
                    </tbody>
                </table>
            </div>
        </div>

        <script>
            let visualFieldsState = [];

            function initVisualBuilder() {{
                try {{
                    visualFieldsState = JSON.parse(document.getElementById('form_fields_json').value || '[]');
                }} catch (_) {{
                    visualFieldsState = [];
                }}
                renderVisualFields();
            }}

            function syncVisualFromTextarea() {{
                try {{
                    visualFieldsState = JSON.parse(document.getElementById('form_fields_json').value || '[]');
                    renderVisualFields();
                }} catch (_) {{}}
            }}

            function syncTextareaFromVisual() {{
                document.getElementById('form_fields_json').value = JSON.stringify(visualFieldsState, null, 2);
            }}

            function renderVisualFields() {{
                const container = document.getElementById('visualFieldsList');
                if (!container) return;
                if (!visualFieldsState.length) {{
                    container.innerHTML = '<div style="padding:14px;text-align:center;color:var(--ink-muted);font-size:12.5px;">No fields yet. Click a field type above to add one.</div>';
                    return;
                }}
                container.innerHTML = visualFieldsState.map((f, idx) => {{
                    const optsVal = Array.isArray(f.options) ? f.options.join(', ') : '';
                    const showOpts = ['select', 'radio', 'checkbox_group'].includes(f.field_type);
                    const showCalc = f.field_type === 'calculated';
                    const showFile = f.field_type === 'file';
                    const condField = (f.condition && f.condition.field) || '';
                    const condVal = (f.condition && f.condition.value) || '';
                    return `<div style="display:grid;grid-template-columns:130px 1fr 110px 95px 75px auto;gap:8px;align-items:center;background:var(--paper-card);border:1px solid var(--border);border-radius:6px;padding:8px 10px;">
                        <input type="text" class="form-input" style="padding:4px 8px;font-size:12px;font-family:var(--font-mono);" value="${{(f.name || '').replace(/"/g, '&quot;')}}" placeholder="field_name" onchange="updateFieldProp(${{idx}}, 'name', this.value)" title="Field Key (name)" />
                        <input type="text" class="form-input" style="padding:4px 8px;font-size:12px;" value="${{(f.label || '').replace(/"/g, '&quot;')}}" placeholder="Field Label" onchange="updateFieldProp(${{idx}}, 'label', this.value)" title="Field Label" />
                        <select class="form-select" style="padding:4px 6px;font-size:12px;" onchange="updateFieldProp(${{idx}}, 'field_type', this.value)">
                            ${{['text','email','tel','url','number','textarea','select','radio','checkbox','checkbox_group','date','time','file','hidden','consent','calculated'].map(t => `<option value="${{t}}" ${{f.field_type===t?'selected':''}}>${{t}}</option>`).join('')}}
                        </select>
                        <select class="form-select" style="padding:4px 6px;font-size:12px;" onchange="updateFieldProp(${{idx}}, 'width', this.value)" title="Column Width">
                            <option value="full" ${{(f.width||'full')==='full'?'selected':''}}>Full (1/1)</option>
                            <option value="half" ${{f.width==='half'?'selected':''}}>Half (1/2)</option>
                            <option value="third" ${{f.width==='third'?'selected':''}}>Third (1/3)</option>
                            <option value="two_thirds" ${{f.width==='two_thirds'?'selected':''}}>2/3 Width</option>
                        </select>
                        <input type="number" class="form-input" style="padding:4px 6px;font-size:12px;" value="${{f.step || 1}}" min="1" max="10" onchange="updateFieldProp(${{idx}}, 'step', parseInt(this.value)||1)" title="Wizard Step #" />
                        <div style="display:flex;gap:4px;align-items:center;">
                            <label style="font-size:11.5px;display:flex;align-items:center;gap:3px;cursor:pointer;" title="Required"><input type="checkbox" ${{f.required?'checked':''}} onchange="updateFieldProp(${{idx}}, 'required', this.checked)" />Req</label>
                            <button type="button" class="btn btn-secondary" style="padding:2px 6px;font-size:11px;" onclick="moveVisualField(${{idx}}, -1)">↑</button>
                            <button type="button" class="btn btn-secondary" style="padding:2px 6px;font-size:11px;" onclick="moveVisualField(${{idx}}, 1)">↓</button>
                            <button type="button" class="btn btn-danger" style="padding:2px 6px;font-size:11px;" onclick="removeVisualField(${{idx}})">&times;</button>
                        </div>
                        ${{showOpts ? `<div style="grid-column:1 / -1;display:flex;gap:8px;align-items:center;"><span style="font-size:11.5px;color:var(--ink-muted);">Options (CSV):</span><input type="text" class="form-input" style="padding:3px 8px;font-size:12px;" value="${{optsVal.replace(/"/g, '&quot;')}}" onchange="updateFieldOptions(${{idx}}, this.value)" placeholder="Option A, Option B, Option C" /></div>` : ''}}
                        ${{showCalc ? `<div style="grid-column:1 / -1;display:flex;gap:8px;align-items:center;"><span style="font-size:11.5px;color:var(--ink-muted);">Formula:</span><input type="text" class="form-input" style="padding:3px 8px;font-size:12px;font-family:var(--font-mono);" value="${{(f.calculation_formula||'').replace(/"/g, '&quot;')}}" onchange="updateFieldProp(${{idx}}, 'calculation_formula', this.value)" placeholder="{{quantity}} * 49" /></div>` : ''}}
                        ${{showFile ? `<div style="grid-column:1 / -1;display:flex;gap:8px;align-items:center;"><span style="font-size:11.5px;color:var(--ink-muted);">Allowed Extensions:</span><input type="text" class="form-input" style="padding:3px 8px;font-size:12px;width:220px;" value="${{(f.accept_extensions||'.pdf,.png,.jpg').replace(/"/g, '&quot;')}}" onchange="updateFieldProp(${{idx}}, 'accept_extensions', this.value)" /><span style="font-size:11.5px;color:var(--ink-muted);">Max MB:</span><input type="number" class="form-input" style="padding:3px 8px;font-size:12px;width:80px;" value="${{f.max_file_size_mb||10}}" onchange="updateFieldProp(${{idx}}, 'max_file_size_mb', parseInt(this.value)||10)" /></div>` : ''}}
                        <div style="grid-column:1 / -1;display:flex;gap:8px;align-items:center;font-size:11.5px;color:var(--ink-muted);">
                            <span>Show if field:</span>
                            <input type="text" class="form-input" style="padding:2px 6px;font-size:11.5px;width:130px;" value="${{condField.replace(/"/g, '&quot;')}}" placeholder="(always visible)" onchange="updateFieldCondition(${{idx}}, this.value, document.getElementById('cond_val_${{idx}}').value)" />
                            <span>equals</span>
                            <input type="text" id="cond_val_${{idx}}" class="form-input" style="padding:2px 6px;font-size:11.5px;width:130px;" value="${{condVal.replace(/"/g, '&quot;')}}" placeholder="value" onchange="updateFieldCondition(${{idx}}, '${{condField.replace(/'/g, '')}}', this.value)" />
                        </div>
                    </div>`;
                }}).join('');
            }}

            function addVisualField(type) {{
                const count = visualFieldsState.length + 1;
                visualFieldsState.push({{
                    name: type + '_' + count,
                    label: type.charAt(0).toUpperCase() + type.slice(1) + ' ' + count,
                    field_type: type,
                    required: type === 'email' || type === 'consent',
                    placeholder: '',
                    options: ['select', 'radio', 'checkbox_group'].includes(type) ? ['Option 1', 'Option 2'] : [],
                    width: 'full',
                    step: 1,
                    default_value: '',
                    help_text: '',
                    accept_extensions: type === 'file' ? '.pdf,.doc,.docx,.png,.jpg,.webp' : '',
                    max_file_size_mb: 10,
                    calculation_formula: type === 'calculated' ? '{{qty}} * 10' : '',
                    condition: null
                }});
                syncTextareaFromVisual();
                renderVisualFields();
            }}

            function updateFieldProp(idx, key, val) {{
                if (!visualFieldsState[idx]) return;
                visualFieldsState[idx][key] = val;
                syncTextareaFromVisual();
                if (key === 'field_type') renderVisualFields();
            }}

            function updateFieldOptions(idx, csv) {{
                if (!visualFieldsState[idx]) return;
                visualFieldsState[idx].options = csv.split(',').map(s => s.trim()).filter(Boolean);
                syncTextareaFromVisual();
            }}

            function updateFieldCondition(idx, fieldName, expectedVal) {{
                if (!visualFieldsState[idx]) return;
                if (!fieldName.trim()) {{
                    visualFieldsState[idx].condition = null;
                }} else {{
                    visualFieldsState[idx].condition = {{
                        field: fieldName.trim(),
                        operator: 'equals',
                        value: expectedVal.trim()
                    }};
                }}
                syncTextareaFromVisual();
            }}

            function moveVisualField(idx, dir) {{
                const target = idx + dir;
                if (target < 0 || target >= visualFieldsState.length) return;
                const tmp = visualFieldsState[idx];
                visualFieldsState[idx] = visualFieldsState[target];
                visualFieldsState[target] = tmp;
                syncTextareaFromVisual();
                renderVisualFields();
            }}

            function removeVisualField(idx) {{
                visualFieldsState.splice(idx, 1);
                syncTextareaFromVisual();
                renderVisualFields();
            }}

            function createNewForm() {{
                document.getElementById('form_id').value = '';
                document.getElementById('formEditBadge').textContent = '(New)';
                document.getElementById('form_title').value = 'New Lead Capture Form';
                visualFieldsState = [
                    {{ name: 'name', label: 'Full Name', field_type: 'text', required: true, placeholder: 'Jane Doe', options: [], width: 'half', step: 1 }},
                    {{ name: 'email', label: 'Email Address', field_type: 'email', required: true, placeholder: 'jane@example.com', options: [], width: 'half', step: 1 }},
                    {{ name: 'message', label: 'How can we help?', field_type: 'textarea', required: true, placeholder: '', options: [], width: 'full', step: 1 }}
                ];
                syncTextareaFromVisual();
                renderVisualFields();
                document.getElementById('form_title').focus();
            }}

            async function saveCurrentForm() {{
                const idVal = parseInt(document.getElementById('form_id').value);
                const title = document.getElementById('form_title').value.trim() || 'Contact Form';
                let fields = [];
                try {{
                    fields = JSON.parse(document.getElementById('form_fields_json').value);
                }} catch (e) {{
                    showToast('Invalid Fields JSON: ' + e.message, true);
                    return;
                }}
                let conditional_routes = [];
                try {{
                    conditional_routes = JSON.parse(document.getElementById('form_cond_routes_json').value || '[]');
                }} catch (e) {{
                    showToast('Invalid Conditional Routes JSON: ' + e.message, true);
                    return;
                }}
                const step_titles = document.getElementById('form_step_titles').value
                    .split(',')
                    .map(s => s.trim())
                    .filter(Boolean);
                const redir = document.getElementById('form_redirect_url').value.trim();
                const wh = document.getElementById('form_webhook_url').value.trim();
                const settings = {{
                    submit_label: document.getElementById('form_submit_label').value.trim() || 'Send Message',
                    success_message: document.getElementById('form_success_msg').value.trim() || 'Thank you!',
                    redirect_url: redir || null,
                    recipient_email: document.getElementById('form_recipient_email').value.trim(),
                    email_subject_template: document.getElementById('form_email_subject').value.trim() || '[Daypress Form] New submission: {{{{form_title}}}}',
                    webhook_url: wh || null,
                    step_titles,
                    conditional_routes,
                    auto_responder_enabled: document.getElementById('form_auto_resp_enabled').checked,
                    auto_responder_email_field: document.getElementById('form_auto_resp_field').value.trim() || 'email',
                    auto_responder_subject: document.getElementById('form_auto_resp_subj').value.trim(),
                    auto_responder_body: document.getElementById('form_auto_resp_body').value,
                    min_submit_seconds: parseInt(document.getElementById('form_min_sec').value) || 0,
                    rate_limit_per_minute: parseInt(document.getElementById('form_rate_limit').value) || 10,
                    captcha_provider: document.getElementById('form_captcha_provider').value,
                    captcha_site_key: document.getElementById('form_captcha_site').value.trim(),
                    captcha_secret_key: document.getElementById('form_captcha_secret').value.trim()
                }};
                const res = await fetch('/d-admin/api/forms', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        action: 'save_form',
                        id: idVal > 0 ? idVal : undefined,
                        title,
                        fields,
                        settings
                    }})
                }});
                if (res.ok) {{
                    showToast('✓ FormStudio schema & settings saved to SQLite!');
                    setTimeout(() => location.reload(), 450);
                }} else {{
                    showToast('Failed to save form', true);
                }}
            }}

            async function deleteForm(id) {{
                if (!confirm('Delete Form #' + id + ' and all its submissions?')) return;
                const res = await fetch('/d-admin/api/forms', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'delete_form', id }})
                }});
                if (res.ok) {{
                    showToast('✓ Form deleted');
                    setTimeout(() => window.location.href = '/d-admin/forms', 350);
                }}
            }}

            async function toggleEntryStar(id) {{
                const res = await fetch('/d-admin/api/forms', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'toggle_entry_star', id }})
                }});
                if (res.ok) location.reload();
            }}

            async function updateEntryStatus(id, status) {{
                if (!status) return;
                const res = await fetch('/d-admin/api/forms', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'update_entry_status', id, status }})
                }});
                if (res.ok) {{
                    showToast('✓ Entry moved to ' + status);
                    setTimeout(() => location.reload(), 300);
                }}
            }}

            async function retryEntryEmail(id) {{
                showToast('Dispatching notification email for entry #' + id + '...');
                const res = await fetch('/d-admin/api/forms', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'retry_email', id }})
                }});
                const data = await res.json().catch(() => ({{}}));
                if (res.ok && data.email_status === 'sent') {{
                    showToast('✓ Notification email sent!');
                }} else {{
                    showToast('Email status: ' + (data.email_status || 'failed') + (data.email_error ? ' (' + data.email_error + ')' : ''), true);
                }}
                setTimeout(() => location.reload(), 500);
            }}

            async function deleteFormEntry(id) {{
                const res = await fetch('/d-admin/api/forms', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'delete_entry', id }})
                }});
                if (res.ok) {{
                    const row = document.getElementById('entry-row-' + id);
                    if (row) row.remove();
                    showToast('✓ Submission deleted');
                }}
            }}

            document.addEventListener('DOMContentLoaded', initVisualBuilder);
        </script>
        "##,
        forms_list_html = forms_list_html,
        edit_id = edit_id,
        edit_title = edit_title,
        edit_submit = escape_html(&parsed_settings.submit_label),
        step_titles_csv = step_titles_csv,
        edit_email = escape_html(&parsed_settings.recipient_email),
        email_subj = escape_html(&parsed_settings.email_subject_template),
        edit_success = escape_html(&parsed_settings.success_message),
        redirect_url = escape_html(&parsed_settings.redirect_url),
        webhook_url = escape_html(&parsed_settings.webhook_url),
        auto_resp_checked = auto_resp_checked,
        auto_resp_field = escape_html(&parsed_settings.auto_responder_email_field),
        auto_resp_subj = escape_html(&parsed_settings.auto_responder_subject),
        auto_resp_body = escape_html(&parsed_settings.auto_responder_body),
        min_sec = parsed_settings.min_submit_seconds,
        rate_limit = parsed_settings.rate_limit_per_minute,
        cap_none = if parsed_settings.captcha_provider == "none" {
            "selected"
        } else {
            ""
        },
        cap_turnstile = if parsed_settings.captcha_provider == "turnstile" {
            "selected"
        } else {
            ""
        },
        cap_hcaptcha = if parsed_settings.captcha_provider == "hcaptcha" {
            "selected"
        } else {
            ""
        },
        cap_site = escape_html(&parsed_settings.captcha_site_key),
        cap_secret = escape_html(&parsed_settings.captcha_secret_key),
        edit_fields_json = escape_html(&edit_fields_json),
        cond_routes_json = escape_html(&cond_routes_json),
        entry_count = entries.len(),
        status_tabs = status_tabs,
        status_filter = escape_html(status_filter),
        search_query = escape_html(search_query),
        entries_rows = entries_rows
    );

    render_admin_shell("forms", "FormStudio & Vault", &body, settings)
}

// 7. Snapshot & Hot Backup Manager (/d-admin/backups)
pub fn render_backups_view(
    settings: &DaypressSettingsResponse,
    snapshots: &[BackupSnapshotItem],
) -> String {
    let mut rows_html = String::new();
    for s in snapshots {
        let fname = escape_html(&s.filename);
        let size_kb = (s.size_bytes / 1024).max(1);
        rows_html.push_str(&format!(
            r#"<tr id="snap-row-{fname}">
                <td style="font-family:var(--font-mono);font-weight:600;color:var(--ink);">{fname}</td>
                <td>{size_kb} KB</td>
                <td style="color:var(--ink-muted);">{created_at}</td>
                <td style="display:flex;gap:8px;">
                    <a href="/d-admin/backups/download/{fname}" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;">⬇ Download</a>
                    <button type="button" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;color:var(--press);" onclick="restoreSnapshot('{fname}')">↺ Restore</button>
                    <button type="button" class="btn btn-danger" style="padding:4px 10px;font-size:12px;" onclick="deleteSnapshot('{fname}')">Delete</button>
                </td>
            </tr>"#,
            fname = fname,
            size_kb = size_kb,
            created_at = escape_html(&s.created_at)
        ));
    }

    if snapshots.is_empty() {
        rows_html = r#"<tr><td colspan="4" style="text-align:center;color:var(--ink-muted);padding:28px;">No SQLite snapshots created yet. Click <strong>Create Hot Snapshot</strong> above to run a zero-downtime <code>VACUUM INTO</code> backup.</td></tr>"#.to_string();
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <div style="display:flex;align-items:center;gap:10px;">
                    <h1 class="page-title">Snapshot &amp; Hot Backup Manager</h1>
                    <span class="badge badge-published">UpdraftPlus / All-in-One Migration Parity</span>
                </div>
                <p class="page-subtitle">Zero-downtime SQLite <code>VACUUM INTO</code> point-in-time snapshots, 1-click restores, and portable archives</p>
            </div>
            <div style="display:flex;gap:10px;">
                <a href="/d-admin/export-json" class="btn btn-secondary">⬇ JSON Bundle</a>
                <a href="/d-admin/export-wxr" class="btn btn-secondary">⬇ WXR XML</a>
            </div>
        </div>

        <div class="day-card" style="padding:24px;margin-bottom:24px;display:flex;justify-content:space-between;align-items:center;">
            <div>
                <h3 style="font-size:16px;font-weight:700;color:var(--ink);margin:0;">Create Atomic SQLite Snapshot</h3>
                <p style="font-size:13px;color:var(--ink-muted);margin-top:4px;">Executes an online <code>VACUUM INTO</code> snapshot without locking public readers.</p>
            </div>
            <div style="display:flex;gap:10px;align-items:center;">
                <input type="text" id="snapshotLabel" class="form-input" placeholder="Optional label (e.g. pre-migration)" style="width:240px;" />
                <button type="button" class="btn btn-primary" onclick="createSnapshot()">Create Hot Snapshot</button>
            </div>
        </div>

        <div class="day-table-wrapper">
            <table class="day-table">
                <thead>
                    <tr>
                        <th>Snapshot Filename</th>
                        <th style="width:120px;">Size</th>
                        <th style="width:200px;">Created (UTC)</th>
                        <th style="width:280px;">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
            </table>
        </div>

        <script>
            async function createSnapshot() {{
                const label = document.getElementById('snapshotLabel').value.trim();
                showToast('Creating hot SQLite snapshot...');
                const res = await fetch('/d-admin/api/backups', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'create', label: label || undefined }})
                }});
                if (res.ok) {{
                    showToast('✓ Hot SQLite snapshot created!');
                    setTimeout(() => location.reload(), 450);
                }} else {{
                    showToast('Failed to create snapshot', true);
                }}
            }}

            async function restoreSnapshot(filename) {{
                if (!confirm('Restore "' + filename + '" into the live SQLite database? Current state will be replaced by this point-in-time snapshot.')) return;
                const res = await fetch('/d-admin/api/backups', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'restore', filename }})
                }});
                if (res.ok) {{
                    showToast('✓ Restored snapshot ' + filename + '!');
                    setTimeout(() => location.reload(), 600);
                }} else {{
                    showToast('Failed to restore snapshot', true);
                }}
            }}

            async function deleteSnapshot(filename) {{
                if (!confirm('Delete snapshot "' + filename + '"?')) return;
                const res = await fetch('/d-admin/api/backups', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'delete', filename }})
                }});
                if (res.ok) {{
                    showToast('✓ Snapshot deleted');
                    setTimeout(() => location.reload(), 350);
                }} else {{
                    showToast('Failed to delete snapshot', true);
                }}
            }}
        </script>
        "##,
        rows_html = rows_html
    );

    render_admin_shell("backups", "Backups", &body, settings)
}
