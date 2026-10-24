//! Admin Navigation and Custom Post Type tab management.

/// Converts a custom post type slug into Title Case format.
///
/// Examples:
/// - "trainingen" -> "Trainingen"
/// - "custom_post_type" -> "Custom Post Type"
/// - "custom_vacatures" -> "Custom Vacatures"
/// - "faq" -> "FAQ"
pub fn format_cpt_title(slug: &str) -> String {
    let clean = slug.trim();
    if clean.is_empty() {
        return String::new();
    }
    if clean.eq_ignore_ascii_case("faq") {
        return "FAQ".to_string();
    }
    clean
        .split(|c| c == '_' || c == '-')
        .filter(|part| !part.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let mut s = first.to_uppercase().collect::<String>();
                    s.push_str(chars.as_str());
                    s
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Renders dynamic admin sidebar navigation links for registered custom post types.
pub fn render_cpt_navigation(current_tab: &str, custom_post_types: &[String]) -> String {
    let mut cpt_html = String::new();
    for slug in custom_post_types {
        let label = format_cpt_title(slug);
        let active_cls = if current_tab == slug { "active" } else { "" };
        cpt_html.push_str(&format!(
            r#"
			<div class="nav-group">
				<a href="/n-admin/edit.php?post_type={slug}" class="nav-item {active_cls}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path></svg>
					<span>{label}</span>
				</a>
			</div>"#,
            slug = slug,
            active_cls = active_cls,
            label = label
        ));
    }
    cpt_html
}

#[derive(Default)]
pub struct NavState {
    pub is_dash: &'static str,
    pub is_dash_open: &'static str,
    pub sub_dash_home: &'static str,
    pub sub_dash_updates: &'static str,
    pub is_posts: &'static str,
    pub is_posts_open: &'static str,
    pub sub_posts_all: &'static str,
    pub sub_posts_new: &'static str,
    pub sub_posts_cats: &'static str,
    pub sub_posts_tags: &'static str,
    pub is_media: &'static str,
    pub is_media_open: &'static str,
    pub sub_media_lib: &'static str,
    pub sub_media_new: &'static str,
    pub is_pages: &'static str,
    pub is_pages_open: &'static str,
    pub sub_pages_all: &'static str,
    pub sub_pages_new: &'static str,
    pub is_comments: &'static str,
    pub is_appearance: &'static str,
    pub is_app_open: &'static str,
    pub sub_app_themes: &'static str,
    pub sub_app_cust: &'static str,
    pub sub_app_widg: &'static str,
    pub sub_app_menus: &'static str,
    pub sub_app_editor: &'static str,
    pub is_plugins: &'static str,
    pub is_plugins_open: &'static str,
    pub sub_plug_list: &'static str,
    pub sub_plug_new: &'static str,
    pub sub_plug_editor: &'static str,
    pub is_seo: &'static str,
    pub is_forms: &'static str,
    pub is_backups: &'static str,
    pub is_users: &'static str,
    pub is_users_open: &'static str,
    pub sub_users_all: &'static str,
    pub sub_users_new: &'static str,
    pub sub_users_prof: &'static str,
    pub is_tools: &'static str,
    pub is_tools_open: &'static str,
    pub sub_tools_avail: &'static str,
    pub sub_tools_import: &'static str,
    pub sub_tools_export: &'static str,
    pub sub_tools_health: &'static str,
    pub sub_tools_exp_data: &'static str,
    pub sub_tools_era_data: &'static str,
    pub is_settings: &'static str,
    pub is_settings_open: &'static str,
    pub sub_set_gen: &'static str,
    pub sub_set_writ: &'static str,
    pub sub_set_read: &'static str,
    pub sub_set_disc: &'static str,
    pub sub_set_media: &'static str,
    pub sub_set_perm: &'static str,
    pub sub_set_priv: &'static str,
    pub is_engine: &'static str,
}

impl NavState {
    pub fn compute(current_tab: &str) -> Self {
        let is_dash = if current_tab == "dashboard" || current_tab == "updates" { "active" } else { "" };
        let is_dash_open = if !is_dash.is_empty() { "open" } else { "" };
        let sub_dash_home = if current_tab == "dashboard" || current_tab == "home" { "active" } else { "" };
        let sub_dash_updates = if current_tab == "updates" { "active" } else { "" };

        let is_posts = if current_tab == "posts"
            || current_tab == "post-new"
            || current_tab == "categories"
            || current_tab == "tags"
        {
            "active"
        } else {
            ""
        };
        let is_posts_open = if !is_posts.is_empty() { "open" } else { "" };
        let sub_posts_all = if current_tab == "posts" { "active" } else { "" };
        let sub_posts_new = if current_tab == "post-new" { "active" } else { "" };
        let sub_posts_cats = if current_tab == "categories" { "active" } else { "" };
        let sub_posts_tags = if current_tab == "tags" { "active" } else { "" };

        let is_media = if current_tab == "media" || current_tab == "media-new" { "active" } else { "" };
        let is_media_open = if !is_media.is_empty() { "open" } else { "" };
        let sub_media_lib = if current_tab == "media" { "active" } else { "" };
        let sub_media_new = if current_tab == "media-new" { "active" } else { "" };

        let is_pages = if current_tab == "pages" || current_tab == "page-new" { "active" } else { "" };
        let is_pages_open = if !is_pages.is_empty() { "open" } else { "" };
        let sub_pages_all = if current_tab == "pages" { "active" } else { "" };
        let sub_pages_new = if current_tab == "page-new" { "active" } else { "" };

        let is_comments = if current_tab == "comments" { "active" } else { "" };

        let is_appearance = if current_tab == "appearance"
            || current_tab == "themes"
            || current_tab == "customize"
            || current_tab == "widgets"
            || current_tab == "menus"
            || current_tab == "theme-editor"
        {
            "active"
        } else {
            ""
        };
        let is_app_open = if !is_appearance.is_empty() { "open" } else { "" };
        let sub_app_themes = if current_tab == "themes" || current_tab == "appearance" { "active" } else { "" };
        let sub_app_cust = if current_tab == "customize" { "active" } else { "" };
        let sub_app_widg = if current_tab == "widgets" { "active" } else { "" };
        let sub_app_menus = if current_tab == "menus" { "active" } else { "" };
        let sub_app_editor = if current_tab == "theme-editor" { "active" } else { "" };

        let is_plugins = if current_tab == "plugins"
            || current_tab == "plugin-install"
            || current_tab == "plugin-editor"
            || current_tab == "plugin-settings"
        {
            "active"
        } else {
            ""
        };
        let is_plugins_open = if !is_plugins.is_empty() { "open" } else { "" };
        let sub_plug_list = if current_tab == "plugins" || current_tab == "plugin-settings" { "active" } else { "" };
        let sub_plug_new = if current_tab == "plugin-install" { "active" } else { "" };
        let sub_plug_editor = if current_tab == "plugin-editor" { "active" } else { "" };

        let is_seo = if current_tab == "seo" { "active" } else { "" };
        let is_forms = if current_tab == "forms" { "active" } else { "" };
        let is_backups = if current_tab == "backups" { "active" } else { "" };

        let is_users = if current_tab == "users" || current_tab == "user-new" || current_tab == "profile" {
            "active"
        } else {
            ""
        };
        let is_users_open = if !is_users.is_empty() { "open" } else { "" };
        let sub_users_all = if current_tab == "users" { "active" } else { "" };
        let sub_users_new = if current_tab == "user-new" { "active" } else { "" };
        let sub_users_prof = if current_tab == "profile" { "active" } else { "" };

        let is_tools = if current_tab == "tools"
            || current_tab == "import"
            || current_tab == "export"
            || current_tab == "seo"
            || current_tab == "backups"
            || current_tab == "site-health"
            || current_tab == "export-personal-data"
            || current_tab == "erase-personal-data"
        {
            "active"
        } else {
            ""
        };
        let is_tools_open = if !is_tools.is_empty() { "open" } else { "" };
        let sub_tools_avail = if current_tab == "tools" { "active" } else { "" };
        let sub_tools_import = if current_tab == "import" { "active" } else { "" };
        let sub_tools_export = if current_tab == "export" { "active" } else { "" };
        let sub_tools_health = if current_tab == "site-health" { "active" } else { "" };
        let sub_tools_exp_data = if current_tab == "export-personal-data" { "active" } else { "" };
        let sub_tools_era_data = if current_tab == "erase-personal-data" { "active" } else { "" };

        let is_settings = if current_tab.starts_with("settings")
            || current_tab == "writing"
            || current_tab == "reading"
            || current_tab == "discussion"
            || current_tab == "options-media"
            || current_tab == "permalinks"
            || current_tab == "privacy"
        {
            "active"
        } else {
            ""
        };
        let is_settings_open = if !is_settings.is_empty() { "open" } else { "" };
        let sub_set_gen = if current_tab == "settings" || current_tab == "settings-general" { "active" } else { "" };
        let sub_set_writ = if current_tab == "writing" || current_tab == "settings-writing" { "active" } else { "" };
        let sub_set_read = if current_tab == "reading" || current_tab == "settings-reading" { "active" } else { "" };
        let sub_set_disc = if current_tab == "discussion" || current_tab == "settings-discussion" { "active" } else { "" };
        let sub_set_media = if current_tab == "options-media" || current_tab == "settings-media" { "active" } else { "" };
        let sub_set_perm = if current_tab == "permalinks"
            || current_tab == "settings-permalink"
            || current_tab == "settings-permalinks"
        {
            "active"
        } else {
            ""
        };
        let sub_set_priv = if current_tab == "privacy" || current_tab == "settings-privacy" { "active" } else { "" };

        let is_engine = if current_tab == "engine" { "active" } else { "" };

        Self {
            is_dash,
            is_dash_open,
            sub_dash_home,
            sub_dash_updates,
            is_posts,
            is_posts_open,
            sub_posts_all,
            sub_posts_new,
            sub_posts_cats,
            sub_posts_tags,
            is_media,
            is_media_open,
            sub_media_lib,
            sub_media_new,
            is_pages,
            is_pages_open,
            sub_pages_all,
            sub_pages_new,
            is_comments,
            is_appearance,
            is_app_open,
            sub_app_themes,
            sub_app_cust,
            sub_app_widg,
            sub_app_menus,
            sub_app_editor,
            is_plugins,
            is_plugins_open,
            sub_plug_list,
            sub_plug_new,
            sub_plug_editor,
            is_seo,
            is_forms,
            is_backups,
            is_users,
            is_users_open,
            sub_users_all,
            sub_users_new,
            sub_users_prof,
            is_tools,
            is_tools_open,
            sub_tools_avail,
            sub_tools_import,
            sub_tools_export,
            sub_tools_health,
            sub_tools_exp_data,
            sub_tools_era_data,
            is_settings,
            is_settings_open,
            sub_set_gen,
            sub_set_writ,
            sub_set_read,
            sub_set_disc,
            sub_set_media,
            sub_set_perm,
            sub_set_priv,
            is_engine,
        }
    }
}

/// Renders the classic WordPress top admin bar (`#wpadminbar`) for the admin environment.
pub fn render_wp_admin_bar(site_name: &str, pending_comments: i64) -> String {
    format!(
        r##"<div id="wpadminbar">
	<div class="ab-top-menu-wrap">
		<ul class="ab-top-menu">
			<li class="menupop">
				<a class="ab-item" href="/n-admin/index.php" title="About Neopress" style="padding: 0 12px;">
					<img src="/static/neopress-icon.svg" style="width: 18px; height: 18px; border-radius: 4px;" alt="Neopress" />
				</a>
				<div class="ab-sub-wrapper">
					<ul class="ab-submenu">
						<li><a class="ab-item" href="/n-admin/index.php">About Neopress</a></li>
						<li><a class="ab-item" href="/n-admin/site-health.php">Site Health</a></li>
						<li><a class="ab-item" href="/n-admin/neopress-engine.php">Engine Telemetry</a></li>
					</ul>
				</div>
			</li>
			<li class="menupop">
				<a class="ab-item" href="/" target="_blank">
					<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
					<span style="font-weight: 500;">{site_name}</span>
				</a>
				<div class="ab-sub-wrapper">
					<ul class="ab-submenu">
						<li><a class="ab-item" href="/" target="_blank">Visit Site &nearr;</a></li>
						<li><a class="ab-item" href="/n-admin/themes.php">Themes</a></li>
						<li><a class="ab-item" href="/n-admin/widgets.php">Widgets</a></li>
						<li><a class="ab-item" href="/n-admin/nav-menus.php">Menus</a></li>
						<li><a class="ab-item" href="/n-admin/customize.php">Customize</a></li>
					</ul>
				</div>
			</li>
			<li>
				<a class="ab-item" href="/n-admin/update-core.php" title="Updates">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><polyline points="23 4 23 10 17 10"></polyline><polyline points="1 20 1 14 7 14"></polyline><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
					<span>0</span>
				</a>
			</li>
			<li>
				<a class="ab-item" href="/n-admin/edit.php" title="Quick Search (Ctrl+K)">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
					<span>Ctrl+K</span>
				</a>
			</li>
			<li>
				<a class="ab-item" href="/n-admin/edit-comments.php" title="Comments">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
					<span>{pending_comments}</span>
				</a>
			</li>
			<li class="menupop">
				<a class="ab-item" href="/n-admin/post-new.php">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
					<span>New</span>
				</a>
				<div class="ab-sub-wrapper">
					<ul class="ab-submenu">
						<li><a class="ab-item" href="/n-admin/post-new.php">Post</a></li>
						<li><a class="ab-item" href="/n-admin/media-new.php">Media</a></li>
						<li><a class="ab-item" href="/n-admin/post-new.php?post_type=page">Page</a></li>
						<li><a class="ab-item" href="/n-admin/user-new.php">User</a></li>
					</ul>
				</div>
			</li>
			<li>
				<a class="ab-item" href="/n-admin/builder" title="Open NeoStudio Visual Builder" style="color: #60a5fa;">
					<span>⚡ NeoStudio</span>
				</a>
			</li>
		</ul>
		<ul class="ab-top-menu ab-top-secondary">
			<li class="menupop">
				<a class="ab-item" href="/n-admin/profile.php">
					<span>Howdy, <strong>admin</strong></span>
					<span class="ab-avatar">A</span>
				</a>
				<div class="ab-sub-wrapper ab-sub-right">
					<ul class="ab-submenu">
						<li><a class="ab-item" href="/n-admin/profile.php">Edit Profile</a></li>
						<li><a class="ab-item" href="/wp-login.php?action=logout" style="color: #f87171;">Log Out</a></li>
					</ul>
				</div>
			</li>
		</ul>
	</div>
</div>"##,
        site_name = site_name,
        pending_comments = pending_comments
    )
}
