use super::layout::render_wp_admin_shell;
use crate::models::{
    BackupSnapshotItem, FormEntryItem, FormFieldSchema, FormSettings, GlobalSeoSettings,
    InstalledPluginRecord, NeopressForm, NotFoundLogEntry, RedirectRule, WpSettingsResponse,
};
use crate::templates::utils::escape_html;

// 1. Installed Plugins (/n-admin/plugins)
pub fn render_wp_plugins_view(
    settings: &WpSettingsResponse,
    installed: &[InstalledPluginRecord],
) -> String {
    let total_count = installed.len();
    let active_count = installed
        .iter()
        .filter(|p| p.active || crate::plugins::is_core_always_on(&p.manifest.slug))
        .count();
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
        let is_always_on = crate::plugins::is_core_always_on(&record.manifest.slug);
        let is_active = record.active || is_always_on;

        let category_label = if record.manifest.is_builtin {
            "First-Party Core"
        } else {
            "Community Extension"
        };

        let badge_html = if is_always_on {
            r#"<span class="badge" style="background:#eff6ff; color:#1d4ed8; border:1px solid #bfdbfe; font-size:11px;">Core · Active</span>"#
        } else if is_active {
            r#"<span class="badge badge-published" style="font-size:11px;">Active</span>"#
        } else {
            r#"<span class="badge badge-draft" style="font-size:11px;">Inactive</span>"#
        };

        let origin_badge = if record.manifest.is_builtin {
            r#"<span style="font-size:10.5px; background:#eff6ff; color:#2563eb; border:1px solid #bfdbfe; padding:1px 6px; border-radius:4px; margin-left:6px; font-weight:600;">Built-In Core</span>"#
        } else {
            r#"<span style="font-size:10.5px; background:#f5f3ff; color:#7c3aed; border:1px solid #ddd6fe; padding:1px 6px; border-radius:4px; margin-left:6px; font-weight:600;">Community Extension</span>"#
        };

        let toggle_action = if is_always_on {
            r#"<span style="color:#64748b; font-size:12px; font-weight:500;">Core (Always Active)</span>"#.to_string()
        } else if is_active {
            format!(
                r#"<a href="javascript:void(0)" onclick="togglePlugin(this, '{slug}', false)" style="color:#d97706; font-weight:600;">Deactivate</a>"#,
                slug = slug
            )
        } else {
            format!(
                r#"<a href="javascript:void(0)" onclick="togglePlugin(this, '{slug}', true)" style="color:#059669; font-weight:600;">Activate</a>"#,
                slug = slug
            )
        };

        let (settings_url, settings_label) = match record.manifest.slug.as_str() {
            "neopress-seo" => ("/n-admin/seo", "SEO Studio & Redirects"),
            "neopress-forms" => ("/n-admin/forms", "Forms Studio & Inbox"),
            "neopress-backups" => ("/n-admin/backups", "Snapshots & Backups"),
            "neopress-wal-optimizer" => ("/n-admin/neopress-engine", "Engine Telemetry"),
            "webp-image-engine" => ("/n-admin/options-media", "Media Settings"),
            "wxr-streaming-xml-importer" => ("/n-admin/import", "Import Tool"),
            "whatsapp-chat" => (
                "/n-admin/plugin-settings?plugin=whatsapp-chat",
                "Configure Chat",
            ),
            "code-snippets" => (
                "/n-admin/plugin-settings?plugin=code-snippets",
                "Configure Snippets",
            ),
            _ => {
                if let Some(ref custom_url) = record.manifest.settings_url {
                    if !custom_url.is_empty() {
                        (custom_url.as_str(), "Settings")
                    } else {
                        ("/n-admin/plugins", "Settings")
                    }
                } else {
                    ("/n-admin/plugins", "Settings")
                }
            }
        };

        let custom_actions = if !record.manifest.is_builtin {
            format!(
                r#"<span>&bull;</span><a href="/n-admin/plugin-editor?plugin={slug}" style="color:#4f46e5; font-weight:500;">Edit Code</a>
                   <span>&bull;</span><a href="javascript:void(0)" onclick="uninstallCustomPlugin('{slug}')" class="trash-action">Uninstall</a>"#,
                slug = slug
            )
        } else {
            String::new()
        };

        let shortcode_hint = if !record.manifest.hooks.shortcode.is_empty() {
            format!(
                r#"<span style="font-family:var(--font-mono);font-size:11px;background:#f1f5f9;color:#0f172a;padding:1px 6px;border-radius:4px;margin-left:6px;">[{}]</span>"#,
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
        let row_bg = if !is_active {
            "background: #fafafa;"
        } else {
            ""
        };

        rows_html.push_str(&format!(
            r##"
            <tr id="plugin-row-{slug}" data-status="{row_status}" data-kind="{row_kind}" style="{row_bg}">
                <td style="width: 32px;"><input type="checkbox" name="checked_plugins[]" value="{slug}" onchange="updateBulkSelection()"></td>
                <td style="width: 320px;">
                    <div style="display:flex;align-items:center;flex-wrap:wrap;gap:2px;">
                        <strong style="color: #0f172a; font-size: 14px;">{name}</strong>
                        {origin_badge}
                    </div>
                    <div class="table-actions" style="margin-top:8px;">
                        {toggle_action}
                        <span>&bull;</span>
                        <a href="{settings_url}">{settings_label}</a>
                        {custom_actions}
                    </div>
                </td>
                <td>
                    <div style="color:#334155;line-height:1.5;">{desc}{shortcode_hint}</div>
                    <div style="font-size: 11.5px; color: #64748b; margin-top: 6px;">
                        Version {version} &bull; By {author} &bull; Category: <strong style="color:#475569;">{category_label}</strong>
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
        ));
    }

    if rows_html.is_empty() {
        rows_html = r#"<tr><td colspan="4" style="text-align:center; padding:36px; color:#64748b;">No plugins found matching this view.</td></tr>"#.to_string();
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Plugins &amp; Native Modules</h1>
                <p class="page-subtitle">Manage built-in first-party Rust subsystems and declarative community plugins</p>
            </div>
            <div style="display: flex; gap: 10px;">
                <a href="/n-admin/api/plugin-sdk-template" class="btn btn-secondary">⬇ Download Starter SDK (.json)</a>
                <a href="/n-admin/plugin-install" class="btn btn-primary">+ Add New / Install Extension</a>
            </div>
        </div>

        <!-- Filter Tabs & Real-time Search -->
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;">
            <div style="display: flex; gap: 8px;" id="pluginFilterTabs">
                <button type="button" class="btn btn-secondary plug-tab status-tab" data-filter="all" style="background: #0f172a; color: #fff;" onclick="filterPluginsByTab('all', this)">All ({total_count})</button>
                <button type="button" class="btn btn-secondary plug-tab status-tab" data-filter="active" onclick="filterPluginsByTab('active', this)">Active ({active_count})</button>
                <button type="button" class="btn btn-secondary plug-tab status-tab" data-filter="inactive" onclick="filterPluginsByTab('inactive', this)">Inactive ({inactive_count})</button>
                <button type="button" class="btn btn-secondary plug-tab status-tab" data-filter="builtin" onclick="filterPluginsByTab('builtin', this)">Built-In Core ({builtin_count})</button>
                <button type="button" class="btn btn-secondary plug-tab status-tab" data-filter="community" onclick="filterPluginsByTab('community', this)">Community ({custom_count})</button>
            </div>
            <div style="display: flex; gap: 10px; align-items: center;">
                <div style="display: flex; gap: 6px; align-items: center;">
                    <select id="bulkPluginAction" class="form-input" style="width: auto; padding: 6px 10px; font-size: 13px;">
                        <option value="">Bulk actions</option>
                        <option value="activate">Activate</option>
                        <option value="deactivate">Deactivate</option>
                    </select>
                    <button type="button" class="btn btn-secondary" style="padding: 6px 12px; font-size: 13px;" onclick="applyBulkPluginAction()">Apply</button>
                </div>
                <input type="text" id="pluginSearchInput" placeholder="Search installed plugins..." class="form-input" style="width: 220px; padding: 6px 12px; font-size: 13px;" oninput="filterPluginsList(this.value)" />
            </div>
        </div>

        <div class="neo-table-wrapper">
            <table class="neo-table" id="pluginsTable">
                <thead>
                    <tr>
                        <th style="width: 32px;"><input type="checkbox" id="selectAllPlugins" onchange="toggleSelectAllPlugins(this)"></th>
                        <th style="width: 320px;">Plugin / Module</th>
                        <th>Description &amp; Capabilities</th>
                        <th style="width: 120px;">Status</th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
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
                    b.style.background = '';
                    b.style.color = '';
                }});
                if (btn) {{
                    btn.style.background = '#0f172a';
                    btn.style.color = '#fff';
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
                try {{
                    const res = await fetch('/n-admin/api/plugins', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
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
                showToast('Updating ' + checked.length + ' plugin(s)...');
                for (const cb of checked) {{
                    const plugin = cb.value;
                    try {{
                        await fetch('/n-admin/api/plugins', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ plugin, active: activate }})
                        }});
                    }} catch (e) {{}}
                }}
                showToast('✓ Bulk update completed');
                setTimeout(() => window.location.reload(), 500);
            }}

            async function uninstallCustomPlugin(slug) {{
                if (!confirm('Uninstall custom plugin "' + slug + '" from SQLite?')) return;
                try {{
                    const res = await fetch('/n-admin/api/plugin-install', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
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

    render_wp_admin_shell("plugins", "Plugins", &body, settings)
}

// 2. Add New Plugin & Community SDK Installer (/n-admin/plugin-install)
pub fn render_wp_plugin_install_view(settings: &WpSettingsResponse) -> String {
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
                <a href="/n-admin/api/plugin-sdk-template" class="btn btn-secondary">⬇ Download Starter SDK (.json)</a>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('pluginManifestFile').click()">Upload Manifest (.json)</button>
                <input type="file" id="pluginManifestFile" accept=".json,application/json" style="display: none;" onchange="handleManifestFileUpload(this)">
            </div>
        </div>

        <!-- Featured Community Extensions -->
        <div style="margin-bottom: 32px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
                <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Featured 1-Click Community Extensions</h3>
                <span style="font-size: 12px; color: #64748b;">Ready to install directly into SQLite</span>
            </div>
            <div id="pluginCatalogGrid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px;">
                <div class="neo-card" style="padding: 22px; display: flex; flex-direction: column; justify-content: space-between;">
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                            <h3 style="font-size: 15px; font-weight: 700; color: #0f172a;">Top Promo Announcement Bar</h3>
                            <span style="font-size: 11px; background: #ecfdf5; color: #059669; padding: 2px 6px; border-radius: 4px; font-weight: 600;">Community SDK</span>
                        </div>
                        <p style="font-size: 13px; color: #64748b; margin: 8px 0 14px 0; line-height: 1.5;">
                            Sticky top banner with customizable colors, link button, and <code>[promo_cta]</code> shortcode for marketing campaigns.
                        </p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 14px; border-top: 1px solid var(--border-subtle);">
                        <span style="font-size: 12px; color: #94a3b8;">Hooks: body_top + shortcode</span>
                        <button class="btn btn-primary" style="font-size: 12px;" onclick="quickInstallPlugin('promo-announcement-bar')">Install Now</button>
                    </div>
                </div>

                <div class="neo-card" style="padding: 22px; display: flex; flex-direction: column; justify-content: space-between;">
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                            <h3 style="font-size: 15px; font-weight: 700; color: #0f172a;">Lightweight Cookie Consent</h3>
                            <span style="font-size: 11px; background: #f5f3ff; color: #7c3aed; padding: 2px 6px; border-radius: 4px; font-weight: 600;">Privacy First</span>
                        </div>
                        <p style="font-size: 13px; color: #64748b; margin: 8px 0 14px 0; line-height: 1.5;">
                            Zero-dependency GDPR &amp; ePrivacy compliant cookie notification bar with customizable disclosure notice and accept button.
                        </p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 14px; border-top: 1px solid var(--border-subtle);">
                        <span style="font-size: 12px; color: #94a3b8;">Hooks: footer</span>
                        <button class="btn btn-primary" style="font-size: 12px;" onclick="quickInstallPlugin('cookie-consent')">Install Now</button>
                    </div>
                </div>

                <div class="neo-card" style="padding: 22px; display: flex; flex-direction: column; justify-content: space-between;">
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                            <h3 style="font-size: 15px; font-weight: 700; color: #0f172a;">Scroll Reading Progress Bar</h3>
                            <span style="font-size: 11px; background: #eff6ff; color: #2563eb; padding: 2px 6px; border-radius: 4px; font-weight: 600;">Editorial</span>
                        </div>
                        <p style="font-size: 13px; color: #64748b; margin: 8px 0 14px 0; line-height: 1.5;">
                            Renders a smooth progress bar at the very top of articles that dynamically fills as the reader scrolls down.
                        </p>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 14px; border-top: 1px solid var(--border-subtle);">
                        <span style="font-size: 12px; color: #94a3b8;">Hooks: body_top</span>
                        <button class="btn btn-primary" style="font-size: 12px;" onclick="quickInstallPlugin('reading-progress-bar')">Install Now</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Community Plugin Manifest Installer -->
        <div class="neo-card" style="padding: 24px; margin-bottom: 24px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                <div>
                    <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Community Plugin Manifest Developer (<code>neopress-plugin.json</code>)</h3>
                    <p style="font-size: 13px; color: #64748b; margin-top: 4px;">
                        Paste or edit custom JSON schemas (<code>settings_schema</code>) and HTML/CSS/JS hook templates (<code>head_html</code>, <code>body_top_html</code>, <code>content_before_html</code>, <code>footer_html</code>, <code>shortcode</code>). Neopress will validate and register the plugin.
                    </p>
                </div>
            </div>
            <textarea id="manifestJsonInput" rows="10" class="neo-html-textarea" style="min-height: 200px; font-size: 12.5px; margin-bottom: 14px;">{sdk_sample}</textarea>
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <button type="button" class="btn btn-primary" onclick="installManifestFromTextarea()">Validate &amp; Install Plugin &rarr;</button>
                <a href="/n-admin/plugins" style="font-size: 13px; color: #2563eb; text-decoration: none; font-weight: 600;">&larr; Return to Installed Plugins</a>
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
                    "settings_url": "/n-admin/plugin-settings?plugin=promo-announcement-bar",
                    "is_builtin": false,
                    "settings_schema": [
                        {{ "key": "banner_text", "label": "Announcement Text", "field_type": "text", "default_value": "⚡ Fast 1-Click Migrations with Neopress CMS!", "description": "Banner message displayed across the top.", "options": [] }},
                        {{ "key": "bg_color", "label": "Background Color", "field_type": "color", "default_value": "#1e293b", "description": "Hex color code.", "options": [] }},
                        {{ "key": "text_color", "label": "Text Color", "field_type": "color", "default_value": "#f8fafc", "description": "Hex text color.", "options": [] }},
                        {{ "key": "cta_link", "label": "Call-to-Action Link URL", "field_type": "text", "default_value": "/contact/", "description": "Optional link URL for the banner button.", "options": [] }},
                        {{ "key": "cta_label", "label": "Button Label", "field_type": "text", "default_value": "Learn More →", "description": "Button text.", "options": [] }}
                    ],
                    "hooks": {{
                        "head_html": "<style>#np-promo-bar {{ position: relative; z-index: 9998; padding: 10px 16px; text-align: center; font-size: 13px; font-weight: 500; display: flex; align-items: center; justify-content: center; gap: 12px; }} #np-promo-bar a {{ text-decoration: underline; font-weight: 600; color: inherit; }}</style>",
                        "body_top_html": "<div id=\"np-promo-bar\" style=\"background: {{{{bg_color}}}}; color: {{{{text_color}}}};\"><span>{{{{banner_text}}}}</span><a href=\"{{{{cta_link}}}}\">{{{{cta_label}}}}</a></div>",
                        "shortcode": "promo_cta",
                        "shortcode_html": "<div style=\"padding: 16px 20px; background: {{{{bg_color}}}}; color: {{{{text_color}}}}; border-radius: 8px; text-align: center; margin: 16px 0;\"><p style=\"margin: 0 0 10px 0; font-size: 15px; font-weight: 600;\">{{{{banner_text}}}}</p><a href=\"{{{{cta_link}}}}\" style=\"display: inline-block; padding: 6px 16px; background: #ffffff; color: #0f172a; border-radius: 4px; font-weight: 600; text-decoration: none;\">{{{{cta_label}}}}</a></div>"
                    }}
                }},
                "cookie-consent": {{
                    "slug": "cookie-consent",
                    "name": "Lightweight Cookie Consent Banner",
                    "version": "1.1.0",
                    "author": "Privacy First",
                    "description": "Zero-dependency GDPR & ePrivacy compliant cookie notification bar with customizable notice and accept button.",
                    "settings_url": "/n-admin/plugin-settings?plugin=cookie-consent",
                    "is_builtin": false,
                    "settings_schema": [
                        {{ "key": "notice_text", "label": "Notice Text", "field_type": "text", "default_value": "We use essential cookies to ensure high-performance browsing.", "description": "Cookie disclosure text.", "options": [] }},
                        {{ "key": "accept_button_text", "label": "Accept Button Text", "field_type": "text", "default_value": "Accept & Close", "description": "Button label.", "options": [] }}
                    ],
                    "hooks": {{
                        "footer_html": "<div id=\"np-cookie-banner\" style=\"position: fixed; bottom: 20px; left: 20px; right: 20px; max-width: 520px; margin: 0 auto; background: #0f172a; color: #f8fafc; padding: 14px 20px; border-radius: 8px; box-shadow: 0 10px 25px rgba(0,0,0,0.3); z-index: 99999; display: flex; align-items: center; justify-content: space-between; gap: 14px; font-size: 12.5px;\"><span>{{{{notice_text}}}}</span><button onclick=\"this.parentElement.remove(); localStorage.setItem('np_cookie_ack', '1')\" style=\"padding: 6px 14px; background: #2563eb; color: #fff; border: none; border-radius: 4px; font-weight: 600; cursor: pointer;\">{{{{accept_button_text}}}}</button></div><script>if (localStorage.getItem('np_cookie_ack')) {{ const b = document.getElementById('np-cookie-banner'); if (b) b.remove(); }}</script>"
                    }}
                }},
                "reading-progress-bar": {{
                    "slug": "reading-progress-bar",
                    "name": "Scroll Reading Progress Bar",
                    "version": "1.0.0",
                    "author": "Editorial Studio",
                    "description": "Renders a smooth progress bar at the very top of articles that fills as the reader scrolls down.",
                    "settings_url": "/n-admin/plugin-settings?plugin=reading-progress-bar",
                    "is_builtin": false,
                    "settings_schema": [
                        {{ "key": "bar_color", "label": "Progress Bar Color", "field_type": "color", "default_value": "#2563eb", "description": "Color of the reading bar.", "options": [] }},
                        {{ "key": "bar_height", "label": "Height (px)", "field_type": "text", "default_value": "4", "description": "Thickness of the progress bar.", "options": [] }}
                    ],
                    "hooks": {{
                        "body_top_html": "<div id=\"np-reading-bar\" style=\"position: fixed; top: 0; left: 0; width: 0%; height: {{{{bar_height}}}}px; background: {{{{bar_color}}}}; z-index: 99999; transition: width 0.1s linear;\"></div><script>window.addEventListener('scroll', () => {{ const h = document.documentElement.scrollHeight - window.innerHeight; const pct = h > 0 ? (window.scrollY / h) * 100 : 0; const el = document.getElementById('np-reading-bar'); if (el) el.style.width = Math.min(100, Math.max(0, pct)) + '%'; }});</script>"
                    }}
                }}
            }};

            async function quickInstallPlugin(slug) {{
                const manifest = CURATED_EXTENSIONS[slug];
                if (!manifest) return;
                showToast('Installing ' + manifest.name + '...');
                try {{
                    const res = await fetch('/n-admin/api/plugin-install', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ manifest }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.success) {{
                        showToast('✓ ' + manifest.name + ' installed & activated!');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/plugin-settings?plugin=' + encodeURIComponent(manifest.slug);
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
                    const res = await fetch('/n-admin/api/plugin-install', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ manifest }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.success) {{
                        showToast('✓ Plugin "' + manifest.name + '" installed & activated!');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/plugin-settings?plugin=' + encodeURIComponent(manifest.slug);
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

    render_wp_admin_shell("plugin-install", "Add Plugins", &body, settings)
}

// 3. Plugin File Editor (/n-admin/plugin-editor)
pub fn render_wp_plugin_editor_view(settings: &WpSettingsResponse) -> String {
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
            <div class="neo-card" style="padding: 20px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                    <span id="activePluginFileName" style="font-family: var(--font-mono); font-size: 13px; font-weight: 600; color: #0f172a;">neopress-optimizer.rs (Active Plugin)</span>
                    <span style="font-size: 12px; color: #10b981; font-weight: 600;">✓ Safe Sandbox</span>
                </div>
                <textarea id="pluginFileContent" class="neo-html-textarea" style="min-height: 500px;">// Neopress WAL Optimizer Plugin
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

            <div class="neo-card" style="padding: 20px; height: fit-content;">
                <h4 style="font-size: 13.5px; font-weight: 700; color: #0f172a; margin-bottom: 12px;">Plugin Files</h4>
                <div style="display: flex; flex-direction: column; gap: 6px;">
                    <a href="javascript:void(0)" onclick="loadPluginFile('neopress-optimizer.rs')" style="padding: 6px 10px; border-radius: 4px; background: #eff6ff; color: #2563eb; font-weight: 600; font-size: 12.5px; text-decoration: none;">neopress-optimizer.rs</a>
                    <a href="javascript:void(0)" onclick="loadPluginFile('webp-engine.rs')" style="padding: 6px 10px; border-radius: 4px; color: #64748b; font-size: 12.5px; text-decoration: none;">webp-engine.rs</a>
                    <a href="javascript:void(0)" onclick="loadPluginFile('wxr-stream.rs')" style="padding: 6px 10px; border-radius: 4px; color: #64748b; font-size: 12.5px; text-decoration: none;">wxr-stream.rs</a>
                </div>
                <div style="margin-top: 20px; padding-top: 14px; border-top: 1px solid var(--border-subtle); font-size: 11.5px; color: #94a3b8;">
                    <strong>Security:</strong> Direct execution is sandboxed against root filesystem writes.
                </div>
            </div>
        </div>

        <script>
            let currentPluginFile = 'neopress-optimizer.rs';
            async function loadPluginFile(name) {
                currentPluginFile = name;
                document.getElementById('activePluginFileName').textContent = name + ' (Plugin File)';
                try {
                    const res = await fetch('/n-admin/api/theme-file?file=' + encodeURIComponent(name));
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
                    const res = await fetch('/n-admin/api/theme-file', {
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

    render_wp_admin_shell("plugin-editor", "Plugin File Editor", body, settings)
}

// 4. Dynamic Plugin Settings Page (/n-admin/plugin-settings?plugin=<slug>)
pub fn render_wp_plugin_settings_view(
    settings: &WpSettingsResponse,
    plugin: Option<&InstalledPluginRecord>,
) -> String {
    let Some(record) = plugin else {
        let not_found = r#"<div class="neo-card" style="padding: 32px; text-align: center;">
            <h2>Plugin Not Found</h2>
            <p style="color:#64748b;margin:8px 0 16px 0;">The requested plugin is not installed in SQLite.</p>
            <a href="/n-admin/plugins" class="btn btn-primary">&larr; Back to Plugins</a>
        </div>"#;
        return render_wp_admin_shell("plugin-settings", "Plugin Settings", not_found, settings);
    };

    let slug = escape_html(&record.manifest.slug);
    let name = escape_html(&record.manifest.name);
    let desc = escape_html(&record.manifest.description);
    let active_badge = if record.active {
        r#"<span class="badge badge-published">Active</span>"#
    } else {
        r#"<span class="badge badge-draft">Inactive</span>"#
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
                <label class="form-label">{label} <span style="font-family:var(--font-mono);font-size:11px;color:#94a3b8;font-weight:400;">({{{{{key}}}}})</span></label>
                {input_control}
            </div>"#
        ));
    }

    if fields_html.is_empty() {
        fields_html = r#"<p style="color:#64748b;">This plugin does not declare any custom settings fields.</p>"#.to_string();
    }

    let shortcode_info = if !record.manifest.hooks.shortcode.is_empty() {
        format!(
            r#"<div class="neo-card" style="padding:18px 24px;margin-bottom:20px;background:#f8fafc;">
                <strong>Shortcode Available:</strong> Embed <code style="background:#e2e8f0;padding:2px 8px;border-radius:4px;font-family:var(--font-mono);">[{}]</code> inside any post or page.
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
                <a href="/n-admin/plugins" class="btn btn-secondary">&larr; All Plugins</a>
                <button type="button" class="btn btn-primary" onclick="savePluginSettings('{slug}')">Save Settings to SQLite</button>
            </div>
        </div>

        {shortcode_info}

        <div class="neo-card" style="padding: 24px; max-width: 780px;">
            {fields_html}
            <div style="padding-top:16px;border-top:1px solid var(--border-subtle);display:flex;justify-content:flex-end;">
                <button type="button" class="btn btn-primary" onclick="savePluginSettings('{slug}')">Save Settings</button>
            </div>
        </div>

        <script>
            async function savePluginSettings(slug) {{
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
                    const res = await fetch('/n-admin/api/plugin-settings', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
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
        desc = desc,
        slug = slug,
        shortcode_info = shortcode_info,
        fields_html = fields_html
    );

    render_wp_admin_shell("plugin-settings", &name, &body, settings)
}

// 5. Full SEO Toolkit & Redirection Suite (/n-admin/seo)
pub fn render_wp_seo_view(
    settings: &WpSettingsResponse,
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
                <td style="font-family:var(--font-mono);font-weight:600;color:#0f172a;">{src}</td>
                <td style="font-family:var(--font-mono);color:#2563eb;">{dst}</td>
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
        redirect_rows = r#"<tr><td colspan="5" style="text-align:center;color:#64748b;padding:20px;">No 301/302 redirect rules configured yet. Changing a published post's slug automatically registers a 301 redirect here.</td></tr>"#.to_string();
    }

    let mut log_404_rows = String::new();
    for entry in logs_404 {
        let path_esc = escape_html(&entry.path);
        log_404_rows.push_str(&format!(
            r#"<tr>
                <td style="font-family:var(--font-mono);color:#dc2626;font-weight:600;">{path}</td>
                <td style="font-weight:700;">{hits}</td>
                <td style="font-size:12px;color:#64748b;">{last_seen}</td>
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
        log_404_rows = r#"<tr><td colspan="4" style="text-align:center;color:#64748b;padding:20px;">Zero 404 Not Found hits recorded.</td></tr>"#.to_string();
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
            <div class="neo-card" style="padding:24px;">
                <h3 style="font-size:16px;font-weight:700;color:#0f172a;margin-bottom:16px;">Global SEO &amp; Schema.org Knowledge Graph</h3>

                <div style="display:grid;grid-template-columns:120px 1fr;gap:14px;">
                    <div class="form-group">
                        <label class="form-label">Title Separator</label>
                        <input type="text" id="seo_sep" class="form-input" value="{sep}" />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Twitter / X Handle</label>
                        <input type="text" id="seo_twitter" class="form-input" value="{twitter}" placeholder="@neopress" />
                    </div>
                </div>

                <div class="form-group">
                    <label class="form-label">Organization / Publisher Name</label>
                    <input type="text" id="seo_org_name" class="form-input" value="{org_name}" placeholder="Neopress Inc." />
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
                    <label class="form-label">Custom <code>robots.txt</code> Rules <span style="font-weight:400;color:#64748b;">(Leave blank for dynamic default)</span></label>
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
                <div class="neo-card" style="padding:24px;">
                    <h3 style="font-size:16px;font-weight:700;color:#0f172a;margin-bottom:6px;">301 / 302 Redirect Manager</h3>
                    <p style="font-size:12.5px;color:#64748b;margin-bottom:14px;">Zero-latency SQLite redirect lookup executed before route resolution.</p>

                    <div style="display:grid;grid-template-columns:1fr 1fr 90px auto;gap:8px;margin-bottom:16px;">
                        <input type="text" id="newRedirSource" class="form-input" placeholder="/old-url-slug" />
                        <input type="text" id="newRedirTarget" class="form-input" placeholder="/new-url-slug/" />
                        <select id="newRedirCode" class="form-select">
                            <option value="301">301</option>
                            <option value="302">302</option>
                        </select>
                        <button type="button" class="btn btn-primary" onclick="addRedirectRule()">+ Add</button>
                    </div>

                    <div class="neo-table-wrapper">
                        <table class="neo-table">
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
                <div class="neo-card" style="padding:24px;">
                    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
                        <div>
                            <h3 style="font-size:16px;font-weight:700;color:#0f172a;margin:0;">404 Not Found Monitor</h3>
                            <p style="font-size:12.5px;color:#64748b;margin-top:2px;">Broken inbound links captured automatically in SQLite.</p>
                        </div>
                        <button type="button" class="btn btn-secondary" style="font-size:12px;" onclick="clear404Logs()">Clear Log</button>
                    </div>

                    <div class="neo-table-wrapper">
                        <table class="neo-table">
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
                    const res = await fetch('/n-admin/api/seo', {{
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
                const res = await fetch('/n-admin/api/redirects', {{
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
                const res = await fetch('/n-admin/api/redirects', {{
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
                const res = await fetch('/n-admin/api/redirects', {{
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

    render_wp_admin_shell("seo", "SEO & Redirects", &body, settings)
}

// 6. Neopress Forms Studio & Submissions Inbox (/n-admin/forms)
pub fn render_wp_forms_view(
    settings: &WpSettingsResponse,
    forms: &[(NeopressForm, i64)],
    entries: &[FormEntryItem],
    selected_form_id: Option<i64>,
) -> String {
    let active_form = selected_form_id
        .and_then(|id| forms.iter().find(|(f, _)| f.id == id).map(|(f, _)| f))
        .or_else(|| forms.first().map(|(f, _)| f));

    let mut forms_list_html = String::new();
    for (f, entry_count) in forms {
        let is_sel = active_form.map(|af| af.id) == Some(f.id);
        let row_bg = if is_sel { "background:#eff6ff;" } else { "" };
        let parsed_fields: Vec<FormFieldSchema> =
            serde_json::from_str(&f.schema_json).unwrap_or_default();
        forms_list_html.push_str(&format!(
            r#"<tr style="{row_bg}">
                <td>
                    <strong><a href="/n-admin/forms?form_id={id}" style="color:#0f172a;text-decoration:none;">{title}</a></strong>
                </td>
                <td>
                    <code style="background:#f1f5f9;padding:2px 6px;border-radius:4px;font-family:var(--font-mono);font-size:12px;">[neopress_form id="{id}"]</code>
                </td>
                <td>{field_count} fields &bull; {entry_count} entries</td>
                <td>
                    <a href="/n-admin/forms?form_id={id}" class="btn btn-secondary" style="padding:3px 8px;font-size:11.5px;">Edit</a>
                </td>
            </tr>"#,
            row_bg = row_bg,
            id = f.id,
            title = escape_html(&f.title),
            field_count = parsed_fields.len(),
            entry_count = entry_count
        ));
    }

    let (edit_id, edit_title, edit_submit, edit_success, edit_email, edit_fields_json) =
        if let Some(f) = active_form {
            let parsed_fields: Vec<FormFieldSchema> =
                serde_json::from_str(&f.schema_json).unwrap_or_default();
            let parsed_settings: FormSettings =
                serde_json::from_str(&f.settings_json).unwrap_or_default();
            (
                f.id,
                escape_html(&f.title),
                escape_html(&parsed_settings.submit_label),
                escape_html(&parsed_settings.success_message),
                escape_html(&parsed_settings.recipient_email),
                serde_json::to_string_pretty(&parsed_fields).unwrap_or_else(|_| "[]".to_string()),
            )
        } else {
            (
                0,
                "Contact Form".to_string(),
                "Send Message".to_string(),
                "Thank you! Your message has been received.".to_string(),
                "".to_string(),
                "[]".to_string(),
            )
        };

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
        let summary_html = if pairs.is_empty() {
            escape_html(&e.data_json)
        } else {
            pairs.join(" &bull; ")
        };

        entries_rows.push_str(&format!(
            r#"<tr id="entry-row-{id}">
                <td style="font-family:var(--font-mono);">#{id}</td>
                <td>Form #{form_id}</td>
                <td style="font-size:13px;line-height:1.5;">{summary_html}</td>
                <td style="font-size:12px;color:#64748b;">{created_at}</td>
                <td>
                    <button type="button" class="btn btn-danger" style="padding:3px 8px;font-size:11.5px;" onclick="deleteFormEntry({id})">Delete</button>
                </td>
            </tr>"#,
            id = e.id,
            form_id = e.form_id,
            summary_html = summary_html,
            created_at = escape_html(&e.created_at)
        ));
    }
    if entries.is_empty() {
        entries_rows = r#"<tr><td colspan="5" style="text-align:center;color:#64748b;padding:24px;">No form submissions received yet. Submit a form on any public page to view entries here.</td></tr>"#.to_string();
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <div style="display:flex;align-items:center;gap:10px;">
                    <h1 class="page-title">Neopress Forms Studio</h1>
                    <span class="badge badge-published">CF7 / WPForms / Ninja Forms Parity</span>
                </div>
                <p class="page-subtitle">Build high-conversion forms, embed via <code>[neopress_form id="X"]</code>, and inspect honeypot-protected SQLite submissions</p>
            </div>
            <div style="display:flex;gap:10px;">
                <a href="/n-admin/api/forms/export-csv" class="btn btn-secondary">⬇ Export Submissions (.csv)</a>
                <button type="button" class="btn btn-primary" onclick="createNewForm()">+ New Form</button>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:1fr 1.2fr;gap:24px;margin-bottom:24px;">
            <!-- Forms List -->
            <div class="neo-card" style="padding:24px;">
                <h3 style="font-size:16px;font-weight:700;color:#0f172a;margin-bottom:12px;">Configured Forms</h3>
                <div class="neo-table-wrapper">
                    <table class="neo-table">
                        <thead>
                            <tr>
                                <th>Form Title</th>
                                <th>Shortcode</th>
                                <th>Fields</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {forms_list_html}
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- Form Schema Editor -->
            <div class="neo-card" style="padding:24px;">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:14px;">
                    <h3 style="font-size:16px;font-weight:700;color:#0f172a;margin:0;">Edit Form <span id="formEditBadge">#{edit_id}</span></h3>
                    <code style="background:#eff6ff;color:#2563eb;padding:3px 8px;border-radius:4px;font-family:var(--font-mono);font-size:12px;">[neopress_form id="{edit_id}"]</code>
                </div>
                <input type="hidden" id="form_id" value="{edit_id}" />

                <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
                    <div class="form-group">
                        <label class="form-label">Form Title</label>
                        <input type="text" id="form_title" class="form-input" value="{edit_title}" />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Submit Button Label</label>
                        <input type="text" id="form_submit_label" class="form-input" value="{edit_submit}" />
                    </div>
                </div>

                <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
                    <div class="form-group">
                        <label class="form-label">Notification Recipient Email</label>
                        <input type="email" id="form_recipient_email" class="form-input" value="{edit_email}" placeholder="admin@example.com" />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Success Confirmation Message</label>
                        <input type="text" id="form_success_msg" class="form-input" value="{edit_success}" />
                    </div>
                </div>

                <div class="form-group">
                    <label class="form-label">Fields Schema JSON (<code>name</code>, <code>label</code>, <code>field_type</code>, <code>required</code>, <code>placeholder</code>)</label>
                    <textarea id="form_fields_json" rows="7" class="form-textarea" style="font-family:var(--font-mono);font-size:12px;">{edit_fields_json}</textarea>
                </div>

                <div style="display:flex;justify-content:flex-end;gap:10px;">
                    <button type="button" class="btn btn-primary" onclick="saveCurrentForm()">Save Form to SQLite</button>
                </div>
            </div>
        </div>

        <!-- Submissions Inbox -->
        <div class="neo-card" style="padding:24px;">
            <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:14px;">
                <h3 style="font-size:16px;font-weight:700;color:#0f172a;margin:0;">Form Submissions Inbox ({entry_count})</h3>
                <a href="/n-admin/api/forms/export-csv" class="btn btn-secondary" style="font-size:12px;">Export CSV</a>
            </div>
            <div class="neo-table-wrapper">
                <table class="neo-table">
                    <thead>
                        <tr>
                            <th style="width:80px;">ID</th>
                            <th style="width:100px;">Form</th>
                            <th>Submitted Fields</th>
                            <th style="width:170px;">Timestamp</th>
                            <th style="width:90px;">Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        {entries_rows}
                    </tbody>
                </table>
            </div>
        </div>

        <script>
            function createNewForm() {{
                document.getElementById('form_id').value = '';
                document.getElementById('formEditBadge').textContent = '(New)';
                document.getElementById('form_title').value = 'New Lead Capture Form';
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
                const settings = {{
                    submit_label: document.getElementById('form_submit_label').value.trim() || 'Send Message',
                    success_message: document.getElementById('form_success_msg').value.trim() || 'Thank you!',
                    recipient_email: document.getElementById('form_recipient_email').value.trim(),
                    webhook_url: ''
                }};
                const res = await fetch('/n-admin/api/forms', {{
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
                    showToast('✓ Form saved to SQLite!');
                    setTimeout(() => location.reload(), 450);
                }} else {{
                    showToast('Failed to save form', true);
                }}
            }}

            async function deleteFormEntry(id) {{
                const res = await fetch('/n-admin/api/forms', {{
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
        </script>
        "##,
        forms_list_html = forms_list_html,
        edit_id = edit_id,
        edit_title = edit_title,
        edit_submit = edit_submit,
        edit_email = edit_email,
        edit_success = edit_success,
        edit_fields_json = escape_html(&edit_fields_json),
        entry_count = entries.len(),
        entries_rows = entries_rows
    );

    render_wp_admin_shell("forms", "Forms Studio", &body, settings)
}

// 7. Snapshot & Hot Backup Manager (/n-admin/backups)
pub fn render_wp_backups_view(
    settings: &WpSettingsResponse,
    snapshots: &[BackupSnapshotItem],
) -> String {
    let mut rows_html = String::new();
    for s in snapshots {
        let fname = escape_html(&s.filename);
        let size_kb = (s.size_bytes / 1024).max(1);
        rows_html.push_str(&format!(
            r#"<tr id="snap-row-{fname}">
                <td style="font-family:var(--font-mono);font-weight:600;color:#0f172a;">{fname}</td>
                <td>{size_kb} KB</td>
                <td style="color:#64748b;">{created_at}</td>
                <td style="display:flex;gap:8px;">
                    <a href="/n-admin/backups/download/{fname}" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;">⬇ Download</a>
                    <button type="button" class="btn btn-secondary" style="padding:4px 10px;font-size:12px;color:#2563eb;" onclick="restoreSnapshot('{fname}')">↺ Restore</button>
                    <button type="button" class="btn btn-danger" style="padding:4px 10px;font-size:12px;" onclick="deleteSnapshot('{fname}')">Delete</button>
                </td>
            </tr>"#,
            fname = fname,
            size_kb = size_kb,
            created_at = escape_html(&s.created_at)
        ));
    }

    if snapshots.is_empty() {
        rows_html = r#"<tr><td colspan="4" style="text-align:center;color:#64748b;padding:28px;">No SQLite snapshots created yet. Click <strong>Create Hot Snapshot</strong> above to run a zero-downtime <code>VACUUM INTO</code> backup.</td></tr>"#.to_string();
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
                <a href="/n-admin/export-json" class="btn btn-secondary">⬇ JSON Bundle</a>
                <a href="/n-admin/export-wxr" class="btn btn-secondary">⬇ WXR XML</a>
            </div>
        </div>

        <div class="neo-card" style="padding:24px;margin-bottom:24px;display:flex;justify-content:space-between;align-items:center;">
            <div>
                <h3 style="font-size:16px;font-weight:700;color:#0f172a;margin:0;">Create Atomic SQLite Snapshot</h3>
                <p style="font-size:13px;color:#64748b;margin-top:4px;">Executes an online <code>VACUUM INTO</code> snapshot without locking public readers.</p>
            </div>
            <div style="display:flex;gap:10px;align-items:center;">
                <input type="text" id="snapshotLabel" class="form-input" placeholder="Optional label (e.g. pre-migration)" style="width:240px;" />
                <button type="button" class="btn btn-primary" onclick="createSnapshot()">⚡ Create Hot Snapshot</button>
            </div>
        </div>

        <div class="neo-table-wrapper">
            <table class="neo-table">
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
                showToast('⚡ Creating hot SQLite snapshot...');
                const res = await fetch('/n-admin/api/backups', {{
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
                const res = await fetch('/n-admin/api/backups', {{
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
                const res = await fetch('/n-admin/api/backups', {{
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

    render_wp_admin_shell("backups", "Backups", &body, settings)
}
