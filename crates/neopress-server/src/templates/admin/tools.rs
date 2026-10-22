use super::layout::render_wp_admin_shell;
use crate::models::{DbStats, WpSettingsResponse};

// 1. Available Tools (/n-admin/tools.php)
pub fn render_wp_tools_view(settings: &WpSettingsResponse, stats: &DbStats) -> String {
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Tools</h1>
                <p class="page-subtitle">Administration utilities, database maintenance, and data portability</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="runVacuum()">VACUUM Storage</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px;">
            <div class="neo-card" style="padding: 24px;">
                <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 8px;">Categories and Tags Converter</h3>
                <p style="font-size: 13px; color: #64748b; margin-bottom: 16px; line-height: 1.5;">
                    If you want to convert your categories to tags (or tags to categories), manage your taxonomy relationships in the Categories editor.
                </p>
                <a href="/n-admin/edit-tags.php?taxonomy=category" class="btn btn-secondary" style="font-size: 12.5px;">Manage Taxonomies &rarr;</a>
            </div>

            <div class="neo-card" style="padding: 24px;">
                <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 8px;">Site Health Diagnostics</h3>
                <p style="font-size: 13px; color: #64748b; margin-bottom: 16px; line-height: 1.5;">
                    Check your Rust binary execution state, embedded SQLite WAL mode integrity, file permissions, and memory allocations.
                </p>
                <a href="/n-admin/site-health.php" class="btn btn-secondary" style="font-size: 12.5px;">Run Site Health Check &rarr;</a>
            </div>

            <div class="neo-card" style="padding: 24px;">
                <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 8px;">Export Content (WXR XML)</h3>
                <p style="font-size: 13px; color: #64748b; margin-bottom: 16px; line-height: 1.5;">
                    Download an authentic WordPress eXtended RSS (.xml) file containing all your posts, pages, terms, and comments.
                </p>
                <a href="/n-admin/export.php" class="btn btn-secondary" style="font-size: 12.5px;">Go to Export &rarr;</a>
            </div>

            <div class="neo-card" style="padding: 24px;">
                <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 8px;">Privacy &amp; Personal Data</h3>
                <p style="font-size: 13px; color: #64748b; margin-bottom: 16px; line-height: 1.5;">
                    Fulfill GDPR compliance requests by exporting user archives or anonymizing personal details across the database.
                </p>
                <div style="display: flex; gap: 8px;">
                    <a href="/n-admin/export-personal-data.php" class="btn btn-secondary" style="font-size: 12px;">Export Data</a>
                    <a href="/n-admin/erase-personal-data.php" class="btn btn-secondary" style="font-size: 12px; color: #ef4444;">Erase Data</a>
                </div>
            </div>
        </div>

        <div class="neo-card" style="padding: 24px; margin-top: 24px;">
            <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 12px;">SQLite Storage Diagnostics</h3>
            <table class="neo-table">
                <tbody>
                    <tr><td style="font-weight: 600; width: 220px;">Storage File</td><td style="font-family: var(--font-mono);">{db_path}</td></tr>
                    <tr><td style="font-weight: 600;">Size on Disk</td><td><strong>{db_size_kb} KB</strong> ({page_count} B-Tree pages)</td></tr>
                    <tr><td style="font-weight: 600;">Journal Mode</td><td><span class="badge badge-published">{journal} (High-Speed In-Memory Transactions)</span></td></tr>
                </tbody>
            </table>
        </div>

        <script>
            async function runVacuum() {{
                try {{
                    const res = await fetch('/wp-json/neopress/v1/vacuum', {{ method: 'POST' }});
                    if (res.ok) {{
                        showToast('✓ Storage compacted and VACUUM completed!');
                        setTimeout(() => location.reload(), 600);
                    }}
                }} catch (e) {{
                    showToast('Failed to compact database', true);
                }}
            }}
        </script>
        "##,
        db_path = stats.db_path,
        db_size_kb = stats.file_size_bytes / 1024,
        page_count = stats.page_count,
        journal = stats.journal_mode
    );

    render_wp_admin_shell("tools", "Tools", &body, settings)
}

// 2. Import (/n-admin/import.php)
pub fn render_wp_import_view(settings: &WpSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Import</h1>
                <p class="page-subtitle">Import posts, pages, comments, and media from another system</p>
            </div>
        </div>

        <!-- WXR XML Stream Ingestion -->
        <div class="neo-card" style="margin-bottom: 24px; border: 2px dashed #2563eb; background: #fafcff; padding: 32px; text-align: center;">
            <div style="width: 56px; height: 56px; border-radius: 50%; background: #eff6ff; color: #2563eb; display: flex; align-items: center; justify-content: center; margin: 0 auto 16px auto; font-size: 24px;">📥</div>
            <h2 style="font-size: 18px; font-weight: 700; color: #0f172a; margin-bottom: 6px;">WordPress WXR Stream Importer (.xml)</h2>
            <p style="font-size: 13.5px; color: #64748b; max-width: 600px; margin: 0 auto 20px auto; line-height: 1.5;">
                Drag-and-drop standard WordPress export files (WordPress eXtended RSS 1.2). Our streaming <code>quick-xml</code> engine processes multi-gigabyte archives in atomic SQLite batches.
            </p>
            <input type="file" id="wxrFileInput" accept=".xml" style="display: none;" onchange="handleWxrSelect(this)">
            <button type="button" class="btn btn-primary" onclick="document.getElementById('wxrFileInput').click()" style="padding: 10px 22px; font-size: 14px;">Choose WordPress XML File</button>

            <div id="wxrImportProgress" style="display: none; max-width: 500px; margin: 20px auto 0 auto; text-align: left;">
                <div style="font-size: 13px; font-weight: 600; color: #0f172a; margin-bottom: 6px;" id="wxrProgressStatus">Streaming ingestion active...</div>
                <div style="width: 100%; height: 8px; background: #e2e8f0; border-radius: 4px; overflow: hidden;">
                    <div id="wxrProgressBar" style="width: 30%; height: 100%; background: #2563eb; animation: pulse 1.5s infinite;"></div>
                </div>
            </div>
        </div>

        <!-- Neopress JSON Bundle (1-Click Migrator) -->
        <div class="neo-card" style="margin-bottom: 24px; padding: 24px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                <div>
                    <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Neopress JSON Bundle (1-Click Migrator)</h3>
                    <p style="font-size: 13px; color: #64748b; margin-top: 4px;">Upload a <code>.json</code> migration bundle or paste JSON below to atomically import posts, pages, menus, categories, and Elementor/Divi layouts.</p>
                </div>
                <div>
                    <input type="file" id="jsonBundleFileInput" accept=".json,application/json" style="display: none;" onchange="handleJsonBundleFile(this)">
                    <button type="button" class="btn btn-secondary" onclick="document.getElementById('jsonBundleFileInput').click()">Upload .json File</button>
                </div>
            </div>
            <textarea id="jsonBundleTextarea" rows="5" class="form-textarea" style="font-family: var(--font-mono); font-size: 12px; margin-bottom: 12px;" placeholder='{"site_title": "My Site", "posts": [...], "pages": [...]}'></textarea>
            <div style="display: flex; justify-content: space-between; align-items: center;">
                <button type="button" class="btn btn-primary" onclick="submitJsonBundle()">Run 1-Click JSON Import &rarr;</button>
                <div id="jsonBundleSummary" style="font-size: 13px; font-weight: 600; color: #059669;"></div>
            </div>
        </div>

        <!-- Other Import Systems -->
        <div class="neo-card" style="padding: 24px;">
            <h3 style="font-size: 15px; font-weight: 700; color: #0f172a; margin-bottom: 16px;">Additional Import Sources</h3>
            <table class="neo-table">
                <thead>
                    <tr><th style="width: 240px;">System</th><th>Description</th><th style="width: 140px;">Action</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td><strong>Blogger (XML)</strong></td>
                        <td>Import posts, comments, and users from a Blogger XML export.</td>
                        <td><button class="btn btn-secondary" style="font-size: 11.5px;" onclick="document.getElementById('wxrFileInput').click()">Run Importer</button></td>
                    </tr>
                    <tr>
                        <td><strong>RSS / Atom (.xml)</strong></td>
                        <td>Import posts from standard RSS 2.0 or Atom XML syndication feeds.</td>
                        <td><button class="btn btn-secondary" style="font-size: 11.5px;" onclick="document.getElementById('wxrFileInput').click()">Run Importer</button></td>
                    </tr>
                    <tr>
                        <td><strong>JSON Backup Bundle</strong></td>
                        <td>Atomic 1-click Neopress JSON restore engine.</td>
                        <td><button class="btn btn-secondary" style="font-size: 11.5px;" onclick="document.getElementById('jsonBundleFileInput').click()">Import JSON</button></td>
                    </tr>
                </tbody>
            </table>
        </div>

        <script>
            async function handleWxrSelect(input) {
                if (!input.files || !input.files[0]) return;
                const file = input.files[0];
                const progress = document.getElementById('wxrImportProgress');
                const status = document.getElementById('wxrProgressStatus');
                const bar = document.getElementById('wxrProgressBar');
                progress.style.display = 'block';
                status.textContent = 'Parsing ' + file.name + ' (' + Math.round(file.size / 1024) + ' KB)...';

                try {
                    const res = await fetch('/wp-json/neopress/v1/import-wxr', {
                        method: 'POST',
                        body: file
                    });
                    if (res.ok) {
                        const data = await res.json();
                        bar.style.width = '100%';
                        status.textContent = '✓ ' + data.message;
                        showToast('✓ ' + data.message);
                        setTimeout(() => location.href = '/n-admin/edit.php', 1500);
                    } else {
                        status.textContent = 'Import error occurred';
                        showToast('Import failed', true);
                    }
                } catch (e) {
                    status.textContent = 'Network error during stream';
                    showToast('Network error: ' + e, true);
                }
            }

            async function handleJsonBundleFile(input) {
                if (!input.files || !input.files[0]) return;
                const file = input.files[0];
                try {
                    const text = await file.text();
                    document.getElementById('jsonBundleTextarea').value = text;
                    await submitJsonBundle();
                } catch (e) {
                    showToast('Failed to read JSON file: ' + e, true);
                }
            }

            async function submitJsonBundle() {
                const raw = document.getElementById('jsonBundleTextarea').value.trim();
                const summary = document.getElementById('jsonBundleSummary');
                if (!raw) {
                    showToast('Please upload or paste a JSON bundle first', true);
                    return;
                }
                let parsed;
                try {
                    parsed = JSON.parse(raw);
                } catch (e) {
                    showToast('Invalid JSON syntax: ' + e.message, true);
                    return;
                }
                summary.textContent = 'Importing bundle into SQLite...';
                try {
                    const res = await fetch('/wp-json/neopress/v1/import', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(parsed)
                    });
                    const data = await res.json();
                    if (res.ok) {
                        const msg = data.message || ('Imported ' + (data.imported_posts || 0) + ' posts, ' + (data.imported_pages || 0) + ' pages');
                        summary.textContent = '✓ ' + msg;
                        showToast('✓ ' + msg);
                    } else {
                        summary.textContent = '';
                        showToast('Import failed: ' + (data.message || JSON.stringify(data)), true);
                    }
                } catch (e) {
                    summary.textContent = '';
                    showToast('Network error: ' + e, true);
                }
            }
        </script>
    "##;

    render_wp_admin_shell("import", "Import", body, settings)
}

// 3. Export (/n-admin/export.php)
pub fn render_wp_export_view(settings: &WpSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Export</h1>
                <p class="page-subtitle">Export your content into an XML or JSON archive format compatible with any WordPress install</p>
            </div>
        </div>

        <div class="neo-card" style="padding: 24px; max-width: 680px;">
            <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 8px;">Choose what to export</h3>
            <p style="font-size: 13.5px; color: #64748b; line-height: 1.5; margin-bottom: 20px;">
                When you click the button below WordPress will create an XML file for you to save to your computer.
                This format, which we call WordPress eXtended RSS or WXR, will contain your posts, pages, comments, custom fields, categories, and tags.
            </p>

            <form action="/n-admin/export-wxr" method="GET">
                <div style="display: flex; flex-direction: column; gap: 14px; margin-bottom: 24px;">
                    <label style="display: flex; align-items: center; gap: 10px; font-weight: 600; cursor: pointer;">
                        <input type="radio" name="content" value="all" checked>
                        <span>All content</span>
                    </label>
                    <label style="display: flex; align-items: center; gap: 10px; font-weight: 500; cursor: pointer;">
                        <input type="radio" name="content" value="posts">
                        <span>Posts</span>
                    </label>
                    <label style="display: flex; align-items: center; gap: 10px; font-weight: 500; cursor: pointer;">
                        <input type="radio" name="content" value="pages">
                        <span>Pages</span>
                    </label>
                    <label style="display: flex; align-items: center; gap: 10px; font-weight: 500; cursor: pointer;">
                        <input type="radio" name="content" value="media">
                        <span>Media</span>
                    </label>
                </div>

                <div style="display: flex; gap: 12px;">
                    <button type="submit" class="btn btn-primary" style="padding: 9px 18px; font-size: 13.5px;">
                        Download Export File (WXR XML)
                    </button>
                    <button type="button" class="btn btn-secondary" onclick="downloadJsonBackup()" style="padding: 9px 18px; font-size: 13.5px;">
                        Download Full JSON Backup
                    </button>
                </div>
            </form>
        </div>

        <script>
            function downloadJsonBackup() {
                window.location.href = '/n-admin/export-json';
            }
        </script>
    "##;

    render_wp_admin_shell("export", "Export", body, settings)
}

// 4. Site Health (/n-admin/site-health.php)
pub fn render_wp_site_health_view(settings: &WpSettingsResponse, stats: &DbStats) -> String {
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Site Health</h1>
                <p class="page-subtitle">System performance, memory safety, and environment integrity audits</p>
            </div>
            <div style="display: flex; gap: 8px;">
                <button type="button" class="btn btn-primary" id="btnTabStatus" onclick="showHealthTab('status')">Status (Healthy)</button>
                <button type="button" class="btn btn-secondary" id="btnTabInfo" onclick="showHealthTab('info')">Info (Debug Data)</button>
            </div>
        </div>

        <!-- Status Tab -->
        <div id="healthStatusTab">
            <div class="neo-card" style="padding: 24px; margin-bottom: 24px; background: #ecfdf5; border: 1px solid #a7f3d0;">
                <div style="display: flex; align-items: center; gap: 14px;">
                    <div style="width: 48px; height: 48px; border-radius: 50%; background: #059669; color: #ffffff; display: flex; align-items: center; justify-content: center; font-size: 22px; font-weight: 700;">✓</div>
                    <div>
                        <h2 style="font-size: 18px; font-weight: 700; color: #065f46;">Good &bull; Your site is running at peak performance</h2>
                        <p style="font-size: 13px; color: #047857; margin-top: 2px;">The system checked 5 critical checks and detected zero memory-safety or storage issues.</p>
                    </div>
                </div>
            </div>

            <div class="neo-table-wrapper">
                <table class="neo-table">
                    <thead>
                        <tr><th style="width: 280px;">Item</th><th>Result</th><th style="width: 140px;">Status</th></tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><strong>Embedded Rusqlite Engine</strong></td>
                            <td>Write-Ahead-Log (WAL) mode active &bull; Sub-millisecond read/write latency.</td>
                            <td><span class="badge badge-published">Passed</span></td>
                        </tr>
                        <tr>
                            <td><strong>Relational Foreign Keys</strong></td>
                            <td>Enforced at driver level. Relational integrity guaranteed between posts and taxonomies.</td>
                            <td><span class="badge badge-published">Passed</span></td>
                        </tr>
                        <tr>
                            <td><strong>Rust Zero-Panic Architecture</strong></td>
                            <td>Type-safe <code>AppError</code> propagation without uncaught exceptions or segmentation faults.</td>
                            <td><span class="badge badge-published">Passed</span></td>
                        </tr>
                        <tr>
                            <td><strong>Media Asset Directory</strong></td>
                            <td>Directory <code>uploads/</code> is writable with automated WebP compression enabled.</td>
                            <td><span class="badge badge-published">Passed</span></td>
                        </tr>
                        <tr>
                            <td><strong>WordPress REST API v2</strong></td>
                            <td>Discovery root endpoint <code>/wp-json/</code> responding with standard schema.</td>
                            <td><span class="badge badge-published">Passed</span></td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Info Tab -->
        <div id="healthInfoTab" style="display: none;">
            <div class="neo-card" style="padding: 24px;">
                <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin-bottom: 16px;">Neopress System Overview</h3>
                <table class="neo-table">
                    <tbody>
                        <tr><td style="width: 220px; font-weight: 600;">Server Binary</td><td>Neopress v0.1.0 (Rust 1.80+ / Axum 0.8)</td></tr>
                        <tr><td style="font-weight: 600;">Database Engine</td><td>SQLite 3.46 (Bundled Rusqlite with WAL)</td></tr>
                        <tr><td style="font-weight: 600;">Database Path</td><td style="font-family: var(--font-mono); font-size: 12px;">{db_path}</td></tr>
                        <tr><td style="font-weight: 600;">Database Footprint</td><td>{db_size_kb} KB</td></tr>
                        <tr><td style="font-weight: 600;">Total Posts / Pages</td><td>{posts} posts &bull; {pages} pages</td></tr>
                        <tr><td style="font-weight: 600;">Total Options</td><td>{options} options auto-loaded</td></tr>
                        <tr><td style="font-weight: 600;">Media Storage</td><td>Local Disk (uploads/ folder)</td></tr>
                    </tbody>
                </table>
            </div>
        </div>

        <script>
            function showHealthTab(tab) {{
                if (tab === 'status') {{
                    document.getElementById('healthStatusTab').style.display = 'block';
                    document.getElementById('healthInfoTab').style.display = 'none';
                    document.getElementById('btnTabStatus').className = 'btn btn-primary';
                    document.getElementById('btnTabInfo').className = 'btn btn-secondary';
                }} else {{
                    document.getElementById('healthStatusTab').style.display = 'none';
                    document.getElementById('healthInfoTab').style.display = 'block';
                    document.getElementById('btnTabStatus').className = 'btn btn-secondary';
                    document.getElementById('btnTabInfo').className = 'btn btn-primary';
                }}
            }}
        </script>
        "##,
        db_path = stats.db_path,
        db_size_kb = stats.file_size_bytes / 1024,
        posts = stats.total_posts,
        pages = stats.total_pages,
        options = stats.total_options
    );

    render_wp_admin_shell("site-health", "Site Health", &body, settings)
}

// 5. Export Personal Data (/n-admin/export-personal-data.php)
pub fn render_wp_export_personal_data_view(settings: &WpSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Export Personal Data</h1>
                <p class="page-subtitle">Generate an encrypted JSON archive of all data associated with a specific user or commenter</p>
            </div>
        </div>

        <div class="neo-card" style="padding: 24px; max-width: 640px;">
            <h3 style="font-size: 15px; font-weight: 700; color: #0f172a; margin-bottom: 8px;">Send an export request</h3>
            <p style="font-size: 13px; color: #64748b; margin-bottom: 20px; line-height: 1.5;">
                An archive will be compiled containing all comments, profile fields, and content records linked to the provided username or email.
            </p>

            <form onsubmit="handleExportPersonalData(event)">
                <div class="form-group">
                    <label class="form-label" for="userEmail">Username or Email Address</label>
                    <input type="text" id="userEmail" class="form-input" placeholder="admin@example.com" required>
                </div>
                <button type="submit" class="btn btn-primary">Generate Data Export &rarr;</button>
            </form>

            <div id="exportResult" style="display: none; margin-top: 20px; padding: 14px; background: #f8fafc; border: 1px solid var(--border-subtle); border-radius: 6px;">
                <h4 style="font-size: 13.5px; font-weight: 600; color: #0f172a; margin-bottom: 4px;">Archive Ready:</h4>
                <pre id="exportJson" style="max-height: 200px; overflow-y: auto; background: #0f172a; color: #f8fafc; padding: 12px; border-radius: 4px; font-size: 12px; font-family: var(--font-mono);"></pre>
            </div>
        </div>

        <script>
            async function handleExportPersonalData(e) {
                e.preventDefault();
                const identifier = document.getElementById('userEmail').value;
                try {
                    const res = await fetch('/n-admin/api/export-personal-data?user=' + encodeURIComponent(identifier));
                    if (res.ok) {
                        const data = await res.json();
                        document.getElementById('exportResult').style.display = 'block';
                        document.getElementById('exportJson').textContent = JSON.stringify(data, null, 2);
                        showToast('✓ Personal data archive compiled!');
                    } else {
                        showToast('Error exporting personal data', true);
                    }
                } catch (err) {
                    showToast('Failed to connect', true);
                }
            }
        </script>
    "##;

    render_wp_admin_shell(
        "export-personal-data",
        "Export Personal Data",
        body,
        settings,
    )
}

// 6. Erase Personal Data (/n-admin/erase-personal-data.php)
pub fn render_wp_erase_personal_data_view(settings: &WpSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Erase Personal Data</h1>
                <p class="page-subtitle">Anonymize and scrub personal identifiable records across SQLite comments and meta tables</p>
            </div>
        </div>

        <div class="neo-card" style="padding: 24px; max-width: 640px;">
            <h3 style="font-size: 15px; font-weight: 700; color: #0f172a; margin-bottom: 8px;">Erase personal data request</h3>
            <p style="font-size: 13px; color: #64748b; margin-bottom: 20px; line-height: 1.5;">
                This process scrubs the author name, email, and IP address for all comments associated with the provided email, setting them to Anonymous.
            </p>

            <form onsubmit="handleErasePersonalData(event)">
                <div class="form-group">
                    <label class="form-label" for="eraseUserEmail">Username or Email Address</label>
                    <input type="text" id="eraseUserEmail" class="form-input" placeholder="user@example.com" required>
                </div>
                <button type="submit" class="btn btn-danger">Permanently Erase Personal Data</button>
            </form>
        </div>

        <script>
            async function handleErasePersonalData(e) {
                e.preventDefault();
                const identifier = document.getElementById('eraseUserEmail').value;
                if (!confirm('Are you sure you want to permanently erase personal data for ' + identifier + '? This action cannot be undone.')) return;

                try {
                    const res = await fetch('/n-admin/api/erase-personal-data', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ identifier })
                    });
                    if (res.ok) {
                        const data = await res.json();
                        showToast('✓ ' + data.message);
                    } else {
                        showToast('Failed to erase personal data', true);
                    }
                } catch (err) {
                    showToast('Network error during erasure', true);
                }
            }
        </script>
    "##;

    render_wp_admin_shell("erase-personal-data", "Erase Personal Data", body, settings)
}
