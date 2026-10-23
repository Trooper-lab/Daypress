//! SQLite Schema & Migration Initializer for Neopress Theme Builder.

use rusqlite::{Connection, Result};

pub const TEMPLATES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS wp_neopress_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    template_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'publish',
    layout_json TEXT NOT NULL DEFAULT '{}',
    rendered_html TEXT DEFAULT NULL,
    settings_json TEXT NOT NULL DEFAULT '{}',
    priority INTEGER NOT NULL DEFAULT 10,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_neopress_templates_type_active 
    ON wp_neopress_templates(template_type, is_active, priority DESC);

CREATE INDEX IF NOT EXISTS idx_neopress_templates_status 
    ON wp_neopress_templates(status);
"#;

pub const CONDITIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS wp_neopress_template_conditions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER NOT NULL REFERENCES wp_neopress_templates(id) ON DELETE CASCADE,
    condition_type TEXT NOT NULL DEFAULT 'include',
    rule_target TEXT NOT NULL,
    target_id TEXT NOT NULL DEFAULT '',
    priority INTEGER NOT NULL DEFAULT 10,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_neopress_template_conditions_tpl 
    ON wp_neopress_template_conditions(template_id);

CREATE INDEX IF NOT EXISTS idx_neopress_template_conditions_rule 
    ON wp_neopress_template_conditions(rule_target, target_id);

CREATE INDEX IF NOT EXISTS idx_neopress_template_conditions_type_priority 
    ON wp_neopress_template_conditions(condition_type, priority DESC);
"#;

/// Initializes the Theme Builder SQLite schema, tables, and composite indices.
pub fn init_theme_builder_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(TEMPLATES_TABLE_SQL)?;
    conn.execute_batch(CONDITIONS_TABLE_SQL)?;
    Ok(())
}
