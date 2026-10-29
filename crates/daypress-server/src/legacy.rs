//! Upgrade path for installs made before Neopress was renamed to Daypress (v0.1.x).
//!
//! This is the only place that still knows the old names. Dropping support for upgrading
//! from v0.1 means deleting this module and its call sites:
//!
//! * [`adopt_legacy_database_file`]: `data/neopress.sqlite` becomes `data/daypress.sqlite`.
//! * [`upgrade_schema`]: tables lose WordPress's `wp_` prefix (`wp_posts` → `posts`,
//!   `wp_neopress_forms` → `daypress_forms`); then [`upgrade_data`] renames `neopress_*`
//!   options, `_neopress_*` meta keys, built-in plugin slugs, and the class names, CSS tokens,
//!   shortcodes and REST paths stored in generated markup.
//! * [`snapshot_table`]: backups made by Neopress can still be restored.
//! * [`env_var`]: `NEOPRESS_*` environment variables still work.
//! * [`routes`] and [`REST_NAMESPACE`]: `/n-admin/*` redirects to `/d-admin/*`, and
//!   `/wp-json/neopress/v1/*` keeps serving the native API for already-installed migrator
//!   plugins and imported forms that post to it.

use axum::{http::Uri, response::Redirect, routing::any, Router};
use regex::{Captures, Regex};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The pre-rebrand native REST namespace, served next to `/daypress/v1`.
pub const REST_NAMESPACE: &str = "/neopress/v1";

const LEGACY_ADMIN: &str = "/n-admin";
const ADMIN: &str = "/d-admin";

/// Redirects for the pre-rebrand admin URLs (bookmarks, links in old emails).
pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route(LEGACY_ADMIN, any(redirect_admin))
        .route("/n-admin/{*rest}", any(redirect_admin))
}

/// Permanent redirect that keeps the method and body, so old admin API calls still land.
async fn redirect_admin(uri: Uri) -> Redirect {
    let target = uri.path_and_query().map_or(uri.path(), |pq| pq.as_str());
    let rest = target.strip_prefix(LEGACY_ADMIN).unwrap_or("/");
    Redirect::permanent(&format!("{ADMIN}{rest}"))
}

/// Reads `name`, falling back to its `NEOPRESS_*` spelling for `DAYPRESS_*` variables.
pub fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().or_else(|| {
        let suffix = name.strip_prefix("DAYPRESS_")?;
        std::env::var(format!("NEOPRESS_{suffix}")).ok()
    })
}

/// Moves a pre-rebrand database (`neopress.sqlite` next to the configured
/// `daypress.sqlite`) to the configured path when that path does not exist yet.
/// Returns the file it moved.
pub fn adopt_legacy_database_file(
    path: &Path,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return Ok(None);
    };
    if path.exists() || !name.contains("daypress") {
        return Ok(None);
    }
    let legacy = path.with_file_name(name.replace("daypress", "neopress"));
    if !legacy.is_file() {
        return Ok(None);
    }

    // Fold the WAL back into the main file first, so only that one file has to move.
    {
        let conn = Connection::open(&legacy)?;
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE);", [], |_| Ok(()))?;
    }
    for suffix in ["-wal", "-shm", "-journal"] {
        let companion = with_suffix(&legacy, suffix);
        if companion.exists() {
            std::fs::rename(&companion, with_suffix(path, suffix))?;
        }
    }
    std::fs::rename(&legacy, path)?;
    Ok(Some(legacy))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// The WordPress-shaped tables, which carried WordPress's `wp_` table prefix up to v0.1.
const CORE_TABLES: [&str; 12] = [
    "posts",
    "postmeta",
    "options",
    "users",
    "usermeta",
    "terms",
    "term_taxonomy",
    "term_relationships",
    "termmeta",
    "comments",
    "commentmeta",
    "application_passwords",
];

/// Current name of a v0.1 table: `wp_posts` → `posts`, `wp_neopress_forms` → `daypress_forms`.
fn current_table_name(legacy: &str) -> Option<String> {
    if let Some(rest) = legacy.strip_prefix("wp_neopress_") {
        return Some(format!("daypress_{rest}"));
    }
    let core = legacy.strip_prefix("wp_")?;
    CORE_TABLES.contains(&core).then(|| core.to_string())
}

/// Name `table` had in v0.1: `posts` → `wp_posts`, `daypress_forms` → `wp_neopress_forms`.
fn legacy_table_name(table: &str) -> String {
    match table.strip_prefix("daypress_") {
        Some(rest) => format!("wp_neopress_{rest}"),
        None => format!("wp_{table}"),
    }
}

/// Where `table` lives in the attached database `schema` (a backup snapshot): under its
/// current name or, in a Neopress backup, under its v0.1 name. `None` when it has neither.
pub fn snapshot_table(
    conn: &Connection,
    schema: &str,
    table: &str,
) -> rusqlite::Result<Option<String>> {
    for name in [table.to_string(), legacy_table_name(table)] {
        let found = conn
            .query_row(
                &format!("SELECT 1 FROM {schema}.sqlite_master WHERE type = 'table' AND name = ?1"),
                params![name],
                |_| Ok(()),
            )
            .optional()?;
        if found.is_some() {
            return Ok(Some(name));
        }
    }
    Ok(None)
}

/// Renames the tables of a v0.1 database and everything stored under the old name. Runs
/// before the schema DDL, in one transaction, and does nothing once no v0.1 table is left.
pub fn upgrade_schema(conn: &Connection) -> rusqlite::Result<()> {
    let renames: Vec<(String, String)> = names_like(conn, "table", "wp\\_%")?
        .into_iter()
        .filter_map(|old| current_table_name(&old).map(|new| (old, new)))
        .collect();
    if renames.is_empty() {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;
    for (old, new) in &renames {
        if table_exists(&tx, new)? {
            let rows: i64 =
                tx.query_row(&format!("SELECT COUNT(*) FROM \"{new}\""), [], |r| r.get(0))?;
            if rows > 0 {
                eprintln!("⚠️ Kept {old}: {new} already has data.");
                continue;
            }
            tx.execute_batch(&format!("DROP TABLE \"{new}\";"))?;
        }
        tx.execute_batch(&format!("ALTER TABLE \"{old}\" RENAME TO \"{new}\";"))?;
    }
    // The schema DDL recreates these under their new names.
    for index in names_like(&tx, "index", "idx\\_neopress\\_%")? {
        tx.execute_batch(&format!("DROP INDEX IF EXISTS \"{index}\";"))?;
    }
    upgrade_data(&tx)?;
    tx.commit()?;
    eprintln!("✓ Upgraded a Neopress v0.1 database to Daypress.");
    Ok(())
}

/// Renames the options, meta keys, plugin slugs and stored markup that still use Neopress
/// names. Rows that already use the new names stay as they are, so this also runs after
/// restoring a Neopress backup.
pub fn upgrade_data(conn: &Connection) -> rusqlite::Result<()> {
    if table_exists(conn, "options")? {
        conn.execute(
            r"UPDATE OR IGNORE options SET option_name = 'daypress_' || substr(option_name, 10)
              WHERE option_name LIKE 'neopress\_%' ESCAPE '\'",
            [],
        )?;
        conn.execute(
            r#"UPDATE options SET option_value = replace(option_value, '"neopress-', '"daypress-')
               WHERE option_name IN ('active_plugins', 'daypress_active_plugins')"#,
            [],
        )?;
        rewrite_column(
            conn,
            "options",
            "option_value",
            r"option_name LIKE 'daypress\_%' ESCAPE '\'",
        )?;
    }
    for meta in ["postmeta", "usermeta", "commentmeta", "termmeta"] {
        if !table_exists(conn, meta)? {
            continue;
        }
        conn.execute(
            &format!(
                r"UPDATE {meta} SET meta_key = '_daypress_' || substr(meta_key, 11)
                  WHERE meta_key LIKE '\_neopress\_%' ESCAPE '\'"
            ),
            [],
        )?;
        conn.execute(
            &format!(
                r"UPDATE {meta} SET meta_key = 'daypress_' || substr(meta_key, 10)
                  WHERE meta_key LIKE 'neopress\_%' ESCAPE '\'"
            ),
            [],
        )?;
    }
    if table_exists(conn, "postmeta")? {
        rewrite_column(
            conn,
            "postmeta",
            "meta_value",
            r"meta_key LIKE '\_daypress\_%' ESCAPE '\'",
        )?;
    }

    if table_exists(conn, "daypress_plugins")? {
        conn.execute(
            "UPDATE OR IGNORE daypress_plugins SET slug = 'daypress-' || substr(slug, 10)
             WHERE slug LIKE 'neopress-%'",
            [],
        )?;
    }
    for column in text_columns(conn, "posts")? {
        if column == "post_content" || column == "layout_json" {
            rewrite_column(conn, "posts", &column, "1")?;
        }
    }
    for table in ["daypress_templates", "daypress_forms"] {
        for column in text_columns(conn, table)? {
            rewrite_column(conn, table, &column, "1")?;
        }
    }
    Ok(())
}

fn names_like(conn: &Connection, kind: &str, pattern: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        r"SELECT name FROM sqlite_master WHERE type = ?1 AND name LIKE ?2 ESCAPE '\' ORDER BY name",
    )?;
    let rows = stmt.query_map(params![kind, pattern], |r| r.get(0))?;
    rows.collect()
}

fn table_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
        params![name],
        |_| Ok(()),
    )
    .optional()
    .map(|found| found.is_some())
}

/// TEXT columns of `table`, or none when the table does not exist.
fn text_columns(conn: &Connection, table: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{table}\")"))?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(1)?, r.get::<_, String>(2)?)))?;
    let mut columns = Vec::new();
    for row in rows {
        let (name, kind) = row?;
        if kind.eq_ignore_ascii_case("TEXT") {
            columns.push(name);
        }
    }
    Ok(columns)
}

fn rewrite_column(
    conn: &Connection,
    table: &str,
    column: &str,
    filter: &str,
) -> rusqlite::Result<()> {
    let rows: Vec<(i64, String)> = {
        let mut stmt = conn.prepare(&format!(
            "SELECT rowid, \"{column}\" FROM \"{table}\" WHERE \"{column}\" IS NOT NULL AND ({filter})"
        ))?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect::<rusqlite::Result<_>>()?
    };
    for (rowid, value) in rows {
        let rewritten = rewrite_stored_markup(&value);
        if rewritten != value {
            conn.execute(
                &format!("UPDATE \"{table}\" SET \"{column}\" = ?1 WHERE rowid = ?2"),
                params![rewritten, rowid],
            )?;
        }
    }
    Ok(())
}

static CSS_VAR_RE: OnceLock<Regex> = OnceLock::new();
static SELECTOR_RE: OnceLock<Regex> = OnceLock::new();
static ATTRIBUTE_RE: OnceLock<Regex> = OnceLock::new();
static ATTRIBUTE_TOKEN_RE: OnceLock<Regex> = OnceLock::new();

fn regex(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("valid legacy rewrite regex"))
}

/// Renames the class names, ids, CSS custom properties, shortcodes and REST paths that
/// Neopress generated into stored markup and CSS. Page text is left alone.
pub fn rewrite_stored_markup(text: &str) -> String {
    let text = text
        .replace("[neopress_form", "[daypress_form")
        .replace("/wp-json/neopress/", "/wp-json/daypress/")
        .replace("data-neostudio", "data-daystudio");
    // `--np-gap`, `var(--neo-ink)`
    let text = regex(&CSS_VAR_RE, r"--(np|neo)-").replace_all(&text, |c: &Captures| {
        format!("--{}-", rebrand_prefix(&c[1]))
    });
    // `.np-card.np-active`, `querySelector('#np-cookie-banner')`
    let text = regex(&SELECTOR_RE, r"([.#])(np|neo)-").replace_all(&text, |c: &Captures| {
        format!("{}{}-", &c[1], rebrand_prefix(&c[2]))
    });
    // `class="np-section neostudio-ssr-header"`, also JSON-escaped (`class=\"…\"`)
    let text = regex(&ATTRIBUTE_RE, r#"\b(class|id)=(\\?["'])([^"'\\]*)"#).replace_all(
        &text,
        |c: &Captures| {
            let value = regex(&ATTRIBUTE_TOKEN_RE, r"(^|\s)(np|neo)([-_])")
                .replace_all(&c[3], |t: &Captures| {
                    format!("{}{}{}", &t[1], rebrand_prefix(&t[2]), &t[3])
                })
                .replace("neostudio", "daystudio")
                .replace("neopress", "daypress");
            format!("{}={}{}", &c[1], &c[2], value)
        },
    );
    text.into_owned()
}

fn rebrand_prefix(prefix: &str) -> &'static str {
    if prefix == "np" {
        "dp"
    } else {
        "day"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_generated_markup_but_not_page_text() {
        let stored = r#"<section class="np-section neostudio-ssr-header" id="np-hero" style="gap:var(--np-gap)"><p>np-hard problems, Neopress</p>[neopress_form id="2"]</section><script>document.querySelector('.np-form-status.np-active');fetch('/wp-json/neopress/v1/forms/2/submit')</script>"#;
        assert_eq!(
            rewrite_stored_markup(stored),
            r#"<section class="dp-section daystudio-ssr-header" id="dp-hero" style="gap:var(--dp-gap)"><p>np-hard problems, Neopress</p>[daypress_form id="2"]</section><script>document.querySelector('.dp-form-status.dp-active');fetch('/wp-json/daypress/v1/forms/2/submit')</script>"#
        );
    }

    #[test]
    fn rewrites_json_escaped_attributes_and_css() {
        let stored = r#"{"html":"<div class=\"np-card neo-card\">x</div>","css":".np-grid{--neo-ink:#000}"}"#;
        assert_eq!(
            rewrite_stored_markup(stored),
            r#"{"html":"<div class=\"dp-card day-card\">x</div>","css":".dp-grid{--day-ink:#000}"}"#
        );
    }

    #[test]
    fn upgrades_a_v0_1_database_in_place() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE wp_options (option_id INTEGER PRIMARY KEY, option_name TEXT UNIQUE, option_value TEXT);
            CREATE TABLE wp_posts (ID INTEGER PRIMARY KEY, post_content TEXT, layout_json TEXT);
            CREATE TABLE wp_postmeta (meta_id INTEGER PRIMARY KEY, post_id INTEGER, meta_key TEXT, meta_value TEXT);
            CREATE TABLE wp_neopress_plugins (slug TEXT PRIMARY KEY, is_active INTEGER);
            CREATE TABLE wp_neopress_templates (id INTEGER PRIMARY KEY, rendered_html TEXT);
            CREATE INDEX idx_neopress_templates_status ON wp_neopress_templates(id);
            CREATE TABLE wp_links (name TEXT);
            INSERT INTO wp_links VALUES ('kept');
            INSERT INTO wp_options (option_name, option_value) VALUES
                ('neopress_auth_secret', 'secret'),
                ('neopress_elementor_kit_css', ':root{--np-primary:#f00}'),
                ('active_plugins', '["neopress-seo","elementor"]'),
                ('blogname', 'My Neopress site');
            INSERT INTO wp_posts VALUES (1, '<div class="np-hero">Neopress rocks</div>[neopress_form]', NULL);
            INSERT INTO wp_postmeta VALUES (1, 1, '_neopress_seo_title', 'Title');
            INSERT INTO wp_neopress_plugins VALUES ('neopress-seo', 0);
            INSERT INTO wp_neopress_templates VALUES (1, '<nav class="np-nav-menu"></nav>');
            "#,
        )
        .unwrap();

        upgrade_schema(&conn).unwrap();

        let value = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, String>(0)).unwrap();
        assert_eq!(
            value("SELECT option_value FROM options WHERE option_name = 'daypress_auth_secret'"),
            "secret"
        );
        assert_eq!(
            value(
                "SELECT option_value FROM options WHERE option_name = 'daypress_elementor_kit_css'"
            ),
            ":root{--dp-primary:#f00}"
        );
        assert_eq!(
            value("SELECT option_value FROM options WHERE option_name = 'active_plugins'"),
            r#"["daypress-seo","elementor"]"#
        );
        assert_eq!(
            value("SELECT option_value FROM options WHERE option_name = 'blogname'"),
            "My Neopress site"
        );
        assert_eq!(
            value("SELECT post_content FROM posts"),
            r#"<div class="dp-hero">Neopress rocks</div>[daypress_form]"#
        );
        assert_eq!(
            value("SELECT meta_key FROM postmeta"),
            "_daypress_seo_title"
        );
        assert_eq!(value("SELECT slug FROM daypress_plugins"), "daypress-seo");
        assert_eq!(
            value("SELECT rendered_html FROM daypress_templates"),
            r#"<nav class="dp-nav-menu"></nav>"#
        );
        assert_eq!(value("SELECT name FROM wp_links"), "kept");
        assert_eq!(
            names_like(&conn, "table", "wp\\_%").unwrap(),
            vec!["wp_links".to_string()]
        );
        assert!(names_like(&conn, "index", "idx\\_neopress\\_%")
            .unwrap()
            .is_empty());

        // A second run finds nothing left to do.
        upgrade_schema(&conn).unwrap();
    }

    #[test]
    fn finds_tables_in_neopress_backups() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "ATTACH DATABASE ':memory:' AS snap_db;
             CREATE TABLE snap_db.wp_posts (ID INTEGER PRIMARY KEY);
             CREATE TABLE snap_db.wp_neopress_forms (id INTEGER PRIMARY KEY);
             CREATE TABLE snap_db.options (option_id INTEGER PRIMARY KEY);",
        )
        .unwrap();
        let find = |table| snapshot_table(&conn, "snap_db", table).unwrap();
        assert_eq!(find("posts").as_deref(), Some("wp_posts"));
        assert_eq!(find("daypress_forms").as_deref(), Some("wp_neopress_forms"));
        assert_eq!(find("options").as_deref(), Some("options"));
        assert_eq!(find("comments"), None);
    }

    #[test]
    fn adopts_the_old_database_file() {
        let dir = std::env::temp_dir().join(format!("daypress-legacy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let old = dir.join("neopress.sqlite");
        let new = dir.join("daypress.sqlite");
        {
            let conn = Connection::open(&old).unwrap();
            conn.execute_batch(
                "PRAGMA journal_mode = WAL; CREATE TABLE t (x); INSERT INTO t VALUES (42);",
            )
            .unwrap();
        }

        assert_eq!(adopt_legacy_database_file(&new).unwrap(), Some(old.clone()));
        assert!(!old.exists());
        let x: i64 = Connection::open(&new)
            .unwrap()
            .query_row("SELECT x FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(x, 42);
        // Nothing to adopt once the new file exists.
        assert_eq!(adopt_legacy_database_file(&new).unwrap(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
