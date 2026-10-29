//! Editor script for the "Scheduled" status (`scripts/schedule.rs`).
//!
//! Shows the date/time picker while "Scheduled" is selected, relabels the publish buttons,
//! and turns the picked local time into the UTC `date_gmt` that `submitData()` sends to
//! `/wp-json/wp/v2/posts`. The server stores the post as `future` and publishes it once
//! that time has passed.

pub fn render_schedule_script(type_singular: &str) -> String {
    format!(
        r#"
            // --- Scheduling ("Scheduled" status) ---
            const dpScheduleGroup = document.getElementById('post_schedule_group');
            const dpInitialStatus = dpScheduleGroup ? (dpScheduleGroup.dataset.initialStatus || '') : '';

            function dpPad(n) {{ return String(n).padStart(2, '0'); }}

            // A Date as the value of an <input type="datetime-local"> (local time).
            function dpLocalInputValue(d) {{
                return d.getFullYear() + '-' + dpPad(d.getMonth() + 1) + '-' + dpPad(d.getDate())
                    + 'T' + dpPad(d.getHours()) + ':' + dpPad(d.getMinutes());
            }}

            function dpScheduledDate() {{
                const input = document.getElementById('post_schedule_at');
                if (!input || !input.value) return null;
                const d = new Date(input.value);
                return isNaN(d.getTime()) ? null : d;
            }}

            // UTC "YYYY-MM-DDTHH:MM:SS", the REST `date_gmt` format.
            function dpUtcIso(d) {{ return d.toISOString().slice(0, 19); }}

            function dpSyncScheduleUi() {{
                const sel = document.getElementById('post_status');
                if (!sel || !dpScheduleGroup) return;
                const scheduling = sel.value === 'future';
                dpScheduleGroup.style.display = scheduling ? '' : 'none';
                document.querySelectorAll('.dp-publish-btn').forEach(btn => {{
                    if (!btn.dataset.defaultLabel) btn.dataset.defaultLabel = btn.textContent;
                    btn.textContent = scheduling ? 'Schedule {type_singular}' : btn.dataset.defaultLabel;
                }});
            }}

            // Extra fields for a save with `status`. Returns null to cancel an explicit save
            // without a usable time; autosaves (`quiet`) never publish or unschedule a post.
            function dpScheduleOverrides(status, quiet) {{
                if (status === 'future') {{
                    const d = dpScheduledDate();
                    if (d && d.getTime() > Date.now() + 60000) return {{ date_gmt: dpUtcIso(d) }};
                    if (quiet) return {{ status: dpInitialStatus || 'draft' }};
                    showToast('Pick a date and time at least a minute ahead to schedule this {type_lower}.', true);
                    return null;
                }}
                if (status === 'publish' && dpInitialStatus === 'future') {{
                    // Choosing "Published" for a scheduled post publishes it now.
                    return quiet ? {{ status: 'future' }} : {{ date_gmt: dpUtcIso(new Date()) }};
                }}
                return {{}};
            }}

            (function dpInitSchedule() {{
                const sel = document.getElementById('post_status');
                const input = document.getElementById('post_schedule_at');
                const gmt = dpScheduleGroup ? (dpScheduleGroup.dataset.scheduledGmt || '') : '';
                if (gmt && input) {{
                    const d = new Date(gmt + 'Z');
                    if (!isNaN(d.getTime())) {{
                        input.value = dpLocalInputValue(d);
                        const label = document.getElementById('post_schedule_label');
                        if (label) label.textContent = d.toLocaleString() + ' (your time)';
                    }}
                }}
                if (input) input.min = dpLocalInputValue(new Date());
                if (sel) sel.addEventListener('change', dpSyncScheduleUi);
                dpSyncScheduleUi();
            }})();
"#,
        type_singular = type_singular,
        type_lower = type_singular.to_lowercase(),
    )
}
