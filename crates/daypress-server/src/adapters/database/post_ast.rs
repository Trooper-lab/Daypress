//! Post AST synchronization helper (`adapters/database/post_ast.rs`).

use rusqlite::{params, Connection};
use std::collections::HashMap;

/// Transpiles post content into a canonical AST layout and updates `posts.layout_json`
/// and `postmeta._daypress_layout`. Uses `daypress_ast::transpile_content` for Gutenberg,
/// Elementor, and Divi, falling back to `Database::synthesize_layout_json_from_html`.
pub fn synchronize_post_ast_layout(
    conn: &Connection,
    post_id: i64,
    content: &str,
    meta_updates: Option<&serde_json::Value>,
) -> rusqlite::Result<()> {
    // 1. Gather existing postmeta
    let mut meta: HashMap<String, String> = HashMap::new();
    {
        let mut stmt =
            conn.prepare("SELECT meta_key, meta_value FROM postmeta WHERE post_id = ?1")?;
        let rows = stmt.query_map(params![post_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows.flatten() {
            meta.insert(row.0, row.1);
        }
    }

    // 2. Overlay new metadata from request if provided
    if let Some(serde_json::Value::Object(map)) = meta_updates {
        for (k, v) in map {
            let val_str = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            meta.insert(k.clone(), val_str);
        }
    }

    // 3. Attempt universal transpilation (Elementor, Divi, or Gutenberg)
    let layout_json = if let Some(layout) = daypress_ast::transpile_content(content, &meta) {
        serde_json::to_string(&layout).ok()
    } else {
        super::Database::synthesize_layout_json_from_html(post_id, content)
    };

    if let Some(l_json) = layout_json {
        conn.execute(
            "UPDATE posts SET layout_json = ?1 WHERE ID = ?2",
            params![l_json, post_id],
        )?;
        conn.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
            params![post_id],
        )?;
        conn.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
            params![post_id, l_json],
        )?;
    }

    Ok(())
}
