//! SQLite Transactional Repository for Theme Templates and Conditions.

use super::models::*;
use crate::adapters::database::Database;
use chrono::Utc;
use neopress_ast::{render_layout_to_html, sanitize_ssr_html, NeopressLayout};
use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};

/// SQLite Repository managing Theme Templates and Display Conditions.
#[derive(Clone)]
pub struct ThemeTemplateRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ThemeTemplateRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn from_database(db: &Database) -> Self {
        Self {
            conn: db.conn.clone(),
        }
    }

    pub fn conn(&self) -> &Arc<Mutex<Connection>> {
        &self.conn
    }

    pub fn create(&self, req: &CreateTemplateRequest) -> Result<ThemeTemplateRecord> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let layout = req.layout.clone().unwrap_or_else(|| {
            req.layout_json
                .as_ref()
                .and_then(|val| serde_json::from_value(val.clone()).ok())
                .unwrap_or_default()
        });
        let layout_str = serde_json::to_string(&layout).unwrap_or_else(|_| "{}".to_string());
        let settings = req.settings.clone().unwrap_or_default();
        let settings_str = serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());
        let status = req.status.as_deref().unwrap_or("publish");
        let priority = req.priority.unwrap_or(10);
        let is_active = req.is_active.unwrap_or(true);

        let raw_html = render_layout_to_html(&layout);
        let sanitized_html = sanitize_ssr_html(&raw_html);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        tx.execute(
            r#"INSERT INTO wp_neopress_templates 
               (title, template_type, status, layout_json, rendered_html, settings_json, priority, is_active, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)"#,
            params![
                req.title,
                req.template_type.as_str(),
                status,
                layout_str,
                Some(sanitized_html.clone()),
                settings_str,
                priority,
                if is_active { 1 } else { 0 },
                now,
            ],
        )?;

        let template_id = tx.last_insert_rowid();
        let mut saved_conditions = Vec::new();

        if let Some(conds) = &req.conditions {
            for c in conds {
                tx.execute(
                    r#"INSERT INTO wp_neopress_template_conditions 
                       (template_id, condition_type, rule_target, target_id, priority, created_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                    params![
                        template_id,
                        c.condition_type.as_str(),
                        c.rule_target.as_str(),
                        c.target_id,
                        c.priority,
                        now,
                    ],
                )?;
                saved_conditions.push(TemplateCondition {
                    id: Some(tx.last_insert_rowid()),
                    template_id,
                    condition_type: c.condition_type,
                    rule_target: c.rule_target,
                    target_id: c.target_id.clone(),
                    priority: c.priority,
                    created_at: Some(now.clone()),
                });
            }
        }

        tx.commit()?;

        Ok(ThemeTemplateRecord {
            id: template_id,
            title: req.title.clone(),
            template_type: req.template_type,
            status: status.to_string(),
            layout,
            rendered_html: Some(sanitized_html),
            settings,
            priority,
            is_active,
            conditions: saved_conditions,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<ThemeTemplateRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, title, template_type, status, layout_json, rendered_html, settings_json, priority, is_active, created_at, updated_at
               FROM wp_neopress_templates WHERE id = ?1"#,
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let tpl_id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let status: String = row.get(3)?;
            let layout_json: String = row.get(4)?;
            let rendered_html: Option<String> = row.get(5)?;
            let settings_json: String = row.get(6)?;
            let priority: i32 = row.get(7)?;
            let is_active_int: i32 = row.get(8)?;
            let created_at: String = row.get(9)?;
            let updated_at: String = row.get(10)?;

            let template_type =
                TemplateType::from_str_loose(&type_str).unwrap_or(TemplateType::Header);
            let layout: NeopressLayout = serde_json::from_str(&layout_json).unwrap_or_default();
            let settings: TemplateSettings =
                serde_json::from_str(&settings_json).unwrap_or_default();

            let conditions = self.load_conditions_with_conn(&conn, tpl_id)?;

            Ok(Some(ThemeTemplateRecord {
                id: tpl_id,
                title,
                template_type,
                status,
                layout,
                rendered_html,
                settings,
                priority,
                is_active: is_active_int == 1,
                conditions,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list(&self, filter: &TemplateFilter) -> Result<Vec<ThemeTemplateRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from(
            "SELECT id, title, template_type, status, layout_json, rendered_html, settings_json, priority, is_active, created_at, updated_at FROM wp_neopress_templates WHERE 1=1"
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(t) = filter.template_type {
            query.push_str(" AND template_type = ?");
            params_vec.push(Box::new(t.as_str().to_string()));
        }
        if let Some(ref st) = filter.status {
            query.push_str(" AND status = ?");
            params_vec.push(Box::new(st.clone()));
        } else {
            query.push_str(" AND status != 'trash'");
        }
        if let Some(act) = filter.is_active {
            query.push_str(" AND is_active = ?");
            params_vec.push(Box::new(if act { 1 } else { 0 }));
        }

        query.push_str(" ORDER BY priority DESC, id DESC");
        let mut stmt = conn.prepare(&query)?;
        let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(&rusqlite_params[..], |row| {
            let tpl_id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let status: String = row.get(3)?;
            let layout_json: String = row.get(4)?;
            let rendered_html: Option<String> = row.get(5)?;
            let settings_json: String = row.get(6)?;
            let priority: i32 = row.get(7)?;
            let is_active_int: i32 = row.get(8)?;
            let created_at: String = row.get(9)?;
            let updated_at: String = row.get(10)?;

            let template_type =
                TemplateType::from_str_loose(&type_str).unwrap_or(TemplateType::Header);
            let layout: NeopressLayout = serde_json::from_str(&layout_json).unwrap_or_default();
            let settings: TemplateSettings =
                serde_json::from_str(&settings_json).unwrap_or_default();

            Ok(ThemeTemplateRecord {
                id: tpl_id,
                title,
                template_type,
                status,
                layout,
                rendered_html,
                settings,
                priority,
                is_active: is_active_int == 1,
                conditions: Vec::new(),
                created_at,
                updated_at,
            })
        })?;

        let mut records = Vec::new();
        for r in rows {
            let mut record = r?;
            record.conditions = self.load_conditions_with_conn(&conn, record.id)?;
            records.push(record);
        }
        Ok(records)
    }

    pub fn list_items(&self, filter: &TemplateFilter) -> Result<Vec<TemplateListItem>> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from(
            r#"SELECT t.id, t.title, t.template_type, t.status, t.priority, t.is_active, 
                      COUNT(c.id) as cond_count, t.updated_at
               FROM wp_neopress_templates t
               LEFT JOIN wp_neopress_template_conditions c ON t.id = c.template_id
               WHERE 1=1"#,
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(t) = filter.template_type {
            query.push_str(" AND t.template_type = ?");
            params_vec.push(Box::new(t.as_str().to_string()));
        }
        if let Some(ref st) = filter.status {
            query.push_str(" AND t.status = ?");
            params_vec.push(Box::new(st.clone()));
        } else {
            query.push_str(" AND t.status != 'trash'");
        }
        if let Some(act) = filter.is_active {
            query.push_str(" AND t.is_active = ?");
            params_vec.push(Box::new(if act { 1 } else { 0 }));
        }

        query.push_str(" GROUP BY t.id ORDER BY t.priority DESC, t.id DESC");
        let mut stmt = conn.prepare(&query)?;
        let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(&rusqlite_params[..], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let type_str: String = row.get(2)?;
            let status: String = row.get(3)?;
            let priority: i32 = row.get(4)?;
            let is_active_int: i32 = row.get(5)?;
            let cond_count: i64 = row.get(6)?;
            let updated_at: String = row.get(7)?;

            let template_type =
                TemplateType::from_str_loose(&type_str).unwrap_or(TemplateType::Header);
            Ok(TemplateListItem {
                id,
                title,
                template_type,
                status,
                priority,
                is_active: is_active_int == 1,
                condition_count: cond_count as usize,
                updated_at,
            })
        })?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }
        Ok(items)
    }

    pub fn update(
        &self,
        id: i64,
        req: &UpdateTemplateRequest,
    ) -> Result<Option<ThemeTemplateRecord>> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let existing = tx.query_row(
            "SELECT title, template_type, status, layout_json, settings_json, priority, is_active FROM wp_neopress_templates WHERE id = ?1",
            params![id],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i32>(5)?,
                r.get::<_, i32>(6)? == 1,
            )),
        );

        let (
            cur_title,
            cur_type,
            cur_status,
            cur_layout_json,
            cur_settings_json,
            cur_priority,
            cur_active,
        ) = match existing {
            Ok(vals) => vals,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e),
        };

        let new_title = req.title.as_ref().unwrap_or(&cur_title);
        let new_type = req
            .template_type
            .map(|t| t.as_str().to_string())
            .unwrap_or(cur_type);
        let new_status = req.status.as_ref().unwrap_or(&cur_status);
        let new_priority = req.priority.unwrap_or(cur_priority);
        let new_active = req.is_active.unwrap_or(cur_active);

        let new_layout: NeopressLayout = if let Some(l) = &req.layout {
            l.clone()
        } else if let Some(val) = &req.layout_json {
            serde_json::from_value(val.clone()).unwrap_or_default()
        } else {
            serde_json::from_str(&cur_layout_json).unwrap_or_default()
        };
        let new_layout_str =
            serde_json::to_string(&new_layout).unwrap_or_else(|_| "{}".to_string());

        let new_settings: TemplateSettings = if let Some(s) = &req.settings {
            s.clone()
        } else {
            serde_json::from_str(&cur_settings_json).unwrap_or_default()
        };
        let new_settings_str =
            serde_json::to_string(&new_settings).unwrap_or_else(|_| "{}".to_string());

        let raw_html = render_layout_to_html(&new_layout);
        let new_rendered_html = req
            .rendered_html
            .clone()
            .unwrap_or_else(|| sanitize_ssr_html(&raw_html));

        tx.execute(
            r#"UPDATE wp_neopress_templates 
               SET title = ?1, template_type = ?2, status = ?3, layout_json = ?4, rendered_html = ?5, 
                   settings_json = ?6, priority = ?7, is_active = ?8, updated_at = ?9
               WHERE id = ?10"#,
            params![
                new_title,
                new_type,
                new_status,
                new_layout_str,
                Some(new_rendered_html.clone()),
                new_settings_str,
                new_priority,
                if new_active { 1 } else { 0 },
                now,
                id,
            ],
        )?;

        if let Some(conds) = &req.conditions {
            tx.execute(
                "DELETE FROM wp_neopress_template_conditions WHERE template_id = ?1",
                params![id],
            )?;
            for c in conds {
                tx.execute(
                    r#"INSERT INTO wp_neopress_template_conditions 
                       (template_id, condition_type, rule_target, target_id, priority, created_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                    params![
                        id,
                        c.condition_type.as_str(),
                        c.rule_target.as_str(),
                        c.target_id,
                        c.priority,
                        now,
                    ],
                )?;
            }
        }

        tx.commit()?;
        drop(conn);

        self.get_by_id(id)
    }

    pub fn delete(&self, id: i64, force: bool) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        if force {
            let rows = conn.execute(
                "DELETE FROM wp_neopress_templates WHERE id = ?1",
                params![id],
            )?;
            Ok(rows > 0)
        } else {
            let rows = conn.execute(
                "UPDATE wp_neopress_templates SET status = 'trash', updated_at = datetime('now') WHERE id = ?1",
                params![id],
            )?;
            Ok(rows > 0)
        }
    }
}
