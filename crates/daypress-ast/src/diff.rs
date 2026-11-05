//! Structural AST Revision Diff Engine & Node Cherry-Pick Patcher (`crates/daypress-ast/src/diff.rs`).
//!
//! Compares two `DaypressLayout` trees node-by-node (`Added`, `Removed`, `Modified`, `Moved`)
//! with granular property-level change inspection (`content`, `styles`, `responsive`) and
//! supports cherry-picking individual nodes from a historical revision into an active layout.

use crate::render::{render_layout_to_html, render_node_to_html};
use crate::schema::{DaypressLayout, DaypressNode, NodeKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AstPropertyChange {
    pub category: String,
    pub field: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AstNodeDiff {
    pub node_id: String,
    pub node_type: String,
    pub label: String,
    pub path: String,
    /// `"added"`, `"removed"`, `"modified"`, or `"moved"`
    pub kind: String,
    pub changes: Vec<AstPropertyChange>,
    pub before_html: String,
    pub after_html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AstLayoutDiffReport {
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
    pub moved_count: usize,
    pub unchanged_count: usize,
    pub nodes: Vec<AstNodeDiff>,
    pub base_rendered_html: String,
    pub target_rendered_html: String,
}

#[derive(Clone)]
struct IndexedNode<'a> {
    node: &'a DaypressNode,
    parent_id: String,
    sibling_index: usize,
    path: String,
}

fn node_type_name(kind: &NodeKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.get("type").and_then(|t| t.as_str()).map(String::from))
        .unwrap_or_else(|| "Section".to_string())
}

fn truncate_preview(raw: &str, max_chars: usize) -> String {
    let stripped: String = raw
        .replace('<', " <")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let clean: String = stripped.chars().take(max_chars).collect();
    if stripped.chars().count() > max_chars {
        format!("{}…", clean)
    } else {
        clean
    }
}

fn node_human_label(node: &DaypressNode) -> String {
    let t = node_type_name(&node.kind);
    let summary = match &node.kind {
        NodeKind::Heading { text, level } => format!("H{}: {}", level, truncate_preview(text, 42)),
        NodeKind::Text { content_html } => truncate_preview(content_html, 42),
        NodeKind::Button { text, url, .. } => format!("{} ({})", truncate_preview(text, 28), url),
        NodeKind::Image { alt, src, .. } => {
            if !alt.trim().is_empty() {
                truncate_preview(alt, 36)
            } else {
                truncate_preview(src, 36)
            }
        }
        NodeKind::CallToAction { title, .. }
        | NodeKind::PriceCard { title, .. }
        | NodeKind::PriceTable { title, .. }
        | NodeKind::IconBox { title, .. }
        | NodeKind::ImageBox { title, .. }
        | NodeKind::ProgressBar { title, .. }
        | NodeKind::Alert { title, .. } => truncate_preview(title, 42),
        NodeKind::LanguageSwitcher { display_style, .. } => format!("Language ({})", display_style),
        _ => String::new(),
    };
    if summary.trim().is_empty() {
        format!("{} ({})", t, node.id)
    } else {
        format!("{}: {}", t, summary)
    }
}

fn index_tree<'a>(
    node: &'a DaypressNode,
    parent_id: &str,
    sibling_index: usize,
    path_prefix: &str,
    counter: &mut usize,
    out: &mut HashMap<String, IndexedNode<'a>>,
    order_list: &mut Vec<String>,
) {
    let current_path = if path_prefix.is_empty() {
        node.id.clone()
    } else {
        format!("{} > {}", path_prefix, node.id)
    };

    if node.id != "root" && !matches!(node.kind, NodeKind::Root) {
        *counter += 1;
        order_list.push(node.id.clone());
        out.insert(
            node.id.clone(),
            IndexedNode {
                node,
                parent_id: parent_id.to_string(),
                sibling_index,
                path: current_path.clone(),
            },
        );
    }

    for (idx, child) in node.children.iter().enumerate() {
        index_tree(
            child,
            &node.id,
            idx,
            &current_path,
            counter,
            out,
            order_list,
        );
    }
}

fn value_to_compact_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn diff_json_maps(
    category: &str,
    base_map: &serde_json::Map<String, Value>,
    target_map: &serde_json::Map<String, Value>,
    ignore_keys: &[&str],
    changes: &mut Vec<AstPropertyChange>,
) {
    let mut keys: Vec<&String> = base_map.keys().chain(target_map.keys()).collect();
    keys.sort();
    keys.dedup();

    for key in keys {
        if ignore_keys.contains(&key.as_str()) {
            continue;
        }
        let b_val = base_map.get(key).unwrap_or(&Value::Null);
        let t_val = target_map.get(key).unwrap_or(&Value::Null);
        if b_val != t_val {
            let b_str = value_to_compact_string(b_val);
            let t_str = value_to_compact_string(t_val);
            if b_str != t_str {
                changes.push(AstPropertyChange {
                    category: category.to_string(),
                    field: key.clone(),
                    before: b_str,
                    after: t_str,
                });
            }
        }
    }
}

fn compute_node_property_changes(
    base_node: &DaypressNode,
    target_node: &DaypressNode,
) -> Vec<AstPropertyChange> {
    let mut changes = Vec::new();

    let b_kind_type = node_type_name(&base_node.kind);
    let t_kind_type = node_type_name(&target_node.kind);
    if b_kind_type != t_kind_type {
        changes.push(AstPropertyChange {
            category: "structure".to_string(),
            field: "type".to_string(),
            before: b_kind_type,
            after: t_kind_type,
        });
    }

    // Compare kind data fields
    let b_kind_val = serde_json::to_value(&base_node.kind).unwrap_or(Value::Null);
    let t_kind_val = serde_json::to_value(&target_node.kind).unwrap_or(Value::Null);
    let empty_map = serde_json::Map::new();
    let b_data = b_kind_val
        .get("data")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);
    let t_data = t_kind_val
        .get("data")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);
    diff_json_maps("content", b_data, t_data, &[], &mut changes);

    // Compare flat styles (ignoring duplicated nested mirrors so diffs stay clean and readable)
    let b_styles_val = serde_json::to_value(&base_node.styles).unwrap_or(Value::Null);
    let t_styles_val = serde_json::to_value(&target_node.styles).unwrap_or(Value::Null);
    let b_styles = b_styles_val.as_object().unwrap_or(&empty_map);
    let t_styles = t_styles_val.as_object().unwrap_or(&empty_map);
    diff_json_maps(
        "styles",
        b_styles,
        t_styles,
        &[
            "spacing",
            "typography",
            "background",
            "border",
            "tablet",
            "mobile",
            "hover",
            "raw_settings",
        ],
        &mut changes,
    );

    // Compare responsive overrides (tablet, mobile, hover)
    for sub in ["tablet", "mobile", "hover"] {
        let b_sub = b_styles
            .get(sub)
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);
        let t_sub = t_styles
            .get(sub)
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);
        if b_sub != t_sub {
            diff_json_maps(
                &format!("responsive:{}", sub),
                b_sub,
                t_sub,
                &["spacing", "typography", "background", "border"],
                &mut changes,
            );
        }
    }

    changes
}

/// Computes a structural node-by-node and property-by-property diff between `base` (older revision)
/// and `target` (newer revision or current live layout).
pub fn diff_layouts(base: &DaypressLayout, target: &DaypressLayout) -> AstLayoutDiffReport {
    let mut base_map = HashMap::new();
    let mut base_order = Vec::new();
    let mut base_counter = 0usize;
    index_tree(
        &base.root,
        "",
        0,
        "",
        &mut base_counter,
        &mut base_map,
        &mut base_order,
    );

    let mut target_map = HashMap::new();
    let mut target_order = Vec::new();
    let mut target_counter = 0usize;
    index_tree(
        &target.root,
        "",
        0,
        "",
        &mut target_counter,
        &mut target_map,
        &mut target_order,
    );

    let mut report = AstLayoutDiffReport {
        base_rendered_html: render_layout_to_html(base),
        target_rendered_html: render_layout_to_html(target),
        ..Default::default()
    };

    // 1. Check all nodes in target (Added, Modified, Moved, or Unchanged)
    for node_id in &target_order {
        let Some(t_entry) = target_map.get(node_id) else {
            continue;
        };
        if let Some(b_entry) = base_map.get(node_id) {
            let mut prop_changes = compute_node_property_changes(b_entry.node, t_entry.node);
            let moved = b_entry.parent_id != t_entry.parent_id
                || b_entry.sibling_index != t_entry.sibling_index;

            if !prop_changes.is_empty() {
                if moved {
                    prop_changes.push(AstPropertyChange {
                        category: "structure".to_string(),
                        field: "position".to_string(),
                        before: format!("{}[{}]", b_entry.parent_id, b_entry.sibling_index),
                        after: format!("{}[{}]", t_entry.parent_id, t_entry.sibling_index),
                    });
                }
                report.modified_count += 1;
                report.nodes.push(AstNodeDiff {
                    node_id: node_id.clone(),
                    node_type: node_type_name(&t_entry.node.kind),
                    label: node_human_label(t_entry.node),
                    path: t_entry.path.clone(),
                    kind: "modified".to_string(),
                    changes: prop_changes,
                    before_html: render_node_to_html(b_entry.node),
                    after_html: render_node_to_html(t_entry.node),
                });
            } else if moved {
                report.moved_count += 1;
                report.nodes.push(AstNodeDiff {
                    node_id: node_id.clone(),
                    node_type: node_type_name(&t_entry.node.kind),
                    label: node_human_label(t_entry.node),
                    path: t_entry.path.clone(),
                    kind: "moved".to_string(),
                    changes: vec![AstPropertyChange {
                        category: "structure".to_string(),
                        field: "position".to_string(),
                        before: format!("{}[{}]", b_entry.parent_id, b_entry.sibling_index),
                        after: format!("{}[{}]", t_entry.parent_id, t_entry.sibling_index),
                    }],
                    before_html: render_node_to_html(b_entry.node),
                    after_html: render_node_to_html(t_entry.node),
                });
            } else {
                report.unchanged_count += 1;
            }
        } else {
            report.added_count += 1;
            report.nodes.push(AstNodeDiff {
                node_id: node_id.clone(),
                node_type: node_type_name(&t_entry.node.kind),
                label: node_human_label(t_entry.node),
                path: t_entry.path.clone(),
                kind: "added".to_string(),
                changes: vec![AstPropertyChange {
                    category: "structure".to_string(),
                    field: "node".to_string(),
                    before: String::new(),
                    after: format!("Added {}", node_human_label(t_entry.node)),
                }],
                before_html: String::new(),
                after_html: render_node_to_html(t_entry.node),
            });
        }
    }

    // 2. Check nodes in base that were removed in target
    for node_id in &base_order {
        if !target_map.contains_key(node_id) {
            if let Some(b_entry) = base_map.get(node_id) {
                report.removed_count += 1;
                report.nodes.push(AstNodeDiff {
                    node_id: node_id.clone(),
                    node_type: node_type_name(&b_entry.node.kind),
                    label: node_human_label(b_entry.node),
                    path: b_entry.path.clone(),
                    kind: "removed".to_string(),
                    changes: vec![AstPropertyChange {
                        category: "structure".to_string(),
                        field: "node".to_string(),
                        before: format!("Removed {}", node_human_label(b_entry.node)),
                        after: String::new(),
                    }],
                    before_html: render_node_to_html(b_entry.node),
                    after_html: String::new(),
                });
            }
        }
    }

    report
}

fn find_node_and_parent<'a>(
    root: &'a DaypressNode,
    target_id: &str,
    parent_id: &str,
    sibling_idx: usize,
) -> Option<(&'a DaypressNode, String, usize)> {
    if root.id == target_id {
        return Some((root, parent_id.to_string(), sibling_idx));
    }
    for (idx, child) in root.children.iter().enumerate() {
        if let Some(found) = find_node_and_parent(child, target_id, &root.id, idx) {
            return Some(found);
        }
    }
    None
}

fn replace_node_in_tree(
    root: &mut DaypressNode,
    target_id: &str,
    replacement: &DaypressNode,
) -> bool {
    if root.id == target_id {
        *root = replacement.clone();
        return true;
    }
    for child in &mut root.children {
        if replace_node_in_tree(child, target_id, replacement) {
            return true;
        }
    }
    false
}

fn insert_under_parent(
    root: &mut DaypressNode,
    parent_id: &str,
    sibling_idx: usize,
    node_to_insert: &DaypressNode,
) -> bool {
    if root.id == parent_id {
        let pos = sibling_idx.min(root.children.len());
        root.children.insert(pos, node_to_insert.clone());
        return true;
    }
    for child in &mut root.children {
        if insert_under_parent(child, parent_id, sibling_idx, node_to_insert) {
            return true;
        }
    }
    false
}

fn remove_node_from_tree(root: &mut DaypressNode, target_id: &str) -> bool {
    if let Some(pos) = root.children.iter().position(|c| c.id == target_id) {
        root.children.remove(pos);
        return true;
    }
    for child in &mut root.children {
        if remove_node_from_tree(child, target_id) {
            return true;
        }
    }
    false
}

/// Restores a single node (`node_id`) from `revision` into `current` without overwriting
/// the rest of `current`.
///
/// - If `node_id` exists in both `revision` and `current`, replaces it in `current`.
/// - If `node_id` exists in `revision` but was deleted in `current`, re-inserts it under its parent
///   (or appends to `current.root` if its parent no longer exists).
/// - If `node_id` does not exist in `revision` (it was added after `revision`), removes it from `current`.
pub fn cherry_pick_node_from_revision(
    current: &DaypressLayout,
    revision: &DaypressLayout,
    node_id: &str,
) -> Result<DaypressLayout, String> {
    let mut patched = current.clone();
    if let Some((rev_node, parent_id, sibling_idx)) =
        find_node_and_parent(&revision.root, node_id, "", 0)
    {
        if replace_node_in_tree(&mut patched.root, node_id, rev_node) {
            return Ok(patched);
        }
        if !parent_id.is_empty()
            && insert_under_parent(&mut patched.root, &parent_id, sibling_idx, rev_node)
        {
            return Ok(patched);
        }
        let pos = sibling_idx.min(patched.root.children.len());
        patched.root.children.insert(pos, rev_node.clone());
        Ok(patched)
    } else if remove_node_from_tree(&mut patched.root, node_id) {
        Ok(patched)
    } else {
        Err(format!(
            "Node '{}' was not found in either the revision or current layout",
            node_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::StyleProperties;

    fn sample_layout(heading_text: &str, include_cta: bool) -> DaypressLayout {
        let mut children = vec![DaypressNode {
            id: "hero_h1".to_string(),
            kind: NodeKind::Heading {
                text: heading_text.to_string(),
                level: 1,
            },
            styles: StyleProperties {
                text_color: Some("#111827".to_string()),
                ..Default::default()
            },
            children: vec![],
        }];
        if include_cta {
            children.push(DaypressNode {
                id: "hero_btn".to_string(),
                kind: NodeKind::Button {
                    text: "Get Started".to_string(),
                    url: "/signup".to_string(),
                    target: "_self".to_string(),
                },
                styles: StyleProperties::default(),
                children: vec![],
            });
        }
        DaypressLayout {
            version: 1,
            global_styles: None,
            root: DaypressNode {
                id: "root".to_string(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![DaypressNode {
                    id: "sec_hero".to_string(),
                    kind: NodeKind::Section,
                    styles: StyleProperties::default(),
                    children,
                }],
            },
        }
    }

    #[test]
    fn test_diff_layouts_and_cherry_pick() {
        let rev_old = sample_layout("Welcome v1", true);
        let mut rev_new = sample_layout("Welcome v2", false);
        rev_new.root.children[0].children[0].styles.text_color = Some("#dc2626".to_string());

        let diff = diff_layouts(&rev_old, &rev_new);
        assert_eq!(diff.modified_count, 1);
        assert_eq!(diff.removed_count, 1);
        assert_eq!(diff.added_count, 0);

        // Cherry-pick the modified heading from rev_old into rev_new
        let restored_h1 =
            cherry_pick_node_from_revision(&rev_new, &rev_old, "hero_h1").expect("cherry-pick h1");
        assert_eq!(
            restored_h1.root.children[0].children[0].kind,
            NodeKind::Heading {
                text: "Welcome v1".to_string(),
                level: 1
            }
        );

        // Cherry-pick the deleted button from rev_old back into restored_h1
        let restored_all = cherry_pick_node_from_revision(&restored_h1, &rev_old, "hero_btn")
            .expect("cherry-pick deleted button");
        assert_eq!(restored_all.root.children[0].children.len(), 2);
    }
}
