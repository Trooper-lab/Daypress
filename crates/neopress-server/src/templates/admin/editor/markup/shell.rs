//! Editor main card, SEO studio card, and media picker modal (`markup/shell.rs`).

pub struct EditorCardParams<'a> {
    pub type_singular: &'a str,
    pub initial_title: &'a str,
    pub initial_slug: &'a str,
    pub initial_content: &'a str,
    pub locked_banner_html: &'a str,
    pub standard_editor_display: &'a str,
    /// `flex` for posts that use ACF, `none` otherwise.
    pub acf_inserter_display: &'a str,
}

pub fn render_editor_card(p: &EditorCardParams) -> String {
    format!(
        r#"<div class="neo-card" style="padding: 24px;">
            <input type="text" name="post_title" id="title" placeholder="Add {type_singular} title..." value="{initial_title}"
                   style="width: 100%; font-size: 28px; font-weight: 700; border: none; outline: none; padding: 4px 0; background: #ffffff; color: #0f172a; letter-spacing: -0.02em;" required />
            
            <div style="display: flex; align-items: center; gap: 6px; padding: 10px 0 16px 0; border-bottom: 1px solid #f1f5f9; margin-bottom: 14px; font-size: 12.5px; color: #64748b;">
                <span>Permalink:</span>
                <span style="font-family: var(--font-mono); color: #0f172a;">http://localhost:8090/</span>
                <input type="text" id="post_name" placeholder="slug" value="{initial_slug}" 
                       style="height: 24px; padding: 0 6px; font-size: 12px; font-family: var(--font-mono); background: #ffffff; color: #0f172a; border: 1px solid #cbd5e1; border-radius: 4px; width: 220px;" />
                <span>/</span>
                <button type="button" class="btn btn-secondary" style="padding: 2px 6px; font-size: 11px; height: 22px;" onclick="copyPermalink()">Copy Link</button>
                <a id="viewPageLink" href="/{initial_slug}/" target="_blank" style="color: #2563eb; text-decoration: none; font-size: 11.5px; margin-left: 4px;">Visit Page ↗</a>
            </div>

            <div id="draftRecoveryBanner" style="display: none; background: #fffbeb; border: 1px solid #fde68a; border-radius: 8px; padding: 10px 16px; margin-bottom: 14px; font-size: 12.5px; color: #92400e; align-items: center; justify-content: space-between;">
                <span><strong>Autosave Recovery:</strong> Found an unsaved local draft from <span id="recoveryTime"></span>.</span>
                <div style="display: flex; gap: 8px;">
                    <button type="button" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px;" onclick="restoreLocalDraft()">Restore Draft</button>
                    <button type="button" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: #ef4444;" onclick="dismissLocalDraft()">Dismiss</button>
                </div>
            </div>

            {locked_banner_html}

            <div id="standardEditorContainer" style="{standard_editor_display}">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; padding-bottom: 12px; border-bottom: 1px solid #f1f5f9;">
                    <div style="display: inline-flex; background: #f1f5f9; padding: 3px; border-radius: 8px; border: 1px solid #e2e8f0; gap: 2px;">
                        <button type="button" id="btnModeBlocks" onmousedown="event.preventDefault()" onclick="switchEditorMode('blocks')" style="display: flex; align-items: center; gap: 6px; padding: 6px 14px; font-size: 12px; font-weight: 600; border-radius: 6px; border: none; cursor: pointer; background: #ffffff; color: #0f172a; box-shadow: 0 1px 3px rgba(0,0,0,0.1); transition: all 0.15s ease;">
                            <span>🧩 Blocks</span>
                        </button>
                        <button type="button" id="btnModeVisual" onmousedown="event.preventDefault()" onclick="switchEditorMode('visual')" style="display: flex; align-items: center; gap: 6px; padding: 6px 14px; font-size: 12px; font-weight: 600; border-radius: 6px; border: none; cursor: pointer; background: transparent; color: #64748b; transition: all 0.15s ease;">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>
                            <span>Visual</span>
                        </button>
                        <button type="button" id="btnModeHtml" onmousedown="event.preventDefault()" onclick="switchEditorMode('html')" style="display: flex; align-items: center; gap: 6px; padding: 6px 14px; font-size: 12px; font-weight: 600; border-radius: 6px; border: none; cursor: pointer; background: transparent; color: #64748b; transition: all 0.15s ease;">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>
                            <span>&lt;/&gt; HTML Source</span>
                        </button>
                    </div>

                    <div style="display: flex; align-items: center; gap: 10px; font-size: 12px; color: #64748b; font-family: var(--font-mono);">
                        <span id="wordCount">0 words</span>
                        <span>&bull;</span>
                        <span id="readTime">0 min read</span>
                        <div id="autosaveBadge" style="display: inline-flex; align-items: center; gap: 6px; background: #ecfdf5; color: #059669; border: 1px solid #a7f3d0; border-radius: 9999px; padding: 3px 10px; font-size: 11px; font-weight: 600; transition: all 0.2s ease;">
                            <span id="autosaveDot" style="width: 6px; height: 6px; border-radius: 50%; background: #10b981;"></span>
                            <span id="autosaveLabel">Synced with SQLite</span>
                        </div>
                    </div>
                </div>

                <div id="editorToolbar" style="display: none; flex-wrap: wrap; align-items: center; gap: 5px; padding-bottom: 12px; border-bottom: 1px solid #f1f5f9; margin-bottom: 14px;">
                    <select id="blockTypeSelect" class="form-select" style="width: auto; padding: 4px 8px; font-size: 12px; height: 30px;" onchange="formatBlockType(this.value); this.selectedIndex=0;">
                        <option value="">Block Type...</option>
                        <option value="p">Paragraph</option>
                        <option value="h1">Heading 1</option>
                        <option value="h2">Heading 2</option>
                        <option value="h3">Heading 3</option>
                        <option value="h4">Heading 4</option>
                    </select>

                    <div style="width: 1px; height: 20px; background: #e2e8f0; margin: 0 4px;"></div>

                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px; font-weight: 700;" title="Bold (Ctrl+B)" onmousedown="event.preventDefault()" onclick="execStyle('bold')">B</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px; font-style: italic;" title="Italic (Ctrl+I)" onmousedown="event.preventDefault()" onclick="execStyle('italic')">I</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px; text-decoration: underline;" title="Underline (Ctrl+U)" onmousedown="event.preventDefault()" onclick="execStyle('underline')">U</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px; text-decoration: line-through;" title="Strikethrough" onmousedown="event.preventDefault()" onclick="execStyle('strikeThrough')">S</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 11px; font-family: var(--font-mono);" title="Inline Code" onmousedown="event.preventDefault()" onclick="insertInlineCode()">&lt;&gt;</button>

                    <div style="width: 1px; height: 20px; background: #e2e8f0; margin: 0 4px;"></div>

                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;" title="Bullet List" onmousedown="event.preventDefault()" onclick="insertList('ul')">• List</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;" title="Numbered List" onmousedown="event.preventDefault()" onclick="insertList('ol')">1. List</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;" title="Blockquote" onmousedown="event.preventDefault()" onclick="insertQuote()">” Quote</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;" title="Insert Link (Ctrl+K)" onmousedown="event.preventDefault()" onclick="insertLinkPrompt()">🔗 Link</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;" title="Insert Table" onmousedown="event.preventDefault()" onclick="insertTable()">▦ Table</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;" title="Callout Box" onmousedown="event.preventDefault()" onclick="insertCallout()">💡 Callout</button>
                    <button type="button" class="btn btn-secondary" style="padding: 4px 8px; font-size: 12px;" title="Clear Formatting" onmousedown="event.preventDefault()" onclick="clearFormatting()">Tx</button>

                    <div style="width: 1px; height: 20px; background: #e2e8f0; margin: 0 4px;"></div>

                    <button type="button" class="btn btn-secondary" style="padding: 4px 10px; font-size: 12px; color: #2563eb; font-weight: 600;" onmousedown="event.preventDefault()" onclick="openMediaPicker('content')">+ Add Media</button>
                </div>

                <div id="editorCanvasWrapper" style="position: relative;">
                    <div id="acfBlocksContainer" style="display: block;">
                        <div style="display: flex; justify-content: space-between; align-items: center; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 14px; margin-bottom: 14px;">
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="font-size: 12px; font-weight: 700; color: #0f172a;">Block Stack</span>
                                <span id="acfBlockCountBadge" style="background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; border-radius: 999px; padding: 2px 8px; font-size: 11px; font-weight: 600;">0 sections</span>
                            </div>
                            <div id="acfBlockInserter" style="display: {acf_inserter_display}; gap: 6px; align-items: center;">
                                <select id="newAcfBlockSelect" class="form-select" style="height: 28px; padding: 2px 8px; font-size: 11.5px; width: auto;">
                                    <option value="section_white">+ ACF Section (White + Content)</option>
                                    <option value="section_black">+ ACF Section (Black + Diagonal Angle)</option>
                                    <option value="section_blue">+ ACF Section (Blue + Content)</option>
                                    <option value="acf_3col">+ ACF Block: 3-Col Feature Cards (acf/threecol)</option>
                                    <option value="acf_logos">+ ACF Block: Client Logos Carousel (acf/logos)</option>
                                    <option value="acf_trainingen">+ ACF Block: Showcase Carousel (acf/showcase)</option>
                                    <option value="acf_faq">+ ACF Block: FAQ Accordion (acf/faq)</option>
                                    <option value="acf_quotes">+ ACF Block: Quotes Testimonial Slider (acf/quotes)</option>
                                </select>
                                <button type="button" class="btn btn-primary" style="height: 28px; padding: 0 12px; font-size: 11.5px;" onclick="insertNewAcfSection()">+ Add Block</button>
                            </div>
                        </div>
                        <div id="acfBlocksList" style="display: flex; flex-direction: column; gap: 12px;"></div>
                    </div>
                    <div id="visualEditor" contenteditable="true" spellcheck="true" class="neo-visual-canvas" style="display: none;"
                         placeholder="Start typing your story, or insert media and headings..."
                         oninput="onVisualInput()" onblur="syncVisualToTextarea()"></div>
                    <textarea id="content" name="content" class="neo-html-textarea" style="display: none;"
                              placeholder="<!-- Write or edit raw HTML source here -->"
                              oninput="onHtmlInput()">{initial_content}</textarea>
                </div>
            </div>
        </div>"#,
        type_singular = p.type_singular,
        initial_title = p.initial_title,
        initial_slug = p.initial_slug,
        initial_content = p.initial_content,
        locked_banner_html = p.locked_banner_html,
        standard_editor_display = p.standard_editor_display,
        acf_inserter_display = p.acf_inserter_display,
    )
}

pub struct SeoCardParams<'a> {
    pub type_singular: &'a str,
    pub site_title: &'a str,
    pub initial_title: &'a str,
    pub initial_slug: &'a str,
    pub initial_excerpt: &'a str,
    pub initial_seo_title: &'a str,
    pub initial_seo_desc: &'a str,
    pub initial_seo_kw: &'a str,
    pub initial_seo_canonical: &'a str,
    pub initial_seo_og_img: &'a str,
    pub seo_noindex_checked: &'a str,
    pub schema_art_sel: &'a str,
    pub schema_web_sel: &'a str,
    pub schema_faq_sel: &'a str,
    pub schema_prod_sel: &'a str,
    pub schema_loc_sel: &'a str,
}

pub fn render_seo_card(p: &SeoCardParams) -> String {
    format!(
        r#"<div class="neo-card" style="padding: 22px;">
            <div class="neo-card-header" style="padding: 0 0 14px 0; margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h2 class="neo-card-title">Neopress SEO &amp; Readability Studio</h2>
                    <span class="badge badge-published" style="font-size: 11px;">Yoast / RankMath Parity</span>
                </div>
                <div id="seoScoreBadge" style="background: #ecfdf5; color: #059669; border: 1px solid #a7f3d0; padding: 3px 10px; border-radius: 999px; font-size: 12px; font-weight: 700;">
                    SEO Score: 85 / 100
                </div>
            </div>

            <div style="margin-bottom: 18px; background: #f8fafc; border: 1px solid var(--border-subtle); border-radius: 8px; padding: 14px;">
                <span style="font-size: 10.5px; font-weight: 700; text-transform: uppercase; color: #64748b; letter-spacing: 0.05em; display: block; margin-bottom: 6px;">Google Search Snippet Preview</span>
                <div id="serpTitle" style="color: #1a0dab; font-size: 17px; font-weight: 500; text-decoration: underline; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{initial_title} – {site_title}</div>
                <div id="serpUrl" style="color: #006621; font-size: 12.5px; font-family: var(--font-mono); margin: 2px 0 4px 0;">http://localhost:8090/{initial_slug}/</div>
                <div id="serpDesc" style="color: #545454; font-size: 13px; line-height: 1.45;">{initial_excerpt}</div>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 14px;">
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">🎯 Focus Keyphrase</label>
                    <input type="text" id="seo_focus_kw" class="form-input" value="{initial_seo_kw}" placeholder="e.g. rust wordpress cms" oninput="updateSerpPreview(); markDirty();" />
                </div>
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">Schema.org Structured Data Type</label>
                    <select id="seo_schema_type" class="form-select" onchange="markDirty()">
                        <option value="Article" {schema_art_sel}>Article / BlogPosting</option>
                        <option value="WebPage" {schema_web_sel}>WebPage</option>
                        <option value="FAQPage" {schema_faq_sel}>FAQPage</option>
                        <option value="Product" {schema_prod_sel}>Product</option>
                        <option value="LocalBusiness" {schema_loc_sel}>LocalBusiness</option>
                    </select>
                </div>
            </div>

            <div class="form-group" style="margin-bottom: 14px;">
                <label class="form-label">Custom SEO Title <span style="font-weight: 400; color: #64748b; font-size: 11.5px;">(Leave blank to use default title + site name)</span></label>
                <input type="text" id="seo_title" class="form-input" value="{initial_seo_title}" placeholder="{initial_title} – {site_title}" oninput="updateSerpPreview(); markDirty();" />
            </div>

            <div class="form-group" style="margin-bottom: 14px;">
                <label class="form-label">SEO Meta Description</label>
                <textarea id="seo_description" rows="2" class="form-textarea" placeholder="Compelling 120–158 character summary for search results..." oninput="updateSerpPreview(); markDirty();">{initial_seo_desc}</textarea>
            </div>

            <div class="form-group" style="margin-bottom: 14px;">
                <label class="form-label">Post Excerpt (Theme Archive Summary)</label>
                <textarea id="excerpt" rows="2" class="form-textarea" placeholder="Handcrafted summary for blog archives and RSS feeds..." oninput="updateSerpPreview(); markDirty();">{initial_excerpt}</textarea>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 14px;">
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">Canonical URL Override</label>
                    <input type="text" id="seo_canonical" class="form-input" value="{initial_seo_canonical}" placeholder="https://example.com/canonical-slug/" oninput="markDirty()" />
                </div>
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">OpenGraph / Social Image URL</label>
                    <input type="text" id="seo_og_image" class="form-input" value="{initial_seo_og_img}" placeholder="/uploads/social-card.webp" oninput="markDirty()" />
                </div>
            </div>

            <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 12px; border-top: 1px solid #f1f5f9;">
                <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; color: #0f172a; font-weight: 500;">
                    <input type="checkbox" id="seo_noindex" {seo_noindex_checked} onchange="updateSerpPreview(); markDirty();" />
                    <span>Discourage search engines from indexing this {type_singular} (<code>noindex, follow</code>)</span>
                </label>
                <div id="seoChecklistSummary" style="font-size: 12px; color: #64748b;"></div>
            </div>
        </div>"#,
        initial_title = p.initial_title,
        site_title = p.site_title,
        initial_slug = p.initial_slug,
        initial_excerpt = p.initial_excerpt,
        initial_seo_kw = p.initial_seo_kw,
        schema_art_sel = p.schema_art_sel,
        schema_web_sel = p.schema_web_sel,
        schema_faq_sel = p.schema_faq_sel,
        schema_prod_sel = p.schema_prod_sel,
        schema_loc_sel = p.schema_loc_sel,
        initial_seo_title = p.initial_seo_title,
        initial_seo_desc = p.initial_seo_desc,
        initial_seo_canonical = p.initial_seo_canonical,
        initial_seo_og_img = p.initial_seo_og_img,
        seo_noindex_checked = p.seo_noindex_checked,
        type_singular = p.type_singular,
    )
}

pub fn render_media_picker_modal(media_picker_items: &str) -> String {
    format!(
        r#"<div id="mediaPickerModal" style="display: none; position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(15,23,42,0.65); backdrop-filter: blur(4px); z-index: 10000; align-items: center; justify-content: center;">
            <div style="background: #ffffff; border-radius: 12px; width: 720px; max-width: 90vw; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);">
                <div style="padding: 16px 20px; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Select Media Asset</h3>
                        <span class="badge badge-published" style="font-size: 11px;">Rusqlite Storage</span>
                    </div>
                    <button type="button" onclick="closeMediaPicker()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: #94a3b8;">&times;</button>
                </div>

                <div style="padding: 12px 20px; border-bottom: 1px solid var(--border-subtle); display: flex; gap: 10px; background: #f8fafc;">
                    <button type="button" class="btn btn-primary" id="tabLibraryBtn" onclick="switchMediaTab('library')">Media Library</button>
                    <button type="button" class="btn btn-secondary" id="tabUploadBtn" onclick="switchMediaTab('upload')">Upload New Image</button>
                </div>

                <div id="mediaTabLibrary" style="padding: 20px; flex: 1; overflow-y: auto;">
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 12px;">
                        {media_picker_items}
                    </div>
                </div>

                <div id="mediaTabUpload" style="padding: 40px; display: none; text-align: center;">
                    <div style="border: 2px dashed #cbd5e1; border-radius: 10px; padding: 40px; cursor: pointer;" onclick="document.getElementById('modalFileInput').click()">
                        <div style="font-size: 32px; margin-bottom: 8px;">📤</div>
                        <h4 style="font-size: 15px; font-weight: 600; margin: 0 0 6px 0;">Drop image here to upload</h4>
                        <p style="font-size: 12.5px; color: #64748b; margin: 0 0 16px 0;">Streamed directly into Neopress uploads directory</p>
                        <button type="button" class="btn btn-primary">Choose File</button>
                        <input type="file" id="modalFileInput" accept="image/*" style="display: none;" onchange="uploadModalImage(this.files[0])" />
                    </div>
                </div>

                <div style="padding: 12px 20px; border-top: 1px solid var(--border-subtle); background: #f8fafc; display: flex; justify-content: flex-end; gap: 8px;">
                    <button type="button" class="btn btn-secondary" onclick="closeMediaPicker()">Cancel</button>
                </div>
            </div>
        </div>"#,
        media_picker_items = media_picker_items
    )
}
