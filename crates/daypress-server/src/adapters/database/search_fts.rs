//! Native SQLite FTS5 Full-Text Search Engine (`adapters/database/search_fts.rs`).
//!
//! Provides BM25 ranking (`bm25(posts_fts, 10.0, 5.0, 1.0, 4.0)`), prefix matching,
//! Levenshtein typo-tolerance fallback, safe `<mark class="dp-search-hl">` snippet highlighting,
//! and automatic indexing of HTML content, Daystudio AST `layout_json`, and ACF postmeta.

use super::Database;
use crate::templates::utils::escape_html;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtsSearchResultItem {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub post_type: String,
    pub post_type_label: String,
    pub link: String,
    pub snippet_html: String,
    pub excerpt: String,
    pub date: String,
    pub reading_time_minutes: u32,
    pub score: f64,
    pub typo_matched: bool,
    #[serde(default)]
    pub lang_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtsSearchResponse {
    pub query: String,
    pub total: usize,
    pub took_us: u64,
    pub engine: String,
    pub typo_corrected: Option<String>,
    pub results: Vec<FtsSearchResultItem>,
}

/// Strips `<script>`, `<style>`, and HTML tags cleanly on UTF-8 character boundaries.
pub fn strip_html_for_fts(raw: &str) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut skip_until: Option<&'static str> = None;

    for ch in raw.chars() {
        if let Some(end_tag) = skip_until {
            tag_buf.push(ch.to_ascii_lowercase());
            if tag_buf.ends_with(end_tag) {
                skip_until = None;
                tag_buf.clear();
            } else if tag_buf.len() > 64 {
                let drain_count = tag_buf.chars().count().saturating_sub(16);
                let kept: String = tag_buf.chars().skip(drain_count).collect();
                tag_buf = kept;
            }
            continue;
        }
        match ch {
            '<' => {
                in_tag = true;
                tag_buf.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let t = tag_buf.trim().to_ascii_lowercase();
                if t.starts_with("style") && !t.ends_with('/') {
                    skip_until = Some("</style>");
                    tag_buf.clear();
                } else if t.starts_with("script") && !t.ends_with('/') {
                    skip_until = Some("</script>");
                    tag_buf.clear();
                }
                out.push(' ');
            }
            _ if in_tag => {
                if tag_buf.len() < 32 {
                    tag_buf.push(ch);
                }
            }
            _ => out.push(ch),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extracts human-readable text from a Daystudio `layout_json` AST string so Visual Builder
/// blocks (`Heading`, `Text`, `Button`, `Accordion`, `Tabs`, `CallToAction`, `PriceCard`, etc.)
/// are indexed in SQLite FTS5.
pub fn extract_ast_plain_text(layout_json: &str) -> String {
    let trimmed = layout_json.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(val) = serde_json::from_str::<Value>(trimmed) else {
        return String::new();
    };
    let mut chunks = Vec::new();
    collect_ast_text_recursive(&val, &mut chunks);
    chunks.join(" ")
}

fn collect_ast_text_recursive(val: &Value, out: &mut Vec<String>) {
    match val {
        Value::Object(map) => {
            for (k, v) in map {
                if matches!(
                    k.as_str(),
                    "text"
                        | "content"
                        | "title"
                        | "subtitle"
                        | "heading"
                        | "description"
                        | "label"
                        | "question"
                        | "answer"
                        | "quote"
                        | "author_name"
                        | "author_title"
                        | "badge_text"
                        | "button_text"
                        | "raw"
                        | "html"
                ) {
                    if let Value::String(s) = v {
                        let clean = strip_html_for_fts(s);
                        if !clean.is_empty() {
                            out.push(clean);
                        }
                    }
                } else if !matches!(
                    k.as_str(),
                    "styles"
                        | "tablet_styles"
                        | "mobile_styles"
                        | "hover_styles"
                        | "global_styles"
                        | "raw_settings"
                        | "id"
                        | "kind"
                        | "type"
                        | "tag"
                        | "url"
                        | "src"
                        | "icon"
                ) {
                    collect_ast_text_recursive(v, out);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_ast_text_recursive(item, out);
            }
        }
        _ => {}
    }
}

/// Calculates estimated reading time in minutes (at 200 words per minute, minimum 1 min).
pub fn estimate_reading_time_minutes(plain_text: &str) -> (usize, u32) {
    let words = plain_text.split_whitespace().count();
    let mins = ((words as f64) / 200.0).ceil() as u32;
    (words, mins.max(1))
}

/// Bounded Levenshtein edit distance on UTF-8 character slices for typo tolerance.
pub fn levenshtein_distance(a: &str, b: &str, max_dist: usize) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();
    if n.abs_diff(m) > max_dist {
        return max_dist + 1;
    }
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            if curr[j] < row_min {
                row_min = curr[j];
            }
        }
        if row_min > max_dist {
            return max_dist + 1;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

fn sanitize_fts_tokens(raw_query: &str) -> Vec<String> {
    raw_query
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s.chars().count() >= 2)
        .take(8)
        .collect()
}

fn highlight_terms_in_plain_text(text: &str, terms: &[String], max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lower = trimmed.to_lowercase();
    let mut first_pos = None;
    for t in terms {
        if !t.is_empty() {
            if let Some(byte_idx) = lower.find(t.as_str()) {
                let char_pos = lower[..byte_idx].chars().count();
                if first_pos.is_none_or(|cur| char_pos < cur) {
                    first_pos = Some(char_pos);
                }
            }
        }
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let total_chars = chars.len();
    let start_char = first_pos.map(|p| p.saturating_sub(45)).unwrap_or(0);
    let end_char = (start_char + max_chars).min(total_chars);
    let mut window: String = chars[start_char..end_char].iter().collect();
    if start_char > 0 {
        window = format!("…{}", window.trim_start());
    }
    if end_char < total_chars {
        window = format!("{}…", window.trim_end());
    }

    // Escape HTML first, then highlight matched words case-insensitively
    let escaped = escape_html(&window);
    let mut result = String::with_capacity(escaped.len() + 64);
    for token in escaped.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
        let word_end = token
            .char_indices()
            .find(|&(_, c)| !c.is_alphanumeric() && c != '_' && c != '-')
            .map(|(idx, _)| idx)
            .unwrap_or(token.len());
        let word = &token[..word_end];
        let trailing = &token[word_end..];
        let word_lower = word.to_lowercase();
        let matched = !word_lower.is_empty()
            && terms
                .iter()
                .any(|t| !t.is_empty() && (word_lower.starts_with(t.as_str()) || word_lower == *t));
        if matched {
            result.push_str("<mark class=\"dp-search-hl\">");
            result.push_str(word);
            result.push_str("</mark>");
            result.push_str(trailing);
        } else {
            result.push_str(token);
        }
    }
    result
}

impl Database {
    /// Synchronizes a single post row (including stripped HTML, `layout_json` AST text, and ACF meta)
    /// into the `posts_fts` virtual table.
    pub(crate) fn sync_post_fts_row(conn: &Connection, post_id: i64) {
        let row: Option<(String, String, String, String, String, String, String)> = conn
            .query_row(
                "SELECT post_title, post_excerpt, post_content, post_name, post_type, post_status, COALESCE(layout_json, '') FROM posts WHERE ID = ?1",
                params![post_id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                },
            )
            .ok();

        let _ = conn.execute("DELETE FROM posts_fts WHERE rowid = ?1", params![post_id]);

        if let Some((title, excerpt, content, slug, post_type, status, layout_json)) = row {
            if matches!(
                post_type.as_str(),
                "revision" | "nav_menu_item" | "attachment" | "elementor_library" | "elementor_kit"
            ) || post_type.starts_with("acf-")
            {
                return;
            }
            let clean_title = strip_html_for_fts(&title);
            let clean_excerpt = strip_html_for_fts(&excerpt);
            let mut clean_body = strip_html_for_fts(&content);
            let ast_text = extract_ast_plain_text(&layout_json);
            if !ast_text.is_empty() {
                if clean_body.is_empty() {
                    clean_body = ast_text;
                } else {
                    clean_body.push(' ');
                    clean_body.push_str(&ast_text);
                }
            }

            // Include non-internal postmeta values (such as ACF fields) in full-text body
            if let Ok(mut stmt) = conn.prepare_cached(
                "SELECT COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key NOT LIKE '\\_%' ESCAPE '\\' LIMIT 40",
            ) {
                if let Ok(rows) = stmt.query_map(params![post_id], |r| r.get::<_, String>(0)) {
                    for meta_val in rows.flatten() {
                        let clean_meta = strip_html_for_fts(&meta_val);
                        if !clean_meta.is_empty() && clean_meta.chars().count() <= 500 {
                            clean_body.push(' ');
                            clean_body.push_str(&clean_meta);
                        }
                    }
                }
            }

            let _ = conn.execute(
                "INSERT INTO posts_fts(rowid, post_title, post_excerpt, post_content, post_name, post_type, post_status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    post_id,
                    clean_title,
                    clean_excerpt,
                    clean_body,
                    slug,
                    post_type,
                    status
                ],
            );
        }
    }

    /// Ensures `posts_fts` is populated and enriched for all indexable posts on startup.
    pub(crate) fn ensure_fts_backfilled_with_conn(conn: &Connection) -> Result<usize> {
        let post_ids: Vec<i64> = {
            let mut stmt = conn.prepare(
                "SELECT ID FROM posts WHERE post_type NOT IN ('revision', 'nav_menu_item', 'attachment', 'elementor_library', 'elementor_kit') AND post_type NOT LIKE 'acf-%'",
            )?;
            let rows = stmt.query_map([], |r| r.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        for pid in &post_ids {
            Self::sync_post_fts_row(conn, *pid);
        }
        Ok(post_ids.len())
    }

    /// Rebuilds the entire `posts_fts` full-text index from `posts` + `layout_json` + `postmeta`.
    pub fn rebuild_fts_index(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM posts_fts;", []);
        Self::ensure_fts_backfilled_with_conn(&conn)
    }

    /// Executes a full-text search using SQLite FTS5 (`bm25` ranking + `snippet` highlighting)
    /// with automatic Levenshtein typo-tolerance fallback.
    pub fn search_fts(
        &self,
        raw_query: &str,
        post_type_filter: Option<&str>,
        limit: usize,
    ) -> Result<FtsSearchResponse> {
        let started = Instant::now();
        let trimmed_query = raw_query.trim();
        let max_results = limit.clamp(1, 100);
        let tokens = sanitize_fts_tokens(trimmed_query);

        if tokens.is_empty() {
            return Ok(FtsSearchResponse {
                query: trimmed_query.to_string(),
                total: 0,
                took_us: started.elapsed().as_micros() as u64,
                engine: "sqlite-fts5-bm25".to_string(),
                typo_corrected: None,
                results: Vec::new(),
            });
        }

        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);
        let type_filter = post_type_filter
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "any" && *s != "all");

        // Map post_type -> singular label
        let mut cpt_labels: HashMap<String, String> = HashMap::new();
        cpt_labels.insert("post".to_string(), "Article".to_string());
        cpt_labels.insert("page".to_string(), "Page".to_string());
        if let Ok(mut stmt) = conn.prepare("SELECT slug, label_singular FROM daypress_post_types") {
            if let Ok(rows) =
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            {
                for (slug, lbl) in rows.flatten() {
                    cpt_labels.insert(slug, lbl);
                }
            }
        }

        let mut seen_ids = HashSet::new();
        let mut raw_hits: Vec<(i64, f64, String, bool)> = Vec::new();

        // Pass 1: FTS5 Prefix AND query (all tokens must match as prefixes)
        let fts_and_expr = tokens
            .iter()
            .map(|t| format!("\"{}\"*", t.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" AND ");
        Self::run_fts5_query_pass(
            &conn,
            &fts_and_expr,
            type_filter,
            max_results,
            &mut seen_ids,
            &mut raw_hits,
        );

        // Pass 2: FTS5 Prefix OR query if AND returned fewer than max_results and multiple tokens exist
        if raw_hits.len() < max_results && tokens.len() > 1 {
            let fts_or_expr = tokens
                .iter()
                .map(|t| format!("\"{}\"*", t.replace('"', "")))
                .collect::<Vec<_>>()
                .join(" OR ");
            Self::run_fts5_query_pass(
                &conn,
                &fts_or_expr,
                type_filter,
                max_results,
                &mut seen_ids,
                &mut raw_hits,
            );
        }

        // Pass 3: Typo-tolerant Levenshtein + Substring fallback when FTS5 finds < max_results
        let mut typo_corrected: Option<String> = None;
        let mut highlight_terms = tokens.clone();

        if raw_hits.len() < max_results {
            let mut candidate_sql = String::from(
                "SELECT rowid, post_title, post_excerpt, post_content, post_name, post_type \
                 FROM posts_fts WHERE post_status = 'publish'",
            );
            if let Some(pt) = type_filter {
                candidate_sql.push_str(&format!(" AND post_type = '{}'", pt.replace('\'', "''")));
            }
            candidate_sql.push_str(" LIMIT 500");

            if let Ok(mut stmt) = conn.prepare(&candidate_sql) {
                if let Ok(rows) = stmt.query_map([], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                    ))
                }) {
                    let mut fuzzy_candidates: Vec<(i64, f64, String, Vec<String>)> = Vec::new();
                    for row in rows.flatten() {
                        let (pid, title, excerpt, body, slug) = row;
                        if seen_ids.contains(&pid) {
                            continue;
                        }
                        let combined = format!("{} {} {} {}", title, slug, excerpt, body);
                        let vocab: Vec<String> = combined
                            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                            .map(|w| w.to_lowercase())
                            .filter(|w| w.chars().count() >= 3)
                            .take(400)
                            .collect();

                        let title_lower = title.to_lowercase();
                        let slug_lower = slug.to_lowercase();
                        let body_lower = body.to_lowercase();

                        let mut total_fuzzy_score = 0.0;
                        let mut matched_words = Vec::new();

                        for q_tok in &tokens {
                            let q_len = q_tok.chars().count();
                            // Substring match (e.g. infix)
                            if title_lower.contains(q_tok.as_str()) {
                                total_fuzzy_score += 8.0;
                                matched_words.push(q_tok.clone());
                                continue;
                            }
                            if slug_lower.contains(q_tok.as_str()) {
                                total_fuzzy_score += 5.0;
                                matched_words.push(q_tok.clone());
                                continue;
                            }
                            if body_lower.contains(q_tok.as_str()) {
                                total_fuzzy_score += 2.5;
                                matched_words.push(q_tok.clone());
                                continue;
                            }

                            // Levenshtein typo tolerance (max edit distance 1 for 3-4 chars, 2 for 5+ chars)
                            let allowed_dist = if q_len >= 5 {
                                2
                            } else if q_len >= 3 {
                                1
                            } else {
                                0
                            };
                            if allowed_dist > 0 {
                                let mut best_dist = allowed_dist + 1;
                                let mut best_word = None;
                                for cand_w in &vocab {
                                    // Compare against full candidate word or same-length prefix of candidate word
                                    let d_full = levenshtein_distance(q_tok, cand_w, allowed_dist);
                                    let prefix_cand: String = cand_w.chars().take(q_len).collect();
                                    let d_pref = if prefix_cand.chars().count() >= 3 {
                                        levenshtein_distance(q_tok, &prefix_cand, allowed_dist)
                                    } else {
                                        allowed_dist + 1
                                    };
                                    let d = d_full.min(d_pref);
                                    if d < best_dist {
                                        best_dist = d;
                                        best_word = Some(cand_w.clone());
                                        if d == 1 {
                                            break;
                                        }
                                    }
                                }
                                if let Some(bw) = best_word {
                                    let boost = if title_lower.contains(bw.as_str()) {
                                        6.0
                                    } else {
                                        2.0
                                    };
                                    total_fuzzy_score += boost / (best_dist as f64 + 0.5);
                                    matched_words.push(bw);
                                }
                            }
                        }

                        if total_fuzzy_score > 0.0 {
                            let source_for_snippet = if !body.trim().is_empty() {
                                body
                            } else if !excerpt.trim().is_empty() {
                                excerpt
                            } else {
                                title
                            };
                            fuzzy_candidates.push((
                                pid,
                                total_fuzzy_score,
                                source_for_snippet,
                                matched_words,
                            ));
                        }
                    }

                    fuzzy_candidates
                        .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                    for (pid, fscore, source_text, matched_words) in fuzzy_candidates {
                        if raw_hits.len() >= max_results {
                            break;
                        }
                        if seen_ids.insert(pid) {
                            if typo_corrected.is_none() && !matched_words.is_empty() {
                                let corrected = matched_words.join(" ");
                                if !corrected.eq_ignore_ascii_case(trimmed_query) {
                                    typo_corrected = Some(corrected);
                                }
                            }
                            for mw in &matched_words {
                                if !highlight_terms.contains(mw) {
                                    highlight_terms.push(mw.clone());
                                }
                            }
                            let snippet =
                                highlight_terms_in_plain_text(&source_text, &matched_words, 160);
                            raw_hits.push((pid, fscore, snippet, true));
                        }
                    }
                }
            }
        }

        // Hydrate full result items
        let mut results = Vec::with_capacity(raw_hits.len());
        let mut fts_stmt = conn
            .prepare(
                r#"SELECT ID, post_date, post_date_gmt, post_modified, post_modified_gmt, post_title, post_content, post_excerpt, post_name, post_status, post_type, guid, post_author, comment_status, ping_status, COALESCE(post_parent, 0), COALESCE(menu_order, 0), COALESCE(is_sticky, 0)
                   FROM posts WHERE ID = ?1"#,
            )
            .ok();
        for (pid, score, raw_snippet, typo_matched) in raw_hits {
            let post_opt = fts_stmt.as_mut().and_then(|s| {
                s.query_map(params![pid], Self::map_post_row)
                    .ok()
                    .and_then(|mut rows| rows.next().and_then(|r| r.ok()))
            });
            if let Some(mut post) = post_opt {
                if post.status != "publish" || post.seo_noindex {
                    continue;
                }
                Self::hydrate_post_relations(&conn, &mut post, &base_url);
                let plain_body = conn
                    .query_row(
                        "SELECT post_content FROM posts_fts WHERE rowid = ?1",
                        params![pid],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| strip_html_for_fts(&post.content.rendered));
                let (_, reading_time_minutes) = estimate_reading_time_minutes(&plain_body);

                let snippet_html = if typo_matched {
                    if raw_snippet.trim().is_empty() {
                        highlight_terms_in_plain_text(&plain_body, &highlight_terms, 160)
                    } else {
                        raw_snippet
                    }
                } else {
                    let clean_snip = escape_html(&raw_snippet)
                        .replace("[[DP_HL_START]]", "<mark class=\"dp-search-hl\">")
                        .replace("[[DP_HL_END]]", "</mark>");
                    if clean_snip.trim().is_empty() {
                        highlight_terms_in_plain_text(&plain_body, &highlight_terms, 160)
                    } else {
                        clean_snip
                    }
                };

                let excerpt_plain = if !post.excerpt.rendered.trim().is_empty() {
                    strip_html_for_fts(&post.excerpt.rendered)
                } else {
                    plain_body.chars().take(150).collect::<String>()
                };

                let post_type_label =
                    cpt_labels.get(&post.post_type).cloned().unwrap_or_else(|| {
                        let mut chars = post.post_type.chars();
                        match chars.next() {
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                            None => "Post".to_string(),
                        }
                    });

                let date_short = post.date.chars().take(10).collect::<String>();

                results.push(FtsSearchResultItem {
                    id: post.id,
                    title: strip_html_for_fts(&post.title.rendered),
                    slug: post.slug,
                    post_type: post.post_type,
                    post_type_label,
                    link: post.link,
                    snippet_html,
                    excerpt: excerpt_plain,
                    date: date_short,
                    reading_time_minutes,
                    score: (score * 100.0).round() / 100.0,
                    typo_matched,
                    lang_code: post.lang_code,
                });
            }
        }

        let total = results.len();
        Ok(FtsSearchResponse {
            query: trimmed_query.to_string(),
            total,
            took_us: started.elapsed().as_micros() as u64,
            engine: "sqlite-fts5-bm25".to_string(),
            typo_corrected,
            results,
        })
    }

    fn run_fts5_query_pass(
        conn: &Connection,
        fts_expr: &str,
        type_filter: Option<&str>,
        max_results: usize,
        seen_ids: &mut HashSet<i64>,
        raw_hits: &mut Vec<(i64, f64, String, bool)>,
    ) {
        let sql = if type_filter.is_some() {
            "SELECT rowid, bm25(posts_fts, 10.0, 5.0, 1.0, 4.0) AS rank, \
             snippet(posts_fts, 2, '[[DP_HL_START]]', '[[DP_HL_END]]', '…', 24) AS snip \
             FROM posts_fts \
             WHERE posts_fts MATCH ?1 AND post_status = 'publish' AND post_type = ?2 \
             ORDER BY rank ASC LIMIT ?3"
        } else {
            "SELECT rowid, bm25(posts_fts, 10.0, 5.0, 1.0, 4.0) AS rank, \
             snippet(posts_fts, 2, '[[DP_HL_START]]', '[[DP_HL_END]]', '…', 24) AS snip \
             FROM posts_fts \
             WHERE posts_fts MATCH ?1 AND post_status = 'publish' \
             ORDER BY rank ASC LIMIT ?2"
        };

        if let Ok(mut stmt) = conn.prepare(sql) {
            let map_row = |r: &rusqlite::Row<'_>| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, f64>(1)?,
                    r.get::<_, String>(2).unwrap_or_default(),
                ))
            };
            let rows_res = if let Some(pt) = type_filter {
                stmt.query_map(params![fts_expr, pt, max_results as i64], map_row)
            } else {
                stmt.query_map(params![fts_expr, max_results as i64], map_row)
            };

            if let Ok(rows) = rows_res {
                for row in rows.flatten() {
                    if raw_hits.len() >= max_results {
                        break;
                    }
                    let (pid, bm25_rank, snip) = row;
                    if seen_ids.insert(pid) {
                        let positive_score = (-bm25_rank).max(0.01);
                        raw_hits.push((pid, positive_score, snip, false));
                    }
                }
            }
        }
    }

    /// Verifies whether an internal URL path (`href`) resolves to an existing post, page, CPT item,
    /// CPT archive, taxonomy term archive, 301/302 redirect rule, or uploaded media asset in SQLite.
    pub fn verify_internal_link(&self, href: &str) -> (bool, String) {
        let trimmed = href.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("mailto:")
            || trimmed.starts_with("tel:")
            || trimmed.starts_with("javascript:")
        {
            return (true, "anchor".to_string());
        }

        let conn = self.conn.lock().unwrap();
        let siteurl: String = conn
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'siteurl'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "http://localhost:8090".to_string());
        let site_base = siteurl.trim_end_matches('/');

        let path_part = if let Some(rest) = trimmed.strip_prefix(site_base) {
            rest
        } else if trimmed.starts_with("http://localhost:")
            || trimmed.starts_with("http://127.0.0.1:")
        {
            if let Some(idx) = trimmed[7..].find('/') {
                &trimmed[7 + idx..]
            } else {
                "/"
            }
        } else if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("//")
        {
            return (true, "external".to_string());
        } else {
            trimmed
        };

        let without_hash = path_part.split('#').next().unwrap_or("");
        let without_query = without_hash.split('?').next().unwrap_or("");
        let clean = without_query.trim_matches('/');

        if clean.is_empty()
            || matches!(
                clean,
                "sitemap.xml" | "wp-sitemap.xml" | "robots.txt" | "feed" | "rss.xml"
            )
            || clean.starts_with("static/")
            || clean.starts_with("d-admin")
            || clean.starts_with("wp-json")
            || clean.starts_with("og-image/")
        {
            return (true, "system".to_string());
        }

        // 1. Check 301/302 Redirect Manager rules
        let lookup_path = format!("/{}", clean);
        let lookup_path_slash = format!("/{}/", clean);
        let has_redirect: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM daypress_redirects WHERE source_path = ?1 OR source_path = ?2)",
                params![lookup_path, lookup_path_slash],
                |r| r.get(0),
            )
            .unwrap_or(false);
        if has_redirect {
            return (true, "redirect".to_string());
        }

        // 2. Check uploaded media assets (`uploads/...` or `wp-content/uploads/...`)
        if clean.starts_with("uploads/") || clean.starts_with("wp-content/uploads/") {
            let rel = clean
                .strip_prefix("wp-content/uploads/")
                .or_else(|| clean.strip_prefix("uploads/"))
                .unwrap_or(clean);
            let disk_path = std::path::Path::new("uploads").join(rel);
            if disk_path.exists() {
                return (true, "asset".to_string());
            }
            let like_pat = format!("%/{}", rel);
            let in_db: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM posts WHERE post_type = 'attachment' AND (guid LIKE ?1 OR post_content LIKE ?1))",
                    params![like_pat],
                    |r| r.get(0),
                )
                .unwrap_or(false);
            return (in_db, if in_db { "asset" } else { "broken" }.to_string());
        }

        let segments: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return (true, "system".to_string());
        }

        // 3. Check taxonomy archives (`/category/{slug}`, `/tag/{slug}`, `/{custom_tax}/{slug}`)
        if segments.len() >= 2 {
            let first = segments[0];
            let last = *segments.last().unwrap_or(&"");
            if matches!(first, "category" | "tag" | "author") {
                if first == "author" {
                    return (true, "taxonomy".to_string());
                }
                let tax = if first == "tag" {
                    "post_tag"
                } else {
                    "category"
                };
                let exists: bool = conn
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM terms t JOIN term_taxonomy tt ON t.term_id = tt.term_id WHERE t.slug = ?1 AND tt.taxonomy = ?2)",
                        params![last, tax],
                        |r| r.get(0),
                    )
                    .unwrap_or(false);
                if exists {
                    return (true, "taxonomy".to_string());
                }
            }

            // Custom taxonomy or Custom Post Type rewrite prefix (`/{cpt_rewrite}/{slug}`)
            let is_cpt_item: bool = conn
                .query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM posts p
                        LEFT JOIN daypress_post_types cpt ON cpt.slug = p.post_type
                        WHERE p.post_name = ?2
                          AND p.post_status != 'trash'
                          AND (p.post_type = ?1 OR cpt.rewrite_slug = ?1)
                    )",
                    params![first, last],
                    |r| r.get(0),
                )
                .unwrap_or(false);
            if is_cpt_item {
                return (true, "cpt".to_string());
            }

            let is_custom_tax: bool = conn
                .query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM terms t
                        JOIN term_taxonomy tt ON t.term_id = tt.term_id
                        LEFT JOIN daypress_taxonomies dt ON dt.slug = tt.taxonomy
                        WHERE t.slug = ?2 AND (tt.taxonomy = ?1 OR dt.rewrite_slug = ?1)
                    )",
                    params![first, last],
                    |r| r.get(0),
                )
                .unwrap_or(false);
            if is_custom_tax {
                return (true, "taxonomy".to_string());
            }
        }

        // 4. Check CPT archive root (`/{cpt_rewrite}/`)
        let first_seg = segments[0];
        let is_cpt_archive: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM daypress_post_types WHERE (slug = ?1 OR rewrite_slug = ?1) AND public = 1)",
                params![first_seg],
                |r| r.get(0),
            )
            .unwrap_or(false);
        if segments.len() == 1 && is_cpt_archive {
            return (true, "cpt".to_string());
        }

        // 5. Check direct post/page slug (`/{slug}/` or hierarchical `/parent/child/`)
        let target_slug = *segments.last().unwrap_or(&clean);
        let post_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM posts WHERE post_name = ?1 AND post_status != 'trash' AND post_type NOT IN ('revision', 'nav_menu_item'))",
                params![target_slug],
                |r| r.get(0),
            )
            .unwrap_or(false);
        if post_exists {
            return (true, "post".to_string());
        }

        (false, "broken".to_string())
    }

    /// Runs a comprehensive WCAG 2.1 AA/AAA Contrast, Broken-Link, Heading Hierarchy, and SEO audit
    /// on a post/page (or in-flight Daystudio draft payload) and persists `_daypress_audit_score` when `post_id` is given.
    pub fn audit_post_wcag_links_seo(&self, req: &AuditRequestPayload) -> AuditReportResponse {
        let mut loaded_title = req.title.clone().unwrap_or_default();
        let _loaded_slug = req.slug.clone().unwrap_or_default();
        let mut loaded_seo_title = req.seo_title.clone().unwrap_or_default();
        let mut loaded_seo_desc = req.seo_description.clone().unwrap_or_default();
        let mut loaded_focus_kw = req.focus_keyphrase.clone().unwrap_or_default();
        let mut loaded_html = req.html.clone().unwrap_or_default();
        let mut loaded_layout_json = req.layout_json.clone().unwrap_or_default();

        if let Some(pid) = req.post_id.filter(|&id| id > 0) {
            if let Ok(Some(post)) = self.get_post_by_id(pid) {
                if loaded_title.trim().is_empty() {
                    loaded_title = strip_html_for_fts(&post.title.rendered);
                }
                if loaded_seo_title.trim().is_empty() {
                    loaded_seo_title = post.seo_title.clone();
                }
                if loaded_seo_desc.trim().is_empty() {
                    loaded_seo_desc = if !post.seo_description.trim().is_empty() {
                        post.seo_description.clone()
                    } else {
                        strip_html_for_fts(&post.excerpt.rendered)
                    };
                }
                if loaded_focus_kw.trim().is_empty() {
                    loaded_focus_kw = post.seo_keyword.clone();
                }
                if loaded_html.trim().is_empty() {
                    loaded_html = post.content.rendered.clone();
                }
            }
            if loaded_layout_json.trim().is_empty() {
                let conn = self.conn.lock().unwrap();
                loaded_layout_json = conn
                    .query_row(
                        "SELECT COALESCE(layout_json, '') FROM posts WHERE ID = ?1",
                        params![pid],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap_or_default();
            }
        }

        let is_dark_theme = req
            .theme
            .as_deref()
            .is_some_and(|t| t.eq_ignore_ascii_case("dark") || t.eq_ignore_ascii_case("night"));
        let default_bg = if is_dark_theme {
            (22u8, 23u8, 26u8) // --paper dark #16171a
        } else {
            (243u8, 238u8, 228u8) // --paper light #f3eee4
        };
        let default_fg = if is_dark_theme {
            (239u8, 233u8, 222u8) // --ink dark #efe9de
        } else {
            (22u8, 23u8, 26u8) // --ink light #16171a
        };

        let mut issues: Vec<AuditIssue> = Vec::new();
        let mut verified_links: Vec<LinkVerificationResult> = Vec::new();
        let mut seen_links: HashSet<String> = HashSet::new();
        let mut seen_media: HashSet<String> = HashSet::new();
        let mut headings_seq: Vec<(u8, String, Option<String>)> = Vec::new();

        // 1. Walk DaypressLayout AST if available
        if !loaded_layout_json.trim().is_empty() {
            if let Ok(layout) =
                serde_json::from_str::<daypress_ast::DaypressLayout>(&loaded_layout_json)
            {
                audit_ast_node_recursive(
                    self,
                    &layout.root,
                    default_bg,
                    default_fg,
                    &loaded_title,
                    &mut issues,
                    &mut verified_links,
                    &mut seen_links,
                    &mut seen_media,
                    &mut headings_seq,
                );
            }
        }

        // 2. Scan HTML for links (`<a href="...">`), images (`<img ...>`), and headings (`<h1..h6>`)
        if !loaded_html.trim().is_empty() {
            if loaded_layout_json.trim().is_empty() {
                if let Ok(img_re) =
                    regex::Regex::new(r#"(?i)<img\b[^>]*\bsrc\s*=\s*["']([^"']+)["']"#)
                {
                    for caps in img_re.captures_iter(&loaded_html) {
                        if let Some(src_m) = caps.get(1) {
                            audit_media_url(
                                self,
                                src_m.as_str(),
                                None,
                                "HTML <img>",
                                &mut issues,
                                &mut seen_media,
                            );
                        }
                    }
                }
            }
            if let Ok(a_re) = regex::Regex::new(r#"(?i)<a\b[^>]*\bhref\s*=\s*["']([^"']+)["']"#) {
                for caps in a_re.captures_iter(&loaded_html) {
                    if let Some(href_m) = caps.get(1) {
                        let href = href_m.as_str().trim();
                        if !href.is_empty() && seen_links.insert(href.to_string()) {
                            let (valid, target_type) = self.verify_internal_link(href);
                            verified_links.push(LinkVerificationResult {
                                href: href.to_string(),
                                valid,
                                target_type,
                                node_id: None,
                            });
                            if !valid {
                                issues.push(AuditIssue {
                                    id: format!("broken-link-{}", verified_links.len()),
                                    category: "links".to_string(),
                                    severity: "error".to_string(),
                                    node_id: None,
                                    title: format!("Broken Internal Link: {}", href),
                                    detail: format!(
                                        "Target '{}' does not match any published post, page, CPT, taxonomy term, redirect rule, or media asset in SQLite.",
                                        href
                                    ),
                                    auto_fix_kind: Some("broken_link".to_string()),
                                    auto_fix_value: Some("/".to_string()),
                                });
                            }
                        }
                    }
                }
            }

            if headings_seq.is_empty() {
                if let Ok(h_re) = regex::Regex::new(r#"(?is)<h([1-6])(?:\s[^>]*)?>(.*?)</h[1-6]>"#)
                {
                    for caps in h_re.captures_iter(&loaded_html) {
                        let lvl: u8 = caps
                            .get(1)
                            .and_then(|m| m.as_str().parse().ok())
                            .unwrap_or(2);
                        let txt = strip_html_for_fts(caps.get(2).map(|m| m.as_str()).unwrap_or(""));
                        if !txt.is_empty() {
                            headings_seq.push((lvl, txt, None));
                        }
                    }
                }
            }
        }

        // 3. Also verify any explicit links array passed from the Daystudio client canvas
        for raw_href in &req.links {
            let href = raw_href.trim();
            if !href.is_empty() && seen_links.insert(href.to_string()) {
                let (valid, target_type) = self.verify_internal_link(href);
                verified_links.push(LinkVerificationResult {
                    href: href.to_string(),
                    valid,
                    target_type,
                    node_id: None,
                });
                if !valid {
                    issues.push(AuditIssue {
                        id: format!("broken-link-{}", verified_links.len()),
                        category: "links".to_string(),
                        severity: "error".to_string(),
                        node_id: None,
                        title: format!("Broken Internal Link: {}", href),
                        detail: format!(
                            "Target '{}' does not match any published post, page, CPT, taxonomy term, redirect rule, or media asset in SQLite.",
                            href
                        ),
                        auto_fix_kind: Some("broken_link".to_string()),
                        auto_fix_value: Some("/".to_string()),
                    });
                }
            }
        }

        // 4. Audit Heading Hierarchy (WCAG 1.3.1 / 2.4.6)
        let h1_count = headings_seq.iter().filter(|(l, _, _)| *l == 1).count();
        if h1_count > 1 {
            let second_h1_node = headings_seq
                .iter()
                .filter(|(l, _, _)| *l == 1)
                .nth(1)
                .and_then(|(_, _, nid)| nid.clone());
            issues.push(AuditIssue {
                id: "wcag-multiple-h1".to_string(),
                category: "wcag".to_string(),
                severity: "warning".to_string(),
                node_id: second_h1_node,
                title: format!("Multiple H1 Headings ({} found)", h1_count),
                detail: "Pages should have a single primary H1 heading for screen-reader document outline clarity and SEO.".to_string(),
                auto_fix_kind: Some("heading_level".to_string()),
                auto_fix_value: Some("2".to_string()),
            });
        }
        for window in headings_seq.windows(2) {
            let (prev_lvl, _, _) = &window[0];
            let (curr_lvl, curr_txt, curr_nid) = &window[1];
            if *curr_lvl > prev_lvl + 1 {
                let target_lvl = prev_lvl + 1;
                issues.push(AuditIssue {
                    id: format!(
                        "wcag-skipped-heading-{}",
                        curr_nid.as_deref().unwrap_or("html")
                    ),
                    category: "wcag".to_string(),
                    severity: "warning".to_string(),
                    node_id: curr_nid.clone(),
                    title: format!("Skipped Heading Level (H{} → H{})", prev_lvl, curr_lvl),
                    detail: format!(
                        "Heading \"{}\" jumps from H{} to H{} (WCAG 1.3.1 Info & Relationships). Change to H{}.",
                        curr_txt.chars().take(48).collect::<String>(),
                        prev_lvl,
                        curr_lvl,
                        target_lvl
                    ),
                    auto_fix_kind: Some("heading_level".to_string()),
                    auto_fix_value: Some(target_lvl.to_string()),
                });
            }
        }

        // 5. Audit SEO Metadata
        let effective_title = if !loaded_seo_title.trim().is_empty() {
            loaded_seo_title.trim()
        } else {
            loaded_title.trim()
        };
        let title_len = effective_title.chars().count();
        if title_len == 0 {
            issues.push(AuditIssue {
                id: "seo-missing-title".to_string(),
                category: "seo".to_string(),
                severity: "error".to_string(),
                node_id: None,
                title: "Missing Page/Post Title".to_string(),
                detail: "Every page requires a descriptive title (ideal length: 30–60 characters)."
                    .to_string(),
                auto_fix_kind: None,
                auto_fix_value: None,
            });
        } else if title_len < 15 {
            issues.push(AuditIssue {
                id: "seo-short-title".to_string(),
                category: "seo".to_string(),
                severity: "warning".to_string(),
                node_id: None,
                title: format!("Title is Too Short ({} chars)", title_len),
                detail:
                    "Aim for 30–60 characters so search engines have sufficient keyword context."
                        .to_string(),
                auto_fix_kind: None,
                auto_fix_value: None,
            });
        } else if title_len > 68 {
            issues.push(AuditIssue {
                id: "seo-long-title".to_string(),
                category: "seo".to_string(),
                severity: "warning".to_string(),
                node_id: None,
                title: format!("Title May Truncate in SERPs ({} chars > 60)", title_len),
                detail: "Google typically truncates titles longer than ~60 characters.".to_string(),
                auto_fix_kind: None,
                auto_fix_value: None,
            });
        }

        let plain_all = if !loaded_layout_json.trim().is_empty() {
            let ast_p = extract_ast_plain_text(&loaded_layout_json);
            if ast_p.is_empty() {
                strip_html_for_fts(&loaded_html)
            } else {
                format!("{} {}", strip_html_for_fts(&loaded_html), ast_p)
            }
        } else {
            strip_html_for_fts(&loaded_html)
        };

        let desc_len = loaded_seo_desc.trim().chars().count();
        if desc_len == 0 {
            let suggested_desc: String = plain_all.chars().take(152).collect();
            issues.push(AuditIssue {
                id: "seo-missing-meta-desc".to_string(),
                category: "seo".to_string(),
                severity: "warning".to_string(),
                node_id: None,
                title: "Missing Meta Description".to_string(),
                detail: "Add a 120–158 character SEO meta description to maximize organic click-through rate.".to_string(),
                auto_fix_kind: if !suggested_desc.trim().is_empty() {
                    Some("seo_description".to_string())
                } else {
                    None
                },
                auto_fix_value: if !suggested_desc.trim().is_empty() {
                    Some(suggested_desc)
                } else {
                    None
                },
            });
        } else if desc_len < 50 {
            issues.push(AuditIssue {
                id: "seo-short-meta-desc".to_string(),
                category: "seo".to_string(),
                severity: "warning".to_string(),
                node_id: None,
                title: format!("Meta Description is Short ({} chars)", desc_len),
                detail: "Aim for 120–158 characters for optimal search snippet coverage."
                    .to_string(),
                auto_fix_kind: None,
                auto_fix_value: None,
            });
        }

        let (word_count, reading_time_minutes) = estimate_reading_time_minutes(&plain_all);
        if !loaded_focus_kw.trim().is_empty() {
            let kw_lower = loaded_focus_kw.trim().to_lowercase();
            if !effective_title.to_lowercase().contains(&kw_lower) {
                issues.push(AuditIssue {
                    id: "seo-kw-missing-in-title".to_string(),
                    category: "seo".to_string(),
                    severity: "warning".to_string(),
                    node_id: None,
                    title: "Focus Keyphrase Not Found in Title".to_string(),
                    detail: format!(
                        "Include the focus keyphrase \"{}\" in your title.",
                        loaded_focus_kw.trim()
                    ),
                    auto_fix_kind: None,
                    auto_fix_value: None,
                });
            }
        }

        // Compute category scores (0..=100)
        let wcag_deductions: i32 = issues
            .iter()
            .filter(|i| i.category == "wcag")
            .map(|i| if i.severity == "error" { 15 } else { 8 })
            .sum();
        let links_deductions: i32 = issues
            .iter()
            .filter(|i| i.category == "links")
            .map(|i| if i.severity == "error" { 20 } else { 10 })
            .sum();
        let seo_deductions: i32 = issues
            .iter()
            .filter(|i| i.category == "seo")
            .map(|i| if i.severity == "error" { 18 } else { 10 })
            .sum();

        let wcag_score = (100 - wcag_deductions).clamp(0, 100) as u8;
        let links_score = (100 - links_deductions).clamp(0, 100) as u8;
        let seo_score = (100 - seo_deductions).clamp(0, 100) as u8;
        let overall_score =
            ((wcag_score as u16 * 40 + links_score as u16 * 30 + seo_score as u16 * 30) / 100)
                as u8;

        // Persist `_daypress_audit_score` to `postmeta` when `post_id` is provided
        if let Some(pid) = req.post_id.filter(|&id| id > 0) {
            let conn = self.conn.lock().unwrap();
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_audit_score')",
                    params![pid],
                    |r| r.get(0),
                )
                .unwrap_or(false);
            let score_str = overall_score.to_string();
            if exists {
                let _ = conn.execute(
                    "UPDATE postmeta SET meta_value = ?1 WHERE post_id = ?2 AND meta_key = '_daypress_audit_score'",
                    params![score_str, pid],
                );
            } else {
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_audit_score', ?2)",
                    params![pid, score_str],
                );
            }
        }

        AuditReportResponse {
            score: overall_score,
            wcag_score,
            links_score,
            seo_score,
            word_count,
            reading_time_minutes: reading_time_minutes as usize,
            issues,
            verified_links,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditRequestPayload {
    pub post_id: Option<i64>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub focus_keyphrase: Option<String>,
    pub html: Option<String>,
    pub layout_json: Option<String>,
    pub theme: Option<String>,
    #[serde(default)]
    pub links: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditIssue {
    pub id: String,
    pub category: String,
    pub severity: String,
    pub node_id: Option<String>,
    pub title: String,
    pub detail: String,
    pub auto_fix_kind: Option<String>,
    pub auto_fix_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkVerificationResult {
    pub href: String,
    pub valid: bool,
    pub target_type: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReportResponse {
    pub score: u8,
    pub wcag_score: u8,
    pub links_score: u8,
    pub seo_score: u8,
    pub word_count: usize,
    pub reading_time_minutes: usize,
    pub issues: Vec<AuditIssue>,
    pub verified_links: Vec<LinkVerificationResult>,
}

/// Parses a CSS color string (`#rgb`, `#rrggbb`, `rgb(r,g,b)`, or Daypress CSS variables) into `(r, g, b)`.
pub fn parse_css_rgb(raw: &str) -> Option<(u8, u8, u8)> {
    let s = raw.trim().to_ascii_lowercase();
    if s.is_empty() || s == "transparent" || s == "inherit" || s == "currentcolor" {
        return None;
    }
    if s.contains("--paper-raised") {
        return Some((251, 248, 242));
    }
    if s.contains("--paper-sunk") {
        return Some((231, 224, 209));
    }
    if s.contains("--paper") {
        return Some((243, 238, 228));
    }
    if s.contains("--ink-muted") {
        return Some((90, 88, 82));
    }
    if s.contains("--ink") {
        return Some((22, 23, 26));
    }
    if s.contains("--oxide") {
        return Some((168, 64, 27));
    }
    if s.contains("--press") {
        return Some((42, 59, 208));
    }
    if s.contains("--moss") {
        return Some((61, 90, 57));
    }

    if let Some(hex) = s.strip_prefix('#') {
        let hex_clean = hex.trim();
        if hex_clean.len() == 3 {
            let r = u8::from_str_radix(&hex_clean[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex_clean[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex_clean[2..3].repeat(2), 16).ok()?;
            return Some((r, g, b));
        } else if hex_clean.len() >= 6 {
            let r = u8::from_str_radix(&hex_clean[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex_clean[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex_clean[4..6], 16).ok()?;
            return Some((r, g, b));
        }
    }

    if let Some(inner) = s
        .strip_prefix("rgba(")
        .or_else(|| s.strip_prefix("rgb("))
        .and_then(|x| x.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r = parts[0].trim().parse::<f32>().ok()? as u8;
            let g = parts[1].trim().parse::<f32>().ok()? as u8;
            let b = parts[2].trim().parse::<f32>().ok()? as u8;
            return Some((r, g, b));
        }
    }
    None
}

fn srgb_channel_luminance(c: u8) -> f64 {
    let v = (c as f64) / 255.0;
    if v <= 0.03928 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(rgb: (u8, u8, u8)) -> f64 {
    0.2126 * srgb_channel_luminance(rgb.0)
        + 0.7152 * srgb_channel_luminance(rgb.1)
        + 0.0722 * srgb_channel_luminance(rgb.2)
}

/// Calculates the WCAG 2.1 contrast ratio `(L1 + 0.05) / (L2 + 0.05)` in range `1.0..=21.0`.
pub fn wcag_contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f64 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (bright, dark) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    ((bright + 0.05) / (dark + 0.05) * 100.0).round() / 100.0
}

fn audit_media_url(
    db: &Database,
    raw_url: &str,
    node_id: Option<&str>,
    label: &str,
    issues: &mut Vec<AuditIssue>,
    seen_media: &mut HashSet<String>,
) {
    let trimmed = raw_url
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.contains("{{")
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("about:blank")
    {
        return;
    }

    let dedup_key = format!("{}|{}", node_id.unwrap_or(""), trimmed);
    if !seen_media.insert(dedup_key) {
        return;
    }
    let issue_suffix = format!("{}-{}", node_id.unwrap_or("html"), seen_media.len());

    if trimmed
        .get(..11)
        .is_some_and(|p| p.eq_ignore_ascii_case("data:image/"))
    {
        issues.push(AuditIssue {
            id: format!("media-inline-data-{}", issue_suffix),
            category: "links".to_string(),
            severity: "warning".to_string(),
            node_id: node_id.map(str::to_string),
            title: "Inline data:image/ URI (Not in Media Library)".to_string(),
            detail: format!(
                "{} embeds an inline data:image/ URI instead of a Media Library asset. Upload it via daypress_import_wp_bundle (bundle.media) and reference /uploads/<file_name>.",
                label
            ),
            auto_fix_kind: None,
            auto_fix_value: None,
        });
        return;
    }

    let conn = db.conn.lock().unwrap();
    let siteurl: String = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = 'siteurl'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "http://localhost:8090".to_string());
    let site_base = siteurl.trim_end_matches('/');

    let path_part = if let Some(rest) = trimmed.strip_prefix(site_base) {
        rest
    } else if let Some(after_scheme) = trimmed
        .strip_prefix("http://localhost:")
        .or_else(|| trimmed.strip_prefix("http://127.0.0.1:"))
    {
        if let Some(idx) = after_scheme.find('/') {
            &after_scheme[idx..]
        } else {
            "/"
        }
    } else if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("//")
    {
        let preview: String = trimmed.chars().take(72).collect();
        issues.push(AuditIssue {
            id: format!("media-external-hotlink-{}", issue_suffix),
            category: "links".to_string(),
            severity: "warning".to_string(),
            node_id: node_id.map(str::to_string),
            title: "External Hotlinked Image (Not in Media Library)".to_string(),
            detail: format!(
                "{} hotlinks '{}'. Import all page and template images into the Media Library (/uploads/...) via daypress_import_wp_bundle so Daypress serves local WebP/AVIF variants and backs them up.",
                label, preview
            ),
            auto_fix_kind: None,
            auto_fix_value: None,
        });
        return;
    } else {
        trimmed
    };

    let without_hash = path_part.split('#').next().unwrap_or("");
    let without_query = without_hash.split('?').next().unwrap_or("");
    let clean = without_query.trim_matches('/');
    if clean.starts_with("uploads/") || clean.starts_with("wp-content/uploads/") {
        let rel = clean
            .strip_prefix("wp-content/uploads/")
            .or_else(|| clean.strip_prefix("uploads/"))
            .unwrap_or(clean);
        if rel.is_empty() {
            return;
        }
        let like_pat = format!("%/{}", rel);
        let in_media_library: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM posts p
                    WHERE p.post_type = 'attachment'
                      AND (
                          p.post_name = ?1
                          OR p.guid = ?2
                          OR p.guid LIKE ?3
                          OR EXISTS(
                              SELECT 1 FROM postmeta pm
                              WHERE pm.post_id = p.ID
                                AND pm.meta_key = '_wp_attached_file'
                                AND pm.meta_value = ?1
                          )
                      )
                )",
                params![rel, format!("/uploads/{}", rel), like_pat],
                |r| r.get(0),
            )
            .unwrap_or(false);
        if !in_media_library {
            let preview: String = rel.chars().take(56).collect();
            issues.push(AuditIssue {
                id: format!("media-unregistered-upload-{}", issue_suffix),
                category: "links".to_string(),
                severity: "warning".to_string(),
                node_id: node_id.map(str::to_string),
                title: format!("Unregistered Media Asset: /uploads/{}", preview),
                detail: format!(
                    "{} references '/uploads/{}', which has no attachment record in the Media Library. Import it via daypress_import_wp_bundle (bundle.media) so it appears in /d-admin/upload and /wp-json/wp/v2/media.",
                    label, preview
                ),
                auto_fix_kind: None,
                auto_fix_value: None,
            });
        }
    }
}

fn audit_style_media(
    db: &Database,
    styles: &daypress_ast::StyleProperties,
    node_id: &str,
    issues: &mut Vec<AuditIssue>,
    seen_media: &mut HashSet<String>,
) {
    if let Some(bg_url) = styles
        .background
        .as_ref()
        .and_then(|b| b.image_url.as_deref())
    {
        audit_media_url(
            db,
            bg_url,
            Some(node_id),
            "Background image",
            issues,
            seen_media,
        );
    }
    if let Some(bg_img) = styles.background_image.as_deref() {
        let trimmed = bg_img.trim();
        if let Ok(url_re) = regex::Regex::new(r#"(?i)url\(\s*["']?([^)"']+)["']?\s*\)"#) {
            let mut matched_any = false;
            for caps in url_re.captures_iter(trimmed) {
                if let Some(m) = caps.get(1) {
                    matched_any = true;
                    audit_media_url(
                        db,
                        m.as_str(),
                        Some(node_id),
                        "CSS background_image",
                        issues,
                        seen_media,
                    );
                }
            }
            let low = trimmed.to_ascii_lowercase();
            if !matched_any
                && !low.is_empty()
                && !low.starts_with("none")
                && !low.starts_with("linear-")
                && !low.starts_with("radial-")
                && !low.starts_with("conic-")
            {
                audit_media_url(
                    db,
                    trimmed,
                    Some(node_id),
                    "CSS background_image",
                    issues,
                    seen_media,
                );
            }
        }
    }
    for layer in [
        styles.tablet.as_deref(),
        styles.mobile.as_deref(),
        styles.hover.as_deref(),
        styles.focus.as_deref(),
        styles.active.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        audit_style_media(db, layer, node_id, issues, seen_media);
    }
}

#[allow(clippy::too_many_arguments)]
fn audit_ast_node_recursive(
    db: &Database,
    node: &daypress_ast::DaypressNode,
    inherited_bg: (u8, u8, u8),
    inherited_fg: (u8, u8, u8),
    fallback_title: &str,
    issues: &mut Vec<AuditIssue>,
    verified_links: &mut Vec<LinkVerificationResult>,
    seen_links: &mut HashSet<String>,
    seen_media: &mut HashSet<String>,
    headings_seq: &mut Vec<(u8, String, Option<String>)>,
) {
    audit_style_media(db, &node.styles, &node.id, issues, seen_media);
    let node_bg = node
        .styles
        .background
        .as_ref()
        .and_then(|b| b.color.as_deref())
        .and_then(parse_css_rgb)
        .or_else(|| {
            node.styles
                .raw_settings
                .as_ref()
                .and_then(|m| {
                    m.get("background_color")
                        .or_else(|| m.get("backgroundColor"))
                })
                .and_then(|v| v.as_str())
                .and_then(parse_css_rgb)
        })
        .unwrap_or(inherited_bg);

    let explicit_fg = node
        .styles
        .typography
        .as_ref()
        .and_then(|t| t.color.as_deref())
        .and_then(parse_css_rgb)
        .or_else(|| {
            node.styles
                .raw_settings
                .as_ref()
                .and_then(|m| m.get("text_color").or_else(|| m.get("color")))
                .and_then(|v| v.as_str())
                .and_then(parse_css_rgb)
        });
    let node_fg = explicit_fg.unwrap_or(inherited_fg);

    match &node.kind {
        daypress_ast::NodeKind::Heading { text, level } => {
            let plain = strip_html_for_fts(text);
            if !plain.is_empty() {
                headings_seq.push((*level, plain.clone(), Some(node.id.clone())));
            }
            if explicit_fg.is_some() || node_bg != inherited_bg {
                let ratio = wcag_contrast_ratio(node_fg, node_bg);
                let min_aa = if *level <= 2 { 3.0 } else { 4.5 };
                if ratio < min_aa {
                    let fix_color = if relative_luminance(node_bg) > 0.35 {
                        "#16171a"
                    } else {
                        "#efe9de"
                    };
                    issues.push(AuditIssue {
                        id: format!("wcag-contrast-{}", node.id),
                        category: "wcag".to_string(),
                        severity: "error".to_string(),
                        node_id: Some(node.id.clone()),
                        title: format!(
                            "Low Heading Contrast ({:.2}:1 < {:.1}:1 WCAG AA)",
                            ratio, min_aa
                        ),
                        detail: format!(
                            "Heading \"{}\" has insufficient contrast against its background.",
                            plain.chars().take(42).collect::<String>()
                        ),
                        auto_fix_kind: Some("contrast_text".to_string()),
                        auto_fix_value: Some(fix_color.to_string()),
                    });
                }
            }
        }
        daypress_ast::NodeKind::Text { content_html } => {
            if let Ok(img_re) = regex::Regex::new(r#"(?i)<img\b[^>]*\bsrc\s*=\s*["']([^"']+)["']"#)
            {
                for caps in img_re.captures_iter(content_html) {
                    if let Some(src_m) = caps.get(1) {
                        audit_media_url(
                            db,
                            src_m.as_str(),
                            Some(&node.id),
                            "Text <img>",
                            issues,
                            seen_media,
                        );
                    }
                }
            }
            let plain = strip_html_for_fts(content_html);
            if !plain.is_empty() && (explicit_fg.is_some() || node_bg != inherited_bg) {
                let ratio = wcag_contrast_ratio(node_fg, node_bg);
                if ratio < 4.5 {
                    let fix_color = if relative_luminance(node_bg) > 0.35 {
                        "#16171a"
                    } else {
                        "#efe9de"
                    };
                    issues.push(AuditIssue {
                        id: format!("wcag-contrast-{}", node.id),
                        category: "wcag".to_string(),
                        severity: "error".to_string(),
                        node_id: Some(node.id.clone()),
                        title: format!("Low Text Contrast ({:.2}:1 < 4.5:1 WCAG AA)", ratio),
                        detail: format!(
                            "Text block \"{}\" fails WCAG 2.1 AA minimum 4.5:1 contrast ratio.",
                            plain.chars().take(42).collect::<String>()
                        ),
                        auto_fix_kind: Some("contrast_text".to_string()),
                        auto_fix_value: Some(fix_color.to_string()),
                    });
                }
            }
        }
        daypress_ast::NodeKind::Button { text, url, .. } => {
            let plain = strip_html_for_fts(text);
            if !plain.is_empty() && explicit_fg.is_some() {
                let ratio = wcag_contrast_ratio(node_fg, node_bg);
                if ratio < 4.5 {
                    let fix_color = if relative_luminance(node_bg) > 0.35 {
                        "#16171a"
                    } else {
                        "#fff8f0"
                    };
                    issues.push(AuditIssue {
                        id: format!("wcag-contrast-{}", node.id),
                        category: "wcag".to_string(),
                        severity: "error".to_string(),
                        node_id: Some(node.id.clone()),
                        title: format!("Low Button Contrast ({:.2}:1 < 4.5:1 WCAG AA)", ratio),
                        detail: format!(
                            "Button \"{}\" fails WCAG 2.1 AA contrast.",
                            plain.chars().take(36).collect::<String>()
                        ),
                        auto_fix_kind: Some("contrast_text".to_string()),
                        auto_fix_value: Some(fix_color.to_string()),
                    });
                }
            }
            let href = url.trim();
            if !href.is_empty() && seen_links.insert(href.to_string()) {
                let (valid, target_type) = db.verify_internal_link(href);
                verified_links.push(LinkVerificationResult {
                    href: href.to_string(),
                    valid,
                    target_type,
                    node_id: Some(node.id.clone()),
                });
                if !valid {
                    issues.push(AuditIssue {
                        id: format!("broken-link-{}", node.id),
                        category: "links".to_string(),
                        severity: "error".to_string(),
                        node_id: Some(node.id.clone()),
                        title: format!("Broken Button Link: {}", href),
                        detail: format!(
                            "Button \"{}\" links to '{}', which does not exist in SQLite.",
                            plain, href
                        ),
                        auto_fix_kind: Some("broken_link".to_string()),
                        auto_fix_value: Some("/".to_string()),
                    });
                }
            }
        }
        daypress_ast::NodeKind::Image { src, alt, link, .. } => {
            audit_media_url(db, src, Some(&node.id), "Image", issues, seen_media);
            if alt.trim().is_empty() {
                let inferred_alt = src
                    .rsplit('/')
                    .next()
                    .and_then(|f| f.split('.').next())
                    .map(|s| s.replace(['-', '_'], " "))
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| {
                        if !fallback_title.trim().is_empty() {
                            fallback_title.trim().to_string()
                        } else {
                            "Editorial illustration".to_string()
                        }
                    });
                issues.push(AuditIssue {
                    id: format!("wcag-img-alt-{}", node.id),
                    category: "wcag".to_string(),
                    severity: "error".to_string(),
                    node_id: Some(node.id.clone()),
                    title: "Missing Image Alt Text (WCAG 1.1.1)".to_string(),
                    detail: format!(
                        "Image '{}' has no alt attribute for screen readers.",
                        src.chars().take(48).collect::<String>()
                    ),
                    auto_fix_kind: Some("image_alt".to_string()),
                    auto_fix_value: Some(inferred_alt),
                });
            }
            if let Some(href) = link.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                if seen_links.insert(href.to_string()) {
                    let (valid, target_type) = db.verify_internal_link(href);
                    verified_links.push(LinkVerificationResult {
                        href: href.to_string(),
                        valid,
                        target_type,
                        node_id: Some(node.id.clone()),
                    });
                    if !valid {
                        issues.push(AuditIssue {
                            id: format!("broken-link-{}", node.id),
                            category: "links".to_string(),
                            severity: "error".to_string(),
                            node_id: Some(node.id.clone()),
                            title: format!("Broken Image Link: {}", href),
                            detail: format!("Image link '{}' does not resolve in SQLite.", href),
                            auto_fix_kind: Some("broken_link".to_string()),
                            auto_fix_value: Some("/".to_string()),
                        });
                    }
                }
            }
        }
        daypress_ast::NodeKind::ImageBox { image_url, .. } => {
            audit_media_url(
                db,
                image_url,
                Some(&node.id),
                "ImageBox",
                issues,
                seen_media,
            );
        }
        daypress_ast::NodeKind::Testimonial {
            image_url: Some(image_url),
            ..
        } => {
            audit_media_url(
                db,
                image_url,
                Some(&node.id),
                "Testimonial avatar",
                issues,
                seen_media,
            );
        }
        daypress_ast::NodeKind::TestimonialCarousel { slides, .. } => {
            for slide in slides {
                if let Some(ref avatar) = slide.avatar_url {
                    audit_media_url(
                        db,
                        avatar,
                        Some(&node.id),
                        "TestimonialCarousel slide",
                        issues,
                        seen_media,
                    );
                }
            }
        }
        daypress_ast::NodeKind::CallToAction {
            image_url: Some(image_url),
            ..
        } => {
            audit_media_url(
                db,
                image_url,
                Some(&node.id),
                "CallToAction image",
                issues,
                seen_media,
            );
        }
        daypress_ast::NodeKind::FlipBox {
            front_image: Some(front_image),
            ..
        } => {
            audit_media_url(
                db,
                front_image,
                Some(&node.id),
                "FlipBox image",
                issues,
                seen_media,
            );
        }
        _ => {}
    }

    for child in &node.children {
        audit_ast_node_recursive(
            db,
            child,
            node_bg,
            node_fg,
            fallback_title,
            issues,
            verified_links,
            seen_links,
            seen_media,
            headings_seq,
        );
    }
}

#[cfg(test)]
mod media_audit_tests {
    use super::*;

    // Minimal valid 1x1 transparent PNG
    const TINY_PNG_B64: &str =
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    #[test]
    fn audit_flags_hotlinks_data_uris_and_unregistered_uploads_until_imported() {
        let db = Database::new_in_memory().expect("in-memory DB");
        let unimported_layout = serde_json::json!({
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": [
                    {
                        "id": "h1",
                        "type": "Heading",
                        "data": { "text": "Artisanal Metal Finishes for Architecture", "level": 1 }
                    },
                    {
                        "id": "img_ext",
                        "type": "Image",
                        "data": { "src": "https://cdn.example.com/atelier.jpg", "alt": "Atelier hero" }
                    },
                    {
                        "id": "img_data",
                        "type": "Image",
                        "data": { "src": format!("data:image/png;base64,{}", TINY_PNG_B64), "alt": "Brand logo" }
                    },
                    {
                        "id": "img_ghost",
                        "type": "Image",
                        "data": { "src": "/uploads/not-in-library.png", "alt": "Ghost local image" }
                    }
                ]
            }
        })
        .to_string();

        let report = db.audit_post_wcag_links_seo(&AuditRequestPayload {
            title: Some("Artisanal Metal Finishes for Architecture".to_string()),
            seo_description: Some(
                "Bespoke liquid metal finishes crafted by hand in our workshop for interior architects and luxury residences."
                    .to_string(),
            ),
            layout_json: Some(unimported_layout),
            theme: Some("light".to_string()),
            ..Default::default()
        });

        assert!(
            report
                .issues
                .iter()
                .any(|i| i.node_id.as_deref() == Some("img_ext")
                    && i.id.starts_with("media-external-hotlink-")),
            "expected hotlink warning on img_ext, got: {:?}",
            report.issues
        );
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.node_id.as_deref() == Some("img_data")
                    && i.id.starts_with("media-inline-data-")),
            "expected inline data:image/ warning on img_data, got: {:?}",
            report.issues
        );
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.node_id.as_deref() == Some("img_ghost")
                    && i.id.starts_with("media-unregistered-upload-")),
            "expected unregistered /uploads/ warning on img_ghost, got: {:?}",
            report.issues
        );

        // Now import the PNG into the Media Library via `import_bundle` (`bundle.media` only)
        let unique_name = format!(
            "unit-test-audit-{}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(1)
        );
        let bundle_req = crate::models::ImportBundleRequest {
            media: vec![crate::models::ImportMedia {
                id: Some(101),
                title: "Imported Brand Logo".to_string(),
                file_name: unique_name.clone(),
                mime_type: "image/png".to_string(),
                source_url: "https://cdn.example.com/logo.png".to_string(),
                original_url: None,
                alt_text: Some("Brand logo".to_string()),
                file_size: None,
                base64_data: Some(TINY_PNG_B64.to_string()),
            }],
            ..Default::default()
        };
        let imported = db.import_bundle(&bundle_req).expect("import media bundle");
        assert_eq!(imported.migrated_media, 1);

        // Verify `_wp_attachment_metadata` and `_filesize` were written automatically
        {
            let conn = db.conn.lock().unwrap();
            let meta_val: String = conn
                .query_row(
                    "SELECT pm.meta_value FROM postmeta pm
                     JOIN posts p ON p.ID = pm.post_id
                     WHERE p.post_type = 'attachment' AND p.post_name = ?1
                       AND pm.meta_key = '_wp_attachment_metadata'",
                    params![unique_name],
                    |r| r.get(0),
                )
                .expect("_wp_attachment_metadata should be persisted by import_bundle");
            assert!(meta_val.contains("\"width\":1"), "meta_val was: {meta_val}");
            assert!(
                meta_val.contains("\"height\":1"),
                "meta_val was: {meta_val}"
            );
        }
        let _ = std::fs::remove_file(std::path::Path::new("uploads").join(&unique_name));

        // Re-audit with the registered `/uploads/<unique_name>` asset
        let clean_layout = serde_json::json!({
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": [
                    {
                        "id": "h1",
                        "type": "Heading",
                        "data": { "text": "Artisanal Metal Finishes for Architecture", "level": 1 }
                    },
                    {
                        "id": "img_imported",
                        "type": "Image",
                        "data": { "src": format!("/uploads/{}", unique_name), "alt": "Brand logo" }
                    }
                ]
            }
        })
        .to_string();

        let clean_report = db.audit_post_wcag_links_seo(&AuditRequestPayload {
            title: Some("Artisanal Metal Finishes for Architecture".to_string()),
            seo_description: Some(
                "Bespoke liquid metal finishes crafted by hand in our workshop for interior architects and luxury residences."
                    .to_string(),
            ),
            layout_json: Some(clean_layout),
            theme: Some("light".to_string()),
            ..Default::default()
        });
        assert!(
            clean_report
                .issues
                .iter()
                .all(|i| !i.id.starts_with("media-")),
            "expected zero media issues after Media Library import, got: {:?}",
            clean_report.issues
        );
    }
}
