//! Daypress SEO & Readability Studio client runtime (`scripts/seo_studio.rs`).

pub fn render_seo_script(type_singular: &str) -> String {
    format!(
        r#"
            function updateSerpPreview() {{
                const rawTitle = titleInput.value || 'Untitled {type_singular}';
                const seoTitleEl = document.getElementById('seo_title');
                const customSeoTitle = seoTitleEl ? seoTitleEl.value.trim() : '';
                const displayTitle = customSeoTitle || (rawTitle + ' – ' + siteTitle);
                const slug = slugInput.value || 'slug';
                const textSource = currentMode === 'visual' ? (visualEditor.innerText || '') : contentArea.value.replace(/<[^>]*>/g, ' ');
                const seoDescEl = document.getElementById('seo_description');
                const customSeoDesc = seoDescEl ? seoDescEl.value.trim() : '';
                const excerpt = customSeoDesc || document.getElementById('excerpt').value || (textSource ? textSource.substring(0, 155) + '...' : 'No meta description provided...');

                document.getElementById('serpTitle').textContent = displayTitle;
                document.getElementById('serpUrl').textContent = window.location.origin + '/' + slug + '/';
                document.getElementById('serpDesc').textContent = excerpt;
                document.getElementById('viewPageLink').href = '/' + slug + '/';

                // Compute live SEO & Readability score
                const kwEl = document.getElementById('seo_focus_kw');
                const kw = kwEl ? kwEl.value.trim().toLowerCase() : '';
                let score = 50;
                const checks = [];
                if (rawTitle.length >= 10) {{ score += 15; checks.push('✓ Title length'); }}
                if (excerpt.length >= 50 && excerpt.length <= 165) {{ score += 15; checks.push('✓ Meta length'); }}
                if (textSource.trim().split(/\s+/).filter(Boolean).length >= 50) {{ score += 10; }}
                if (kw) {{
                    if (displayTitle.toLowerCase().includes(kw)) {{ score += 5; checks.push('✓ KW in title'); }}
                    if (excerpt.toLowerCase().includes(kw) || textSource.toLowerCase().includes(kw)) {{ score += 5; checks.push('✓ KW in body'); }}
                }} else {{
                    score += 5;
                }}
                if (score > 100) score = 100;
                const scoreBadge = document.getElementById('seoScoreBadge');
                if (scoreBadge) {{
                    scoreBadge.textContent = 'SEO Score: ' + score + ' / 100';
                    if (score >= 80) {{
                        scoreBadge.style.background = 'var(--moss-wash)';
                        scoreBadge.style.color = 'var(--moss)';
                        scoreBadge.style.borderColor = 'var(--moss)';
                    }} else {{
                        scoreBadge.style.background = 'var(--ochre-wash)';
                        scoreBadge.style.color = 'var(--ink)';
                        scoreBadge.style.borderColor = 'var(--ochre)';
                    }}
                }}
                const summaryEl = document.getElementById('seoChecklistSummary');
                if (summaryEl) {{
                    summaryEl.textContent = checks.join(' • ');
                }}
            }}

            function addTagChip(tag) {{
                const input = document.getElementById('post_tags');
                if (!input) return;
                const current = input.value.split(',').map(s => s.trim()).filter(Boolean);
                if (!current.includes(tag)) {{
                    current.push(tag);
                    input.value = current.join(', ');
                    showToast('✓ Tag added: ' + tag);
                    markDirty();
                }}
            }}

            async function inlineAddCategory() {{
                const input = document.getElementById('newCatName');
                const name = input.value.trim();
                if (!name) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/categories', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ name }})
                    }});
                    const data = await res.json();
                    if (res.ok) {{
                        const list = document.getElementById('categoryList');
                        const label = document.createElement('label');
                        label.style = "display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer;";
                        label.innerHTML = '<input type="checkbox" name="post_category" value="' + data.id + '" checked /> <span>' + data.name + '</span>';
                        list.appendChild(label);
                        input.value = '';
                        showToast('✓ Category created: ' + name);
                        markDirty();
                    }}
                }} catch(e) {{
                    showToast('Failed to create category', true);
                }}
            }}
        "#,
        type_singular = type_singular,
    )
}
