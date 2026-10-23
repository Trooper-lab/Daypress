//! Navigation menu parsing, 2-pass hierarchical tree construction, and options persistence.

use crate::importer::urls::remap_menu_url;
use crate::models::NavMenuItemDto;
use crate::templates::utils::escape_html;
use rusqlite::{params, Result, Transaction};
use std::collections::{HashMap, HashSet};

fn build_menu_tree_recursive(
    parent_id: i64,
    children_by_parent: &mut HashMap<i64, Vec<neopress_ast::NavMenuItem>>,
    visited: &mut HashSet<i64>,
    depth: usize,
) -> Vec<neopress_ast::NavMenuItem> {
    if depth > 20 {
        return Vec::new();
    }
    if let Some(mut items) = children_by_parent.remove(&parent_id) {
        items.sort_by_key(|i| i.order);
        for item in &mut items {
            if visited.insert(item.id) {
                item.children =
                    build_menu_tree_recursive(item.id, children_by_parent, visited, depth + 1);
            }
        }
        items
    } else {
        Vec::new()
    }
}

pub fn ingest_nav_menus(
    tx: &Transaction,
    raw_nav_menu_items: Vec<(i64, String, i64, Option<String>, HashMap<String, String>)>,
    old_to_new_post_id: &HashMap<i64, i64>,
    post_urls: &HashMap<String, String>,
    source_site_url: Option<&str>,
) -> Result<HashMap<String, Vec<neopress_ast::NavMenuItem>>> {
    let mut flat_items_by_slug: HashMap<String, Vec<neopress_ast::NavMenuItem>> = HashMap::new();
    let mut nav_dtos: Vec<NavMenuItemDto> = Vec::with_capacity(raw_nav_menu_items.len());

    if !raw_nav_menu_items.is_empty() {
        let has_primary = raw_nav_menu_items.iter().any(|(_, _, _, s, _)| {
            s.as_deref() == Some("primary")
                || s.as_deref() == Some("main")
                || s.as_deref() == Some("hoofdmenu")
        });

        // Pass 1: Resolve metadata, remap parents, update wp_posts and wp_postmeta
        for (new_item_id, item_title, menu_order, menu_slug_opt, meta) in raw_nav_menu_items {
            let mut resolved_title = item_title;
            let mut resolved_url = meta
                .get("_menu_item_url")
                .map(|u| remap_menu_url(u, source_site_url))
                .unwrap_or_default();

            if meta.get("_menu_item_type").map(|s| s.as_str()) == Some("post_type_archive") {
                if let Some(obj_type) = meta.get("_menu_item_object") {
                    if resolved_url.is_empty() {
                        resolved_url = format!("/{}/", obj_type.trim());
                    }
                    if resolved_title.is_empty() {
                        resolved_title =
                            crate::templates::admin::layout::format_cpt_title(obj_type.trim());
                    }
                }
            } else if let Some(obj_id_str) = meta.get("_menu_item_object_id") {
                if let Ok(old_obj_id) = obj_id_str.trim().parse::<i64>() {
                    let target_post_id = old_to_new_post_id
                        .get(&old_obj_id)
                        .copied()
                        .unwrap_or(old_obj_id);
                    if let Ok((target_slug, target_title)) = tx.query_row(
                        "SELECT post_name, post_title FROM wp_posts WHERE ID = ?1",
                        params![target_post_id],
                        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                    ) {
                        if resolved_title.is_empty() {
                            resolved_title = target_title;
                        }
                        if resolved_url.is_empty() && !target_slug.is_empty() {
                            if let Some(permalink) = post_urls.get(&target_post_id.to_string()) {
                                resolved_url = permalink.clone();
                            } else {
                                resolved_url = format!("/{}/", target_slug);
                            }
                        }
                    }
                }
            }

            if resolved_url.is_empty() {
                resolved_url = "/".to_string();
            }
            if resolved_title.is_empty() {
                resolved_title = "Menu Item".to_string();
            }

            let old_menu_parent: i64 = meta
                .get("_menu_item_menu_item_parent")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            let mapped_menu_parent = if old_menu_parent > 0 {
                old_to_new_post_id
                    .get(&old_menu_parent)
                    .copied()
                    .unwrap_or(0)
            } else {
                0
            };

            // Update wp_posts.post_parent and postmeta for the menu item
            let _ = tx.execute(
                "UPDATE wp_posts SET post_parent = ?1 WHERE ID = ?2",
                params![mapped_menu_parent, new_item_id],
            );
            let _ = tx.execute(
                "UPDATE wp_postmeta SET meta_value = ?1 WHERE post_id = ?2 AND meta_key = '_menu_item_menu_item_parent'",
                params![mapped_menu_parent.to_string(), new_item_id],
            );

            let target = meta.get("_menu_item_target").cloned().unwrap_or_default();
            let slug_key = menu_slug_opt.unwrap_or_else(|| "primary".to_string());

            let nav_item = neopress_ast::NavMenuItem {
                id: new_item_id,
                title: resolved_title.clone(),
                url: resolved_url.clone(),
                target: target.clone(),
                parent: mapped_menu_parent,
                order: menu_order,
                children: Vec::new(),
            };

            flat_items_by_slug.entry(slug_key.clone()).or_default().push(nav_item);

            if !has_primary
                || slug_key == "primary"
                || slug_key == "main"
                || slug_key == "hoofdmenu"
            {
                nav_dtos.push(NavMenuItemDto {
                    id: new_item_id,
                    title: resolved_title,
                    url: resolved_url,
                    target,
                    parent: mapped_menu_parent,
                    menu_order,
                });
            }
        }
    }

    // Pass 2: Assemble hierarchical N-level tree with cycle and depth guard
    let mut menus_by_slug: HashMap<String, Vec<neopress_ast::NavMenuItem>> = HashMap::new();

    for (slug, flat_items) in flat_items_by_slug {
        let mut children_by_parent: HashMap<i64, Vec<neopress_ast::NavMenuItem>> = HashMap::new();
        for item in flat_items {
            children_by_parent.entry(item.parent).or_default().push(item);
        }

        let mut visited = HashSet::new();
        let mut tree = build_menu_tree_recursive(0, &mut children_by_parent, &mut visited, 0);

        // Hoist any unvisited orphan items to root level
        for (_orphan_parent, orphan_items) in children_by_parent {
            for orphan in orphan_items {
                if visited.insert(orphan.id) {
                    tree.push(orphan);
                }
            }
        }

        tree.sort_by_key(|i| i.order);
        menus_by_slug.insert(slug, tree);
    }

    if !menus_by_slug.is_empty() {
        if let Ok(menus_json) = serde_json::to_string(&menus_by_slug) {
            let _ = tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_nav_menus_by_slug', ?1) \
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![menus_json],
            );
        }

        nav_dtos.sort_by_key(|i| i.menu_order);
        if let Ok(json_str) = serde_json::to_string(&nav_dtos) {
            let _ = tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_nav_menu_items', ?1) \
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![json_str],
            );
        }
    }

    Ok(menus_by_slug)
}

/// Recursively renders hierarchical navigation menu items with standard WordPress classes.
/// Emits `<li class="menu-item menu-item-has-children">...<ul class="sub-menu">...</ul></li>`
/// for nodes with children. Guarded by depth <= 20 to prevent recursion overflow.
pub fn render_universal_nav_items_recursive(
    items: &[neopress_ast::NavMenuItem],
    depth: usize,
) -> String {
    if depth > 20 || items.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for item in items {
        let has_children = !item.children.is_empty();
        let li_class = if has_children {
            "menu-item menu-item-has-children"
        } else {
            "menu-item"
        };
        let safe_url = escape_html(&item.url);
        let safe_title = escape_html(&item.title);
        let target_attr = if !item.target.is_empty() {
            format!(r#" target="{}""#, escape_html(&item.target))
        } else {
            String::new()
        };
        out.push_str(&format!(
            r#"<li class="{}"><a href="{}"{}>{}</a>"#,
            li_class, safe_url, target_attr, safe_title
        ));
        if has_children {
            out.push_str(r#"<ul class="sub-menu">"#);
            out.push_str(&render_universal_nav_items_recursive(&item.children, depth + 1));
            out.push_str("</ul>");
        }
        out.push_str("</li>");
    }
    out
}

/// Builds a semantic, theme-agnostic header HTML from available primary navigation menus.
pub fn build_universal_theme_header_html(
    menus_by_slug: &HashMap<String, Vec<neopress_ast::NavMenuItem>>,
    site_title: &str,
    logo_url: Option<&str>,
) -> String {
    let mut links = String::new();
    let menu_items = menus_by_slug
        .get("primary")
        .or_else(|| menus_by_slug.get("main"))
        .or_else(|| menus_by_slug.get("hoofdmenu"))
        .or_else(|| menus_by_slug.values().next());

    if let Some(items) = menu_items {
        links.push_str(&render_universal_nav_items_recursive(items, 0));
    }
    if links.is_empty() {
        links.push_str(r#"<li class="menu-item"><a href="/">Home</a></li>"#);
    }

    let logo_html = if let Some(url) = logo_url.filter(|u| !u.is_empty()) {
        format!(r#"<img src="{}" alt="{}" class="logo" />"#, escape_html(url), escape_html(site_title))
    } else {
        format!(r#"<span class="site-title">{}</span>"#, escape_html(site_title))
    };

    format!(
        r#"<header class="theme-header">
    <div class="container">
        <div class="header-inner" style="display: flex; justify-content: space-between; align-items: center; padding: 18px 0;">
            <a href="/" class="site-brand" style="text-decoration: none; color: inherit; font-weight: 700;">
                {}
            </a>
            <nav class="theme-nav">
                <ul class="nav-menu" style="display: flex; gap: 24px; list-style: none; margin: 0; padding: 0;">
                    {}
                </ul>
            </nav>
        </div>
    </div>
</header>"#,
        logo_html, links
    )
}

/// Builds a semantic, theme-agnostic footer HTML from available footer navigation menus.
pub fn build_universal_theme_footer_html(
    menus_by_slug: &HashMap<String, Vec<neopress_ast::NavMenuItem>>,
    site_title: &str,
    logo_url: Option<&str>,
) -> String {
    let mut links = String::new();
    let footer_items = menus_by_slug
        .get("footer")
        .or_else(|| menus_by_slug.get("footermenu"))
        .or_else(|| menus_by_slug.get("secondary"));

    if let Some(items) = footer_items {
        links.push_str(&render_universal_nav_items_recursive(items, 0));
    }

    let logo_html = if let Some(url) = logo_url.filter(|u| !u.is_empty()) {
        format!(r#"<img src="{}" alt="{}" class="footer-logo" style="max-height: 32px;" />"#, escape_html(url), escape_html(site_title))
    } else {
        format!(r#"<span class="footer-title">{}</span>"#, escape_html(site_title))
    };

    format!(
        r#"<footer class="theme-footer" style="padding: 48px 0 32px; background: #0f172a; color: #f8fafc;">
    <div class="container">
        <div class="footer-inner" style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 24px;">
            <div class="footer-brand">
                {}
                <p style="margin: 8px 0 0; font-size: 13px; color: #94a3b8;">&copy; {} {}. Powered by Neopress.</p>
            </div>
            <nav class="footer-nav">
                <ul class="footer-menu" style="display: flex; gap: 20px; list-style: none; margin: 0; padding: 0; font-size: 13px;">
                    {}
                </ul>
            </nav>
        </div>
    </div>
</footer>"#,
        logo_html,
        chrono::Utc::now().format("%Y"),
        escape_html(site_title),
        links
    )
}

