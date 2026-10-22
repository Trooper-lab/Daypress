use super::layout::render_wp_admin_shell;
use crate::models::{WpCategoryResponse, WpMediaResponse, WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

#[allow(clippy::too_many_arguments)]
pub fn render_wp_post_new_view(
    settings: &WpSettingsResponse,
    post_type: &str,
    existing: Option<&WpPostResponse>,
    all_pages: &[WpPostResponse],
    all_categories: &[WpCategoryResponse],
    all_tags: &[WpCategoryResponse],
    all_media: &[WpMediaResponse],
    has_visual_layout: bool,
) -> String {
    let is_page = post_type == "page";
    let type_singular = if is_page { "Page" } else { "Post" };

    let (
        initial_seo_title,
        initial_seo_desc,
        initial_seo_kw,
        initial_seo_canonical,
        initial_seo_og_img,
        initial_seo_noindex,
        initial_seo_schema,
    ) = if let Some(post) = existing {
        (
            escape_html(&post.seo_title),
            escape_html(&post.seo_description),
            escape_html(&post.seo_keyword),
            escape_html(&post.seo_canonical),
            escape_html(&post.seo_og_image),
            post.seo_noindex,
            if post.seo_schema_type.is_empty() {
                if is_page {
                    "WebPage".to_string()
                } else {
                    "Article".to_string()
                }
            } else {
                post.seo_schema_type.clone()
            },
        )
    } else {
        (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            false,
            if is_page {
                "WebPage".to_string()
            } else {
                "Article".to_string()
            },
        )
    };
    let seo_noindex_checked = if initial_seo_noindex { "checked" } else { "" };
    let schema_art_sel = if initial_seo_schema == "Article" { "selected" } else { "" };
    let schema_web_sel = if initial_seo_schema == "WebPage" { "selected" } else { "" };
    let schema_faq_sel = if initial_seo_schema == "FAQPage" { "selected" } else { "" };
    let schema_prod_sel = if initial_seo_schema == "Product" { "selected" } else { "" };
    let schema_loc_sel = if initial_seo_schema == "LocalBusiness" { "selected" } else { "" };
    let duplicate_btn_html = if existing.is_some() {
        format!(
            r#"<button type="button" class="btn btn-secondary" onclick="duplicateCurrentPost()" title="Deep-clone this {} (AST layout, metadata &amp; terms)">⧉ Duplicate</button>"#,
            type_singular
        )
    } else {
        String::new()
    };

    let (
        initial_id,
        initial_title,
        initial_slug,
        initial_content,
        initial_excerpt,
        initial_status,
        initial_parent,
        initial_menu_order,
        initial_featured_media,
        initial_categories,
        initial_tags,
        initial_date,
        initial_modified,
    ) = if let Some(post) = existing {
        (
            post.id.to_string(),
            escape_html(&post.title.rendered),
            escape_html(&post.slug),
            escape_html(&post.content.rendered),
            escape_html(&post.excerpt.rendered),
            post.status.clone(),
            post.parent,
            post.menu_order,
            post.featured_media,
            post.categories.clone(),
            post.tags.clone(),
            post.date.clone(),
            post.modified.clone(),
        )
    } else {
        (
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "publish".to_string(),
            0,
            0,
            0,
            vec![1],
            vec![],
            "".to_string(),
            "".to_string(),
        )
    };

    let title_label = if existing.is_some() {
        format!("Edit {}", type_singular)
    } else {
        format!("Add New {}", type_singular)
    };

    let btn_text = if existing.is_some() {
        format!("Update {}", type_singular)
    } else {
        format!("Publish {}", type_singular)
    };

    let is_pub_selected = if initial_status == "publish" {
        "selected"
    } else {
        ""
    };
    let is_draft_selected = if initial_status == "draft" {
        "selected"
    } else {
        ""
    };
    let is_pending_selected = if initial_status == "pending" {
        "selected"
    } else {
        ""
    };
    let is_private_selected = if initial_status == "private" {
        "selected"
    } else {
        ""
    };

    // Parent page options
    let mut parent_options = String::from(r#"<option value="0">(no parent - top level)</option>"#);
    for p in all_pages {
        if existing.map(|e| e.id) != Some(p.id) {
            let sel = if p.id == initial_parent {
                "selected"
            } else {
                ""
            };
            let t_esc = escape_html(&p.title.rendered);
            parent_options.push_str(&format!(
                r#"<option value="{}" {}>{}</option>"#,
                p.id, sel, t_esc
            ));
        }
    }

    // Categories checkboxes
    let mut cat_checkboxes = String::new();
    for c in all_categories {
        let checked = if initial_categories.contains(&c.id) {
            "checked"
        } else {
            ""
        };
        let c_name = escape_html(&c.name);
        cat_checkboxes.push_str(&format!(
            r#"<label style="display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer;">
                <input type="checkbox" name="post_category" value="{}" {} />
                <span>{}</span>
            </label>"#,
            c.id, checked, c_name
        ));
    }

    // Featured image preview
    let featured_img_preview = if initial_featured_media > 0 {
        if let Some(m) = all_media.iter().find(|m| m.id == initial_featured_media) {
            let url_esc = escape_html(&m.source_url);
            format!(
                r#"<div id="featuredImgContainer" style="position: relative; border-radius: 8px; overflow: hidden; height: 140px; background: #0f172a; margin-bottom: 10px;">
                    <img id="featuredImgElem" src="{}" style="width: 100%; height: 100%; object-fit: cover;" />
                </div>
                <div style="display: flex; gap: 8px;">
                    <button type="button" class="btn btn-secondary" style="flex: 1; font-size: 11.5px; padding: 4px;" onclick="openMediaPicker('featured')">Replace</button>
                    <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 4px; color: #ef4444;" onclick="removeFeaturedImage()">Remove</button>
                </div>"#,
                url_esc
            )
        } else {
            r#"<div id="featuredImgContainer" style="display: none; height: 140px; border-radius: 8px; overflow: hidden; background: #0f172a; margin-bottom: 10px;">
                <img id="featuredImgElem" src="" style="width: 100%; height: 100%; object-fit: cover;" />
            </div>
            <button type="button" class="btn btn-secondary" style="width: 100%; justify-content: center; font-size: 12px;" onclick="openMediaPicker('featured')">+ Set Featured Image</button>"#.to_string()
        }
    } else {
        r#"<div id="featuredImgContainer" style="display: none; height: 140px; border-radius: 8px; overflow: hidden; background: #0f172a; margin-bottom: 10px;">
            <img id="featuredImgElem" src="" style="width: 100%; height: 100%; object-fit: cover;" />
        </div>
        <button type="button" class="btn btn-secondary" style="width: 100%; justify-content: center; font-size: 12px;" onclick="openMediaPicker('featured')">+ Set Featured Image</button>"#.to_string()
    };

    // Media Picker items for modal
    let mut media_picker_items = String::new();
    for m in all_media {
        let url_esc = escape_html(&m.source_url);
        let title_esc = escape_html(&m.title.rendered);
        let is_img = m.mime_type.starts_with("image/");
        if is_img {
            media_picker_items.push_str(&format!(
                r#"<div class="media-pick-item" onclick="selectMediaItem({}, '{}', '{}')" style="cursor: pointer; border: 2px solid transparent; border-radius: 8px; overflow: hidden; height: 110px; background: #0f172a; position: relative; transition: all 0.15s ease;" onmouseover="this.style.borderColor='#2563eb'" onmouseout="this.style.borderColor='transparent'">
                    <img src="{}" alt="{}" style="width: 100%; height: 100%; object-fit: cover;" loading="lazy" />
                    <div style="position: absolute; bottom: 0; left: 0; right: 0; background: rgba(15,23,42,0.85); color: #ffffff; font-size: 10px; padding: 2px 6px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{}</div>
                </div>"#,
                m.id, url_esc, title_esc, url_esc, title_esc, title_esc
            ));
        }
    }

    // Tag pills
    let mut popular_tag_chips = String::new();
    for t in all_tags.iter().take(6) {
        let t_name = escape_html(&t.name);
        popular_tag_chips.push_str(&format!(
            r#"<span class="tag-chip" onclick="addTagChip('{}')" style="background: #f1f5f9; color: #475569; font-size: 11px; padding: 2px 8px; border-radius: 4px; cursor: pointer; border: 1px solid #cbd5e1;">+ {}</span>"#,
            t_name, t_name
        ));
    }

    let initial_tag_names: Vec<String> = all_tags
        .iter()
        .filter(|t| initial_tags.contains(&t.id))
        .map(|t| t.name.clone())
        .collect();
    let initial_tag_str = initial_tag_names.join(", ");

    let mut known_tags_entries = Vec::new();
    for t in all_tags {
        known_tags_entries.push(format!(
            r#""{}": {}"#,
            escape_html(&t.name.to_lowercase()),
            t.id
        ));
    }
    let known_tags_json = format!("{{{}}}", known_tags_entries.join(", "));
    let initial_template = existing
        .map(|p| p.template.as_str())
        .unwrap_or("default");
    let tpl_default_sel = if initial_template.is_empty() || initial_template == "default" {
        "selected"
    } else {
        ""
    };
    let tpl_full_sel = if initial_template == "full-width" {
        "selected"
    } else {
        ""
    };
    let tpl_canvas_sel = if initial_template == "canvas" {
        "selected"
    } else {
        ""
    };
    let tpl_landing_sel = if initial_template == "landing" {
        "selected"
    } else {
        ""
    };
    let tpl_editorial_sel = if initial_template == "editorial" {
        "selected"
    } else {
        ""
    };

    let site_title = &settings.title;

    let top_builder_btn = if has_visual_layout {
        format!(
            r#"<a href="/n-admin/builder?post={}" class="btn" style="background: linear-gradient(135deg, #e02660 0%, #f43f5e 100%); color: #ffffff; border: none; font-weight: 700; display: inline-flex; align-items: center; gap: 8px; padding: 7px 18px; border-radius: 6px; box-shadow: 0 2px 10px rgba(224, 38, 96, 0.4); text-decoration: none; transition: all 0.2s ease;">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
                <span>⚡ Edit with Visual Builder</span>
            </a>"#,
            initial_id
        )
    } else {
        r#"<button type="button" class="btn" id="btnEditVisualBuilder" onclick="switchToVisualBuilder()" style="background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%); color: #ffffff; border: none; font-weight: 600; display: inline-flex; align-items: center; gap: 8px; padding: 7px 16px; border-radius: 6px; box-shadow: 0 2px 8px rgba(99, 102, 241, 0.35); cursor: pointer; transition: all 0.2s ease;">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
            <span>⚡ Edit with Visual Builder</span>
        </button>"#.to_string()
    };

    let locked_banner_html = if has_visual_layout {
        r###"<div id="visualBuilderLockedNotice" style="margin-top: 18px; background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%); border: 1px solid #334155; border-radius: 12px; padding: 48px 32px; text-align: center; color: #ffffff; box-shadow: 0 4px 24px rgba(0,0,0,0.12);">
            <div style="width: 58px; height: 58px; margin: 0 auto 16px auto; border-radius: 14px; background: linear-gradient(135deg, #e02660 0%, #f43f5e 100%); display: flex; align-items: center; justify-content: center; box-shadow: 0 4px 14px rgba(224, 38, 96, 0.45);">
                <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
            </div>
            <div style="display: inline-flex; align-items: center; gap: 6px; background: rgba(224, 38, 96, 0.2); color: #fda4af; border: 1px solid rgba(224, 38, 96, 0.4); padding: 3px 12px; border-radius: 9999px; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 12px;">
                <span>🔒 Text Editor Locked &bull; Visual Builder Active</span>
            </div>
            <h2 style="font-size: 22px; font-weight: 700; color: #ffffff; margin-bottom: 10px;">This __TYPE_SINGULAR__ is Managed by Neopress Visual Builder</h2>
            <p style="font-size: 14px; color: #94a3b8; max-width: 520px; margin: 0 auto 28px auto; line-height: 1.6;">
                The standard content editor is locked to protect your drag-and-drop layout structure and zero-JS semantic compilation. Update title and metadata here, or launch the visual editor below.
            </p>
            <div style="display: flex; gap: 14px; justify-content: center; align-items: center; flex-wrap: wrap;">
                <a href="/n-admin/builder?post=__INITIAL_ID__" class="btn" style="background: linear-gradient(135deg, #e02660 0%, #f43f5e 100%); color: #ffffff; font-weight: 700; padding: 12px 28px; border-radius: 8px; font-size: 14px; display: inline-flex; align-items: center; gap: 8px; text-decoration: none; box-shadow: 0 4px 14px rgba(224, 38, 96, 0.4);">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
                    <span>Launch Visual Drag &amp; Drop Studio</span>
                </a>
                <button type="button" class="btn btn-secondary" onclick="confirmSwitchToStandard()" style="background: rgba(255,255,255,0.08); color: #cbd5e1; border: 1px solid rgba(255,255,255,0.18); font-weight: 600; padding: 11px 20px; border-radius: 8px; font-size: 13px; cursor: pointer;">
                    Discard Visual Layout &amp; Unlock Standard Editor
                </button>
            </div>
        </div>"###
            .replace("__TYPE_SINGULAR__", type_singular)
            .replace("__INITIAL_ID__", &initial_id)
    } else {
        String::new()
    };

    let standard_editor_display = if has_visual_layout { "display: none;" } else { "display: block;" };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h1 class="page-title">{title_label}</h1>
                    <span class="badge badge-published" style="font-size: 11px; text-transform: uppercase;">{post_type}</span>
                </div>
                <p class="page-subtitle">Craft and publish high-performance editorial content directly to embedded Rusqlite</p>
            </div>
            <div style="display: flex; gap: 10px; align-items: center;">
{top_builder_btn}
                {duplicate_btn_html}
                <button type="button" class="btn btn-secondary" onclick="saveDraft()">Save Draft</button>
                <button type="button" class="btn btn-secondary" onclick="previewPage()">View Live ↗</button>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('postFormSubmit').click()">{btn_text}</button>
            </div>
        </div>

        <form id="postForm" onsubmit="savePost(event)">
            <button type="submit" id="postFormSubmit" style="display: none;"></button>
            <input type="hidden" id="featured_media" value="{initial_featured_media}" />
            
            <div style="display: grid; grid-template-columns: 3fr 1fr; gap: 24px;">
                
                <!-- Left: Title, Canvas & Excerpt -->
                <div style="display: flex; flex-direction: column; gap: 20px;">
                    
                    <!-- Main Editor Card -->
                    <div class="neo-card" style="padding: 24px;">
                        <!-- Title Field -->
                        <input type="text" name="post_title" id="title" placeholder="Add {type_singular} title..." value="{initial_title}"
                               style="width: 100%; font-size: 28px; font-weight: 700; border: none; outline: none; padding: 4px 0; color: #0f172a; letter-spacing: -0.02em;" required />
                        
                        <!-- Permalink & Live Slug Preview -->
                        <div style="display: flex; align-items: center; gap: 6px; padding: 10px 0 16px 0; border-bottom: 1px solid #f1f5f9; margin-bottom: 14px; font-size: 12.5px; color: #64748b;">
                            <span>Permalink:</span>
                            <span style="font-family: var(--font-mono); color: #0f172a;">http://localhost:8090/</span>
                            <input type="text" id="post_name" placeholder="slug" value="{initial_slug}" 
                                   style="height: 24px; padding: 0 6px; font-size: 12px; font-family: var(--font-mono); border: 1px solid #cbd5e1; border-radius: 4px; width: 180px;" />
                            <span>/</span>
                            <button type="button" class="btn btn-secondary" style="padding: 2px 6px; font-size: 11px; height: 22px;" onclick="copyPermalink()">Copy Link</button>
                            <a id="viewPageLink" href="/{initial_slug}/" target="_blank" style="color: #2563eb; text-decoration: none; font-size: 11.5px; margin-left: 4px;">Visit Page ↗</a>
                        </div>

                        <!-- Autosave Recovery Banner -->
                        <div id="draftRecoveryBanner" style="display: none; background: #fffbeb; border: 1px solid #fde68a; border-radius: 8px; padding: 10px 16px; margin-bottom: 14px; font-size: 12.5px; color: #92400e; align-items: center; justify-content: space-between;">
                            <span><strong>Autosave Recovery:</strong> Found an unsaved local draft from <span id="recoveryTime"></span>.</span>
                            <div style="display: flex; gap: 8px;">
                                <button type="button" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px;" onclick="restoreLocalDraft()">Restore Draft</button>
                                <button type="button" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: #ef4444;" onclick="dismissLocalDraft()">Dismiss</button>
                            </div>
                        </div>

                        {locked_banner_html}

                        <div id="standardEditorContainer" style="{standard_editor_display}">
                        <!-- Mode Switcher & Real-time Metrics & Autosave Badge -->
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; padding-bottom: 12px; border-bottom: 1px solid #f1f5f9;">
                            <div style="display: inline-flex; background: #f1f5f9; padding: 3px; border-radius: 8px; border: 1px solid #e2e8f0; gap: 2px;">
                                <button type="button" id="btnModeVisual" onmousedown="event.preventDefault()" onclick="switchEditorMode('visual')" style="display: flex; align-items: center; gap: 6px; padding: 6px 14px; font-size: 12px; font-weight: 600; border-radius: 6px; border: none; cursor: pointer; background: #ffffff; color: #0f172a; box-shadow: 0 1px 3px rgba(0,0,0,0.1); transition: all 0.15s ease;">
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>
                                    <span>Visual Editor (Live)</span>
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
                                
                                <!-- Interactive Autosave Status Badge -->
                                <div id="autosaveBadge" style="display: inline-flex; align-items: center; gap: 6px; background: #ecfdf5; color: #059669; border: 1px solid #a7f3d0; border-radius: 9999px; padding: 3px 10px; font-size: 11px; font-weight: 600; transition: all 0.2s ease;">
                                    <span id="autosaveDot" style="width: 6px; height: 6px; border-radius: 50%; background: #10b981;"></span>
                                    <span id="autosaveLabel">Synced with SQLite</span>
                                </div>
                            </div>
                        </div>

                        <!-- 2026 Comprehensive Formatting Toolbar -->
                        <div id="editorToolbar" style="display: flex; flex-wrap: wrap; align-items: center; gap: 5px; padding-bottom: 12px; border-bottom: 1px solid #f1f5f9; margin-bottom: 14px;">
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

                        <!-- Editor Canvas: Visual Live WYSIWYG (Default) vs HTML Textarea vs Visual Drag & Drop Builder -->
                        <div id="editorCanvasWrapper" style="position: relative;">
                            <div id="visualEditor" contenteditable="true" spellcheck="true" class="neo-visual-canvas"
                                 placeholder="Start typing your story, or insert media and headings..."
                                 oninput="onVisualInput()" onblur="syncVisualToTextarea()"></div>
                            <textarea id="content" name="content" class="neo-html-textarea" style="display: none;"
                                      placeholder="<!-- Write or edit raw HTML source here -->"
                                      oninput="onHtmlInput()">{initial_content}</textarea>

                        </div>
                        </div> <!-- end #standardEditorContainer -->
                    </div>

                    <!-- Neopress SEO & Readability Studio Card (Yoast / RankMath Parity) -->
                    <div class="neo-card" style="padding: 22px;">
                        <div class="neo-card-header" style="padding: 0 0 14px 0; margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center;">
                            <div style="display: flex; align-items: center; gap: 10px;">
                                <h2 class="neo-card-title">Neopress SEO &amp; Readability Studio</h2>
                                <span class="badge badge-published" style="font-size: 11px;">Yoast / RankMath Parity</span>
                            </div>
                            <div id="seoScoreBadge" style="background: #ecfdf5; color: #059669; border: 1px solid #a7f3d0; padding: 3px 10px; border-radius: 999px; font-size: 12px; font-weight: 700;">
                                SEO Score: 85 / 100
                            </div>
                        </div>

                        <!-- Live Google SERP Preview -->
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
                    </div>

                </div>

                <!-- Right: Meta Drawer -->
                <div style="display: flex; flex-direction: column; gap: 20px;">
                    
                    <!-- 1. Publish Card -->
                    <div class="neo-card">
                        <div class="neo-card-header">
                            <h2 class="neo-card-title">Status &amp; Visibility</h2>
                        </div>
                        <div class="neo-card-body">
                            <div class="form-group">
                                <label class="form-label">Status</label>
                                <select id="post_status" class="form-select">
                                    <option value="publish" {is_pub_selected}>Published</option>
                                    <option value="draft" {is_draft_selected}>Draft</option>
                                    <option value="pending" {is_pending_selected}>Pending Review</option>
                                    <option value="private" {is_private_selected}>Private</option>
                                </select>
                            </div>

                            <div class="form-group">
                                <label class="form-label">Author</label>
                                <div style="display: flex; align-items: center; gap: 8px; font-size: 13px;">
                                    <div class="user-avatar" style="width: 22px; height: 22px; font-size: 10px;">B</div>
                                    <span>admin (Administrator)</span>
                                </div>
                            </div>

                            <div style="font-size: 12px; color: #64748b; margin-top: 10px; line-height: 1.5; border-top: 1px solid #f1f5f9; padding-top: 10px;">
                                <div>Created: <span style="font-family: var(--font-mono);">{initial_date}</span></div>
                                <div>Modified: <span style="font-family: var(--font-mono);">{initial_modified}</span></div>
                            </div>

                            <div style="border-top: 1px solid #f1f5f9; padding-top: 14px; margin-top: 14px; display: flex; justify-content: space-between; align-items: center;">
                                <button type="button" onclick="trashPost()" style="color: #ef4444; font-size: 12px; background: none; border: none; cursor: pointer; padding: 4px;">Move to Trash</button>
                                <button type="submit" class="btn btn-primary">{btn_text}</button>
                            </div>
                        </div>
                    </div>

                    <!-- 2. Page Attributes Card (when post_type == page) -->
                    <div class="neo-card" style="{page_attr_style}">
                        <div class="neo-card-header">
                            <h2 class="neo-card-title">Page Attributes</h2>
                        </div>
                        <div class="neo-card-body">
                            <div class="form-group">
                                <label class="form-label">Parent Page</label>
                                <select id="post_parent" class="form-select">
                                    {parent_options}
                                </select>
                                <small style="color: #94a3b8; font-size: 11.5px; margin-top: 4px; display: block;">Organize pages into hierarchical sub-pages.</small>
                            </div>

                            <div class="form-group">
                                <label class="form-label">Template</label>
                                <select id="page_template" class="form-select">
                                    <option value="default" {tpl_default_sel}>Default Template</option>
                                    <option value="full-width" {tpl_full_sel}>Full-Width Clean Canvas</option>
                                    <option value="canvas" {tpl_canvas_sel}>Elementor Canvas (No Header/Footer)</option>
                                    <option value="landing" {tpl_landing_sel}>Landing Page Hero</option>
                                    <option value="editorial" {tpl_editorial_sel}>Editorial Longform</option>
                                </select>
                            </div>

                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label">Order</label>
                                <input type="number" id="menu_order" class="form-input" value="{initial_menu_order}" style="width: 100px;" />
                                <small style="color: #94a3b8; font-size: 11.5px; margin-top: 4px; display: block;">Numerical ordering for navigation menus.</small>
                            </div>
                        </div>
                    </div>

                    <!-- 3. Taxonomy Card (when post_type == post) -->
                    <div class="neo-card" style="{post_tax_style}">
                        <div class="neo-card-header" style="display: flex; justify-content: space-between; align-items: center;">
                            <h2 class="neo-card-title">Categories</h2>
                            <a href="/n-admin/edit-tags.php?taxonomy=category" target="_blank" style="font-size: 11.5px; color: #2563eb; text-decoration: none;">Manage</a>
                        </div>
                        <div class="neo-card-body">
                            <div id="categoryList" style="max-height: 160px; overflow-y: auto; padding-right: 4px; margin-bottom: 12px;">
                                {cat_checkboxes}
                            </div>
                            
                            <!-- Inline Add Category -->
                            <div style="border-top: 1px solid #f1f5f9; padding-top: 10px;">
                                <div style="display: flex; gap: 6px;">
                                    <input type="text" id="newCatName" placeholder="New category..." class="form-input" style="height: 28px; font-size: 12px;" />
                                    <button type="button" class="btn btn-secondary" style="height: 28px; font-size: 11.5px; padding: 0 8px;" onclick="inlineAddCategory()">Add</button>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Tags Card (when post_type == post) -->
                    <div class="neo-card" style="{post_tax_style}">
                        <div class="neo-card-header">
                            <h2 class="neo-card-title">Tags</h2>
                        </div>
                        <div class="neo-card-body">
                            <input type="text" id="post_tags" class="form-input" value="{initial_tag_str}" placeholder="rust, axum, sqlite..." style="margin-bottom: 8px;" />
                            <small style="color: #94a3b8; font-size: 11.5px; margin-bottom: 10px; display: block;">Separate tags with commas.</small>
                            
                            <div style="font-size: 11px; color: #64748b; margin-bottom: 6px;">Popular Tags:</div>
                            <div style="display: flex; flex-wrap: wrap; gap: 6px;">
                                {popular_tag_chips}
                            </div>
                        </div>
                    </div>

                    <!-- 4. Featured Image Card -->
                    <div class="neo-card">
                        <div class="neo-card-header">
                            <h2 class="neo-card-title">Featured Image</h2>
                        </div>
                        <div class="neo-card-body">
                            {featured_img_preview}
                        </div>
                    </div>

                </div>
            </div>
        </form>

        <!-- Media Library Picker Modal -->
        <div id="mediaPickerModal" style="display: none; position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(15,23,42,0.65); backdrop-filter: blur(4px); z-index: 10000; align-items: center; justify-content: center;">
            <div style="background: #ffffff; border-radius: 12px; width: 720px; max-width: 90vw; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);">
                <!-- Modal Header -->
                <div style="padding: 16px 20px; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <h3 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Select Media Asset</h3>
                        <span class="badge badge-published" style="font-size: 11px;">Rusqlite Storage</span>
                    </div>
                    <button type="button" onclick="closeMediaPicker()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: #94a3b8;">&times;</button>
                </div>

                <!-- Modal Body: Tabs -->
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

                <!-- Modal Footer -->
                <div style="padding: 12px 20px; border-top: 1px solid var(--border-subtle); background: #f8fafc; display: flex; justify-content: flex-end; gap: 8px;">
                    <button type="button" class="btn btn-secondary" onclick="closeMediaPicker()">Cancel</button>
                </div>
            </div>
        </div>

        <script>
            let currentMode = 'visual'; // 'visual' (default) or 'html'
            let currentMediaTarget = 'content'; // 'content' or 'featured'
            const siteTitle = '{site_title}';
            let activePostId = '{initial_id}' || '';
            let isDirty = false;
            let isAutosaving = false;
            let lastAutosaveTime = null;

            const titleInput = document.getElementById('title');
            const slugInput = document.getElementById('post_name');
            const contentArea = document.getElementById('content');
            const visualEditor = document.getElementById('visualEditor');

            // Auto slug generator from title
            titleInput.addEventListener('input', () => {{
                if (!slugInput.value || slugInput.dataset.auto !== 'false') {{
                    slugInput.value = titleInput.value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
                    updateSerpPreview();
                }}
                markDirty();
            }});
            slugInput.addEventListener('input', () => {{
                slugInput.dataset.auto = 'false';
                updateSerpPreview();
                markDirty();
            }});

            // Initialize editor on DOM load: Live Visual Editor is active by default
            document.addEventListener('DOMContentLoaded', () => {{
                const raw = contentArea.value || '';
                visualEditor.innerHTML = cleanWpContent(raw);
                if (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '' || visualEditor.innerHTML === '<p></p>') {{
                    visualEditor.innerHTML = '<p><br></p>';
                }}

                updateWordCount();
                updateSerpPreview();
                checkLocalDraftRecovery();
            }});

            // Clicking empty visual canvas focuses into first paragraph
            visualEditor.addEventListener('click', (e) => {{
                if (e.target === visualEditor && (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '')) {{
                    visualEditor.innerHTML = '<p><br></p>';
                }}
            }});

            function cleanWpContent(raw) {{
                if (!raw) return '<p></p>';
                // Strip WordPress Gutenberg block comments safely
                let cleaned = raw.replace(/<!--[\\s\\S]*?-->/g, '').trim();
                if (!cleaned) return '<p></p>';
                // If pure plain text without tags, wrap into clean paragraphs
                if (!cleaned.includes('<p>') && !cleaned.includes('<div>') && !cleaned.includes('<h') && !cleaned.includes('<table')) {{
                    cleaned = cleaned.split('\\n\\n').map(p => '<p>' + p.trim().replace(/\\n/g, '<br>') + '</p>').join('');
                }}
                return cleaned;
            }}

            function syncVisualToTextarea() {{
                contentArea.value = visualEditor.innerHTML;
            }}

            function onVisualInput() {{
                syncVisualToTextarea();
                updateWordCount();
                updateSerpPreview();
                markDirty();
            }}

            function onHtmlInput() {{
                updateWordCount();
                updateSerpPreview();
                markDirty();
            }}

            // Comprehensive Keyboard Shortcuts: Ctrl+S (Save), Ctrl+B, Ctrl+I, Ctrl+U, Ctrl+K (Link), Ctrl+Shift+H / Alt+Shift+H (Toggle View)
            document.addEventListener('keydown', (e) => {{
                const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
                const modifier = isMac ? e.metaKey : e.ctrlKey;

                if (modifier && (e.key === 's' || e.key === 'S')) {{
                    e.preventDefault();
                    triggerAutosave(true); // explicit save with toast
                    return;
                }}

                if (modifier && (e.key === 'k' || e.key === 'K')) {{
                    e.preventDefault();
                    insertLinkPrompt();
                    return;
                }}

                if (modifier && (e.key === 'b' || e.key === 'B')) {{
                    e.preventDefault();
                    execStyle('bold');
                    return;
                }}

                if (modifier && (e.key === 'i' || e.key === 'I')) {{
                    e.preventDefault();
                    execStyle('italic');
                    return;
                }}

                if (modifier && (e.key === 'u' || e.key === 'U')) {{
                    e.preventDefault();
                    execStyle('underline');
                    return;
                }}

                if (modifier && (e.key === 'h' || e.key === 'H') && (e.shiftKey || e.altKey)) {{
                    e.preventDefault();
                    switchEditorMode(currentMode === 'visual' ? 'html' : 'visual');
                    return;
                }}
            }});

            // Tab key support inside HTML textarea
            contentArea.addEventListener('keydown', (e) => {{
                if (e.key === 'Tab') {{
                    e.preventDefault();
                    const start = contentArea.selectionStart;
                    const end = contentArea.selectionEnd;
                    contentArea.value = contentArea.value.substring(0, start) + '  ' + contentArea.value.substring(end);
                    contentArea.selectionStart = contentArea.selectionEnd = start + 2;
                    markDirty();
                }}
            }});

            const isVisualLocked = {has_visual_layout};

            function switchToVisualBuilder() {{
                if (activePostId && activePostId.length > 0) {{
                    window.location.href = '/n-admin/builder?post=' + activePostId;
                }} else {{
                    alert('Please save this ' + '{type_singular}' + ' first so we can initialize your visual layout.');
                }}
            }}

            async function confirmSwitchToStandard() {{
                if (!confirm("Are you sure you want to discard the visual layout?\n\nThis will unlock the standard text editor. Your compiled HTML text will be retained as base content.")) {{
                    return;
                }}
                if (!activePostId) return;
                try {{
                    const res = await fetch('/wp-json/neopress/v1/layout/' + activePostId, {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        showToast('✓ Visual layout discarded. Unlocking standard text editor...');
                        setTimeout(() => {{
                            window.location.reload();
                        }}, 400);
                    }} else {{
                        showToast('Error removing layout', true);
                    }}
                }} catch(e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            function switchEditorMode(mode) {{
                if (mode === currentMode) return;
                currentMode = mode;

                const btnVisual = document.getElementById('btnModeVisual');
                const btnHtml = document.getElementById('btnModeHtml');
                const btnBuilder = document.getElementById('btnModeBuilder');
                const toolbar = document.getElementById('editorToolbar');
                const inlineBuilder = document.getElementById('inlineBuilderContainer');

                // Reset all tabs
                [btnVisual, btnHtml, btnBuilder].forEach(b => {{
                    if (b) {{
                        b.style.background = 'transparent';
                        b.style.color = '#64748b';
                        b.style.boxShadow = 'none';
                    }}
                }});

                if (mode === 'visual') {{
                    visualEditor.innerHTML = cleanWpContent(contentArea.value);
                    if (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '') {{
                        visualEditor.innerHTML = '<p><br></p>';
                    }}
                    visualEditor.style.display = 'block';
                    contentArea.style.display = 'none';
                    if (inlineBuilder) inlineBuilder.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'flex';

                    btnVisual.style.background = '#ffffff';
                    btnVisual.style.color = '#0f172a';
                    btnVisual.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';
                    showToast('Switched to Visual Live Editor');
                }} else if (mode === 'html') {{
                    syncVisualToTextarea();
                    visualEditor.style.display = 'none';
                    contentArea.style.display = 'block';
                    if (inlineBuilder) inlineBuilder.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'flex';

                    btnHtml.style.background = '#ffffff';
                    btnHtml.style.color = '#0f172a';
                    btnHtml.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';
                    showToast('Switched to HTML Source View');
                }} else if (mode === 'builder') {{
                    visualEditor.style.display = 'none';
                    contentArea.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'none';
                    if (inlineBuilder) inlineBuilder.style.display = 'block';

                    btnBuilder.style.background = '#ffffff';
                    btnBuilder.style.color = '#4f46e5';
                    btnBuilder.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';

                    if (!inlineLayoutState) {{
                        initInlineLayout();
                    }}
                    renderInlineCanvas();
                    showToast('⚡ Neopress Visual Drag-and-Drop Builder Active');
                }}
                updateWordCount();
            }}

            function initInlineLayout() {{
                inlineLayoutState = {{
                    version: 1,
                    root: {{
                        id: 'sec_root',
                        type: 'Section',
                        styles: {{ padding_top: '30px', padding_bottom: '30px', background_color: '#f8fafc' }},
                        children: [
                            {{
                                id: 'node_h1',
                                type: 'Heading',
                                data: {{ text: titleInput.value || 'Crafted with Neopress Visual Builder', level: 1 }},
                                styles: {{ font_size: '34px', text_color: '#0f172a', margin_bottom: '14px', text_align: 'center' }}
                            }},
                            {{
                                id: 'node_p1',
                                type: 'Text',
                                data: {{ content_html: '<p>Drag, drop, and edit semantic components rendered directly in native Rust. Sub-millisecond performance with zero runtime JS.</p>' }},
                                styles: {{ font_size: '16px', text_color: '#64748b', margin_bottom: '24px', text_align: 'center' }}
                            }},
                            {{
                                id: 'node_btn1',
                                type: 'Button',
                                data: {{ text: 'Get Started Now ⚡', url: '#' }},
                                styles: {{ background_color: '#4f46e5', text_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '24px', padding_right: '24px', border_radius: '6px', text_align: 'center' }}
                            }}
                        ]
                    }}
                }};
            }}

            function renderInlineCanvas() {{
                const root = document.getElementById('inlineCanvasRoot');
                if (!root || !inlineLayoutState) return;

                function renderNode(node) {{
                    const s = node.styles || {{}};
                    const styleStr = Object.entries(s).map(([k, v]) => `${{k.replace(/_/g, '-')}}: ${{v}}`).join('; ');
                    const type = node.type || 'Container';

                    const actionBtns = `
                        <div style="position: absolute; top: 4px; right: 4px; display: none; gap: 4px; background: #1e293b; padding: 2px 6px; border-radius: 4px; font-size: 10px;" class="inline-node-actions">
                            <span style="color: #a5b4fc; font-weight: 700;">${{type}}</span>
                            <button type="button" onclick="event.stopPropagation(); deleteInlineNode('${{node.id}}')" style="color: #ef4444; background: none; border: none; cursor: pointer;">✕</button>
                        </div>
                    `;

                    let inner = '';
                    if (node.children && node.children.length > 0) {{
                        inner = node.children.map(renderNode).join('');
                    }}

                    if (type === 'Heading') {{
                        const text = (node.data && node.data.text) || 'Heading';
                        const lvl = (node.data && node.data.level) || 2;
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 6px; margin: 4px 0; border: 1px dashed transparent; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <h${{lvl}} contenteditable="true" onblur="inlineUpdateText('${{node.id}}', this.innerText)" style="margin: 0; outline: none;">${{text}}</h${{lvl}}>
                        </div>`;
                    }}

                    if (type === 'Text') {{
                        const html = (node.data && node.data.content_html) || '<p>Paragraph text</p>';
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 6px; margin: 4px 0; border: 1px dashed transparent; line-height: 1.6; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <div contenteditable="true" onblur="inlineUpdateHtml('${{node.id}}', this.innerHTML)" style="outline: none;">${{html}}</div>
                        </div>`;
                    }}

                    if (type === 'Button') {{
                        const btnText = (node.data && node.data.text) || 'Button';
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 6px; margin: 4px 0; border: 1px dashed transparent; text-align: ${{s.text_align || 'left'}};" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <span contenteditable="true" onblur="inlineUpdateText('${{node.id}}', this.innerText)" style="display: inline-block; cursor: pointer; outline: none; ${{styleStr}}">${{btnText}}</span>
                        </div>`;
                    }}

                    if (type === 'Form') {{
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 16px; margin: 8px 0; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <div style="font-weight: 700; color: #0f172a; margin-bottom: 6px;">⚡ Declarative Honeypot Form</div>
                            <div style="display: flex; gap: 8px; margin-bottom: 8px;">
                                <input type="text" placeholder="Name" style="flex: 1; padding: 6px 10px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px;" disabled />
                                <input type="email" placeholder="Email" style="flex: 1; padding: 6px 10px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px;" disabled />
                            </div>
                        </div>`;
                    }}

                    if (type === 'Counter') {{
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; text-align: center; padding: 14px; margin: 6px 0; border: 1px dashed transparent; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <div style="font-size: 32px; font-weight: 800; color: #4f46e5;">99.9%</div>
                            <div style="font-size: 12px; color: #64748b;">Sub-millisecond SLA</div>
                        </div>`;
                    }}

                    return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 10px; margin: 6px 0; border: 1px dashed #e2e8f0; border-radius: 6px; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                        ${{actionBtns}}
                        ${{inner}}
                    </div>`;
                }}

                root.innerHTML = renderNode(inlineLayoutState.root);
                syncInlineToHtml();
            }}

            function syncInlineToHtml() {{
                const root = document.getElementById('inlineCanvasRoot');
                if (root) {{
                    // Clone and clean action bars
                    const clone = root.cloneNode(true);
                    clone.querySelectorAll('.inline-node-actions').forEach(el => el.remove());
                    contentArea.value = clone.innerHTML;
                    markDirty();
                }}
            }}

            function inlineUpdateText(id, text) {{
                function update(n) {{
                    if (n.id === id) {{
                        if (!n.data) n.data = {{}};
                        n.data.text = text;
                        return true;
                    }}
                    if (n.children) for (const c of n.children) if (update(c)) return true;
                    return false;
                }}
                if (inlineLayoutState) {{
                    update(inlineLayoutState.root);
                    syncInlineToHtml();
                }}
            }}

            function inlineUpdateHtml(id, html) {{
                function update(n) {{
                    if (n.id === id) {{
                        if (!n.data) n.data = {{}};
                        n.data.content_html = html;
                        return true;
                    }}
                    if (n.children) for (const c of n.children) if (update(c)) return true;
                    return false;
                }}
                if (inlineLayoutState) {{
                    update(inlineLayoutState.root);
                    syncInlineToHtml();
                }}
            }}

            function addInlineBlock(type) {{
                if (!inlineLayoutState) initInlineLayout();
                const uid = 'inline_' + Math.random().toString(36).substr(2, 6);
                let node = {{ id: uid, type, styles: {{}}, children: [] }};

                if (type === 'Section') {{
                    node.styles = {{ padding_top: '24px', padding_bottom: '24px', background_color: '#f1f5f9' }};
                    node.children = [
                        {{ id: uid + '_h', type: 'Heading', data: {{ text: 'New Feature Spotlight', level: 2 }}, styles: {{ font_size: '24px', text_color: '#0f172a' }} }},
                        {{ id: uid + '_t', type: 'Text', data: {{ content_html: '<p>Built with native Rust ergonomics and zero plugin bloat.</p>' }}, styles: {{ font_size: '14px', text_color: '#64748b' }} }}
                    ];
                }} else if (type === 'Heading') {{
                    node.data = {{ text: 'Section Heading', level: 2 }};
                    node.styles = {{ font_size: '26px', text_color: '#0f172a', margin_bottom: '8px' }};
                }} else if (type === 'Text') {{
                    node.data = {{ content_html: '<p>Add rich editorial prose, formatted quotes, and bullet points.</p>' }};
                    node.styles = {{ font_size: '15px', text_color: '#475569' }};
                }} else if (type === 'Button') {{
                    node.data = {{ text: 'Explore Documentation ↗', url: '#' }};
                    node.styles = {{ background_color: '#4f46e5', text_color: '#ffffff', padding_top: '8px', padding_bottom: '8px', padding_left: '18px', padding_right: '18px', border_radius: '6px' }};
                }} else if (type === 'Grid2') {{
                    node.styles = {{ display: 'grid', grid_template_columns: '1fr 1fr', gap: '16px', margin: '12px 0' }};
                    node.children = [
                        {{ id: uid + '_col1', type: 'Text', data: {{ content_html: '<strong>Column One:</strong> High-performance SQLite engine.' }}, styles: {{ background_color: '#ffffff', padding_top: '12px', padding_bottom: '12px', padding_left: '12px', padding_right: '12px', border_radius: '6px' }} }},
                        {{ id: uid + '_col2', type: 'Text', data: {{ content_html: '<strong>Column Two:</strong> Streaming WXR XML parser.' }}, styles: {{ background_color: '#ffffff', padding_top: '12px', padding_bottom: '12px', padding_left: '12px', padding_right: '12px', border_radius: '6px' }} }}
                    ];
                }} else if (type === 'Grid3') {{
                    node.styles = {{ display: 'grid', grid_template_columns: '1fr 1fr 1fr', gap: '14px', margin: '12px 0' }};
                    node.children = [
                        {{ id: uid + '_c1', type: 'Text', data: {{ content_html: '<strong>Fast:</strong> &lt;1ms TTFB' }}, styles: {{ background_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '10px', padding_right: '10px', border_radius: '6px' }} }},
                        {{ id: uid + '_c2', type: 'Text', data: {{ content_html: '<strong>Safe:</strong> Zero CVEs' }}, styles: {{ background_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '10px', padding_right: '10px', border_radius: '6px' }} }},
                        {{ id: uid + '_c3', type: 'Text', data: {{ content_html: '<strong>Portable:</strong> Single binary' }}, styles: {{ background_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '10px', padding_right: '10px', border_radius: '6px' }} }}
                    ];
                }}

                inlineLayoutState.root.children.push(node);
                renderInlineCanvas();
                showToast(`Added ${{type}} block`);
            }}

            function deleteInlineNode(id) {{
                function remove(parent) {{
                    if (!parent.children) return false;
                    const idx = parent.children.findIndex(c => c.id === id);
                    if (idx !== -1) {{
                        parent.children.splice(idx, 1);
                        return true;
                    }}
                    for (const c of parent.children) if (remove(c)) return true;
                    return false;
                }}
                if (inlineLayoutState) {{
                    remove(inlineLayoutState.root);
                    renderInlineCanvas();
                    showToast('Block removed');
                }}
            }}

            function openSchemaModal() {{
                if (!inlineLayoutState) initInlineLayout();
                alert('⚡ Neopress AST Schema JSON:\n\n' + JSON.stringify(inlineLayoutState, null, 2));
            }}

            function updateWordCount() {{
                const text = currentMode === 'visual' ? (visualEditor.innerText || '') : (contentArea.value || '');
                const clean = text.replace(/<[^>]*>/g, ' ').trim();
                const words = clean ? clean.split(/\\s+/).filter(Boolean).length : 0;
                const readTime = Math.ceil(words / 200) || 1;
                document.getElementById('wordCount').textContent = words + ' words';
                document.getElementById('readTime').textContent = readTime + ' min read';
            }}

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
                if (textSource.trim().split(/\\s+/).filter(Boolean).length >= 50) {{ score += 10; }}
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
                        scoreBadge.style.background = '#ecfdf5';
                        scoreBadge.style.color = '#059669';
                        scoreBadge.style.borderColor = '#a7f3d0';
                    }} else {{
                        scoreBadge.style.background = '#fffbeb';
                        scoreBadge.style.color = '#d97706';
                        scoreBadge.style.borderColor = '#fde68a';
                    }}
                }}
                const summaryEl = document.getElementById('seoChecklistSummary');
                if (summaryEl) {{
                    summaryEl.textContent = checks.join(' • ');
                }}
            }}

            async function duplicateCurrentPost() {{
                if (!activePostId) return;
                showToast('⚡ Cloning #' + activePostId + '...');
                try {{
                    const res = await fetch('/wp-json/neopress/v1/posts/' + activePostId + '/duplicate', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    if (res.ok) {{
                        const cloned = await res.json();
                        showToast('✓ Cloned! Redirecting to Draft #' + cloned.id + '...');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/post.php?post=' + cloned.id + '&action=edit';
                        }}, 500);
                    }} else {{
                        showToast('Failed to duplicate item', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            // Formatting toolbar actions
            function formatBlockType(tag) {{
                if (!tag) return;
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('formatBlock', false, tag);
                    syncVisualToTextarea();
                }} else {{
                    wrapSelectionInTextarea(tag);
                }}
                markDirty();
            }}

            function execStyle(cmd) {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand(cmd, false, null);
                    syncVisualToTextarea();
                }} else {{
                    const map = {{ 'bold': 'strong', 'italic': 'em', 'underline': 'u', 'strikeThrough': 's' }};
                    wrapSelectionInTextarea(map[cmd] || cmd);
                }}
                markDirty();
            }}

            function wrapSelectionInTextarea(tag) {{
                const start = contentArea.selectionStart;
                const end = contentArea.selectionEnd;
                const sel = contentArea.value.substring(start, end);
                const before = contentArea.value.substring(0, start);
                const after = contentArea.value.substring(end);
                const replacement = '<' + tag + '>' + (sel || 'text') + '</' + tag + '>';
                contentArea.value = before + replacement + after;
                contentArea.focus();
                contentArea.selectionStart = start + tag.length + 2;
                contentArea.selectionEnd = start + tag.length + 2 + (sel || 'text').length;
                updateWordCount();
                markDirty();
            }}

            function insertInlineCode() {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    const sel = window.getSelection();
                    const text = sel ? sel.toString() : '';
                    document.execCommand('insertHTML', false, '<code>' + (text || 'code') + '</code>');
                    syncVisualToTextarea();
                }} else {{
                    wrapSelectionInTextarea('code');
                }}
                markDirty();
            }}

            function insertList(type) {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    if (type === 'ul') {{
                        document.execCommand('insertUnorderedList', false, null);
                    }} else {{
                        document.execCommand('insertOrderedList', false, null);
                    }}
                    syncVisualToTextarea();
                }} else {{
                    const text = '\\n<' + type + '>\\n  <li>List item 1</li>\\n  <li>List item 2</li>\\n</' + type + '>\\n';
                    insertRaw(text);
                }}
                markDirty();
            }}

            function insertQuote() {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('formatBlock', false, '<blockquote>');
                    syncVisualToTextarea();
                }} else {{
                    wrapSelectionInTextarea('blockquote');
                }}
                markDirty();
            }}

            function insertLinkPrompt() {{
                const url = prompt("Enter URL destination (e.g. https://... or /about-us/):", "https://");
                if (!url) return;
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('createLink', false, url);
                    syncVisualToTextarea();
                }} else {{
                    const start = contentArea.selectionStart;
                    const end = contentArea.selectionEnd;
                    const sel = contentArea.value.substring(start, end) || url;
                    const replace = '<a href="' + url + '">' + sel + '</a>';
                    contentArea.value = contentArea.value.substring(0, start) + replace + contentArea.value.substring(end);
                    contentArea.focus();
                    updateWordCount();
                }}
                markDirty();
            }}

            function clearFormatting() {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('removeFormat', false, null);
                    syncVisualToTextarea();
                }}
                markDirty();
            }}

            function insertTable() {{
                const tableHtml = '\\n<table class="neo-table" style="width: 100%; border-collapse: collapse; margin: 16px 0;">\\n  <thead>\\n    <tr style="background: #f8fafc;">\\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 1</th>\\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 2</th>\\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 3</th>\\n    </tr>\\n  </thead>\\n  <tbody>\\n    <tr>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A1</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A2</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A3</td>\\n    </tr>\\n    <tr>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B1</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B2</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B3</td>\\n    </tr>\\n  </tbody>\\n</table>\\n<p></p>\\n';
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('insertHTML', false, tableHtml);
                    syncVisualToTextarea();
                }} else {{
                    insertRaw(tableHtml);
                }}
                markDirty();
            }}

            function insertCallout() {{
                const calloutHtml = '\\n<div style="background: #eff6ff; border-left: 4px solid #2563eb; padding: 14px 18px; border-radius: 6px; margin: 16px 0; font-size: 14px; color: #1e3a8a;">\\n    <strong>💡 Note:</strong> Add important highlighted details or tips here.\\n</div>\\n<p></p>\\n';
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('insertHTML', false, calloutHtml);
                    syncVisualToTextarea();
                }} else {{
                    insertRaw(calloutHtml);
                }}
                markDirty();
            }}

            function insertRaw(text) {{
                const start = contentArea.selectionStart;
                const end = contentArea.selectionEnd;
                contentArea.value = contentArea.value.substring(0, start) + text + contentArea.value.substring(end);
                contentArea.focus();
                updateWordCount();
                markDirty();
            }}

            function copyPermalink() {{
                const slug = slugInput.value;
                const url = window.location.origin + '/' + slug + '/';
                navigator.clipboard.writeText(url);
                showToast('✓ Permalink copied: ' + url);
            }}

            function previewPage() {{
                const slug = slugInput.value;
                if (!slug) {{
                    showToast('Please enter a title or slug first', true);
                    return;
                }}
                window.open('/' + slug + '/', '_blank');
            }}

            // Media Picker Modal
            function openMediaPicker(target) {{
                currentMediaTarget = target;
                document.getElementById('mediaPickerModal').style.display = 'flex';
            }}

            function closeMediaPicker() {{
                document.getElementById('mediaPickerModal').style.display = 'none';
            }}

            function switchMediaTab(tab) {{
                if (tab === 'library') {{
                    document.getElementById('mediaTabLibrary').style.display = 'block';
                    document.getElementById('mediaTabUpload').style.display = 'none';
                    document.getElementById('tabLibraryBtn').className = 'btn btn-primary';
                    document.getElementById('tabUploadBtn').className = 'btn btn-secondary';
                }} else {{
                    document.getElementById('mediaTabLibrary').style.display = 'none';
                    document.getElementById('mediaTabUpload').style.display = 'block';
                    document.getElementById('tabLibraryBtn').className = 'btn btn-secondary';
                    document.getElementById('tabUploadBtn').className = 'btn btn-primary';
                }}
            }}

            function selectMediaItem(id, url, title) {{
                if (currentMediaTarget === 'featured') {{
                    document.getElementById('featured_media').value = id;
                    const elem = document.getElementById('featuredImgElem');
                    elem.src = url;
                    document.getElementById('featuredImgContainer').style.display = 'block';
                    showToast('✓ Featured image assigned!');
                }} else {{
                    const imgHtml = '<figure style="margin: 16px 0;"><img src="' + url + '" alt="' + (title || 'Image') + '" style="max-width: 100%; height: auto; border-radius: 8px; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.08);" /><figcaption style="font-size: 12px; color: #64748b; margin-top: 6px; text-align: center;">' + (title || '') + '</figcaption></figure><p></p>';
                    if (currentMode === 'visual') {{
                        visualEditor.focus();
                        document.execCommand('insertHTML', false, imgHtml);
                        syncVisualToTextarea();
                    }} else {{
                        insertRaw('\\n' + imgHtml + '\\n');
                    }}
                    showToast('✓ Image inserted into {type_singular}');
                }}
                markDirty();
                closeMediaPicker();
            }}

            function removeFeaturedImage() {{
                document.getElementById('featured_media').value = '0';
                document.getElementById('featuredImgContainer').style.display = 'none';
                showToast('Featured image removed');
                markDirty();
            }}

            async function uploadModalImage(file) {{
                if (!file) return;
                const formData = new FormData();
                formData.append('file', file);
                showToast('Uploading to Neopress storage...');
                try {{
                    const res = await fetch('/wp-json/wp/v2/media', {{ method: 'POST', body: formData }});
                    const data = await res.json();
                    if (res.ok && data.source_url) {{
                        selectMediaItem(data.id, data.source_url, data.slug);
                    }} else {{
                        showToast('Error uploading image', true);
                    }}
                }} catch(e) {{
                    showToast('Upload failed: ' + e, true);
                }}
            }}

            async function uploadEditorImage(file) {{
                uploadModalImage(file);
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

            // Autosave Engine
            function markDirty() {{
                isDirty = true;
                updateAutosaveBadge('editing');
                saveToLocalStorage();
            }}

            function updateAutosaveBadge(state) {{
                const badge = document.getElementById('autosaveBadge');
                const dot = document.getElementById('autosaveDot');
                const label = document.getElementById('autosaveLabel');
                if (!badge || !dot || !label) return;

                if (state === 'editing') {{
                    badge.style.background = '#fef3c7';
                    badge.style.borderColor = '#fde68a';
                    badge.style.color = '#b45309';
                    dot.style.background = '#f59e0b';
                    label.textContent = 'Unsaved changes';
                }} else if (state === 'saving') {{
                    badge.style.background = '#eff6ff';
                    badge.style.borderColor = '#bfdbfe';
                    badge.style.color = '#1d4ed8';
                    dot.style.background = '#3b82f6';
                    label.textContent = 'Autosaving...';
                }} else if (state === 'synced') {{
                    badge.style.background = '#ecfdf5';
                    badge.style.borderColor = '#a7f3d0';
                    badge.style.color = '#059669';
                    dot.style.background = '#10b981';
                    const timeStr = lastAutosaveTime ? lastAutosaveTime.toLocaleTimeString([], {{ hour: '2-digit', minute: '2-digit', second: '2-digit' }}) : 'now';
                    label.textContent = 'Autosaved ' + timeStr;
                }} else if (state === 'error') {{
                    badge.style.background = '#fef2f2';
                    badge.style.borderColor = '#fecaca';
                    badge.style.color = '#b91c1c';
                    dot.style.background = '#ef4444';
                    label.textContent = 'Autosave offline (Local Backup)';
                }}
            }}

            function saveToLocalStorage() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    const draft = {{
                        title: titleInput.value,
                        slug: slugInput.value,
                        content: currentMode === 'visual' ? visualEditor.innerHTML : contentArea.value,
                        excerpt: document.getElementById('excerpt').value,
                        timestamp: Date.now()
                    }};
                    localStorage.setItem(key, JSON.stringify(draft));
                }} catch(e) {{}}
            }}

            function checkLocalDraftRecovery() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    const raw = localStorage.getItem(key);
                    if (!raw) return;
                    const draft = JSON.parse(raw);
                    if (draft && draft.timestamp && (Date.now() - draft.timestamp) < 7 * 86400 * 1000) {{
                        const banner = document.getElementById('draftRecoveryBanner');
                        const timeElem = document.getElementById('recoveryTime');
                        if (banner && timeElem) {{
                            timeElem.textContent = new Date(draft.timestamp).toLocaleTimeString();
                            banner.style.display = 'flex';
                        }}
                    }}
                }} catch(e) {{}}
            }}

            function restoreLocalDraft() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    const raw = localStorage.getItem(key);
                    if (!raw) return;
                    const draft = JSON.parse(raw);
                    if (draft.title) titleInput.value = draft.title;
                    if (draft.slug) slugInput.value = draft.slug;
                    if (draft.content) {{
                        contentArea.value = draft.content;
                        visualEditor.innerHTML = cleanWpContent(draft.content);
                    }}
                    if (draft.excerpt) document.getElementById('excerpt').value = draft.excerpt;
                    updateWordCount();
                    updateSerpPreview();
                    dismissLocalDraft();
                    showToast('✓ Local draft restored!');
                }} catch(e) {{}}
            }}

            function dismissLocalDraft() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    localStorage.removeItem(key);
                    const banner = document.getElementById('draftRecoveryBanner');
                    if (banner) banner.style.display = 'none';
                }} catch(e) {{}}
            }}

            // Periodic background autosave interval (every 15 seconds)
            setInterval(() => {{
                if (isDirty && !isAutosaving) {{
                    triggerAutosave(false);
                }}
            }}, 15000);

            async function triggerAutosave(showExplicitToast = false) {{
                if (isAutosaving || isVisualLocked) return;
                if (currentMode === 'visual') syncVisualToTextarea();

                const title = titleInput.value.trim();
                const content = contentArea.value.trim();
                if (!title && !content) return; // Don't autosave empty

                isAutosaving = true;
                updateAutosaveBadge('saving');

                const slug = slugInput.value.trim();
                const excerpt = document.getElementById('excerpt').value;
                const post_type = '{post_type}';
                const status = document.getElementById('post_status').value || 'publish';
                const parent = document.getElementById('post_parent') ? parseInt(document.getElementById('post_parent').value) || 0 : 0;
                const menu_order = document.getElementById('menu_order') ? parseInt(document.getElementById('menu_order').value) || 0 : 0;
                const featured_media = parseInt(document.getElementById('featured_media').value) || 0;
                const categories = Array.from(document.querySelectorAll('input[name="post_category"]:checked')).map(c => parseInt(c.value));

                try {{
                    let res, data;
                    if (activePostId && activePostId.length > 0) {{
                        res = await fetch('/wp-json/wp/v2/posts/' + activePostId, {{
                            method: 'PUT',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{
                                title: title || 'Draft ' + new Date().toLocaleTimeString(),
                                slug: slug || undefined,
                                content: isVisualLocked ? undefined : content,
                                excerpt: excerpt || undefined,
                                status,
                                parent,
                                menu_order,
                                featured_media: featured_media > 0 ? featured_media : undefined,
                                categories
                            }})
                        }});
                        data = await res.json();
                    }} else {{
                        res = await fetch('/wp-json/wp/v2/posts', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{
                                title: title || 'Draft ' + new Date().toLocaleTimeString(),
                                slug: slug || undefined,
                                content,
                                excerpt: excerpt || undefined,
                                status: 'draft',
                                post_type,
                                parent,
                                menu_order,
                                featured_media: featured_media > 0 ? featured_media : undefined,
                                categories
                            }})
                        }});
                        data = await res.json();
                        if (res.ok && data.id) {{
                            activePostId = data.id.toString();
                            const newUrl = '/n-admin/post.php?post=' + data.id + '&action=edit' + (post_type === 'page' ? '&post_type=page' : '');
                            window.history.replaceState({{ id: data.id }}, '', newUrl);
                        }}
                    }}

                    if (res.ok) {{
                        isDirty = false;
                        lastAutosaveTime = new Date();
                        updateAutosaveBadge('synced');
                        const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                        localStorage.removeItem(key);
                        if (showExplicitToast) {{
                            showToast('✓ Saved to SQLite (WAL)');
                        }}
                    }} else {{
                        updateAutosaveBadge('error');
                    }}
                }} catch(e) {{
                    console.error('Autosave error:', e);
                    updateAutosaveBadge('error');
                }} finally {{
                    isAutosaving = false;
                }}
            }}

            async function trashPost() {{
                const existingId = activePostId;
                if (!existingId) return;
                if (!confirm('Are you sure you want to permanently delete this {type_singular}?')) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + existingId, {{ method: 'DELETE' }});
                    if (res.ok) {{
                        showToast('✓ {type_singular} deleted');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/edit.php' + ('{post_type}' === 'page' ? '?post_type=page' : '');
                        }}, 400);
                    }}
                }} catch(e) {{
                    showToast('Error: ' + e, true);
                }}
            }}

            async function savePost(e) {{
                if (e) e.preventDefault();
                await submitData(document.getElementById('post_status').value || 'publish');
            }}

            async function saveDraft() {{
                await submitData('draft');
            }}

            async function submitData(status) {{
                if (currentMode === 'visual') {{
                    syncVisualToTextarea();
                }}

                const title = titleInput.value.trim();
                const slug = slugInput.value.trim();
                const content = contentArea.value;
                const excerpt = document.getElementById('excerpt').value;
                const post_type = '{post_type}';
                const existingId = activePostId;

                const parent = document.getElementById('post_parent') ? parseInt(document.getElementById('post_parent').value) || 0 : 0;
                const menu_order = document.getElementById('menu_order') ? parseInt(document.getElementById('menu_order').value) || 0 : 0;
                const featured_media = parseInt(document.getElementById('featured_media').value) || 0;

                const categories = Array.from(document.querySelectorAll('input[name="post_category"]:checked')).map(c => parseInt(c.value));
                const tplEl = document.getElementById('page_template') || document.getElementById('page-template-select');
                const template = tplEl ? tplEl.value : 'default';

                const knownTags = {known_tags_json};
                const tagsEl = document.getElementById('post_tags') || document.getElementById('post-tags-input');
                const tagsRaw = tagsEl ? tagsEl.value : '';
                const tagNames = tagsRaw.split(',').map(s => s.trim()).filter(Boolean);
                const tags = [];
                for (const tName of tagNames) {{
                    const lower = tName.toLowerCase();
                    if (knownTags[lower]) {{
                        tags.push(knownTags[lower]);
                    }} else {{
                        try {{
                            const tRes = await fetch('/wp-json/wp/v2/tags', {{
                                method: 'POST',
                                headers: {{ 'Content-Type': 'application/json' }},
                                body: JSON.stringify({{ name: tName }})
                            }});
                            if (tRes.ok) {{
                                const tData = await tRes.json();
                                if (tData && tData.id) {{
                                    knownTags[lower] = tData.id;
                                    tags.push(tData.id);
                                }}
                            }}
                        }} catch (_) {{}}
                    }}
                }}

                let url = '/wp-json/wp/v2/posts';
                let method = 'POST';
                if (existingId && existingId.length > 0) {{
                    url = '/wp-json/wp/v2/posts/' + existingId;
                    method = 'PUT';
                }}

                const seo_title = document.getElementById('seo_title') ? document.getElementById('seo_title').value.trim() : '';
                const seo_description = document.getElementById('seo_description') ? document.getElementById('seo_description').value.trim() : '';
                const seo_keyword = document.getElementById('seo_focus_kw') ? document.getElementById('seo_focus_kw').value.trim() : '';
                const seo_canonical = document.getElementById('seo_canonical') ? document.getElementById('seo_canonical').value.trim() : '';
                const seo_og_image = document.getElementById('seo_og_image') ? document.getElementById('seo_og_image').value.trim() : '';
                const seo_noindex = document.getElementById('seo_noindex') ? document.getElementById('seo_noindex').checked : false;
                const seo_schema_type = document.getElementById('seo_schema_type') ? document.getElementById('seo_schema_type').value : 'Article';

                const payload = {{
                    title: title || 'Untitled',
                    slug: slug || undefined,
                    content: isVisualLocked ? undefined : content,
                    excerpt: excerpt || undefined,
                    status,
                    post_type,
                    parent,
                    menu_order,
                    template,
                    featured_media: featured_media > 0 ? featured_media : undefined,
                    categories,
                    tags,
                    seo_title,
                    seo_description,
                    seo_keyword,
                    seo_canonical,
                    seo_og_image,
                    seo_noindex,
                    seo_schema_type
                }};

                try {{
                    const res = await fetch(url, {{
                        method: method,
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});

                    if (res.ok) {{
                        isDirty = false;
                        const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                        localStorage.removeItem(key);
                        showToast('✓ {type_singular} saved to SQLite successfully!');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/edit.php' + (post_type === 'page' ? '?post_type=page' : '');
                        }}, 500);
                    }} else {{
                        showToast('Error saving {type_singular}', true);
                    }}
                }} catch(e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
        </script>
        "##,
        title_label = title_label,
        type_singular = type_singular,
        post_type = post_type,
        btn_text = btn_text,
        top_builder_btn = top_builder_btn,
        duplicate_btn_html = duplicate_btn_html,
        locked_banner_html = locked_banner_html,
        standard_editor_display = standard_editor_display,
        has_visual_layout = has_visual_layout,
        initial_title = initial_title,
        initial_slug = initial_slug,
        initial_content = initial_content,
        initial_excerpt = initial_excerpt,
        initial_seo_title = initial_seo_title,
        initial_seo_desc = initial_seo_desc,
        initial_seo_kw = initial_seo_kw,
        initial_seo_canonical = initial_seo_canonical,
        initial_seo_og_img = initial_seo_og_img,
        seo_noindex_checked = seo_noindex_checked,
        schema_art_sel = schema_art_sel,
        schema_web_sel = schema_web_sel,
        schema_faq_sel = schema_faq_sel,
        schema_prod_sel = schema_prod_sel,
        schema_loc_sel = schema_loc_sel,
        initial_date = initial_date,
        initial_modified = initial_modified,
        initial_featured_media = initial_featured_media,
        initial_menu_order = initial_menu_order,
        initial_id = initial_id,
        initial_tag_str = initial_tag_str,
        is_pub_selected = is_pub_selected,
        is_draft_selected = is_draft_selected,
        is_pending_selected = is_pending_selected,
        is_private_selected = is_private_selected,
        parent_options = parent_options,
        cat_checkboxes = cat_checkboxes,
        featured_img_preview = featured_img_preview,
        media_picker_items = media_picker_items,
        popular_tag_chips = popular_tag_chips,
        known_tags_json = known_tags_json,
        tpl_default_sel = tpl_default_sel,
        tpl_full_sel = tpl_full_sel,
        tpl_canvas_sel = tpl_canvas_sel,
        tpl_landing_sel = tpl_landing_sel,
        tpl_editorial_sel = tpl_editorial_sel,
        site_title = site_title,
        page_attr_style = if is_page { "" } else { "display: none;" },
        post_tax_style = if !is_page { "" } else { "display: none;" }
    );

    let tab = if is_page { "pages" } else { "posts" };
    render_wp_admin_shell(tab, &title_label, &body, settings)
}
