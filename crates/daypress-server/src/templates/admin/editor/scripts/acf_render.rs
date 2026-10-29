//! ACF and Gutenberg blocks card list renderer (`scripts/acf_render.rs`).

pub const JS_ACF_RENDER: &str = r###"
            function escapeAttr(str) {
                return (str || '').replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
            }

            function renderAcfBlocksList() {
                const container = document.getElementById('acfBlocksList');
                const badge = document.getElementById('acfBlockCountBadge');
                if (!container) return;
                if (badge) badge.textContent = acfBlocks.length + (acfBlocks.length === 1 ? ' section' : ' sections');

                if (acfBlocks.length === 0) {
                    container.innerHTML = `<div style="padding: 28px; text-align: center; border: 1px dashed var(--rule-strong); border-radius: var(--radius-sm); color: var(--ink-muted); font-size: 13px;">No structured ACF or Gutenberg sections yet. Click <strong>+ Add Block</strong> above or switch to Theme Preview.</div>`;
                    return;
                }

                container.innerHTML = acfBlocks.map((b, i) => {
                    const mainTitle = (b.headings && b.headings[0] && b.headings[0].text) ? b.headings[0].text : (b.isPageheader ? 'Pageheader Hero Banner' : (b.blockType || `Section #${i + 1}`));
                    const bgColorSwatch = b.bg === 'black' ? '#121212' : (b.bg === 'blue' ? '#0066ff' : (b.bg === 'lblue' ? '#eff6ff' : '#ffffff'));
                    const bgTextSwatch = (b.bg === 'black' || b.bg === 'blue') ? '#ffffff' : '#0f172a';

                    let controlsHtml = '';
                    if (b.isSection) {
                        controlsHtml = `
                            <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap; padding: 8px 12px; background: var(--paper-raised); border-bottom: 1px solid var(--rule); font-size: 11.5px;">
                                <label style="display: flex; align-items: center; gap: 6px; font-weight: 600; color: var(--ink-muted);">
                                    <span>ACF Section Theme:</span>
                                    <select onchange="updateAcfSectionProp(${i}, 'bg', this.value)" style="padding: 2px 8px; border-radius: 4px; border: 1px solid var(--rule-strong); font-size: 11.5px; font-weight: 600; background: ${bgColorSwatch}; color: ${bgTextSwatch};">
                                        <option value="white" ${b.bg === 'white' ? 'selected' : ''} style="background:var(--paper-raised);color:var(--ink);">White (.section.white)</option>
                                        <option value="black" ${b.bg === 'black' ? 'selected' : ''} style="background:var(--ink);color:var(--paper-raised);">Black (.section.black)</option>
                                        <option value="blue" ${b.bg === 'blue' ? 'selected' : ''} style="background:var(--press);color:var(--paper-raised);">Primary Blue (.section.blue)</option>
                                        <option value="lblue" ${b.bg === 'lblue' ? 'selected' : ''} style="background:var(--press-wash);color:var(--ink);">Light Blue (.section.lblue)</option>
                                    </select>
                                </label>
                                <label style="display: flex; align-items: center; gap: 5px; font-weight: 600; color: var(--ink-muted); cursor: pointer;">
                                    <input type="checkbox" ${b.hasAngle ? 'checked' : ''} onchange="updateAcfSectionProp(${i}, 'angle', this.checked)" />
                                    <span>Diagonal Angle Divider (<code>.angle</code>)</span>
                                </label>
                                ${b.slideCount > 0 ? `<span style="margin-left: auto; color: var(--ink-muted); font-family: var(--font-mono); font-size: 11px;">${b.slideCount} inner items / columns</span>` : ''}
                            </div>
                        `;
                    } else if (b.isPageheader) {
                        controlsHtml = `
                            <div style="display: flex; align-items: center; gap: 12px; padding: 8px 12px; background: var(--press-wash); border-bottom: 1px solid var(--press); font-size: 11.5px; color: var(--press);">
                                <label style="display: flex; align-items: center; gap: 6px; font-weight: 600;">
                                    <span>Banner Mode:</span>
                                    <select onchange="updateAcfSectionProp(${i}, 'heroFull', this.value === 'full')" style="padding: 2px 8px; border-radius: 4px; border: 1px solid var(--press); font-size: 11.5px; background: var(--paper-raised);">
                                        <option value="full" ${b.heroFull ? 'selected' : ''}>Full Height Hero (.pageheader.full)</option>
                                        <option value="compact" ${!b.heroFull ? 'selected' : ''}>Compact Pageheader (.pageheader)</option>
                                    </select>
                                </label>
                                <span style="font-size: 11px; color: var(--press);">Synced with ACF Pageheader Settings below</span>
                            </div>
                        `;
                    }

                    const headingsEditorHtml = (b.headings || []).slice(0, 4).map((h, hIdx) => `
                        <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px;">
                            <span style="font-size: 10px; font-weight: 700; text-transform: uppercase; background: var(--paper-sunk); color: var(--ink-muted); border: 1px solid var(--rule-strong); border-radius: 4px; padding: 2px 6px; min-width: 32px; text-align: center; font-family: var(--font-mono);">${h.tag}</span>
                            <input type="text" class="form-input" value="${escapeAttr(h.text)}" oninput="updateAcfHeading(${i}, ${hIdx}, this.value)" style="flex: 1; font-weight: 700; font-size: 13.5px; height: 32px;" />
                        </div>
                    `).join('');

                    const paragraphsEditorHtml = (b.paragraphs || []).slice(0, 3).map((pHtml, pIdx) => `
                        <div style="margin-bottom: 8px;">
                            <label style="font-size: 10.5px; font-weight: 600; color: var(--ink-muted); display: block; margin-bottom: 3px;">Paragraph / Body Copy #${pIdx + 1}</label>
                            <textarea class="form-textarea" rows="2" oninput="updateAcfParagraph(${i}, ${pIdx}, this.value)" style="font-size: 12.5px; line-height: 1.45;">${pHtml.replace(/</g, '&lt;').replace(/>/g, '&gt;')}</textarea>
                        </div>
                    `).join('');

                    const buttonsEditorHtml = (b.buttons && b.buttons.length > 0) ? `
                        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 8px; margin-top: 8px; padding-top: 8px; border-top: 1px dashed var(--rule);">
                            ${b.buttons.slice(0, 4).map((btn, btnIdx) => `
                                <div style="display: flex; align-items: center; gap: 6px; background: var(--paper-raised); padding: 6px 8px; border-radius: 6px; border: 1px solid var(--rule);">
                                    <span style="font-size: 10px; font-weight: 700; color: var(--press);">CTA</span>
                                    <input type="text" value="${escapeAttr(btn.text)}" oninput="updateAcfButton(${i}, ${btnIdx}, 'text', this.value)" placeholder="Button label" style="flex: 1; font-size: 11.5px; padding: 4px 6px; background: var(--paper-raised); color: var(--ink); border: 1px solid var(--rule-strong); border-radius: 4px;" />
                                    <input type="text" value="${escapeAttr(btn.href)}" oninput="updateAcfButton(${i}, ${btnIdx}, 'href', this.value)" placeholder="URL" style="width: 110px; font-size: 11px; font-family: var(--font-mono); padding: 4px 6px; background: var(--paper-raised); color: var(--ink); border: 1px solid var(--rule-strong); border-radius: 4px;" />
                                </div>
                            `).join('')}
                        </div>
                    ` : '';

                    return `
                        <div class="acf-block-card" style="border: 1px solid var(--rule-strong); border-radius: var(--radius-sm); background: var(--paper-raised); overflow: hidden; box-shadow: 0 1px 2px rgba(15,23,42,0.04);">
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; background: var(--paper-raised); border-bottom: 1px solid var(--rule);">
                                <div style="display: flex; align-items: center; gap: 10px; min-width: 0;">
                                    <span style="font-size: 11px; font-weight: 700; font-family: var(--font-mono); color: var(--ink-muted);">#${i + 1}</span>
                                    <span style="background: ${b.badgeBg}; color: ${b.badgeColor}; font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 999px; white-space: nowrap;">${b.blockType}</span>
                                    <strong style="font-size: 13px; color: var(--ink); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${escapeAttr(mainTitle)}</strong>
                                </div>
                                <div style="display: flex; align-items: center; gap: 4px;">
                                    <button type="button" class="btn btn-secondary" style="padding: 2px 7px; font-size: 11px; height: 24px;" title="Move Up" onclick="moveAcfBlock(${i}, -1)" ${i === 0 ? 'disabled' : ''}>↑</button>
                                    <button type="button" class="btn btn-secondary" style="padding: 2px 7px; font-size: 11px; height: 24px;" title="Move Down" onclick="moveAcfBlock(${i}, 1)" ${i === acfBlocks.length - 1 ? 'disabled' : ''}>↓</button>
                                    <button type="button" class="btn btn-secondary" style="padding: 2px 7px; font-size: 11px; height: 24px;" title="Duplicate Section" onclick="duplicateAcfBlock(${i})">⧉</button>
                                    <button type="button" class="btn btn-secondary" style="padding: 2px 7px; font-size: 11px; height: 24px; font-family: var(--font-mono);" title="Inspect / Edit Section HTML" onclick="toggleAcfRawHtml(${i})">&lt;/&gt;</button>
                                    <button type="button" class="btn btn-secondary" style="padding: 2px 7px; font-size: 11px; height: 24px; color: var(--oxide);" title="Delete Section" onclick="deleteAcfBlock(${i})">✕</button>
                                </div>
                            </div>
                            ${controlsHtml}
                            <div style="padding: 12px 14px;">
                                ${headingsEditorHtml || '<div style="font-size: 12px; color: var(--ink-muted); margin-bottom: 6px;">Dynamic ACF / Gutenberg template block</div>'}
                                ${paragraphsEditorHtml}
                                ${buttonsEditorHtml}
                                <div id="acfRawWrap_${i}" style="display: ${b.showRaw ? 'block' : 'none'}; margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--rule);">
                                    <label style="font-size: 11px; font-weight: 700; color: var(--ink-muted); display: block; margin-bottom: 4px; font-family: var(--font-mono);">Section HTML Markup (Synced with Post Content &amp; DayStudio AST)</label>
                                    <textarea class="form-textarea" rows="5" style="font-family: var(--font-mono); font-size: 11.5px; background: var(--ink); color: var(--ink-muted); padding: 10px;" oninput="updateAcfRawHtml(${i}, this.value)">${(b.outerHtml || '').replace(/</g, '&lt;').replace(/>/g, '&gt;')}</textarea>
                                </div>
                            </div>
                        </div>
                    `;
                }).join('');
            }
"###;
