//! Public Navigation Menu Tree & SSR Renderers (`templates/public/menus.rs`).

use crate::models::NavMenuItemDto;
use crate::templates::utils::escape_html;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NavMenuNode {
    pub item: NavMenuItemDto,
    pub children: Vec<NavMenuNode>,
}

/// Builds a hierarchical menu tree from a flat list of `NavMenuItemDto` items.
pub fn build_nav_menu_tree(items: &[NavMenuItemDto]) -> Vec<NavMenuNode> {
    if items.is_empty() {
        return Vec::new();
    }

    let mut root_items: Vec<NavMenuItemDto> = Vec::new();
    let mut children_map: HashMap<i64, Vec<NavMenuItemDto>> = HashMap::new();

    for it in items {
        if it.parent == 0 {
            root_items.push(it.clone());
        } else {
            children_map.entry(it.parent).or_default().push(it.clone());
        }
    }

    fn attach_children(
        parent: NavMenuItemDto,
        children_map: &HashMap<i64, Vec<NavMenuItemDto>>,
        depth: usize,
    ) -> NavMenuNode {
        if depth >= 20 {
            return NavMenuNode {
                item: parent,
                children: Vec::new(),
            };
        }
        let children = if let Some(sub) = children_map.get(&parent.id) {
            sub.iter()
                .map(|child| attach_children(child.clone(), children_map, depth + 1))
                .collect()
        } else {
            Vec::new()
        };
        NavMenuNode {
            item: parent,
            children,
        }
    }

    root_items
        .into_iter()
        .map(|root| attach_children(root, &children_map, 0))
        .collect()
}

/// Recursively renders WordPress standard hierarchical menu markup:
/// `<ul class="menu"><li class="menu-item menu-item-has-children"><a ...>Parent</a><ul class="sub-menu">...</ul></li></ul>`
pub fn render_hierarchical_nav_menu(nodes: &[NavMenuNode], is_sub_menu: bool) -> String {
    if nodes.is_empty() {
        return String::new();
    }

    let ul_class = if is_sub_menu {
        "sub-menu"
    } else {
        "menu np-primary-menu"
    };
    let mut out = format!(r#"<ul class="{}">"#, ul_class);

    for node in nodes {
        let has_children = !node.children.is_empty();
        let mut li_classes = vec!["menu-item", "np-nav-item"];
        if has_children {
            li_classes.push("menu-item-has-children");
        }
        let target_attr = if !node.item.target.is_empty() {
            format!(r#" target="{}""#, escape_html(&node.item.target))
        } else {
            String::new()
        };

        out.push_str(&format!(
            r#"<li class="{}"><a href="{}" class="nav-link"{}>{}</a>"#,
            li_classes.join(" "),
            escape_html(&node.item.url),
            target_attr,
            escape_html(&node.item.title)
        ));

        if has_children {
            out.push_str(&render_hierarchical_nav_menu(&node.children, true));
        }

        out.push_str("</li>");
    }

    out.push_str("</ul>");
    out
}

/// Renders flat footer links from menu items.
pub fn render_footer_nav_links(items: &[NavMenuItemDto]) -> String {
    let mut f_html = String::new();
    for item in items {
        let target_attr = if !item.target.is_empty() {
            format!(r#" target="{}""#, escape_html(&item.target))
        } else {
            String::new()
        };
        f_html.push_str(&format!(
            r#"<a href="{}"{}>{}</a>"#,
            escape_html(&item.url),
            target_attr,
            escape_html(&item.title)
        ));
    }
    f_html
}
