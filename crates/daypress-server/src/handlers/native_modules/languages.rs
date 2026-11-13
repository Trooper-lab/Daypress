#![allow(unused_imports)]
use super::*;
use crate::auth::OptionalAuthUser;
use crate::db::Database;
use crate::models;
use crate::templates;
use axum::{
    extract::{Form, Path as AxPath, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use serde::Deserialize;

// =========================================================================
// MULTILINGUAL (i18n) & LANGUAGES ADMIN PORTAL (/d-admin/languages)
// =========================================================================

pub async fn handle_admin_languages(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let languages = db.list_languages().unwrap_or_default();
    let prefix_default = db.prefix_default_language_enabled();

    let mut lang_rows = String::new();
    for lang in &languages {
        let code_esc = templates::utils::escape_html(&lang.code);
        let name_esc = templates::utils::escape_html(&lang.name);
        let native_esc = templates::utils::escape_html(&lang.native_name);
        let locale_esc = templates::utils::escape_html(&lang.locale);
        let flag_esc = templates::utils::escape_html(&lang.flag_emoji);
        let default_badge = if lang.is_default {
            r#"<span class="badge badge-published">Default</span>"#.to_string()
        } else if lang.is_rtl {
            r#"<span class="badge badge-draft">Secondary</span> <span class="badge" style="background:var(--ochre-wash);color:var(--ink);border:1px solid var(--ochre);margin-left:4px;">RTL</span>"#.to_string()
        } else {
            r#"<span class="badge badge-draft">Secondary</span>"#.to_string()
        };
        let url_pattern = if lang.is_default && !prefix_default {
            "/slug/".to_string()
        } else {
            format!("/{}/slug/", code_esc)
        };
        let actions_html = if lang.is_default {
            r#"<span style="font-size:11.5px;color:var(--ink-muted);">Primary Site Locale</span>"#
                .to_string()
        } else {
            format!(
                r#"<button type="button" class="btn btn-secondary" style="font-size:11.5px;padding:4px 10px;" data-code="{code_esc}" data-name="{name_esc}" data-native="{native_esc}" data-locale="{locale_esc}" data-flag="{flag_esc}" onclick="setAsDefaultLangFromBtn(this)">Make Default</button>
                <button type="button" class="btn btn-secondary" style="font-size:11.5px;padding:4px 10px;color:var(--oxide);" data-code="{code_esc}" onclick="removeLang(this.dataset.code)">Remove</button>"#
            )
        };
        lang_rows.push_str(&format!(
            r#"<tr>
                <td style="font-size:18px;width:44px;text-align:center;">{flag_esc}</td>
                <td><strong>{native_esc}</strong> <span style="color:var(--ink-muted);font-size:12px;">({name_esc})</span></td>
                <td><code>{code_esc}</code></td>
                <td><code>{locale_esc}</code></td>
                <td><code>{url_pattern}</code></td>
                <td>{default_badge}</td>
                <td style="text-align:right;display:flex;gap:6px;justify-content:flex-end;">{actions_html}</td>
            </tr>"#
        ));
    }

    let default_lang_code = languages
        .iter()
        .find(|l| l.is_default)
        .map(|l| l.code.clone())
        .unwrap_or_else(|| "en".to_string());
    let mut matrix_headers = String::new();
    for lang in &languages {
        matrix_headers.push_str(&format!(
            r#"<th style="text-align:center;">{} {}</th>"#,
            templates::utils::escape_html(&lang.flag_emoji),
            templates::utils::escape_html(&lang.code.to_uppercase())
        ));
    }
    let mut all_items = db
        .get_posts("page", Some("any"), None, 25)
        .unwrap_or_default();
    let mut recent_posts = db
        .get_posts("post", Some("any"), None, 25)
        .unwrap_or_default();
    all_items.append(&mut recent_posts);
    let primary_items: Vec<_> = all_items
        .iter()
        .filter(|p| {
            p.status != "trash"
                && (p.lang_code.is_empty() || p.lang_code.eq_ignore_ascii_case(&default_lang_code))
        })
        .take(20)
        .collect();
    let mut matrix_rows = String::new();
    for item in primary_items {
        let mut cells = String::new();
        for lang in &languages {
            if lang.code.eq_ignore_ascii_case(&default_lang_code) {
                cells.push_str(&format!(
                    r#"<td style="text-align:center;"><a href="/d-admin/post?post={}&action=edit" class="badge badge-published" style="text-decoration:none;">#{} Source</a></td>"#,
                    item.id, item.id
                ));
            } else if let Some(&target_id) = item.translations.get(&lang.code) {
                cells.push_str(&format!(
                    r#"<td style="text-align:center;"><a href="/d-admin/post?post={}&action=edit" class="badge badge-published" style="text-decoration:none;">✓ #{}</a></td>"#,
                    target_id, target_id
                ));
            } else {
                let lcode = templates::utils::escape_html(&lang.code);
                cells.push_str(&format!(
                    r#"<td style="text-align:center;"><button type="button" class="btn btn-secondary" style="font-size:11px;padding:2px 8px;" data-post-id="{}" data-lang="{}" onclick="quickCreateTranslation(Number(this.dataset.postId), this.dataset.lang)">+ Translate</button></td>"#,
                    item.id, lcode
                ));
            }
        }
        let title_esc = templates::utils::escape_html(&item.title.rendered);
        let ptype_esc = templates::utils::escape_html(&item.post_type.to_uppercase());
        matrix_rows.push_str(&format!(
            r#"<tr>
                <td><strong>{title_esc}</strong> <span style="font-size:10.5px;color:var(--ink-muted);margin-left:6px;">{ptype_esc}</span></td>
                {cells}
            </tr>"#
        ));
    }
    if matrix_rows.is_empty() {
        matrix_rows = format!(
            r#"<tr><td colspan="{}" style="text-align:center;color:var(--ink-muted);padding:18px;">No primary-language posts or pages yet.</td></tr>"#,
            languages.len() + 1
        );
    }

    let unprefixed_checked = if !prefix_default { "checked" } else { "" };
    let prefixed_checked = if prefix_default { "checked" } else { "" };

    let body = format!(
        r#"
        <div class="page-header">
            <div>
                <h1 class="page-title">Languages &amp; Multilingual (i18n)</h1>
                <p class="page-subtitle">Configure Polylang/WPML-compatible localized URL routing, hreflang SEO tags, and translation groups</p>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:2fr 1fr;gap:24px;align-items:start;">
            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Configured Site Languages</h2>
                    </div>
                    <div class="day-card-body" style="padding:0;">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Flag</th>
                                    <th>Language</th>
                                    <th>ISO Code</th>
                                    <th>Locale</th>
                                    <th>URL Prefix</th>
                                    <th>Role</th>
                                    <th style="text-align:right;">Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {lang_rows}
                            </tbody>
                        </table>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">URL Routing &amp; hreflang SEO Strategy</h2>
                    </div>
                    <div class="day-card-body">
                        <div style="display:flex;flex-direction:column;gap:10px;margin-bottom:14px;">
                            <label style="display:flex;align-items:flex-start;gap:10px;cursor:pointer;">
                                <input type="radio" name="i18n_prefix_mode" value="false" {unprefixed_checked} onchange="savePrefixMode(false)" style="margin-top:3px;" />
                                <div>
                                    <div style="font-weight:600;font-size:13.5px;">Unprefixed Default Locale (Recommended — Polylang / WPML Default)</div>
                                    <div style="font-size:12px;color:var(--ink-muted);">Default language serves at <code>/slug/</code>; secondary languages serve at <code>/{{lang}}/{{slug}}/</code> (e.g. <code>/de/willkommen/</code>).</div>
                                </div>
                            </label>
                            <label style="display:flex;align-items:flex-start;gap:10px;cursor:pointer;">
                                <input type="radio" name="i18n_prefix_mode" value="true" {prefixed_checked} onchange="savePrefixMode(true)" style="margin-top:3px;" />
                                <div>
                                    <div style="font-weight:600;font-size:13.5px;">Prefix All Locales Including Default</div>
                                    <div style="font-size:12px;color:var(--ink-muted);">Every language uses its ISO prefix: <code>/en/slug/</code>, <code>/de/slug/</code>, <code>/nl/slug/</code>.</div>
                                </div>
                            </label>
                        </div>
                        <div style="font-size:12px;color:var(--ink-muted);background:var(--paper-sunk);padding:10px 12px;border-radius:6px;">
                            <strong>Automatic SEO &amp; Sitemap:</strong> Linked translations automatically emit <code>&lt;link rel="alternate" hreflang="..."&gt;</code> (plus <code>x-default</code>) in SSR <code>&lt;head&gt;</code> and <code>&lt;xhtml:link rel="alternate"&gt;</code> in <code>/sitemap.xml</code>. Use <code>[daypress_language_switcher]</code> or the DayStudio <strong>Language Switcher</strong> widget anywhere.
                        </div>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Translation Coverage Matrix</h2>
                    </div>
                    <div class="day-card-body" style="padding:0;overflow-x:auto;">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Content Item</th>
                                    {matrix_headers}
                                </tr>
                            </thead>
                            <tbody>
                                {matrix_rows}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Add Language</h2>
                    </div>
                    <div class="day-card-body">
                        <div class="form-group">
                            <label class="form-label">Quick Preset</label>
                            <select id="langPresetSelect" class="form-select" onchange="applyLangPreset(this.value)">
                                <option value="">Choose a language preset...</option>
                                <option value="de|German|Deutsch|de_DE|🇩🇪">🇩🇪 Deutsch (de_DE)</option>
                                <option value="nl|Dutch|Nederlands|nl_NL|🇳🇱">🇳🇱 Nederlands (nl_NL)</option>
                                <option value="fr|French|Français|fr_FR|🇫🇷">🇫🇷 Français (fr_FR)</option>
                                <option value="es|Spanish|Español|es_ES|🇪🇸">🇪🇸 Español (es_ES)</option>
                                <option value="it|Italian|Italiano|it_IT|🇮🇹">🇮🇹 Italiano (it_IT)</option>
                                <option value="pt|Portuguese|Português|pt_PT|🇵🇹">🇵🇹 Português (pt_PT)</option>
                                <option value="pl|Polish|Polski|pl_PL|🇵🇱">🇵🇱 Polski (pl_PL)</option>
                                <option value="sv|Swedish|Svenska|sv_SE|🇸🇪">🇸🇪 Svenska (sv_SE)</option>
                                <option value="ja|Japanese|日本語|ja_JP|🇯🇵">🇯🇵 日本語 (ja_JP)</option>
                                <option value="ar|Arabic|العربية|ar|🇸🇦">🇸🇦 العربية (ar)</option>
                            </select>
                        </div>
                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                            <div class="form-group">
                                <label class="form-label">ISO Code</label>
                                <input type="text" id="newLangCode" class="form-input" placeholder="de" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">Flag Emoji</label>
                                <input type="text" id="newLangFlag" class="form-input" placeholder="🇩🇪" />
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Native Name</label>
                            <input type="text" id="newLangNative" class="form-input" placeholder="Deutsch" />
                        </div>
                        <div class="form-group">
                            <label class="form-label">English Name &amp; Locale</label>
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
                                <input type="text" id="newLangName" class="form-input" placeholder="German" />
                                <input type="text" id="newLangLocale" class="form-input" placeholder="de_DE" />
                            </div>
                        </div>
                        <div class="form-group" style="margin-bottom:12px;">
                            <label style="display:flex;align-items:center;gap:8px;font-size:12.5px;cursor:pointer;">
                                <input type="checkbox" id="newLangRtl" />
                                <span>Right-to-Left (RTL) Script (e.g. Arabic, Hebrew)</span>
                            </label>
                        </div>
                        <button type="button" class="btn btn-primary" style="width:100%;" onclick="addSiteLanguage()">+ Add Language</button>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Link Existing Translations</h2>
                    </div>
                    <div class="day-card-body">
                        <div class="form-group">
                            <label class="form-label">Source Post ID + Lang</label>
                            <div style="display:grid;grid-template-columns:1fr 90px;gap:8px;">
                                <input type="number" id="linkSrcId" class="form-input" placeholder="Post ID (e.g. 1)" />
                                <input type="text" id="linkSrcLang" class="form-input" value="en" placeholder="en" />
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Target Post ID + Lang</label>
                            <div style="display:grid;grid-template-columns:1fr 90px;gap:8px;">
                                <input type="number" id="linkTargetId" class="form-input" placeholder="Post ID (e.g. 42)" />
                                <input type="text" id="linkTargetLang" class="form-input" value="de" placeholder="de" />
                            </div>
                        </div>
                        <button type="button" class="btn btn-secondary" style="width:100%;" onclick="linkTranslationsManual()">Link Translation Group</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
            function applyLangPreset(val) {{
                if (!val) return;
                const [code, name, nativeName, locale, flag] = val.split('|');
                document.getElementById('newLangCode').value = code || '';
                document.getElementById('newLangName').value = name || '';
                document.getElementById('newLangNative').value = nativeName || '';
                document.getElementById('newLangLocale').value = locale || '';
                document.getElementById('newLangFlag').value = flag || '🌐';
                const rtlBox = document.getElementById('newLangRtl');
                if (rtlBox) rtlBox.checked = (code === 'ar' || code === 'he' || code === 'fa' || code === 'ur');
            }}

            async function addSiteLanguage() {{
                const code = document.getElementById('newLangCode').value.trim().toLowerCase();
                if (!code) {{ showToast('Please enter an ISO code (e.g. de, nl, fr)', true); return; }}
                const rtlBox = document.getElementById('newLangRtl');
                const payload = {{
                    code,
                    name: document.getElementById('newLangName').value.trim() || code.toUpperCase(),
                    native_name: document.getElementById('newLangNative').value.trim() || code.toUpperCase(),
                    locale: document.getElementById('newLangLocale').value.trim() || code,
                    flag_emoji: document.getElementById('newLangFlag').value.trim() || '🌐',
                    is_rtl: rtlBox ? rtlBox.checked : (code === 'ar' || code === 'he'),
                    is_default: false,
                    is_active: true
                }};
                const res = await fetch('/wp-json/daypress/v1/languages', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(payload)
                }});
                if (res.ok) {{
                    showToast('✓ Language ' + code.toUpperCase() + ' saved to SQLite');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to add language', true);
                }}
            }}

            function setAsDefaultLangFromBtn(el) {{
                return setAsDefaultLang(
                    el.dataset.code || '',
                    el.dataset.name || '',
                    el.dataset.native || '',
                    el.dataset.locale || '',
                    el.dataset.flag || ''
                );
            }}

            async function setAsDefaultLang(code, name, native_name, locale, flag_emoji) {{
                const res = await fetch('/wp-json/daypress/v1/languages', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'set_default', code, name, native_name, locale, flag_emoji, is_default: true, is_active: true }})
                }});
                if (res.ok) {{
                    showToast('✓ Default language updated to ' + code.toUpperCase());
                    setTimeout(() => window.location.reload(), 400);
                }}
            }}

            async function removeLang(code) {{
                if (!confirm('Remove language ' + code.toUpperCase() + '?')) return;
                const res = await fetch('/wp-json/daypress/v1/languages/' + encodeURIComponent(code), {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('✓ Removed language ' + code.toUpperCase());
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Cannot delete default language', true);
                }}
            }}

            async function savePrefixMode(prefixDefault) {{
                const res = await fetch('/wp-json/daypress/v1/languages', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'set_prefix_default', prefix_default: !!prefixDefault }})
                }});
                if (res.ok) {{
                    showToast('✓ URL prefix routing updated in SQLite');
                    setTimeout(() => window.location.reload(), 400);
                }}
            }}

            async function quickCreateTranslation(postId, targetLang) {{
                showToast('Creating ' + targetLang.toUpperCase() + ' translation draft...');
                const res = await fetch('/wp-json/daypress/v1/posts/' + postId + '/translate', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ lang_code: targetLang }})
                }});
                if (res.ok) {{
                    const cloned = await res.json();
                    showToast('✓ Created ' + targetLang.toUpperCase() + ' draft #' + cloned.id);
                    setTimeout(() => {{ window.location.href = '/d-admin/post?post=' + cloned.id + '&action=edit'; }}, 400);
                }} else {{
                    showToast('Failed to create translation draft', true);
                }}
            }}

            async function linkTranslationsManual() {{
                const source_id = parseInt(document.getElementById('linkSrcId').value, 10) || 0;
                const target_id = parseInt(document.getElementById('linkTargetId').value, 10) || 0;
                const source_lang = document.getElementById('linkSrcLang').value.trim() || 'en';
                const target_lang = document.getElementById('linkTargetLang').value.trim() || 'de';
                if (!source_id || !target_id) {{ showToast('Enter both source and target Post IDs', true); return; }}
                const res = await fetch('/wp-json/daypress/v1/translations/link', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ object_type: 'post', source_id, source_lang, target_id, target_lang }})
                }});
                if (res.ok) {{
                    const data = await res.json();
                    showToast('✓ Linked posts into translation group: ' + data.translation_group_id);
                }} else {{
                    showToast('Failed to link translations', true);
                }}
            }}
        </script>
        "#
    );

    Html(templates::render_admin_shell(
        "languages",
        "Languages & Multilingual (i18n)",
        &body,
        &settings,
    ))
}
