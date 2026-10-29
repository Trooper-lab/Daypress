use super::layout::render_wp_admin_shell;
use crate::models::{WpMediaResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

pub fn render_wp_media_view(settings: &WpSettingsResponse, media: &[WpMediaResponse]) -> String {
    let mut media_cards = String::new();

    if media.is_empty() {
        media_cards.push_str(r#"
            <div style="grid-column: 1 / -1; text-align: center; padding: 48px; background: var(--paper-raised); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm);">
                <h3 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 4px;">Media library is empty</h3>
                <p style="font-size: 13px; color: var(--ink-muted); margin-bottom: 16px;">Drop files into the zone above or browse from your computer.</p>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('mediaUploadInput').click()">Select Images to Upload</button>
            </div>
        "#);
    } else {
        for m in media {
            let is_img = m.mime_type.starts_with("image/");
            let title_escaped = escape_html(&m.title.rendered);
            let url_escaped = escape_html(&m.source_url);
            let mime_escaped = escape_html(&m.mime_type);
            let date_escaped = escape_html(&m.date);

            let preview_html = if is_img {
                format!(
                    r#"<div style="height: 150px; background: var(--ink); overflow: hidden; display: flex; align-items: center; justify-content: center;">
                        <img src="{url_escaped}" alt="{title_escaped}" style="width: 100%; height: 100%; object-fit: cover; transition: transform 0.2s ease;" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'" loading="lazy" />
                    </div>"#
                )
            } else {
                format!(
                    r#"<div style="height: 150px; background: var(--paper-raised); display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--ink-muted);">
                        <svg style="width: 38px; height: 38px;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
                        <span style="font-family: var(--font-mono); font-size: 11px; margin-top: 6px; font-weight: 600;">{mime_escaped}</span>
                    </div>"#
                )
            };

            media_cards.push_str(&format!(
                r#"
                <div class="day-card" style="display: flex; flex-direction: column; overflow: hidden; cursor: pointer;" onclick="openMediaDetails({id})">
                    {preview_html}
                    <div style="padding: 12px; flex: 1;">
                        <div style="font-weight: 600; font-size: 13px; color: var(--ink); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title="{title_escaped}">{title_escaped}</div>
                        <div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 4px; display: flex; justify-content: space-between;">
                            <span style="font-family: var(--font-mono);">{mime_escaped}</span>
                            <span>{date_escaped}</span>
                        </div>
                    </div>
                    <div style="padding: 8px 12px; border-top: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised);" onclick="event.stopPropagation()">
                        <button type="button" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11.5px;" onclick="openMediaDetails({id})">Edit</button>
                        <button type="button" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11.5px;" onclick="copyMediaUrl('{url_escaped}')">Copy URL</button>
                        <button type="button" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11.5px; color: var(--oxide);" onclick="deleteMedia({id})">Delete</button>
                    </div>
                </div>
                "#,
                preview_html = preview_html,
                title_escaped = title_escaped,
                url_escaped = url_escaped,
                mime_escaped = mime_escaped,
                date_escaped = date_escaped,
                id = m.id
            ));
        }
    }

    let media_count = media.len();
    let media_json_items: Vec<serde_json::Value> = media
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "title": m.title.rendered,
                "source_url": m.source_url,
                "mime_type": m.mime_type,
                "file_size": m.file_size,
                "date": m.date,
                "alt_text": m.alt_text,
                "caption": m.caption.rendered,
                "description": m.description.rendered,
            })
        })
        .collect();
    let media_json_str =
        serde_json::to_string(&media_json_items).unwrap_or_else(|_| "[]".to_string());

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Media Library</h1>
                <p class="page-subtitle">Upload and manage images, video, and documents stored directly in embedded Rusqlite</p>
            </div>
            <div style="display: flex; align-items: center; gap: 12px;">
                <span class="badge badge-published" style="padding: 6px 12px; font-size: 12px;">{media_count} Assets</span>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('mediaUploadInput').click()">+ Upload Files</button>
                <input type="file" id="mediaUploadInput" multiple accept="image/*,video/*,application/pdf" style="display: none;" onchange="uploadFiles(this.files)" />
            </div>
        </div>

        <!-- Drag & Drop Upload Zone -->
        <div id="dropZone" style="border: 2px dashed var(--rule-strong); background: var(--paper-raised); border-radius: var(--radius-sm); padding: 36px 20px; text-align: center; margin-bottom: 24px; transition: all 0.2s ease; cursor: pointer;"
             onclick="document.getElementById('mediaUploadInput').click()"
             ondragover="event.preventDefault(); this.style.borderColor = 'var(--press)'; this.style.background = 'var(--press-wash)';"
             ondragleave="event.preventDefault(); this.style.borderColor = 'var(--rule-strong)'; this.style.background = 'var(--paper-raised)';"
             ondrop="event.preventDefault(); this.style.borderColor = 'var(--rule-strong)'; this.style.background = 'var(--paper-raised)'; uploadFiles(event.dataTransfer.files);">
            <div style="width: 44px; height: 44px; margin: 0 auto 10px auto; color: var(--press); background: var(--press-wash); border-radius: 50%; display: flex; align-items: center; justify-content: center;">
                <svg style="width: 22px; height: 22px;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
            </div>
            <h2 style="font-size: 15px; font-weight: 600; margin: 0 0 4px 0; color: var(--ink);">Drop files here to upload to Daypress storage</h2>
            <p style="font-size: 12.5px; color: var(--ink-muted); margin: 0 0 12px 0;">Supports PNG, JPG, WebP, SVG, GIF, PDF &bull; Fast async streaming pipeline</p>
            <button type="button" class="btn btn-secondary" style="font-size: 12px; pointer-events: none;">Select Files from Computer</button>
        </div>

        <!-- Media Grid -->
        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 18px;">
            {media_cards}
        </div>

        <!-- Attachment Details Modal -->
        <div id="mediaDetailsModal" style="display: none; position: fixed; inset: 0; background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(4px); z-index: 99999; align-items: center; justify-content: center; padding: 20px;">
            <div style="background: var(--paper-raised); border-radius: var(--radius-sm); width: 100%; max-width: 840px; max-height: 90vh; overflow-y: auto; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2); border: 1px solid var(--border-subtle); display: flex; flex-direction: column;">
                <div style="padding: 16px 20px; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised);">
                    <h3 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;">Attachment Details</h3>
                    <button type="button" onclick="closeMediaDetails()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted); line-height: 1;">&times;</button>
                </div>
                <div style="padding: 24px; display: grid; grid-template-columns: 1fr 1fr; gap: 24px;">
                    <!-- Left: Preview & Metadata -->
                    <div>
                        <div id="modalPreviewBox" style="width: 100%; height: 240px; background: var(--ink); border-radius: var(--radius-sm); overflow: hidden; display: flex; align-items: center; justify-content: center; margin-bottom: 14px;"></div>
                        <div style="font-size: 12px; color: var(--ink-muted); display: flex; flex-direction: column; gap: 6px; background: var(--paper-raised); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle);">
                            <div><strong style="color: var(--ink-muted);">File URL:</strong> <a id="modalFileUrlLink" href="#" target="_blank" style="color: var(--press); word-break: break-all;"></a></div>
                            <div><strong style="color: var(--ink-muted);">MIME Type:</strong> <span id="modalMime"></span></div>
                            <div><strong style="color: var(--ink-muted);">File Size:</strong> <span id="modalSize"></span></div>
                            <div><strong style="color: var(--ink-muted);">Upload Date:</strong> <span id="modalDate"></span></div>
                        </div>
                    </div>
                    <!-- Right: Editable Fields -->
                    <div style="display: flex; flex-direction: column; gap: 14px;">
                        <div>
                            <label class="form-label">Alternative Text (Alt Text)</label>
                            <input type="text" id="modalAltText" class="form-input" placeholder="Describe image for SEO &amp; accessibility" />
                        </div>
                        <div>
                            <label class="form-label">Title</label>
                            <input type="text" id="modalTitle" class="form-input" />
                        </div>
                        <div>
                            <label class="form-label">Caption</label>
                            <textarea id="modalCaption" class="form-textarea" style="height: 60px; min-height: 60px;" placeholder="Optional caption"></textarea>
                        </div>
                        <div>
                            <label class="form-label">Description</label>
                            <textarea id="modalDescription" class="form-textarea" style="height: 80px; min-height: 80px;" placeholder="Optional description"></textarea>
                        </div>
                    </div>
                </div>
                <div style="padding: 14px 20px; border-top: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised);">
                    <button type="button" class="btn btn-secondary" style="color: var(--oxide); border-color: var(--oxide);" onclick="deleteModalMedia()">Delete Permanently</button>
                    <div style="display: flex; gap: 10px;">
                        <button type="button" class="btn btn-secondary" onclick="closeMediaDetails()">Cancel</button>
                        <button type="button" class="btn btn-primary" onclick="saveMediaDetails()">Save Changes</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
            const mediaList = {media_json_str};
            const mediaData = {{}};
            mediaList.forEach(m => {{ mediaData[m.id] = m; }});
            let currentModalMediaId = null;

            function openMediaDetails(id) {{
                const item = mediaData[id];
                if (!item) return;
                currentModalMediaId = id;
                document.getElementById('modalTitle').value = item.title || '';
                document.getElementById('modalAltText').value = item.alt_text || '';
                document.getElementById('modalCaption').value = item.caption || '';
                document.getElementById('modalDescription').value = item.description || '';
                document.getElementById('modalFileUrlLink').href = item.source_url;
                document.getElementById('modalFileUrlLink').textContent = item.source_url;
                document.getElementById('modalMime').textContent = item.mime_type;
                document.getElementById('modalSize').textContent = item.file_size ? (Math.round(item.file_size / 1024) + ' KB') : '—';
                document.getElementById('modalDate').textContent = item.date ? item.date.substring(0, 10) : '—';

                const pBox = document.getElementById('modalPreviewBox');
                if (item.mime_type.startsWith('image/')) {{
                    pBox.innerHTML = '<img src="' + item.source_url + '" alt="" style="max-width: 100%; max-height: 100%; object-fit: contain;" />';
                }} else {{
                    pBox.innerHTML = '<div style="color: var(--ink-muted); font-size: 14px; font-weight: 600;">' + item.mime_type + '</div>';
                }}

                document.getElementById('mediaDetailsModal').style.display = 'flex';
            }}

            function closeMediaDetails() {{
                document.getElementById('mediaDetailsModal').style.display = 'none';
                currentModalMediaId = null;
            }}

            async function saveMediaDetails() {{
                const id = currentModalMediaId;
                if (!id) return;
                const title = document.getElementById('modalTitle').value;
                const alt_text = document.getElementById('modalAltText').value;
                const caption = document.getElementById('modalCaption').value;
                const description = document.getElementById('modalDescription').value;

                showToast('Saving attachment metadata...');
                try {{
                    const res = await fetch('/wp-json/wp/v2/media/' + id, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            title: title,
                            alt_text: alt_text,
                            caption: caption,
                            description: description
                        }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Media attachment #' + id + ' saved to SQLite!');
                        if (mediaData[id]) {{
                            mediaData[id].title = title;
                            mediaData[id].alt_text = alt_text;
                            mediaData[id].caption = caption;
                            mediaData[id].description = description;
                        }}
                        closeMediaDetails();
                        setTimeout(() => window.location.reload(), 500);
                    }} else {{
                        showToast('Failed to update media details', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            function deleteModalMedia() {{
                if (currentModalMediaId) {{
                    deleteMedia(currentModalMediaId);
                }}
            }}
            async function uploadFiles(files) {{
                if (!files || files.length === 0) return;
                showToast('Uploading ' + files.length + ' file(s) to SQLite storage...');
                let successCount = 0;

                for (let i = 0; i < files.length; i++) {{
                    const file = files[i];
                    const formData = new FormData();
                    formData.append('file', file);

                    try {{
                        const res = await fetch('/wp-json/wp/v2/media', {{
                            method: 'POST',
                            body: formData
                        }});
                        if (res.ok) {{
                            successCount++;
                        }}
                    }} catch (e) {{
                        console.error('Upload failed for ' + file.name, e);
                    }}
                }}

                if (successCount > 0) {{
                    showToast('✓ Successfully uploaded ' + successCount + ' file(s)!');
                    setTimeout(() => window.location.reload(), 600);
                }} else {{
                    showToast('Failed to upload files', true);
                }}
            }}

            function copyMediaUrl(url) {{
                const fullUrl = window.location.origin + url;
                navigator.clipboard.writeText(fullUrl);
                showToast('✓ Media URL copied: ' + fullUrl);
            }}

            async function deleteMedia(id) {{
                if (!confirm('Are you sure you want to permanently delete this media file?')) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/media/' + id, {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        showToast('✓ Media asset deleted successfully');
                        setTimeout(() => window.location.reload(), 500);
                    }} else {{
                        showToast('Failed to delete media asset', true);
                    }}
                }} catch (e) {{
                    showToast('Error: ' + e, true);
                }}
            }}
        </script>
        "##,
        media_count = media_count,
        media_cards = media_cards
    );

    render_wp_admin_shell("media", "Media Library", &body, settings)
}

pub fn render_wp_media_new_view(settings: &WpSettingsResponse) -> String {
    let body = r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Upload New Media</h1>
                <p class="page-subtitle">Add images, audio, video, or documents to your Daypress asset storage</p>
            </div>
            <div>
                <a href="/d-admin/upload" class="btn btn-secondary">&larr; Return to Library</a>
            </div>
        </div>

        <div class="day-card" style="padding: 48px; text-align: center; border: 2px dashed var(--border-hover); background: var(--paper-raised); border-radius: var(--radius-sm);" id="dropZone" ondragover="handleDragOver(event)" ondragleave="handleDragLeave(event)" ondrop="handleDrop(event)">
            <input type="file" id="multiFileInput" multiple style="display: none;" onchange="handleFileSelect(event)" />
            <div style="width: 56px; height: 56px; border-radius: 50%; background: var(--press-wash); color: var(--press); display: flex; align-items: center; justify-content: center; margin: 0 auto 16px auto;">
                <svg style="width: 28px; height: 28px;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
            </div>
            <h2 style="font-size: 18px; font-weight: 700; color: var(--ink); margin-bottom: 6px;">Drop files anywhere to upload</h2>
            <p style="font-size: 13px; color: var(--ink-muted); margin-bottom: 20px;">Or choose files from your device to automatically scale and convert to WebP variants</p>
            <button type="button" class="btn btn-primary" onclick="document.getElementById('multiFileInput').click()" style="padding: 10px 22px; font-size: 14px;">Select Files</button>
            <div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 18px;">Maximum upload file size: 64 MB. Supports JPEG, PNG, WebP, GIF, SVG, PDF.</div>
        </div>

        <div id="uploadProgressContainer" style="display: none; margin-top: 24px;">
            <div class="day-card" style="padding: 20px;">
                <h3 style="font-size: 14px; font-weight: 600; margin-bottom: 12px; color: var(--ink);">Uploading Assets...</h3>
                <div style="width: 100%; height: 8px; background: var(--paper-sunk); border-radius: 4px; overflow: hidden;">
                    <div id="uploadProgressBar" style="width: 0%; height: 100%; background: var(--press); transition: width 0.2s;"></div>
                </div>
                <div id="uploadedFilesList" style="margin-top: 16px; display: flex; flex-direction: column; gap: 8px;"></div>
            </div>
        </div>

        <script>
            function handleDragOver(e) {
                e.preventDefault();
                document.getElementById('dropZone').style.borderColor = 'var(--press)';
                document.getElementById('dropZone').style.background = '#f0f7ff';
            }
            function handleDragLeave(e) {
                e.preventDefault();
                document.getElementById('dropZone').style.borderColor = 'var(--border-hover)';
                document.getElementById('dropZone').style.background = 'var(--paper-raised)';
            }
            function handleDrop(e) {
                e.preventDefault();
                handleDragLeave(e);
                if (e.dataTransfer && e.dataTransfer.files) {
                    uploadFiles(e.dataTransfer.files);
                }
            }
            function handleFileSelect(e) {
                if (e.target.files) {
                    uploadFiles(e.target.files);
                }
            }
            async function uploadFiles(files) {
                if (!files || files.length === 0) return;
                const container = document.getElementById('uploadProgressContainer');
                const bar = document.getElementById('uploadProgressBar');
                const list = document.getElementById('uploadedFilesList');
                container.style.display = 'block';
                list.innerHTML = '';

                let completed = 0;
                for (let i = 0; i < files.length; i++) {
                    const file = files[i];
                    const formData = new FormData();
                    formData.append('file', file);
                    try {
                        const res = await fetch('/wp-json/wp/v2/media', {
                            method: 'POST',
                            body: formData
                        });
                        completed++;
                        const pct = Math.round((completed / files.length) * 100);
                        bar.style.width = pct + '%';
                        if (res.ok) {
                            const data = await res.json();
                            const item = document.createElement('div');
                            item.style.display = 'flex';
                            item.style.justifyContent = 'space-between';
                            item.style.fontSize = '13px';
                            item.style.padding = '8px 12px';
                            item.style.background = 'var(--paper-raised)';
                            item.style.borderRadius = '6px';
                            item.innerHTML = '<span>✓ ' + file.name + '</span><a href="' + data.source_url + '" target="_blank" style="color: var(--press); text-decoration: none;">View File &nearr;</a>';
                            list.appendChild(item);
                        } else {
                            showToast('Failed to upload ' + file.name, true);
                        }
                    } catch (err) {
                        showToast('Error uploading ' + file.name, true);
                    }
                }
                showToast('✓ Upload complete! ' + completed + ' files processed');
            }
        </script>
    "##;

    render_wp_admin_shell("media-new", "Upload New Media", body, settings)
}
