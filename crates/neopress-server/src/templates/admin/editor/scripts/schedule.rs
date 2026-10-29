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
            const npScheduleGroup = document.getElementById('post_schedule_group');
            const npInitialStatus = npScheduleGroup ? (npScheduleGroup.dataset.initialStatus || '') : '';

            function npPad(n) {{ return String(n).padStart(2, '0'); }}

            // A Date as the value of an <input type="datetime-local"> (local time).
            function npLocalInputValue(d) {{
                return d.getFullYear() + '-' + npPad(d.getMonth() + 1) + '-' + npPad(d.getDate())
                    + 'T' + npPad(d.getHours()) + ':' + npPad(d.getMinutes());
            }}

            function npScheduledDate() {{
                const input = document.getElementById('post_schedule_at');
                if (!input || !input.value) return null;
                const d = new Date(input.value);
                return isNaN(d.getTime()) ? null : d;
            }}

            // UTC "YYYY-MM-DDTHH:MM:SS", the REST `date_gmt` format.
            function npUtcIso(d) {{ return d.toISOString().slice(0, 19); }}

            function npSyncScheduleUi() {{
                const sel = document.getElementById('post_status');
                if (!sel || !npScheduleGroup) return;
                const scheduling = sel.value === 'future';
                npScheduleGroup.style.display = scheduling ? '' : 'none';
                document.querySelectorAll('.np-publish-btn').forEach(btn => {{
                    if (!btn.dataset.defaultLabel) btn.dataset.defaultLabel = btn.textContent;
                    btn.textContent = scheduling ? 'Schedule {type_singular}' : btn.dataset.defaultLabel;
                }});
            }}

            // Extra fields for a save with `status`. Returns null to cancel an explicit save
            // without a usable time; autosaves (`quiet`) never publish or unschedule a post.
            function npScheduleOverrides(status, quiet) {{
                if (status === 'future') {{
                    const d = npScheduledDate();
                    if (d && d.getTime() > Date.now() + 60000) return {{ date_gmt: npUtcIso(d) }};
                    if (quiet) return {{ status: npInitialStatus || 'draft' }};
                    showToast('Pick a date and time at least a minute ahead to schedule this {type_lower}.', true);
                    return null;
                }}
                if (status === 'publish' && npInitialStatus === 'future') {{
                    // Choosing "Published" for a scheduled post publishes it now.
                    return quiet ? {{ status: 'future' }} : {{ date_gmt: npUtcIso(new Date()) }};
                }}
                return {{}};
            }}

            (function npInitSchedule() {{
                const sel = document.getElementById('post_status');
                const input = document.getElementById('post_schedule_at');
                const gmt = npScheduleGroup ? (npScheduleGroup.dataset.scheduledGmt || '') : '';
                if (gmt && input) {{
                    const d = new Date(gmt + 'Z');
                    if (!isNaN(d.getTime())) {{
                        input.value = npLocalInputValue(d);
                        const label = document.getElementById('post_schedule_label');
                        if (label) label.textContent = d.toLocaleString() + ' (your time)';
                    }}
                }}
                if (input) input.min = npLocalInputValue(new Date());
                if (sel) sel.addEventListener('change', npSyncScheduleUi);
                npSyncScheduleUi();
            }})();
"#,
        type_singular = type_singular,
        type_lower = type_singular.to_lowercase(),
    )
}
