use super::layout::render_admin_shell;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::utils::escape_html;
use crate::theme_builder::{HeaderFooterMatrixResponse, ResolvedSlotStatus};

pub fn render_posts_view(
    posts: &[DaypressPostResponse],
    settings: &DaypressSettingsResponse,
    is_page: bool,
) -> String {
    let ptype = if is_page { "page" } else { "post" };
    render_posts_view_for_type(posts, settings, ptype)
}

pub fn render_posts_view_for_type(
    posts: &[DaypressPostResponse],
    settings: &DaypressSettingsResponse,
    post_type: &str,
) -> String {
    render_posts_view_with_matrix(posts, settings, post_type, None)
}

fn format_slot_mini_badge(prefix: &str, slot: &ResolvedSlotStatus) -> String {
    let (bg, color, border) = match slot.badge_kind.as_str() {
        "custom" => ("#eff6ff", "#1d4ed8", "#bfdbfe"),
        "global" => ("#ecfdf5", "#047857", "#a7f3d0"),
        "hidden" => ("#fef2f2", "#b91c1c", "#fecaca"),
        _ => ("#f8fafc", "#475569", "#e2e8f0"),
    };
    let name_short: String = slot.display_name.chars().take(24).collect();
    format!(
        r#"<a href="/d-admin/headers-footers?tab=matrix" title="{prefix}: {full} ({reason})" style="display:inline-flex;align-items:center;gap:4px;font-size:11px;background:{bg};color:{color};border:1px solid {border};padding:2px 7px;border-radius:5px;font-weight:600;text-decoration:none;white-space:nowrap;max-width:190px;overflow:hidden;text-overflow:ellipsis;"><span style="opacity:0.75;font-weight:700;">{prefix}:</span> {short}</a>"#,
        prefix = escape_html(prefix),
        full = escape_html(&slot.display_name),
        reason = escape_html(&slot.reason),
        bg = bg,
        color = color,
        border = border,
        short = escape_html(&name_short),
    )
}

pub fn render_posts_view_with_matrix(
    posts: &[DaypressPostResponse],
    settings: &DaypressSettingsResponse,
    post_type: &str,
    matrix: Option<&HeaderFooterMatrixResponse>,
) -> String {
    let is_page = post_type == "page";
    let formatted_cpt = crate::templates::admin::layout::format_cpt_title(post_type);
    let (post_type_label, singular) = match post_type {
        "page" => ("Pages", "Page"),
        "post" => ("Posts", "Post"),
        _ => (formatted_cpt.as_str(), formatted_cpt.as_str()),
    };
    let add_new_url = if post_type == "post" {
        "/d-admin/post-new".to_string()
    } else {
        format!("/d-admin/post-new?post_type={}", post_type)
    };

    let all_non_trash_count = posts.iter().filter(|p| p.status != "trash").count();
    let pub_count = posts.iter().filter(|p| p.status == "publish").count();
    let draft_count = posts.iter().filter(|p| p.status == "draft").count();
    let scheduled_count = posts.iter().filter(|p| p.status == "future").count();
    // Like WordPress, the "Scheduled" view only appears while something is scheduled.
    let scheduled_tab = if scheduled_count > 0 {
        format!(
            r#"<button type="button" class="btn btn-secondary status-tab" data-filter="future" onclick="filterByStatus('future', this)">Scheduled ({scheduled_count})</button>"#
        )
    } else {
        String::new()
    };
    let trash_count = posts.iter().filter(|p| p.status == "trash").count();

    let nav_slugs: std::collections::HashSet<String> = settings
        .nav_menu_items
        .iter()
        .map(|m| m.url.trim_matches('/').to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let mut ordered_posts: Vec<&DaypressPostResponse> = posts.iter().collect();
    ordered_posts.sort_by(|a, b| {
        let a_front = a.id == settings.page_on_front;
        let b_front = b.id == settings.page_on_front;
        let a_pub = a.status == "publish";
        let b_pub = b.status == "publish";
        let a_nav = a_pub && nav_slugs.contains(&a.slug.to_lowercase());
        let b_nav = b_pub && nav_slugs.contains(&b.slug.to_lowercase());
        let a_titled = !a.title.rendered.trim().is_empty();
        let b_titled = !b.title.rendered.trim().is_empty();
        b_front
            .cmp(&a_front)
            .then_with(|| b_nav.cmp(&a_nav))
            .then_with(|| b_pub.cmp(&a_pub))
            .then_with(|| b_titled.cmp(&a_titled))
            .then_with(|| b.date.cmp(&a.date))
    });

    let mut rows_html = String::new();
    for p in ordered_posts {
        let edit_link = format!(
            "/d-admin/post?post={}&action=edit&post_type={}",
            p.id,
            escape_html(&p.post_type)
        );
        let view_link = if !p.link.trim().is_empty() {
            p.link.clone()
        } else if p.id == settings.page_on_front {
            "/".to_string()
        } else if matches!(p.post_type.as_str(), "post" | "page") {
            format!("/{}/", p.slug)
        } else {
            format!("/{}/{}/", p.post_type, p.slug)
        };
        let status_badge = match p.status.as_str() {
            "draft" => r#"<span class="badge badge-draft">Draft</span>"#.to_string(),
            "trash" => {
                r#"<span class="badge" style="background:var(--oxide-wash);color:var(--oxide);border:1px solid var(--oxide);">Trash</span>"#.to_string()
            }
            "future" => {
                let when: String = p.date.replace('T', " ").chars().take(16).collect();
                format!(
                    r#"<span class="badge" style="background:var(--ochre-wash);color:var(--ink);border:1px solid var(--ochre);" title="Goes live {} UTC">Scheduled</span>"#,
                    escape_html(&when)
                )
            }
            _ => r#"<span class="badge badge-published">Published</span>"#.to_string(),
        };

        let author_display = if p.author_name.is_empty()
            || p.author_name == "Daypress Core"
            || p.author_name == "admin"
        {
            "Editor".to_string()
        } else {
            escape_html(&p.author_name)
        };
        let categories_display = if is_page {
            if let Some(m) = matrix {
                if let Some(row) = m.pages.iter().find(|r| r.post_id == p.id) {
                    format!(
                        r#"<div style="display:flex;flex-direction:column;gap:4px;align-items:flex-start;">{}{}</div>"#,
                        format_slot_mini_badge("H", &row.header),
                        format_slot_mini_badge("F", &row.footer),
                    )
                } else {
                    "Default Template".to_string()
                }
            } else if p.template == "page_templates/leeg.php" {
                r#"<span style="font-size:11.5px;color:var(--ink-muted);font-family:var(--font-mono);">leeg.php (Custom HTML)</span>"#.to_string()
            } else if p.content.rendered.contains("<section") {
                r#"<span style="font-size:11.5px;color:var(--press);font-weight:600;">ACF + Gutenberg</span>"#.to_string()
            } else {
                "Default Template".to_string()
            }
        } else if !p.category_names.is_empty() {
            escape_html(&p.category_names.join(", "))
        } else if p.template == "page_templates/leeg.php" {
            r#"<span style="font-size:11.5px;color:var(--ink-muted);font-family:var(--font-mono);">leeg.php (Custom HTML)</span>"#.to_string()
        } else if p.content.rendered.contains("<section") {
            r#"<span style="font-size:11.5px;color:var(--press);font-weight:600;">ACF + Gutenberg</span>"#.to_string()
        } else {
            "—".to_string()
        };
        let date_short: String = p.date.chars().take(10).collect();

        let mut chips = String::new();
        if !p.lang_code.is_empty() {
            let (_, _, flag, _) = crate::i18n::preset_language_info(&p.lang_code);
            chips.push_str(&format!(
                r#"<span style="display:inline-flex;align-items:center;gap:3px;font-size:10px;background:var(--paper-sunk);color:var(--ink);border:1px solid var(--rule);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:700;text-transform:uppercase;" title="Language: {}">{} {}</span>"#,
                escape_html(&p.lang_code),
                escape_html(flag),
                escape_html(&p.lang_code),
            ));
        }
        if let Some(srev_id) = p.scheduled_revision_id {
            let sched_dt = p.scheduled_revision_date.as_deref().unwrap_or("");
            chips.push_str(&format!(
                r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:var(--ochre-wash);color:var(--ink);border:1px solid var(--ochre);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:700;" title="Scheduled Revision #{} goes live {} UTC">&#9201; Scheduled Update</span>"#,
                srev_id,
                escape_html(sched_dt),
            ));
        }
        if p.id == settings.page_on_front {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:var(--moss-wash);color:var(--moss);border:1px solid var(--moss);padding:1px 7px;border-radius:4px;margin-left:6px;font-weight:700;">Front Page</span>"#);
        }
        if p.sticky {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:var(--ochre-wash);color:var(--ink);border:1px solid var(--ochre);padding:1px 7px;border-radius:4px;margin-left:6px;font-weight:700;">📌 Sticky</span>"#);
        }
        if !p.password.is_empty() {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:var(--paper-sunk);color:var(--ink-muted);border:1px solid var(--rule-strong);padding:1px 7px;border-radius:4px;margin-left:6px;font-weight:700;">🔒 Password</span>"#);
        }
        if nav_slugs.contains(&p.slug.to_lowercase()) && p.status == "publish" {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:var(--press-wash);color:var(--press);border:1px solid var(--press);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;">Main Menu</span>"#);
        }
        if !p.seo_keyword.is_empty() {
            chips.push_str(&format!(
                r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:var(--moss-wash);color:var(--moss);border:1px solid var(--moss);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;" title="Focus Keyphrase">{}</span>"#,
                escape_html(&p.seo_keyword)
            ));
        } else if p.seo_noindex {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;font-size:10.5px;background:var(--oxide-wash);color:var(--oxide);border:1px solid var(--oxide);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;">noindex</span>"#);
        }

        let display_title = if p.title.rendered.trim().is_empty() {
            format!("(Untitled — #{})", p.id)
        } else {
            p.title.rendered.clone()
        };
        let initial_row_style = if p.status == "trash" {
            "display: none;"
        } else {
            ""
        };

        let row_actions = if p.status == "trash" {
            format!(
                r##"<a href="#" onclick="restorePostRow({id}); return false;" style="color: var(--moss); font-weight: 600;">Restore</a>
                <a href="#" onclick="deletePost({id}); return false;" class="trash-action">Delete Permanently</a>"##,
                id = p.id
            )
        } else {
            format!(
                r##"<a href="{edit_link}" style="font-weight: 600;">Edit Blocks &amp; ACF</a>
                <a href="#" onclick="openQuickEditRow({id}); return false;" style="color: var(--press); font-weight: 600;">Quick Edit</a>
                <a href="/d-admin/builder?post={id}" style="color: var(--press); font-weight: 600;">Edit with DayStudio</a>
                <a href="#" onclick="duplicatePostRow({id}); return false;" style="color: var(--press); font-weight: 600;">Duplicate</a>
                <a href="{view_link}" target="_blank">View &nearr;</a>
                <a href="#" onclick="trashPostRow({id}); return false;" style="color: var(--ink);">Trash</a>
                <a href="#" onclick="deletePost({id}); return false;" class="trash-action">Delete</a>"##,
                id = p.id,
                edit_link = edit_link,
                view_link = view_link
            )
        };

        let month_short: String = p.date.chars().take(7).collect();
        let cats_csv = escape_html(&p.category_names.join(","));

        rows_html.push_str(&format!(
            r##"
            <tr id="post-{id}" data-id="{id}" data-status="{status}" data-lang="{lang}" data-title="{title_attr}" data-slug="{slug_attr}" data-date="{date_attr}" data-month="{month_attr}" data-author="{author}" data-categories="{cats_csv}" data-parent="{parent}" data-menu-order="{menu_order}" data-template="{template_attr}" data-sticky="{sticky_attr}" data-password="{password_attr}" data-comment-status="{comment_status_attr}" data-ping-status="{ping_status_attr}" style="{initial_row_style}">
                <td style="width: 32px;"><input type="checkbox" name="post[]" value="{id}"></td>
                <td class="table-title-cell">
                    <strong><a href="{edit_link}" style="color: var(--ink); text-decoration: none;">{title}</a></strong>{seo_chip}
                    <div class="table-row-slug">Permalink: {view_link}</div>
                    <div class="table-actions">
                        {actions}
                    </div>
                </td>
                <td class="col-author" style="width: 130px;"><span style="color: var(--ink-muted);">{author}</span></td>
                <td class="col-category" style="width: 160px;">{categories}</td>
                <td class="col-status" style="width: 110px;">{status_badge}</td>
                <td class="col-date" style="width: 130px; color: var(--ink-muted); font-size: 12.5px;">{date}</td>
            </tr>
            "##,
            id = p.id,
            status = escape_html(&p.status),
            lang = escape_html(if p.lang_code.is_empty() { "en" } else { &p.lang_code }),
            title_attr = escape_html(&p.title.rendered),
            slug_attr = escape_html(&p.slug),
            date_attr = escape_html(&p.date),
            month_attr = escape_html(&month_short),
            cats_csv = cats_csv,
            parent = p.parent,
            menu_order = p.menu_order,
            template_attr = escape_html(if p.template.is_empty() { "default" } else { &p.template }),
            sticky_attr = if p.sticky { "1" } else { "0" },
            password_attr = escape_html(&p.password),
            comment_status_attr = escape_html(if p.comment_status.is_empty() { "open" } else { &p.comment_status }),
            ping_status_attr = escape_html(if p.ping_status.is_empty() { "open" } else { &p.ping_status }),
            initial_row_style = initial_row_style,
            edit_link = edit_link,
            view_link = view_link,
            title = display_title,
            seo_chip = chips,
            actions = row_actions,
            author = author_display,
            categories = categories_display,
            status_badge = status_badge,
            date = escape_html(&date_short)
        ));
    }

    if posts.is_empty() {
        rows_html = format!(
            r#"<tr><td colspan="6" style="text-align: center; color: var(--ink-muted); padding: 32px;">No {} found.</td></tr>"#,
            post_type_label.to_lowercase()
        );
    }

    let fourth_col_header = if is_page {
        "Header &amp; Footer"
    } else {
        "Category"
    };
    let header_footer_btn = if is_page {
        r#"<a href="/d-admin/headers-footers" class="btn btn-secondary" style="margin-right: 8px;">Header &amp; Footer Manager</a>"#
    } else {
        ""
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">{post_type_label}</h1>
                <p class="page-subtitle">Manage, create, and organize your publication articles</p>
            </div>
            <div style="display: flex; gap: 8px; align-items: center;">
                <button type="button" class="btn btn-secondary" onclick="toggleListScreenOptions()" title="Show/hide columns &amp; items per page">⚙ Screen Options ▾</button>
                {header_footer_btn}
                <a href="{add_new_url}" class="btn btn-primary">+ Add New {singular}</a>
            </div>
        </div>

        <div id="listScreenOptionsPanel" class="day-card" style="display: none; padding: 14px 18px; margin-bottom: 16px; border: 1px solid var(--press); background: var(--paper-raised);">
            <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 14px;">
                <div style="display: flex; align-items: center; gap: 14px; flex-wrap: wrap; font-size: 12.5px;">
                    <strong style="font-size: 12px; text-transform: uppercase; color: var(--ink-muted);">Columns:</strong>
                    <label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-list-col-cb" data-col="col-author" checked onchange="togglePostsListColumn('col-author', this.checked)" /> Author</label>
                    <label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-list-col-cb" data-col="col-category" checked onchange="togglePostsListColumn('col-category', this.checked)" /> {fourth_col_header}</label>
                    <label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-list-col-cb" data-col="col-status" checked onchange="togglePostsListColumn('col-status', this.checked)" /> Status</label>
                    <label style="display:inline-flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" class="dp-list-col-cb" data-col="col-date" checked onchange="togglePostsListColumn('col-date', this.checked)" /> Date</label>
                </div>
                <div style="display: flex; align-items: center; gap: 8px; font-size: 12.5px;">
                    <strong style="font-size: 12px; text-transform: uppercase; color: var(--ink-muted);">Number of items per page:</strong>
                    <select id="listPerPageSelect" class="form-select" style="width: 90px; padding: 4px 8px; font-size: 12.5px;" onchange="onListPerPageChange(this.value)">
                        <option value="20">20</option>
                        <option value="50" selected>50</option>
                        <option value="100">100</option>
                        <option value="9999">All</option>
                    </select>
                </div>
            </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;">
            <div style="display: flex; gap: 8px; flex-wrap: wrap;" id="statusFilterTabs">
                <button type="button" class="btn btn-secondary status-tab" data-filter="all" style="background: var(--ink); color: var(--paper-raised);" onclick="filterByStatus('all', this)">All ({count})</button>
                <button type="button" class="btn btn-secondary status-tab" data-filter="publish" onclick="filterByStatus('publish', this)">Published ({pub_count})</button>
                <button type="button" class="btn btn-secondary status-tab" data-filter="draft" onclick="filterByStatus('draft', this)">Draft ({draft_count})</button>
                {scheduled_tab}
                <button type="button" class="btn btn-secondary status-tab" data-filter="trash" onclick="filterByStatus('trash', this)">Trash ({trash_count})</button>
                <button type="button" id="btnEmptyTrash" class="btn btn-secondary" style="display: none; color: var(--oxide); border-color: var(--oxide); font-weight: 600;" onclick="emptyTrashForPostType()">🗑 Empty Trash</button>
            </div>
            <div style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap;">
                <div style="display: flex; gap: 6px; align-items: center;">
                    <select id="bulkActionSelect" class="form-input" style="width: auto; padding: 6px 10px; font-size: 13px;">
                        <option value="">Bulk actions</option>
                        <option value="edit">Edit (Bulk Edit)</option>
                        <option value="trash">Move to Trash</option>
                        <option value="restore">Restore</option>
                        <option value="delete">Delete Permanently</option>
                    </select>
                    <button type="button" class="btn btn-secondary" style="padding: 6px 12px; font-size: 13px;" onclick="applyBulkActions()">Apply</button>
                </div>
                <select id="monthFilterSelect" class="form-input" style="width: auto; padding: 6px 10px; font-size: 13px;" onchange="filterRows(document.getElementById('postSearchInput').value)">
                    <option value="all">📅 All Dates</option>
                </select>
                <select id="catFilterSelect" class="form-input" style="width: auto; padding: 6px 10px; font-size: 13px;" onchange="filterRows(document.getElementById('postSearchInput').value)">
                    <option value="all">🏷 All Categories</option>
                </select>
                <select id="authorFilterSelect" class="form-input" style="width: auto; padding: 6px 10px; font-size: 13px;" onchange="filterRows(document.getElementById('postSearchInput').value)">
                    <option value="all">👤 All Authors</option>
                </select>
                <select id="langFilterSelect" class="form-input" style="width: auto; padding: 6px 10px; font-size: 13px;" onchange="filterRows(document.getElementById('postSearchInput').value)">
                    <option value="all">🌐 All Languages</option>
                    <option value="en">🇺🇸 EN</option>
                    <option value="de">🇩🇪 DE</option>
                    <option value="nl">🇳🇱 NL</option>
                    <option value="fr">🇫🇷 FR</option>
                    <option value="es">🇪🇸 ES</option>
                </select>
                <input type="text" id="postSearchInput" placeholder="Search {plural_lower}..." class="form-input" style="width: 200px; padding: 6px 12px; font-size: 13px;" oninput="filterRows(this.value)" />
            </div>
        </div>

        <div id="bulkEditDrawer" class="day-card" style="display: none; padding: 18px 20px; margin-bottom: 16px; border: 1px solid var(--press); background: var(--paper-raised);">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; padding-bottom: 10px; border-bottom: 1px solid var(--rule);">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <strong style="font-size: 14px; color: var(--ink);">Bulk Edit Selected {post_type_label}</strong>
                    <span id="bulkEditCountBadge" class="badge badge-published">0 selected</span>
                </div>
                <button type="button" class="btn btn-secondary" style="height: 26px; padding: 0 10px; font-size: 12px;" onclick="closeBulkEditDrawer()">Cancel</button>
            </div>
            <div style="display: grid; grid-template-columns: 1fr 2fr; gap: 18px; align-items: start;">
                <div>
                    <div style="font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--ink-muted); margin-bottom: 6px;">Items in Batch</div>
                    <div id="bulkEditSelectedPills" style="display: flex; flex-direction: column; gap: 5px; max-height: 150px; overflow-y: auto; padding: 8px; border: 1px solid var(--rule); border-radius: 6px; background: var(--paper-sunk); font-size: 12px;"></div>
                </div>
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px;">
                    <div>
                        <label class="form-label" style="font-size: 11.5px;">Status</label>
                        <select id="bulkEditStatus" class="form-select" style="font-size: 12.5px; padding: 6px 10px;">
                            <option value="-1">— No Change —</option>
                            <option value="publish">Published</option>
                            <option value="draft">Draft</option>
                            <option value="pending">Pending Review</option>
                            <option value="private">Private</option>
                        </select>
                    </div>
                    <div>
                        <label class="form-label" style="font-size: 11.5px;">Template</label>
                        <select id="bulkEditTemplate" class="form-select" style="font-size: 12.5px; padding: 6px 10px;">
                            <option value="-1">— No Change —</option>
                            <option value="default">Default Template</option>
                            <option value="full-width">Full-Width Clean Canvas</option>
                            <option value="canvas">DayStudio Canvas</option>
                            <option value="landing">Landing Page Hero</option>
                            <option value="editorial">Editorial Longform</option>
                        </select>
                    </div>
                    <div>
                        <label class="form-label" style="font-size: 11.5px;">Sticky</label>
                        <select id="bulkEditSticky" class="form-select" style="font-size: 12.5px; padding: 6px 10px;">
                            <option value="-1">— No Change —</option>
                            <option value="true">Sticky</option>
                            <option value="false">Not Sticky</option>
                        </select>
                    </div>
                    <div>
                        <label class="form-label" style="font-size: 11.5px;">Comments</label>
                        <select id="bulkEditComments" class="form-select" style="font-size: 12.5px; padding: 6px 10px;">
                            <option value="-1">— No Change —</option>
                            <option value="open">Allow</option>
                            <option value="closed">Do not allow</option>
                        </select>
                    </div>
                    <div>
                        <label class="form-label" style="font-size: 11.5px;">Pings</label>
                        <select id="bulkEditPings" class="form-select" style="font-size: 12.5px; padding: 6px 10px;">
                            <option value="-1">— No Change —</option>
                            <option value="open">Allow</option>
                            <option value="closed">Do not allow</option>
                        </select>
                    </div>
                    <div>
                        <label class="form-label" style="font-size: 11.5px;">Add Tags (comma-separated)</label>
                        <input type="text" id="bulkEditAddTags" class="form-input" placeholder="e.g. featured, v1" style="font-size: 12.5px; padding: 6px 10px;" />
                    </div>
                </div>
            </div>
            <div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--rule);">
                <button type="button" class="btn btn-secondary" onclick="closeBulkEditDrawer()">Cancel</button>
                <button type="button" class="btn btn-primary" onclick="submitBulkEdit()">Update Selected {post_type_label}</button>
            </div>
        </div>

        <div class="day-table-wrapper">
            <table class="day-table" id="postsTable">
                <thead>
                    <tr>
                        <th style="width: 32px;"><input type="checkbox" id="selectAllPosts" onchange="toggleSelectAll(this)"></th>
                        <th style="cursor: pointer; user-select: none;" onclick="sortPostsTable('title')">Title <span id="sortInd-title" style="font-size: 11px; color: var(--ink-muted);">↕</span></th>
                        <th class="col-author" style="cursor: pointer; user-select: none;" onclick="sortPostsTable('author')">Author <span id="sortInd-author" style="font-size: 11px; color: var(--ink-muted);">↕</span></th>
                        <th class="col-category">{fourth_col_header}</th>
                        <th class="col-status">Status</th>
                        <th class="col-date" style="cursor: pointer; user-select: none;" onclick="sortPostsTable('date')">Date <span id="sortInd-date" style="font-size: 11px; color: var(--ink-muted);">↓</span></th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
            </table>
        </div>

        <script>
            let activeStatusFilter = 'all';
            const currentListPostType = '{post_type_slug}';
            let currentSortCol = 'date';
            let currentSortAsc = false;
            let listScreenState = {{ hiddenCols: {{}}, perPage: 50 }};

            function toggleSelectAll(master) {{
                document.querySelectorAll('#postsTable tbody input[name="post[]"]').forEach(cb => {{
                    const row = cb.closest('tr');
                    if (row && row.style.display !== 'none') {{
                        cb.checked = master.checked;
                    }}
                }});
            }}

            function filterByStatus(status, btn) {{
                activeStatusFilter = status;
                document.querySelectorAll('#statusFilterTabs .status-tab').forEach(b => {{
                    b.style.background = '';
                    b.style.color = '';
                }});
                if (btn) {{
                    btn.style.background = 'var(--ink)';
                    btn.style.color = 'var(--paper-raised)';
                }}
                const emptyBtn = document.getElementById('btnEmptyTrash');
                if (emptyBtn) {{
                    emptyBtn.style.display = (status === 'trash' || ({trash_count} > 0 && status === 'trash')) ? 'inline-flex' : 'none';
                }}
                const master = document.getElementById('selectAllPosts');
                if (master) master.checked = false;
                const q = document.getElementById('postSearchInput') ? document.getElementById('postSearchInput').value : '';
                filterRows(q);
            }}

            function openQuickEditRow(id) {{
                const existing = document.getElementById('quick-edit-' + id);
                if (existing) {{
                    existing.remove();
                    return;
                }}
                document.querySelectorAll('tr.dp-quick-edit-row').forEach(el => el.remove());
                const row = document.getElementById('post-' + id);
                if (!row) return;

                const title = row.getAttribute('data-title') || '';
                const slug = row.getAttribute('data-slug') || '';
                const dateRaw = (row.getAttribute('data-date') || '').slice(0, 16);
                const status = row.getAttribute('data-status') || 'publish';
                const parent = row.getAttribute('data-parent') || '0';
                const menuOrder = row.getAttribute('data-menu-order') || '0';
                const tpl = row.getAttribute('data-template') || 'default';
                const sticky = row.getAttribute('data-sticky') === '1';
                const password = row.getAttribute('data-password') || '';
                const commentStatus = row.getAttribute('data-comment-status') || 'open';
                const pingStatus = row.getAttribute('data-ping-status') || 'open';

                const qeTr = document.createElement('tr');
                qeTr.id = 'quick-edit-' + id;
                qeTr.className = 'dp-quick-edit-row';
                qeTr.style.background = 'var(--paper-raised)';
                qeTr.innerHTML = `
                    <td colspan="6" style="padding: 16px 20px; border-top: 2px solid var(--press); border-bottom: 2px solid var(--press);">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                            <strong style="font-size: 13px; color: var(--press); text-transform: uppercase; letter-spacing: 0.04em;">⚡ Quick Edit — #${{id}}</strong>
                            <span style="font-size: 11.5px; color: var(--ink-muted);">Saves directly via <code>PUT /wp-json/wp/v2/posts/${{id}}</code></span>
                        </div>
                        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px;">
                            <div>
                                <div style="margin-bottom: 8px;">
                                    <label class="form-label" style="font-size: 11.5px;">Title</label>
                                    <input type="text" id="qe-title-${{id}}" class="form-input" value="${{title.replace(/"/g, '&quot;')}}" style="font-size: 12.5px; padding: 6px 10px;" />
                                </div>
                                <div style="margin-bottom: 8px;">
                                    <label class="form-label" style="font-size: 11.5px;">Slug</label>
                                    <input type="text" id="qe-slug-${{id}}" class="form-input" value="${{slug.replace(/"/g, '&quot;')}}" style="font-size: 12.5px; font-family: var(--font-mono); padding: 6px 10px;" />
                                </div>
                                <div>
                                    <label class="form-label" style="font-size: 11.5px;">Date</label>
                                    <input type="datetime-local" id="qe-date-${{id}}" class="form-input" value="${{dateRaw}}" style="font-size: 12.5px; padding: 6px 10px;" />
                                </div>
                            </div>
                            <div>
                                <div style="margin-bottom: 8px;">
                                    <label class="form-label" style="font-size: 11.5px;">Status</label>
                                    <select id="qe-status-${{id}}" class="form-select" style="font-size: 12.5px; padding: 6px 10px;">
                                        <option value="publish" ${{status === 'publish' ? 'selected' : ''}}>Published</option>
                                        <option value="draft" ${{status === 'draft' ? 'selected' : ''}}>Draft</option>
                                        <option value="pending" ${{status === 'pending' ? 'selected' : ''}}>Pending Review</option>
                                        <option value="private" ${{status === 'private' ? 'selected' : ''}}>Private</option>
                                        <option value="future" ${{status === 'future' ? 'selected' : ''}}>Scheduled</option>
                                    </select>
                                </div>
                                <div style="margin-bottom: 8px;">
                                    <label class="form-label" style="font-size: 11.5px;">Password <span style="font-weight:400;color:var(--ink-muted);">(or leave blank)</span></label>
                                    <input type="text" id="qe-password-${{id}}" class="form-input" value="${{password.replace(/"/g, '&quot;')}}" placeholder="Public" style="font-size: 12.5px; padding: 6px 10px;" />
                                </div>
                                <div style="display: flex; gap: 14px; padding-top: 6px;">
                                    <label style="display:inline-flex;align-items:center;gap:6px;font-size:12.5px;cursor:pointer;">
                                        <input type="checkbox" id="qe-sticky-${{id}}" ${{sticky ? 'checked' : ''}} /> Make this post sticky
                                    </label>
                                </div>
                            </div>
                            <div>
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 8px;">
                                    <div>
                                        <label class="form-label" style="font-size: 11.5px;">Parent ID</label>
                                        <input type="number" id="qe-parent-${{id}}" class="form-input" value="${{parent}}" style="font-size: 12.5px; padding: 6px 10px;" />
                                    </div>
                                    <div>
                                        <label class="form-label" style="font-size: 11.5px;">Order</label>
                                        <input type="number" id="qe-order-${{id}}" class="form-input" value="${{menuOrder}}" style="font-size: 12.5px; padding: 6px 10px;" />
                                    </div>
                                </div>
                                <div style="margin-bottom: 8px;">
                                    <label class="form-label" style="font-size: 11.5px;">Template</label>
                                    <select id="qe-template-${{id}}" class="form-select" style="font-size: 12.5px; padding: 6px 10px;">
                                        <option value="default" ${{tpl === 'default' ? 'selected' : ''}}>Default Template</option>
                                        <option value="full-width" ${{tpl === 'full-width' ? 'selected' : ''}}>Full-Width Clean Canvas</option>
                                        <option value="canvas" ${{tpl === 'canvas' ? 'selected' : ''}}>DayStudio Canvas</option>
                                        <option value="landing" ${{tpl === 'landing' ? 'selected' : ''}}>Landing Page Hero</option>
                                        <option value="editorial" ${{tpl === 'editorial' ? 'selected' : ''}}>Editorial Longform</option>
                                    </select>
                                </div>
                                <div style="display: flex; gap: 14px; padding-top: 6px;">
                                    <label style="display:inline-flex;align-items:center;gap:6px;font-size:12.5px;cursor:pointer;">
                                        <input type="checkbox" id="qe-comments-${{id}}" ${{commentStatus !== 'closed' ? 'checked' : ''}} /> Allow Comments
                                    </label>
                                    <label style="display:inline-flex;align-items:center;gap:6px;font-size:12.5px;cursor:pointer;">
                                        <input type="checkbox" id="qe-pings-${{id}}" ${{pingStatus !== 'closed' ? 'checked' : ''}} /> Allow Pings
                                    </label>
                                </div>
                            </div>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--rule);">
                            <button type="button" class="btn btn-secondary" onclick="document.getElementById('quick-edit-${{id}}')?.remove()">Cancel</button>
                            <button type="button" class="btn btn-primary" onclick="saveQuickEditRow(${{id}})">Update</button>
                        </div>
                    </td>
                `;
                row.insertAdjacentElement('afterend', qeTr);
            }}

            async function saveQuickEditRow(id) {{
                const title = document.getElementById('qe-title-' + id)?.value.trim() || 'Untitled';
                const slug = document.getElementById('qe-slug-' + id)?.value.trim() || undefined;
                const dateVal = document.getElementById('qe-date-' + id)?.value || '';
                const status = document.getElementById('qe-status-' + id)?.value || 'publish';
                const password = document.getElementById('qe-password-' + id)?.value || '';
                const sticky = !!document.getElementById('qe-sticky-' + id)?.checked;
                const parent = parseInt(document.getElementById('qe-parent-' + id)?.value || '0', 10) || 0;
                const menu_order = parseInt(document.getElementById('qe-order-' + id)?.value || '0', 10) || 0;
                const template = document.getElementById('qe-template-' + id)?.value || 'default';
                const comment_status = document.getElementById('qe-comments-' + id)?.checked ? 'open' : 'closed';
                const ping_status = document.getElementById('qe-pings-' + id)?.checked ? 'open' : 'closed';

                const payload = {{
                    title,
                    slug,
                    status,
                    password,
                    sticky,
                    parent,
                    menu_order,
                    template,
                    comment_status,
                    ping_status,
                    date: dateVal ? (dateVal.length === 16 ? dateVal + ':00' : dateVal) : undefined
                }};
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + id, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});
                    if (res.ok) {{
                        showToast('✓ Quick Edit saved for #' + id);
                        setTimeout(() => window.location.reload(), 400);
                    }} else {{
                        showToast('Failed to save Quick Edit', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function duplicatePostRow(id) {{
                showToast('Cloning #' + id + ' (AST layout, metadata & terms)...');
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/' + id + '/duplicate', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    if (res.ok) {{
                        const cloned = await res.json();
                        showToast('✓ Cloned as Draft #' + cloned.id + ': ' + (cloned.title && cloned.title.rendered ? cloned.title.rendered : 'Copy'));
                        setTimeout(() => window.location.reload(), 550);
                    }} else {{
                        showToast('Error duplicating item', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function restorePostRow(id) {{
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + id, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ status: 'draft' }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Restored #' + id + ' as Draft');
                        setTimeout(() => window.location.reload(), 500);
                    }} else {{
                        showToast('Error restoring item', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function trashPostRow(id) {{
                const res = await fetch('/wp-json/wp/v2/posts/' + id, {{
                    method: 'PUT',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ status: 'trash' }})
                }});
                if (res.ok) {{
                    showToast('✓ Moved #' + id + ' to Trash');
                    setTimeout(() => window.location.reload(), 500);
                }} else {{
                    showToast('Error trashing item', true);
                }}
            }}

            async function deletePost(id) {{
                if (!confirm('Permanently delete #' + id + '?')) return;
                const res = await fetch('/wp-json/wp/v2/posts/' + id + '?force=true', {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('✓ Permanently deleted');
                    const row = document.getElementById('post-' + id);
                    if (row) row.remove();
                }} else {{
                    showToast('Error deleting post', true);
                }}
            }}

            async function emptyTrashForPostType() {{
                if (!confirm('Permanently delete all items in Trash? This action cannot be undone.')) return;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/empty-trash', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ post_type: currentListPostType }})
                    }});
                    if (res.ok) {{
                        const data = await res.json();
                        showToast('✓ Emptied Trash (' + (data.deleted_count || 0) + ' deleted)');
                        setTimeout(() => window.location.reload(), 450);
                    }} else {{
                        showToast('Failed to empty trash', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            function openBulkEditDrawer(checkedBoxes) {{
                const drawer = document.getElementById('bulkEditDrawer');
                const pills = document.getElementById('bulkEditSelectedPills');
                const badge = document.getElementById('bulkEditCountBadge');
                if (!drawer || !pills) return;
                badge.textContent = checkedBoxes.length + ' selected';
                pills.innerHTML = checkedBoxes.map(cb => {{
                    const row = cb.closest('tr');
                    const title = (row && row.getAttribute('data-title')) || ('#' + cb.value);
                    return `<div style="display:flex;justify-content:space-between;align-items:center;gap:6px;background:var(--paper-raised);padding:4px 8px;border-radius:4px;border:1px solid var(--rule);">
                        <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">#${{cb.value}} — ${{title}}</span>
                        <button type="button" onclick="uncheckBulkEditItem(${{cb.value}})" style="border:none;background:none;color:var(--oxide);cursor:pointer;font-size:12px;">✕</button>
                    </div>`;
                }}).join('');
                drawer.style.display = 'block';
            }}

            function uncheckBulkEditItem(id) {{
                const cb = document.querySelector(`#postsTable tbody input[name="post[]"][value="${{id}}"]`);
                if (cb) cb.checked = false;
                const checked = Array.from(document.querySelectorAll('#postsTable tbody input[name="post[]"]:checked'));
                if (checked.length === 0) closeBulkEditDrawer();
                else openBulkEditDrawer(checked);
            }}

            function closeBulkEditDrawer() {{
                const drawer = document.getElementById('bulkEditDrawer');
                if (drawer) drawer.style.display = 'none';
            }}

            async function submitBulkEdit() {{
                const checked = Array.from(document.querySelectorAll('#postsTable tbody input[name="post[]"]:checked'));
                const post_ids = checked.map(cb => parseInt(cb.value, 10)).filter(n => !isNaN(n));
                if (post_ids.length === 0) {{
                    showToast('No items selected', true);
                    return;
                }}
                const statusVal = document.getElementById('bulkEditStatus')?.value || '-1';
                const tplVal = document.getElementById('bulkEditTemplate')?.value || '-1';
                const stickyVal = document.getElementById('bulkEditSticky')?.value || '-1';
                const commentsVal = document.getElementById('bulkEditComments')?.value || '-1';
                const pingsVal = document.getElementById('bulkEditPings')?.value || '-1';
                const addTagsVal = document.getElementById('bulkEditAddTags')?.value.trim() || '';

                const payload = {{
                    post_ids,
                    status: statusVal !== '-1' ? statusVal : undefined,
                    template: tplVal !== '-1' ? tplVal : undefined,
                    sticky: stickyVal === 'true' ? true : (stickyVal === 'false' ? false : undefined),
                    comment_status: commentsVal !== '-1' ? commentsVal : undefined,
                    ping_status: pingsVal !== '-1' ? pingsVal : undefined,
                    add_tags: addTagsVal || undefined
                }};
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/bulk-edit', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});
                    if (res.ok) {{
                        const data = await res.json();
                        showToast('✓ Bulk updated ' + (data.updated_count || post_ids.length) + ' item(s)');
                        setTimeout(() => window.location.reload(), 450);
                    }} else {{
                        showToast('Bulk Edit failed', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function applyBulkActions() {{
                const select = document.getElementById('bulkActionSelect');
                const action = select ? select.value : '';
                if (!action) {{
                    showToast('Please select a bulk action.', true);
                    return;
                }}
                const checked = Array.from(document.querySelectorAll('#postsTable tbody input[name="post[]"]:checked'));
                if (checked.length === 0) {{
                    showToast('No items selected.', true);
                    return;
                }}
                if (action === 'edit') {{
                    openBulkEditDrawer(checked);
                    return;
                }}
                if (action === 'delete' && !confirm('Permanently delete ' + checked.length + ' item(s)?')) {{
                    return;
                }}

                showToast('Applying ' + action + ' to ' + checked.length + ' item(s)...');
                let count = 0;
                for (const cb of checked) {{
                    const id = cb.value;
                    try {{
                        let res;
                        if (action === 'trash') {{
                            res = await fetch('/wp-json/wp/v2/posts/' + id, {{
                                method: 'PUT',
                                headers: {{ 'Content-Type': 'application/json' }},
                                body: JSON.stringify({{ status: 'trash' }})
                            }});
                        }} else if (action === 'restore') {{
                            res = await fetch('/wp-json/wp/v2/posts/' + id, {{
                                method: 'PUT',
                                headers: {{ 'Content-Type': 'application/json' }},
                                body: JSON.stringify({{ status: 'draft' }})
                            }});
                        }} else if (action === 'delete') {{
                            res = await fetch('/wp-json/wp/v2/posts/' + id + '?force=true', {{
                                method: 'DELETE'
                            }});
                        }}
                        if (res && res.ok) count++;
                    }} catch (e) {{
                        console.error(e);
                    }}
                }}
                showToast('✓ Updated ' + count + ' item(s)');
                setTimeout(() => window.location.reload(), 600);
            }}

            function sortPostsTable(col) {{
                if (currentSortCol === col) {{
                    currentSortAsc = !currentSortAsc;
                }} else {{
                    currentSortCol = col;
                    currentSortAsc = col !== 'date';
                }}
                ['title', 'author', 'date'].forEach(c => {{
                    const ind = document.getElementById('sortInd-' + c);
                    if (ind) ind.textContent = c === currentSortCol ? (currentSortAsc ? '↑' : '↓') : '↕';
                }});
                const tbody = document.querySelector('#postsTable tbody');
                if (!tbody) return;
                document.querySelectorAll('tr.dp-quick-edit-row').forEach(el => el.remove());
                const rows = Array.from(tbody.querySelectorAll('tr[data-status]'));
                rows.sort((a, b) => {{
                    const av = (a.getAttribute('data-' + col) || '').toLowerCase();
                    const bv = (b.getAttribute('data-' + col) || '').toLowerCase();
                    if (av < bv) return currentSortAsc ? -1 : 1;
                    if (av > bv) return currentSortAsc ? 1 : -1;
                    return 0;
                }});
                rows.forEach(r => tbody.appendChild(r));
            }}

            function filterRows(q) {{
                const filter = (q || '').toLowerCase();
                const langSel = document.getElementById('langFilterSelect');
                const activeLang = langSel ? (langSel.value || 'all').toLowerCase() : 'all';
                const monthSel = document.getElementById('monthFilterSelect');
                const activeMonth = monthSel ? (monthSel.value || 'all') : 'all';
                const catSel = document.getElementById('catFilterSelect');
                const activeCat = catSel ? (catSel.value || 'all').toLowerCase() : 'all';
                const authorSel = document.getElementById('authorFilterSelect');
                const activeAuthor = authorSel ? (authorSel.value || 'all').toLowerCase() : 'all';
                const maxPerPage = parseInt(listScreenState.perPage || 50, 10) || 50;

                const rows = document.querySelectorAll('#postsTable tbody tr[data-status]');
                let shownCount = 0;
                rows.forEach(r => {{
                    const text = r.textContent.toLowerCase();
                    const rowStatus = r.getAttribute('data-status') || 'publish';
                    const rowLang = (r.getAttribute('data-lang') || 'en').toLowerCase();
                    const rowMonth = r.getAttribute('data-month') || '';
                    const rowCats = (r.getAttribute('data-categories') || '').toLowerCase();
                    const rowAuthor = (r.getAttribute('data-author') || '').toLowerCase();

                    const statusMatch = activeStatusFilter === 'all' ? (rowStatus !== 'trash') : (rowStatus === activeStatusFilter);
                    const langMatch = activeLang === 'all' || rowLang === activeLang;
                    const monthMatch = activeMonth === 'all' || rowMonth === activeMonth;
                    const catMatch = activeCat === 'all' || rowCats.split(',').map(s => s.trim()).includes(activeCat);
                    const authorMatch = activeAuthor === 'all' || rowAuthor === activeAuthor;
                    const textMatch = !filter || text.includes(filter);

                    if (statusMatch && langMatch && monthMatch && catMatch && authorMatch && textMatch) {{
                        shownCount++;
                        r.style.display = shownCount <= maxPerPage ? '' : 'none';
                    }} else {{
                        r.style.display = 'none';
                    }}
                }});
            }}

            function toggleListScreenOptions() {{
                const p = document.getElementById('listScreenOptionsPanel');
                if (p) p.style.display = (p.style.display === 'none' || !p.style.display) ? 'block' : 'none';
            }}

            function togglePostsListColumn(colClass, visible) {{
                listScreenState.hiddenCols[colClass] = !visible;
                document.querySelectorAll('#postsTable .' + colClass).forEach(el => {{
                    el.style.display = visible ? '' : 'none';
                }});
                savePostsListScreenState();
            }}

            function onListPerPageChange(val) {{
                listScreenState.perPage = parseInt(val, 10) || 50;
                filterRows(document.getElementById('postSearchInput')?.value || '');
                savePostsListScreenState();
            }}

            function savePostsListScreenState() {{
                fetch('/wp-json/daypress/v1/user-preferences', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ screens: {{ posts_list: listScreenState }} }})
                }}).catch(() => {{}});
            }}

            document.addEventListener('DOMContentLoaded', async () => {{
                // Populate dynamic Month, Category, and Author filter dropdowns from table rows
                const months = new Set();
                const cats = new Set();
                const authors = new Set();
                document.querySelectorAll('#postsTable tbody tr[data-status]').forEach(r => {{
                    const m = r.getAttribute('data-month');
                    if (m && m.length === 7) months.add(m);
                    (r.getAttribute('data-categories') || '').split(',').map(s => s.trim()).filter(Boolean).forEach(c => cats.add(c));
                    const a = r.getAttribute('data-author');
                    if (a) authors.add(a);
                }});
                const monthSel = document.getElementById('monthFilterSelect');
                if (monthSel && months.size > 0) {{
                    Array.from(months).sort().reverse().forEach(m => {{
                        const opt = document.createElement('option');
                        opt.value = m; opt.textContent = '📅 ' + m;
                        monthSel.appendChild(opt);
                    }});
                }}
                const catSel = document.getElementById('catFilterSelect');
                if (catSel && cats.size > 0) {{
                    Array.from(cats).sort().forEach(c => {{
                        const opt = document.createElement('option');
                        opt.value = c.toLowerCase(); opt.textContent = '🏷 ' + c;
                        catSel.appendChild(opt);
                    }});
                }}
                const authorSel = document.getElementById('authorFilterSelect');
                if (authorSel && authors.size > 0) {{
                    Array.from(authors).sort().forEach(a => {{
                        const opt = document.createElement('option');
                        opt.value = a.toLowerCase(); opt.textContent = '👤 ' + a;
                        authorSel.appendChild(opt);
                    }});
                }}

                fetch('/wp-json/daypress/v1/user-preferences')
                    .then(r => r.ok ? r.json() : null)
                    .then(data => {{
                        const prefs = data && data.preferences && data.preferences.screens && data.preferences.screens.posts_list;
                        if (prefs && typeof prefs === 'object') {{
                            listScreenState = Object.assign(listScreenState, prefs);
                            Object.entries(listScreenState.hiddenCols || {{}}).forEach(([colClass, hidden]) => {{
                                const cb = document.querySelector(`.dp-list-col-cb[data-col="${{colClass}}"]`);
                                if (cb) cb.checked = !hidden;
                                document.querySelectorAll('#postsTable .' + colClass).forEach(el => {{
                                    el.style.display = hidden ? 'none' : '';
                                }});
                            }});
                            const ppSel = document.getElementById('listPerPageSelect');
                            if (ppSel && listScreenState.perPage) ppSel.value = String(listScreenState.perPage);
                            filterRows(document.getElementById('postSearchInput')?.value || '');
                        }}
                    }})
                    .catch(() => {{}});

                const langSel = document.getElementById('langFilterSelect');
                if (!langSel) return;
                const urlParams = new URLSearchParams(window.location.search);
                const initLang = (urlParams.get('lang') || 'all').toLowerCase();
                try {{
                    const res = await fetch('/wp-json/daypress/v1/languages');
                    if (res.ok) {{
                        const data = await res.json();
                        const langs = data.languages || [];
                        if (langs.length > 0) {{
                            langSel.innerHTML = '<option value="all">🌐 All Languages</option>' +
                                langs.map(l =>
                                    '<option value="' + l.code + '"' + (l.code === initLang ? ' selected' : '') + '>' +
                                    (l.flag_emoji || '🌐') + ' ' + l.code.toUpperCase() + ' (' + l.native_name + ')</option>'
                                ).join('');
                        }}
                    }}
                }} catch (_) {{}}
                if (initLang !== 'all') {{
                    langSel.value = initLang;
                    filterRows('');
                }}
            }});
        </script>
        "##,
        post_type_label = post_type_label,
        post_type_slug = escape_html(post_type),
        header_footer_btn = header_footer_btn,
        fourth_col_header = fourth_col_header,
        add_new_url = add_new_url,
        singular = singular,
        plural_lower = post_type_label.to_lowercase(),
        count = all_non_trash_count,
        pub_count = pub_count,
        draft_count = draft_count,
        scheduled_tab = scheduled_tab,
        trash_count = trash_count,
        rows_html = rows_html
    );

    let tab = if is_page {
        "pages"
    } else if post_type == "post" {
        "posts"
    } else {
        post_type
    };
    render_admin_shell(tab, post_type_label, &body, settings)
}
