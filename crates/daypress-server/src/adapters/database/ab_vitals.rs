//! v0.9 Pillar 3: Zero-Flicker Server-Side A/B Split Testing & Real-User Core Web Vitals (`adapters/database/ab_vitals.rs`).

use super::Database;
use crate::errors::AppError;
use crate::models::{
    AbTestRecord, CoreWebVitalsSummary, CreateAbTestRequest, RouteWebVitalsMetric,
    WebVitalsBeaconRequest,
};
use chrono::Utc;
use rusqlite::{params, Result};
use sha2::{Digest, Sha256};

/// Deterministically assigns a visitor to Variant `"A"` (Control) or `"B"` (Variant)
/// based on `SHA-256("{test_id}:{visitor_hash}")` modulo 100 and `traffic_split` (`1..99`).
pub fn assign_ab_variant(test_id: i64, visitor_hash: &str, traffic_split: i64) -> &'static str {
    let split = traffic_split.clamp(1, 99) as u64;
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", test_id, visitor_hash).as_bytes());
    let digest = hasher.finalize();
    let bucket = u64::from_be_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ]) % 100;
    if bucket < split {
        "B"
    } else {
        "A"
    }
}

/// Standard normal cumulative distribution function $\Phi(x)$ via Abramowitz & Stegun 7.1.26.
fn standard_normal_cdf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let z = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * z);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-z * z).exp();
    0.5 * (1.0 + sign * y)
}

/// Computes conversion rates, relative uplift (`%`), two-proportion `z_score`,
/// two-tailed statistical `confidence_pct`, and `recommended_winner` (`Some("A" | "B")`).
pub fn compute_ab_statistics(
    control_views: i64,
    control_conversions: i64,
    variant_views: i64,
    variant_conversions: i64,
) -> (f64, f64, f64, f64, f64, Option<String>) {
    let n_a = control_views.max(0) as f64;
    let c_a = control_conversions.clamp(0, control_views.max(0)) as f64;
    let n_b = variant_views.max(0) as f64;
    let c_b = variant_conversions.clamp(0, variant_views.max(0)) as f64;

    let cr_a = if n_a > 0.0 { c_a / n_a } else { 0.0 };
    let cr_b = if n_b > 0.0 { c_b / n_b } else { 0.0 };

    let cr_a_pct = (cr_a * 10000.0).round() / 100.0;
    let cr_b_pct = (cr_b * 10000.0).round() / 100.0;

    let uplift_pct = if cr_a > 0.0 {
        (((cr_b - cr_a) / cr_a) * 1000.0).round() / 10.0
    } else if cr_b > 0.0 {
        100.0
    } else {
        0.0
    };

    if n_a < 2.0 || n_b < 2.0 || (c_a + c_b) == 0.0 {
        return (cr_a_pct, cr_b_pct, uplift_pct, 0.0, 0.0, None);
    }

    let p_pool = (c_a + c_b) / (n_a + n_b);
    let se_sq = p_pool * (1.0 - p_pool) * (1.0 / n_a + 1.0 / n_b);
    if se_sq <= 1e-12 {
        return (cr_a_pct, cr_b_pct, uplift_pct, 0.0, 0.0, None);
    }
    let z = (cr_b - cr_a) / se_sq.sqrt();
    let z_rounded = (z * 100.0).round() / 100.0;
    let p_two_tailed = 2.0 * (1.0 - standard_normal_cdf(z.abs()));
    let confidence = ((1.0 - p_two_tailed).clamp(0.0, 0.999) * 1000.0).round() / 10.0;

    let recommended = if confidence >= 90.0 && (n_a + n_b) >= 10.0 {
        if cr_b > cr_a {
            Some("B".to_string())
        } else if cr_a > cr_b {
            Some("A".to_string())
        } else {
            None
        }
    } else {
        None
    };

    (
        cr_a_pct,
        cr_b_pct,
        uplift_pct,
        z_rounded,
        confidence,
        recommended,
    )
}

fn percentile_p75(mut values: Vec<f64>) -> f64 {
    values.retain(|v| v.is_finite() && *v >= 0.0);
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((values.len() as f64 * 0.75).ceil() as usize)
        .saturating_sub(1)
        .min(values.len() - 1);
    (values[idx] * 100.0).round() / 100.0
}

impl Database {
    fn map_ab_test_row(r: &rusqlite::Row) -> rusqlite::Result<AbTestRecord> {
        let control_views: i64 = r.get(9)?;
        let control_conversions: i64 = r.get(10)?;
        let variant_views: i64 = r.get(11)?;
        let variant_conversions: i64 = r.get(12)?;
        let (cr_a, cr_b, uplift, z, conf, rec_winner) = compute_ab_statistics(
            control_views,
            control_conversions,
            variant_views,
            variant_conversions,
        );
        Ok(AbTestRecord {
            id: r.get(0)?,
            name: r.get(1)?,
            post_id: r.get(2)?,
            post_title: r.get(3)?,
            target_selector: r.get(4)?,
            control_text: r.get(5)?,
            variant_text: r.get(6)?,
            goal_event: r.get(7)?,
            traffic_split: r.get(8)?,
            control_views,
            control_conversions,
            variant_views,
            variant_conversions,
            control_cr_pct: cr_a,
            variant_cr_pct: cr_b,
            uplift_pct: uplift,
            z_score: z,
            confidence_pct: conf,
            recommended_winner: rec_winner,
            status: r.get(13)?,
            winner: r.get(14)?,
            created_at: r.get(15)?,
        })
    }

    pub fn save_ab_test(
        &self,
        req: &CreateAbTestRequest,
    ) -> std::result::Result<AbTestRecord, AppError> {
        let name = req.name.trim();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "A/B test name cannot be empty.".to_string(),
            ));
        }
        if req.post_id <= 0 {
            return Err(AppError::BadRequest(
                "A valid target post_id is required.".to_string(),
            ));
        }
        let control_text = req.control_text.trim();
        let variant_text = req.variant_text.trim();
        if control_text.is_empty() || variant_text.is_empty() {
            return Err(AppError::BadRequest(
                "Both control_text (Variant A) and variant_text (Variant B) are required."
                    .to_string(),
            ));
        }
        let split = req.traffic_split.clamp(1, 99);
        let goal = if req.goal_event.trim().is_empty() {
            "conversion"
        } else {
            req.goal_event.trim()
        };
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let conn = self.conn.lock().unwrap();
        let test_id = if let Some(id) = req.id.filter(|&i| i > 0) {
            conn.execute(
                r#"UPDATE daypress_ab_tests
                   SET name = ?1, post_id = ?2, target_selector = ?3, control_text = ?4, variant_text = ?5, goal_event = ?6, traffic_split = ?7
                   WHERE id = ?8"#,
                params![
                    name,
                    req.post_id,
                    req.target_selector.trim(),
                    control_text,
                    variant_text,
                    goal,
                    split,
                    id,
                ],
            )?;
            id
        } else {
            conn.execute(
                r#"INSERT INTO daypress_ab_tests
                   (name, post_id, target_selector, control_text, variant_text, goal_event, traffic_split, status, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'running', ?8)"#,
                params![
                    name,
                    req.post_id,
                    req.target_selector.trim(),
                    control_text,
                    variant_text,
                    goal,
                    split,
                    now,
                ],
            )?;
            conn.last_insert_rowid()
        };
        Self::activate_builtin_plugin_with_conn(&conn, "daypress-ab-testing");

        let rec = conn.query_row(
            r#"SELECT t.id, t.name, t.post_id, COALESCE(p.post_title, 'Post #' || t.post_id),
                      t.target_selector, t.control_text, t.variant_text, t.goal_event, t.traffic_split,
                      t.control_views, t.control_conversions, t.variant_views, t.variant_conversions,
                      t.status, t.winner, t.created_at
               FROM daypress_ab_tests t
               LEFT JOIN posts p ON p.ID = t.post_id
               WHERE t.id = ?1"#,
            params![test_id],
            Self::map_ab_test_row,
        )?;
        drop(conn);
        self.invalidate_hot_cache("ab_test_saved");
        Ok(rec)
    }

    pub fn list_ab_tests(&self, post_id: Option<i64>) -> Result<Vec<AbTestRecord>> {
        let conn = self.conn.lock().unwrap();
        let sql = if post_id.is_some() {
            r#"SELECT t.id, t.name, t.post_id, COALESCE(p.post_title, 'Post #' || t.post_id),
                      t.target_selector, t.control_text, t.variant_text, t.goal_event, t.traffic_split,
                      t.control_views, t.control_conversions, t.variant_views, t.variant_conversions,
                      t.status, t.winner, t.created_at
               FROM daypress_ab_tests t
               LEFT JOIN posts p ON p.ID = t.post_id
               WHERE t.post_id = ?1
               ORDER BY t.id DESC"#
        } else {
            r#"SELECT t.id, t.name, t.post_id, COALESCE(p.post_title, 'Post #' || t.post_id),
                      t.target_selector, t.control_text, t.variant_text, t.goal_event, t.traffic_split,
                      t.control_views, t.control_conversions, t.variant_views, t.variant_conversions,
                      t.status, t.winner, t.created_at
               FROM daypress_ab_tests t
               LEFT JOIN posts p ON p.ID = t.post_id
               ORDER BY t.id DESC"#
        };
        let mut stmt = conn.prepare(sql)?;
        let out: Vec<AbTestRecord> = if let Some(pid) = post_id {
            stmt.query_map(params![pid], Self::map_ab_test_row)?
                .flatten()
                .collect()
        } else {
            stmt.query_map([], Self::map_ab_test_row)?
                .flatten()
                .collect()
        };
        Ok(out)
    }

    pub fn delete_ab_test(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "DELETE FROM daypress_ab_exposures WHERE test_id = ?1",
            params![id],
        );
        let deleted = conn.execute("DELETE FROM daypress_ab_tests WHERE id = ?1", params![id])?;
        drop(conn);
        if deleted > 0 {
            self.invalidate_hot_cache("ab_test_deleted");
        }
        Ok(deleted > 0)
    }

    /// Returns `true` if `post_id` has at least one active (`status = 'running'`) A/B split test.
    pub fn has_running_ab_test_for_post(&self, post_id: i64) -> bool {
        let Ok(conn) = self.conn.lock() else {
            return false;
        };
        conn.query_row(
            "SELECT COUNT(*) FROM daypress_ab_tests WHERE post_id = ?1 AND status = 'running'",
            params![post_id],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
            > 0
    }

    /// Server-side Zero-Flicker A/B variant evaluator:
    /// 1. Finds all `status = 'running'` experiments for `post_id`.
    /// 2. Deterministically assigns `visitor_hash` to `"A"` or `"B"` (preserving sticky assignment in `daypress_ab_exposures`).
    /// 3. Increments `control_views` / `variant_views` on first exposure for `visitor_hash`.
    /// 4. Mutates `rendered_html` in-memory before SSR response is sent (zero client-side JS flicker).
    pub fn apply_ab_experiments_for_post(
        &self,
        post_id: i64,
        visitor_hash: &str,
        rendered_html: &str,
    ) -> (String, Vec<(i64, String)>) {
        let Ok(conn) = self.conn.lock() else {
            return (rendered_html.to_string(), Vec::new());
        };
        let Ok(mut stmt) = conn.prepare(
            r#"SELECT id, control_text, variant_text, traffic_split
               FROM daypress_ab_tests
               WHERE post_id = ?1 AND status = 'running'
               ORDER BY id ASC"#,
        ) else {
            return (rendered_html.to_string(), Vec::new());
        };
        let tests: Vec<(i64, String, String, i64)> = stmt
            .query_map(params![post_id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .ok()
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default();
        if tests.is_empty() {
            return (rendered_html.to_string(), Vec::new());
        }

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut out_html = rendered_html.to_string();
        let mut assignments = Vec::new();

        for (test_id, control_text, variant_text, split) in tests {
            let existing_variant: Option<String> = conn
                .query_row(
                    "SELECT variant FROM daypress_ab_exposures WHERE test_id = ?1 AND visitor_hash = ?2",
                    params![test_id, visitor_hash],
                    |r| r.get(0),
                )
                .ok();

            let variant = match existing_variant {
                Some(v) => v,
                None => {
                    let assigned = assign_ab_variant(test_id, visitor_hash, split).to_string();
                    let inserted = conn
                        .execute(
                            r#"INSERT OR IGNORE INTO daypress_ab_exposures
                               (test_id, visitor_hash, variant, converted, created_at)
                               VALUES (?1, ?2, ?3, 0, ?4)"#,
                            params![test_id, visitor_hash, assigned, now],
                        )
                        .unwrap_or(0);
                    if inserted > 0 {
                        if assigned == "B" {
                            let _ = conn.execute(
                                "UPDATE daypress_ab_tests SET variant_views = variant_views + 1 WHERE id = ?1",
                                params![test_id],
                            );
                        } else {
                            let _ = conn.execute(
                                "UPDATE daypress_ab_tests SET control_views = control_views + 1 WHERE id = ?1",
                                params![test_id],
                            );
                        }
                    }
                    assigned
                }
            };

            if variant == "B" && !control_text.is_empty() {
                out_html = out_html.replace(&control_text, &variant_text);
            }
            assignments.push((test_id, variant));
        }

        if !assignments.is_empty() {
            let attr_val = assignments
                .iter()
                .map(|(tid, var)| format!("{}:{}", tid, var))
                .collect::<Vec<_>>()
                .join(",");
            let badge_attr = format!(" data-dp-ab=\"{}\"", attr_val);
            if let Some(pos) = out_html.find("<body") {
                if let Some(close_rel) = out_html[pos..].find('>') {
                    let insert_at = pos + close_rel;
                    out_html.insert_str(insert_at, &badge_attr);
                }
            } else {
                out_html.push_str(&format!("<!--dp-ab:{}-->", attr_val));
            }
        }

        (out_html, assignments)
    }

    /// Attributes a visitor conversion (`form_submit`, `newsletter_signup`, `shortlink`, `conversion`)
    /// to any active A/B test where `visitor_hash` has an unconverted exposure.
    pub fn record_ab_conversion_for_visitor(
        &self,
        visitor_hash: &str,
        event_type: &str,
        goal_name: Option<&str>,
    ) {
        if visitor_hash.trim().is_empty() || event_type.eq_ignore_ascii_case("pageview") {
            return;
        }
        if !self.is_plugin_active("daypress-ab-testing") {
            return;
        }
        let Ok(conn) = self.conn.lock() else {
            return;
        };
        let Ok(mut stmt) = conn.prepare(
            r#"SELECT e.test_id, e.variant, t.goal_event
               FROM daypress_ab_exposures e
               INNER JOIN daypress_ab_tests t ON t.id = e.test_id
               WHERE e.visitor_hash = ?1 AND e.converted = 0 AND t.status = 'running'"#,
        ) else {
            return;
        };
        let exposures: Vec<(i64, String, String)> = stmt
            .query_map(params![visitor_hash], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .ok()
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default();

        for (test_id, variant, goal_event) in exposures {
            let goal_clean = goal_event.trim().to_ascii_lowercase();
            let ev_clean = event_type.trim().to_ascii_lowercase();
            let gname_clean = goal_name.unwrap_or("").trim().to_ascii_lowercase();
            let matches = goal_clean.is_empty()
                || goal_clean == "conversion"
                || goal_clean == "any"
                || goal_clean == ev_clean
                || (!gname_clean.is_empty() && gname_clean.contains(&goal_clean));
            if !matches {
                continue;
            }
            let updated = conn
                .execute(
                    "UPDATE daypress_ab_exposures SET converted = 1 WHERE test_id = ?1 AND visitor_hash = ?2 AND converted = 0",
                    params![test_id, visitor_hash],
                )
                .unwrap_or(0);
            if updated > 0 {
                if variant == "B" {
                    let _ = conn.execute(
                        "UPDATE daypress_ab_tests SET variant_conversions = variant_conversions + 1 WHERE id = ?1",
                        params![test_id],
                    );
                } else {
                    let _ = conn.execute(
                        "UPDATE daypress_ab_tests SET control_conversions = control_conversions + 1 WHERE id = ?1",
                        params![test_id],
                    );
                }
            }
        }
    }

    /// 1-Click Promote Winner:
    /// Marks the A/B test `completed` with `winner = 'A' | 'B'`. If `winning_variant == "B"`,
    /// permanently replaces `control_text` with `variant_text` in the target post's `post_content`
    /// and `layout_json` so Variant B becomes the canonical page!
    pub fn promote_ab_test_winner(
        &self,
        test_id: i64,
        winning_variant: &str,
    ) -> std::result::Result<AbTestRecord, AppError> {
        let winner = if winning_variant.trim().eq_ignore_ascii_case("A") {
            "A"
        } else {
            "B"
        };
        let (post_id, control_text, variant_text): (i64, String, String) = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT post_id, control_text, variant_text FROM daypress_ab_tests WHERE id = ?1",
                params![test_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(|_| AppError::NotFound(format!("A/B test #{test_id} not found")))?
        };

        {
            let conn = self.conn.lock().unwrap();
            if winner == "B" && !control_text.is_empty() {
                if let Ok((content, layout_opt)) = conn.query_row(
                    "SELECT post_content, layout_json FROM posts WHERE ID = ?1",
                    params![post_id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
                ) {
                    let new_content = content.replace(&control_text, &variant_text);
                    let new_layout = layout_opt.map(|lj| lj.replace(&control_text, &variant_text));
                    let _ = conn.execute(
                        "UPDATE posts SET post_content = ?1, layout_json = ?2 WHERE ID = ?3",
                        params![new_content, new_layout, post_id],
                    );
                    if let Some(ref lj) = new_layout {
                        let _ = conn.execute(
                            "UPDATE postmeta SET meta_value = ?1 WHERE post_id = ?2 AND meta_key = '_daypress_layout'",
                            params![lj, post_id],
                        );
                    }
                }
            }
            conn.execute(
                "UPDATE daypress_ab_tests SET status = 'completed', winner = ?1 WHERE id = ?2",
                params![winner, test_id],
            )?;
        }

        self.invalidate_hot_cache("ab_test_promoted");
        let conn = self.conn.lock().unwrap();
        let rec = conn.query_row(
            r#"SELECT t.id, t.name, t.post_id, COALESCE(p.post_title, 'Post #' || t.post_id),
                      t.target_selector, t.control_text, t.variant_text, t.goal_event, t.traffic_split,
                      t.control_views, t.control_conversions, t.variant_views, t.variant_conversions,
                      t.status, t.winner, t.created_at
               FROM daypress_ab_tests t
               LEFT JOIN posts p ON p.ID = t.post_id
               WHERE t.id = ?1"#,
            params![test_id],
            Self::map_ab_test_row,
        )?;
        Ok(rec)
    }

    // =========================================================================
    // Real-User Core Web Vitals (`LCP` / `INP` / `CLS` / `TTFB` p75)
    // =========================================================================

    pub fn record_web_vitals_beacon(&self, req: &WebVitalsBeaconRequest) -> Result<()> {
        let page_url = if req.page_url.trim().is_empty() {
            "/"
        } else {
            req.page_url.trim()
        };
        let device = match req.device_type.trim().to_ascii_lowercase().as_str() {
            "mobile" => "mobile",
            "tablet" => "tablet",
            _ => "desktop",
        };
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO daypress_web_vitals
               (page_url, device_type, lcp_ms, inp_ms, cls_score, ttfb_ms, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                page_url,
                device,
                req.lcp_ms,
                req.inp_ms,
                req.cls_score,
                req.ttfb_ms,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn get_core_web_vitals_summary(&self) -> Result<CoreWebVitalsSummary> {
        self.get_core_web_vitals_summary_for_range(None)
    }

    pub fn get_core_web_vitals_summary_for_range(
        &self,
        range: Option<&str>,
    ) -> Result<CoreWebVitalsSummary> {
        use chrono::Duration;
        use std::collections::BTreeMap;

        let cutoff_opt: Option<String> = range.and_then(|r| {
            let now = Utc::now();
            match r.trim() {
                "24h" | "today" => Some(
                    (now - Duration::hours(24))
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                ),
                "7d" => Some(
                    (now - Duration::days(7))
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                ),
                "30d" => Some(
                    (now - Duration::days(30))
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                ),
                "90d" => Some(
                    (now - Duration::days(90))
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                ),
                "12m" => Some(
                    (now - Duration::days(365))
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                ),
                "all" => None,
                _ => None,
            }
        });

        let conn = self.conn.lock().unwrap();
        let mut lcps = Vec::new();
        let mut inps = Vec::new();
        let mut clss = Vec::new();
        let mut ttfbs = Vec::new();
        let mut count = 0_i64;

        type RouteBucket = (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, i64);
        let mut by_route: BTreeMap<String, RouteBucket> = BTreeMap::new();

        let mut ingest_row = |route: String,
                              lcp: Option<f64>,
                              inp: Option<f64>,
                              cls: Option<f64>,
                              ttfb: Option<f64>| {
            count += 1;
            let entry = by_route
                .entry(route)
                .or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new(), 0));
            entry.4 += 1;
            if let Some(v) = lcp {
                lcps.push(v);
                entry.0.push(v);
            }
            if let Some(v) = inp {
                inps.push(v);
                entry.1.push(v);
            }
            if let Some(v) = cls {
                clss.push(v);
                entry.2.push(v);
            }
            if let Some(v) = ttfb {
                ttfbs.push(v);
                entry.3.push(v);
            }
        };

        if let Some(ref cutoff) = cutoff_opt {
            let mut stmt = conn.prepare(
                r#"SELECT page_url, lcp_ms, inp_ms, cls_score, ttfb_ms
                   FROM daypress_web_vitals
                   WHERE created_at >= ?1
                   ORDER BY id DESC
                   LIMIT 2000"#,
            )?;
            let rows = stmt.query_map(params![cutoff], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<f64>>(1)?,
                    r.get::<_, Option<f64>>(2)?,
                    r.get::<_, Option<f64>>(3)?,
                    r.get::<_, Option<f64>>(4)?,
                ))
            })?;
            for row in rows.flatten() {
                ingest_row(row.0, row.1, row.2, row.3, row.4);
            }
        } else {
            let mut stmt = conn.prepare(
                r#"SELECT page_url, lcp_ms, inp_ms, cls_score, ttfb_ms
                   FROM daypress_web_vitals
                   ORDER BY id DESC
                   LIMIT 1000"#,
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<f64>>(1)?,
                    r.get::<_, Option<f64>>(2)?,
                    r.get::<_, Option<f64>>(3)?,
                    r.get::<_, Option<f64>>(4)?,
                ))
            })?;
            for row in rows.flatten() {
                ingest_row(row.0, row.1, row.2, row.3, row.4);
            }
        }

        let rate_cwv = |n: i64,
                        lcp: f64,
                        inp: f64,
                        cls: f64|
         -> (&'static str, &'static str, &'static str, &'static str) {
            let l_st = if n == 0 || lcp <= 2500.0 {
                "good"
            } else if lcp <= 4000.0 {
                "needs_improvement"
            } else {
                "poor"
            };
            let i_st = if n == 0 || inp <= 200.0 {
                "good"
            } else if inp <= 500.0 {
                "needs_improvement"
            } else {
                "poor"
            };
            let c_st = if n == 0 || cls <= 0.10 {
                "good"
            } else if cls <= 0.25 {
                "needs_improvement"
            } else {
                "poor"
            };
            let ov = if [l_st, i_st, c_st].contains(&"poor") {
                "poor"
            } else if [l_st, i_st, c_st].contains(&"needs_improvement") {
                "needs_improvement"
            } else {
                "good"
            };
            (l_st, i_st, c_st, ov)
        };

        let lcp_p75 = percentile_p75(lcps);
        let inp_p75 = percentile_p75(inps);
        let cls_p75 = percentile_p75(clss);
        let ttfb_p75 = percentile_p75(ttfbs);
        let (lcp_status, inp_status, cls_status, overall_status) =
            rate_cwv(count, lcp_p75, inp_p75, cls_p75);

        let mut routes: Vec<RouteWebVitalsMetric> = by_route
            .into_iter()
            .map(|(page_url, (rl, ri, rc, rt, samples))| {
                let lp = percentile_p75(rl);
                let ip = percentile_p75(ri);
                let cp = percentile_p75(rc);
                let tp = percentile_p75(rt);
                let (_, _, _, st) = rate_cwv(samples, lp, ip, cp);
                RouteWebVitalsMetric {
                    page_url,
                    sample_count: samples,
                    lcp_p75_ms: lp,
                    inp_p75_ms: ip,
                    cls_p75: cp,
                    ttfb_p75_ms: tp,
                    status: st.to_string(),
                }
            })
            .collect();
        routes.sort_by(|a, b| {
            b.lcp_p75_ms
                .partial_cmp(&a.lcp_p75_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.sample_count.cmp(&a.sample_count))
        });
        routes.truncate(8);

        Ok(CoreWebVitalsSummary {
            sample_count: count,
            lcp_p75_ms: lcp_p75,
            inp_p75_ms: inp_p75,
            cls_p75,
            ttfb_p75_ms: ttfb_p75,
            lcp_status: lcp_status.to_string(),
            inp_status: inp_status.to_string(),
            cls_status: cls_status.to_string(),
            overall_status: overall_status.to_string(),
            routes,
        })
    }
}
