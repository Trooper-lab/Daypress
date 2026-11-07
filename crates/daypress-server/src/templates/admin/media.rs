use super::layout::render_admin_shell;
use crate::models::{DaypressMediaResponse, DaypressSettingsResponse};
use crate::templates::utils::escape_html;

pub fn render_media_view(
    settings: &DaypressSettingsResponse,
    media: &[DaypressMediaResponse],
) -> String {
    let mut media_cards = String::new();
    let mut all_tags_set = std::collections::BTreeSet::<String>::new();

    if media.is_empty() {
        media_cards.push_str(r#"
            <div id="emptyMediaState" style="grid-column: 1 / -1; text-align: center; padding: 48px; background: var(--paper-raised); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm);">
                <h3 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 4px;">Media library is empty</h3>
                <p style="font-size: 13px; color: var(--ink-muted); margin-bottom: 16px;">Drop files into the zone above or browse from your computer.</p>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('mediaUploadInput').click()">Select Images to Upload</button>
            </div>
        "#);
    } else {
        for m in media {
            for t in &m.tags {
                let clean_t = t.trim();
                if !clean_t.is_empty() {
                    all_tags_set.insert(clean_t.to_string());
                }
            }
            let is_img = m.mime_type.starts_with("image/");
            let title_escaped = escape_html(&m.title.rendered);
            let url_escaped = escape_html(&m.source_url);
            let mime_escaped = escape_html(&m.mime_type);
            let date_escaped = escape_html(&m.date);
            let folder_attr = m
                .folder_id
                .map(|fid| fid.to_string())
                .unwrap_or_else(|| "0".to_string());
            let tags_csv = escape_html(&m.tags.join(","));
            let fx_pct = (m.focal_x.clamp(0.0, 1.0) * 100.0).round() as i32;
            let fy_pct = (m.focal_y.clamp(0.0, 1.0) * 100.0).round() as i32;

            let has_webp = m.media_details.sizes.contains_key("webp")
                || m.mime_type.eq_ignore_ascii_case("image/webp");
            let has_avif = m.media_details.sizes.contains_key("avif")
                || m.mime_type.eq_ignore_ascii_case("image/avif");

            let mut variant_badges = String::new();
            if has_webp {
                variant_badges.push_str(r#"<span style="background: rgba(16, 185, 129, 0.9); color: #fff; font-family: var(--font-mono); font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 3px;">WebP</span>"#);
            }
            if has_avif {
                variant_badges.push_str(r#"<span style="background: rgba(59, 130, 246, 0.9); color: #fff; font-family: var(--font-mono); font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 3px;">AVIF</span>"#);
            }

            let preview_html = if is_img {
                format!(
                    r#"<div style="position: relative; height: 150px; background: var(--ink); overflow: hidden; display: flex; align-items: center; justify-content: center;">
                        <img id="gridMediaImg-{id}" src="{url_escaped}" alt="{title_escaped}" style="width: 100%; height: 100%; object-fit: cover; object-position: {fx_pct}% {fy_pct}%; transition: transform 0.2s ease;" onmouseover="this.style.transform='scale(1.05)'" onmouseout="this.style.transform='scale(1)'" loading="lazy" />
                        <div style="position: absolute; top: 8px; right: 8px; display: flex; gap: 4px;">{variant_badges}</div>
                    </div>"#,
                    id = m.id,
                    url_escaped = url_escaped,
                    title_escaped = title_escaped,
                    fx_pct = fx_pct,
                    fy_pct = fy_pct,
                    variant_badges = variant_badges
                )
            } else {
                format!(
                    r#"<div style="height: 150px; background: var(--paper-raised); display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--ink-muted);">
                        <svg style="width: 38px; height: 38px;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
                        <span style="font-family: var(--font-mono); font-size: 11px; margin-top: 6px; font-weight: 600;">{mime_escaped}</span>
                    </div>"#
                )
            };

            let tag_pills_html = if m.tags.is_empty() {
                String::new()
            } else {
                let pills: Vec<String> = m
                    .tags
                    .iter()
                    .take(3)
                    .map(|t| {
                        let te = escape_html(t);
                        format!(
                            r#"<span style="font-size: 10px; padding: 1px 6px; border-radius: 10px; background: var(--press-wash); color: var(--press); font-weight: 600;">#{te}</span>"#
                        )
                    })
                    .collect();
                format!(
                    r#"<div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px;">{}</div>"#,
                    pills.join("")
                )
            };

            media_cards.push_str(&format!(
                r#"
                <div class="day-card media-asset-card" id="mediaCard-{id}" data-media-id="{id}" data-folder-id="{folder_attr}" data-tags="{tags_csv}" style="position: relative; display: flex; flex-direction: column; overflow: hidden; cursor: pointer;" onclick="handleMediaCardClick(event, {id})">
                    <div class="media-bulk-check-wrap" style="display: none; position: absolute; top: 8px; left: 8px; z-index: 5; background: rgba(15, 23, 42, 0.78); padding: 4px 6px; border-radius: 4px;" onclick="event.stopPropagation()">
                        <input type="checkbox" class="media-bulk-checkbox" value="{id}" onchange="syncMediaSelection({id}, this.checked)" style="cursor: pointer; width: 15px; height: 15px;" />
                    </div>
                    {preview_html}
                    <div style="padding: 12px; flex: 1;">
                        <div style="font-weight: 600; font-size: 13px; color: var(--ink); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title="{title_escaped}">{title_escaped}</div>
                        <div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 4px; display: flex; justify-content: space-between;">
                            <span style="font-family: var(--font-mono);">{mime_escaped}</span>
                            <span>{date_escaped}</span>
                        </div>
                        {tag_pills_html}
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
                tag_pills_html = tag_pills_html,
                folder_attr = folder_attr,
                tags_csv = tags_csv,
                id = m.id
            ));
        }
    }

    let mut tag_filter_pills = String::new();
    for tag in &all_tags_set {
        let te = escape_html(tag);
        tag_filter_pills.push_str(&format!(
            r#"<button type="button" class="btn btn-secondary tag-pill-btn" data-tag="{te}" style="padding: 3px 10px; font-size: 11.5px; border-radius: 999px;" onclick="filterByTag('{te}')">#{te}</button>"#
        ));
    }

    let media_count = media.len();
    let media_json_items: Vec<serde_json::Value> = media
        .iter()
        .map(|m| {
            let has_webp = m.media_details.sizes.contains_key("webp")
                || m.mime_type.eq_ignore_ascii_case("image/webp");
            let has_avif = m.media_details.sizes.contains_key("avif")
                || m.mime_type.eq_ignore_ascii_case("image/avif");
            let webp_url = m
                .media_details
                .sizes
                .get("webp")
                .map(|s| s.source_url.clone())
                .unwrap_or_default();
            let avif_url = m
                .media_details
                .sizes
                .get("avif")
                .map(|s| s.source_url.clone())
                .unwrap_or_default();
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
                "folder_id": m.folder_id,
                "tags": m.tags,
                "focal_x": m.focal_x,
                "focal_y": m.focal_y,
                "has_webp": has_webp,
                "has_avif": has_avif,
                "webp_url": webp_url,
                "avif_url": avif_url,
            })
        })
        .collect();
    let media_json_str =
        serde_json::to_string(&media_json_items).unwrap_or_else(|_| "[]".to_string());

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Media Library &amp; DAM</h1>
                <p class="page-subtitle">Organize virtual folders, focal-point crops, sanitized SVGs, and dual WebP/AVIF variants</p>
            </div>
            <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
                <span class="badge badge-published" style="padding: 6px 12px; font-size: 12px;">{media_count} Assets</span>
                <div style="display: inline-flex; border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); overflow: hidden;">
                    <button type="button" id="btnMediaViewGrid" class="btn btn-primary" style="border-radius: 0; border: none; padding: 6px 11px; font-size: 12px;" onclick="setMediaViewMode('grid')" title="Grid View">&#8862; Grid</button>
                    <button type="button" id="btnMediaViewList" class="btn btn-secondary" style="border-radius: 0; border: none; padding: 6px 11px; font-size: 12px;" onclick="setMediaViewMode('list')" title="List View">&#9776; List</button>
                </div>
                <button type="button" class="btn btn-secondary" id="btnMediaBulkSelect" onclick="toggleMediaBulkSelectMode()">&#9745; Bulk Select</button>
                <button type="button" class="btn btn-secondary" id="scanUnusedBtn" onclick="scanUnusedMedia()" style="display: inline-flex; align-items: center; gap: 6px;">
                    <span>Scan Unused Assets</span>
                    <span id="unusedAssetsBadge" style="display: none; background: var(--oxide); color: #fff; font-size: 11px; font-weight: 700; padding: 1px 7px; border-radius: 999px;">0</span>
                </button>
                <button type="button" class="btn btn-secondary" id="bulkDeleteUnusedBtn" onclick="bulkDeleteUnusedMedia()" style="display: none; color: var(--oxide); border-color: var(--oxide);">
                    Bulk Delete Unused
                </button>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('mediaUploadInput').click()">+ Upload Files</button>
                <input type="file" id="mediaUploadInput" multiple accept="image/*,video/*,application/pdf,.svg" style="display: none;" onchange="uploadFiles(this.files)" />
            </div>
        </div>

        <!-- Bulk Select Action Bar -->
        <div id="mediaBulkActionBar" class="day-card" style="display: none; padding: 12px 18px; margin-bottom: 16px; border: 1px solid var(--press); background: var(--press-wash); align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px;">
            <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
                <span id="mediaBulkSelectedCount" style="font-size: 12.5px; font-weight: 700; color: var(--press);">0 selected</span>
                <button type="button" class="btn btn-secondary" style="padding: 4px 10px; font-size: 11.5px;" onclick="selectAllVisibleMedia(true)">Select All Visible</button>
                <button type="button" class="btn btn-secondary" style="padding: 4px 10px; font-size: 11.5px;" onclick="selectAllVisibleMedia(false)">Clear Selection</button>
            </div>
            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                <select id="bulkMediaFolderSelect" class="form-input" style="width: 170px; height: 32px; padding: 4px 8px; font-size: 12px;">
                    <option value="">&mdash; Move to Folder &mdash;</option>
                    <option value="0">Uncategorized</option>
                </select>
                <button type="button" class="btn btn-secondary" style="padding: 5px 11px; font-size: 12px;" onclick="bulkMoveSelectedMedia()">Move</button>
                <input type="text" id="bulkMediaTagInput" class="form-input" placeholder="Add tag..." style="width: 140px; height: 32px; padding: 4px 8px; font-size: 12px;" />
                <button type="button" class="btn btn-secondary" style="padding: 5px 11px; font-size: 12px;" onclick="bulkTagSelectedMedia()">+ Tag</button>
                <button type="button" class="btn btn-secondary" style="padding: 5px 12px; font-size: 12px; color: var(--oxide); border-color: var(--oxide);" onclick="bulkDeleteSelectedMedia()">Delete Selected</button>
            </div>
        </div>

        <!-- DAM Virtual Folders & Tag Filter Toolbar -->
        <div class="day-card" style="padding: 14px 18px; margin-bottom: 18px; display: flex; flex-direction: column; gap: 12px;">
            <div style="display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 10px;">
                <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;" id="mediaFoldersBar">
                    <span style="font-size: 12px; font-weight: 700; color: var(--ink-muted); text-transform: uppercase; letter-spacing: 0.04em; margin-right: 4px;">Folders:</span>
                    <button type="button" class="btn btn-primary folder-tab-btn" data-folder-filter="all" style="padding: 5px 12px; font-size: 12px;" onclick="filterByFolder('all')">All Media ({media_count})</button>
                    <button type="button" class="btn btn-secondary folder-tab-btn" data-folder-filter="0" style="padding: 5px 12px; font-size: 12px;" onclick="filterByFolder('0')">Uncategorized</button>
                    <div id="dynamicFolderTabs" style="display: inline-flex; align-items: center; gap: 6px; flex-wrap: wrap;"></div>
                </div>
                <div style="display: flex; align-items: center; gap: 8px;">
                    <input type="text" id="newFolderNameInput" class="form-input" placeholder="New folder name..." style="width: 170px; padding: 5px 10px; font-size: 12px; height: 32px;" />
                    <button type="button" class="btn btn-secondary" style="padding: 5px 12px; font-size: 12px; height: 32px;" onclick="createMediaFolder()">+ New Folder</button>
                </div>
            </div>

            <div style="display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 10px; border-top: 1px solid var(--border-subtle); padding-top: 10px;">
                <div style="display: flex; align-items: center; gap: 6px; flex-wrap: wrap;">
                    <span style="font-size: 12px; font-weight: 600; color: var(--ink-muted);">Tag Filter:</span>
                    <input type="text" id="mediaTagFilterInput" class="form-input" placeholder="Filter by tag or title..." style="width: 210px; padding: 4px 10px; font-size: 12px; height: 30px;" oninput="applyMediaFilters()" />
                    {tag_filter_pills}
                </div>
                <div id="activeFilterStatus" style="font-size: 12px; color: var(--ink-muted);">Showing all media assets</div>
            </div>
        </div>

        <!-- Drag & Drop Upload Zone -->
        <div id="dropZone" style="border: 2px dashed var(--rule-strong); background: var(--paper-raised); border-radius: var(--radius-sm); padding: 28px 20px; text-align: center; margin-bottom: 24px; transition: all 0.2s ease; cursor: pointer;"
             onclick="document.getElementById('mediaUploadInput').click()"
             ondragover="event.preventDefault(); this.style.borderColor = 'var(--press)'; this.style.background = 'var(--press-wash)';"
             ondragleave="event.preventDefault(); this.style.borderColor = 'var(--rule-strong)'; this.style.background = 'var(--paper-raised)';"
             ondrop="event.preventDefault(); this.style.borderColor = 'var(--rule-strong)'; this.style.background = 'var(--paper-raised)'; uploadFiles(event.dataTransfer.files);">
            <div style="width: 40px; height: 40px; margin: 0 auto 8px auto; color: var(--press); background: var(--press-wash); border-radius: 50%; display: flex; align-items: center; justify-content: center;">
                <svg style="width: 20px; height: 20px;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
            </div>
            <h2 style="font-size: 14.5px; font-weight: 600; margin: 0 0 4px 0; color: var(--ink);">Drop files here to upload to Daypress DAM</h2>
            <p style="font-size: 12px; color: var(--ink-muted); margin: 0 0 10px 0;">Supports PNG, JPG, WebP, AVIF, Sanitized SVG, GIF, PDF &bull; Auto-generates WebP &amp; AVIF variants</p>
            <button type="button" class="btn btn-secondary" style="font-size: 12px; pointer-events: none;">Select Files from Computer</button>
        </div>

        <!-- Media Grid -->
        <div id="mediaGridContainer" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 18px;">
            {media_cards}
        </div>

        <!-- Media List View Table -->
        <div id="mediaListViewContainer" class="day-table-wrapper" style="display: none;">
            <table class="day-table">
                <thead>
                    <tr>
                        <th style="width: 36px;"><input type="checkbox" onchange="selectAllVisibleMedia(this.checked)" /></th>
                        <th style="width: 64px;">Preview</th>
                        <th>File / Title</th>
                        <th>MIME Type</th>
                        <th>Folder &amp; Tags</th>
                        <th>Size</th>
                        <th>Date</th>
                        <th style="text-align: right;">Actions</th>
                    </tr>
                </thead>
                <tbody id="mediaListTableBody"></tbody>
            </table>
        </div>

        <!-- Attachment Details Modal -->
        <div id="mediaDetailsModal" style="display: none; position: fixed; inset: 0; background: rgba(15, 23, 42, 0.6); backdrop-filter: blur(4px); z-index: 99999; align-items: center; justify-content: center; padding: 20px;">
            <div style="background: var(--paper-raised); border-radius: var(--radius-sm); width: 100%; max-width: 900px; max-height: 92vh; overflow-y: auto; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2); border: 1px solid var(--border-subtle); display: flex; flex-direction: column;">
                <div style="padding: 16px 20px; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; background: var(--paper-raised);">
                    <div style="display: flex; align-items: center; gap: 10px;">
                        <h3 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;">Attachment Details &amp; Focal Crop</h3>
                        <div id="modalVariantBadges" style="display: flex; gap: 6px;"></div>
                    </div>
                    <button type="button" onclick="closeMediaDetails()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted); line-height: 1;">&times;</button>
                </div>
                <div style="padding: 24px; display: grid; grid-template-columns: 1.05fr 1fr; gap: 24px;">
                    <!-- Left: Interactive Focal-Point Preview, Replace File & Metadata -->
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                            <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Click image to set Focal Point crosshair</span>
                            <span id="focalCoordsLabel" style="font-family: var(--font-mono); font-size: 11.5px; color: var(--press); font-weight: 600;">Focal: 50%, 50%</span>
                        </div>
                        <div id="modalPreviewBox" onclick="handleFocalPointClick(event)" style="position: relative; width: 100%; height: 260px; background: var(--ink); border-radius: var(--radius-sm); overflow: hidden; display: flex; align-items: center; justify-content: center; margin-bottom: 12px; cursor: crosshair; user-select: none;"></div>

                        <!-- Replace Media File (Keep URL & ID) -->
                        <div style="margin-bottom: 12px; display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 10px 12px; background: var(--paper-sunk); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm);">
                            <div>
                                <div style="font-size: 12.5px; font-weight: 600; color: var(--ink);">Replace Media File (Keep URL &amp; ID)</div>
                                <div style="font-size: 11px; color: var(--ink-muted);">Overwrites file in-place without breaking existing post embeds</div>
                            </div>
                            <button type="button" class="btn btn-secondary" style="font-size: 12px; white-space: nowrap;" onclick="document.getElementById('replaceMediaFileInput').click()">Replace File...</button>
                            <input type="file" id="replaceMediaFileInput" accept="image/*,video/*,application/pdf,.svg" style="display: none;" onchange="replaceCurrentMediaFile(this.files)" />
                        </div>

                        <div style="font-size: 12px; color: var(--ink-muted); display: flex; flex-direction: column; gap: 6px; background: var(--paper-raised); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle);">
                            <div><strong style="color: var(--ink-muted);">File URL:</strong> <a id="modalFileUrlLink" href="#" target="_blank" style="color: var(--press); word-break: break-all;"></a></div>
                            <div><strong style="color: var(--ink-muted);">MIME Type:</strong> <span id="modalMime"></span></div>
                            <div><strong style="color: var(--ink-muted);">File Size:</strong> <span id="modalSize"></span></div>
                            <div><strong style="color: var(--ink-muted);">Upload Date:</strong> <span id="modalDate"></span></div>
                        </div>
                    </div>
                    <!-- Right: Editable Fields -->
                    <div style="display: flex; flex-direction: column; gap: 12px;">
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
                            <div>
                                <label class="form-label">Virtual Folder</label>
                                <select id="modalFolderSelect" class="form-input">
                                    <option value="0">Uncategorized</option>
                                </select>
                            </div>
                            <div>
                                <label class="form-label">Tags (comma-separated)</label>
                                <input type="text" id="modalTagsInput" class="form-input" placeholder="hero, product, brand" />
                            </div>
                        </div>
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
                            <textarea id="modalCaption" class="form-textarea" style="height: 54px; min-height: 54px;" placeholder="Optional caption"></textarea>
                        </div>
                        <div>
                            <label class="form-label">Description</label>
                            <textarea id="modalDescription" class="form-textarea" style="height: 68px; min-height: 68px;" placeholder="Optional description"></textarea>
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
            let currentFocalX = 0.5;
            let currentFocalY = 0.5;
            let activeFolderFilter = 'all';
            let unusedMediaIds = null;
            let foldersList = [];
            let mediaViewMode = 'grid';
            let mediaBulkSelectActive = false;
            const selectedMediaIds = new Set();

            async function loadMediaFolders() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/media/folders');
                    if (!res.ok) return;
                    foldersList = await res.json();
                    renderFolderControls();
                    renderMediaListView();
                }} catch (e) {{
                    console.error('Failed to load media folders', e);
                }}
            }}

            function getFolderNameById(fid) {{
                if (!fid || String(fid) === '0') return 'Uncategorized';
                const found = foldersList.find(f => String(f.id) === String(fid));
                return found ? found.name : 'Uncategorized';
            }}

            function renderFolderControls() {{
                const container = document.getElementById('dynamicFolderTabs');
                const selectEl = document.getElementById('modalFolderSelect');
                const bulkSelectEl = document.getElementById('bulkMediaFolderSelect');
                if (container) {{
                    container.innerHTML = '';
                    foldersList.forEach(f => {{
                        const wrap = document.createElement('div');
                        wrap.style.display = 'inline-flex';
                        wrap.style.alignItems = 'center';
                        wrap.style.gap = '2px';

                        const btn = document.createElement('button');
                        btn.type = 'button';
                        btn.className = (String(activeFolderFilter) === String(f.id)) ? 'btn btn-primary folder-tab-btn' : 'btn btn-secondary folder-tab-btn';
                        btn.setAttribute('data-folder-filter', String(f.id));
                        btn.style.padding = '5px 10px';
                        btn.style.fontSize = '12px';
                        btn.textContent = f.name + ' (' + (f.count || 0) + ')';
                        btn.onclick = () => filterByFolder(String(f.id));

                        const delBtn = document.createElement('button');
                        delBtn.type = 'button';
                        delBtn.className = 'btn btn-secondary';
                        delBtn.style.padding = '5px 6px';
                        delBtn.style.fontSize = '11px';
                        delBtn.style.color = 'var(--oxide)';
                        delBtn.title = 'Delete folder ' + f.name;
                        delBtn.innerHTML = '&times;';
                        delBtn.onclick = (ev) => {{
                            ev.stopPropagation();
                            deleteMediaFolder(f.id, f.name);
                        }};

                        wrap.appendChild(btn);
                        wrap.appendChild(delBtn);
                        container.appendChild(wrap);
                    }});
                }}

                if (selectEl) {{
                    const prevVal = selectEl.value;
                    selectEl.innerHTML = '<option value="0">Uncategorized</option>';
                    foldersList.forEach(f => {{
                        const opt = document.createElement('option');
                        opt.value = String(f.id);
                        opt.textContent = f.name;
                        selectEl.appendChild(opt);
                    }});
                    if (prevVal) selectEl.value = prevVal;
                }}

                if (bulkSelectEl) {{
                    const prevBulkVal = bulkSelectEl.value;
                    bulkSelectEl.innerHTML = '<option value="">\u2014 Move to Folder \u2014</option><option value="0">Uncategorized</option>';
                    foldersList.forEach(f => {{
                        const opt = document.createElement('option');
                        opt.value = String(f.id);
                        opt.textContent = f.name;
                        bulkSelectEl.appendChild(opt);
                    }});
                    if (prevBulkVal) bulkSelectEl.value = prevBulkVal;
                }}
            }}

            function setMediaViewMode(mode, skipSave) {{
                mediaViewMode = mode === 'list' ? 'list' : 'grid';
                const gridEl = document.getElementById('mediaGridContainer');
                const listEl = document.getElementById('mediaListViewContainer');
                const btnGrid = document.getElementById('btnMediaViewGrid');
                const btnList = document.getElementById('btnMediaViewList');
                if (gridEl) gridEl.style.display = mediaViewMode === 'grid' ? 'grid' : 'none';
                if (listEl) listEl.style.display = mediaViewMode === 'list' ? 'block' : 'none';
                if (btnGrid) btnGrid.className = mediaViewMode === 'grid' ? 'btn btn-primary' : 'btn btn-secondary';
                if (btnList) btnList.className = mediaViewMode === 'list' ? 'btn btn-primary' : 'btn btn-secondary';
                renderMediaListView();
                if (!skipSave) {{
                    try {{ localStorage.setItem('dp_media_view_mode', mediaViewMode); }} catch (_) {{}}
                    fetch('/wp-json/daypress/v1/user-preferences', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ preferences: {{ media_view_mode: mediaViewMode }} }})
                    }}).catch(() => {{}});
                }}
            }}

            function toggleMediaBulkSelectMode() {{
                mediaBulkSelectActive = !mediaBulkSelectActive;
                const btn = document.getElementById('btnMediaBulkSelect');
                const bar = document.getElementById('mediaBulkActionBar');
                if (btn) {{
                    btn.className = mediaBulkSelectActive ? 'btn btn-primary' : 'btn btn-secondary';
                    btn.textContent = mediaBulkSelectActive ? '\u2713 Bulk Select Active' : '\u2611 Bulk Select';
                }}
                if (bar) bar.style.display = mediaBulkSelectActive ? 'flex' : 'none';
                document.querySelectorAll('.media-bulk-check-wrap').forEach(el => {{
                    el.style.display = mediaBulkSelectActive ? 'block' : 'none';
                }});
                if (!mediaBulkSelectActive) {{
                    selectAllVisibleMedia(false);
                }}
            }}

            function handleMediaCardClick(event, id) {{
                if (mediaBulkSelectActive) {{
                    event.preventDefault();
                    event.stopPropagation();
                    const nextChecked = !selectedMediaIds.has(Number(id));
                    syncMediaSelection(id, nextChecked);
                }} else {{
                    openMediaDetails(id);
                }}
            }}

            function syncMediaSelection(id, isChecked) {{
                const numId = Number(id);
                if (isChecked) {{
                    selectedMediaIds.add(numId);
                }} else {{
                    selectedMediaIds.delete(numId);
                }}
                document.querySelectorAll('.media-bulk-checkbox[value="' + numId + '"]').forEach(cb => {{
                    cb.checked = isChecked;
                }});
                const card = document.getElementById('mediaCard-' + numId);
                if (card) {{
                    card.style.outline = isChecked ? '2px solid var(--press)' : 'none';
                }}
                const countEl = document.getElementById('mediaBulkSelectedCount');
                if (countEl) countEl.textContent = selectedMediaIds.size + ' selected';
                if (selectedMediaIds.size > 0 && !mediaBulkSelectActive) {{
                    const bar = document.getElementById('mediaBulkActionBar');
                    if (bar) bar.style.display = 'flex';
                }}
            }}

            function selectAllVisibleMedia(selectAll) {{
                document.querySelectorAll('.media-asset-card').forEach(card => {{
                    if (card.style.display !== 'none') {{
                        const id = Number(card.getAttribute('data-media-id'));
                        syncMediaSelection(id, selectAll);
                    }}
                }});
                if (!selectAll) {{
                    selectedMediaIds.clear();
                    document.querySelectorAll('.media-bulk-checkbox').forEach(cb => {{ cb.checked = false; }});
                    document.querySelectorAll('.media-asset-card').forEach(c => {{ c.style.outline = 'none'; }});
                    const countEl = document.getElementById('mediaBulkSelectedCount');
                    if (countEl) countEl.textContent = '0 selected';
                }}
            }}

            async function bulkMoveSelectedMedia() {{
                const folderVal = document.getElementById('bulkMediaFolderSelect')?.value || '';
                if (folderVal === '') {{
                    showToast('Choose a target folder first', true);
                    return;
                }}
                const ids = Array.from(selectedMediaIds);
                if (ids.length === 0) {{
                    showToast('Select at least one media asset first', true);
                    return;
                }}
                const targetFolderId = Number(folderVal);
                showToast('Moving ' + ids.length + ' asset(s)...');
                let updated = 0;
                for (const id of ids) {{
                    try {{
                        const res = await fetch('/wp-json/wp/v2/media/' + id, {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ folder_id: targetFolderId }})
                        }});
                        if (res.ok) {{
                            updated++;
                            if (mediaData[id]) mediaData[id].folder_id = targetFolderId > 0 ? targetFolderId : null;
                            const card = document.getElementById('mediaCard-' + id);
                            if (card) card.setAttribute('data-folder-id', String(targetFolderId));
                        }}
                    }} catch (_) {{}}
                }}
                showToast('\u2713 Moved ' + updated + ' asset(s)');
                await loadMediaFolders();
                applyMediaFilters();
            }}

            async function bulkTagSelectedMedia() {{
                const newTag = (document.getElementById('bulkMediaTagInput')?.value || '').trim();
                if (!newTag) {{
                    showToast('Enter a tag to add', true);
                    return;
                }}
                const ids = Array.from(selectedMediaIds);
                if (ids.length === 0) {{
                    showToast('Select at least one media asset first', true);
                    return;
                }}
                showToast('Tagging ' + ids.length + ' asset(s) with #' + newTag + '...');
                let updated = 0;
                for (const id of ids) {{
                    const item = mediaData[id];
                    if (!item) continue;
                    const existingTags = Array.isArray(item.tags) ? item.tags.slice() : [];
                    if (!existingTags.includes(newTag)) existingTags.push(newTag);
                    try {{
                        const res = await fetch('/wp-json/wp/v2/media/' + id, {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ tags: existingTags }})
                        }});
                        if (res.ok) updated++;
                    }} catch (_) {{}}
                }}
                showToast('\u2713 Tagged ' + updated + ' asset(s)');
                setTimeout(() => window.location.reload(), 450);
            }}

            async function bulkDeleteSelectedMedia() {{
                const ids = Array.from(selectedMediaIds);
                if (ids.length === 0) {{
                    showToast('Select at least one media asset first', true);
                    return;
                }}
                if (!confirm('Permanently delete ' + ids.length + ' selected media asset(s)?')) return;
                let deleted = 0;
                for (const id of ids) {{
                    try {{
                        const res = await fetch('/wp-json/wp/v2/media/' + id, {{ method: 'DELETE' }});
                        if (res.ok) deleted++;
                    }} catch (_) {{}}
                }}
                showToast('\u2713 Deleted ' + deleted + ' media asset(s)');
                setTimeout(() => window.location.reload(), 450);
            }}

            function renderMediaListView() {{
                const tbody = document.getElementById('mediaListTableBody');
                if (!tbody) return;
                const q = (document.getElementById('mediaTagFilterInput')?.value || '').trim().toLowerCase();
                let html = '';
                mediaList.forEach(item => {{
                    const id = Number(item.id);
                    const folderId = item.folder_id ? String(item.folder_id) : '0';
                    const tagsArr = Array.isArray(item.tags) ? item.tags : [];
                    const tagsStr = tagsArr.join(',').toLowerCase();
                    const titleStr = (item.title || '').toLowerCase();

                    const matchFolder = (activeFolderFilter === 'all') || (folderId === activeFolderFilter);
                    const matchQuery = !q || tagsStr.includes(q) || titleStr.includes(q);
                    const matchUnused = (unusedMediaIds === null) || unusedMediaIds.has(id);
                    if (!matchFolder || !matchQuery || !matchUnused) return;

                    const isImg = (item.mime_type || '').startsWith('image/');
                    const thumbHtml = isImg
                        ? '<img src="' + item.source_url + '" alt="" style="width: 44px; height: 44px; object-fit: cover; border-radius: 4px; background: var(--ink);" loading="lazy" />'
                        : '<span style="display:inline-flex;width:44px;height:44px;align-items:center;justify-content:center;background:var(--paper-sunk);border-radius:4px;font-family:var(--font-mono);font-size:10px;color:var(--ink-muted);">FILE</span>';
                    const checkedAttr = selectedMediaIds.has(id) ? 'checked' : '';
                    const sizeStr = item.file_size ? (Math.round(item.file_size / 1024) + ' KB') : '\u2014';
                    const dateStr = item.date ? item.date.substring(0, 10) : '\u2014';
                    const folderLabel = getFolderNameById(folderId);
                    const tagBadges = tagsArr.map(t => '<span style="font-size:10px;padding:1px 6px;border-radius:10px;background:var(--press-wash);color:var(--press);margin-left:4px;">#' + t + '</span>').join('');

                    html += '<tr data-media-id="' + id + '">' +
                        '<td><input type="checkbox" class="media-bulk-checkbox" value="' + id + '" ' + checkedAttr + ' onchange="syncMediaSelection(' + id + ', this.checked)" /></td>' +
                        '<td onclick="openMediaDetails(' + id + ')" style="cursor:pointer;">' + thumbHtml + '</td>' +
                        '<td onclick="openMediaDetails(' + id + ')" style="cursor:pointer;"><div style="font-weight:600;color:var(--ink);">' + (item.title || 'Untitled') + '</div><div style="font-family:var(--font-mono);font-size:11px;color:var(--ink-muted);">' + item.source_url + '</div></td>' +
                        '<td><span style="font-family:var(--font-mono);font-size:11.5px;color:var(--ink-muted);">' + (item.mime_type || '') + '</span></td>' +
                        '<td><span class="badge badge-draft" style="font-size:11px;">' + folderLabel + '</span>' + tagBadges + '</td>' +
                        '<td style="font-family:var(--font-mono);font-size:12px;color:var(--ink-muted);">' + sizeStr + '</td>' +
                        '<td style="font-size:12px;color:var(--ink-muted);">' + dateStr + '</td>' +
                        '<td style="text-align:right;white-space:nowrap;">' +
                            '<button type="button" class="btn btn-secondary" style="padding:3px 8px;font-size:11.5px;margin-right:4px;" onclick="openMediaDetails(' + id + ')">Edit</button>' +
                            '<button type="button" class="btn btn-secondary" style="padding:3px 8px;font-size:11.5px;margin-right:4px;" onclick="copyMediaUrl(\'' + item.source_url + '\')">Copy URL</button>' +
                            '<button type="button" class="btn btn-secondary" style="padding:3px 8px;font-size:11.5px;color:var(--oxide);" onclick="deleteMedia(' + id + ')">Delete</button>' +
                        '</td>' +
                    '</tr>';
                }});
                if (!html) {{
                    html = '<tr><td colspan="8" style="text-align:center;padding:32px;color:var(--ink-muted);">No matching media assets found.</td></tr>';
                }}
                tbody.innerHTML = html;
            }}

            async function createMediaFolder() {{
                const input = document.getElementById('newFolderNameInput');
                const name = input ? input.value.trim() : '';
                if (!name) {{
                    showToast('Enter a folder name first', true);
                    return;
                }}
                try {{
                    const res = await fetch('/wp-json/daypress/v1/media/folders', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ name: name, parent_id: 0 }})
                    }});
                    if (res.ok) {{
                        input.value = '';
                        showToast('✓ Created folder "' + name + '"');
                        await loadMediaFolders();
                    }} else {{
                        showToast('Failed to create media folder', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function deleteMediaFolder(folderId, folderName) {{
                if (!confirm('Delete virtual folder "' + folderName + '"? Assets inside will move to Uncategorized.')) return;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/media/folders/' + folderId, {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        showToast('✓ Deleted folder "' + folderName + '"');
                        if (String(activeFolderFilter) === String(folderId)) {{
                            activeFolderFilter = 'all';
                        }}
                        await loadMediaFolders();
                        applyMediaFilters();
                    }} else {{
                        showToast('Failed to delete folder', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            function filterByFolder(folderId) {{
                activeFolderFilter = String(folderId);
                unusedMediaIds = null;
                document.getElementById('bulkDeleteUnusedBtn').style.display = 'none';
                document.querySelectorAll('.folder-tab-btn').forEach(btn => {{
                    if (btn.getAttribute('data-folder-filter') === activeFolderFilter) {{
                        btn.className = 'btn btn-primary folder-tab-btn';
                    }} else {{
                        btn.className = 'btn btn-secondary folder-tab-btn';
                    }}
                }});
                applyMediaFilters();
            }}

            function filterByTag(tag) {{
                const input = document.getElementById('mediaTagFilterInput');
                if (!input) return;
                if (input.value.trim().toLowerCase() === tag.toLowerCase()) {{
                    input.value = '';
                }} else {{
                    input.value = tag;
                }}
                unusedMediaIds = null;
                document.getElementById('bulkDeleteUnusedBtn').style.display = 'none';
                applyMediaFilters();
            }}

            function applyMediaFilters() {{
                const q = (document.getElementById('mediaTagFilterInput')?.value || '').trim().toLowerCase();
                const cards = document.querySelectorAll('.media-asset-card');
                let visibleCount = 0;

                cards.forEach(card => {{
                    const id = Number(card.getAttribute('data-media-id'));
                    const folderId = card.getAttribute('data-folder-id') || '0';
                    const tags = (card.getAttribute('data-tags') || '').toLowerCase();
                    const item = mediaData[id];
                    const title = (item && item.title ? item.title : '').toLowerCase();

                    let matchFolder = (activeFolderFilter === 'all') || (folderId === activeFolderFilter);
                    let matchQuery = !q || tags.includes(q) || title.includes(q);
                    let matchUnused = (unusedMediaIds === null) || unusedMediaIds.has(id);

                    if (matchFolder && matchQuery && matchUnused) {{
                        card.style.display = 'flex';
                        visibleCount++;
                    }} else {{
                        card.style.display = 'none';
                    }}
                }});

                const statusEl = document.getElementById('activeFilterStatus');
                if (statusEl) {{
                    if (unusedMediaIds !== null) {{
                        statusEl.textContent = 'Showing ' + visibleCount + ' unused media asset(s)';
                    }} else {{
                        statusEl.textContent = 'Showing ' + visibleCount + ' of ' + cards.length + ' asset(s)';
                    }}
                }}
                renderMediaListView();
            }}

            async function scanUnusedMedia() {{
                showToast('Scanning posts, AST layouts, templates, and options for unused media...');
                try {{
                    const res = await fetch('/wp-json/daypress/v1/media/unused');
                    if (!res.ok) {{
                        showToast('Failed to scan unused assets', true);
                        return;
                    }}
                    const unusedList = await res.json();
                    unusedMediaIds = new Set(unusedList.map(m => m.id));
                    const badge = document.getElementById('unusedAssetsBadge');
                    if (badge) {{
                        badge.style.display = 'inline-block';
                        badge.textContent = String(unusedList.length);
                    }}
                    const bulkBtn = document.getElementById('bulkDeleteUnusedBtn');
                    if (bulkBtn) {{
                        bulkBtn.style.display = unusedList.length > 0 ? 'inline-flex' : 'none';
                    }}
                    applyMediaFilters();
                    showToast('✓ Found ' + unusedList.length + ' unused media asset(s)');
                }} catch (e) {{
                    showToast('Error scanning unused media: ' + e, true);
                }}
            }}

            async function bulkDeleteUnusedMedia() {{
                if (!unusedMediaIds || unusedMediaIds.size === 0) return;
                const ids = Array.from(unusedMediaIds);
                if (!confirm('Permanently delete ' + ids.length + ' unused media asset(s)?')) return;
                let deleted = 0;
                for (const id of ids) {{
                    try {{
                        const res = await fetch('/wp-json/wp/v2/media/' + id, {{ method: 'DELETE' }});
                        if (res.ok) deleted++;
                    }} catch (e) {{
                        console.error('Failed deleting unused media ' + id, e);
                    }}
                }}
                showToast('✓ Deleted ' + deleted + ' unused asset(s)');
                setTimeout(() => window.location.reload(), 500);
            }}

            function renderModalPreview(item, cacheBust) {{
                const pBox = document.getElementById('modalPreviewBox');
                const fxPct = Math.round((currentFocalX || 0.5) * 100);
                const fyPct = Math.round((currentFocalY || 0.5) * 100);
                document.getElementById('focalCoordsLabel').textContent = 'Focal: ' + fxPct + '%, ' + fyPct + '%';

                if (item.mime_type.startsWith('image/')) {{
                    const srcUrl = cacheBust ? (item.source_url + '?t=' + cacheBust) : item.source_url;
                    pBox.innerHTML =
                        '<img id="modalPreviewImage" src="' + srcUrl + '" alt="" style="width: 100%; height: 100%; object-fit: contain; pointer-events: none;" />' +
                        '<div id="focalPointMarker" style="position: absolute; left: ' + fxPct + '%; top: ' + fyPct + '%; transform: translate(-50%, -50%); width: 22px; height: 22px; border: 2px solid #fff; border-radius: 50%; background: rgba(59, 130, 246, 0.55); box-shadow: 0 0 0 2px rgba(15, 23, 42, 0.75); pointer-events: none;">' +
                            '<div style="position: absolute; left: 50%; top: -5px; bottom: -5px; width: 1.5px; background: #fff; transform: translateX(-50%);"></div>' +
                            '<div style="position: absolute; top: 50%; left: -5px; right: -5px; height: 1.5px; background: #fff; transform: translateY(-50%);"></div>' +
                        '</div>';
                }} else {{
                    pBox.innerHTML = '<div style="color: var(--ink-muted); font-size: 14px; font-weight: 600;">' + item.mime_type + '</div>';
                }}
            }}

            function openMediaDetails(id) {{
                const item = mediaData[id];
                if (!item) return;
                currentModalMediaId = id;
                currentFocalX = typeof item.focal_x === 'number' ? item.focal_x : 0.5;
                currentFocalY = typeof item.focal_y === 'number' ? item.focal_y : 0.5;

                document.getElementById('modalTitle').value = item.title || '';
                document.getElementById('modalAltText').value = item.alt_text || '';
                document.getElementById('modalCaption').value = item.caption || '';
                document.getElementById('modalDescription').value = item.description || '';
                document.getElementById('modalFolderSelect').value = item.folder_id ? String(item.folder_id) : '0';
                document.getElementById('modalTagsInput').value = Array.isArray(item.tags) ? item.tags.join(', ') : '';

                document.getElementById('modalFileUrlLink').href = item.source_url;
                document.getElementById('modalFileUrlLink').textContent = item.source_url;
                document.getElementById('modalMime').textContent = item.mime_type;
                document.getElementById('modalSize').textContent = item.file_size ? (Math.round(item.file_size / 1024) + ' KB') : '—';
                document.getElementById('modalDate').textContent = item.date ? item.date.substring(0, 10) : '—';

                const badgesEl = document.getElementById('modalVariantBadges');
                if (badgesEl) {{
                    let bHtml = '';
                    if (item.has_webp) {{
                        bHtml += '<a href="' + (item.webp_url || item.source_url) + '" target="_blank" style="background: rgba(16, 185, 129, 0.15); color: #059669; border: 1px solid rgba(16, 185, 129, 0.4); font-family: var(--font-mono); font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 4px; text-decoration: none;">WebP Ready</a>';
                    }}
                    if (item.has_avif) {{
                        bHtml += '<a href="' + (item.avif_url || item.source_url) + '" target="_blank" style="background: rgba(59, 130, 246, 0.15); color: #2563eb; border: 1px solid rgba(59, 130, 246, 0.4); font-family: var(--font-mono); font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 4px; text-decoration: none;">AVIF Ready</a>';
                    }}
                    badgesEl.innerHTML = bHtml;
                }}

                renderModalPreview(item, null);
                document.getElementById('mediaDetailsModal').style.display = 'flex';
            }}

            async function handleFocalPointClick(event) {{
                const id = currentModalMediaId;
                if (!id || !mediaData[id] || !mediaData[id].mime_type.startsWith('image/')) return;
                const box = document.getElementById('modalPreviewBox');
                const rect = box.getBoundingClientRect();
                if (!rect.width || !rect.height) return;

                const fx = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
                const fy = Math.max(0, Math.min(1, (event.clientY - rect.top) / rect.height));
                currentFocalX = Number(fx.toFixed(4));
                currentFocalY = Number(fy.toFixed(4));

                const marker = document.getElementById('focalPointMarker');
                const fxPct = Math.round(currentFocalX * 100);
                const fyPct = Math.round(currentFocalY * 100);
                if (marker) {{
                    marker.style.left = fxPct + '%';
                    marker.style.top = fyPct + '%';
                }}
                document.getElementById('focalCoordsLabel').textContent = 'Focal: ' + fxPct + '%, ' + fyPct + '%';

                try {{
                    const res = await fetch('/wp-json/wp/v2/media/' + id, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            focal_x: currentFocalX,
                            focal_y: currentFocalY
                        }})
                    }});
                    if (res.ok) {{
                        mediaData[id].focal_x = currentFocalX;
                        mediaData[id].focal_y = currentFocalY;
                        const gridImg = document.getElementById('gridMediaImg-' + id);
                        if (gridImg) {{
                            gridImg.style.objectPosition = fxPct + '% ' + fyPct + '%';
                        }}
                        showToast('✓ Focal point updated (' + fxPct + '%, ' + fyPct + '%)');
                    }}
                }} catch (e) {{
                    showToast('Failed to save focal point: ' + e, true);
                }}
            }}

            async function replaceCurrentMediaFile(files) {{
                const id = currentModalMediaId;
                if (!id || !files || files.length === 0) return;
                const file = files[0];
                const formData = new FormData();
                formData.append('file', file);

                showToast('Replacing media file #' + id + ' in-place...');
                try {{
                    const res = await fetch('/wp-json/daypress/v1/media/' + id + '/replace', {{
                        method: 'POST',
                        body: formData
                    }});
                    if (res.ok) {{
                        const updated = await res.json();
                        const stamp = Date.now();
                        mediaData[id].file_size = updated.file_size;
                        mediaData[id].mime_type = updated.mime_type;
                        document.getElementById('modalSize').textContent = updated.file_size ? (Math.round(updated.file_size / 1024) + ' KB') : '—';
                        document.getElementById('modalMime').textContent = updated.mime_type;
                        renderModalPreview(mediaData[id], stamp);
                        const gridImg = document.getElementById('gridMediaImg-' + id);
                        if (gridImg) {{
                            gridImg.src = updated.source_url + '?t=' + stamp;
                        }}
                        showToast('✓ Media file #' + id + ' replaced without changing URL or ID!');
                    }} else {{
                        showToast('Failed to replace media file', true);
                    }}
                }} catch (e) {{
                    showToast('Error replacing file: ' + e, true);
                }}
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
                const folder_id = Number(document.getElementById('modalFolderSelect').value || '0');
                const tagsRaw = document.getElementById('modalTagsInput').value || '';
                const tags = tagsRaw.split(',').map(t => t.trim()).filter(Boolean);

                showToast('Saving attachment metadata...');
                try {{
                    const res = await fetch('/wp-json/wp/v2/media/' + id, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            title: title,
                            alt_text: alt_text,
                            caption: caption,
                            description: description,
                            folder_id: folder_id,
                            tags: tags,
                            focal_x: currentFocalX,
                            focal_y: currentFocalY
                        }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Media attachment #' + id + ' saved to SQLite!');
                        if (mediaData[id]) {{
                            mediaData[id].title = title;
                            mediaData[id].alt_text = alt_text;
                            mediaData[id].caption = caption;
                            mediaData[id].description = description;
                            mediaData[id].folder_id = folder_id > 0 ? folder_id : null;
                            mediaData[id].tags = tags;
                            mediaData[id].focal_x = currentFocalX;
                            mediaData[id].focal_y = currentFocalY;
                        }}
                        closeMediaDetails();
                        setTimeout(() => window.location.reload(), 400);
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
                    if (activeFolderFilter !== 'all' && activeFolderFilter !== '0') {{
                        formData.append('folder_id', activeFolderFilter);
                    }}
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

            document.addEventListener('DOMContentLoaded', async () => {{
                try {{
                    const savedMode = localStorage.getItem('dp_media_view_mode');
                    if (savedMode === 'list' || savedMode === 'grid') {{
                        setMediaViewMode(savedMode, true);
                    }}
                }} catch (_) {{}}
                await loadMediaFolders();
                try {{
                    const res = await fetch('/wp-json/daypress/v1/user-preferences');
                    if (res.ok) {{
                        const data = await res.json();
                        const prefs = data && data.preferences ? data.preferences : {{}};
                        if (prefs.media_view_mode === 'list' || prefs.media_view_mode === 'grid') {{
                            setMediaViewMode(prefs.media_view_mode, true);
                        }}
                    }}
                }} catch (_) {{}}
            }});
        </script>
        "##,
        media_count = media_count,
        media_cards = media_cards,
        tag_filter_pills = tag_filter_pills,
        media_json_str = media_json_str
    );

    render_admin_shell("media", "Media Library", &body, settings)
}

pub fn render_media_new_view(settings: &DaypressSettingsResponse) -> String {
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

    render_admin_shell("media-new", "Upload New Media", body, settings)
}
