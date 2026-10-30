//! Scheduled posts (`future` status) and site time zone support, WordPress style.
//!
//! * Saving a post as `publish` (or `future`) with a date at least a minute ahead stores it
//!   as `future`; a `future` post whose date is not ahead is stored as `publish`
//!   (WordPress `wp_insert_post`). Attachments are never scheduled.
//! * Every public read path only shows `publish`, so a `future` post stays hidden.
//! * Due posts are promoted lazily: [`Database::publish_due_scheduled_posts`] runs a single
//!   indexed `UPDATE` (a no-op when nothing is due). The HTTP layer calls it before each
//!   request (`handlers::scheduled_posts`) and the MCP repository port calls it before reads,
//!   so nothing can observe a post whose date has passed while it is still `future`.
//! * Site time zone (`timezone_string` / `gmt_offset` in `options`):
//!   - Naive dates submitted via `date` (editor or REST API) or imported without `post_date_gmt`
//!     are interpreted in the site's configured timezone (`post_date` holds local site time,
//!     `post_date_gmt` holds the corresponding UTC timestamp).
//!   - When `date_gmt` is provided, `post_date_gmt` uses it directly and `post_date` is shifted
//!     to site local time (unless `date` was also explicitly given).

use super::Database;
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Utc, Weekday};
use rusqlite::{params, Connection, OptionalExtension, Result};

/// Storage format of `post_date` / `post_date_gmt`.
pub const DB_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// How far ahead (seconds) a date must be for `publish` to become `future` (as in WordPress).
const SCHEDULE_THRESHOLD_SECS: i64 = 60;

/// Returns the day-of-month of the last occurrence of `weekday` in `(year, month)`.
fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> u32 {
    let last_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    };
    for d in (1..=last_day).rev() {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, d) {
            if date.weekday() == weekday {
                return d;
            }
        }
    }
    last_day
}

/// Returns the day-of-month of the Nth (1-based) occurrence of `weekday` in `(year, month)`.
fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, n: u32) -> u32 {
    let mut count = 0;
    for d in 1..=31 {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, d) {
            if date.weekday() == weekday {
                count += 1;
                if count == n {
                    return d;
                }
            }
        }
    }
    1
}

/// Checks if a naive local datetime falls within European Summer Time (last Sunday of March 02:00 to last Sunday of October 03:00).
fn is_eu_dst(dt: NaiveDateTime) -> bool {
    let y = dt.year();
    let m = dt.month();
    if m > 3 && m < 10 {
        return true;
    }
    if !(3..=10).contains(&m) {
        return false;
    }
    if m == 3 {
        let sun = last_weekday_of_month(y, 3, Weekday::Sun);
        let d = dt.day();
        d > sun || (d == sun && dt.hour() >= 2)
    } else {
        let sun = last_weekday_of_month(y, 10, Weekday::Sun);
        let d = dt.day();
        d < sun || (d == sun && dt.hour() < 3)
    }
}

/// Checks if a naive local datetime falls within North American DST (2nd Sunday of March 02:00 to 1st Sunday of November 02:00).
fn is_us_dst(dt: NaiveDateTime) -> bool {
    let y = dt.year();
    let m = dt.month();
    if m > 3 && m < 11 {
        return true;
    }
    if !(3..=11).contains(&m) {
        return false;
    }
    if m == 3 {
        let sun = nth_weekday_of_month(y, 3, Weekday::Sun, 2);
        let d = dt.day();
        d > sun || (d == sun && dt.hour() >= 2)
    } else {
        let sun = nth_weekday_of_month(y, 11, Weekday::Sun, 1);
        let d = dt.day();
        d < sun || (d == sun && dt.hour() < 2)
    }
}

/// Checks if a naive local datetime falls within Southern Hemisphere (Australia) DST (1st Sunday of October to 1st Sunday of April).
fn is_au_dst(dt: NaiveDateTime) -> bool {
    let y = dt.year();
    let m = dt.month();
    if !(4..=10).contains(&m) {
        return true;
    }
    if m > 4 && m < 10 {
        return false;
    }
    if m == 10 {
        let sun = nth_weekday_of_month(y, 10, Weekday::Sun, 1);
        let d = dt.day();
        d > sun || (d == sun && dt.hour() >= 2)
    } else {
        let sun = nth_weekday_of_month(y, 4, Weekday::Sun, 1);
        let d = dt.day();
        d < sun || (d == sun && dt.hour() < 3)
    }
}

/// Checks if a naive local datetime falls within New Zealand DST (last Sunday of September to 1st Sunday of April).
fn is_nz_dst(dt: NaiveDateTime) -> bool {
    let y = dt.year();
    let m = dt.month();
    if !(4..=9).contains(&m) {
        return true;
    }
    if m > 4 && m < 9 {
        return false;
    }
    if m == 9 {
        let sun = last_weekday_of_month(y, 9, Weekday::Sun);
        let d = dt.day();
        d > sun || (d == sun && dt.hour() >= 2)
    } else {
        let sun = nth_weekday_of_month(y, 4, Weekday::Sun, 1);
        let d = dt.day();
        d < sun || (d == sun && dt.hour() < 3)
    }
}

/// Parses an offset string like `"+2"`, `"-5:30"`, `"+05:45"`, `"UTC+2"`, `"UTC-5.5"`, `"Etc/GMT-2"`.
fn parse_explicit_offset_seconds(raw: &str) -> Option<i32> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if s.eq_ignore_ascii_case("UTC")
        || s.eq_ignore_ascii_case("GMT")
        || s.eq_ignore_ascii_case("Etc/UTC")
        || s.eq_ignore_ascii_case("Etc/GMT")
        || s.eq_ignore_ascii_case("Z")
    {
        return Some(0);
    }
    if let Some(rest) = s
        .strip_prefix("Etc/GMT")
        .or_else(|| s.strip_prefix("etc/gmt"))
    {
        if rest.is_empty() || rest == "0" || rest == "+0" || rest == "-0" {
            return Some(0);
        }
        // POSIX Etc/GMT inverts the sign: Etc/GMT-2 means UTC+2
        if let Ok(hours) = rest.parse::<f64>() {
            return Some((-hours * 3600.0).round() as i32);
        }
    }
    let body = s
        .strip_prefix("UTC")
        .or_else(|| s.strip_prefix("utc"))
        .or_else(|| s.strip_prefix("GMT"))
        .or_else(|| s.strip_prefix("gmt"))
        .unwrap_or(s)
        .trim();
    if body.is_empty() {
        return Some(0);
    }
    if let Some((h_str, m_str)) = body.split_once(':') {
        let sign = if h_str.trim().starts_with('-') { -1 } else { 1 };
        let hours: i32 = h_str.trim().trim_start_matches(['+', '-']).parse().ok()?;
        let mins: i32 = m_str.trim().parse().ok()?;
        return Some(sign * (hours * 3600 + mins * 60));
    }
    if let Ok(hours) = body.parse::<f64>() {
        if (-14.0..=14.0).contains(&hours) {
            return Some((hours * 3600.0).round() as i32);
        }
    }
    None
}

/// Resolves the UTC offset in seconds for a WordPress `timezone_string` (and optional `gmt_offset`)
/// at the given reference datetime (for DST-aware IANA zones).
pub fn resolve_timezone_offset_seconds(
    timezone_string: &str,
    gmt_offset: Option<&str>,
    ref_dt: Option<NaiveDateTime>,
) -> i32 {
    let tz = timezone_string.trim();
    if let Some(explicit) = parse_explicit_offset_seconds(tz) {
        return explicit;
    }
    let dt = ref_dt.unwrap_or_else(|| Utc::now().naive_utc());

    #[derive(Clone, Copy)]
    enum DstRule {
        None,
        Eu,
        Us,
        Au,
        Nz,
    }

    let (std_secs, rule) = match tz {
        // UTC / UK / Portugal
        "Europe/London" | "Europe/Dublin" | "Europe/Lisbon" | "Atlantic/Canary" => (0, DstRule::Eu),
        "Atlantic/Reykjavik" | "Africa/Abidjan" | "Africa/Accra" | "Africa/Dakar" => {
            (0, DstRule::None)
        }
        "Atlantic/Azores" => (-3600, DstRule::Eu),
        // Central European Time (UTC+1 / UTC+2 CEST)
        "Europe/Amsterdam" | "Europe/Andorra" | "Europe/Belgrade" | "Europe/Berlin"
        | "Europe/Bratislava" | "Europe/Brussels" | "Europe/Budapest" | "Europe/Copenhagen"
        | "Europe/Gibraltar" | "Europe/Ljubljana" | "Europe/Luxembourg" | "Europe/Madrid"
        | "Europe/Malta" | "Europe/Monaco" | "Europe/Oslo" | "Europe/Paris"
        | "Europe/Podgorica" | "Europe/Prague" | "Europe/Rome" | "Europe/San_Marino"
        | "Europe/Sarajevo" | "Europe/Skopje" | "Europe/Stockholm" | "Europe/Tirane"
        | "Europe/Vaduz" | "Europe/Vatican" | "Europe/Vienna" | "Europe/Warsaw"
        | "Europe/Zagreb" | "Europe/Zurich" => (3600, DstRule::Eu),
        // Eastern European Time (UTC+2 / UTC+3 EEST)
        "Europe/Athens" | "Europe/Bucharest" | "Europe/Chisinau" | "Europe/Helsinki"
        | "Europe/Kiev" | "Europe/Kyiv" | "Europe/Mariehamn" | "Europe/Nicosia" | "Europe/Riga"
        | "Europe/Sofia" | "Europe/Tallinn" | "Europe/Uzhgorod" | "Europe/Vilnius"
        | "Europe/Zaporozhye" => (2 * 3600, DstRule::Eu),
        // Further Eastern Europe & Middle East / Africa
        "Europe/Istanbul" | "Europe/Moscow" | "Europe/Minsk" | "Asia/Riyadh" | "Asia/Baghdad"
        | "Asia/Kuwait" | "Asia/Qatar" | "Asia/Bahrain" | "Africa/Nairobi"
        | "Africa/Addis_Ababa" => (3 * 3600, DstRule::None),
        "Africa/Cairo" | "Africa/Johannesburg" | "Africa/Harare" | "Africa/Maputo" => {
            (2 * 3600, DstRule::None)
        }
        "Africa/Lagos" | "Africa/Algiers" | "Africa/Tunis" | "Africa/Casablanca" => {
            (3600, DstRule::None)
        }
        "Asia/Jerusalem" | "Asia/Tel_Aviv" => (2 * 3600, DstRule::Eu),
        "Asia/Tehran" => (3 * 3600 + 1800, DstRule::None),
        "Asia/Dubai" | "Asia/Muscat" | "Asia/Baku" | "Asia/Tbilisi" | "Asia/Yerevan" => {
            (4 * 3600, DstRule::None)
        }
        "Asia/Kabul" => (4 * 3600 + 1800, DstRule::None),
        "Asia/Karachi" | "Asia/Tashkent" | "Asia/Yekaterinburg" | "Asia/Samarkand" => {
            (5 * 3600, DstRule::None)
        }
        "Asia/Kolkata" | "Asia/Calcutta" | "Asia/Colombo" => (5 * 3600 + 1800, DstRule::None),
        "Asia/Kathmandu" => (5 * 3600 + 2700, DstRule::None),
        "Asia/Dhaka" | "Asia/Almaty" | "Asia/Bishkek" => (6 * 3600, DstRule::None),
        "Asia/Yangon" | "Asia/Rangoon" => (6 * 3600 + 1800, DstRule::None),
        "Asia/Bangkok" | "Asia/Ho_Chi_Minh" | "Asia/Saigon" | "Asia/Jakarta"
        | "Asia/Phnom_Penh" | "Asia/Vientiane" => (7 * 3600, DstRule::None),
        "Asia/Singapore" | "Asia/Kuala_Lumpur" | "Asia/Manila" | "Asia/Hong_Kong"
        | "Asia/Shanghai" | "Asia/Chongqing" | "Asia/Taipei" | "Asia/Macau" | "Australia/Perth" => {
            (8 * 3600, DstRule::None)
        }
        "Asia/Tokyo" | "Asia/Seoul" | "Asia/Pyongyang" => (9 * 3600, DstRule::None),
        "Australia/Darwin" => (9 * 3600 + 1800, DstRule::None),
        "Australia/Adelaide" => (9 * 3600 + 1800, DstRule::Au),
        "Australia/Brisbane" | "Pacific/Guam" | "Pacific/Port_Moresby" => {
            (10 * 3600, DstRule::None)
        }
        "Australia/Sydney" | "Australia/Melbourne" | "Australia/Hobart" | "Australia/Canberra" => {
            (10 * 3600, DstRule::Au)
        }
        "Pacific/Auckland" => (12 * 3600, DstRule::Nz),
        "Pacific/Fiji" => (12 * 3600, DstRule::None),
        // Americas
        "Pacific/Honolulu" => (-10 * 3600, DstRule::None),
        "America/Anchorage" | "America/Juneau" => (-9 * 3600, DstRule::Us),
        "America/Los_Angeles" | "America/Vancouver" | "America/Tijuana" | "US/Pacific" => {
            (-8 * 3600, DstRule::Us)
        }
        "America/Phoenix" | "America/Hermosillo" | "US/Arizona" => (-7 * 3600, DstRule::None),
        "America/Denver" | "America/Edmonton" | "America/Boise" | "US/Mountain" => {
            (-7 * 3600, DstRule::Us)
        }
        "America/Chicago" | "America/Winnipeg" | "America/Dallas" | "US/Central" => {
            (-6 * 3600, DstRule::Us)
        }
        "America/Mexico_City" | "America/Guatemala" | "America/Costa_Rica" => {
            (-6 * 3600, DstRule::None)
        }
        "America/New_York"
        | "America/Detroit"
        | "America/Toronto"
        | "America/Montreal"
        | "America/Indiana/Indianapolis"
        | "US/Eastern" => (-5 * 3600, DstRule::Us),
        "America/Bogota" | "America/Lima" | "America/Panama" | "America/Guayaquil" => {
            (-5 * 3600, DstRule::None)
        }
        "America/Halifax" | "America/Moncton" | "Atlantic/Bermuda" => (-4 * 3600, DstRule::Us),
        "America/Caracas" | "America/La_Paz" | "America/Puerto_Rico" | "America/Santo_Domingo" => {
            (-4 * 3600, DstRule::None)
        }
        "America/Santiago" => (-4 * 3600, DstRule::Au),
        "America/St_Johns" => (-3 * 3600 - 1800, DstRule::Us),
        "America/Sao_Paulo"
        | "America/Argentina/Buenos_Aires"
        | "America/Buenos_Aires"
        | "America/Montevideo" => (-3 * 3600, DstRule::None),
        _ => {
            if let Some(off) = gmt_offset.and_then(parse_explicit_offset_seconds) {
                return off;
            }
            return 0;
        }
    };

    let dst_extra = match rule {
        DstRule::None => 0,
        DstRule::Eu => {
            if is_eu_dst(dt) {
                3600
            } else {
                0
            }
        }
        DstRule::Us => {
            if is_us_dst(dt) {
                3600
            } else {
                0
            }
        }
        DstRule::Au => {
            if is_au_dst(dt) {
                3600
            } else {
                0
            }
        }
        DstRule::Nz => {
            if is_nz_dst(dt) {
                3600
            } else {
                0
            }
        }
    };
    std_secs + dst_extra
}

/// Converts a timezone setting string into a numeric hours offset (`gmt_offset`, e.g. `2.0`, `-5.0`, `5.5`).
pub fn timezone_to_gmt_offset_hours(timezone_string: &str) -> f64 {
    let secs = resolve_timezone_offset_seconds(timezone_string, None, None);
    (secs as f64 / 3600.0 * 100.0).round() / 100.0
}

/// Reads the site's configured timezone (`timezone_string` / `gmt_offset`) from SQLite and returns
/// its UTC offset in seconds at `ref_dt`.
pub fn read_site_timezone_offset(conn: &Connection, ref_dt: Option<NaiveDateTime>) -> i32 {
    let tz: String = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = 'timezone_string'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let gmt_off: Option<String> = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = 'gmt_offset'",
            [],
            |r| r.get(0),
        )
        .ok();
    resolve_timezone_offset_seconds(&tz, gmt_off.as_deref(), ref_dt)
}

/// Parses a naive datetime string (`2026-11-01T09:30:00`, `2026-11-01 09:30`, etc.).
pub fn parse_naive_date(raw: &str) -> Option<NaiveDateTime> {
    let s = raw.trim();
    if s.is_empty() || s.starts_with("0000") {
        return None;
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
}

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
    parse_naive_date(s).map(|n| n.format(DB_DATE_FORMAT).to_string())
}

/// Converts a raw date string (either RFC3339 with explicit offset or naive site-local time)
/// into `(local_post_date, utc_post_date_gmt)` using the site's timezone settings.
pub fn normalize_post_date_with_tz(
    raw: &str,
    timezone_string: &str,
    gmt_offset: Option<&str>,
) -> Option<(String, String)> {
    let s = raw.trim();
    if s.is_empty() || s.starts_with("0000") {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        let utc_naive = dt.with_timezone(&Utc).naive_utc();
        let offset_secs =
            resolve_timezone_offset_seconds(timezone_string, gmt_offset, Some(utc_naive));
        let local_naive = utc_naive + Duration::seconds(offset_secs as i64);
        return Some((
            local_naive.format(DB_DATE_FORMAT).to_string(),
            utc_naive.format(DB_DATE_FORMAT).to_string(),
        ));
    }
    let local_naive = parse_naive_date(s)?;
    let offset_secs =
        resolve_timezone_offset_seconds(timezone_string, gmt_offset, Some(local_naive));
    let utc_naive = local_naive - Duration::seconds(offset_secs as i64);
    Some((
        local_naive.format(DB_DATE_FORMAT).to_string(),
        utc_naive.format(DB_DATE_FORMAT).to_string(),
    ))
}

/// Resolves `(local_post_date, utc_post_date_gmt)` from optional `date` (site local) and `date_gmt` (UTC).
pub fn requested_post_dates_with_tz(
    date: Option<&str>,
    date_gmt: Option<&str>,
    timezone_string: &str,
    gmt_offset: Option<&str>,
) -> Option<(String, String)> {
    let has_date_gmt = date_gmt
        .map(|v| !v.trim().is_empty() && !v.trim().starts_with("0000"))
        .unwrap_or(false);
    let has_date = date
        .map(|v| !v.trim().is_empty() && !v.trim().starts_with("0000"))
        .unwrap_or(false);

    if has_date_gmt {
        let utc_str = normalize_post_date(date_gmt.unwrap_or_default())?;
        let utc_naive = parse_naive_date(&utc_str)?;
        if has_date {
            if let Some((local_str, _)) =
                normalize_post_date_with_tz(date.unwrap_or_default(), timezone_string, gmt_offset)
            {
                return Some((local_str, utc_str));
            }
        }
        let offset_secs =
            resolve_timezone_offset_seconds(timezone_string, gmt_offset, Some(utc_naive));
        let local_naive = utc_naive + Duration::seconds(offset_secs as i64);
        return Some((local_naive.format(DB_DATE_FORMAT).to_string(), utc_str));
    }

    if has_date {
        return normalize_post_date_with_tz(date.unwrap_or_default(), timezone_string, gmt_offset);
    }

    None
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
#[allow(dead_code)]
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

/// `(local_post_date, utc_post_date_gmt, post_status)` for a new post created at `now` (UTC),
/// respecting the site's configured `timezone_string` / `gmt_offset`.
pub(crate) fn schedule_for_new_post_in_site_tz(
    conn: &Connection,
    status: &str,
    post_type: &str,
    date: Option<&str>,
    date_gmt: Option<&str>,
    now: &str,
) -> (String, String, String) {
    let tz: String = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = 'timezone_string'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let gmt_off: Option<String> = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = 'gmt_offset'",
            [],
            |r| r.get(0),
        )
        .ok();
    let now_naive = now_utc(now);
    let (local_date, utc_date) =
        requested_post_dates_with_tz(date, date_gmt, &tz, gmt_off.as_deref()).unwrap_or_else(
            || {
                let offset_secs =
                    resolve_timezone_offset_seconds(&tz, gmt_off.as_deref(), Some(now_naive));
                let local_now = now_naive + Duration::seconds(offset_secs as i64);
                (
                    local_now.format(DB_DATE_FORMAT).to_string(),
                    now.to_string(),
                )
            },
        );
    let resolved_status = resolve_scheduled_status(status, post_type, &utc_date, &now_naive);
    (local_date, utc_date, resolved_status)
}

/// Applies an update's date (if any) and re-resolves the status when the status or the
/// date changed. Runs inside `update_post`'s transaction.
pub(crate) fn apply_schedule_on_update(
    conn: &Connection,
    id: i64,
    status_changed: bool,
    new_dates: Option<(&str, &str)>,
    now: &str,
) -> Result<()> {
    let tz: String = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = 'timezone_string'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let gmt_off: Option<String> = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = 'gmt_offset'",
            [],
            |r| r.get(0),
        )
        .ok();
    let now_naive = now_utc(now);
    let now_offset = resolve_timezone_offset_seconds(&tz, gmt_off.as_deref(), Some(now_naive));
    let local_now = (now_naive + Duration::seconds(now_offset as i64))
        .format(DB_DATE_FORMAT)
        .to_string();

    if let Some((local_date, utc_date)) = new_dates {
        conn.execute(
            "UPDATE posts SET post_date = ?1, post_date_gmt = ?2, post_modified = ?3, post_modified_gmt = ?4 WHERE ID = ?5",
            params![local_date, utc_date, local_now, now, id],
        )?;
    }
    if !status_changed && new_dates.is_none() {
        return Ok(());
    }
    let row: Option<(String, String, String, String)> = conn
        .query_row(
            "SELECT post_status, post_type, post_date, post_date_gmt FROM posts WHERE ID = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let Some((status, post_type, date, date_gmt)) = row else {
        return Ok(());
    };
    let effective_utc = if parse_db_date(&date_gmt).is_some() {
        date_gmt
    } else if let Some((_, utc)) = normalize_post_date_with_tz(&date, &tz, gmt_off.as_deref()) {
        utc
    } else {
        date
    };
    let resolved = resolve_scheduled_status(&status, &post_type, &effective_utc, &now_naive);
    if resolved != status {
        conn.execute(
            "UPDATE posts SET post_status = ?1 WHERE ID = ?2",
            params![resolved, id],
        )?;
    }
    Ok(())
}

/// Publishes every `future` post whose UTC date has passed at `now`; returns how many.
pub(crate) fn publish_due_posts(conn: &Connection, now: &NaiveDateTime) -> Result<usize> {
    let mut stmt = conn.prepare_cached(
        "UPDATE posts SET post_status = 'publish'
         WHERE post_status = 'future'
           AND REPLACE(CASE WHEN post_date_gmt = '' OR post_date_gmt LIKE '0000%'
                            THEN post_date ELSE post_date_gmt END, 'T', ' ') <= ?1",
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
    fn converts_naive_dates_with_site_timezone() {
        // Europe/Amsterdam in winter (November) is UTC+1
        let (local, utc) =
            normalize_post_date_with_tz("2026-11-01T11:30:00", "Europe/Amsterdam", None).unwrap();
        assert_eq!(local, "2026-11-01 11:30:00");
        assert_eq!(utc, "2026-11-01 10:30:00");

        // Europe/Amsterdam in summer (July) is UTC+2 (CEST)
        let (local_s, utc_s) =
            normalize_post_date_with_tz("2026-07-15T11:30:00", "Europe/Amsterdam", None).unwrap();
        assert_eq!(local_s, "2026-07-15 11:30:00");
        assert_eq!(utc_s, "2026-07-15 09:30:00");

        // America/New_York in winter (December) is UTC-5
        let (local_ny, utc_ny) =
            normalize_post_date_with_tz("2026-12-01T09:00:00", "America/New_York", None).unwrap();
        assert_eq!(local_ny, "2026-12-01 09:00:00");
        assert_eq!(utc_ny, "2026-12-01 14:00:00");

        // Explicit UTC+05:30
        let (local_in, utc_in) =
            normalize_post_date_with_tz("2026-11-01 15:00:00", "UTC+5:30", None).unwrap();
        assert_eq!(local_in, "2026-11-01 15:00:00");
        assert_eq!(utc_in, "2026-11-01 09:30:00");
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
                "INSERT INTO posts (post_date, post_date_gmt, post_modified, post_modified_gmt, post_title, post_status, post_name, post_type)
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
                "SELECT post_status FROM posts WHERE post_title = ?1",
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
