//! Supplementary transactional repository operations (Conditions, Duplication, Active Filtering).

use super::models::*;
use super::repository::ThemeTemplateRepository;
use chrono::Utc;
use rusqlite::{params, Connection, Result};

impl ThemeTemplateRepository {
    pub fn duplicate(&self, id: i64) -> Result<Option<ThemeTemplateRecord>> {
        let orig = match self.get_by_id(id)? {
            Some(o) => o,
            None => return Ok(None),
        };

        let copy_req = CreateTemplateRequest {
            title: format!("{} (Copy)", orig.title),
            template_type: orig.template_type,
            status: Some("draft".to_string()),
            layout: Some(orig.layout),
            layout_json: None,
            settings: Some(orig.settings),
            priority: Some(orig.priority),
            is_active: Some(false),
            conditions: Some(
                orig.conditions
                    .into_iter()
                    .map(|c| CreateConditionDto {
                        id: None,
                        condition_type: c.condition_type,
                        rule_target: c.rule_target,
                        target_id: c.target_id,
                        priority: c.priority,
                    })
                    .collect(),
            ),
        };

        self.create(&copy_req).map(Some)
    }

    pub fn get_conditions(&self, template_id: i64) -> Result<Vec<TemplateCondition>> {
        let conn = self.conn().lock().unwrap();
        self.load_conditions_with_conn(&conn, template_id)
    }

    pub fn set_conditions(
        &self,
        template_id: i64,
        conds: &[CreateConditionDto],
    ) -> Result<Vec<TemplateCondition>> {
        let mut conn = self.conn().lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        tx.execute(
            "DELETE FROM daypress_template_conditions WHERE template_id = ?1",
            params![template_id],
        )?;

        let mut saved = Vec::new();
        for c in conds {
            tx.execute(
                r#"INSERT INTO daypress_template_conditions 
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
            saved.push(TemplateCondition {
                id: Some(tx.last_insert_rowid()),
                template_id,
                condition_type: c.condition_type,
                rule_target: c.rule_target,
                target_id: c.target_id.clone(),
                priority: c.priority,
                created_at: Some(now.clone()),
            });
        }

        tx.commit()?;
        Ok(saved)
    }

    pub fn find_active_by_type(
        &self,
        template_type: TemplateType,
    ) -> Result<Vec<ThemeTemplateRecord>> {
        self.list(&TemplateFilter {
            template_type: Some(template_type),
            status: Some("publish".to_string()),
            is_active: Some(true),
        })
    }

    pub(crate) fn load_conditions_with_conn(
        &self,
        conn: &Connection,
        template_id: i64,
    ) -> Result<Vec<TemplateCondition>> {
        let mut cond_stmt = conn.prepare(
            r#"SELECT id, template_id, condition_type, rule_target, target_id, priority, created_at
               FROM daypress_template_conditions WHERE template_id = ?1 ORDER BY priority DESC, id ASC"#,
        )?;
        let cond_rows = cond_stmt.query_map(params![template_id], |cr| {
            let c_type_str: String = cr.get(2)?;
            let r_target_str: String = cr.get(3)?;
            Ok(TemplateCondition {
                id: Some(cr.get(0)?),
                template_id: cr.get(1)?,
                condition_type: ConditionType::from_str_loose(&c_type_str)
                    .unwrap_or(ConditionType::Include),
                rule_target: RuleTarget::from_str_loose(&r_target_str)
                    .unwrap_or(RuleTarget::EntireSite),
                target_id: cr.get(4)?,
                priority: cr.get(5)?,
                created_at: Some(cr.get(6)?),
            })
        })?;

        let mut conditions = Vec::new();
        for c in cond_rows {
            conditions.push(c?);
        }
        Ok(conditions)
    }
}
