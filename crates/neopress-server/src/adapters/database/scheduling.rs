//! Scheduled posts (`future` status), WordPress style.
//!
//! * Saving a post as `publish` (or `future`) with a date at least a minute ahead stores it
//!   as `future`; a `future` post whose date is not ahead is stored as `publish`
//!   (WordPress `wp_insert_post`). Attachments are never scheduled.
//! * Every public read path only shows `publish`, so a `future` post stays hidden.
//! * Due posts are promoted lazily: [`Database::publish_due_scheduled_posts`] runs a single
//!   indexed `UPDATE` (a no-op when nothing is due). The HTTP layer calls it before each
//!   request (`handlers::scheduled_posts`) and the MCP repository port calls it before reads,
//!   so nothing can observe a post whose date has passed while it is still `future`.
//!
//! Neopress keeps site time in UTC (`gmt_offset` is always 0): saving a date writes the same
//! UTC timestamp to `post_date` and `post_date_gmt`. A date with an explicit offset is
//! converted to UTC; a date without one is taken as UTC. `post_date` is the one compared,
//! since importers that update an existing post only rewrite `post_date`.

use super::Database;
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result};

/// Storage format of `post_date` / `post_date_gmt`.
pub const DB_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// How far ahead (seconds) a date must be for `publish` to become `future` (as in WordPress).
const SCHEDULE_THRESHOLD_SECS: i64 = 60;

/// Parses a REST or editor date (`2026-11-01T09:30:00`, `2026-11-01 09:30`,
/// `2026-11-01T09:30:00Z`, `2026-11-01T11:30:00+02:00`, ...) into the stored UTC form.
pub fn normalize_post_date(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc).format(DB_DATE_FORMAT).to_string());
    }
    const NAIVE_FORMATS: [&str; 6] = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M",
    ];
    NAIVE_FORMATS
        .iter()
        .find_map(|fmt| NaiveDateTime::parse_from_str(s, fmt).ok())
        .map(|n| n.format(DB_DATE_FORMAT).to_string())
}

/// The UTC date a create/update request asks for: `date_gmt` wins over `date` (both are UTC
/// here unless they carry an offset). Empty or unparseable values count as "not given".
pub fn requested_post_date(date: Option<&str>, date_gmt: Option<&str>) -> Option<String> {
    date_gmt
        .and_then(normalize_post_date)
        .or_else(|| date.and_then(normalize_post_date))
}

/// Checks the `date` / `date_gmt` fields of a REST request, naming the first invalid one.
pub fn validate_requested_dates(
    date: Option<&str>,
    date_gmt: Option<&str>,
) -> std::result::Result<(), String> {
    for (field, value) in [("date", date), ("date_gmt", date_gmt)] {
        if let Some(v) = value.filter(|v| !v.trim().is_empty()) {
            if normalize_post_date(v).is_none() {
                return Err(format!(
                    "Invalid {field}: expected an ISO 8601 date such as 2026-11-01T09:30:00"
                ));
            }
        }
    }
    Ok(())
}

fn parse_db_date(raw: &str) -> Option<NaiveDateTime> {
    let s = raw.trim().replace('T', " ");
    if s.starts_with("0000") {
        return None;
    }
    NaiveDateTime::parse_from_str(&s, DB_DATE_FORMAT)
        .ok()
        .or_else(|| {
            normalize_post_date(&s)
                .and_then(|n| NaiveDateTime::parse_from_str(&n, DB_DATE_FORMAT).ok())
        })
}

/// The status a post is stored with, given its requested status and (UTC) post date.
pub fn resolve_scheduled_status(
    status: &str,
    post_type: &str,
    post_date: &str,
    now: &NaiveDateTime,
) -> String {
    if post_type == "attachment" || !matches!(status, "publish" | "future") {
        return status.to_string();
    }
    match parse_db_date(post_date) {
        Some(date) if (date - *now).num_seconds() >= SCHEDULE_THRESHOLD_SECS => "future".into(),
        Some(_) => "publish".into(),
        // Without a usable date a `future` post could never go live: publish it now.
        None => "publish".into(),
    }
}

fn now_utc(now: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(now, DB_DATE_FORMAT).unwrap_or_else(|_| Utc::now().naive_utc())
}

/// `(post_date, post_status)` for a new post created at `now` (`DB_DATE_FORMAT`, UTC).
pub(crate) fn schedule_for_new_post(
    status: &str,
    post_type: &str,
    date: Option<&str>,
    date_gmt: Option<&str>,
    now: &str,
) -> (String, String) {
    let post_date = requested_post_date(date, date_gmt).unwrap_or_else(|| now.to_string());
    let status = resolve_scheduled_status(status, post_type, &post_date, &now_utc(now));
    (post_date, status)
}

/// Applies an update's date (if any) and re-resolves the status when the status or the
/// date changed. Runs inside `update_post`'s transaction.
pub(crate) fn apply_schedule_on_update(
    conn: &Connection,
    id: i64,
    status_changed: bool,
    new_date: Option<&str>,
    now: &str,
) -> Result<()> {
    if let Some(date) = new_date {
        conn.execute(
            "UPDATE wp_posts SET post_date = ?1, post_date_gmt = ?1, post_modified = ?2 WHERE ID = ?3",
            params![date, now, id],
        )?;
    }
    if !status_changed && new_date.is_none() {
        return Ok(());
    }
    let row: Option<(String, String, String, String)> = conn
        .query_row(
            "SELECT post_status, post_type, post_date, post_date_gmt FROM wp_posts WHERE ID = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let Some((status, post_type, date, date_gmt)) = row else {
        return Ok(());
    };
    let effective = if parse_db_date(&date).is_some() {
        date
    } else {
        date_gmt
    };
    let resolved = resolve_scheduled_status(&status, &post_type, &effective, &now_utc(now));
    if resolved != status {
        conn.execute(
            "UPDATE wp_posts SET post_status = ?1 WHERE ID = ?2",
            params![resolved, id],
        )?;
    }
    Ok(())
}

/// Publishes every `future` post whose date has passed at `now`; returns how many.
pub(crate) fn publish_due_posts(conn: &Connection, now: &NaiveDateTime) -> Result<usize> {
    let mut stmt = conn.prepare_cached(
        "UPDATE wp_posts SET post_status = 'publish'
         WHERE post_status = 'future'
           AND REPLACE(CASE WHEN post_date = '' OR post_date LIKE '0000%'
                            THEN post_date_gmt ELSE post_date END, 'T', ' ') <= ?1",
    )?;
    stmt.execute(params![now.format(DB_DATE_FORMAT).to_string()])
}

impl Database {
    /// Promotes scheduled (`future`) posts whose date has passed to `publish`.
    pub fn publish_due_scheduled_posts(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        publish_due_posts(&conn, &Utc::now().naive_utc())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, DB_DATE_FORMAT).unwrap()
    }

    #[test]
    fn normalizes_rest_and_editor_dates_to_utc() {
        assert_eq!(
            normalize_post_date("2026-11-01T09:30:00").as_deref(),
            Some("2026-11-01 09:30:00")
        );
        assert_eq!(
            normalize_post_date("2026-11-01T09:30").as_deref(),
            Some("2026-11-01 09:30:00")
        );
        assert_eq!(
            normalize_post_date("2026-11-01T11:30:00+02:00").as_deref(),
            Some("2026-11-01 09:30:00")
        );
        assert_eq!(
            normalize_post_date("2026-11-01T09:30:00.123Z").as_deref(),
            Some("2026-11-01 09:30:00")
        );
        assert_eq!(normalize_post_date("next tuesday"), None);
        assert_eq!(normalize_post_date("  "), None);
        assert!(validate_requested_dates(Some("2026-13-40"), None).is_err());
        assert!(validate_requested_dates(Some(""), Some("2026-11-01 09:30:00")).is_ok());
    }

    #[test]
    fn resolves_status_like_wordpress() {
        let now = at("2026-10-28 12:00:00");
        let ahead = "2026-10-29 12:00:00";
        let past = "2026-10-27 12:00:00";
        assert_eq!(
            resolve_scheduled_status("publish", "post", ahead, &now),
            "future"
        );
        assert_eq!(
            resolve_scheduled_status("future", "page", ahead, &now),
            "future"
        );
        assert_eq!(
            resolve_scheduled_status("future", "post", past, &now),
            "publish"
        );
        assert_eq!(
            resolve_scheduled_status("publish", "post", past, &now),
            "publish"
        );
        assert_eq!(
            resolve_scheduled_status("draft", "post", ahead, &now),
            "draft"
        );
        assert_eq!(
            resolve_scheduled_status("publish", "attachment", ahead, &now),
            "publish"
        );
        // Less than a minute ahead publishes right away.
        assert_eq!(
            resolve_scheduled_status("publish", "post", "2026-10-28 12:00:30", &now),
            "publish"
        );
        assert_eq!(
            resolve_scheduled_status("future", "post", "0000-00-00 00:00:00", &now),
            "publish"
        );
    }

    #[test]
    fn publishes_only_due_future_posts() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        for (title, status, date) in [
            ("Due", "future", "2026-10-28 11:59:00"),
            ("Due iso", "future", "2026-10-28T11:30:00"),
            ("Later", "future", "2026-10-28 12:01:00"),
            ("Draft", "draft", "2026-10-01 00:00:00"),
        ] {
            conn.execute(
                "INSERT INTO wp_posts (post_date, post_date_gmt, post_modified, post_modified_gmt, post_title, post_status, post_name, post_type)
                 VALUES (?1, ?1, ?1, ?1, ?2, ?3, ?2, 'post')",
                params![date, title, status],
            )
            .unwrap();
        }
        let now = at("2026-10-28 12:00:00");
        assert_eq!(publish_due_posts(&conn, &now).unwrap(), 2);
        assert_eq!(publish_due_posts(&conn, &now).unwrap(), 0);
        let status_of = |title: &str| -> String {
            conn.query_row(
                "SELECT post_status FROM wp_posts WHERE post_title = ?1",
                params![title],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(status_of("Due"), "publish");
        assert_eq!(status_of("Due iso"), "publish");
        assert_eq!(status_of("Later"), "future");
        assert_eq!(status_of("Draft"), "draft");
    }
}
