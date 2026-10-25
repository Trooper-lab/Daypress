use super::layout::render_wp_admin_shell;
use crate::models::{
    DiscussionSettings, MediaSettings, PermalinkSettings, PrivacySettings, ReadingSettings,
    WpCategoryResponse, WpPostResponse, WpSettingsResponse, WritingSettings,
};
use crate::templates::utils::escape_html;

pub fn render_wp_settings_view(settings: &WpSettingsResponse) -> String {
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">General Settings</h1>
                <p class="page-subtitle">Configure site identity, URLs, and administration preferences</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('settingsFormSubmit').click()">Save Changes</button>
            </div>
        </div>

        <form id="settingsForm" onsubmit="saveSettings(event)">
            <button type="submit" id="settingsFormSubmit" style="display: none;"></button>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header">
                    <h2 class="neo-card-title">Site Identity</h2>
                </div>
                <div class="neo-card-body">
                    <div class="form-group">
                        <label class="form-label" for="blogname">Site Title</label>
                        <input name="blogname" type="text" id="blogname" value="{title}" class="form-input" style="max-width: 480px;" required>
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="blogdescription">Tagline</label>
                        <input name="blogdescription" type="text" id="blogdescription" value="{desc}" class="form-input" style="max-width: 480px;">
                        <p class="form-hint">In a few words, explain what this site is about.</p>
                    </div>
                </div>
            </div>

            <div class="neo-card">
                <div class="neo-card-header">
                    <h2 class="neo-card-title">Addresses &amp; Administration</h2>
                </div>
                <div class="neo-card-body">
                    <div class="form-group">
                        <label class="form-label" for="siteurl">Site Address (URL)</label>
                        <input name="siteurl" type="text" id="siteurl" value="{url}" class="form-input" style="max-width: 480px; font-family: var(--font-mono);">
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="admin_email">Administration Email Address</label>
                        <input name="admin_email" type="email" id="admin_email" value="{email}" class="form-input" style="max-width: 480px;">
                        <p class="form-hint">Used for admin purposes and engine alerts.</p>
                    </div>
                </div>
            </div>
        </form>

        <div class="neo-card" style="margin-top: 20px;">
            <div class="neo-card-header">
                <h2 class="neo-card-title">Email Delivery</h2>
            </div>
            <div class="neo-card-body">
                <p style="font-size: 13.5px; color: #475569; line-height: 1.6; margin-bottom: 14px;">
                    Form notifications are sent over SMTP. Configure the server with the
                    <code>NEOPRESS_SMTP_HOST</code>, <code>NEOPRESS_SMTP_PORT</code>, <code>NEOPRESS_SMTP_SECURITY</code>,
                    <code>NEOPRESS_SMTP_USER</code>, <code>NEOPRESS_SMTP_PASSWORD</code> and <code>NEOPRESS_SMTP_FROM</code>
                    environment variables, then restart Neopress.
                </p>
                <button type="button" class="btn btn-secondary" id="sendTestEmailBtn" onclick="sendTestEmail()">Send test email to the administration address</button>
                <p id="testEmailResult" style="font-size: 13px; margin-top: 10px;"></p>
            </div>
        </div>

        <script>
            async function sendTestEmail() {{
                const out = document.getElementById('testEmailResult');
                const to = document.getElementById('admin_email').value;
                out.style.color = '#64748b';
                out.textContent = 'Sending to ' + to + '...';
                const res = await fetch('/wp-json/neopress/v1/email/send', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json', 'X-WP-Nonce': (window.wpApiSettings && window.wpApiSettings.nonce) || '' }},
                    body: JSON.stringify({{
                        to,
                        subject: 'Neopress test email',
                        body_html: '<p>Your Neopress site can send email.</p>'
                    }})
                }});
                const data = await res.json().catch(() => ({{}}));
                out.style.color = res.ok ? '#059669' : '#dc2626';
                out.textContent = res.ok ? '✓ Test email sent to ' + to + '.' : (data.message || 'Sending failed.');
            }}

            async function saveSettings(e) {{
                e.preventDefault();
                const title = document.getElementById('blogname').value;
                const description = document.getElementById('blogdescription').value;
                const email = document.getElementById('admin_email').value;
                const url = document.getElementById('siteurl').value;

                const res = await fetch('/wp-json/wp/v2/settings', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ title, description, email, url }})
                }});

                if (res.ok) {{
                    showToast('✓ General settings updated successfully in SQLite!');
                }} else {{
                    showToast('Error saving settings', true);
                }}
            }}
        </script>
        "##,
        title = escape_html(&settings.title),
        desc = escape_html(&settings.description),
        url = escape_html(&settings.url),
        email = escape_html(&settings.email)
    );

    render_wp_admin_shell("settings", "General Settings", &body, settings)
}

// 2. Writing Settings (/n-admin/options-writing)
pub fn render_wp_settings_writing_view(
    settings: &WpSettingsResponse,
    writing: &WritingSettings,
    categories: &[WpCategoryResponse],
) -> String {
    let mut category_options = String::new();
    for cat in categories {
        let selected = if cat.id == writing.default_category {
            "selected"
        } else {
            ""
        };
        category_options.push_str(&format!(
            r#"<option value="{}" {}>{}</option>"#,
            cat.id,
            selected,
            escape_html(&cat.name)
        ));
    }

    let formats = [
        ("standard", "Standard"),
        ("aside", "Aside"),
        ("chat", "Chat"),
        ("gallery", "Gallery"),
        ("link", "Link"),
        ("image", "Image"),
        ("quote", "Quote"),
        ("status", "Status"),
        ("video", "Video"),
        ("audio", "Audio"),
    ];

    let mut format_options = String::new();
    for (f_val, f_label) in formats {
        let selected = if f_val == writing.default_post_format {
            "selected"
        } else {
            ""
        };
        format_options.push_str(&format!(
            r#"<option value="{}" {}>{}</option>"#,
            f_val, selected, f_label
        ));
    }

    let smilies_checked = if writing.use_smilies { "checked" } else { "" };
    let pingback_checked = if writing.default_pingback_flag {
        "checked"
    } else {
        ""
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Writing Settings</h1>
                <p class="page-subtitle">Configure publishing conventions, default categories, and mail servers</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('writingFormSubmit').click()">Save Changes</button>
            </div>
        </div>

        <form id="writingForm" onsubmit="saveWritingSettings(event)">
            <button type="submit" id="writingFormSubmit" style="display: none;"></button>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Default Publishing Defaults</h2></div>
                <div class="neo-card-body">
                    <div class="form-group">
                        <label class="form-label" for="default_category">Default Post Category</label>
                        <select name="default_category" id="default_category" class="form-select" style="max-width: 380px;">
                            {category_options}
                        </select>
                    </div>
                    <!-- Post formats, smilies and pingbacks are not implemented in v0.1; kept hidden so saved values round-trip. -->
                    <div hidden>
                    <div class="form-group">
                        <label class="form-label" for="default_post_format">Default Post Format</label>
                        <select name="default_post_format" id="default_post_format" class="form-select" style="max-width: 380px;">
                            {format_options}
                        </select>
                    </div>
                    <div class="form-group">
                        <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                            <input type="checkbox" id="use_smilies" {smilies_checked}>
                            <span>Convert emoticons like :-) and :-P to graphics on display</span>
                        </label>
                    </div>
                    <div class="form-group">
                        <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                            <input type="checkbox" id="default_pingback_flag" {pingback_checked}>
                            <span>Attempt to notify any blogs linked to from the post</span>
                        </label>
                    </div>
                    </div>
                </div>
            </div>

            <!-- Post via Email is not implemented in v0.1. -->
            <div class="neo-card" hidden>
                <div class="neo-card-header"><h2 class="neo-card-title">Post via Email</h2></div>
                <div class="neo-card-body">
                    <p style="font-size: 13px; color: #64748b; margin-bottom: 16px;">To post to WordPress by email, set up a secret email account with POP3 access.</p>
                    <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 16px;">
                        <div class="form-group">
                            <label class="form-label" for="mailserver_url">Mail Server</label>
                            <input type="text" id="mailserver_url" value="{mailserver_url}" class="form-input">
                        </div>
                        <div class="form-group">
                            <label class="form-label" for="mailserver_port">Port</label>
                            <input type="number" id="mailserver_port" value="{mailserver_port}" class="form-input">
                        </div>
                    </div>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                        <div class="form-group">
                            <label class="form-label" for="mailserver_login">Login Name</label>
                            <input type="text" id="mailserver_login" value="{mailserver_login}" class="form-input">
                        </div>
                        <div class="form-group">
                            <label class="form-label" for="mailserver_pass">Password</label>
                            <input type="password" id="mailserver_pass" value="" class="form-input">
                        </div>
                    </div>
                </div>
            </div>
        </form>

        <script>
            async function saveWritingSettings(e) {{
                e.preventDefault();
                const payload = {{
                    default_category: parseInt(document.getElementById('default_category').value, 10),
                    default_post_format: document.getElementById('default_post_format').value,
                    use_smilies: document.getElementById('use_smilies').checked,
                    default_pingback_flag: document.getElementById('default_pingback_flag').checked,
                    mailserver_url: document.getElementById('mailserver_url').value,
                    mailserver_port: parseInt(document.getElementById('mailserver_port').value, 10),
                    mailserver_login: document.getElementById('mailserver_login').value,
                    mailserver_pass: document.getElementById('mailserver_pass').value
                }};

                const res = await fetch('/n-admin/api/options-writing', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(payload)
                }});

                if (res.ok) {{
                    showToast('✓ Writing settings updated successfully in SQLite!');
                }} else {{
                    showToast('Error saving writing settings', true);
                }}
            }}
        </script>
        "##,
        category_options = category_options,
        format_options = format_options,
        smilies_checked = smilies_checked,
        pingback_checked = pingback_checked,
        mailserver_url = escape_html(&writing.mailserver_url),
        mailserver_port = writing.mailserver_port,
        mailserver_login = escape_html(&writing.mailserver_login),
    );

    render_wp_admin_shell("writing", "Writing Settings", &body, settings)
}

// 3. Reading Settings (/n-admin/options-reading)
pub fn render_wp_settings_reading_view(
    settings: &WpSettingsResponse,
    reading: &ReadingSettings,
    pages: &[WpPostResponse],
) -> String {
    let posts_checked = if reading.show_on_front == "posts" {
        "checked"
    } else {
        ""
    };
    let page_checked = if reading.show_on_front == "page" {
        "checked"
    } else {
        ""
    };
    let search_checked = if !reading.blog_public { "checked" } else { "" };

    let mut front_page_options = r#"<option value="0">&mdash; Select &mdash;</option>"#.to_string();
    let mut posts_page_options = r#"<option value="0">&mdash; Select &mdash;</option>"#.to_string();
    for p in pages {
        let sel_front = if p.id == reading.page_on_front {
            "selected"
        } else {
            ""
        };
        let sel_posts = if p.id == reading.page_for_posts {
            "selected"
        } else {
            ""
        };
        front_page_options.push_str(&format!(
            r#"<option value="{}" {}>{}</option>"#,
            p.id,
            sel_front,
            escape_html(&p.title.rendered)
        ));
        posts_page_options.push_str(&format!(
            r#"<option value="{}" {}>{}</option>"#,
            p.id,
            sel_posts,
            escape_html(&p.title.rendered)
        ));
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Reading Settings</h1>
                <p class="page-subtitle">Configure homepage display, article pagination, and syndication feeds</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('readingFormSubmit').click()">Save Changes</button>
            </div>
        </div>

        <form id="readingForm" onsubmit="saveReadingSettings(event)">
            <button type="submit" id="readingFormSubmit" style="display: none;"></button>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Your homepage displays</h2></div>
                <div class="neo-card-body">
                    <div style="margin-bottom: 14px;">
                        <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                            <input type="radio" name="show_on_front" value="posts" {posts_checked}>
                            <span>Your latest posts</span>
                        </label>
                    </div>
                    <div>
                        <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                            <input type="radio" name="show_on_front" value="page" {page_checked}>
                            <span>A static page (select below)</span>
                        </label>
                        <div style="margin-left: 24px; margin-top: 10px; display: flex; flex-direction: column; gap: 8px;">
                            <div style="display: flex; align-items: center; gap: 10px;">
                                <span style="font-size: 13px; color: #64748b; width: 100px;">Homepage:</span>
                                <select id="page_on_front" class="form-select" style="max-width: 320px;">{front_page_options}</select>
                            </div>
                            <div style="display: flex; align-items: center; gap: 10px;">
                                <span style="font-size: 13px; color: #64748b; width: 100px;">Posts page:</span>
                                <select id="page_for_posts" class="form-select" style="max-width: 320px;">{posts_page_options}</select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Pagination &amp; Feeds</h2></div>
                <div class="neo-card-body">
                    <div class="form-group" style="display: flex; align-items: center; gap: 12px;">
                        <label class="form-label" style="margin-bottom: 0;">Blog pages show at most</label>
                        <input type="number" id="posts_per_page" value="{posts_per_page}" min="1" max="100" class="form-input" style="width: 80px;">
                        <span style="font-size: 13px; color: #64748b;">posts</span>
                    </div>
                    <div class="form-group" style="display: flex; align-items: center; gap: 12px;">
                        <label class="form-label" style="margin-bottom: 0;">Syndication feeds show the most recent</label>
                        <input type="number" id="posts_per_rss" value="{posts_per_rss}" min="1" max="100" class="form-input" style="width: 80px;">
                        <span style="font-size: 13px; color: #64748b;">items</span>
                    </div>
                </div>
            </div>

            <div class="neo-card">
                <div class="neo-card-header"><h2 class="neo-card-title">Search Engine Visibility</h2></div>
                <div class="neo-card-body">
                    <label style="display: flex; align-items: flex-start; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="discourage_search" {search_checked} style="margin-top: 3px;">
                        <div>
                            <span>Discourage search engines from indexing this site</span>
                            <p class="form-hint" style="margin-top: 2px;">It is up to search engines to honor this request.</p>
                        </div>
                    </label>
                </div>
            </div>
        </form>

        <script>
            async function saveReadingSettings(e) {{
                e.preventDefault();
                const show_on_front = document.querySelector('input[name="show_on_front"]:checked').value;
                const page_on_front = parseInt(document.getElementById('page_on_front').value, 10);
                const page_for_posts = parseInt(document.getElementById('page_for_posts').value, 10);
                const posts_per_page = parseInt(document.getElementById('posts_per_page').value, 10);
                const posts_per_rss = parseInt(document.getElementById('posts_per_rss').value, 10);
                const blog_public = !document.getElementById('discourage_search').checked;

                const res = await fetch('/n-admin/api/options-reading', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ show_on_front, page_on_front, page_for_posts, posts_per_page, posts_per_rss, blog_public }})
                }});

                if (res.ok) {{
                    showToast('✓ Reading settings saved in SQLite!');
                }} else {{
                    showToast('Failed to save reading settings', true);
                }}
            }}
        </script>
        "##,
        posts_checked = posts_checked,
        page_checked = page_checked,
        front_page_options = front_page_options,
        posts_page_options = posts_page_options,
        posts_per_page = reading.posts_per_page,
        posts_per_rss = reading.posts_per_rss,
        search_checked = search_checked
    );

    render_wp_admin_shell("reading", "Reading Settings", &body, settings)
}

// 4. Discussion Settings (/n-admin/options-discussion)
pub fn render_wp_settings_discussion_view(
    settings: &WpSettingsResponse,
    discussion: &DiscussionSettings,
) -> String {
    let chk = |b: bool| if b { "checked" } else { "" };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Discussion Settings</h1>
                <p class="page-subtitle">Configure comment moderation, registration rules, and author avatars</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('discussionFormSubmit').click()">Save Changes</button>
            </div>
        </div>

        <form id="discussionForm" onsubmit="saveDiscussionSettings(event)">
            <button type="submit" id="discussionFormSubmit" style="display: none;"></button>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Default Article Settings</h2></div>
                <div class="neo-card-body" style="display: flex; flex-direction: column; gap: 10px;">
                    <label hidden style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="default_ping_status" {ping_open}>
                        <span>Allow link notifications from other blogs (pingbacks and trackbacks) on new articles</span>
                    </label>
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="default_comment_status" {comm_open}>
                        <span>Allow people to submit comments on new posts</span>
                    </label>
                </div>
            </div>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Other Comment Settings</h2></div>
                <div class="neo-card-body" style="display: flex; flex-direction: column; gap: 10px;">
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="require_name_email" {req_author}>
                        <span>Comment author must fill out name and email</span>
                    </label>
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="comment_registration" {req_reg}>
                        <span>Users must be registered and logged in to comment</span>
                    </label>
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="close_old" {close_old}>
                        <span>Automatically close comments on articles older than</span>
                        <input type="number" id="close_days" value="{close_days}" style="width: 60px; padding: 4px;" class="form-input">
                        <span>days</span>
                    </label>
                    <label hidden style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="thread_comments" {thread_comm}>
                        <span>Enable threaded (nested) comments</span>
                        <input type="number" id="thread_depth" value="{thread_depth}" style="width: 60px; padding: 4px;" class="form-input">
                        <span>levels deep</span>
                    </label>
                </div>
            </div>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Before a comment appears</h2></div>
                <div class="neo-card-body" style="display: flex; flex-direction: column; gap: 10px;">
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="comment_moderation" {comm_mod}>
                        <span>Comment must be manually approved</span>
                    </label>
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="comment_previously_approved" {prev_app}>
                        <span>Comment author must have a previously approved comment</span>
                    </label>
                </div>
            </div>

            <div class="neo-card" hidden>
                <div class="neo-card-header"><h2 class="neo-card-title">Avatars</h2></div>
                <div class="neo-card-body">
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="show_avatars" {show_avatars}>
                        <span>Show Avatars from Gravatar</span>
                    </label>
                </div>
            </div>
        </form>

        <script>
            async function saveDiscussionSettings(e) {{
                e.preventDefault();
                const payload = {{
                    default_ping_status: document.getElementById('default_ping_status').checked ? 'open' : 'closed',
                    default_comment_status: document.getElementById('default_comment_status').checked ? 'open' : 'closed',
                    require_name_email: document.getElementById('require_name_email').checked,
                    comment_registration: document.getElementById('comment_registration').checked,
                    close_comments_for_old_posts: document.getElementById('close_old').checked,
                    close_comments_days_old: parseInt(document.getElementById('close_days').value, 10),
                    thread_comments: document.getElementById('thread_comments').checked,
                    thread_comments_depth: parseInt(document.getElementById('thread_depth').value, 10),
                    comment_moderation: document.getElementById('comment_moderation').checked,
                    comment_previously_approved: document.getElementById('comment_previously_approved').checked,
                    show_avatars: document.getElementById('show_avatars').checked
                }};

                const res = await fetch('/n-admin/api/options-discussion', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(payload)
                }});

                if (res.ok) {{
                    showToast('✓ Discussion settings saved in SQLite!');
                }} else {{
                    showToast('Error saving discussion settings', true);
                }}
            }}
        </script>
        "##,
        ping_open = chk(discussion.default_ping_status == "open"),
        comm_open = chk(discussion.default_comment_status == "open"),
        req_author = chk(discussion.require_name_email),
        req_reg = chk(discussion.comment_registration),
        close_old = chk(discussion.close_comments_for_old_posts),
        close_days = discussion.close_comments_days_old,
        thread_comm = chk(discussion.thread_comments),
        thread_depth = discussion.thread_comments_depth,
        comm_mod = chk(discussion.comment_moderation),
        prev_app = chk(discussion.comment_previously_approved),
        show_avatars = chk(discussion.show_avatars)
    );

    render_wp_admin_shell("discussion", "Discussion Settings", &body, settings)
}

// 5. Media Settings (/n-admin/options-media)
pub fn render_wp_settings_media_view(
    settings: &WpSettingsResponse,
    media: &MediaSettings,
) -> String {
    let crop_checked = if media.thumbnail_crop { "checked" } else { "" };
    let month_checked = if media.uploads_use_yearmonth_folders {
        "checked"
    } else {
        ""
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Media Settings</h1>
                <p class="page-subtitle">Configure image dimension thresholds, thumbnail crops, and folder hierarchy</p>
            </div>
            <div>

            </div>
        </div>

        <div class="neo-card" style="margin-bottom: 20px;">
            <div class="neo-card-header"><h2 class="neo-card-title">Image Sizes</h2></div>
            <div class="neo-card-body">
                <p style="font-size: 13.5px; color: #475569; line-height: 1.6;">
                    Neopress generates image variants automatically: a 150&nbsp;px thumbnail, a 300&nbsp;px medium and a
                    1024&nbsp;px large size, plus a WebP copy. Other WordPress sizes (768, 1536 and 2048&nbsp;px) are created on demand.
                    Custom size settings are planned for a later release.
                </p>
            </div>
        </div>

        <!-- Custom image sizes and dated upload folders are not implemented in v0.1. -->
        <form id="mediaForm" onsubmit="saveMediaSettings(event)" hidden>
            <button type="submit" id="mediaFormSubmit" style="display: none;"></button>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Image Sizes</h2></div>
                <div class="neo-card-body">
                    <p style="font-size: 13px; color: #64748b; margin-bottom: 20px;">The sizes listed below determine the maximum dimensions in pixels to use when inserting an image into the body of a post.</p>

                    <h4 style="font-size: 14px; font-weight: 600; color: #0f172a; margin-bottom: 8px;">Thumbnail size</h4>
                    <div style="display: flex; gap: 16px; align-items: center; margin-bottom: 8px;">
                        <label style="font-size: 13px;">Width <input type="number" id="thumbnail_size_w" value="{thumb_w}" class="form-input" style="width: 80px; display: inline-block; margin-left: 6px;"></label>
                        <label style="font-size: 13px;">Height <input type="number" id="thumbnail_size_h" value="{thumb_h}" class="form-input" style="width: 80px; display: inline-block; margin-left: 6px;"></label>
                    </div>
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; margin-bottom: 24px; cursor: pointer;">
                        <input type="checkbox" id="thumbnail_crop" {crop_checked}>
                        <span>Crop thumbnail to exact dimensions (normally thumbnails are proportional)</span>
                    </label>

                    <h4 style="font-size: 14px; font-weight: 600; color: #0f172a; margin-bottom: 8px;">Medium size</h4>
                    <div style="display: flex; gap: 16px; align-items: center; margin-bottom: 24px;">
                        <label style="font-size: 13px;">Max Width <input type="number" id="medium_size_w" value="{med_w}" class="form-input" style="width: 80px; display: inline-block; margin-left: 6px;"></label>
                        <label style="font-size: 13px;">Max Height <input type="number" id="medium_size_h" value="{med_h}" class="form-input" style="width: 80px; display: inline-block; margin-left: 6px;"></label>
                    </div>

                    <h4 style="font-size: 14px; font-weight: 600; color: #0f172a; margin-bottom: 8px;">Large size</h4>
                    <div style="display: flex; gap: 16px; align-items: center;">
                        <label style="font-size: 13px;">Max Width <input type="number" id="large_size_w" value="{large_w}" class="form-input" style="width: 80px; display: inline-block; margin-left: 6px;"></label>
                        <label style="font-size: 13px;">Max Height <input type="number" id="large_size_h" value="{large_h}" class="form-input" style="width: 80px; display: inline-block; margin-left: 6px;"></label>
                    </div>
                </div>
            </div>

            <div class="neo-card">
                <div class="neo-card-header"><h2 class="neo-card-title">Uploading Files</h2></div>
                <div class="neo-card-body">
                    <label style="display: flex; align-items: center; gap: 8px; font-weight: 500; cursor: pointer;">
                        <input type="checkbox" id="uploads_use_yearmonth_folders" {month_checked}>
                        <span>Organize my uploads into month- and year-based folders</span>
                    </label>
                </div>
            </div>
        </form>

        <script>
            async function saveMediaSettings(e) {{
                e.preventDefault();
                const payload = {{
                    thumbnail_size_w: parseInt(document.getElementById('thumbnail_size_w').value, 10),
                    thumbnail_size_h: parseInt(document.getElementById('thumbnail_size_h').value, 10),
                    thumbnail_crop: document.getElementById('thumbnail_crop').checked,
                    medium_size_w: parseInt(document.getElementById('medium_size_w').value, 10),
                    medium_size_h: parseInt(document.getElementById('medium_size_h').value, 10),
                    large_size_w: parseInt(document.getElementById('large_size_w').value, 10),
                    large_size_h: parseInt(document.getElementById('large_size_h').value, 10),
                    uploads_use_yearmonth_folders: document.getElementById('uploads_use_yearmonth_folders').checked
                }};

                const res = await fetch('/n-admin/api/options-media', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(payload)
                }});

                if (res.ok) {{
                    showToast('✓ Media dimensions updated successfully in SQLite!');
                }} else {{
                    showToast('Error saving media settings', true);
                }}
            }}
        </script>
        "##,
        thumb_w = media.thumbnail_size_w,
        thumb_h = media.thumbnail_size_h,
        crop_checked = crop_checked,
        med_w = media.medium_size_w,
        med_h = media.medium_size_h,
        large_w = media.large_size_w,
        large_h = media.large_size_h,
        month_checked = month_checked
    );

    render_wp_admin_shell("options-media", "Media Settings", &body, settings)
}

// 6. Permalinks Settings (/n-admin/options-permalink)
pub fn render_wp_settings_permalink_view(
    settings: &WpSettingsResponse,
    permalinks: &PermalinkSettings,
) -> String {
    let str = &permalinks.permalink_structure;
    let plain_chk = if str.is_empty() { "checked" } else { "" };
    let day_chk = if str == "/%year%/%monthnum%/%day%/%postname%/" {
        "checked"
    } else {
        ""
    };
    let month_chk = if str == "/%year%/%monthnum%/%postname%/" {
        "checked"
    } else {
        ""
    };
    let num_chk = if str == "/archives/%post_id%" {
        "checked"
    } else {
        ""
    };
    let post_chk = if str == "/%postname%/" { "checked" } else { "" };
    let custom_val = str;

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Permalink Settings</h1>
                <p class="page-subtitle">Choose the URL structure for your permanent links and article archives</p>
            </div>
            <div>

            </div>
        </div>

        <div class="neo-card" style="margin-bottom: 20px;">
            <div class="neo-card-header"><h2 class="neo-card-title">Permalink Structure</h2></div>
            <div class="neo-card-body">
                <p style="font-size: 13.5px; color: #475569; line-height: 1.6;">
                    Neopress uses <code>/%postname%/</code> for every post, page and custom post type
                    (child pages also resolve at <code>/parent/child/</code>). Categories, tags and authors live at
                    <code>/category/…</code>, <code>/tag/…</code> and <code>/author/…</code>.
                    Custom structures are planned for a later release; changed slugs get an automatic 301 redirect.
                </p>
            </div>
        </div>

        <!-- Custom permalink structures and term bases are not implemented in v0.1. -->
        <form id="permalinkForm" onsubmit="savePermalinkSettings(event)" hidden>
            <button type="submit" id="permalinkFormSubmit" style="display: none;"></button>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Common Settings</h2></div>
                <div class="neo-card-body" style="display: flex; flex-direction: column; gap: 14px;">
                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                        <input type="radio" name="permalink_structure" value="" {plain_chk} onchange="setPermStructure('')">
                        <span style="font-weight: 600; width: 140px;">Plain</span>
                        <code style="font-family: var(--font-mono); font-size: 12.5px; color: #64748b;">{url}/?p=123</code>
                    </label>

                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                        <input type="radio" name="permalink_structure" value="/%year%/%monthnum%/%day%/%postname%/" {day_chk} onchange="setPermStructure('/%year%/%monthnum%/%day%/%postname%/')">
                        <span style="font-weight: 600; width: 140px;">Day and name</span>
                        <code style="font-family: var(--font-mono); font-size: 12.5px; color: #64748b;">{url}/2026/10/21/sample-post/</code>
                    </label>

                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                        <input type="radio" name="permalink_structure" value="/%year%/%monthnum%/%postname%/" {month_chk} onchange="setPermStructure('/%year%/%monthnum%/%postname%/')">
                        <span style="font-weight: 600; width: 140px;">Month and name</span>
                        <code style="font-family: var(--font-mono); font-size: 12.5px; color: #64748b;">{url}/2026/10/sample-post/</code>
                    </label>

                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                        <input type="radio" name="permalink_structure" value="/archives/%post_id%" {num_chk} onchange="setPermStructure('/archives/%post_id%')">
                        <span style="font-weight: 600; width: 140px;">Numeric</span>
                        <code style="font-family: var(--font-mono); font-size: 12.5px; color: #64748b;">{url}/archives/123</code>
                    </label>

                    <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
                        <input type="radio" name="permalink_structure" value="/%postname%/" {post_chk} onchange="setPermStructure('/%postname%/')">
                        <span style="font-weight: 600; width: 140px;">Post name</span>
                        <code style="font-family: var(--font-mono); font-size: 12.5px; color: #059669; font-weight: 600;">{url}/sample-post/</code>
                    </label>

                    <div style="margin-top: 10px; padding-top: 14px; border-top: 1px solid var(--border-subtle);">
                        <label class="form-label" for="custom_structure">Custom Structure</label>
                        <input type="text" id="custom_structure" value="{custom_val}" class="form-input" style="max-width: 480px; font-family: var(--font-mono);">
                    </div>
                </div>
            </div>

            <div class="neo-card">
                <div class="neo-card-header"><h2 class="neo-card-title">Optional Taxonomies Base</h2></div>
                <div class="neo-card-body">
                    <div class="form-group">
                        <label class="form-label" for="category_base">Category Base</label>
                        <input type="text" id="category_base" value="{cat_base}" placeholder="category" class="form-input" style="max-width: 320px;">
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="tag_base">Tag Base</label>
                        <input type="text" id="tag_base" value="{tag_base}" placeholder="tag" class="form-input" style="max-width: 320px;">
                    </div>
                </div>
            </div>
        </form>

        <script>
            function setPermStructure(val) {{
                document.getElementById('custom_structure').value = val;
            }}
            async function savePermalinkSettings(e) {{
                e.preventDefault();
                const permalink_structure = document.getElementById('custom_structure').value;
                const category_base = document.getElementById('category_base').value;
                const tag_base = document.getElementById('tag_base').value;

                const res = await fetch('/n-admin/api/options-permalink', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ permalink_structure, category_base, tag_base }})
                }});

                if (res.ok) {{
                    showToast('✓ Permalink rewrite rules updated in SQLite!');
                }} else {{
                    showToast('Error saving permalink settings', true);
                }}
            }}
        </script>
        "##,
        url = escape_html(&settings.url),
        plain_chk = plain_chk,
        day_chk = day_chk,
        month_chk = month_chk,
        num_chk = num_chk,
        post_chk = post_chk,
        custom_val = escape_html(custom_val),
        cat_base = escape_html(&permalinks.category_base),
        tag_base = escape_html(&permalinks.tag_base)
    );

    render_wp_admin_shell("permalinks", "Permalink Settings", &body, settings)
}

// 7. Privacy Settings (/n-admin/options-privacy)
pub fn render_wp_settings_privacy_view(
    settings: &WpSettingsResponse,
    privacy: &PrivacySettings,
    pages: &[WpPostResponse],
) -> String {
    let mut page_options = r#"<option value="0">&mdash; Select &mdash;</option>"#.to_string();
    for p in pages {
        let selected = if p.id == privacy.wp_page_for_privacy_policy {
            "selected"
        } else {
            ""
        };
        page_options.push_str(&format!(
            r#"<option value="{}" {}>{}</option>"#,
            p.id,
            selected,
            escape_html(&p.title.rendered)
        ));
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Privacy Settings</h1>
                <p class="page-subtitle">Designate your official site privacy policy page and GDPR data handling requirements</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="document.getElementById('privacyFormSubmit').click()">Save Changes</button>
            </div>
        </div>

        <form id="privacyForm" onsubmit="savePrivacySettings(event)">
            <button type="submit" id="privacyFormSubmit" style="display: none;"></button>

            <div class="neo-card" style="margin-bottom: 20px;">
                <div class="neo-card-header"><h2 class="neo-card-title">Change your Privacy Policy page</h2></div>
                <div class="neo-card-body">
                    <p style="font-size: 13.5px; color: #475569; line-height: 1.6; margin-bottom: 16px;">
                        As a website owner, you may need to comply with national or international privacy laws. For example, you may need to create and display a Privacy Policy.
                    </p>
                    <div class="form-group" style="display: flex; align-items: center; gap: 12px;">
                        <label class="form-label" for="privacy_page_id" style="margin-bottom: 0;">Select a Privacy Policy page:</label>
                        <select id="privacy_page_id" class="form-select" style="max-width: 320px;">
                            {page_options}
                        </select>
                        <button type="button" class="btn btn-secondary" onclick="createPrivacyPolicyPage()">Create New Page</button>
                    </div>
                </div>
            </div>

            <div class="neo-card">
                <div class="neo-card-header"><h2 class="neo-card-title">Privacy Policy Guide &amp; Compliance</h2></div>
                <div class="neo-card-body">
                    <p style="font-size: 13px; color: #64748b; line-height: 1.6;">
                        Need help putting together your new Privacy Policy page? Check out our suggested policy text below covering comments, media cookies, embedded content, analytics, and data retention in SQLite.
                    </p>
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 16px; margin-top: 14px; font-size: 12.5px; color: #334155;">
                        <strong>What personal data we collect and why:</strong>
                        <ul style="padding-left: 20px; margin-top: 6px;">
                            <li><strong>Comments:</strong> When visitors leave comments, we collect data shown in comments form and IP address.</li>
                            <li><strong>Media:</strong> If you upload images, avoid uploading images with embedded location data (EXIF GPS).</li>
                            <li><strong>Cookies:</strong> If you leave a comment, you may opt-in to saving your name and email in cookies.</li>
                        </ul>
                    </div>
                </div>
            </div>
        </form>

        <script>
            async function savePrivacySettings(e) {{
                e.preventDefault();
                const wp_page_for_privacy_policy = parseInt(document.getElementById('privacy_page_id').value, 10);
                const res = await fetch('/n-admin/api/options-privacy', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ wp_page_for_privacy_policy }})
                }});
                if (res.ok) {{
                    showToast('✓ Privacy policy page assigned successfully in SQLite!');
                }} else {{
                    showToast('Error updating privacy page', true);
                }}
            }}

            async function createPrivacyPolicyPage() {{
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            title: 'Privacy Policy',
                            content: '<h2>Who we are</h2><p>Our website address is: ' + window.location.origin + '</p><h2>Comments</h2><p>When visitors leave comments on the site we collect the data shown in the comments form.</p>',
                            status: 'draft',
                            post_type: 'page'
                        }})
                    }});
                    if (res.ok) {{
                        const page = await res.json();
                        showToast('✓ Privacy Policy draft page created! ID: ' + page.id);
                        setTimeout(() => window.location.reload(), 600);
                    }} else {{
                        showToast('Failed to create page', true);
                    }}
                }} catch (e) {{
                    showToast('Error creating draft page', true);
                }}
            }}
        </script>
        "##,
        page_options = page_options
    );

    render_wp_admin_shell("privacy", "Privacy Settings", &body, settings)
}
