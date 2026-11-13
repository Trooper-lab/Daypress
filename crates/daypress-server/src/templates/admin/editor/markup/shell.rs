//! Editor main card, SEO studio card, and media picker modal (`markup/shell.rs`).

pub struct EditorCardParams<'a> {
    pub type_singular: &'a str,
    pub permalink_prefix: &'a str,
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
        r#"<div id="post-body-content" style="margin-bottom: 16px;">
            <div id="titlediv">
                <div id="titlewrap">
                    <input type="text" name="post_title" id="title" class="wp-title-input" placeholder="Add title" value="{initial_title}" required />
                </div>
                <div id="edit-slug-box">
                    <strong>Permalink:</strong>
                    <span id="sample-permalink">http://localhost:8090/{permalink_prefix}<input type="text" id="post_name" class="wp-slug-input" placeholder="slug" value="{initial_slug}" />/</span>
                    <button type="button" class="btn btn-secondary" style="height: 24px; min-height: 24px; padding: 0 8px; font-size: 11px;" onclick="copyPermalink()">Copy Link</button>
                    <a id="viewPageLink" href="/{permalink_prefix}{initial_slug}/" target="_blank" style="color: var(--press); text-decoration: none; font-size: 12px; margin-left: 4px;">View {type_singular}</a>
                </div>
            </div>

            <div id="draftRecoveryBanner" style="display: none; background: var(--ochre-wash); border: 1px solid var(--ochre); border-radius: var(--radius-sm); padding: 10px 16px; margin-bottom: 14px; font-size: 12.5px; color: var(--ink); align-items: center; justify-content: space-between;">
                <span><strong>Autosave Recovery:</strong> Found an unsaved local draft from <span id="recoveryTime"></span>.</span>
                <div style="display: flex; gap: 8px;">
                    <button type="button" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px;" onclick="restoreLocalDraft()">Restore Draft</button>
                    <button type="button" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--oxide);" onclick="dismissLocalDraft()">Dismiss</button>
                </div>
            </div>

            {locked_banner_html}

            <div id="standardEditorContainer" style="{standard_editor_display}">
                <div id="wp-content-editor-tools">
                    <div id="wp-content-media-buttons">
                        <button type="button" class="btn btn-secondary" onmousedown="event.preventDefault()" onclick="openMediaPicker('content')">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
                            <span>Add Media</span>
                        </button>
                    </div>
                    <div class="wp-editor-tabs">
                        <button type="button" id="btnModeVisual" class="wp-switch-editor" onmousedown="event.preventDefault()" onclick="switchEditorMode('visual')">Visual</button>
                        <button type="button" id="btnModeHtml" class="wp-switch-editor" onmousedown="event.preventDefault()" onclick="switchEditorMode('html')">Code</button>
                        <button type="button" id="btnModeBlocks" class="wp-switch-editor is-active" onmousedown="event.preventDefault()" onclick="switchEditorMode('blocks')">Blocks</button>
                    </div>
                </div>

                <div id="wp-content-editor-container">
                    <div id="editorToolbar" class="wp-editor-toolbar" style="display: none;">
                        <select id="blockTypeSelect" class="form-select" style="width: auto; padding: 2px 24px 2px 8px; font-size: 12px; height: 26px; min-height: 26px;" onchange="formatBlockType(this.value); this.selectedIndex=0;">
                            <option value="p">Paragraph</option>
                            <option value="h1">Heading 1</option>
                            <option value="h2">Heading 2</option>
                            <option value="h3">Heading 3</option>
                            <option value="h4">Heading 4</option>
                        </select>
                        <span class="wp-mce-sep"></span>
                        <button type="button" class="wp-mce-btn" style="font-weight: 700;" title="Bold (Ctrl+B)" onmousedown="event.preventDefault()" onclick="execStyle('bold')">B</button>
                        <button type="button" class="wp-mce-btn" style="font-style: italic;" title="Italic (Ctrl+I)" onmousedown="event.preventDefault()" onclick="execStyle('italic')">I</button>
                        <button type="button" class="wp-mce-btn" style="text-decoration: underline;" title="Underline (Ctrl+U)" onmousedown="event.preventDefault()" onclick="execStyle('underline')">U</button>
                        <button type="button" class="wp-mce-btn" style="text-decoration: line-through;" title="Strikethrough" onmousedown="event.preventDefault()" onclick="execStyle('strikeThrough')">S</button>
                        <button type="button" class="wp-mce-btn" style="font-family: var(--font-mono); font-size: 11px;" title="Inline Code" onmousedown="event.preventDefault()" onclick="insertInlineCode()">&lt;&gt;</button>
                        <span class="wp-mce-sep"></span>
                        <button type="button" class="wp-mce-btn" title="Bullet List" onmousedown="event.preventDefault()" onclick="insertList('ul')">&bull;</button>
                        <button type="button" class="wp-mce-btn" title="Numbered List" onmousedown="event.preventDefault()" onclick="insertList('ol')">1.</button>
                        <button type="button" class="wp-mce-btn" title="Blockquote" onmousedown="event.preventDefault()" onclick="insertQuote()">&ldquo;</button>
                        <button type="button" class="wp-mce-btn" title="Insert/edit link (Ctrl+K)" onmousedown="event.preventDefault()" onclick="insertLinkPrompt()"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg></button>
                        <button type="button" class="wp-mce-btn" title="Insert Table" onmousedown="event.preventDefault()" onclick="insertTable()">&#9638;</button>
                        <button type="button" class="wp-mce-btn" title="Callout Box" onmousedown="event.preventDefault()" onclick="insertCallout()">&#9432;</button>
                        <button type="button" class="wp-mce-btn" title="Clear Formatting" onmousedown="event.preventDefault()" onclick="clearFormatting()">Tx</button>
                        <button type="button" id="btnEditorFullscreen" class="wp-mce-btn" style="margin-left: auto;" title="Distraction-free writing mode (Alt+Shift+W)" onclick="toggleEditorFullscreen()"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/></svg></button>
                    </div>

                    <div id="editorCanvasWrapper" style="position: relative;">
                        <div id="acfBlocksContainer" style="display: block; padding: 14px;">
                            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 10px; background: var(--paper); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 8px 12px; margin-bottom: 12px;">
                                <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                    <span style="font-size: 12px; font-weight: 600; color: var(--ink);">Blocks</span>
                                    <span id="acfBlockCountBadge" style="background: var(--paper-raised); color: var(--ink-soft); border: 1px solid var(--rule); border-radius: 3px; padding: 1px 6px; font-size: 11px;">0 sections</span>
                                    <button type="button" class="btn btn-secondary" style="height: 24px; min-height: 24px; padding: 0 8px; font-size: 11px;" title="Collapse all blocks" onclick="toggleAllAcfBlocks(true)">Collapse All</button>
                                    <button type="button" class="btn btn-secondary" style="height: 24px; min-height: 24px; padding: 0 8px; font-size: 11px;" title="Expand all blocks" onclick="toggleAllAcfBlocks(false)">Expand All</button>
                                </div>
                                <div id="acfBlockInserter" style="display: {acf_inserter_display}; gap: 6px; align-items: center;">
                                    <select id="newAcfBlockSelect" class="form-select" style="height: 26px; min-height: 26px; padding: 2px 24px 2px 8px; font-size: 11.5px; width: auto;">
                                        <optgroup label="Core Gutenberg Blocks">
                                            <option value="wp_paragraph">+ Paragraph Block (core/paragraph)</option>
                                            <option value="wp_heading">+ Heading + Lead Copy (core/heading)</option>
                                            <option value="wp_columns_2">+ 2-Column Split Row (core/columns)</option>
                                            <option value="wp_cta_box">+ Callout / CTA Banner (core/group)</option>
                                        </optgroup>
                                        <optgroup label="ACF &amp; Theme Sections">
                                            <option value="section_white">+ ACF Section (White + Content)</option>
                                            <option value="section_black">+ ACF Section (Black + Diagonal Angle)</option>
                                            <option value="section_blue">+ ACF Section (Blue + Content)</option>
                                            <option value="acf_3col">+ ACF Block: 3-Col Feature Cards (acf/threecol)</option>
                                            <option value="acf_logos">+ ACF Block: Client Logos Carousel (acf/logos)</option>
                                            <option value="acf_trainingen">+ ACF Block: Showcase Carousel (acf/showcase)</option>
                                            <option value="acf_faq">+ ACF Block: FAQ Accordion (acf/faq)</option>
                                            <option value="acf_quotes">+ ACF Block: Quotes Testimonial Slider (acf/quotes)</option>
                                        </optgroup>
                                    </select>
                                    <button type="button" class="btn btn-secondary" style="height: 26px; min-height: 26px; padding: 0 10px; font-size: 11.5px;" onclick="insertNewAcfSection()">+ Add Block</button>
                                </div>
                            </div>
                            <div id="acfBlocksList" style="display: flex; flex-direction: column; gap: 12px;"></div>
                        </div>
                        <div id="visualEditor" contenteditable="true" spellcheck="true" class="day-visual-canvas" style="display: none; border: none; border-radius: 0;"
                             placeholder="Start writing or type / to choose a block..."
                             oninput="onVisualInput()" onblur="syncVisualToTextarea()"></div>
                        <textarea id="content" name="content" class="day-html-textarea" style="display: none; border: none; border-radius: 0;"
                                  placeholder="<!-- Write or edit raw HTML source here -->"
                                  oninput="onHtmlInput()">{initial_content}</textarea>
                    </div>
                </div>

                <div id="post-status-info">
                    <div>Word count: <span id="wordCount">0 words</span> &bull; <span id="readTime">0 min read</span></div>
                    <div id="autosaveBadge" style="display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--ink-muted);">
                        <span id="autosaveDot" style="width: 6px; height: 6px; border-radius: 50%; background: var(--moss);"></span>
                        <span id="autosaveLabel">Saved</span>
                    </div>
                </div>
            </div>
        </div>"#,
        type_singular = p.type_singular,
        permalink_prefix = p.permalink_prefix,
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
        r#"<div class="day-card" id="metabox-excerpt" data-metabox-id="excerpt" data-metabox-title="Excerpt">
            <div class="day-card-header">
                <h2 class="day-card-title">Excerpt</h2>
            </div>
            <div class="day-card-body">
                <textarea id="excerpt" rows="3" class="form-textarea" placeholder="Write an excerpt (optional)..." oninput="updateSerpPreview(); markDirty();">{initial_excerpt}</textarea>
                <p style="color: var(--ink-muted); font-size: 12px; margin: 6px 0 0 0;">Excerpts are optional hand-crafted summaries of your content that can be used in your theme.</p>
            </div>
        </div>
        <div class="day-card" id="metabox-seo_studio" data-metabox-id="seo_studio" data-metabox-title="SEO &amp; Readability">
            <div class="day-card-header">
                <h2 class="day-card-title">SEO &amp; Readability</h2>
            </div>
            <div class="day-card-body">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                <span style="font-size: 12px; font-weight: 600; color: var(--ink-soft);">Search Engine Preview</span>
                <div id="seoScoreBadge" style="background: var(--moss-wash); color: var(--moss); border: 1px solid var(--moss); padding: 2px 8px; border-radius: 3px; font-size: 11.5px; font-weight: 600;">
                    SEO Score: 85 / 100
                </div>
            </div>
            <div style="margin-bottom: 16px; background: var(--paper); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 12px;">
                <div id="serpTitle" style="color: #1a0dab; font-size: 16px; font-weight: 500; text-decoration: underline; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{initial_title} – {site_title}</div>
                <div id="serpUrl" style="color: #006621; font-size: 12px; font-family: var(--font-mono); margin: 2px 0 4px 0;">http://localhost:8090/{initial_slug}/</div>
                <div id="serpDesc" style="color: #545454; font-size: 12.5px; line-height: 1.45;">{initial_excerpt}</div>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 14px;">
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">Focus Keyphrase</label>
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
                <label class="form-label">Custom SEO Title <span style="font-weight: 400; color: var(--ink-muted); font-size: 11.5px;">(Leave blank to use default title + site name)</span></label>
                <input type="text" id="seo_title" class="form-input" value="{initial_seo_title}" placeholder="{initial_title} – {site_title}" oninput="updateSerpPreview(); markDirty();" />
            </div>

            <div class="form-group" style="margin-bottom: 14px;">
                <label class="form-label">SEO Meta Description</label>
                <textarea id="seo_description" rows="2" class="form-textarea" placeholder="Compelling 120–158 character summary for search results..." oninput="updateSerpPreview(); markDirty();">{initial_seo_desc}</textarea>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 14px;">
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">Canonical URL Override</label>
                    <input type="text" id="seo_canonical" class="form-input" value="{initial_seo_canonical}" placeholder="https://example.com/canonical-slug/" oninput="markDirty()" />
                </div>
                <div class="form-group" style="margin-bottom: 0;">
                    <label class="form-label">OpenGraph / Social Image URL <span style="font-weight: 400; color: var(--ink-muted); font-size: 11px;">(Auto-OG card generated when blank)</span></label>
                    <input type="text" id="seo_og_image" class="form-input" value="{initial_seo_og_img}" placeholder="/og-image/auto.png (Auto-rendered 1200x630)" oninput="markDirty()" />
                </div>
            </div>

            <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 12px; border-top: 1px solid var(--rule);">
                <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; color: var(--ink); font-weight: 500;">
                    <input type="checkbox" id="seo_noindex" {seo_noindex_checked} onchange="updateSerpPreview(); markDirty();" />
                    <span>Discourage search engines from indexing this {type_singular} (<code>noindex, follow</code>)</span>
                </label>
                <div id="seoChecklistSummary" style="font-size: 12px; color: var(--ink-muted);"></div>
            </div>
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
            <div style="background: var(--paper-raised); border-radius: var(--radius-sm); width: 720px; max-width: 90vw; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);">
                <div style="padding: 16px 20px; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <h3 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;">Select Media Asset</h3>
                        <span class="badge badge-published" style="font-size: 11px;">Rusqlite Storage</span>
                    </div>
                    <button type="button" onclick="closeMediaPicker()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted);">&times;</button>
                </div>

                <div style="padding: 12px 20px; border-bottom: 1px solid var(--border-subtle); display: flex; gap: 10px; background: var(--paper-raised);">
                    <button type="button" class="btn btn-primary" id="tabLibraryBtn" onclick="switchMediaTab('library')">Media Library</button>
                    <button type="button" class="btn btn-secondary" id="tabUploadBtn" onclick="switchMediaTab('upload')">Upload New Image</button>
                </div>

                <div id="mediaTabLibrary" style="padding: 20px; flex: 1; overflow-y: auto;">
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 12px;">
                        {media_picker_items}
                    </div>
                </div>

                <div id="mediaTabUpload" style="padding: 40px; display: none; text-align: center;">
                    <div style="border: 2px dashed var(--rule-strong); border-radius: var(--radius-sm); padding: 40px; cursor: pointer;" onclick="document.getElementById('modalFileInput').click()">
                        <h4 style="font-size: 15px; font-weight: 600; margin: 0 0 6px 0;">Drop image here to upload</h4>
                        <p style="font-size: 12.5px; color: var(--ink-muted); margin: 0 0 16px 0;">Streamed directly into Daypress uploads directory</p>
                        <button type="button" class="btn btn-primary">Choose File</button>
                        <input type="file" id="modalFileInput" accept="image/*" style="display: none;" onchange="uploadModalImage(this.files[0])" />
                    </div>
                </div>

                <div style="padding: 12px 20px; border-top: 1px solid var(--border-subtle); background: var(--paper-raised); display: flex; justify-content: flex-end; gap: 8px;">
                    <button type="button" class="btn btn-secondary" onclick="closeMediaPicker()">Cancel</button>
                </div>
            </div>
        </div>"#,
        media_picker_items = media_picker_items
    )
}
