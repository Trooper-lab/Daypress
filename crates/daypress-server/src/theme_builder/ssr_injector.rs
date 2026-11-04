//! Dynamic Placeholder AST Hydration & SSR Injection Pipeline.

use super::conditions::{evaluate_template, resolve_template_for_type};
use super::models::{BreadcrumbItem, RequestContext, ResolvedThemePartials, TemplateType};
use super::repository::ThemeTemplateRepository;
use crate::adapters::database::Database;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use daypress_ast::{
    compile_layout_css, render_layout_to_html, sanitize_ssr_html, DaypressLayout, DaypressNode,
    NavMenuItem, NodeKind, SymbolOverride,
};
use std::collections::HashMap;

pub struct HydrationContext<'a> {
    pub db: Option<&'a Database>,
    pub post: Option<&'a DaypressPostResponse>,
    pub settings: &'a DaypressSettingsResponse,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub nav_menus: HashMap<String, Vec<NavMenuItem>>,
    pub archive_title: Option<String>,
    pub archive_subtitle: Option<String>,
    pub archive_posts: Vec<DaypressPostResponse>,
    pub pagination_html: Option<String>,
    pub symbol_depth: u8,
    pub collection_tax: Option<String>,
    pub collection_term: Option<String>,
    pub collection_sort: Option<String>,
    pub collection_search: Option<String>,
    pub collection_page: Option<usize>,
}

pub struct SsrInjector;

impl SsrInjector {
    /// Hydrates a post or page's own `layout_json` (containing `LoopGrid`, `SymbolInstance`, or `{{acf.*}}` / `{{post.*}}` tokens)
    /// so canvas-built pages render live SQLite collections and dynamic fields on SSR.
    pub fn hydrate_post_layout_json(
        db: &Database,
        req_ctx: &RequestContext,
        settings: &DaypressSettingsResponse,
        post: Option<&DaypressPostResponse>,
        layout_json: Option<&str>,
    ) -> Option<String> {
        Self::hydrate_post_layout_json_with_query(
            db,
            req_ctx,
            settings,
            post,
            layout_json,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn hydrate_post_layout_json_with_query(
        db: &Database,
        req_ctx: &RequestContext,
        settings: &DaypressSettingsResponse,
        post: Option<&DaypressPostResponse>,
        layout_json: Option<&str>,
        dp_tax: Option<String>,
        dp_term: Option<String>,
        dp_sort: Option<String>,
        dp_q: Option<String>,
        dp_page: Option<usize>,
    ) -> Option<String> {
        let raw = layout_json?;
        let mut layout: DaypressLayout = serde_json::from_str(raw).ok()?;
        let mut hyd_ctx = Self::create_hydration_context(req_ctx, settings, post);
        hyd_ctx.db = Some(db);
        hyd_ctx.collection_tax = dp_tax;
        hyd_ctx.collection_term = dp_term;
        hyd_ctx.collection_sort = dp_sort;
        hyd_ctx.collection_search = dp_q;
        hyd_ctx.collection_page = dp_page;
        Self::hydrate_layout(&mut layout, &hyd_ctx);
        serde_json::to_string(&layout).ok()
    }
    /// Resolves active Header, Footer, and layout templates, hydrates dynamic placeholders,
    /// and compiles scoped responsive stylesheets.
    pub fn resolve_and_hydrate_partials(
        db: &Database,
        req_ctx: &RequestContext,
        settings: &DaypressSettingsResponse,
    ) -> ResolvedThemePartials {
        Self::resolve_and_hydrate_partials_with_post(db, req_ctx, settings, None)
    }

    /// Resolves active Header, Footer, Single, 404, and Popup templates with an optional active post,
    /// hydrates dynamic placeholders, and compiles scoped responsive stylesheets.
    pub fn resolve_and_hydrate_partials_with_post(
        db: &Database,
        req_ctx: &RequestContext,
        settings: &DaypressSettingsResponse,
        post: Option<&DaypressPostResponse>,
    ) -> ResolvedThemePartials {
        let mut hyd_ctx = Self::create_hydration_context(req_ctx, settings, post);
        hyd_ctx.db = Some(db);
        Self::resolve_and_hydrate_with_context(db, req_ctx, post, &hyd_ctx)
    }

    /// Resolves active Header, Footer, Archive, and Popup templates for archive routes,
    /// hydrating `{{archive.title}}`, `{{archive.description}}`, and `LoopGrid` nodes with paginated posts.
    pub fn resolve_and_hydrate_archive_partials(
        db: &Database,
        req_ctx: &RequestContext,
        settings: &DaypressSettingsResponse,
        archive_title: &str,
        archive_subtitle: &str,
        posts: &[DaypressPostResponse],
        pagination_html: &str,
    ) -> ResolvedThemePartials {
        let mut hyd_ctx = Self::create_hydration_context(req_ctx, settings, None);
        hyd_ctx.db = Some(db);
        hyd_ctx.archive_title = Some(archive_title.to_string());
        hyd_ctx.archive_subtitle = Some(archive_subtitle.to_string());
        hyd_ctx.archive_posts = posts.to_vec();
        hyd_ctx.pagination_html = if pagination_html.trim().is_empty() {
            None
        } else {
            Some(pagination_html.to_string())
        };
        Self::resolve_and_hydrate_with_context(db, req_ctx, None, &hyd_ctx)
    }

    pub fn resolve_and_hydrate_with_context(
        db: &Database,
        req_ctx: &RequestContext,
        post: Option<&DaypressPostResponse>,
        hyd_ctx: &HydrationContext,
    ) -> ResolvedThemePartials {
        let repo = ThemeTemplateRepository::from_database(db);
        let mut partials = ResolvedThemePartials::default();

        // 1. Resolve Header Template (checking per-post & per-route overrides first)
        let header_override = super::matrix::get_post_slot_override(db, post, TemplateType::Header)
            .or_else(|| {
                super::matrix::context_route_keys(req_ctx)
                    .into_iter()
                    .find_map(|rk| {
                        super::matrix::get_route_slot_override(db, rk, TemplateType::Header)
                    })
            });

        if header_override.as_deref() == Some("none") {
            partials.header_hidden = true;
        } else if header_override.as_deref() == Some("fallback") {
            partials.header_force_fallback = true;
        } else if let Ok(headers) = repo.find_active_by_type(TemplateType::Header) {
            if !headers.is_empty() {
                partials.has_theme_builder_header = true;
                if let Some(tpl) =
                    resolve_template_for_type(&headers, TemplateType::Header, req_ctx)
                {
                    let mut layout = tpl.layout.clone();
                    Self::hydrate_layout(&mut layout, hyd_ctx);
                    let raw_html = render_layout_to_html(&layout);
                    let sanitized_html = sanitize_ssr_html(&raw_html);
                    let css = compile_layout_css(&layout);

                    partials.header_html = Some(sanitized_html);
                    partials.header_css = if css.trim().is_empty() {
                        None
                    } else {
                        Some(css)
                    };
                    partials.header_sticky = tpl.settings.sticky;
                    partials.header_transparent = tpl.settings.transparent;
                }
            }
        }

        // 2. Resolve Footer Template (checking per-post & per-route overrides first)
        let footer_override = super::matrix::get_post_slot_override(db, post, TemplateType::Footer)
            .or_else(|| {
                super::matrix::context_route_keys(req_ctx)
                    .into_iter()
                    .find_map(|rk| {
                        super::matrix::get_route_slot_override(db, rk, TemplateType::Footer)
                    })
            });

        if footer_override.as_deref() == Some("none") {
            partials.footer_hidden = true;
        } else if footer_override.as_deref() == Some("fallback") {
            partials.footer_force_fallback = true;
        } else if let Ok(footers) = repo.find_active_by_type(TemplateType::Footer) {
            if !footers.is_empty() {
                partials.has_theme_builder_footer = true;
                if let Some(tpl) =
                    resolve_template_for_type(&footers, TemplateType::Footer, req_ctx)
                {
                    let mut layout = tpl.layout.clone();
                    Self::hydrate_layout(&mut layout, hyd_ctx);
                    let raw_html = render_layout_to_html(&layout);
                    let sanitized_html = sanitize_ssr_html(&raw_html);
                    let css = compile_layout_css(&layout);

                    partials.footer_html = Some(sanitized_html);
                    partials.footer_css = if css.trim().is_empty() {
                        None
                    } else {
                        Some(css)
                    };
                }
            }
        }

        // 3. Resolve SinglePost / SinglePage Template when rendering a post or page
        if let Some(p) = post {
            let single_type = if p.post_type == "page" {
                TemplateType::SinglePage
            } else {
                TemplateType::SinglePost
            };
            let single_override = super::matrix::get_post_slot_override(db, post, single_type)
                .or_else(|| {
                    super::matrix::context_route_keys(req_ctx)
                        .into_iter()
                        .find_map(|rk| super::matrix::get_route_slot_override(db, rk, single_type))
                });

            if single_override.as_deref() != Some("none")
                && single_override.as_deref() != Some("fallback")
            {
                if let Ok(singles) = repo.find_active_by_type(single_type) {
                    if let Some(tpl) = resolve_template_for_type(&singles, single_type, req_ctx) {
                        let mut layout = tpl.layout.clone();
                        Self::hydrate_layout(&mut layout, hyd_ctx);
                        let raw_html = render_layout_to_html(&layout);
                        let sanitized_html = sanitize_ssr_html(&raw_html);
                        let css = compile_layout_css(&layout);

                        if !sanitized_html.trim().is_empty() {
                            partials.single_html = Some(sanitized_html);
                            partials.single_css = if css.trim().is_empty() {
                                None
                            } else {
                                Some(css)
                            };
                        }
                    }
                }
            }
        }

        // 4. Resolve Archive Template when rendering an archive route
        if req_ctx.is_archive {
            if let Ok(archives) = repo.find_active_by_type(TemplateType::Archive) {
                if let Some(tpl) =
                    resolve_template_for_type(&archives, TemplateType::Archive, req_ctx)
                {
                    let mut layout = tpl.layout.clone();
                    Self::hydrate_layout(&mut layout, hyd_ctx);
                    let raw_html = render_layout_to_html(&layout);
                    let sanitized_html = sanitize_ssr_html(&raw_html);
                    let css = compile_layout_css(&layout);

                    if !sanitized_html.trim().is_empty() {
                        partials.archive_html = Some(sanitized_html);
                        partials.archive_css = if css.trim().is_empty() {
                            None
                        } else {
                            Some(css)
                        };
                    }
                }
            }
        }

        // 5. Resolve Error404 Template when rendering a 404 route
        if req_ctx.is_404 {
            if let Ok(errors) = repo.find_active_by_type(TemplateType::Error404) {
                if let Some(tpl) =
                    resolve_template_for_type(&errors, TemplateType::Error404, req_ctx)
                {
                    let mut layout = tpl.layout.clone();
                    Self::hydrate_layout(&mut layout, hyd_ctx);
                    let raw_html = render_layout_to_html(&layout);
                    let sanitized_html = sanitize_ssr_html(&raw_html);
                    let css = compile_layout_css(&layout);

                    if !sanitized_html.trim().is_empty() {
                        partials.error_404_html = Some(sanitized_html);
                        partials.error_404_css = if css.trim().is_empty() {
                            None
                        } else {
                            Some(css)
                        };
                    }
                }
            }
        }

        // 6. Resolve all matching active Popup / Announcement Bar / Slide-In templates
        if let Ok(popups) = repo.find_active_by_type(TemplateType::Popup) {
            let mut combined_html = String::new();
            let mut combined_css = String::new();
            let mut matched_any = false;

            for tpl in &popups {
                if evaluate_template(tpl, req_ctx).is_some() {
                    let mut layout = tpl.layout.clone();
                    Self::hydrate_layout(&mut layout, hyd_ctx);
                    let raw_html = render_layout_to_html(&layout);
                    let sanitized_inner = sanitize_ssr_html(&raw_html);
                    if sanitized_inner.trim().is_empty() {
                        continue;
                    }
                    matched_any = true;
                    let css = compile_layout_css(&layout);
                    if !css.trim().is_empty() {
                        combined_css.push_str(&css);
                        combined_css.push('\n');
                    }

                    let variant = tpl.settings.popup_variant.as_deref().unwrap_or("modal");
                    let dismiss_days = tpl.settings.dismiss_days.unwrap_or(7);
                    let settings_val = serde_json::to_value(&tpl.settings).unwrap_or_default();
                    let trigger = settings_val
                        .get("popup_trigger")
                        .or_else(|| settings_val.get("trigger"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("load");
                    let delay = settings_val
                        .get("popup_delay_ms")
                        .or_else(|| settings_val.get("trigger_delay_ms"))
                        .or_else(|| settings_val.get("delay_ms"))
                        .or_else(|| settings_val.get("delay"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let scroll_pct = settings_val
                        .get("popup_scroll_pct")
                        .or_else(|| settings_val.get("trigger_scroll_pct"))
                        .or_else(|| settings_val.get("scroll_pct"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(50);

                    combined_html.push_str(&Self::wrap_popup_container(
                        tpl.id,
                        variant,
                        trigger,
                        delay,
                        scroll_pct,
                        dismiss_days,
                        &sanitized_inner,
                    ));
                }
            }

            if matched_any {
                let base_popup_css = Self::popup_base_css();
                partials.popups_css = Some(format!("{}\n{}", base_popup_css, combined_css.trim()));
                combined_html.push_str(Self::popup_controller_script());
                partials.popups_html = Some(combined_html);
            }
        }

        partials
    }

    fn wrap_popup_container(
        id: i64,
        variant: &str,
        trigger: &str,
        delay: u64,
        scroll_pct: u64,
        dismiss_days: u32,
        inner_html: &str,
    ) -> String {
        let normalized_variant = match variant {
            "announcement_bar_top"
            | "announcement_bar_bottom"
            | "slide_in_right"
            | "slide_in_left"
            | "modal" => variant,
            _ => "modal",
        };
        let role_attrs = if normalized_variant == "modal" {
            r#"role="dialog" aria-modal="true""#
        } else {
            r#"role="region""#
        };
        format!(
            r#"<div id="dp-popup-{id}" class="dp-popup-container dp-popup-{variant}" data-popup-id="{id}" data-popup-variant="{variant}" data-trigger="{trigger}" data-delay="{delay}" data-scroll-pct="{scroll_pct}" data-dismiss-days="{dismiss_days}" {role_attrs} style="display:none;"><div class="dp-popup-surface">{inner}<button type="button" class="dp-popup-close" aria-label="Close" data-dp-popup-close="{id}">&times;</button></div></div>"#,
            id = id,
            variant = html_escape(normalized_variant),
            trigger = html_escape(trigger),
            delay = delay,
            scroll_pct = scroll_pct,
            dismiss_days = dismiss_days,
            role_attrs = role_attrs,
            inner = inner_html,
        )
    }

    fn popup_base_css() -> &'static str {
        r#".dp-popup-container{box-sizing:border-box;z-index:99990}
.dp-popup-modal{position:fixed;inset:0;background:rgba(0,0,0,0.6);align-items:center;justify-content:center;padding:20px}
.dp-popup-modal .dp-popup-surface{position:relative;max-width:600px;width:100%;border-radius:12px;overflow:hidden;box-shadow:0 24px 60px rgba(0,0,0,0.4)}
.dp-popup-announcement_bar_top{position:sticky;top:0;left:0;right:0;width:100%;z-index:99991}
.dp-popup-announcement_bar_bottom{position:fixed;bottom:0;left:0;right:0;width:100%;z-index:99991}
.dp-popup-announcement_bar_top .dp-popup-surface,.dp-popup-announcement_bar_bottom .dp-popup-surface{position:relative;width:100%}
.dp-popup-slide_in_right{position:fixed;right:20px;bottom:20px;max-width:380px;width:calc(100% - 40px);z-index:99991}
.dp-popup-slide_in_left{position:fixed;left:20px;bottom:20px;max-width:380px;width:calc(100% - 40px);z-index:99991}
.dp-popup-slide_in_right .dp-popup-surface,.dp-popup-slide_in_left .dp-popup-surface{position:relative;border-radius:12px;overflow:hidden;box-shadow:0 16px 40px rgba(0,0,0,0.35)}
.dp-popup-close{position:absolute;top:8px;right:10px;background:rgba(0,0,0,0.25);color:#fff;border:none;width:26px;height:26px;border-radius:999px;cursor:pointer;font-size:16px;line-height:1;display:flex;align-items:center;justify-content:center}"#
    }

    fn popup_controller_script() -> &'static str {
        r#"<script>
(function(){
	function isDismissed(id){
		try{
			var raw=localStorage.getItem('dp_popup_dismissed_'+id);
			if(!raw)return false;
			if(raw==='1'||raw==='true')return true;
			var exp=parseInt(raw,10);
			if(!isNaN(exp)&&exp>Date.now())return true;
			localStorage.removeItem('dp_popup_dismissed_'+id);
		}catch(e){}
		return false;
	}
	window.dpDismissPopup=function(id,days){
		try{
			var d=typeof days==='number'&&days>0?days:7;
			localStorage.setItem('dp_popup_dismissed_'+id,String(Date.now()+d*86400000));
		}catch(e){}
		var el=document.getElementById('dp-popup-'+id);
		if(el)el.style.display='none';
	};
	function showPopup(el){
		var variant=el.getAttribute('data-popup-variant')||'modal';
		el.style.display=(variant==='modal')?'flex':'block';
	}
	document.querySelectorAll('.dp-popup-container[data-popup-id]').forEach(function(el){
		var id=el.getAttribute('data-popup-id');
		if(!id||isDismissed(id))return;
		var trigger=el.getAttribute('data-trigger')||'load';
		var delay=parseInt(el.getAttribute('data-delay')||'0',10)||0;
		var scrollPct=parseInt(el.getAttribute('data-scroll-pct')||'50',10)||50;
		var dismissDays=parseInt(el.getAttribute('data-dismiss-days')||'7',10)||7;
		el.querySelectorAll('[data-dp-popup-close]').forEach(function(btn){
			btn.addEventListener('click',function(){window.dpDismissPopup(id,dismissDays);});
		});
		if(trigger==='scroll'){
			var shown=false;
			window.addEventListener('scroll',function(){
				if(shown||isDismissed(id))return;
				var docH=Math.max(document.documentElement.scrollHeight-window.innerHeight,1);
				if((window.scrollY/docH)*100>=scrollPct){shown=true;showPopup(el);}
			},{passive:true});
		}else if(trigger==='exit_intent'){
			var shownExit=false;
			document.addEventListener('mouseout',function(e){
				if(!shownExit&&!isDismissed(id)&&e.clientY<=0){shownExit=true;showPopup(el);}
			});
		}else{
			setTimeout(function(){if(!isDismissed(id))showPopup(el);},delay);
		}
	});
})();
</script>"#
    }

    /// Recursively traverses the AST layout and hydrates dynamic nodes in-place.
    pub fn hydrate_layout(layout: &mut DaypressLayout, ctx: &HydrationContext) {
        Self::hydrate_node(&mut layout.root, ctx);
    }

    /// Hydrates a single node and recursively processes all child nodes.
    pub fn hydrate_node(node: &mut DaypressNode, ctx: &HydrationContext) {
        // 0. Hydrate Synced Global Symbol (`NodeKind::SymbolInstance`)
        if let NodeKind::SymbolInstance {
            symbol_id,
            overrides,
        } = &node.kind
        {
            if ctx.symbol_depth < 4 {
                let sym_id = *symbol_id;
                let ov_map = overrides.clone();
                let mut symbol_children = if let Some(db) = ctx.db {
                    if let Ok(Some(tpl)) =
                        ThemeTemplateRepository::from_database(db).get_by_id(sym_id)
                    {
                        tpl.layout.root.children.clone()
                    } else {
                        node.children.clone()
                    }
                } else {
                    node.children.clone()
                };
                Self::apply_symbol_overrides(&mut symbol_children, &ov_map);
                let nested_ctx = HydrationContext {
                    db: ctx.db,
                    post: ctx.post,
                    settings: ctx.settings,
                    breadcrumbs: ctx.breadcrumbs.clone(),
                    nav_menus: ctx.nav_menus.clone(),
                    archive_title: ctx.archive_title.clone(),
                    archive_subtitle: ctx.archive_subtitle.clone(),
                    archive_posts: ctx.archive_posts.clone(),
                    pagination_html: ctx.pagination_html.clone(),
                    symbol_depth: ctx.symbol_depth + 1,
                    collection_tax: ctx.collection_tax.clone(),
                    collection_term: ctx.collection_term.clone(),
                    collection_sort: ctx.collection_sort.clone(),
                    collection_search: ctx.collection_search.clone(),
                    collection_page: ctx.collection_page,
                };
                for child in &mut symbol_children {
                    Self::hydrate_node(child, &nested_ctx);
                }
                node.children = symbol_children;
                return;
            }
        }

        // A. Hydrate NavMenu & Mega Menus
        if let NodeKind::NavMenu {
            menu_slug, items, ..
        } = &mut node.kind
        {
            if items.is_empty() {
                if let Some(cached) = ctx.nav_menus.get(menu_slug.as_str()) {
                    *items = cached.clone();
                } else if !ctx.settings.nav_menu_items.is_empty() {
                    *items = ctx
                        .settings
                        .nav_menu_items
                        .iter()
                        .enumerate()
                        .map(|(idx, it)| NavMenuItem {
                            id: it.id,
                            title: it.title.clone(),
                            url: it.url.clone(),
                            target: it.target.clone(),
                            parent: it.parent,
                            order: idx as i64,
                            icon: it.icon.clone(),
                            badge: it.badge.clone(),
                            description: it.description.clone(),
                            mega_menu_template_id: it.mega_menu_template_id,
                            ..Default::default()
                        })
                        .collect();
                }
            }
            Self::hydrate_nav_menu_items(items, ctx);
        }

        // B. Hydrate Breadcrumbs
        if let NodeKind::Breadcrumbs {
            separator,
            home_text,
        } = &node.kind
        {
            let markup = Self::render_breadcrumbs_html(&ctx.breadcrumbs, separator, home_text);
            node.kind = NodeKind::CustomHtml { raw: markup };
        }

        // B2. Server-Side LoopGrid / Filterable CollectionGrid Hydration
        if let NodeKind::LoopGrid {
            post_type,
            posts_per_page,
            columns,
            show_thumbnail,
            show_excerpt,
            show_meta,
            show_pagination,
            taxonomy,
            enable_filter_bar,
            enable_sort,
            enable_search,
            default_sort,
            card_skin,
            meta_fields,
        } = &node.kind
        {
            let has_interactive_toolbar = *enable_filter_bar || *enable_sort || *enable_search;
            let raw = if let Some(db) = ctx.db {
                if ctx.archive_posts.is_empty()
                    || has_interactive_toolbar
                    || ctx.collection_tax.is_some()
                    || ctx.collection_term.is_some()
                    || ctx.collection_sort.is_some()
                    || ctx.collection_search.is_some()
                {
                    Self::render_collection_grid_ssr(
                        db,
                        &node.id,
                        post_type,
                        *posts_per_page,
                        *columns,
                        *show_thumbnail,
                        *show_excerpt,
                        *show_meta,
                        *show_pagination,
                        taxonomy.as_deref(),
                        *enable_filter_bar,
                        *enable_sort,
                        *enable_search,
                        default_sort,
                        card_skin,
                        meta_fields,
                        ctx,
                    )
                } else {
                    Self::render_loop_grid_html_with_skin(
                        &ctx.archive_posts,
                        *columns,
                        *show_thumbnail,
                        *show_excerpt,
                        *show_meta,
                        *show_pagination,
                        ctx.pagination_html.as_deref(),
                        card_skin,
                        meta_fields,
                    )
                }
            } else {
                Self::render_loop_grid_html_with_skin(
                    &ctx.archive_posts,
                    *columns,
                    *show_thumbnail,
                    *show_excerpt,
                    *show_meta,
                    *show_pagination,
                    ctx.pagination_html.as_deref(),
                    card_skin,
                    meta_fields,
                )
            };
            node.kind = NodeKind::CustomHtml { raw };
        }

        // C. Dynamic tag detection
        let dynamic_tag: Option<String> = node
            .styles
            .raw_settings
            .as_ref()
            .and_then(|m| {
                m.get("data-dynamic")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .or_else(|| {
                let lower_id = node.id.to_lowercase();
                if lower_id.contains("post-title") || lower_id.contains("post_title") {
                    Some("post_title".to_string())
                } else if lower_id.contains("featured-image") || lower_id.contains("featured_image")
                {
                    Some("featured_image".to_string())
                } else if lower_id.contains("post-content") || lower_id.contains("post_content") {
                    Some("post_content".to_string())
                } else if lower_id.contains("post-date") || lower_id.contains("post_date") {
                    Some("post_date".to_string())
                } else if lower_id.contains("author-box") || lower_id.contains("author_box") {
                    Some("author_box".to_string())
                } else if lower_id.contains("site-logo") || lower_id.contains("site_logo") {
                    Some("site_logo".to_string())
                } else if lower_id.contains("site-title") || lower_id.contains("site_title") {
                    Some("site_title".to_string())
                } else {
                    None
                }
            });

        if let Some(ref tag) = dynamic_tag {
            Self::apply_dynamic_tag(node, tag, ctx);
        }

        // D. General token interpolation on text, image, and button nodes
        Self::interpolate_text_tokens(node, ctx);

        // Recursively hydrate children
        for child in &mut node.children {
            Self::hydrate_node(child, ctx);
        }
    }

    /// Recursively applies per-instance overrides (`text`, `src`, `alt`, `url`) to matching child node IDs in a symbol tree.
    pub fn apply_symbol_overrides(
        nodes: &mut [DaypressNode],
        overrides: &HashMap<String, SymbolOverride>,
    ) {
        if overrides.is_empty() {
            return;
        }
        for node in nodes.iter_mut() {
            if let Some(ov) = overrides.get(&node.id) {
                if let Some(ref txt) = ov.text {
                    match &mut node.kind {
                        NodeKind::Heading { text, .. } => *text = txt.clone(),
                        NodeKind::Text { content_html } => *content_html = txt.clone(),
                        NodeKind::Button { text, .. } => *text = txt.clone(),
                        _ => {}
                    }
                }
                if let Some(ref new_src) = ov.src {
                    if let NodeKind::Image { src, .. } = &mut node.kind {
                        *src = new_src.clone();
                    }
                }
                if let Some(ref new_alt) = ov.alt {
                    if let NodeKind::Image { alt, .. } = &mut node.kind {
                        *alt = new_alt.clone();
                    }
                }
                if let Some(ref new_url) = ov.url {
                    match &mut node.kind {
                        NodeKind::Button { url, .. } => *url = new_url.clone(),
                        NodeKind::Image { link, .. } => *link = Some(new_url.clone()),
                        _ => {}
                    }
                }
            }
            Self::apply_symbol_overrides(&mut node.children, overrides);
        }
    }

    fn hydrate_nav_menu_items(items: &mut [NavMenuItem], ctx: &HydrationContext) {
        for item in items.iter_mut() {
            if item.mega_menu_template_id.is_none()
                && item.icon.is_none()
                && item.badge.is_none()
                && item.description.is_none()
            {
                if let Some(matched) = ctx.settings.nav_menu_items.iter().find(|it| {
                    (item.id > 0 && it.id == item.id)
                        || (it.title == item.title && it.url == item.url)
                }) {
                    item.icon = matched.icon.clone();
                    item.badge = matched.badge.clone();
                    item.description = matched.description.clone();
                    item.mega_menu_template_id = matched.mega_menu_template_id;
                }
            }

            if let Some(tpl_id) = item.mega_menu_template_id {
                if tpl_id > 0 && ctx.symbol_depth < 4 {
                    if let Some(db) = ctx.db {
                        if let Ok(Some(tpl)) =
                            ThemeTemplateRepository::from_database(db).get_by_id(tpl_id)
                        {
                            let mut mega_layout = tpl.layout.clone();
                            let nested_ctx = HydrationContext {
                                db: ctx.db,
                                post: ctx.post,
                                settings: ctx.settings,
                                breadcrumbs: ctx.breadcrumbs.clone(),
                                nav_menus: ctx.nav_menus.clone(),
                                archive_title: ctx.archive_title.clone(),
                                archive_subtitle: ctx.archive_subtitle.clone(),
                                archive_posts: ctx.archive_posts.clone(),
                                pagination_html: ctx.pagination_html.clone(),
                                symbol_depth: ctx.symbol_depth + 1,
                                collection_tax: ctx.collection_tax.clone(),
                                collection_term: ctx.collection_term.clone(),
                                collection_sort: ctx.collection_sort.clone(),
                                collection_search: ctx.collection_search.clone(),
                                collection_page: ctx.collection_page,
                            };
                            Self::hydrate_layout(&mut mega_layout, &nested_ctx);
                            let raw_html = render_layout_to_html(&mega_layout);
                            let sanitized_html = sanitize_ssr_html(&raw_html);
                            let css = compile_layout_css(&mega_layout);
                            let combined = if css.trim().is_empty() {
                                sanitized_html
                            } else {
                                format!("<style>{}</style>{}", css, sanitized_html)
                            };
                            item.mega_menu_html = Some(combined);
                        }
                    }
                }
            }

            if !item.children.is_empty() {
                Self::hydrate_nav_menu_items(&mut item.children, ctx);
            }
        }
    }

    pub fn render_loop_grid_html(
        posts: &[DaypressPostResponse],
        columns: u8,
        show_thumbnail: bool,
        show_excerpt: bool,
        show_meta: bool,
        show_pagination: bool,
        pagination_html: Option<&str>,
    ) -> String {
        Self::render_loop_grid_html_with_skin(
            posts,
            columns,
            show_thumbnail,
            show_excerpt,
            show_meta,
            show_pagination,
            pagination_html,
            "editorial",
            &[],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_loop_grid_html_with_skin(
        posts: &[DaypressPostResponse],
        columns: u8,
        show_thumbnail: bool,
        show_excerpt: bool,
        show_meta: bool,
        show_pagination: bool,
        pagination_html: Option<&str>,
        card_skin: &str,
        meta_fields: &[String],
    ) -> String {
        let cols = columns.clamp(1, 6);
        let skin = if card_skin.trim().is_empty() {
            "editorial"
        } else {
            card_skin.trim()
        };
        let cards_html = Self::render_collection_cards_inner(
            posts,
            show_thumbnail,
            show_excerpt,
            show_meta,
            skin,
            meta_fields,
        );

        let pag = if show_pagination {
            pagination_html.unwrap_or("")
        } else {
            ""
        };

        format!(
            r#"<div class="dp-loop-grid dp-grid" data-columns="{cols}" data-skin="{skin}" style="display: grid; grid-template-columns: repeat({cols}, minmax(0, 1fr)); gap: 24px;">{cards_html}</div>{pag}"#,
            cols = cols,
            skin = html_escape(skin),
            cards_html = cards_html,
            pag = pag
        )
    }

    pub fn render_collection_cards_inner(
        posts: &[DaypressPostResponse],
        show_thumbnail: bool,
        show_excerpt: bool,
        show_meta: bool,
        skin: &str,
        meta_fields: &[String],
    ) -> String {
        let mut cards_html = String::new();
        if posts.is_empty() {
            cards_html.push_str(
                r#"<div class="dp-loop-grid-empty" style="grid-column: 1 / -1; text-align: center; padding: 48px 24px; background: var(--bg-card, var(--paper-raised)); border: 1px solid var(--border, var(--rule)); border-radius: 8px; color: var(--text-muted, var(--ink-muted));">No matching items found.</div>"#,
            );
            return cards_html;
        }

        for post in posts {
            let resolved_img_url = post
                .featured_image_url
                .as_deref()
                .filter(|u| !u.is_empty())
                .map(|u| u.to_string())
                .or_else(|| {
                    let meta_obj = post.meta.as_object()?;
                    for key in [
                        "banner_image",
                        "profile_img",
                        "header_image",
                        "_thumbnail_id",
                        "image",
                    ] {
                        if let Some(val) = meta_obj.get(key).and_then(|v| v.as_str()) {
                            if val.starts_with("http://")
                                || val.starts_with("https://")
                                || val.starts_with('/')
                            {
                                return Some(val.to_string());
                            }
                        }
                    }
                    None
                });

            let thumb_html = if show_thumbnail && skin != "minimal" {
                if let Some(ref img_url) = resolved_img_url {
                    if !img_url.is_empty() {
                        let thumb_style = if skin == "horizontal" {
                            "width: 180px; min-height: 150px; flex-shrink: 0; overflow: hidden;"
                        } else {
                            "overflow: hidden; border-radius: 6px 6px 0 0;"
                        };
                        format!(
                            r#"<div class="dp-card-thumb" style="{thumb_style}"><img src="{}" alt="{}" class="dp-image" style="width: 100%; height: 200px; object-fit: cover; display: block;" loading="lazy" /></div>"#,
                            html_escape(img_url),
                            html_escape(&post.title.rendered)
                        )
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let meta_html = if show_meta {
                let date_short: String = post.date.chars().take(10).collect();
                let author_display = if post.author_name.is_empty() {
                    "admin".to_string()
                } else {
                    html_escape(&post.author_name)
                };
                format!(
                    r#"<div class="dp-card-meta"><span class="dp-badge">{}</span><span>&bull;</span><span>{}</span><span>&bull;</span><span>{}</span></div>"#,
                    html_escape(&post.post_type),
                    html_escape(&date_short),
                    author_display
                )
            } else {
                String::new()
            };

            // Structured field badges from post.acf / post.meta
            let mut structured_badges = String::new();
            if let Some(acf_obj) = post.acf.as_object() {
                let keys_to_show: Vec<String> = if !meta_fields.is_empty() {
                    meta_fields.to_vec()
                } else {
                    acf_obj
                        .keys()
                        .filter(|k| {
                            !k.starts_with('_')
                                && !matches!(
                                    k.as_str(),
                                    "header_height"
                                        | "header_hoogte"
                                        | "show_title"
                                        | "header_title"
                                        | "show_icon"
                                        | "header_shape"
                                        | "header_left"
                                )
                        })
                        .take(3)
                        .cloned()
                        .collect()
                };
                let mut chips = Vec::new();
                for k in keys_to_show {
                    if let Some(val) = acf_obj.get(&k) {
                        let formatted = format_structured_field_scalar(val);
                        if !formatted.is_empty() && formatted.chars().count() <= 48 {
                            chips.push(format!(
                                r#"<span class="dp-acf-chip" data-field="{}" style="display:inline-flex;align-items:center;padding:2px 8px;border-radius:3px;font-size:11px;font-family:var(--font-mono, monospace);background:var(--paper-sunk, rgba(0,0,0,0.05));border:1px solid var(--rule, rgba(0,0,0,0.1));color:var(--ink, inherit);">{}</span>"#,
                                html_escape(&k),
                                html_escape(&formatted)
                            ));
                        }
                    }
                }
                if !chips.is_empty() {
                    structured_badges = format!(
                        r#"<div class="dp-card-structured-fields" style="display:flex;flex-wrap:wrap;gap:6px;margin:8px 0;">{}</div>"#,
                        chips.join("")
                    );
                }
            }

            let excerpt_html = if show_excerpt {
                let source = if !post.excerpt.rendered.trim().is_empty() {
                    &post.excerpt.rendered
                } else {
                    &post.content.rendered
                };
                let excerpt = plain_excerpt_chars(source, 140);
                format!(
                    r#"<p class="dp-card-excerpt">{}</p>"#,
                    html_escape(&excerpt)
                )
            } else {
                String::new()
            };

            let card_cls = if post.sticky {
                format!("dp-card dp-card-skin-{} sticky", html_escape(skin))
            } else {
                format!("dp-card dp-card-skin-{}", html_escape(skin))
            };

            let card_inline_style = match skin {
                "horizontal" => "display: flex; flex-direction: row; align-items: stretch;",
                "minimal" => "border: none; border-bottom: 1px solid var(--rule, #16171a2e); border-radius: 0; background: transparent; padding: 16px 0;",
                "boxed" => "border: 1px solid var(--rule-strong, #16171a); border-radius: var(--radius-md, 6px); background: var(--paper-raised, #fbf8f2);",
                "overlay" => "position: relative; border: 1px solid var(--rule-strong, #16171a); border-radius: var(--radius-md, 6px); background: var(--ink, #16171a); color: var(--paper, #f3eee4); overflow: hidden;",
                _ => "border-top: 1px solid var(--rule-strong, #16171a); background: var(--paper-raised, #fbf8f2);",
            };

            let href = if post.post_type != "post" && post.post_type != "page" {
                format!("/{}/{}/", post.post_type, post.slug)
            } else if !post.link.is_empty() {
                post.link.clone()
            } else {
                format!("/{}/", post.slug)
            };

            cards_html.push_str(&format!(
                r#"<a href="{href}" class="{card_cls}" style="{card_inline_style}">{thumb_html}<div class="dp-card-body">{meta_html}<h3 class="dp-card-title">{title}</h3>{structured_badges}{excerpt_html}<div class="dp-card-link">Read more &rarr;</div></div></a>"#,
                href = html_escape(&href),
                card_cls = card_cls,
                card_inline_style = card_inline_style,
                thumb_html = thumb_html,
                meta_html = meta_html,
                title = post.title.rendered,
                structured_badges = structured_badges,
                excerpt_html = excerpt_html,
            ));
        }

        cards_html
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_collection_grid_ssr(
        db: &Database,
        node_id: &str,
        post_type: &str,
        posts_per_page: u32,
        columns: u8,
        show_thumbnail: bool,
        show_excerpt: bool,
        show_meta: bool,
        show_pagination: bool,
        taxonomy: Option<&str>,
        enable_filter_bar: bool,
        enable_sort: bool,
        enable_search: bool,
        default_sort: &str,
        card_skin: &str,
        meta_fields: &[String],
        ctx: &HydrationContext,
    ) -> String {
        let cols = columns.clamp(1, 6);
        let per_page = posts_per_page.clamp(1, 50) as usize;
        let skin = if card_skin.trim().is_empty() {
            "editorial"
        } else {
            card_skin.trim()
        };

        // Resolve filter taxonomy
        let resolved_tax: Option<String> = taxonomy
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| ctx.collection_tax.clone())
            .or_else(|| {
                if enable_filter_bar {
                    db.get_taxonomies_for_post_type(post_type, None)
                        .ok()
                        .and_then(|list| list.into_iter().next().map(|t| t.slug))
                } else {
                    None
                }
            });

        let active_term = ctx.collection_term.as_deref();
        let active_search = ctx.collection_search.as_deref();
        let active_sort = ctx
            .collection_sort
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(if default_sort.is_empty() {
                "date_desc"
            } else {
                default_sort
            });
        let active_page = ctx.collection_page.unwrap_or(1).max(1);

        let (posts, total_items, resolved_filter_tax, term_counts) = db
            .query_collection_posts(
                post_type,
                resolved_tax.as_deref(),
                active_term,
                Some(active_sort),
                active_search,
                active_page,
                per_page,
                &ctx.settings.url,
            )
            .unwrap_or_else(|_| (Vec::new(), 0, String::new(), Vec::new()));
        if resolved_tax.is_none() && !resolved_filter_tax.is_empty() {
            // Keep resolved_filter_tax for pill links
        }
        let resolved_tax = resolved_tax.or(if resolved_filter_tax.is_empty() {
            None
        } else {
            Some(resolved_filter_tax)
        });

        // Determine meta_fields: if node didn't specify explicit meta_fields, check FieldGroupDefinition show_in_collection_card
        let effective_meta_fields: Vec<String> = if !meta_fields.is_empty() {
            meta_fields.to_vec()
        } else {
            db.list_field_groups(Some(post_type))
                .unwrap_or_default()
                .into_iter()
                .flat_map(|g| g.fields)
                .filter(|f| f.show_in_collection_card)
                .map(|f| f.name)
                .collect()
        };

        // Build interactive toolbar if enabled
        let mut toolbar_html = String::new();
        if enable_filter_bar || enable_sort || enable_search {
            let mut pills_html = String::new();
            if enable_filter_bar {
                let all_active = active_term.unwrap_or("").is_empty();
                let all_style = if all_active {
                    "background:var(--ink, #16171a);color:var(--paper, #f3eee4);border-color:var(--ink, #16171a);"
                } else {
                    "background:var(--paper-raised, #fbf8f2);color:var(--ink, #16171a);border-color:var(--rule, #16171a2e);"
                };
                pills_html.push_str(&format!(
                    r#"<a href="?" class="dp-filter-pill{}" data-term="" onclick="return window.dpFilterCollectionGrid ? window.dpFilterCollectionGrid(event, '{}', {{term:''}}) : true;" style="display:inline-flex;align-items:center;gap:6px;padding:6px 14px;border-radius:var(--radius-sm, 2px);border:1px solid;font-size:12.5px;font-weight:600;text-decoration:none;transition:all 0.15s ease;{}">All</a>"#,
                    if all_active { " active" } else { "" },
                    html_escape(node_id),
                    all_style
                ));
                let tax_param = resolved_tax.as_deref().unwrap_or("");
                for (t_slug, t_name, t_count) in &term_counts {
                    let is_act = active_term.is_some_and(|a| a == t_slug);
                    let p_style = if is_act {
                        "background:var(--ink, #16171a);color:var(--paper, #f3eee4);border-color:var(--ink, #16171a);"
                    } else {
                        "background:var(--paper-raised, #fbf8f2);color:var(--ink, #16171a);border-color:var(--rule, #16171a2e);"
                    };
                    pills_html.push_str(&format!(
                        r#"<a href="?dp_tax={tax}&amp;dp_term={slug}" class="dp-filter-pill{act}" data-term="{slug}" onclick="return window.dpFilterCollectionGrid ? window.dpFilterCollectionGrid(event, '{nid}', {{term:'{slug}'}}) : true;" style="display:inline-flex;align-items:center;gap:6px;padding:6px 14px;border-radius:var(--radius-sm, 2px);border:1px solid;font-size:12.5px;font-weight:500;text-decoration:none;transition:all 0.15s ease;{style}"><span>{name}</span><span style="font-size:10.5px;opacity:0.7;font-family:var(--font-mono, monospace);">{cnt}</span></a>"#,
                        tax = html_escape(tax_param),
                        slug = html_escape(t_slug),
                        act = if is_act { " active" } else { "" },
                        nid = html_escape(node_id),
                        style = p_style,
                        name = html_escape(t_name),
                        cnt = t_count,
                    ));
                }
            }

            let mut right_controls = String::new();
            if enable_search {
                let q_val = html_escape(active_search.unwrap_or(""));
                right_controls.push_str(&format!(
                    r#"<input type="search" class="dp-collection-search" value="{q_val}" placeholder="Filter items..." oninput="window.dpFilterCollectionGrid && window.dpFilterCollectionGrid(null, '{nid}', {{q: this.value}})" style="height:34px;padding:0 12px;border-radius:var(--radius-sm, 2px);border:1px solid var(--rule-strong, #16171a);background:var(--paper-raised, #fbf8f2);color:var(--ink, #16171a);font-size:12.5px;min-width:180px;" />"#,
                    q_val = q_val,
                    nid = html_escape(node_id),
                ));
            }
            if enable_sort {
                let s_desc = if active_sort == "date_desc" {
                    "selected"
                } else {
                    ""
                };
                let s_asc = if active_sort == "date_asc" {
                    "selected"
                } else {
                    ""
                };
                let s_tasc = if active_sort == "title_asc" {
                    "selected"
                } else {
                    ""
                };
                let s_tdesc = if active_sort == "title_desc" {
                    "selected"
                } else {
                    ""
                };
                right_controls.push_str(&format!(
                    r#"<select class="dp-collection-sort" aria-label="Sort collection" onchange="window.dpFilterCollectionGrid && window.dpFilterCollectionGrid(null, '{nid}', {{sort: this.value}})" style="height:34px;padding:0 10px;border-radius:var(--radius-sm, 2px);border:1px solid var(--rule-strong, #16171a);background:var(--paper-raised, #fbf8f2);color:var(--ink, #16171a);font-size:12.5px;">
                        <option value="date_desc" {s_desc}>Newest First</option>
                        <option value="date_asc" {s_asc}>Oldest First</option>
                        <option value="title_asc" {s_tasc}>Title A–Z</option>
                        <option value="title_desc" {s_tdesc}>Title Z–A</option>
                    </select>"#,
                    nid = html_escape(node_id),
                ));
            }

            toolbar_html = format!(
                r#"<div class="dp-collection-toolbar" style="display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:14px;margin-bottom:24px;padding-bottom:16px;border-bottom:1px solid var(--rule, #16171a2e);">
                    <div class="dp-collection-filters" style="display:flex;flex-wrap:wrap;gap:8px;">{pills_html}</div>
                    <div class="dp-collection-controls" style="display:flex;flex-wrap:wrap;gap:10px;align-items:center;">{right_controls}</div>
                </div>"#
            );
        }

        let cards_html = Self::render_collection_cards_inner(
            &posts,
            show_thumbnail,
            show_excerpt,
            show_meta,
            skin,
            &effective_meta_fields,
        );

        let total_pages = total_items.div_ceil(per_page.max(1)).max(1);
        let pag_html = if show_pagination {
            if let Some(existing_pag) = ctx.pagination_html.as_deref() {
                existing_pag.to_string()
            } else if total_pages > 1 {
                let mut page_links = String::new();
                for p_idx in 1..=total_pages {
                    let is_curr = p_idx == active_page;
                    let p_style = if is_curr {
                        "background:var(--ink, #16171a);color:var(--paper, #f3eee4);border-color:var(--ink, #16171a);"
                    } else {
                        "background:var(--paper-raised, #fbf8f2);color:var(--ink, #16171a);border-color:var(--rule, #16171a2e);"
                    };
                    page_links.push_str(&format!(
                        r#"<a href="?dp_page={p_idx}" class="dp-page-link{curr}" onclick="return window.dpFilterCollectionGrid ? window.dpFilterCollectionGrid(event, '{nid}', {{page:{p_idx}}}) : true;" style="display:inline-flex;align-items:center;justify-content:center;min-width:34px;height:34px;padding:0 10px;border-radius:var(--radius-sm, 2px);border:1px solid;font-size:12.5px;font-family:var(--font-mono, monospace);text-decoration:none;{p_style}">{p_idx}</a>"#,
                        p_idx = p_idx,
                        curr = if is_curr { " active" } else { "" },
                        nid = html_escape(node_id),
                        p_style = p_style,
                    ));
                }
                format!(
                    r#"<nav class="dp-pagination" aria-label="Collection Pagination" style="display:flex;justify-content:center;gap:8px;margin-top:32px;">{page_links}</nav>"#
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let meta_fields_attr = html_escape(&effective_meta_fields.join(","));
        let tax_attr = html_escape(resolved_tax.as_deref().unwrap_or(""));
        let term_attr = html_escape(active_term.unwrap_or(""));
        let sort_attr = html_escape(active_sort);
        let q_attr = html_escape(active_search.unwrap_or(""));

        let controller_script = if enable_filter_bar
            || enable_sort
            || enable_search
            || show_pagination
        {
            r#"<script>
            if (!window.dpFilterCollectionGrid) {
                window.dpFilterCollectionGrid = function(ev, gridId, patch) {
                    if (ev && ev.preventDefault) ev.preventDefault();
                    const wrap = document.getElementById('dp-collection-' + gridId);
                    if (!wrap) return true;
                    if (patch.term !== undefined) { wrap.dataset.term = patch.term; wrap.dataset.page = '1'; }
                    if (patch.sort !== undefined) { wrap.dataset.sort = patch.sort; wrap.dataset.page = '1'; }
                    if (patch.q !== undefined) { wrap.dataset.q = patch.q; wrap.dataset.page = '1'; }
                    if (patch.page !== undefined) { wrap.dataset.page = String(patch.page); }
                    const params = new URLSearchParams({
                        post_type: wrap.dataset.postType || 'post',
                        per_page: wrap.dataset.perPage || '6',
                        columns: wrap.dataset.columns || '3',
                        taxonomy: wrap.dataset.taxonomy || '',
                        term: wrap.dataset.term || '',
                        sort: wrap.dataset.sort || 'date_desc',
                        q: wrap.dataset.q || '',
                        page: wrap.dataset.page || '1',
                        skin: wrap.dataset.skin || 'editorial',
                        meta_fields: wrap.dataset.metaFields || '',
                        show_thumbnail: wrap.dataset.showThumbnail || 'true',
                        show_excerpt: wrap.dataset.showExcerpt || 'true',
                        show_meta: wrap.dataset.showMeta || 'true'
                    });
                    wrap.querySelectorAll('.dp-filter-pill').forEach(p => {
                        const active = (p.getAttribute('data-term') || '') === (wrap.dataset.term || '');
                        p.classList.toggle('active', active);
                        p.style.background = active ? 'var(--ink, #16171a)' : 'var(--paper-raised, #fbf8f2)';
                        p.style.color = active ? 'var(--paper, #f3eee4)' : 'var(--ink, #16171a)';
                    });
                    fetch('/wp-json/daypress/v1/collection-query?' + params.toString())
                        .then(r => r.json())
                        .then(data => {
                            const gridEl = wrap.querySelector('.dp-loop-grid');
                            if (gridEl && data.cards_html !== undefined) {
                                gridEl.outerHTML = data.cards_html;
                            }
                            const url = new URL(window.location.href);
                            if (wrap.dataset.term) { url.searchParams.set('dp_term', wrap.dataset.term); } else { url.searchParams.delete('dp_term'); }
                            if (wrap.dataset.sort && wrap.dataset.sort !== 'date_desc') { url.searchParams.set('dp_sort', wrap.dataset.sort); } else { url.searchParams.delete('dp_sort'); }
                            if (wrap.dataset.q) { url.searchParams.set('dp_q', wrap.dataset.q); } else { url.searchParams.delete('dp_q'); }
                            window.history.replaceState({}, '', url.toString());
                        })
                        .catch(() => {});
                    return false;
                };
            }
            </script>"#
        } else {
            ""
        };

        format!(
            r#"<div id="dp-collection-{nid}" class="dp-collection-container" data-post-type="{pt}" data-per-page="{per_page}" data-columns="{cols}" data-taxonomy="{tax}" data-term="{term}" data-sort="{sort}" data-q="{q}" data-page="{page}" data-skin="{skin}" data-meta-fields="{meta}" data-show-thumbnail="{st}" data-show-excerpt="{se}" data-show-meta="{sm}">{toolbar_html}<div class="dp-loop-grid dp-grid" data-columns="{cols}" data-skin="{skin}" style="display: grid; grid-template-columns: repeat({cols}, minmax(0, 1fr)); gap: 24px;">{cards_html}</div>{pag_html}</div>{controller_script}"#,
            nid = html_escape(node_id),
            pt = html_escape(post_type),
            per_page = per_page,
            cols = cols,
            tax = tax_attr,
            term = term_attr,
            sort = sort_attr,
            q = q_attr,
            page = active_page,
            skin = html_escape(skin),
            meta = meta_fields_attr,
            st = show_thumbnail,
            se = show_excerpt,
            sm = show_meta,
            toolbar_html = toolbar_html,
            cards_html = cards_html,
            pag_html = pag_html,
            controller_script = controller_script,
        )
    }

    fn apply_dynamic_tag(node: &mut DaypressNode, tag: &str, ctx: &HydrationContext) {
        let normalized = tag
            .trim()
            .trim_start_matches("{{")
            .trim_end_matches("}}")
            .trim();
        match normalized {
            "post_title" | "post.title" => {
                let title = ctx
                    .post
                    .map(|p| p.title.rendered.clone())
                    .or_else(|| ctx.archive_title.clone())
                    .unwrap_or_else(|| ctx.settings.title.clone());
                match &mut node.kind {
                    NodeKind::Heading { text, .. } => *text = title,
                    NodeKind::Text { content_html } => *content_html = title,
                    NodeKind::CustomHtml { raw } => *raw = title,
                    _ => {
                        node.kind = NodeKind::Heading {
                            text: title,
                            level: 1,
                        }
                    }
                }
            }
            "featured_image" | "post.featured_image" => {
                let img_url = ctx
                    .post
                    .and_then(|p| p.featured_image_url.as_ref())
                    .filter(|u| !u.is_empty())
                    .cloned()
                    .unwrap_or_else(|| "/static/placeholder.svg".to_string());
                let alt = ctx
                    .post
                    .map(|p| p.title.rendered.clone())
                    .unwrap_or_default();
                match &mut node.kind {
                    NodeKind::Image { src, alt: a, .. } => {
                        *src = img_url;
                        *a = alt;
                    }
                    _ => {
                        node.kind = NodeKind::Image {
                            src: img_url,
                            alt,
                            width: None,
                            height: None,
                            link: None,
                        };
                    }
                }
            }
            "post_content" | "post.content" => {
                if let Some(post) = ctx.post {
                    node.kind = NodeKind::CustomHtml {
                        raw: post.content.rendered.clone(),
                    };
                }
            }
            "post_excerpt" | "post.excerpt" => {
                let excerpt = ctx
                    .post
                    .map(|p| {
                        if !p.excerpt.rendered.trim().is_empty() {
                            p.excerpt.rendered.clone()
                        } else {
                            plain_excerpt_chars(&p.content.rendered, 160)
                        }
                    })
                    .unwrap_or_default();
                match &mut node.kind {
                    NodeKind::Text { content_html } => *content_html = excerpt,
                    NodeKind::Heading { text, .. } => *text = excerpt,
                    NodeKind::CustomHtml { raw } => *raw = excerpt,
                    _ => {
                        node.kind = NodeKind::Text {
                            content_html: excerpt,
                        }
                    }
                }
            }
            "post_date" | "post.date" => {
                let date_str = ctx
                    .post
                    .map(|p| p.date.chars().take(10).collect::<String>())
                    .unwrap_or_default();
                match &mut node.kind {
                    NodeKind::Text { content_html } => *content_html = date_str.clone(),
                    NodeKind::Heading { text, .. } => *text = date_str.clone(),
                    _ => {
                        node.kind = NodeKind::Text {
                            content_html: date_str,
                        }
                    }
                }
            }
            "post_url" | "post.url" => {
                let url_str = ctx
                    .post
                    .map(|p| {
                        if !p.link.is_empty() {
                            p.link.clone()
                        } else {
                            format!("/{}/", p.slug.trim_matches('/'))
                        }
                    })
                    .unwrap_or_else(|| "/".to_string());
                match &mut node.kind {
                    NodeKind::Button { url, .. } => *url = url_str,
                    NodeKind::Image { link, .. } => *link = Some(url_str),
                    NodeKind::Text { content_html } => *content_html = url_str,
                    _ => {}
                }
            }
            "post_author" | "author_name" | "author.name" => {
                let author_name = ctx
                    .post
                    .map(|p| {
                        if p.author_name.is_empty() {
                            "admin".to_string()
                        } else {
                            p.author_name.clone()
                        }
                    })
                    .unwrap_or_else(|| "admin".to_string());
                match &mut node.kind {
                    NodeKind::Text { content_html } => *content_html = author_name,
                    NodeKind::Heading { text, .. } => *text = author_name,
                    NodeKind::CustomHtml { raw } => *raw = author_name,
                    _ => {
                        node.kind = NodeKind::Text {
                            content_html: author_name,
                        }
                    }
                }
            }
            "author_box" => {
                let author_name = ctx
                    .post
                    .map(|p| {
                        if p.author_name.is_empty() {
                            "admin"
                        } else {
                            &p.author_name
                        }
                    })
                    .unwrap_or("admin");
                let escaped_author = html_escape(author_name);
                let card = format!(
                    r#"<div class="dp-author-box"><div class="dp-author-avatar"><i class="fa-solid fa-user"></i></div><div class="dp-author-info"><strong class="dp-author-name">{}</strong><span class="dp-author-role">Author</span></div></div>"#,
                    escaped_author
                );
                node.kind = NodeKind::CustomHtml { raw: card };
            }
            "archive_title" | "archive.title" => {
                let title = ctx
                    .archive_title
                    .clone()
                    .unwrap_or_else(|| ctx.settings.title.clone());
                match &mut node.kind {
                    NodeKind::Heading { text, .. } => *text = title,
                    NodeKind::Text { content_html } => *content_html = title,
                    _ => {
                        node.kind = NodeKind::Heading {
                            text: title,
                            level: 1,
                        }
                    }
                }
            }
            "archive_description" | "archive.description" => {
                let desc = ctx
                    .archive_subtitle
                    .clone()
                    .unwrap_or_else(|| ctx.settings.description.clone());
                match &mut node.kind {
                    NodeKind::Text { content_html } => *content_html = desc,
                    NodeKind::Heading { text, .. } => *text = desc,
                    _ => node.kind = NodeKind::Text { content_html: desc },
                }
            }
            "site_logo" | "site.logo" => {
                let logo_url = if !ctx.settings.site_logo_url.is_empty() {
                    ctx.settings.site_logo_url.clone()
                } else {
                    "/static/favicon.svg".to_string()
                };
                match &mut node.kind {
                    NodeKind::Image { src, .. } => *src = logo_url,
                    _ => {
                        node.kind = NodeKind::Image {
                            src: logo_url,
                            alt: ctx.settings.title.clone(),
                            width: None,
                            height: None,
                            link: Some("/".to_string()),
                        };
                    }
                }
            }
            "site_title" | "site.title" => {
                let title = ctx.settings.title.clone();
                match &mut node.kind {
                    NodeKind::Heading { text, .. } => *text = title,
                    NodeKind::Text { content_html } => *content_html = title,
                    _ => {
                        node.kind = NodeKind::Heading {
                            text: title,
                            level: 2,
                        }
                    }
                }
            }
            "site_description" | "site.description" => {
                let desc = ctx.settings.description.clone();
                match &mut node.kind {
                    NodeKind::Text { content_html } => *content_html = desc,
                    NodeKind::Heading { text, .. } => *text = desc,
                    _ => node.kind = NodeKind::Text { content_html: desc },
                }
            }
            other => {
                let field_key = other
                    .strip_prefix("acf.")
                    .or_else(|| other.strip_prefix("acf:"))
                    .or_else(|| other.strip_prefix("acf_"))
                    .or_else(|| other.strip_prefix("meta."))
                    .or_else(|| other.strip_prefix("meta:"));
                if let Some(key) = field_key {
                    let resolved = resolve_post_field_token(ctx.post, key);
                    match &mut node.kind {
                        NodeKind::Heading { text, .. } => *text = resolved,
                        NodeKind::Text { content_html } => *content_html = resolved,
                        NodeKind::CustomHtml { raw } => *raw = resolved,
                        NodeKind::Image { src, .. } => {
                            if !resolved.is_empty() {
                                *src = resolved;
                            }
                        }
                        NodeKind::Button { url, text, .. } => {
                            if resolved.starts_with("http://")
                                || resolved.starts_with("https://")
                                || resolved.starts_with('/')
                            {
                                *url = resolved;
                            } else if !resolved.is_empty() {
                                *text = resolved;
                            }
                        }
                        _ => {
                            node.kind = NodeKind::Text {
                                content_html: resolved,
                            };
                        }
                    }
                }
            }
        }
    }

    fn interpolate_text_tokens(node: &mut DaypressNode, ctx: &HydrationContext) {
        match &mut node.kind {
            NodeKind::Heading { text, .. } => {
                *text = Self::interpolate_tokens(text, ctx);
            }
            NodeKind::Text { content_html } => {
                *content_html = Self::interpolate_tokens(content_html, ctx);
            }
            NodeKind::CustomHtml { raw } => {
                *raw = Self::interpolate_tokens(raw, ctx);
            }
            NodeKind::Image { src, alt, link, .. } => {
                *src = Self::interpolate_tokens(src, ctx);
                *alt = Self::interpolate_tokens(alt, ctx);
                if let Some(ref mut l) = link {
                    *l = Self::interpolate_tokens(l, ctx);
                }
            }
            NodeKind::Button { text, url, .. } => {
                *text = Self::interpolate_tokens(text, ctx);
                *url = Self::interpolate_tokens(url, ctx);
            }
            _ => {}
        }
    }

    pub fn interpolate_tokens(text: &str, ctx: &HydrationContext) -> String {
        if !text.contains("{{") {
            return text.to_string();
        }
        let post_title = ctx.post.map(|p| p.title.rendered.as_str()).unwrap_or("");
        let post_date: String = ctx
            .post
            .map(|p| p.date.chars().take(10).collect())
            .unwrap_or_default();
        let post_excerpt: String = ctx
            .post
            .map(|p| {
                if !p.excerpt.rendered.trim().is_empty() {
                    p.excerpt.rendered.clone()
                } else {
                    plain_excerpt_chars(&p.content.rendered, 160)
                }
            })
            .unwrap_or_default();
        let post_content = ctx.post.map(|p| p.content.rendered.as_str()).unwrap_or("");
        let featured_image = ctx
            .post
            .and_then(|p| p.featured_image_url.as_deref())
            .filter(|u| !u.is_empty())
            .unwrap_or("/static/placeholder.svg");
        let post_url = ctx
            .post
            .map(|p| {
                if !p.link.is_empty() {
                    p.link.clone()
                } else {
                    format!("/{}/", p.slug.trim_matches('/'))
                }
            })
            .unwrap_or_else(|| "/".to_string());
        let post_author = ctx
            .post
            .map(|p| {
                if p.author_name.is_empty() {
                    "admin"
                } else {
                    p.author_name.as_str()
                }
            })
            .unwrap_or("admin");
        let archive_title = ctx
            .archive_title
            .as_deref()
            .or_else(|| ctx.post.map(|p| p.title.rendered.as_str()))
            .unwrap_or(&ctx.settings.title);
        let archive_description = ctx
            .archive_subtitle
            .as_deref()
            .unwrap_or(&ctx.settings.description);

        let mut out = text
            .replace("{{post.title}}", post_title)
            .replace("{{post_title}}", post_title)
            .replace("{{post.date}}", &post_date)
            .replace("{{post_date}}", &post_date)
            .replace("{{post.excerpt}}", &post_excerpt)
            .replace("{{post_excerpt}}", &post_excerpt)
            .replace("{{post.content}}", post_content)
            .replace("{{post_content}}", post_content)
            .replace("{{post.featured_image}}", featured_image)
            .replace("{{featured_image}}", featured_image)
            .replace("{{post.url}}", &post_url)
            .replace("{{post_url}}", &post_url)
            .replace("{{author.name}}", post_author)
            .replace("{{post_author}}", post_author)
            .replace("{{archive.title}}", archive_title)
            .replace("{{archive_title}}", archive_title)
            .replace("{{archive.description}}", archive_description)
            .replace("{{archive_description}}", archive_description)
            .replace("{{site.title}}", &ctx.settings.title)
            .replace("{{site_title}}", &ctx.settings.title)
            .replace("{{site.description}}", &ctx.settings.description)
            .replace("{{site_description}}", &ctx.settings.description)
            .replace("{{current_year}}", "2026");

        if out.contains("reading_time") || out.contains("word_count") {
            let (words, mins) =
                crate::adapters::database::search_fts::estimate_reading_time_minutes(post_content);
            let rt_str = format!("{} min read", mins);
            let wc_str = words.to_string();
            out = out
                .replace("{{post.reading_time}}", &rt_str)
                .replace("{{reading_time}}", &rt_str)
                .replace("{{post.word_count}}", &wc_str)
                .replace("{{word_count}}", &wc_str);
        }
        if out.contains("{{post.toc}}") || out.contains("{{toc}}") {
            let (_, toc_html, _) =
                crate::templates::public::single::inject_heading_anchors_and_build_toc(
                    post_content,
                    "Table of Contents",
                    2,
                    4,
                    true,
                );
            out = out
                .replace("{{post.toc}}", &toc_html)
                .replace("{{toc}}", &toc_html);
        }

        if out.contains("{{acf.") || out.contains("{{meta.") || out.contains("{{acf_") {
            out = interpolate_acf_and_meta_tokens(&out, ctx.post);
        }
        out
    }

    pub fn render_breadcrumbs_html(
        items: &[BreadcrumbItem],
        separator: &str,
        home_text: &str,
    ) -> String {
        let mut list_html = String::new();
        let mut all_items = Vec::new();
        all_items.push(BreadcrumbItem {
            title: if home_text.is_empty() {
                "Home".to_string()
            } else {
                home_text.to_string()
            },
            url: "/".to_string(),
            is_current: items.is_empty(),
        });
        all_items.extend_from_slice(items);

        let sep_escaped = html_escape(if separator.is_empty() { "/" } else { separator });

        for (idx, item) in all_items.iter().enumerate() {
            if idx > 0 {
                list_html.push_str(&format!(
                    r#"<li class="dp-breadcrumb-separator" aria-hidden="true">{}</li>"#,
                    sep_escaped
                ));
            }
            let title_escaped = html_escape(&item.title);
            let url_escaped = html_escape(&item.url);
            if item.is_current {
                list_html.push_str(&format!(
                    r#"<li class="dp-breadcrumb-item active" aria-current="page"><span>{}</span></li>"#,
                    title_escaped
                ));
            } else {
                list_html.push_str(&format!(
                    r#"<li class="dp-breadcrumb-item"><a href="{}">{}</a></li>"#,
                    url_escaped, title_escaped
                ));
            }
        }

        format!(
            r#"<nav class="dp-breadcrumbs" aria-label="Breadcrumb"><ol class="dp-breadcrumb-list">{}</ol></nav>"#,
            list_html
        )
    }

    fn create_hydration_context<'a>(
        req_ctx: &RequestContext,
        settings: &'a DaypressSettingsResponse,
        post: Option<&'a DaypressPostResponse>,
    ) -> HydrationContext<'a> {
        let mut nav_menus = HashMap::new();
        if !settings.nav_menu_items.is_empty() {
            let primary_items: Vec<NavMenuItem> = settings
                .nav_menu_items
                .iter()
                .enumerate()
                .map(|(idx, it)| NavMenuItem {
                    id: it.id,
                    title: it.title.clone(),
                    url: it.url.clone(),
                    target: it.target.clone(),
                    parent: it.parent,
                    order: idx as i64,
                    icon: it.icon.clone(),
                    badge: it.badge.clone(),
                    description: it.description.clone(),
                    mega_menu_template_id: it.mega_menu_template_id,
                    ..Default::default()
                })
                .collect();
            nav_menus.insert("primary".to_string(), primary_items.clone());
            nav_menus.insert("main-menu".to_string(), primary_items);
        }

        let breadcrumbs = req_ctx.build_breadcrumbs();

        HydrationContext {
            db: None,
            post,
            settings,
            breadcrumbs,
            nav_menus,
            archive_title: None,
            archive_subtitle: None,
            archive_posts: Vec::new(),
            pagination_html: None,
            symbol_depth: 0,
            collection_tax: None,
            collection_term: None,
            collection_sort: None,
            collection_search: None,
            collection_page: None,
        }
    }
}

fn format_structured_field_scalar(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|item| {
                if let Some(s) = item.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = item.as_object() {
                    let lbl = obj.get("label").and_then(|v| v.as_str()).unwrap_or("");
                    let v = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    if !lbl.is_empty() && !v.is_empty() {
                        Some(format!("{}: {}", lbl, v))
                    } else if !v.is_empty() {
                        Some(v.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(obj) => obj
            .get("title")
            .or_else(|| obj.get("url"))
            .or_else(|| obj.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        serde_json::Value::Null => String::new(),
    }
}

fn resolve_post_field_token(post: Option<&DaypressPostResponse>, key: &str) -> String {
    let Some(p) = post else {
        return String::new();
    };
    let clean_key = key.trim();
    if let Some(acf_obj) = p.acf.as_object() {
        if let Some(val) = acf_obj.get(clean_key) {
            return format_structured_field_scalar(val);
        }
    }
    if let Some(meta_obj) = p.meta.as_object() {
        if let Some(val) = meta_obj.get(clean_key) {
            return format_structured_field_scalar(val);
        }
    }
    String::new()
}

fn interpolate_acf_and_meta_tokens(text: &str, post: Option<&DaypressPostResponse>) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let token_inner = after_open[..end].trim();
            let field_key = token_inner
                .strip_prefix("acf.")
                .or_else(|| token_inner.strip_prefix("meta."))
                .or_else(|| token_inner.strip_prefix("acf_"));
            if let Some(key) = field_key {
                result.push_str(&resolve_post_field_token(post, key));
            } else {
                result.push_str("{{");
                result.push_str(&after_open[..end]);
                result.push_str("}}");
            }
            rest = &after_open[end + 2..];
        } else {
            result.push_str(&rest[start..]);
            rest = "";
        }
    }
    result.push_str(rest);
    result
}

fn plain_excerpt_chars(html: &str, max_chars: usize) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let collapsed: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let count = collapsed.chars().count();
    if count <= max_chars {
        collapsed
    } else {
        let truncated: String = collapsed.chars().take(max_chars).collect();
        format!("{}...", truncated.trim_end())
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DaypressRenderedText;
    use daypress_ast::StyleProperties;

    fn sample_post() -> DaypressPostResponse {
        DaypressPostResponse {
            id: 42,
            slug: "rust-edge-cms".to_string(),
            post_type: "post".to_string(),
            date: "2026-05-12T10:00:00Z".to_string(),
            author_name: "Ada Lovelace".to_string(),
            featured_image_url: Some("https://daypress.dev/uploads/hero.webp".to_string()),
            title: DaypressRenderedText {
                rendered: "Rust Edge CMS Architecture".to_string(),
                protected: false,
            },
            excerpt: DaypressRenderedText {
                rendered: "Zero-overhead SSR with SQLite.".to_string(),
                protected: false,
            },
            content: DaypressRenderedText {
                rendered: "<p>Full article body content.</p>".to_string(),
                protected: false,
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_dot_and_underscore_token_interpolation() {
        let post = sample_post();
        let settings = DaypressSettingsResponse {
            title: "Daypress Docs".to_string(),
            description: "Next-gen CMS".to_string(),
            ..Default::default()
        };

        let req_ctx = RequestContext::default();
        let mut ctx = SsrInjector::create_hydration_context(&req_ctx, &settings, Some(&post));
        ctx.archive_title = Some("Engineering Blog".to_string());
        ctx.archive_subtitle = Some("Deep dives into Rust".to_string());

        let template_str = "{{post.title}} | {{author.name}} | {{post.date}} | {{post.featured_image}} | {{site.title}} | {{archive.title}}";
        let hydrated = SsrInjector::interpolate_tokens(template_str, &ctx);
        assert_eq!(
            hydrated,
            "Rust Edge CMS Architecture | Ada Lovelace | 2026-05-12 | https://daypress.dev/uploads/hero.webp | Daypress Docs | Engineering Blog"
        );

        let mut img_node = DaypressNode {
            id: "hero-img".to_string(),
            kind: NodeKind::Image {
                src: "{{post.featured_image}}".to_string(),
                alt: "{{post.title}}".to_string(),
                width: None,
                height: None,
                link: Some("{{post.url}}".to_string()),
            },
            styles: StyleProperties::default(),
            children: vec![],
        };
        SsrInjector::hydrate_node(&mut img_node, &ctx);
        if let NodeKind::Image { src, alt, link, .. } = img_node.kind {
            assert_eq!(src, "https://daypress.dev/uploads/hero.webp");
            assert_eq!(alt, "Rust Edge CMS Architecture");
            assert_eq!(link.as_deref(), Some("/rust-edge-cms/"));
        } else {
            panic!("Expected Image node");
        }
    }

    #[test]
    fn test_symbol_instance_overrides() {
        let settings = DaypressSettingsResponse::default();
        let req_ctx = RequestContext::default();
        let ctx = SsrInjector::create_hydration_context(&req_ctx, &settings, None);

        let mut overrides = HashMap::new();
        overrides.insert(
            "cta-heading".to_string(),
            SymbolOverride {
                text: Some("Custom Enterprise Heading".to_string()),
                src: None,
                alt: None,
                url: None,
            },
        );
        overrides.insert(
            "cta-btn".to_string(),
            SymbolOverride {
                text: Some("Book Demo".to_string()),
                src: None,
                alt: None,
                url: Some("/enterprise-demo".to_string()),
            },
        );

        let mut symbol_node = DaypressNode {
            id: "sym-instance-1".to_string(),
            kind: NodeKind::SymbolInstance {
                symbol_id: 99,
                overrides,
            },
            styles: StyleProperties::default(),
            children: vec![
                DaypressNode {
                    id: "cta-heading".to_string(),
                    kind: NodeKind::Heading {
                        text: "Default Heading".to_string(),
                        level: 2,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
                DaypressNode {
                    id: "cta-btn".to_string(),
                    kind: NodeKind::Button {
                        text: "Default CTA".to_string(),
                        url: "/signup".to_string(),
                        target: "_self".to_string(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                },
            ],
        };

        SsrInjector::hydrate_node(&mut symbol_node, &ctx);
        match &symbol_node.children[0].kind {
            NodeKind::Heading { text, .. } => assert_eq!(text, "Custom Enterprise Heading"),
            _ => panic!("Expected Heading child"),
        }
        match &symbol_node.children[1].kind {
            NodeKind::Button { text, url, .. } => {
                assert_eq!(text, "Book Demo");
                assert_eq!(url, "/enterprise-demo");
            }
            _ => panic!("Expected Button child"),
        }
    }

    #[test]
    fn test_loop_grid_hydration() {
        let post = sample_post();
        let settings = DaypressSettingsResponse::default();
        let req_ctx = RequestContext::default();
        let mut ctx = SsrInjector::create_hydration_context(&req_ctx, &settings, None);
        ctx.archive_posts = vec![post];
        ctx.pagination_html = Some("<nav class=\"dp-pagination\">Page 1</nav>".to_string());

        let mut loop_node = DaypressNode {
            id: "archive-grid".to_string(),
            kind: NodeKind::LoopGrid {
                post_type: "post".to_string(),
                posts_per_page: 6,
                columns: 3,
                show_thumbnail: true,
                show_excerpt: true,
                show_meta: true,
                show_pagination: true,
                taxonomy: None,
                enable_filter_bar: false,
                enable_sort: false,
                enable_search: false,
                default_sort: "date_desc".to_string(),
                card_skin: "editorial".to_string(),
                meta_fields: Vec::new(),
            },
            styles: StyleProperties::default(),
            children: vec![],
        };

        SsrInjector::hydrate_node(&mut loop_node, &ctx);
        match loop_node.kind {
            NodeKind::CustomHtml { raw } => {
                assert!(raw.contains("Rust Edge CMS Architecture"));
                assert!(raw.contains("Ada Lovelace"));
                assert!(raw.contains("https://daypress.dev/uploads/hero.webp"));
                assert!(raw.contains("Zero-overhead SSR with SQLite."));
                assert!(raw.contains("dp-pagination"));
            }
            _ => panic!("Expected LoopGrid to hydrate into CustomHtml"),
        }
    }
}
