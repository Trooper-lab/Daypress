//! "Scheduled" status controls of the editor's Status & Visibility panel (`markup/schedule.rs`).
//!
//! The date/time picker is a `datetime-local` input in the editor's own time zone; the
//! editor script (`scripts/schedule.rs`) converts it to UTC and sends it as `date_gmt`.
//! Site time is UTC, so the stored `post_date` is the UTC time the post goes live.

use crate::models::DaypressPostResponse;
use crate::templates::utils::escape_html;

/// `selected` when `current` is `status`.
pub(super) fn selected_if(current: &str, status: &str) -> &'static str {
    if current == status {
        "selected"
    } else {
        ""
    }
}

/// The picker (shown while "Scheduled" is selected) and, for a post that is already
/// scheduled, the time it goes live.
pub(super) fn render_schedule_controls(existing: Option<&DaypressPostResponse>) -> String {
    let initial_status = existing.map(|p| p.status.as_str()).unwrap_or("");
    let scheduled_gmt = existing
        .filter(|p| p.status == "future")
        .map(|p| p.date.replace(' ', "T"))
        .unwrap_or_default();
    let scheduled_note = if scheduled_gmt.is_empty() {
        String::new()
    } else {
        let utc_label: String = scheduled_gmt.replace('T', " ").chars().take(16).collect();
        format!(
            r#"<div id="post_schedule_note" style="font-size: 12px; color: var(--ink); background: var(--ochre-wash); border: 1px solid var(--ochre); border-radius: 6px; padding: 6px 8px; margin-bottom: 12px;">Scheduled for <time id="post_schedule_label" datetime="{gmt}Z">{label} UTC</time></div>"#,
            gmt = escape_html(&scheduled_gmt),
            label = escape_html(&utc_label),
        )
    };
    format!(
        r#"{scheduled_note}
                    <div class="form-group" id="post_schedule_group" data-initial-status="{status}" data-scheduled-gmt="{gmt}" style="display: none;">
                        <label class="form-label" for="post_schedule_at">Publish on</label>
                        <input type="datetime-local" id="post_schedule_at" class="form-input" step="60" onchange="dpSyncScheduleUi(); markDirty();" />
                        <small style="color: var(--ink-muted); font-size: 11.5px; margin-top: 4px; display: block;">In your local time. Visitors see the post once this time has passed.</small>
                    </div>"#,
        scheduled_note = scheduled_note,
        status = escape_html(initial_status),
        gmt = escape_html(&scheduled_gmt),
    )
}
